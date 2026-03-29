use dioxus::prelude::*;
use ui::{Button, ButtonVariant};

use crate::services::settings_service;
use crate::stores::auth_store::AuthStore;
use crate::stores::user_store::UserStore;

/// 系统设置页面
///
/// - 普通用户：无此页面入口（个人偏好通过导航栏按鈕切换，存 localStorage）
/// - Admin：全局系统参数配置（调用 SettingsApi，需 Admin token）
#[component]
pub fn Settings() -> Element {
    let user_store = use_context::<UserStore>();
    let auth_store = use_context::<AuthStore>();
    let is_admin = user_store
        .info
        .read()
        .as_ref()
        .map(|u| u.is_admin())
        .unwrap_or(false);

    let settings = use_resource(move || async move {
        let token = auth_store.token().unwrap_or_default();
        settings_service::get_all(&token).await
    });

    let mut _saving = use_signal(|| false);
    let save_error = use_signal(String::new);
    let save_ok = use_signal(|| false);

    let get_val = move |key: &str| -> String {
        match settings() {
            Some(Ok(ref m)) => m
                .get(key)
                .and_then(|v| v.as_string())
                .unwrap_or("")
                .to_string(),
            _ => String::new(),
        }
    };

    let platform_name = get_val("platform_name");
    let register_mode = get_val("register_mode");
    let currency = get_val("default_currency");
    let min_recharge = get_val("min_recharge_amount");
    let jwt_expire = get_val("jwt_expire_hours");
    let email_verify = get_val("email_verification");

    rsx! {
        div { class: "page-header",
            h1 { class: "page-title", "系统设置" }
            p { class: "page-description",
                if is_admin { "配置平台全局系统参数" }
                else { "查看系统运行配置（仅供参考）" }
            }
        }

        if !is_admin {
            div { class: "alert alert-info",
                span { class: "alert-icon", "ℹ" }
                div { class: "alert-content",
                    p { class: "alert-body",
                        "系统设置仅 Admin 可修改。个人语言/主题偏好请通过顶部导航栏右侧按鈕切换。"
                    }
                }
            }
        }

        match settings() {
            None => rsx! { p { class: "text-secondary", "加载中..." } },
            Some(Err(_)) => rsx! { p { class: "text-secondary", "设置加载失败" } },
            Some(Ok(_)) => rsx! {
                if save_ok() {
                    div { class: "alert alert-success",
                        span { "✔ 设置已保存" }
                    }
                }
                if !save_error().is_empty() {
                    div { class: "alert alert-error",
                        span { "{save_error}" }
                    }
                }

                // 基础系统配置
                div { class: "card",
                    div { class: "card-header",
                        h3 { class: "card-title", "基础配置" }
                    }
                    div { class: "card-body",
                        div { class: "settings-grid",
                            SettingItem { label: "平台名称", value: platform_name.clone(), editable: is_admin }
                            SettingItem { label: "注册模式", value: register_mode.clone(), editable: is_admin }
                            SettingItem { label: "默认货币", value: currency.clone(), editable: is_admin }
                            SettingItem { label: "最低充値金额", value: min_recharge.clone(), editable: is_admin }
                        }
                    }
                }

                // 安全配置
                div { class: "card",
                    div { class: "card-header",
                        h3 { class: "card-title", "安全配置" }
                    }
                    div { class: "card-body",
                        div { class: "settings-grid",
                            SettingItem { label: "JWT Token 有效期（小时）", value: jwt_expire.clone(), editable: is_admin }
                            SettingItem { label: "邮筱验证", value: email_verify.clone(), editable: is_admin }
                        }
                    }
                }
            },
        }
    }
}

// ── 内部组件

#[component]
fn SettingItem(label: String, value: String, editable: bool) -> Element {
    let mut edit_val = use_signal(|| value.clone());

    rsx! {
        div { class: "setting-item",
            span { class: "setting-label", "{label}" }
            if editable {
                div { class: "setting-input-row",
                    input {
                        class: "input-field",
                        value: "{edit_val}",
                        oninput: move |e| *edit_val.write() = e.value(),
                    }
                    Button {
                        variant: ButtonVariant::Secondary,
                        size: ui::ButtonSize::Small,
                        onclick: move |_| {},
                        "保存"
                    }
                }
            } else {
                span { class: "setting-value",
                    if value.is_empty() { "—" } else { "{value}" }
                }
            }
        }
    }
}
