use std::sync::atomic::{AtomicU32, Ordering};

/// Token 累积器：streaming 过程中原子更新
#[derive(Debug, Default)]
pub struct UsageAccumulator {
    input_tokens: AtomicU32,
    output_tokens: AtomicU32,
}

impl UsageAccumulator {
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加输出 token（流式响应中累积）
    pub fn add_output(&self, tokens: u32) {
        self.output_tokens.fetch_add(tokens, Ordering::Relaxed);
    }

    /// 设置输入 token（通常从 provider 报告获取）
    pub fn set_input(&self, tokens: u32) {
        self.input_tokens.store(tokens, Ordering::Relaxed);
    }

    /// 获取当前用量快照
    pub fn snapshot(&self) -> (u32, u32) {
        (
            self.input_tokens.load(Ordering::Relaxed),
            self.output_tokens.load(Ordering::Relaxed),
        )
    }

    /// 获取总 token 数
    pub fn total_tokens(&self) -> u32 {
        let (input, output) = self.snapshot();
        input + output
    }
}

/// 从 snapshot 创建新的 UsageAccumulator
///
/// 注意：这会创建独立的 Accumulator，不会共享状态。
/// 如需共享状态，请使用 Arc<UsageAccumulator>。
impl From<(u32, u32)> for UsageAccumulator {
    fn from((input, output): (u32, u32)) -> Self {
        let acc = Self::new();
        acc.set_input(input);
        // 直接设置输出 token，避免循环
        acc.output_tokens.store(output, Ordering::Relaxed);
        acc
    }
}

/// 最终用量记录
#[derive(Debug, Clone, Copy)]
pub struct UsageRecord {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

impl UsageRecord {
    pub fn total(&self) -> u32 {
        self.input_tokens + self.output_tokens
    }
}

impl From<(u32, u32)> for UsageRecord {
    fn from((input, output): (u32, u32)) -> Self {
        Self {
            input_tokens: input,
            output_tokens: output,
        }
    }
}
