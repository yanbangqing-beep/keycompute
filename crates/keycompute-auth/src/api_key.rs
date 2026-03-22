//! API Key 验证
//!
//! 处理 API Key 的验证和解析。

use keycompute_types::{KeyComputeError, Result};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{AuthContext, Permission};

/// API Key 验证器
#[derive(Debug, Clone)]
pub struct ApiKeyValidator {
    /// 密钥（实际应该从数据库或配置加载）
    secret: String,
}

impl ApiKeyValidator {
    /// 创建新的 API Key 验证器
    pub fn new(secret: impl Into<String>) -> Self {
        Self {
            secret: secret.into(),
        }
    }

    /// 验证 API Key
    ///
    /// API Key 格式: `sk-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx`
    pub async fn validate(&self, key: &str) -> Result<AuthContext> {
        // 检查格式
        if !key.starts_with("sk-") {
            return Err(KeyComputeError::AuthError(
                "Invalid API key format".into(),
            ));
        }

        // 计算 key 的 hash
        let key_hash = Self::hash_key(key);

        // TODO: 从数据库查询 API Key
        // 这里简化处理，直接返回模拟数据
        tracing::debug!(key_hash = %key_hash, "Validating API key");

        // 模拟验证成功
        // 实际应该查询数据库验证 key_hash 并获取用户信息
        let user_id = Uuid::new_v4();
        let tenant_id = Uuid::new_v4();
        let api_key_id = Uuid::new_v4();

        Ok(AuthContext {
            user_id,
            tenant_id,
            api_key_id,
            role: "user".to_string(),
            permissions: vec![Permission::UseApi],
        })
    }

    /// 计算 API Key 的 SHA256 hash
    pub fn hash_key(key: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// 生成新的 API Key
    pub fn generate_key() -> String {
        let uuid = Uuid::new_v4();
        format!("sk-{}", uuid.to_string().replace("-", ""))
    }
}

impl Default for ApiKeyValidator {
    fn default() -> Self {
        Self::new("default-secret")
    }
}

/// API Key 认证 trait
#[async_trait::async_trait]
pub trait ApiKeyAuth: Send + Sync {
    /// 验证 API Key
    async fn authenticate(&self, key: &str) -> Result<AuthContext>;
}

#[async_trait::async_trait]
impl ApiKeyAuth for ApiKeyValidator {
    async fn authenticate(&self, key: &str) -> Result<AuthContext> {
        self.validate(key).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_key() {
        let key = ApiKeyValidator::generate_key();
        assert!(key.starts_with("sk-"));
        assert_eq!(key.len(), 35); // "sk-" + 32 个字符
    }

    #[test]
    fn test_hash_key() {
        let key = "sk-test123";
        let hash1 = ApiKeyValidator::hash_key(key);
        let hash2 = ApiKeyValidator::hash_key(key);
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64); // SHA256 hex 长度
    }

    #[tokio::test]
    async fn test_validate_invalid_format() {
        let validator = ApiKeyValidator::new("secret");
        let result = validator.validate("invalid-key").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_valid_format() {
        let validator = ApiKeyValidator::new("secret");
        let key = ApiKeyValidator::generate_key();
        let result = validator.validate(&key).await;
        assert!(result.is_ok());

        let ctx = result.unwrap();
        assert!(!ctx.is_admin());
        assert!(ctx.has_permission(&Permission::UseApi));
    }
}
