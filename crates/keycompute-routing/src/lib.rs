//! Routing Engine
//!
//! 路由引擎，双层路由，只读无副作用。
//! 架构约束：只读 Pricing 和 Runtime 状态快照，不写任何状态。

use keycompute_runtime::AccountStateStore;
use keycompute_types::{ExecutionPlan, ExecutionTarget, KeyComputeError, PricingSnapshot, RequestContext, Result};
use std::sync::Arc;
use uuid::Uuid;

/// 路由引擎
///
/// 双层路由：Layer1 模型路由，Layer2 账号路由
#[derive(Debug, Clone)]
pub struct RoutingEngine {
    /// 账号状态存储（只读）
    account_states: Arc<AccountStateStore>,
    /// 可用 Provider 列表
    providers: Vec<String>,
}

impl RoutingEngine {
    /// 创建新的路由引擎
    pub fn new(account_states: Arc<AccountStateStore>) -> Self {
        Self {
            account_states,
            providers: vec![
                "openai".to_string(),
                "claude".to_string(),
                "deepseek".to_string(),
            ],
        }
    }

    /// 生成执行计划（只读操作）
    ///
    /// 根据 RequestContext 路由到最优的 Provider 和账号
    pub async fn route(&self, ctx: &RequestContext) -> Result<ExecutionPlan> {
        // Layer1: 模型路由 - 选择 provider 排序
        let ranked_providers = self.rank_providers(&ctx.model, &ctx.pricing_snapshot).await?;

        // Layer2: 账号路由 - 为每个 provider 选择最优账号
        let mut targets = Vec::new();
        for provider in ranked_providers {
            if let Some(target) = self.select_account(&provider).await? {
                targets.push(target);
            }
        }

        if targets.is_empty() {
            return Err(KeyComputeError::RoutingFailed);
        }

        Ok(ExecutionPlan {
            primary: targets.remove(0),
            fallback_chain: targets,
        })
    }

    /// Layer1: 模型路由
    ///
    /// 根据模型、价格、延迟、成功率对 Provider 排序
    /// score = 0.5*cost + 0.3*latency - 0.2*success
    async fn rank_providers(
        &self,
        _model: &str,
        pricing: &PricingSnapshot,
    ) -> Result<Vec<String>> {
        // 简化实现：基于价格排序
        let mut scored_providers: Vec<(String, f64)> = self
            .providers
            .iter()
            .map(|p| {
                let score = self.score_provider(p, pricing);
                (p.clone(), score)
            })
            .collect();

        // 按分数排序（分数越低越好）
        scored_providers.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        Ok(scored_providers.into_iter().map(|(p, _)| p).collect())
    }

    /// 计算 Provider 评分
    fn score_provider(&self, provider: &str, pricing: &PricingSnapshot) -> f64 {
        // 基础分数
        let mut score = 50.0;

        // 根据 Provider 调整分数
        match provider {
            "openai" => score -= 10.0, // OpenAI 优先级稍高
            "deepseek" => score -= 5.0,
            _ => {}
        }

        // 价格因素（价格越低分数越好）
        let price_factor: f64 = pricing.input_price_per_1k.to_string().parse().unwrap_or(1.0);
        score += price_factor * 10.0;

        score
    }

    /// Layer2: 账号路由
    ///
    /// 为指定 Provider 选择最优账号
    /// score = current_rpm/rpm_limit + error_rate*2
    async fn select_account(&self, provider: &str) -> Result<Option<ExecutionTarget>> {
        // TODO: 从数据库加载该 Provider 的可用账号
        // 这里简化处理，返回模拟数据

        let account_id = Uuid::new_v4();

        // 检查账号是否在冷却中
        if self.account_states.is_cooling_down(&account_id) {
            tracing::debug!(provider = %provider, "Account is cooling down, skipping");
            return Ok(None);
        }

        // 构建执行目标
        let target = ExecutionTarget {
            provider: provider.to_string(),
            account_id,
            endpoint: format!("https://api.{}.com/v1/chat/completions", provider),
            api_key: "mock-api-key".to_string(),
        };

        Ok(Some(target))
    }

    /// 获取健康 Provider 列表
    pub fn healthy_providers(&self) -> &[String] {
        &self.providers
    }

    /// 添加 Provider
    pub fn add_provider(&mut self, provider: impl Into<String>) {
        self.providers.push(provider.into());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use keycompute_types::PricingSnapshot;
    use rust_decimal::Decimal;

    fn create_test_context() -> RequestContext {
        RequestContext {
            request_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            api_key_id: Uuid::new_v4(),
            model: "gpt-4o".to_string(),
            messages: vec![],
            stream: true,
            pricing_snapshot: PricingSnapshot {
                model_name: "gpt-4o".to_string(),
                currency: "CNY".to_string(),
                input_price_per_1k: Decimal::from(1),
                output_price_per_1k: Decimal::from(2),
            },
            usage: Default::default(),
            started_at: chrono::Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_routing_engine_new() {
        let account_states = Arc::new(AccountStateStore::new());
        let engine = RoutingEngine::new(account_states);

        assert_eq!(engine.healthy_providers().len(), 3);
    }

    #[tokio::test]
    async fn test_route() {
        let account_states = Arc::new(AccountStateStore::new());
        let engine = RoutingEngine::new(account_states);
        let ctx = create_test_context();

        let plan = engine.route(&ctx).await;
        assert!(plan.is_ok());

        let plan = plan.unwrap();
        assert!(!plan.primary.provider.is_empty());
    }

    #[test]
    fn test_score_provider() {
        let account_states = Arc::new(AccountStateStore::new());
        let engine = RoutingEngine::new(account_states);
        let pricing = PricingSnapshot {
            model_name: "test".to_string(),
            currency: "CNY".to_string(),
            input_price_per_1k: Decimal::from(1),
            output_price_per_1k: Decimal::from(2),
        };

        let openai_score = engine.score_provider("openai", &pricing);
        let other_score = engine.score_provider("other", &pricing);

        // OpenAI 应该有更低（更好）的分数
        assert!(openai_score < other_score);
    }
}
