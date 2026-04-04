use client_api::error::Result;
use client_api::{
    ApiKeyApi,
    api::api_key::{ApiKeyInfo, CreateApiKeyRequest, CreateApiKeyResponse, MessageResponse},
};

use super::api_client::get_client;

pub async fn list(include_revoked: bool, token: &str) -> Result<Vec<ApiKeyInfo>> {
    let client = get_client();
    ApiKeyApi::new(&client)
        .list_my_api_keys(include_revoked, token)
        .await
}

pub async fn create(name: &str, token: &str) -> Result<CreateApiKeyResponse> {
    let client = get_client();
    ApiKeyApi::new(&client)
        .create_api_key(&CreateApiKeyRequest::new(name), token)
        .await
}

pub async fn delete(id: &str, token: &str) -> Result<MessageResponse> {
    let client = get_client();
    ApiKeyApi::new(&client).delete_api_key(id, token).await
}
