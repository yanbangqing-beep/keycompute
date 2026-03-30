//! 租户管理模块
//!
//! 处理租户列表查询、创建、更新、删除（Admin）

use crate::client::ApiClient;
use crate::error::Result;
use serde::{Deserialize, Serialize};

/// 租户 API 客户端
#[derive(Debug, Clone)]
pub struct TenantApi {
    client: ApiClient,
}

impl TenantApi {
    /// 创建新的租户 API 客户端
    pub fn new(client: &ApiClient) -> Self {
        Self {
            client: client.clone(),
        }
    }

    /// 获取租户列表（Admin）
    pub async fn list_tenants(
        &self,
        params: Option<&TenantQueryParams>,
        token: &str,
    ) -> Result<Vec<TenantInfo>> {
        let path = if let Some(p) = params {
            format!("/api/v1/tenants?{}", p.to_query_string())
        } else {
            "/api/v1/tenants".to_string()
        };
        self.client.get_json(&path, Some(token)).await
    }

    /// 创建租户（Admin）
    pub async fn create_tenant(
        &self,
        req: &CreateTenantRequest,
        token: &str,
    ) -> Result<TenantInfo> {
        self.client
            .post_json("/api/v1/tenants", req, Some(token))
            .await
    }

    /// 更新租户信息（Admin）
    pub async fn update_tenant(
        &self,
        tenant_id: &str,
        req: &UpdateTenantRequest,
        token: &str,
    ) -> Result<TenantInfo> {
        let path = format!("/api/v1/tenants/{}", tenant_id);
        self.client.put_json(&path, req, Some(token)).await
    }

    /// 删除租户（Admin）
    pub async fn delete_tenant(
        &self,
        tenant_id: &str,
        token: &str,
    ) -> Result<crate::api::common::MessageResponse> {
        let path = format!("/api/v1/tenants/{}", tenant_id);
        self.client.delete_json(&path, Some(token)).await
    }
}

/// 创建租户请求
#[derive(Debug, Clone, Serialize)]
pub struct CreateTenantRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

impl CreateTenantRequest {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: None,
        }
    }

    pub fn with_status(mut self, status: impl Into<String>) -> Self {
        self.status = Some(status.into());
        self
    }
}

/// 更新租户请求
#[derive(Debug, Clone, Serialize, Default)]
pub struct UpdateTenantRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

impl UpdateTenantRequest {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn with_status(mut self, status: impl Into<String>) -> Self {
        self.status = Some(status.into());
        self
    }
}

/// 租户查询参数
#[derive(Debug, Clone, Serialize, Default)]
pub struct TenantQueryParams {
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

impl TenantQueryParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_limit(mut self, limit: i32) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn with_offset(mut self, offset: i32) -> Self {
        self.offset = Some(offset);
        self
    }

    pub fn to_query_string(&self) -> String {
        let mut params = Vec::new();
        if let Some(limit) = self.limit {
            params.push(format!("limit={}", limit));
        }
        if let Some(offset) = self.offset {
            params.push(format!("offset={}", offset));
        }
        params.join("&")
    }
}

/// 租户信息
#[derive(Debug, Clone, Deserialize)]
pub struct TenantInfo {
    pub id: String,
    pub name: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}
