use client_api::api::admin::CreatePricingRequest;
use dioxus::prelude::*;
use ui::{Badge, BadgeVariant, Pagination, Table, TableHead};

const PAGE_SIZE: usize = 20;

use crate::services::{api_client::with_auto_refresh, pricing_service};
use crate::stores::auth_store::AuthStore;
use crate::stores::user_store::UserStore;
use crate::utils::time::format_time;

/// 定价管理页面
///
/// - 普通用户：只读查看定价策略列表
/// - Admin：完整 CRUD（创建/删除/设置默认）
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

    // 控制创建弹窗
    let mut show_create = use_signal(|| false);
    // 操作结果提示
    let mut op_ok = use_signal(String::new);
    let mut op_err = use_signal(String::new);
    // 刷新触发器
    let mut refresh_tick = use_signal(|| 0u32);
    // 分页
    let mut page = use_signal(|| 1u32);

    let pricing_list = use_resource(move || async move {
        let _tick = refresh_tick();
        with_auto_refresh(auth_store, |token| async move {
            pricing_service::list(&token).await
        })
        .await
    });

    let col_count: u32 = if is_admin { 7 } else { 6 };

    rsx! {
        div { class: "page-container",
            div { class: "page-header",
                h1 { class: "page-title", "定价管理" }
                p { class: "page-description",
                    if is_admin { "管理平台定价策略，设置模型调用费率" }
                    else { "查看当前平台可用的定价策略" }
                }
                if is_admin {
                    button {
                        class: "btn btn-primary",
                        onclick: move |_| show_create.set(true),
                        "+ 新建定价"
                    }
                }
            }

            // 操作结果提示
            if !op_ok().is_empty() {
                div { class: "alert alert-success", "{op_ok}" }
            }
            if !op_err().is_empty() {
                div { class: "alert alert-error", "{op_err}" }
            }

            {
                let (is_empty, empty_text) = match pricing_list() {
                    None => (true, "加载中..."),
                    Some(Err(_)) => (true, "加载失败"),
                    Some(Ok(ref l)) if l.is_empty() => (true, "暂无定价策略"),
                    _ => (false, ""),
                };
                let total = pricing_list().and_then(|r| r.ok()).map(|l| l.len()).unwrap_or(0);
                let total_pages = total.div_ceil(PAGE_SIZE).max(1) as u32;
                let start = (page() as usize - 1) * PAGE_SIZE;
                let paged_list: Vec<_> = pricing_list()
                    .and_then(|r| r.ok())
                    .map(|l| l.into_iter().skip(start).take(PAGE_SIZE).collect())
                    .unwrap_or_default();
                rsx! {
                    Table {
                        empty: is_empty,
                        empty_text: empty_text.to_string(),
                        col_count,
                        thead {
                            tr {
                                TableHead { "模型" }
                                TableHead { "输入单价（/1K tokens）" }
                                TableHead { "输出单价（/1K tokens）" }
                                TableHead { "货币" }
                                TableHead { "默认" }
                                TableHead { "创建时间" }
                                if is_admin {
                                    TableHead { "操作" }
                                }
                            }
                        }
                        tbody {
                            if pricing_list().and_then(|r| r.ok()).is_some() {
                                for p in paged_list.iter() {
                                    tr {
                                        key: "{p.id}",
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
                                        td { { format_time(&p.created_at) } }
                                        if is_admin {
                                            td {
                                                div { class: "action-buttons",
                                                    if !p.is_default {
                                                        {
                                                            let pid = p.id.clone();
                                                            rsx! {
                                                                button {
                                                                    class: "btn btn-sm btn-secondary",
                                                                    onclick: move |_| {
                                                                        let id = pid.clone();
                                                                        let token = auth_store.token().unwrap_or_default();
                                                                        spawn(async move {
                                                                            use client_api::api::admin::SetDefaultPricingRequest;
                                                                            let req = SetDefaultPricingRequest { model_ids: vec![id] };
                                                                            match pricing_service::set_defaults(req, &token).await {
                                                                                Ok(_) => {
                                                                                    op_ok.set("已设为默认定价".to_string());
                                                                                    op_err.set(String::new());
                                                                                    *refresh_tick.write() += 1;
                                                                                }
                                                                                Err(e) => {
                                                                                    op_err.set(format!("设置默认失败：{e}"));
                                                                                }
                                                                            }
                                                                        });
                                                                    },
                                                                    "设为默认"
                                                                }
                                                            }
                                                        }
                                                    }
                                                    {
                                                        let pid = p.id.clone();
                                                        rsx! {
                                                            button {
                                                                class: "btn btn-sm btn-danger",
                                                                onclick: move |_| {
                                                                    let id = pid.clone();
                                                                    let token = auth_store.token().unwrap_or_default();
                                                                    spawn(async move {
                                                                        match pricing_service::delete(&id, &token).await {
                                                                            Ok(_) => {
                                                                                op_ok.set("定价已删除".to_string());
                                                                                op_err.set(String::new());
                                                                                *refresh_tick.write() += 1;
                                                                            }
                                                                            Err(e) => {
                                                                                op_err.set(format!("删除失败：{e}"));
                                                                            }
                                                                        }
                                                                    });
                                                                },
                                                                "删除"
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

            // 创建定价弹窗
            if show_create() {
                CreatePricingModal {
                    auth_store,
                    on_close: move |_| show_create.set(false),
                    on_created: move |_| {
                        show_create.set(false);
                        op_ok.set("定价创建成功".to_string());
                        op_err.set(String::new());
                        page.set(1);
                        *refresh_tick.write() += 1;
                    },
                }
            }
        }
    }
}

/// 创建定价弹窗
#[component]
fn CreatePricingModal(
    auth_store: AuthStore,
    on_close: EventHandler,
    on_created: EventHandler,
) -> Element {
    let mut model = use_signal(String::new);
    let mut input_price = use_signal(String::new);
    let mut output_price = use_signal(String::new);
    let mut currency = use_signal(|| "CNY".to_string());
    let mut saving = use_signal(|| false);
    let mut form_err = use_signal(String::new);

    let on_submit = move |evt: Event<FormData>| {
        evt.prevent_default();
        let m = model();
        let ip_str = input_price();
        let op_str = output_price();
        let cur = currency();
        if m.is_empty() || ip_str.is_empty() || op_str.is_empty() {
            form_err.set("请填写所有字段".to_string());
            return;
        }
        let ip: f64 = match ip_str.parse() {
            Ok(v) => v,
            Err(_) => {
                form_err.set("输入单价格式不正确".to_string());
                return;
            }
        };
        let op: f64 = match op_str.parse() {
            Ok(v) => v,
            Err(_) => {
                form_err.set("输出单价格式不正确".to_string());
                return;
            }
        };
        saving.set(true);
        form_err.set(String::new());
        let token = auth_store.token().unwrap_or_default();
        spawn(async move {
            let req = CreatePricingRequest::new(m, ip, op, cur);
            match pricing_service::create(req, &token).await {
                Ok(_) => {
                    saving.set(false);
                    on_created.call(());
                }
                Err(e) => {
                    form_err.set(format!("创建失败：{e}"));
                    saving.set(false);
                }
            }
        });
    };

    rsx! {
        div { class: "modal-overlay",
            onclick: move |_| on_close.call(()),
        }
        div { class: "modal",
            div { class: "modal-header",
                h3 { class: "modal-title", "新建定价" }
                button {
                    class: "modal-close",
                    r#type: "button",
                    onclick: move |_| on_close.call(()),
                    "×"
                }
            }
            div { class: "modal-body",
                if !form_err().is_empty() {
                    div { class: "alert alert-error", "{form_err}" }
                }
                form {
                    onsubmit: on_submit,
                    div { class: "form-group",
                        label { class: "form-label", "模型名称" }
                        input {
                            class: "form-input",
                            r#type: "text",
                            placeholder: "如 gpt-4o",
                            value: "{model}",
                            oninput: move |e| model.set(e.value()),
                        }
                    }
                    div { class: "form-group",
                        label { class: "form-label", "输入单价（每1K tokens）" }
                        input {
                            class: "form-input",
                            r#type: "number",
                            placeholder: "如 0.000005",
                            step: "0.000001",
                            value: "{input_price}",
                            oninput: move |e| input_price.set(e.value()),
                        }
                    }
                    div { class: "form-group",
                        label { class: "form-label", "输出单价（每1K tokens）" }
                        input {
                            class: "form-input",
                            r#type: "number",
                            placeholder: "如 0.000015",
                            step: "0.000001",
                            value: "{output_price}",
                            oninput: move |e| output_price.set(e.value()),
                        }
                    }
                    div { class: "form-group",
                        label { class: "form-label", "货币" }
                        select {
                            class: "form-input",
                            value: "{currency}",
                            onchange: move |e| currency.set(e.value()),
                            option { value: "CNY", "CNY（人民币）" }
                            option { value: "USD", "USD（美元）" }
                        }
                    }
                    div { class: "modal-footer",
                        button {
                            class: "btn btn-secondary",
                            r#type: "button",
                            onclick: move |_| on_close.call(()),
                            "取消"
                        }
                        button {
                            class: "btn btn-primary",
                            r#type: "submit",
                            disabled: saving(),
                            if saving() { "创建中..." } else { "创建" }
                        }
                    }
                }
            }
        }
    }
}
