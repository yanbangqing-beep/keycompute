use dioxus::prelude::*;
use ui::{Badge, BadgeVariant, Table, TableHead};

use crate::services::pricing_service;
use crate::stores::auth_store::AuthStore;
use crate::stores::user_store::UserStore;

/// 定价管理页面
///
/// - 普通用户：只读查看定价策略列表
/// - Admin：完整 CRUD（创建/编辑/删除/设置默认）
#[component]
pub fn Pricing() -> Element {
    let user_store = use_context::<UserStore>();
    let auth_store = use_context::<AuthStore>();
    let is_admin = user_store
        .info
        .read()
        .as_ref()
        .map(|u| u.is_admin())
        .unwrap_or(false);

    let pricing_list = use_resource(move || async move {
        let token = auth_store.token().unwrap_or_default();
        pricing_service::list(&token).await
    });

    rsx! {
        div { class: "page-header",
            h1 { class: "page-title", "定价管理" }
            p { class: "page-description",
                if is_admin { "管理平台定价策略，设置模型调用费率" }
                else { "查看当前平台可用的定价策略" }
            }
        }

        {
            let (is_empty, empty_text) = match pricing_list() {
                None => (true, "加载中..."),
                Some(Err(_)) => (true, "加载失败"),
                Some(Ok(ref l)) if l.is_empty() => (true, "暂无定价策略"),
                _ => (false, ""),
            };
            rsx! {
                Table {
                    empty: is_empty,
                    empty_text: empty_text.to_string(),
                    col_count: 6,
                    thead {
                        tr {
                            TableHead { "模型" }
                            TableHead { "输入单价（/1K tokens）" }
                            TableHead { "输出单价（/1K tokens）" }
                            TableHead { "货币" }
                            TableHead { "默认" }
                            TableHead { "创建时间" }
                        }
                    }
                    tbody {
                        if let Some(Ok(ref list)) = pricing_list() {
                            for p in list.iter() {
                                tr {
                                    td { code { "{p.model}" } }
                                    td { "{p.input_price:.6}" }
                                    td { "{p.output_price:.6}" }
                                    td { "{p.currency}" }
                                    td {
                                        if p.is_default {
                                            Badge { variant: BadgeVariant::Success, "默认" }
                                        } else {
                                            Badge { variant: BadgeVariant::Neutral, "非默认" }
                                        }
                                    }
                                    td { "{p.created_at}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
