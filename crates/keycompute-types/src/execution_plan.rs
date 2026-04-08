use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// 敏感字符串：用于保护 API Key 等敏感信息
///
/// - 序列化时会隐藏内容（显示为 ***REDACTED***）
/// - Debug 时隐藏内容
/// - Display 时隐藏内容
#[derive(Clone, Default, Deserialize, PartialEq)]
pub struct SensitiveString(String);

impl SensitiveString {
    /// 创建新的敏感字符串
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// 获取内部值（谨慎使用）
    pub fn expose(&self) -> &str {
        &self.0
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// 获取长度
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl fmt::Debug for SensitiveString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "***REDACTED***")
    }
}

impl fmt::Display for SensitiveString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "***REDACTED***")
    }
}

impl Serialize for SensitiveString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str("***REDACTED***")
    }
}

impl From<String> for SensitiveString {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for SensitiveString {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// 执行计划：包含主目标和回退链
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    pub primary: ExecutionTarget,
    pub fallback_chain: Vec<ExecutionTarget>,
}

impl ExecutionPlan {
    pub fn new(primary: ExecutionTarget) -> Self {
        Self {
            primary,
            fallback_chain: Vec::new(),
        }
    }

    pub fn with_fallback(mut self, fallback: ExecutionTarget) -> Self {
        self.fallback_chain.push(fallback);
        self
    }

    pub fn with_fallbacks(mut self, fallbacks: Vec<ExecutionTarget>) -> Self {
        self.fallback_chain.extend(fallbacks);
        self
    }

