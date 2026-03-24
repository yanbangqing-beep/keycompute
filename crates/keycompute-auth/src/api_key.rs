//! Produce AI Key 验证
//!
//! 处理 Produce AI Key（用户访问系统的 API Key）的验证和解析。

use keycompute_db::{ProduceAiKey, User};
use keycompute_types::{KeyComputeError, Result};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::{AuthContext, Permission};

/// Produce AI Key 验证器
#[derive(Clone)]
pub struct ProduceAiKeyValidator {
    /// 数据库连接池（可选）
    pool: Option<Arc<PgPool>>,
}

impl std::fmt::Debug for ProduceAiKeyValidator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProduceAiKeyValidator")
            .field("pool", &self.pool.as_ref().map(|_| "PgPool"))
            .finish()
    }
}

impl ProduceAiKeyValidator {
    /// 创建新的 Produce AI Key 验证器（无数据库连接）
    pub fn new() -> Self {
        Self { pool: None }
    }

    /// 创建带数据库连接的验证器
    pub fn with_pool(pool: Arc<PgPool>) -> Self {
        Self { pool: Some(pool) }
    }

    /// 验证 Produce AI Key
    ///
    /// Produce AI Key 格式: `sk-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx`
    pub async fn validate(&self, key: &str) -> Result<AuthContext> {
        // 检查格式
        if !key.starts_with("sk-") {
            return Err(KeyComputeError::AuthError("Invalid API key format".into()));
        }

        // 计算 key 的 hash
        let key_hash = Self::hash_key(key);

        // 尝试从数据库验证
        if let Some(pool) = &self.pool {
            return self.validate_from_database(pool, &key_hash).await;
        }

        // 无数据库连接，使用回退逻辑
        self.validate_fallback(&key_hash).await
    }

    /// 从数据库验证 Produce AI Key
    async fn validate_from_database(&self, pool: &PgPool, key_hash: &str) -> Result<AuthContext> {
        // 查询 Produce AI Key
        let produce_ai_key = ProduceAiKey::find_by_hash(pool, key_hash)
            .await
            .map_err(|e| {
                KeyComputeError::DatabaseError(format!("Failed to query API key: {}", e))
            })?;

        let Some(produce_ai_key) = produce_ai_key else {
            tracing::warn!(key_hash = %key_hash, "Produce AI key not found");
            return Err(KeyComputeError::AuthError("Invalid API key".into()));
        };

        // 检查是否有效
        if !produce_ai_key.is_valid() {
            tracing::warn!(
                produce_ai_key_id = %produce_ai_key.id,
                revoked = produce_ai_key.revoked,
                "Produce AI key is not valid"
            );
            return Err(KeyComputeError::AuthError(
                "API key is revoked or expired".into(),
            ));
        }

        // 查询用户信息
        let user = User::find_by_id(pool, produce_ai_key.user_id)
            .await
            .map_err(|e| KeyComputeError::DatabaseError(format!("Failed to query user: {}", e)))?;

        let Some(user) = user else {
            tracing::warn!(user_id = %produce_ai_key.user_id, "User not found");
            return Err(KeyComputeError::AuthError("User not found".into()));
        };

        // 验证用户租户 ID 与 Produce AI Key 租户 ID 一致
        if user.tenant_id != produce_ai_key.tenant_id {
            tracing::warn!(
                user_id = %user.id,
                user_tenant_id = %user.tenant_id,
                produce_ai_key_tenant_id = %produce_ai_key.tenant_id,
                "User tenant does not match Produce AI key tenant"
            );
            return Err(KeyComputeError::AuthError("User tenant mismatch".into()));
        }

        // 查询租户信息并验证状态
        use keycompute_db::Tenant;
        let tenant = Tenant::find_by_id(pool, user.tenant_id)
            .await
            .map_err(|e| {
                KeyComputeError::DatabaseError(format!("Failed to query tenant: {}", e))
            })?;

        let Some(tenant) = tenant else {
            tracing::warn!(tenant_id = %user.tenant_id, "Tenant not found");
            return Err(KeyComputeError::AuthError("Tenant not found".into()));
        };

        // 检查租户状态
        if !tenant.is_active() {
            tracing::warn!(
                tenant_id = %tenant.id,
                status = %tenant.status,
                "Tenant is not active"
            );
            return Err(KeyComputeError::AuthError(format!(
                "Tenant is not active: {}",
                tenant.status
            )));
        }

        // 更新最后使用时间
        let _ = produce_ai_key.update_last_used(pool).await;

        tracing::info!(
            user_id = %user.id,
            tenant_id = %user.tenant_id,
            produce_ai_key_id = %produce_ai_key.id,
            role = %user.role,
            "Produce AI key validated successfully"
        );

        // 构建权限列表
        let permissions = match user.role.as_str() {
            "admin" | "system" => vec![
                Permission::UseApi,
                Permission::ManageUsers,
                Permission::ManageApiKeys,
                Permission::ViewBilling,
                Permission::ManageBilling,
            ],
            "tenant_admin" => vec![
                Permission::UseApi,
                Permission::ViewUsage,
                Permission::ManageApiKeys,
                Permission::ManageUsers,
                Permission::ManageTenant,
                Permission::ViewBilling,
            ],
            "user" => vec![Permission::UseApi, Permission::ViewBilling],
            _ => vec![Permission::UseApi],
        };

        Ok(AuthContext {
            user_id: user.id,
            tenant_id: user.tenant_id,
            produce_ai_key_id: produce_ai_key.id,
            role: user.role,
            permissions,
            user_info: None,
            tenant_info: None,
        })
    }

