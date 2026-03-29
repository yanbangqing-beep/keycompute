use dioxus::prelude::*;

/// UI 全局状态（侧边栏、Toast 通知等页面级状态）
#[derive(Clone, Copy)]
pub struct UiStore {
    /// 全局 Toast 消息
    pub toast: Signal<Option<ToastMsg>>,
}

/// Toast 消息
#[derive(Clone, PartialEq)]
pub struct ToastMsg {
    pub kind: ToastKind,
    pub title: String,
    pub message: Option<String>,
}

#[derive(Clone, PartialEq)]
pub enum ToastKind {
    Success,
    Error,
    Warning,
    Info,
}

impl ToastKind {
    pub fn css_class(&self) -> &'static str {
        match self {
            Self::Success => "toast toast-success",
            Self::Error => "toast toast-error",
            Self::Warning => "toast toast-warning",
            Self::Info => "toast toast-info",
        }
    }
}

impl UiStore {
    pub fn new() -> Self {
        Self {
            toast: use_signal(|| None),
        }
    }

    pub fn show_success(&mut self, title: impl Into<String>) {
        let mut toast = self.toast;
        *toast.write() = Some(ToastMsg {
            kind: ToastKind::Success,
            title: title.into(),
            message: None,
        });
        spawn(async move {
            gloo_timers::future::TimeoutFuture::new(3_000).await;
            *toast.write() = None;
        });
    }

    pub fn show_error(&mut self, title: impl Into<String>) {
        let mut toast = self.toast;
        *toast.write() = Some(ToastMsg {
            kind: ToastKind::Error,
            title: title.into(),
            message: None,
        });
        spawn(async move {
            gloo_timers::future::TimeoutFuture::new(5_000).await;
            *toast.write() = None;
        });
    }

    #[allow(dead_code)]
    pub fn show_error_msg(&mut self, title: impl Into<String>, msg: impl Into<String>) {
        let mut toast = self.toast;
        *toast.write() = Some(ToastMsg {
            kind: ToastKind::Error,
            title: title.into(),
            message: Some(msg.into()),
        });
        spawn(async move {
            gloo_timers::future::TimeoutFuture::new(5_000).await;
            *toast.write() = None;
        });
    }

    #[allow(dead_code)]
    pub fn clear_toast(&mut self) {
        *self.toast.write() = None;
    }
}
