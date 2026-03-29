use dioxus::prelude::*;
use ui::{Badge, BadgeVariant, Table, TableHead};

use crate::services::payment_service;
use crate::stores::auth_store::AuthStore;

#[component]
pub fn PaymentsOverview() -> Element {
    let auth_store = use_context::<AuthStore>();

    let balance = use_resource(move || async move {
        let token = auth_store.token().unwrap_or_default();
        payment_service::get_balance(&token).await
    });

    let orders = use_resource(move || async move {
        let token = auth_store.token().unwrap_or_default();
        payment_service::list_orders(None, &token).await
    });

    rsx! {
        div {
            class: "page-container",
            div {
                class: "page-header",
                h1 { class: "page-title", "支付与账单" }
            }

            // 余额卡片
            div {
                class: "stats-grid",
                div {
                    class: "stat-card",
                    p { class: "stat-title", "账户余额" }
                    match balance() {
                        None => rsx! { p { class: "stat-value", "加载中..." } },
                        Some(Err(e)) => rsx! { p { class: "stat-value text-error", "错误: {e}" } },
                        Some(Ok(b)) => rsx! {
                            p { class: "stat-value", "¥ {b.balance:.2}" }
                            p { class: "stat-label", "{b.currency}" }
                        },
                    }
                }
                div {
                    class: "stat-card",
                    p { class: "stat-title", "冻结金额" }
                    match balance() {
                        Some(Ok(b)) => rsx! { p { class: "stat-value", "¥ {b.frozen_balance:.2}" } },
                        _ => rsx! { p { class: "stat-value", "—" } },
                    }
                }
            }

            // 充值记录
            div {
                class: "section",
                div {
                    class: "section-header",
                    h2 { class: "section-title", "充值记录" }
                    a { class: "btn btn-primary", href: "/payments/recharge", "立即充値" }
                }
                match orders() {
                    None => rsx! { div { class: "loading-state", "加载中..." } },
                    Some(Err(e)) => rsx! { div { class: "alert alert-error", "加载失败：{e}" } },
                    Some(Ok(list)) => {
                        if list.is_empty() {
                            rsx! { div { class: "empty-state", p { "暂无充値记录" } } }
                        } else {
                            rsx! {
                                Table {
                                    col_count: 4,
                                    thead {
                                        tr {
                                            TableHead { "订单号" }
                                            TableHead { "金额" }
                                            TableHead { "状态" }
                                            TableHead { "时间" }
                                        }
                                    }
                                    tbody {
                                        for order in list.iter() {
                                            tr {
                                                key: "{order.id}",
                                                td { code { "{order.out_trade_no}" } }
                                                td { "¥ {order.amount:.2}" }
                                                td {
                                                    Badge {
                                                        variant: payment_status_variant(&order.status),
                                                        "{order.status}"
                                                    }
                                                }
                                                td { "{order.created_at}" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn payment_status_variant(status: &str) -> BadgeVariant {
    match status {
        "paid" | "success" => BadgeVariant::Success,
        "pending" | "processing" => BadgeVariant::Warning,
        "failed" | "cancelled" => BadgeVariant::Error,
        "refunded" => BadgeVariant::Info,
        _ => BadgeVariant::Neutral,
    }
}