    /// 获取所有执行目标（主目标 + 回退链）
    pub fn all_targets(&self) -> impl Iterator<Item = &ExecutionTarget> {
        std::iter::once(&self.primary).chain(self.fallback_chain.iter())
    }
}

/// 执行目标：指定具体的 provider 和账号
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionTarget {
    pub provider: String,
    pub account_id: Uuid,
    pub endpoint: String,
    pub upstream_api_key: SensitiveString, // 已解密的上游 Provider API Key（敏感信息自动隐藏）
}

impl ExecutionTarget {
    pub fn new(
        provider: impl Into<String>,
        account_id: Uuid,
        endpoint: impl Into<String>,
        upstream_api_key: impl Into<SensitiveString>,
    ) -> Self {
        Self {
            provider: provider.into(),
            account_id,
            endpoint: endpoint.into(),
            upstream_api_key: upstream_api_key.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sensitive_string_creation() {
        let secret = SensitiveString::new("my-secret-key");
        assert_eq!(secret.expose(), "my-secret-key");
        assert_eq!(secret.len(), 13);
        assert!(!secret.is_empty());
    }

    #[test]
    fn test_sensitive_string_debug() {
        let secret = SensitiveString::new("my-secret-key");
        let debug_str = format!("{:?}", secret);
        assert_eq!(debug_str, "***REDACTED***");
        assert!(!debug_str.contains("my-secret-key"));
    }

    #[test]
    fn test_sensitive_string_display() {
        let secret = SensitiveString::new("my-secret-key");
        let display_str = format!("{}", secret);
        assert_eq!(display_str, "***REDACTED***");
        assert!(!display_str.contains("my-secret-key"));
    }

    #[test]
    fn test_sensitive_string_serialize() {
        let secret = SensitiveString::new("my-secret-key");
        let json = serde_json::to_string(&secret).unwrap();
        assert_eq!(json, "\"***REDACTED***\"");
        assert!(!json.contains("my-secret-key"));
    }

    #[test]
    fn test_sensitive_string_deserialize() {
        let json = "\"my-secret-key\"";
        let secret: SensitiveString = serde_json::from_str(json).unwrap();
        assert_eq!(secret.expose(), "my-secret-key");
    }

    #[test]
    fn test_sensitive_string_default() {
        let secret = SensitiveString::default();
        assert!(secret.is_empty());
        assert_eq!(secret.len(), 0);
    }

    #[test]
    fn test_sensitive_string_partial_eq() {
        let s1 = SensitiveString::new("key");
        let s2 = SensitiveString::new("key");
        let s3 = SensitiveString::new("different");
        assert_eq!(s1, s2);
        assert_ne!(s1, s3);
    }

    #[test]
    fn test_sensitive_string_partial_eq_no_leak() {
        // 验证 PartialEq 比较不会泄露敏感信息
        let s1 = SensitiveString::new("secret-key-123");
        let s2 = SensitiveString::new("secret-key-123");
        let s3 = SensitiveString::new("different-key");

        // 相等比较应该正常工作
        assert!(s1 == s2);
        assert!(s1 != s3);

        // 验证 Debug 输出不包含原始值
        let debug = format!("{:?}", s1);
        assert!(!debug.contains("secret-key-123"));

        // 验证 Display 输出不包含原始值
        let display = format!("{}", s1);
        assert!(!display.contains("secret-key-123"));

        // 只有通过 expose() 才能获取原始值
        assert_eq!(s1.expose(), "secret-key-123");
    }

    #[test]
    fn test_sensitive_string_from_str() {
        let s1: SensitiveString = "test-key".into();
        let s2 = SensitiveString::new("test-key");
        assert_eq!(s1, s2);
        assert_eq!(s1.expose(), "test-key");
    }

    #[test]
    fn test_sensitive_string_from_string() {
        let s1: SensitiveString = String::from("test-key").into();
        let s2 = SensitiveString::new("test-key");
        assert_eq!(s1, s2);
    }

    #[test]
    fn test_execution_plan_new() {
        let target = ExecutionTarget::new(
            "openai",
            Uuid::new_v4(),
            "https://api.openai.com",
            "sk-test-key",
        );
        let plan = ExecutionPlan::new(target);
        assert_eq!(plan.primary.provider, "openai");
        assert!(plan.fallback_chain.is_empty());
    }

    #[test]
    fn test_execution_plan_with_fallback() {
        let primary = ExecutionTarget::new(
            "openai",
            Uuid::new_v4(),
            "https://api.openai.com",
            "sk-primary-key",
        );
        let fallback = ExecutionTarget::new(
            "claude",
            Uuid::new_v4(),
            "https://api.anthropic.com",
            "sk-fallback-key",
        );
        let plan = ExecutionPlan::new(primary).with_fallback(fallback);
        assert_eq!(plan.fallback_chain.len(), 1);
        assert_eq!(plan.fallback_chain[0].provider, "claude");
    }

    #[test]
    fn test_execution_plan_all_targets() {
        let primary = ExecutionTarget::new(
            "openai",
            Uuid::new_v4(),
            "https://api.openai.com",
            "sk-primary-key",
        );
        let fallback1 = ExecutionTarget::new(
            "claude",
            Uuid::new_v4(),
            "https://api.anthropic.com",
            "sk-fallback1-key",
        );
        let fallback2 = ExecutionTarget::new(
            "gemini",
            Uuid::new_v4(),
            "https://api.gemini.com",
            "sk-fallback2-key",
        );
        let plan = ExecutionPlan::new(primary)
            .with_fallback(fallback1)
            .with_fallback(fallback2);

        let targets: Vec<_> = plan.all_targets().collect();
        assert_eq!(targets.len(), 3);
        assert_eq!(targets[0].provider, "openai");
        assert_eq!(targets[1].provider, "claude");
        assert_eq!(targets[2].provider, "gemini");
    }

    #[test]
    fn test_execution_target_api_key_hidden_in_debug() {
        let target = ExecutionTarget::new(
            "openai",
            Uuid::new_v4(),
            "https://api.openai.com",
            "sk-secret-key",
        );
        let debug_str = format!("{:?}", target);
        assert!(debug_str.contains("***REDACTED***"));
        assert!(!debug_str.contains("sk-secret-key"));
    }

    #[test]
    fn test_execution_target_api_key_hidden_in_serialize() {
        let target = ExecutionTarget::new(
            "openai",
            Uuid::new_v4(),
            "https://api.openai.com",
            "sk-secret-key",
        );
        let json = serde_json::to_string(&target).unwrap();
        assert!(json.contains("***REDACTED***"));
        assert!(!json.contains("sk-secret-key"));
    }

    #[test]
    fn test_execution_target_api_key_expose() {
        let target = ExecutionTarget::new(
            "openai",
            Uuid::new_v4(),
            "https://api.openai.com",
            "sk-secret-key",
        );
        // expose() 方法可以获取原始值（用于实际请求）
        assert_eq!(target.upstream_api_key.expose(), "sk-secret-key");
    }
}
