//! KeyCompute 数据库访问层
//!
//! 提供 PostgreSQL 数据库连接池、ORM 模型和迁移支持

pub mod models;
pub mod schema;

use sqlx::{PgPool, Postgres, migrate::Migrator, postgres::PgPoolOptions};
use std::time::Duration;

pub use models::*;
pub use schema::*;

// ============================================================================
// 错误类型定义
// ============================================================================

/// 数据库错误类型
#[derive(Debug, thiserror::Error)]
pub enum DbError {
    /// 连接错误
    #[error("database connection failed: {0}")]
    ConnectionError(String),

    /// 迁移错误
    #[error("migration failed: {0}")]
    MigrationError(String),

    /// 实体未找到
    #[error("{entity} not found: {id}")]
    NotFound { entity: String, id: String },

    /// 余额不足
    #[error("insufficient balance: required {required}, available {available}")]
    InsufficientBalance { required: String, available: String },

    /// 唯一约束冲突
    #[error("duplicate key: {entity} with {field}={value} already exists")]
    DuplicateKey {
        entity: String,
        field: String,
        value: String,
    },

    /// 订单状态无效
    #[error("invalid order status: expected {expected}, actual {actual}")]
    InvalidOrderStatus { expected: String, actual: String },

    /// 数据库原生错误
    #[error("database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    /// 其他错误
    #[error("{0}")]
    Other(String),
}

impl DbError {
    /// 创建 NotFound 错误
    pub fn not_found(entity: impl Into<String>, id: impl Into<String>) -> Self {
        Self::NotFound {
            entity: entity.into(),
            id: id.into(),
        }
    }

    /// 创建 InsufficientBalance 错误
    pub fn insufficient_balance(required: impl Into<String>, available: impl Into<String>) -> Self {
        Self::InsufficientBalance {
            required: required.into(),
            available: available.into(),
        }
    }

    /// 创建 DuplicateKey 错误
    pub fn duplicate_key(
        entity: impl Into<String>,
        field: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        Self::DuplicateKey {
            entity: entity.into(),
            field: field.into(),
            value: value.into(),
        }
    }

    /// 检查是否为未找到错误
    pub fn is_not_found(&self) -> bool {
        matches!(self, Self::NotFound { .. })
    }

    /// 检查是否为余额不足错误
    pub fn is_insufficient_balance(&self) -> bool {
        matches!(self, Self::InsufficientBalance { .. })
    }

    /// 检查是否为唯一约束冲突
    pub fn is_duplicate(&self) -> bool {
        matches!(self, Self::DuplicateKey { .. })
            || matches!(self, Self::DatabaseError(sqlx::Error::Database(e)) if e.constraint().is_some())
    }

    /// 从 sqlx::Error 转换，保留语义
    pub fn from_sqlx(err: sqlx::Error, entity: &str, id: &str) -> Self {
        match err {
            sqlx::Error::RowNotFound => Self::NotFound {
                entity: entity.to_string(),
                id: id.to_string(),
            },
            sqlx::Error::Database(ref e) if e.constraint().is_some() => Self::DuplicateKey {
                entity: e.table().unwrap_or(entity).to_string(),
                field: e.constraint().unwrap_or("unknown").to_string(),
                value: id.to_string(),
            },
            other => Self::DatabaseError(other),
        }
    }
}

// ============================================================================
// 数据库配置
// ============================================================================

/// 数据库配置
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    /// 数据库连接 URL
    pub url: String,
    /// 最大连接数
    pub max_connections: u32,
    /// 最小连接数
    pub min_connections: u32,
    /// 连接超时时间（秒）
    pub connect_timeout: u64,
    /// 连接空闲超时时间（秒）
    pub idle_timeout: u64,
    /// 连接最大生命周期（秒）
    pub max_lifetime: u64,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://localhost/keycompute".to_string()),
            max_connections: 10,
            min_connections: 2,
            connect_timeout: 30,
            idle_timeout: 600,
            max_lifetime: 1800,
        }
    }
}

// ============================================================================
// 连接池管理
// ============================================================================