    /// 回退验证（无数据库时使用）
    async fn validate_fallback(&self, key_hash: &str) -> Result<AuthContext> {
        tracing::debug!(key_hash = %key_hash, "Validating Produce AI key (fallback mode)");

        // 模拟验证成功
        let user_id = Uuid::new_v4();
        let tenant_id = Uuid::new_v4();
        let produce_ai_key_id = Uuid::new_v4();

        Ok(AuthContext {
            user_id,
            tenant_id,
            produce_ai_key_id,
            role: "user".to_string(),
            permissions: vec![Permission::UseApi],
            user_info: None,
            tenant_info: None,
        })
    }

    /// 计算 Produce AI Key 的 SHA256 hash
    pub fn hash_key(key: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// 生成新的 Produce AI Key
    pub fn generate_key() -> String {
        let uuid = Uuid::new_v4();
        format!("sk-{}", uuid.to_string().replace("-", ""))
    }
}

impl Default for ProduceAiKeyValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Produce AI Key 认证 trait
#[async_trait::async_trait]
pub trait ProduceAiKeyAuth: Send + Sync {
    /// 验证 Produce AI Key
    async fn authenticate(&self, key: &str) -> Result<AuthContext>;
}

#[async_trait::async_trait]
impl ProduceAiKeyAuth for ProduceAiKeyValidator {
    async fn authenticate(&self, key: &str) -> Result<AuthContext> {
        self.validate(key).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_key() {
        let key = ProduceAiKeyValidator::generate_key();
        assert!(key.starts_with("sk-"));
        assert_eq!(key.len(), 35); // "sk-" + 32 个字符
    }

    #[test]
    fn test_hash_key() {
        let key = "sk-test123";
        let hash1 = ProduceAiKeyValidator::hash_key(key);
        let hash2 = ProduceAiKeyValidator::hash_key(key);
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64); // SHA256 hex 长度
    }

    #[tokio::test]
    async fn test_validate_invalid_format() {
        let validator = ProduceAiKeyValidator::new();
        let result = validator.validate("invalid-key").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_valid_format() {
        let validator = ProduceAiKeyValidator::new();
        let key = ProduceAiKeyValidator::generate_key();
        let result = validator.validate(&key).await;
        assert!(result.is_ok());

        let ctx = result.unwrap();
        assert!(!ctx.is_admin());
        assert!(ctx.has_permission(&Permission::UseApi));
    }
}
