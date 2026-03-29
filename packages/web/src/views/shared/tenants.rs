use client_api::api::tenant::TenantInfo;
use dioxus::prelude::*;
use ui::{Badge, BadgeVariant, Table, TableHead};

use crate::services::tenant_service;
use crate::stores::auth_store::AuthStore;
use crate::stores::user_store::UserStore;
use crate::views::shared::accounts::NoPermissionView;

/// 租户管理页面（仅 Admin 可访问）
///
/// - 普通用户：无权限提示
/// - Admin：查看全平台租户列表（调用 TenantApi）
#[component]
pub fn Tenants() -> Element {
    let user_store = use_context::<UserStore>();
    let auth_store = use_context::<AuthStore>();
    let is_admin = user_store
        .info
        .read()
        .as_ref()
        .map(|u| u.is_admin())
        .unwrap_or(false);

    if !is_admin {
        return rsx! { NoPermissionView { resource: "租户管理" } };
    }

    let mut search = use_signal(String::new);

    let tenants = use_resource(move || async move {
        let token = auth_store.token().unwrap_or_default();
        tenant_service::list(None, &token).await
    });

    let filtered = move || -> Vec<TenantInfo> {
        let q = search().to_lowercase();
        match tenants() {
            Some(Ok(ref list)) => list
                .iter()
                .filter(|t| {
                    q.is_empty()
                        || t.id.to_lowercase().contains(&q)
                        || t.name.to_lowercase().contains(&q)
                })
                .cloned()
                .collect::<Vec<_>>(),
            _ => vec![],
        }
    };

    rsx! {
        div { class: "page-header",
            h1 { class: "page-title", "租户管理" }
            p { class: "page-description", "查看和管理平台所有租户信息" }
        }

        div { class: "toolbar",
            div { class: "toolbar-left",
                div { class: "input-wrapper",
                    input {
                        class: "input-field",
                        r#type: "search",
                        placeholder: "搜索租户名称或 ID...",
                        value: "{search}",
                        oninput: move |e| *search.write() = e.value(),
                    }
                }
            }
        }

        {
            let (is_empty, empty_text) = match tenants() {
                None => (true, "加载中..."),
                Some(Err(_)) => (true, "加载失败"),
                Some(Ok(_)) if filtered().is_empty() => (true, "暂无租户数据"),
                _ => (false, ""),
            };
            rsx! {
                Table {
                    empty: is_empty,
                    empty_text: empty_text.to_string(),
                    col_count: 4,
                    thead {
                        tr {
                            TableHead { "租户 ID" }
                            TableHead { "名称" }
                            TableHead { "状态" }
                            TableHead { "创建时间" }
                        }
                    }
                    tbody {
                        for t in filtered().iter() {
                            tr {
                                td { code { "{t.id}" } }
                                td { "{t.name}" }
                                td {
                                    if t.status == "active" {
                                        Badge { variant: BadgeVariant::Success, "活跃" }
                                    } else {
                                        Badge { variant: BadgeVariant::Neutral, "{t.status}" }
                                    }
                                }
                                td { "{t.created_at}" }
                            }
                        }
                    }
                }
            }
        }

        div { class: "pagination",
            span { class: "pagination-info",
                "共 { filtered().len() } 条"
            }
        }
    }
}