/// 初始化数据库连接池
///
/// # Examples
///
/// ```rust,no_run
/// use keycompute_db::{init_pool, DatabaseConfig};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let config = DatabaseConfig::default();
///     let pool = init_pool(&config).await?;
///     Ok(())
/// }
/// ```
pub async fn init_pool(config: &DatabaseConfig) -> Result<PgPool, DbError> {
    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .acquire_timeout(Duration::from_secs(config.connect_timeout))
        .idle_timeout(Duration::from_secs(config.idle_timeout))
        .max_lifetime(Duration::from_secs(config.max_lifetime))
        .connect(&config.url)
        .await
        .map_err(|e| DbError::ConnectionError(e.to_string()))?;

    tracing::info!("Database pool initialized successfully");

    Ok(pool)
}

/// 运行数据库迁移
///
/// 使用 sqlx 的嵌入式迁移
pub async fn run_migrations(pool: &PgPool) -> Result<(), DbError> {
    // 嵌入式迁移文件
    static MIGRATOR: Migrator = sqlx::migrate!("src/migrations");

    MIGRATOR
        .run(pool)
        .await
        .map_err(|e| DbError::MigrationError(e.to_string()))?;

    tracing::info!("Database migrations completed successfully");

    Ok(())
}

// ============================================================================
// 数据库管理器
// ============================================================================

use sqlx::{PgConnection, Transaction};

/// 数据库管理器
///
/// 封装数据库连接池，提供统一的数据库访问入口
#[derive(Clone)]
pub struct Database {
    pool: PgPool,
}

impl Database {
    /// 创建新的数据库实例
    pub async fn new(config: &DatabaseConfig) -> Result<Self, DbError> {
        let pool = init_pool(config).await?;
        Ok(Self { pool })
    }

    /// 从现有连接池创建
    pub fn from_pool(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 从环境变量创建
    pub async fn from_env() -> Result<Self, DbError> {
        let config = DatabaseConfig::default();
        Self::new(&config).await
    }

    /// 获取连接池引用
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// 获取连接池（消费）
    pub fn into_pool(self) -> PgPool {
        self.pool
    }

    /// 开始一个事务
    pub async fn begin(&self) -> Result<Transaction<'_, Postgres>, sqlx::Error> {
        self.pool.begin().await
    }

    /// 获取一个连接
    pub async fn acquire(&self) -> Result<PgConnection, sqlx::Error> {
        self.pool.acquire().await.map(|c| c.detach())
    }

    /// 运行迁移
    pub async fn migrate(&self) -> Result<(), DbError> {
        run_migrations(&self.pool).await
    }

    /// 测试连接
    pub async fn test_connection(&self) -> Result<(), DbError> {
        sqlx::query("SELECT 1").fetch_one(&self.pool).await?;
        Ok(())
    }

    /// 获取连接池状态信息
    pub fn pool_status(&self) -> PoolStatus {
        PoolStatus {
            size: self.pool.size(),
            idle: self.pool.num_idle() as u32,
            is_closed: self.pool.is_closed(),
        }
    }
}

/// 连接池状态
#[derive(Debug, Clone)]
pub struct PoolStatus {
    /// 连接池大小
    pub size: u32,
    /// 空闲连接数
    pub idle: u32,
    /// 是否已关闭
    pub is_closed: bool,
}

/// 数据库连接管理器（已弃用，使用 Database）
#[deprecated(since = "0.2.0", note = "Use `Database` instead")]
pub type DatabaseManager = Database;

/// 重新导出 sqlx 类型
pub use sqlx::{PgConnection as SqlxPgConnection, PgPool as SqlxPgPool, Row};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_config_default() {
        let config = DatabaseConfig::default();
        assert_eq!(config.max_connections, 10);
        assert_eq!(config.min_connections, 2);
    }

    #[test]
    fn test_db_error_helpers() {
        let err = DbError::not_found("User", "123");
        assert!(err.is_not_found());
        assert!(err.to_string().contains("User not found"));

        let err = DbError::insufficient_balance("100", "50");
        assert!(err.is_insufficient_balance());
        assert!(err.to_string().contains("insufficient balance"));

        let err = DbError::duplicate_key("User", "email", "test@example.com");
        assert!(err.is_duplicate());
    }

    #[test]
    fn test_db_error_from_sqlx() {
        let err = DbError::from_sqlx(sqlx::Error::RowNotFound, "User", "123");
        assert!(err.is_not_found());
    }
}
