use dioxus::prelude::*;

use crate::services::{api_client::with_auto_refresh, distribution_service};
use crate::stores::auth_store::AuthStore;

#[component]
pub fn DistributionOverview() -> Element {
    let auth_store = use_context::<AuthStore>();

    // 收益数据
    let earnings = use_resource(move || async move {
        with_auto_refresh(auth_store, |token| async move {
            distribution_service::get_earnings(&token).await
        })
        .await
    });

    // 推荐码
    let referral_code = use_resource(move || async move {
        with_auto_refresh(auth_store, |token| async move {
            distribution_service::get_referral_code(&token).await
        })
        .await
    });

    // 推荐列表
    let referrals = use_resource(move || async move {
        with_auto_refresh(auth_store, |token| async move {
            distribution_service::get_referrals(&token).await
        })
        .await
    });

    let total_earnings = match earnings() {
        Some(Ok(ref e)) => format!("¥{:.2}", e.total_earnings),
        Some(Err(_)) => "加载失败".to_string(),
        None => "加载中...".to_string(),
    };
    let available_earnings = match earnings() {
        Some(Ok(ref e)) => format!("¥{:.2}", e.available_earnings),
        _ => "—".to_string(),
    };
    let pending_earnings = match earnings() {
        Some(Ok(ref e)) => format!("¥{:.2}", e.pending_earnings),
        _ => "—".to_string(),
    };
    let referral_count = match earnings() {
        Some(Ok(ref e)) => e.referral_count.to_string(),
        _ => "—".to_string(),
    };
    let code_text = match referral_code() {
        Some(Ok(ref r)) => r.referral_code.clone(),
        Some(Err(_)) => "获取失败".to_string(),
        None => "加载中...".to_string(),
    };
    let invite_link = match referral_code() {
        Some(Ok(ref r)) => r.referral_link.clone(),
        _ => String::new(),
    };

    rsx! {
        div {
            class: "page-container",
            div {
                class: "page-header",
                h1 { class: "page-title", "分销管理" }
                p { class: "page-subtitle", "查看您的分销收益和推荐记录" }
            }

            // 收益统计
            div {
                class: "stats-grid",
                div { class: "stat-card card",
                    div { class: "card-body",
                        p { class: "stat-label", "总收益" }
                        p { class: "stat-value", "{total_earnings}" }
                    }
                }
                div { class: "stat-card card",
                    div { class: "card-body",
                        p { class: "stat-label", "可用余额" }
                        p { class: "stat-value", "{available_earnings}" }
                    }
                }
                div { class: "stat-card card",
                    div { class: "card-body",
                        p { class: "stat-label", "待结算" }
                        p { class: "stat-value", "{pending_earnings}" }
                    }
                }
                div { class: "stat-card card",
                    div { class: "card-body",
                        p { class: "stat-label", "推荐人数" }
                        p { class: "stat-value", "{referral_count}" }
                    }
                }
            }

            // 推荐码
            div { class: "card",
                div { class: "card-header",
                    h3 { class: "card-title", "我的推荐码" }
                }
                div { class: "card-body",
                    div { class: "info-grid",
                        div { class: "info-item",
                            span { class: "info-label", "推荐码" }
                            span { class: "info-value",
                                code { "{code_text}" }
                            }
                        }
                        if !invite_link.is_empty() {
                            div { class: "info-item",
                                span { class: "info-label", "邀请链接" }
                                span { class: "info-value",
                                    a { href: "{invite_link}", target: "_blank", "{invite_link}" }
                                }
                            }
                        }
                    }
                }
            }

            // 推荐列表
            div { class: "card",
                div { class: "card-header",
                    h3 { class: "card-title", "推荐用户" }
                }
                div { class: "table-container",
                    table { class: "table",
                        thead {
                            tr {
                                th { "用户" }
                                th { "加入时间" }
                                th { "消费总额" }
                                th { "我的收益" }
                            }
                        }
                        tbody {
                            match referrals() {
                                Some(Ok(ref list)) if !list.is_empty() => rsx! {
                                    for r in list.iter() {
                                        tr {
                                            td {
                                                div { class: "user-cell",
                                                    span { class: "user-name",
                                                        { r.name.clone().unwrap_or_else(|| r.email.clone()) }
                                                    }
                                                    span { class: "user-email", "{r.email}" }
                                                }
                                            }
                                            td { "{r.joined_at}" }
                                            td { "¥{r.total_spent:.2}" }
                                            td { "¥{r.earnings_from_referral:.2}" }
                                        }
                                    }
                                },
                                Some(Err(_)) => rsx! {
                                    tr { td { colspan: "4", class: "table-empty", "加载失败" } }
                                },
                                _ => rsx! {
                                    tr { td { colspan: "4", class: "table-empty", "暂无推荐记录" } }
                                },
                            }
                        }
                    }
                }
            }
        }
    }
}
