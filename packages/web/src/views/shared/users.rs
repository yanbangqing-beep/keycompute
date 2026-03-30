use client_api::{
    AdminApi,
    api::admin::{UserDetail, UserQueryParams},
};
use dioxus::prelude::*;
use ui::{Badge, BadgeVariant, Button, ButtonSize, ButtonVariant, Pagination, Table, TableHead};

use crate::services::api_client::{get_client, with_auto_refresh};
use crate::stores::auth_store::AuthStore;
use crate::stores::user_store::UserStore;
use crate::utils::time::format_time;

const PAGE_SIZE: usize = 20;

#[component]
pub fn Users() -> Element {
    let user_store = use_context::<UserStore>();
    let is_admin = user_store
        .info
        .read()
        .as_ref()
        .map(|u| u.is_admin())
        .unwrap_or(false);

    if is_admin {
        rsx! { AdminUsersView {} }
    } else {
        rsx! { UserSelfView {} }
    }
}

// ── Admin 视图 ────────────────────────────────────────────────────────

#[component]
fn AdminUsersView() -> Element {
    let auth_store = use_context::<AuthStore>();
    let mut search = use_signal(String::new);
    let mut page = use_signal(|| 1u32);

    let users = use_resource(move || async move {
        with_auto_refresh(auth_store, |token| async move {
            let client = get_client();
            let params = UserQueryParams::new().with_limit(50);
            AdminApi::new(&client)
                .list_all_users(Some(&params), &token)
                .await
        })
        .await
    });

    let filtered_users = move || -> Vec<UserDetail> {
        let q = search().to_lowercase();
        match users() {
            Some(Ok(ref list)) => list
                .iter()
                .filter(|u| {
                    q.is_empty()
                        || u.email.to_lowercase().contains(&q)
                        || u.name.as_deref().unwrap_or("").to_lowercase().contains(&q)
                })
                .cloned()
                .collect::<Vec<UserDetail>>(),
            _ => vec![],
        }
    };

    let total_pages = move || {
        let len = filtered_users().len();
        len.div_ceil(PAGE_SIZE).max(1) as u32
    };

    let paged_users = move || {
        let p = page() as usize;
        let all = filtered_users();
        let start = (p - 1) * PAGE_SIZE;
        all.into_iter()
            .skip(start)
            .take(PAGE_SIZE)
            .collect::<Vec<_>>()
    };

    rsx! {
        div { class: "page-header",
            h1 { class: "page-title", "用户管理" }
            p { class: "page-description", "查看和管理平台所有注册用户" }
        }

        div { class: "toolbar",
            div { class: "toolbar-left",
                div { class: "input-wrapper",
                    input {
                        class: "input-field",
                        r#type: "search",
                        placeholder: "搜索邮笱或用户名...",
                        value: "{search}",
                        oninput: move |e| {
                                    *search.write() = e.value();
                                    page.set(1);
                                },
                    }
                }
            }
            div { class: "toolbar-right",
                Button {
                    variant: ButtonVariant::Secondary,
                    size: ButtonSize::Small,
                    "导出"
                }
            }
        }

        div { class: "card",
            {
                let (is_empty, empty_text) = match users() {
                    None => (true, "加载中..."),
                    Some(Err(_)) => (true, "加载失败"),
                    Some(Ok(_)) if filtered_users().is_empty() => (true, "暂无用户数据"),
                    _ => (false, ""),
                };
                rsx! {
                    Table {
                        empty: is_empty,
                        empty_text: empty_text.to_string(),
                        col_count: 4,
                        thead {
                            tr {
                                TableHead { "用户" }
                                TableHead { "角色" }
                                TableHead { "租户" }
                                TableHead { "注册时间" }
                            }
                        }
                        tbody {
                            for u in paged_users().iter() {
                                tr {
                                    td {
                                        div { class: "user-cell",
                                            span { class: "user-name",
                                                { u.name.clone().unwrap_or_else(|| u.email.clone()) }
                                            }
                                            span { class: "user-email text-secondary", "{u.email}" }
                                        }
                                    }
                                    td {
                                        Badge { variant: BadgeVariant::Info, "{u.role}" }
                                    }
                                    td { "{u.tenant_id}" }
                                    td { { format_time(&u.created_at) } }
                                }
                            }
                        }
                    }
                }
            }
        }

        div { class: "pagination",
            span { class: "pagination-info",
                "共 { filtered_users().len() } 条"
            }
            Pagination {
                current: page(),
                total_pages: total_pages(),
                on_page_change: move |p| page.set(p),
            }
        }
    }
}

// ── 普通用户视图 ──────────────────────────────────────────────────────

#[component]
fn UserSelfView() -> Element {
    let user_store = use_context::<UserStore>();
    let user_info = user_store.info.read();

    let display_name = user_info
        .as_ref()
        .map(|u| u.display_name().to_string())
        .unwrap_or_default();
    let email = user_info
        .as_ref()
        .map(|u| u.email.clone())
        .unwrap_or_default();
    let role = user_info
        .as_ref()
        .map(|u| u.role.clone())
        .unwrap_or_default();

    rsx! {
        div { class: "page-header",
            h1 { class: "page-title", "我的账户" }
            p { class: "page-description", "查看和管理您的个人账户信息" }
        }

        div { class: "card",
            div { class: "card-header",
                h3 { class: "card-title", "账户信息" }
                Button {
                    variant: ButtonVariant::Secondary,
                    size: ButtonSize::Small,
                    onclick: move |_| {},
                    "编辑资料"
                }
            }
            div { class: "card-body",
                div { class: "info-grid",
                    div { class: "info-item",
                        span { class: "info-label", "显示名称" }
                        span { class: "info-value", "{display_name}" }
                    }
                    div { class: "info-item",
                        span { class: "info-label", "邮笱" }
                        span { class: "info-value", "{email}" }
                    }
                    div { class: "info-item",
                        span { class: "info-label", "角色" }
                        Badge { variant: BadgeVariant::Info, "{role}" }
                    }
                }
            }
        }
    }
}
