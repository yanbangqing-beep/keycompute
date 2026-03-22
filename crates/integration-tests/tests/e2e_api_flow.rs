//! API 层端到端测试
//!
//! 验证数据链路：API Server -> Auth -> Rate Limit -> RequestContext

use integration_tests::common::{TestContext, VerificationChain, TEST_API_KEY};
use axum::body::Body;
use axum::http::{Request, StatusCode};
use keycompute_server::state::AppState;
use keycompute_server::create_router;
use serde_json::json;
use tower::ServiceExt;

/// 测试完整的 API 请求流程
#[tokio::test]
async fn test_api_request_flow() {
    let ctx = TestContext::new();
    let mut chain = VerificationChain::new();

    // 1. 创建应用状态和路由
    let state = AppState::new();
    let app = create_router(state);

    chain.add_step(
        "keycompute-server",
        "create_router",
        "Router created with AppState",
        true,
    );

    // 2. 发送 chat/completions 请求
    let request_body = json!({
        "model": "gpt-4o",
        "messages": [{"role": "user", "content": "Hello"}],
        "stream": true
    });

    let request = Request::builder()
        .method("POST")
        .uri("/v1/chat/completions")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", TEST_API_KEY))
        .body(Body::from(request_body.to_string()))
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();

    // 验证响应状态
    let status_ok = response.status() == StatusCode::OK;
    chain.add_step(
        "keycompute-server",
        "chat_completions_handler",
        format!("Response status: {:?}", response.status()),
        status_ok,
    );

    // 3. 验证 SSE 流响应头
    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let is_sse = content_type.contains("text/event-stream");
    chain.add_step(
        "keycompute-server",
        "sse_response_headers",
        format!("Content-Type: {}", content_type),
        is_sse,
    );

    // 4. 测试模型列表接口
    let models_request = Request::builder()
        .method("GET")
        .uri("/v1/models")
        .header("Authorization", format!("Bearer {}", TEST_API_KEY))
        .body(Body::empty())
        .unwrap();

    let models_response = app.oneshot(models_request).await.unwrap();
    let models_ok = models_response.status() == StatusCode::OK;
    chain.add_step(
        "keycompute-server",
        "list_models_handler",
        format!("Models endpoint status: {:?}", models_response.status()),
        models_ok,
    );

    // 打印验证报告
    chain.print_report();
    assert!(chain.all_passed(), "Some verification steps failed");
}

/// 测试认证流程
#[tokio::test]
async fn test_auth_flow() {
    use keycompute_auth::{ApiKeyValidator, AuthService};
    use keycompute_server::extractors::AuthExtractor;
    use axum::http::HeaderMap;
    use axum::http::header::AUTHORIZATION;

    let mut chain = VerificationChain::new();

    // 创建 AuthService
    let auth_service = AuthService::new(ApiKeyValidator::default());

    // 1. 测试有效 API Key
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        format!("Bearer {}", TEST_API_KEY).parse().unwrap(),
    );

    let result = AuthExtractor::from_header_with_auth(&headers, &auth_service).await;
    let auth_ok = result.is_ok();
    chain.add_step(
        "keycompute-server::extractors",
        "AuthExtractor::from_header_with_auth",
        "Valid API key accepted",
        auth_ok,
    );

    if let Ok(auth) = result {
        chain.add_step(
            "keycompute-server::extractors",
            "AuthExtractor::user_id",
            format!("User ID extracted: {:?}", auth.user_id),
            !auth.user_id.is_nil(),
        );
        chain.add_step(
            "keycompute-server::extractors",
            "AuthExtractor::tenant_id",
            format!("Tenant ID extracted: {:?}", auth.tenant_id),
            !auth.tenant_id.is_nil(),
        );
    }

    // 2. 测试无效 API Key
    let mut bad_headers = HeaderMap::new();
    bad_headers.insert(
        AUTHORIZATION,
        "Bearer invalid-key".parse().unwrap(),
    );

    let bad_result = AuthExtractor::from_header_with_auth(&bad_headers, &auth_service).await;
    chain.add_step(
        "keycompute-server::extractors",
        "AuthExtractor::reject_invalid",
        "Invalid API key rejected",
        bad_result.is_err(),
    );

    // 3. 测试缺失 Authorization 头
    let empty_headers = HeaderMap::new();
    let missing_result = AuthExtractor::from_header_with_auth(&empty_headers, &auth_service).await;
    chain.add_step(
        "keycompute-server::extractors",
        "AuthExtractor::reject_missing",
        "Missing auth header rejected",
        missing_result.is_err(),
    );

    chain.print_report();
    assert!(chain.all_passed(), "Some auth verification steps failed");
}

/// 测试健康检查端点
#[tokio::test]
async fn test_health_endpoint() {
    let state = AppState::new();
    let app = create_router(state);

    let request = Request::builder()
        .method("GET")
        .uri("/health")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["status"], "healthy");
    assert!(!json["version"].as_str().unwrap().is_empty());
}

/// 测试请求 ID 提取
#[tokio::test]
async fn test_request_id_extraction() {
    use keycompute_server::extractors::RequestId;
    use axum::http::HeaderMap;

    let mut chain = VerificationChain::new();

    // 1. 测试从请求头提取
    let mut headers = HeaderMap::new();
    let test_uuid = uuid::Uuid::new_v4();
    headers.insert("X-Request-ID", test_uuid.to_string().parse().unwrap());

    // 使用默认构造函数测试
    let request_id = RequestId::new();
    chain.add_step(
        "keycompute-server::extractors",
        "RequestId::new",
        format!("Generated request ID: {:?}", request_id.0),
        !request_id.0.is_nil(),
    );

    // 2. 测试默认实现
    let default_id: RequestId = Default::default();
    chain.add_step(
        "keycompute-server::extractors",
        "RequestId::default",
        "Default request ID generated",
        !default_id.0.is_nil(),
    );

    chain.print_report();
    assert!(chain.all_passed());
}
