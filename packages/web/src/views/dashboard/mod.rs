use dioxus::prelude::*;

use crate::services::{payment_service, usage_service};
use crate::stores::auth_store::AuthStore;
use crate::stores::user_store::UserStore;

#[component]
pub fn Dashboard() -> Element {
    let user_store = use_context::<UserStore>();
    let auth_store = use_context::<AuthStore>();

    let user_info = user_store.info.read();
    let greeting = if let Some(ref u) = *user_info {
        format!("你好，{}", u.name.as_deref().unwrap_or(&u.email))
    } else {
        "你好".to_string()
    };
    drop(user_info);

    // 拉取用量统计
    let usage_stats = use_resource(move || async move {
        let token = auth_store.token().unwrap_or_default();
        usage_service::stats(&token).await
    });

    // 拉取账户余额
    let balance = use_resource(move || async move {
        let token = auth_store.token().unwrap_or_default();
        payment_service::get_balance(&token).await
    });

    // 拉取 API Key 数量（利用展示活跃 Key 数）
    let api_keys = use_resource(move || async move {
        let token = auth_store.token().unwrap_or_default();
        crate::services::api_key_service::list(&token).await
    });

    let total_requests = match usage_stats() {
        Some(Ok(ref s)) => s.total_requests.to_string(),
        Some(Err(_)) => "加载失败".to_string(),
        None => "加载中...".to_string(),
    };
    let today_cost = match usage_stats() {
        Some(Ok(ref s)) => format!("¥{:.4}", s.total_cost),
        _ => "—".to_string(),
    };
    let balance_val = match balance() {
        Some(Ok(ref b)) => format!("¥{:.2}", b.balance),
        Some(Err(_)) => "加载失败".to_string(),
        None => "加载中...".to_string(),
    };
    let active_keys = match api_keys() {
        Some(Ok(ref keys)) => keys.iter().filter(|k| !k.revoked).count().to_string(),
        Some(Err(_)) => "—".to_string(),
        None => "—".to_string(),
    };

    rsx! {
        div {
            class: "page-container",
            div {
                class: "page-header",
                h1 { class: "page-title", "{greeting}" }
                p { class: "page-subtitle", "这是您的控制台概览" }
            }

            div {
                class: "stats-grid",
                StatCard {
                    title: "API 调用次数",
                    value: "{total_requests}",
                    label: "本周累计",
                    icon: "key",
                }
                StatCard {
                    title: "账户余额",
                    value: "{balance_val}",
                    label: "可用",
                    icon: "wallet",
                }
                StatCard {
                    title: "活跃 Key",
                    value: "{active_keys}",
                    label: "总计",
                    icon: "list",
                }
                StatCard {
                    title: "本周消耗",
                    value: "{today_cost}",
                    label: "已用",
                    icon: "chart",
                }
            }

            div {
                class: "section",
                h2 { class: "section-title", "快速入口" }
                div {
                    class: "quick-links",
                    QuickLink { href: "/api-keys", label: "管理 API Key" }
                    QuickLink { href: "/payments", label: "充値余额" }
                    QuickLink { href: "/user/profile", label: "账户设置" }
                }
            }
        }
    }
}

#[component]
fn StatCard(title: String, value: String, label: String, icon: String) -> Element {
    rsx! {
        div {
            class: "stat-card",
            div {
                class: "stat-icon stat-icon-{icon}",
            }
            div {
                class: "stat-body",
                p { class: "stat-title", "{title}" }
                p { class: "stat-value", "{value}" }
                p { class: "stat-label", "{label}" }
            }
        }
    }
}

#[component]
fn QuickLink(href: String, label: String) -> Element {
    rsx! {
        a {
            class: "quick-link-card",
            href: "{href}",
            "{label}"
        }
    }
}
