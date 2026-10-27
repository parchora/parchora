//! DB 错误类型（fail-closed：版本超前拒绝启动，design §3.2「版本迁移」）。

use core::fmt;

/// parchora-db 统一错误。
///
/// 口径（DEC-89）：`SchemaTooNew` 为 fail-closed 闸门——库版本高于当前支持版本时
/// 整体拒绝打开，禁止「降级也能跑」的双口径（沿 sidecar 语义：reject「应用过旧」）。
#[derive(Debug)]
pub enum DbError {
    /// 库内 `user_version` 高于本 crate 支持版本（应用过旧，拒绝启动）。
    SchemaTooNew {
        /// 库内实际版本
        db_version: i64,
        /// 本 crate 支持的最高版本
        supported: i64,
    },
    /// SQLite 打开 / 查询 / 约束失败（含迁移降维承载，见 `From`）
    Open(rusqlite::Error),
    /// 文件系统操作失败（`data/` 目录创建等）
    Io(std::io::Error),
    /// `RootStore` 无活跃根时请求连接 / 事务
    NoActiveRoot,
    /// Mutex 中毒（持锁线程 panic）——fail-closed 透传，禁止静默重建连接掩盖现场
    Poisoned,
}

impl fmt::Display for DbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DbError::SchemaTooNew {
                db_version,
                supported,
            } => write!(
                f,
                "数据库 schema 版本 {db_version} 高于本应用支持的版本 {supported}（应用过旧，已拒绝打开）"
            ),
            DbError::Open(e) => write!(f, "SQLite 操作失败: {e}"),
            DbError::Io(e) => write!(f, "DB 文件系统操作失败: {e}"),
            DbError::NoActiveRoot => write!(f, "无活跃存储根连接（switch_to 后方可使用）"),
            DbError::Poisoned => write!(f, "DB 互斥锁中毒（内部并发异常，禁止静默重建）"),
        }
    }
}

impl std::error::Error for DbError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            DbError::Open(e) => Some(e),
            DbError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<rusqlite::Error> for DbError {
    fn from(e: rusqlite::Error) -> Self {
        DbError::Open(e)
    }
}

impl From<std::io::Error> for DbError {
    fn from(e: std::io::Error) -> Self {
        DbError::Io(e)
    }
}

impl From<rusqlite_migration::Error> for DbError {
    fn from(e: rusqlite_migration::Error) -> Self {
        // 迁移器错误不单设变体（最小 API 面）：统一降维为 `Open` 承载——文本完整保留，
        // 禁止吞错（§7.7.1：错误不吞）。rusqlite 无 Message 型通用变体，
        // 用通用码 SQLITE_ERROR + 全文承载。
        DbError::Open(rusqlite::Error::SqliteFailure(
            rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_ERROR),
            Some(e.to_string()),
        ))
    }
}
