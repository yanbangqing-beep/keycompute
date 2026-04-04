//! API Key 管理模块
//!
//! 处理用户 API Key 的创建、查询和删除

use crate::client::ApiClient;
use crate::error::Result;
use serde::{Deserialize, Serialize};

pub use super::common::MessageResponse;

/// API Key API 客户端
#[derive(Debug, Clone)]
pub struct ApiKeyApi {
    client: ApiClient,
}

impl ApiKeyApi {
    /// 创建新的 API Key API 客户端
    pub fn new(client: &ApiClient) -> Self {
        Self {
            client: client.clone(),
        }
    }

    /// 获取我的 API Keys 列表
    ///
    /// # 参数
    /// - `include_revoked`: 是否包含已撤销的 Key（默认 false）
    pub async fn list_my_api_keys(
        &self,
        include_revoked: bool,
        token: &str,
    ) -> Result<Vec<ApiKeyInfo>> {
        let path = if include_revoked {
            "/api/v1/keys?include_revoked=true"
        } else {
            "/api/v1/keys"
        };
        self.client.get_json(path, Some(token)).await
    }

    /// 创建新的 API Key
    pub async fn create_api_key(
        &self,
        req: &CreateApiKeyRequest,
        token: &str,
    ) -> Result<CreateApiKeyResponse> {
        self.client
            .post_json("/api/v1/keys", req, Some(token))
            .await
    }

    /// 删除 API Key
    pub async fn delete_api_key(&self, id: &str, token: &str) -> Result<MessageResponse> {
        self.client
            .delete_json(&format!("/api/v1/keys/{}", id), Some(token))
            .await
    }
}

/// API Key 信息
#[derive(Debug, Clone, Deserialize)]
pub struct ApiKeyInfo {
    pub id: String,
    pub name: String,
    pub key_preview: String,
    /// 是否活跃（后端返回 is_active，前端转换为 !revoked）
    #[serde(rename = "is_active")]
    pub is_active: bool,
    pub created_at: String,
    pub last_used_at: Option<String>,
    pub expires_at: Option<String>,
}

impl ApiKeyInfo {
    /// 返回是否已撤销（与 is_active 相反）
    pub fn revoked(&self) -> bool {
        !self.is_active
    }
}

/// 创建 API Key 请求
#[derive(Debug, Clone, Serialize)]
pub struct CreateApiKeyRequest {
    pub name: String,
    pub expires_at: Option<String>,
}

impl CreateApiKeyRequest {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            expires_at: None,
        }
    }

    pub fn with_expires_at(mut self, expires_at: impl Into<String>) -> Self {
        self.expires_at = Some(expires_at.into());
        self
    }
}

/// 创建 API Key 响应（包含完整 key，仅创建时返回一次）
#[derive(Debug, Clone, Deserialize)]
pub struct CreateApiKeyResponse {
    /// 是否成功
    pub success: bool,
    /// 消息
    pub message: Option<String>,
    /// API Key ID（后端字段名为 key_id）
    #[serde(rename = "key_id")]
    pub id: String,
    /// API Key 名称
    pub name: String,
    /// 完整的 API Key（后端字段名为 key）
    #[serde(rename = "key")]
    pub api_key: String,
    /// 过期时间
    pub expires_at: Option<String>,
    /// 创建时间
    pub created_at: String,
    /// 是否永不过期
    pub never_expires: Option<bool>,
}
