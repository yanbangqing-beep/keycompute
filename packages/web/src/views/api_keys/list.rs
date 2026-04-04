use dioxus::prelude::*;
use ui::{Badge, BadgeVariant, Button, ButtonSize, ButtonVariant, Pagination, Table, TableHead};

const PAGE_SIZE: usize = 20;

use crate::services::{api_client::with_auto_refresh, api_key_service, model_service};
use crate::stores::auth_store::AuthStore;
use crate::utils::time::format_time;

/// 复制文本到剪贴板（WASM 环境）
fn copy_to_clipboard(text: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        let _ = web_sys::window().map(|w| {
            let clipboard = w.navigator().clipboard();
            clipboard.write_text(text)
        });
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = text; // 非 WASM 环境暂不支持
    }
}

#[component]
pub fn ApiKeyList() -> Element {
    let auth_store = use_context::<AuthStore>();
    let mut show_create = use_signal(|| false);
    let mut new_key_name = use_signal(String::new);
    let mut creating = use_signal(|| false);
    let mut create_error = use_signal(|| Option::<String>::None);
    let mut new_key_value = use_signal(|| Option::<String>::None);
    let mut page = use_signal(|| 1u32);
    // 是否显示已撤销的 Key（默认不显示）
    let mut include_revoked = use_signal(|| false);
    // 复制状态
    let mut copied = use_signal(|| false);

    // 获取模型列表（用于显示用法示例）
    let models = use_resource(move || async move { model_service::list_models().await.ok() });

    // 拉取 key 列表
    let mut keys = use_resource(move || async move {
        with_auto_refresh(auth_store, |token| async move {
            api_key_service::list(include_revoked(), &token).await
        })
        .await
    });

    let on_create = move |evt: Event<FormData>| {
        evt.prevent_default();
        let name = new_key_name();
        if name.is_empty() {
            return;
        }
        creating.set(true);
        create_error.set(None);
        spawn(async move {
            let token = auth_store.token().unwrap_or_default();
            match api_key_service::create(&name, &token).await {
                Ok(resp) => {
                    new_key_value.set(Some(resp.api_key));
                    show_create.set(false);
                    new_key_name.set(String::new());
                    creating.set(false);
                    page.set(1);
                    // 重新拉取列表
                    keys.restart();
                }
                Err(e) => {
                    create_error.set(Some(format!("创建失败：{e}")));
                    creating.set(false);
                }
            }
        });
    };

    let on_delete = move |id: String| {
        spawn(async move {
            let token = auth_store.token().unwrap_or_default();
            if api_key_service::delete(&id, &token).await.is_ok() {
                keys.restart();
            }
        });
    };

    rsx! {
        div {
            class: "page-container",
            div {
                class: "page-header",
                h1 { class: "page-title", "API Key 管理" }
                Button {
                    variant: ButtonVariant::Primary,
                    onclick: move |_| {
                        show_create.set(true);
                        new_key_value.set(None);
                    },
                    "+ 创建 API Key"
                }
            }

            // 筛选工具栏
            div { class: "toolbar",
                div { class: "toolbar-left",
                    div { class: "filter-tabs",
                        button {
                            class: if !include_revoked() { "filter-tab active" } else { "filter-tab" },
                            r#type: "button",
                            onclick: move |_| {
                                include_revoked.set(false);
                                page.set(1);
                                keys.restart();
                            },
                            "活跃"
                        }
                        button {
                            class: if include_revoked() { "filter-tab active" } else { "filter-tab" },
                            r#type: "button",
                            onclick: move |_| {
                                include_revoked.set(true);
                                page.set(1);
                                keys.restart();
                            },
                            "全部（含已撤销）"
                        }
                    }
                }
            }

            // 新建成功后展示完整密钥（仅一次）
            if let Some(key) = new_key_value() {
                {
                    // 使用编译时配置的 API_BASE_URL，移除末尾的 /v1 后缀（如果有）
                    let api_url = crate::services::api_client::get_client()
                        .config()
                        .base_url
                        .trim_end_matches('/')
                        .trim_end_matches("/v1")
                        .to_string();

                    // 获取第一个模型作为示例
                    let sample_model = models()
                        .flatten()
                        .and_then(|m| m.data.first().map(|model| model.id.clone()))
                        .unwrap_or_else(|| "deepseek-chat".to_string());

                    // 生成要复制的文本
                    let example_text = format!(
                        r#"API_URL="{}/v1"
API_KEY="{}"
API_MODEL="{}""#,
                        api_url, key, sample_model
                    );
                    let example_text_for_click = example_text.clone();

                    rsx! {
                        div {
                            class: "alert alert-success",
                            p { strong { "API Key 已创建，请妥善保存（仅显示一次）：" } }
                            code { class: "key-display", "{key}" }
                            p { style: "margin-top: 12px; font-weight: bold;", "使用示例：" }
                            div {
                                style: "position: relative; margin-top: 8px;",
                                pre {
                                    style: if copied() {
                                        "background: #e8f5e9; padding: 12px; border-radius: 4px; overflow-x: auto; font-size: 13px; cursor: pointer; border: 2px solid #4caf50;"
                                    } else {
                                        "background: #f5f5f5; padding: 12px; border-radius: 4px; overflow-x: auto; font-size: 13px; cursor: pointer; border: 2px solid transparent;"
                                    },
                                    title: if copied() { "已复制!" } else { "点击复制" },
                                    onclick: {
                                        let text = example_text_for_click.clone();
                                        move |_| {
                                            copy_to_clipboard(&text);
                                            copied.set(true);
                                            // 2秒后重置状态
                                            let mut copied_clone = copied.clone();
                                            spawn(async move {
                                                gloo_timers::future::TimeoutFuture::new(2000).await;
                                                copied_clone.set(false);
                                            });
                                        }
                                    },
                                    "{example_text}"
                                }
                                div {
                                    style: "position: absolute; top: 8px; right: 8px; font-size: 12px; color: #666; pointer-events: none;",
                                    if copied() {
                                        "✓ 已复制"
                                    } else {
                                        "点击复制"
                                    }
                                }
                            }
                            p {
                                style: "margin-top: 8px; color: #666; font-size: 13px;",
                                "将以上配置用于 OpenAI 兼容的 SDK 或工具中。"
                            }
                            Button {
                                variant: ButtonVariant::Ghost,
                                size: ButtonSize::Small,
                                onclick: move |_| {
                                    new_key_value.set(None);
                                    copied.set(false);
                                },
                                "我已记录，关闭"
                            }
                        }
                    }
                }
            }

            // 创建弹窗
            if show_create() {
                div {
                    class: "modal-overlay",
                    div {
                        class: "modal",
                        h2 { class: "modal-title", "创建 API Key" }
                        if let Some(err) = create_error() {
                            div { class: "alert alert-error", "{err}" }
                        }
                        form {
                            onsubmit: on_create,
                            div {
                                class: "form-group",
                                label { class: "form-label", "名称" }
                                input {
                                    class: "form-input",
                                    r#type: "text",
                                    placeholder: "为此 Key 取个名字",
                                    value: "{new_key_name}",
                                    oninput: move |e| new_key_name.set(e.value()),
                                }
                            }
                            div {
                                class: "modal-actions",
                                Button {
                                    variant: ButtonVariant::Ghost,
                                    r#type: "button".to_string(),
                                    onclick: move |_| show_create.set(false),
                                    "取消"
                                }
                                Button {
                                    variant: ButtonVariant::Primary,
                                    r#type: "submit".to_string(),
                                    loading: creating(),
                                    if creating() { "创建中..." } else { "创建" }
                                }
                            }
                        }
                    }
                }
            }

            match keys() {
                None => rsx! {
                    div { class: "loading-state", "加载中..." }
                },
                Some(Err(e)) => rsx! {
                    div { class: "alert alert-error", "加载失败：{e}" }
                },
                Some(Ok(list)) => {
                    let total = list.len();
                    let total_pages = total.div_ceil(PAGE_SIZE).max(1) as u32;
                    let start = (page() as usize - 1) * PAGE_SIZE;
                    let paged: Vec<_> = list.iter().skip(start).take(PAGE_SIZE).collect();
                    if paged.is_empty() && total == 0 {
                        rsx! {
                            Table {
                                col_count: 5,
                                empty: true,
                                empty_text: "暂无可用的 API Key，点击上方按钮创建".to_string(),
                                thead { tr { TableHead { "" } } }
                            }
                        }
                    } else {
                        rsx! {
                            Table {
                                col_count: 5,
                                thead {
                                    tr {
                                        TableHead { "名称" }
                                        TableHead { "前缀" }
                                        TableHead { "状态" }
                                        TableHead { "创建时间" }
                                        TableHead { "操作" }
                                    }
                                }
                                tbody {
                                    for key in paged.iter() {
                                        tr {
                                            key: "{key.id}",
                                            td { "{key.name}" }
                                            td { code { "{key.key_preview}" } }
                                            td {
                                                Badge {
                                                    variant: if key.revoked() { BadgeVariant::Error } else { BadgeVariant::Success },
                                                    if key.revoked() { "已撤销" } else { "活跃" }
                                                }
                                            }
                                            td { { format_time(&key.created_at) } }
                                            td {
                                                Button {
                                                    variant: ButtonVariant::Danger,
                                                    size: ButtonSize::Small,
                                                    onclick: {
                                                        let id = key.id.to_string();
                                                        move |_| on_delete(id.clone())
                                                    },
                                                    "删除"
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
                }
            }
        }
    }
}
