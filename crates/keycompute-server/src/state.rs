//! 应用状态
//!
//! AppState 定义（DB Pool, Redis, 各模块 Handle）

use keycompute_auth::{ApiKeyValidator, AuthService};
use std::sync::Arc;

/// 应用状态
#[derive(Debug, Clone)]
pub struct AppState {
    /// 认证服务
    pub auth: Arc<AuthService>,
    /// 限流服务
    pub rate_limiter: Arc<keycompute_ratelimit::RateLimitService>,
    // TODO: 添加其他模块服务
    // pub pricing: Arc<keycompute_pricing::PricingService>,
    // pub routing: Arc<keycompute_routing::RoutingEngine>,
    // pub runtime: Arc<keycompute_runtime::RuntimeManager>,
    // pub gateway: Arc<llm_gateway::GatewayExecutor>,
    // pub billing: Arc<keycompute_billing::BillingService>,
}

impl AppState {
    /// 创建新的应用状态
    pub fn new() -> Self {
        // 创建 API Key 验证器
        let api_key_validator = ApiKeyValidator::new("default-secret");
        let auth_service = AuthService::new(api_key_validator);

        Self {
            auth: Arc::new(auth_service),
            rate_limiter: Arc::new(keycompute_ratelimit::RateLimitService::default_memory()),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state_new() {
        let state = AppState::new();
        // 基础测试，确保可以创建
        let _ = state;
    }
}
