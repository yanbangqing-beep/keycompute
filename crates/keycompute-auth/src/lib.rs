//! Auth & User Module
//!
//! API Key / JWT 解析，User / Tenant 加载。

pub mod api_key;
pub mod jwt;
pub mod permission;
pub mod user;

pub use api_key::{ApiKeyAuth, ApiKeyValidator};
pub use jwt::{JwtClaims, JwtValidator};
pub use permission::{Permission, PermissionChecker};
pub use user::{UserInfo, UserService};

use keycompute_types::{KeyComputeError, Result};
use uuid::Uuid;

/// 认证上下文
///
/// 包含用户认证后的所有信息
#[derive(Debug, Clone)]
pub struct AuthContext {
    /// 用户 ID
    pub user_id: Uuid,
    /// 租户 ID
    pub tenant_id: Uuid,
    /// API Key ID
    pub api_key_id: Uuid,
    /// 用户角色
    pub role: String,
    /// 权限列表
    pub permissions: Vec<Permission>,
}

impl AuthContext {
    /// 创建新的认证上下文
    pub fn new(
        user_id: Uuid,
        tenant_id: Uuid,
        api_key_id: Uuid,
        role: impl Into<String>,
    ) -> Self {
        Self {
            user_id,
            tenant_id,
            api_key_id,
            role: role.into(),
            permissions: Vec::new(),
        }
    }

    /// 检查是否有指定权限
    pub fn has_permission(&self, permission: &Permission) -> bool {
        self.permissions.contains(permission) || self.role == "admin"
    }

    /// 是否是管理员
    pub fn is_admin(&self) -> bool {
        self.role == "admin"
    }
}

/// 认证服务
///
/// 统一处理 API Key 和 JWT 认证
#[derive(Debug, Clone)]
pub struct AuthService {
    api_key_validator: ApiKeyValidator,
    jwt_validator: Option<JwtValidator>,
}

impl AuthService {
    /// 创建只使用 API Key 认证的 AuthService
    pub fn new(api_key_validator: ApiKeyValidator) -> Self {
        Self {
            api_key_validator,
            jwt_validator: None,
        }
    }

    /// 创建支持 JWT 的 AuthService
    pub fn with_jwt(mut self, jwt_validator: JwtValidator) -> Self {
        self.jwt_validator = Some(jwt_validator);
        self
    }

    /// 验证 API Key
    pub async fn verify_api_key(&self, key: &str) -> Result<AuthContext> {
        self.api_key_validator.validate(key).await
    }

    /// 验证 JWT Token（如果配置了 JWT）
    pub fn verify_jwt(&self, token: &str) -> Result<AuthContext> {
        match &self.jwt_validator {
            Some(validator) => validator.validate(token),
            None => Err(KeyComputeError::AuthError(
                "JWT validation not configured".into(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_context() {
        let ctx = AuthContext::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            "user",
        );

        assert!(!ctx.is_admin());
        assert!(!ctx.has_permission(&Permission::ManageUsers));
    }

    #[test]
    fn test_auth_context_admin() {
        let ctx = AuthContext::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            "admin",
        );

        assert!(ctx.is_admin());
        assert!(ctx.has_permission(&Permission::ManageUsers));
    }
}
