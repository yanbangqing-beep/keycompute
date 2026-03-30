use dioxus::prelude::*;
use ui::{Badge, BadgeVariant, Pagination, Table, TableHead};

const PAGE_SIZE: usize = 20;

use crate::services::{api_client::with_auto_refresh, distribution_service};
use crate::stores::auth_store::AuthStore;
use crate::stores::user_store::UserStore;
use crate::utils::time::format_time;

/// 分销记录页面
///
/// - 普通用户：查看自己的推荐用户明细（真实表格数据）
/// - Admin：查看全平台分销记录，配置分销规则
#[component]
pub fn DistributionRecords() -> Element {
    let user_store = use_context::<UserStore>();
    let auth_store = use_context::<AuthStore>();
    let is_admin = user_store
        .info
        .read()
        .as_ref()
        .map(|u| u.is_admin())
        .unwrap_or(false);

    // 收益数据（普通用户）
    let earnings = use_resource(move || async move {
        with_auto_refresh(auth_store, |token| async move {
            distribution_service::get_earnings(&token).await
        })
        .await
    });

    // 普通用户：推荐明细列表
    let referrals = use_resource(move || async move {
        if is_admin {
            return Ok(vec![]);
        }
        with_auto_refresh(auth_store, |token| async move {
            distribution_service::get_referrals(&token).await
        })
        .await
    });

    // Admin：全平台分销记录
    let admin_records = use_resource(move || async move {
        if !is_admin {
            return Ok(vec![]);
        }
        with_auto_refresh(auth_store, |token| async move {
            use crate::services::api_client::get_client;
            use client_api::DistributionApi;
            let client = get_client();
            DistributionApi::new(&client)
                .list_distribution_records(None, &token)
                .await
        })
        .await
    });

    let total_earnings = match earnings() {
        Some(Ok(ref e)) => format!("¥{:.2}", e.total_earnings),
        _ => "¥ 0.00".to_string(),
    };
    let available = match earnings() {
        Some(Ok(ref e)) => format!("¥{:.2}", e.available_earnings),
        _ => "¥ 0.00".to_string(),
    };
    let pending = match earnings() {
        Some(Ok(ref e)) => format!("¥{:.2}", e.pending_earnings),
        _ => "¥ 0.00".to_string(),
    };

    let mut page = use_signal(|| 1u32);

    rsx! {
        div { class: "page-header",
            h1 { class: "page-title", "分销记录" }
            p { class: "page-description",
                if is_admin { "查看全平台分销收益记录，管理分销规则" }
                else { "查看您通过邀请获得的分销收益明细" }
            }
        }

        // 收益统计卡片
        div { class: "stats-grid",
            div { class: "stat-card card",
                div { class: "card-body",
                    p { class: "stat-label", "总收益" }
                    p { class: "stat-value", "{total_earnings}" }
                }
            }
            div { class: "stat-card card",
                div { class: "card-body",
                    p { class: "stat-label", "可用余额" }
                    p { class: "stat-value", "{available}" }
                }
            }
            div { class: "stat-card card",
                div { class: "card-body",
                    p { class: "stat-label", "待结算" }
                    p { class: "stat-value", "{pending}" }
                }
            }
        }

        if is_admin {
            div { class: "toolbar",
                div { class: "toolbar-right",
                    button { class: "btn btn-secondary btn-sm", r#type: "button",
                        "分销规则配置"
                    }
                }
            }
        }

        // 表格：admin 视图 / 普通用户视图分别渲染
        if is_admin {
            {
                let (is_empty, empty_text) = match admin_records() {
                    None => (true, "加载中..."),
                    Some(Err(_)) => (true, "加载失败"),
                    Some(Ok(ref l)) if l.is_empty() => (true, "暂无分销记录"),
                    _ => (false, ""),
                };
                let admin_start = (page() as usize - 1) * PAGE_SIZE;
                rsx! {
                    Table {
                        empty: is_empty,
                        empty_text: empty_text.to_string(),
                        col_count: 7u32,
                        thead {
                            tr {
                                TableHead { "记录编号" }
                                TableHead { "来源用户" }
                                TableHead { "消费金额" }
                                TableHead { "分销金额" }
                                TableHead { "状态" }
                                TableHead { "创建时间" }
                                TableHead { "推荐人" }
                            }
                        }
                        tbody {
                            if let Some(Ok(ref list)) = admin_records() {
                                for rec in list.iter().skip(admin_start).take(PAGE_SIZE) {
                                    tr {
                                        td { code { "{rec.id}" } }
                                        td { "{rec.referred_id}" }
                                        td { "¥{rec.amount:.2}" }
                                        td { "¥{rec.commission:.2}" }
                                        td {
                                            Badge {
                                                variant: dist_status_variant(&rec.status),
                                                "{rec.status}"
                                            }
                                        }
                                        td { { format_time(&rec.created_at) } }
                                        td { "{rec.referrer_id}" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        } else {
            {
                let (is_empty, empty_text) = match referrals() {
                    None => (true, "加载中..."),
                    Some(Err(_)) => (true, "加载失败"),
                    Some(Ok(ref l)) if l.is_empty() => (true, "暂无推荐记录"),
                    _ => (false, ""),
                };
                let ref_start = (page() as usize - 1) * PAGE_SIZE;
                rsx! {
                    Table {
                        empty: is_empty,
                        empty_text: empty_text.to_string(),
                        col_count: 4u32,
                        thead {
                            tr {
                                TableHead { "被推荐用户" }
                                TableHead { "加入时间" }
                                TableHead { "消费总额" }
                                TableHead { "我的收益" }
                            }
                        }
                        tbody {
                            if let Some(Ok(ref list)) = referrals() {
                                for r in list.iter().skip(ref_start).take(PAGE_SIZE) {
                                    tr {
                                        td {
                                            div { class: "user-cell",
                                                span { class: "user-name",
                                                    { r.name.clone().unwrap_or_else(|| r.email.clone()) }
                                                }
                                                span { class: "user-email text-secondary", "{r.email}" }
                                            }
                                        }
                                        td { { format_time(&r.joined_at) } }
                                        td { "¥{r.total_spent:.2}" }
                                        td { "¥{r.earnings_from_referral:.2}" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        {
            let total = if is_admin {
                admin_records().and_then(|r| r.ok()).map(|l| l.len()).unwrap_or(0)
            } else {
                referrals().and_then(|r| r.ok()).map(|l| l.len()).unwrap_or(0)
            };
            let total_pages = total.div_ceil(PAGE_SIZE).max(1) as u32;
            rsx! {
                div { class: "pagination",
                    span { class: "pagination-info", "共 {total} 条" }
                    Pagination {
                        current: page(),
                        total_pages,
                        on_page_change: move |p| page.set(p),
                    }
                }
            }
        }
    }
}

fn dist_status_variant(status: &str) -> BadgeVariant {
    match status {
        "settled" | "paid" => BadgeVariant::Success,
        "pending" => BadgeVariant::Warning,
        "cancelled" | "failed" => BadgeVariant::Error,
        _ => BadgeVariant::Neutral,
    }
}
