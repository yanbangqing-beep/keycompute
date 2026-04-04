//! Runtime Core Layer
//!
//! 运行时核心层，提供加密和存储抽象。
//! 注意：Provider 健康状态和账号状态已移至 routing 模块。

pub mod crypto;
pub mod store;

#[cfg(feature = "redis")]
pub mod redis_store;

pub use crypto::{
    ApiKeyCrypto, CryptoError, EncryptedApiKey, decrypt_api_key, encrypt_api_key, global_crypto,
    set_global_crypto,
};
pub use store::RuntimeStore;

#[cfg(feature = "redis")]
pub use redis_store::RedisRuntimeStore;

use std::sync::Arc;

/// 运行时存储后端类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeBackend {
    /// 内存后端
    Memory,
    /// Redis 后端
    Redis,
}

/// 运行时核心管理器
///
/// 提供加密和底层存储功能。
/// 注意：Provider 健康状态和账号状态已移至 routing 模块。
#[derive(Debug, Clone)]
pub struct RuntimeManager {
    /// 存储后端类型
    backend: RuntimeBackend,
}

impl RuntimeManager {
    /// 创建新的运行时管理器（内存后端）
    pub fn new() -> Self {
        Self {
            backend: RuntimeBackend::Memory,
        }
    }

    /// 获取存储后端类型
    pub fn backend(&self) -> RuntimeBackend {
        self.backend
    }
}

impl Default for RuntimeManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "redis")]
impl RuntimeManager {
    /// 创建带 Redis 后端的运行时管理器
    ///
    /// # 参数
    /// - `redis_url`: Redis 连接 URL
    pub fn new_redis(redis_url: &str) -> Result<Self, redis::RedisError> {
        let _store = RedisRuntimeStore::new(redis_url)?;
        let _store = Arc::new(_store);

        Ok(Self {
            backend: RuntimeBackend::Redis,
        })
    }

    /// 创建带 Redis 后端的运行时管理器（带自定义前缀）
    pub fn new_redis_with_prefix(
        redis_url: &str,
        prefix: impl Into<String>,
    ) -> Result<Self, redis::RedisError> {
        let store = RedisRuntimeStore::with_prefix(redis_url, prefix)?;
        let _store = Arc::new(store);

        Ok(Self {
            backend: RuntimeBackend::Redis,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_manager_new() {
        let manager = RuntimeManager::new();
        assert_eq!(manager.backend(), RuntimeBackend::Memory);
    }
}
