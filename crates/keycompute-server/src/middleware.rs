//! 中间件
//!
//! 自定义中间件：认证、限流、可观测性等

use crate::state::AppState;
use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use keycompute_ratelimit::RateLimitKey;
use std::time::Instant;
use tracing::{error, info};

/// 请求日志中间件
pub async fn request_logger(req: Request, next: Next) -> Response {
    let start = Instant::now();
    let method = req.method().clone();
    let uri = req.uri().clone();
    
    // 提前克隆 request_id，避免借用冲突
    let request_id = req
        .headers()
        .get("X-Request-ID")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown")
        .to_string();

    info!(
        request_id = %request_id,
        method = %method,
        uri = %uri,
        "Request started"
    );

    let response = next.run(req).await;

    let duration = start.elapsed();
    let status = response.status();

    info!(
        request_id = %request_id,
        method = %method,
        uri = %uri,
        status = %status.as_u16(),
        duration_ms = %duration.as_millis(),
        "Request completed"
    );

    response
}

/// CORS 中间件配置
pub fn cors_layer() -> tower_http::cors::CorsLayer {
    tower_http::cors::CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods(tower_http::cors::Any)
        .allow_headers(tower_http::cors::Any)
}

/// 追踪 ID 注入中间件
pub async fn trace_id_middleware(mut req: Request, next: Next) -> Response {
    // 如果没有 X-Request-ID，生成一个
    if !req.headers().contains_key("X-Request-ID") {
        let request_id = uuid::Uuid::new_v4().to_string();
        req.headers_mut().insert(
            "X-Request-ID",
            request_id.parse().unwrap(),
        );
    }
    next.run(req).await
}

/// 限流中间件
///
/// 基于用户/租户/API Key 进行请求限流
/// 注意：此中间件应在认证中间件之后运行，以获取真实的认证信息
pub async fn rate_limit_middleware(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Response {
    // 从请求头中提取认证信息
    let headers = req.headers();
    let auth_header = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok());

    // 如果没有认证头，直接放行（由认证中间件处理）
    let Some(auth_header) = auth_header else {
        return next.run(req).await;
    };

    // 解析 Bearer token
    let Some(token) = auth_header.strip_prefix("Bearer ") else {
        return next.run(req).await;
    };

    // 使用 AuthService 验证 token 获取真实的用户信息
    let rate_key = match state.auth.verify_api_key(token).await {
        Ok(auth_context) => {
            // 使用真实的 user_id, tenant_id, api_key_id 创建限流键
            RateLimitKey::new(
                auth_context.tenant_id,
                auth_context.user_id,
                auth_context.api_key_id,
            )
        }
        Err(_) => {
            // 认证失败，直接放行（由认证层处理错误）
            return next.run(req).await;
        }
    };

    // 检查限流
    match state.rate_limiter.check_and_record(&rate_key).await {
        Ok(()) => {
            // 限流检查通过，继续处理请求
            next.run(req).await
        }
        Err(keycompute_types::KeyComputeError::RateLimitExceeded) => {
            // 触发限流
            info!(
                "Rate limit exceeded for tenant: {}, user: {}",
                rate_key.tenant_id, rate_key.user_id
            );
            (
                StatusCode::TOO_MANY_REQUESTS,
                serde_json::json!({
                    "error": {
                        "message": "Rate limit exceeded. Please try again later.",
                        "type": "rate_limit_exceeded",
                        "code": "rate_limit_exceeded"
                    }
                })
                .to_string(),
            )
            .into_response()
        }
        Err(e) => {
            // 限流检查出错，记录错误但放行（避免阻塞正常请求）
            error!("Rate limit check error: {}", e);
            next.run(req).await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cors_layer() {
        let cors = cors_layer();
        // 确保可以创建 CORS 层
        let _ = cors;
    }
}
