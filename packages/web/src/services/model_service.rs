//! 模型服务
//!
//! 获取系统支持的模型列表

use client_api::ClientConfig;
use client_api::error::Result;
use client_api::{OpenAiClient, api::openai::ModelListResponse};

use super::api_client::get_client;

/// 获取可用模型列表（无需认证）
pub async fn list_models() -> Result<ModelListResponse> {
    let client = get_client();
    let base_url = client.config().base_url.clone();
    let openai_client = OpenAiClient::new(ClientConfig::new(base_url))?;
    // 使用空字符串作为 API key，因为后端允许匿名访问 /v1/models
    openai_client.get_json("/v1/models", "").await
}
