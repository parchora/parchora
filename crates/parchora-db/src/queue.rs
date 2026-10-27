//! `queue_entries` 表 CRUD（DEC-212 · 承诺式队列跨会话持久 · 每动作事务即写）。
//!
//! 形态口径：DB 侧自有行投影（uuid/text/ord/created_at 皆 String/i64）——
//! capture 内存类型（`u64` id）不入 db 接口，uuid ↔ 内存 id 映射由 app 层
//! QueueHost 承担。读序以 `ord` 升序（入队序）为唯一权威，FIFO/LIFO 消费序
//! 由设置档在 app 层裁定。错误上浮不吞——app 层以「披露 + 内存队列继续」
//! 兜底（不自伤）。

use rusqlite::params;

use crate::error::DbError;
use crate::store::Db;

/// 队列条目行（读路径投影）。
#[derive(Debug, Clone, PartialEq)]
pub struct QueueEntryRow {
    pub uuid: String,
    pub text: String,
    pub ord: i64,
    pub created_at: String,
}

/// 单条事务写（入队动作即写；uuid 主键防重复入表）。
pub fn insert_queue_entry(
    db: &Db,
    uuid: &str,
    text: &str,
    ord: i64,
    created_at: &str,
) -> Result<(), DbError> {
    db.with_tx(|tx| {
        tx.execute(
            "INSERT INTO queue_entries (uuid, text, ord, created_at) VALUES (?1,?2,?3,?4)",
            params![uuid, text, ord, created_at],
        )?;
        Ok(())
    })
}

/// 单条删除（消费动作即写）。返回删除行数（0 = 已不在表，幂等）。
pub fn delete_queue_entry(db: &Db, uuid: &str) -> Result<usize, DbError> {
    db.with_tx(|tx| tx.execute("DELETE FROM queue_entries WHERE uuid = ?1", params![uuid]))
}

/// 全清（Stop/Clear 收口动作即写）。返回删除行数。
pub fn clear_queue_entries(db: &Db) -> Result<usize, DbError> {
    db.with_tx(|tx| tx.execute("DELETE FROM queue_entries", []))
}

/// 全量读取（`ord` 升序＝入队序权威 · rowid 平手兜底；启动恢复唯一入口）。
pub fn list_queue_entries(db: &Db) -> Result<Vec<QueueEntryRow>, DbError> {
    db.with_tx(|tx| {
        let mut stmt = tx.prepare(
            "SELECT uuid, text, ord, created_at \
             FROM queue_entries ORDER BY ord ASC, rowid ASC",
        )?;
        let rows = stmt
            .query_map([], |r| {
                Ok(QueueEntryRow {
                    uuid: r.get(0)?,
                    text: r.get(1)?,
                    ord: r.get(2)?,
                    created_at: r.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>, rusqlite::Error>>()?;
        Ok(rows)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn temp_db() -> Arc<Db> {
        // 目录名唯一性三级（同 clips.rs 测试口径：pid + 纳秒 + 进程内原子序号）。
        static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let seq = SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!(
            "parchora-db-queue-test-{}-{}-{}",
            std::process::id(),
            seq,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        std::fs::create_dir_all(&dir).unwrap();
        Db::open(&dir).unwrap()
    }

    #[test]
    fn insert_list_roundtrip_ordered_by_ord() {
        let db = temp_db();
        // 乱序 ord 写入，读回必须按 ord 升序（入队序权威）。
        insert_queue_entry(&db, "u-3", "丙", 3, "2026-10-23T10:00:00Z").unwrap();
        insert_queue_entry(&db, "u-1", "甲", 1, "2026-10-23T09:58:00Z").unwrap();
        insert_queue_entry(&db, "u-2", "乙", 2, "2026-10-23T09:59:00Z").unwrap();
        let rows = list_queue_entries(&db).unwrap();
        assert_eq!(
            rows,
            vec![
                QueueEntryRow {
                    uuid: "u-1".into(),
                    text: "甲".into(),
                    ord: 1,
                    created_at: "2026-10-23T09:58:00Z".into(),
                },
                QueueEntryRow {
                    uuid: "u-2".into(),
                    text: "乙".into(),
                    ord: 2,
                    created_at: "2026-10-23T09:59:00Z".into(),
                },
                QueueEntryRow {
                    uuid: "u-3".into(),
                    text: "丙".into(),
                    ord: 3,
                    created_at: "2026-10-23T10:00:00Z".into(),
                },
            ],
            "读回按 ord 升序、字段逐列一致"
        );
    }

    #[test]
    fn delete_single_row_is_idempotent() {
        let db = temp_db();
        insert_queue_entry(&db, "u-1", "甲", 1, "2026-10-23T09:58:00Z").unwrap();
        insert_queue_entry(&db, "u-2", "乙", 2, "2026-10-23T09:59:00Z").unwrap();
        assert_eq!(delete_queue_entry(&db, "u-1").unwrap(), 1);
        let rest = list_queue_entries(&db).unwrap();
        assert_eq!(rest.len(), 1);
        assert_eq!(rest[0].uuid, "u-2");
        // 幂等：再删同 uuid 计 0。
        assert_eq!(delete_queue_entry(&db, "u-1").unwrap(), 0);
    }

    #[test]
    fn clear_removes_all_and_counts() {
        let db = temp_db();
        insert_queue_entry(&db, "u-1", "甲", 1, "2026-10-23T09:58:00Z").unwrap();
        insert_queue_entry(&db, "u-2", "乙", 2, "2026-10-23T09:59:00Z").unwrap();
        assert_eq!(clear_queue_entries(&db).unwrap(), 2);
        assert!(list_queue_entries(&db).unwrap().is_empty());
        // 幂等：空表再清计 0。
        assert_eq!(clear_queue_entries(&db).unwrap(), 0);
    }

    #[test]
    fn duplicate_uuid_surfaces_not_swallowed() {
        // 主键冲突（同 uuid 重插）→ 错误上浮，不静默。
        let db = temp_db();
        insert_queue_entry(&db, "u-1", "甲", 1, "2026-10-23T09:58:00Z").unwrap();
        assert!(insert_queue_entry(&db, "u-1", "重插", 2, "2026-10-23T09:59:00Z").is_err());
    }
}
