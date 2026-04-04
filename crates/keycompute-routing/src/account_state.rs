//! 账号状态管理
//!
//! 管理账号的冷却状态和错误计数（用于管理员调试）。
//! Gateway 写入，Routing 只读。
//!
//! 注意：冷却机制为纯手动触发，只能通过 API 设置冷却状态。
//! 已移除 RPM 负载均衡逻辑。

use dashmap::DashMap;
use std::time::{Duration, Instant};
use uuid::Uuid;

/// 账号状态
///
/// 简化的账号运行时状态，仅保留冷却标记和调试用的错误计数。
#[derive(Debug, Clone, Default)]
pub struct AccountState {
    /// 冷却直到时间
    pub cooldown_until: Option<Instant>,
    /// 错误计数（仅用于管理员调试测试）
    pub error_count: u32,
    /// 最后一次错误时间（仅用于管理员调试测试）
    pub last_error_at: Option<Instant>,
}

impl AccountState {
    /// 创建新的账号状态
    pub fn new() -> Self {
        Self::default()
    }

    /// 检查是否在冷却中
    pub fn is_cooling_down(&self) -> bool {
        self.cooldown_until
            .map(|t| t > Instant::now())
            .unwrap_or(false)
    }

    /// 获取剩余冷却时间
    pub fn cooldown_remaining(&self) -> Option<Duration> {
        self.cooldown_until.map(|t| {
            let now = Instant::now();
            if t > now {
                t - now
            } else {
                Duration::from_secs(0)
            }
        })
    }
}

/// 账号状态存储
///
/// 使用 DashMap 实现并发安全的读写。
/// 仅管理手动冷却状态和调试用的错误计数。
#[derive(Debug)]
pub struct AccountStateStore {
    states: DashMap<Uuid, AccountState>,
}

impl Default for AccountStateStore {
    fn default() -> Self {
        Self::new()
    }
}

impl AccountStateStore {
    /// 创建新的账号状态存储
    pub fn new() -> Self {
        Self {
            states: DashMap::new(),
        }
    }

    /// 手动设置账号冷却状态
    ///
    /// 通过 API 调用触发，立即进入冷却状态
    pub fn set_cooldown(&self, account_id: Uuid, duration_secs: u64) {
        let duration = Duration::from_secs(duration_secs);
        self.states
            .entry(account_id)
            .and_modify(|state| {
                state.cooldown_until = Some(Instant::now() + duration);
                tracing::info!(
                    account_id = %account_id,
                    duration_secs = duration_secs,
                    "Account manually entered cooldown state"
                );
            })
            .or_insert_with(|| {
                let mut state = AccountState::new();
                state.cooldown_until = Some(Instant::now() + duration);
                state
            });
    }

    /// 清除账号冷却状态
    pub fn clear_cooldown(&self, account_id: Uuid) {
        self.states.entry(account_id).and_modify(|state| {
            state.cooldown_until = None;
            state.error_count = 0;
            tracing::info!(account_id = %account_id, "Account cooldown cleared");
        });
    }

    /// Gateway 调用：标记成功
    ///
    /// 清除冷却状态和错误计数
    pub fn mark_success(&self, account_id: Uuid) {
        self.states
            .entry(account_id)
            .and_modify(|state| {
                if state.error_count > 0 {
                    tracing::info!(
                        account_id = %account_id,
                        previous_errors = state.error_count,
                        "Account error count reset after success"
                    );
                }
                state.error_count = 0;
                state.cooldown_until = None;
                state.last_error_at = None;
            })
            .or_default();
    }

    /// 管理员调试：标记错误（仅用于账号连接测试）
    ///
    /// 仅记录错误计数和时间，不触发冷却。
    pub fn mark_error(&self, account_id: Uuid) {
        self.states
            .entry(account_id)
            .and_modify(|state| {
                state.error_count += 1;
                state.last_error_at = Some(Instant::now());
            })
            .or_insert_with(|| {
                let mut state = AccountState::new();
                state.error_count = 1;
                state.last_error_at = Some(Instant::now());
                state
            });
    }

    /// Routing 调用：检查是否在冷却中
    pub fn is_cooling_down(&self, account_id: &Uuid) -> bool {
        self.states
            .get(account_id)
            .map(|s| s.is_cooling_down())
            .unwrap_or(false)
    }

