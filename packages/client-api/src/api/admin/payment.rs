//! 支付订单管理相关类型（Admin 视角）

use serde::{Deserialize, Serialize};

use crate::api::common::encode_query_value;

/// 支付订单查询参数（Admin）
#[derive(Debug, Clone, Serialize, Default)]
pub struct PaymentQueryParams {
    pub status: Option<String>,
    pub user_id: Option<String>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

impl PaymentQueryParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_status(mut self, status: impl Into<String>) -> Self {
        self.status = Some(status.into());
        self
    }

    pub fn with_user_id(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
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
        if let Some(ref status) = self.status {
            params.push(format!("status={}", encode_query_value(status)));
        }
        if let Some(ref user_id) = self.user_id {
            params.push(format!("user_id={}", encode_query_value(user_id)));
        }
        if let Some(limit) = self.limit {
            params.push(format!("limit={}", limit));
        }
        if let Some(offset) = self.offset {
            params.push(format!("offset={}", offset));
        }
        params.join("&")
    }
}

/// 支付订单信息（Admin 视角，含 user_id）
#[derive(Debug, Clone, Deserialize)]
pub struct PaymentOrderInfo {
    pub id: String,
    pub tenant_id: Option<String>,
    pub user_id: String,
    pub out_trade_no: String,
    pub trade_no: Option<String>,
    /// 金额（字符串格式，如 "100.00"）
    pub amount: String,
    pub status: String,
    pub subject: Option<String>,
    pub created_at: String,
}