    /// Routing 调用：获取账号状态
    pub fn get(&self, account_id: &Uuid) -> AccountState {
        self.states
            .get(account_id)
            .map(|s| s.clone())
            .unwrap_or_default()
    }

    /// Routing 调用：获取所有可用账号（未冷却）
    pub fn available_accounts(&self, account_ids: &[Uuid]) -> Vec<Uuid> {
        account_ids
            .iter()
            .filter(|id| !self.is_cooling_down(id))
            .copied()
            .collect()
    }

    /// 获取所有账号状态（用于监控）
    pub fn all_states(&self) -> Vec<(Uuid, AccountState)> {
        self.states
            .iter()
            .map(|entry| (*entry.key(), entry.value().clone()))
            .collect()
    }

    /// 获取所有在冷却中的账号
    pub fn cooling_accounts(&self) -> Vec<(Uuid, AccountState)> {
        self.states
            .iter()
            .filter(|entry| entry.value().is_cooling_down())
            .map(|entry| (*entry.key(), entry.value().clone()))
            .collect()
    }

    /// 清理过期的冷却状态（可由后台任务定期调用）
    pub fn cleanup_expired_cooldowns(&self) {
        let now = Instant::now();
        self.states.retain(|_id, state| {
            if let Some(cooldown) = state.cooldown_until
                && cooldown <= now
            {
                state.cooldown_until = None;
            }
            true // 保留所有条目
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_state_new() {
        let state = AccountState::new();
        assert_eq!(state.error_count, 0);
        assert!(state.last_error_at.is_none());
        assert!(state.cooldown_until.is_none());
        assert!(!state.is_cooling_down());
    }

    #[test]
    fn test_mark_error() {
        let store = AccountStateStore::new();
        let account_id = Uuid::new_v4();

        // 第一次错误（仅用于调试）
        store.mark_error(account_id);
        let state = store.get(&account_id);
        assert_eq!(state.error_count, 1);
        // 错误不会自动触发冷却
        assert!(!state.is_cooling_down());

        // 第二次错误
        store.mark_error(account_id);
        let state = store.get(&account_id);
        assert_eq!(state.error_count, 2);
        // 仍然不会自动冷却
        assert!(!state.is_cooling_down());

        // 手动设置冷却
        store.set_cooldown(account_id, 60);
        let state = store.get(&account_id);
        assert!(state.is_cooling_down());
    }

    #[test]
    fn test_mark_success() {
        let store = AccountStateStore::new();
        let account_id = Uuid::new_v4();

        // 标记错误
        store.mark_error(account_id);
        assert!(!store.is_cooling_down(&account_id));

        // 标记成功清除错误和冷却
        store.mark_success(account_id);
        let state = store.get(&account_id);
        assert_eq!(state.error_count, 0);
        assert!(!state.is_cooling_down());
    }

    #[test]
    fn test_set_cooldown() {
        let store = AccountStateStore::new();
        let account_id = Uuid::new_v4();

        // 初始状态不在冷却中
        assert!(!store.is_cooling_down(&account_id));

        // 设置冷却
        store.set_cooldown(account_id, 60);
        let state = store.get(&account_id);
        assert!(state.is_cooling_down());
        assert!(state.cooldown_remaining().is_some());
    }

    #[test]
    fn test_available_accounts() {
        let store = AccountStateStore::new();
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();

        // 手动设置 id1 冷却
        store.set_cooldown(id1, 60);

        // 验证 id1 确实在冷却中
        assert!(store.is_cooling_down(&id1), "id1 should be cooling down");

        let available = store.available_accounts(&[id1, id2]);
        assert_eq!(available.len(), 1);
        assert_eq!(available[0], id2);
    }

    #[test]
    fn test_cooling_accounts() {
        let store = AccountStateStore::new();
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();

        // 设置 id1 冷却
        store.set_cooldown(id1, 60);

        // 获取冷却中的账号列表
        let cooling = store.cooling_accounts();
        assert_eq!(cooling.len(), 1);
        assert_eq!(cooling[0].0, id1);

        // id2 不在冷却中
        assert!(!store.is_cooling_down(&id2));
    }
}
