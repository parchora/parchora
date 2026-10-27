//! `clips` 表 CRUD（TASK-300 · REQ-CAPTURE-PERSIST 每命中事务写 · design §3.2 投影）。
//!
//! 形态口径：core::content::Clip 即投影唯一形态源；DB 列序/名与 schema v1 DDL
//! 逐列对账。读路径脏值（content_type 反解失败）fail-soft 跳过该行（不虚构、
//! 不中断回填；跳过面在 app 层无感——历史脏行本就不可展示）。读写统一走
//! `Db::with_tx`（单写者事务封装 · §2.6）。

use rusqlite::params;

use crate::error::DbError;
use crate::store::Db;
use parchora_core::content::{Clip, ContentType, Uuid};

/// 单条事务写（REQ-CAPTURE-PERSIST：每次捕获命中即写，崩溃/重启仍在）。
pub fn insert_clip(db: &Db, clip: &Clip) -> Result<(), DbError> {
    db.with_tx(|tx| {
        tx.execute(
            "INSERT INTO clips (id, content_type, text_content, content_hash, source_app, \
             size, is_pinned, blob_rel_path, thumb_rel_path, edited_at, created_at) \
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
            params![
                clip.id.to_string(),
                clip.content_type.as_db_value(),
                clip.text_content,
                clip.content_hash,
                clip.source_app,
                clip.size,
                clip.is_pinned,
                clip.blob_rel_path,
                clip.thumb_rel_path,
                clip.edited_at,
                clip.created_at,
            ],
        )?;
        Ok(())
    })
}

/// 全量读取（created_at 倒序 · rowid 倒序平手兜底——同秒多条写入的稳定次序；
/// 启动回填唯一入口）。
pub fn list_clips(db: &Db) -> Result<Vec<Clip>, DbError> {
    let list: Vec<Clip> = db.with_tx(|tx| {
        let mut stmt = tx.prepare(
            "SELECT content_type, text_content, content_hash, source_app, size, \
                        is_pinned, blob_rel_path, thumb_rel_path, edited_at, created_at, id \
                        FROM clips ORDER BY created_at DESC, rowid DESC",
        )?;
        let rows = stmt
            .query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, Option<String>>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, Option<String>>(3)?,
                    r.get::<_, i64>(4)?,
                    r.get::<_, i64>(5)?,
                    r.get::<_, Option<String>>(6)?,
                    r.get::<_, Option<String>>(7)?,
                    r.get::<_, Option<String>>(8)?,
                    r.get::<_, String>(9)?,
                    r.get::<_, String>(10)?,
                ))
            })?
            .collect::<Result<Vec<_>, rusqlite::Error>>()?;
        // 脏行过滤（content_type 反解失败 / id 非法 → 跳过）在闭包内续做——
        // with_tx 形态保持 rusqlite::Result，过滤不触错误路径。
        let mapped = rows
            .into_iter()
            .filter_map(
                |(ct, text, hash, app, size, pinned, blob, thumb, edited, created, id)| {
                    let content_type = ContentType::parse_db_value(&ct)?;
                    Some(Clip {
                        id: Uuid::parse_str(&id).ok()?,
                        content_type,
                        text_content: text,
                        content_hash: hash,
                        source_app: app,
                        size,
                        is_pinned: pinned != 0,
                        blob_rel_path: blob,
                        thumb_rel_path: thumb,
                        edited_at: edited,
                        created_at: created,
                    })
                },
            )
            .collect();
        Ok(mapped)
    })?;
    Ok(list)
}

/// 未 Pin 存量按墙钟截止清理（REQ-CAPTURE-LIMIT 精神 · DEC-163 存活阈值轴）：
/// `created_at < cutoff_iso` 且 is_pinned=0 的事务删除。返回删除行数。
pub fn delete_unpinned_before(db: &Db, cutoff_iso: &str) -> Result<usize, DbError> {
    db.with_tx(|tx| {
        tx.execute(
            "DELETE FROM clips WHERE is_pinned = 0 AND created_at < ?1",
            params![cutoff_iso],
        )
    })
}

/// 单条删除（拾记右键「删除」· REQ-CAPTURE-PERSIST 逆向动作）。
pub fn delete_clip(db: &Db, id: &Uuid) -> Result<usize, DbError> {
    db.with_tx(|tx| tx.execute("DELETE FROM clips WHERE id = ?1", params![id.to_string()]))
}

/// 未 Pin 存量按计数上限裁剪（REQ-CAPTURE-LIMIT · design §3.2 技术口径「按
/// created_at 保留最新 N 条非 Pin」）：is_pinned=0 中 `created_at DESC, rowid DESC`
/// 序保留最新 keep 条，其余事务删除（Pin 恒豁免；rowid 平手兜底与 list 读序
/// 同源）。返回删除行数。
pub fn delete_unpinned_over_limit(db: &Db, keep: usize) -> Result<usize, DbError> {
    db.with_tx(|tx| {
        tx.execute(
            "DELETE FROM clips WHERE is_pinned = 0 AND id NOT IN (\
             SELECT id FROM clips WHERE is_pinned = 0 \
             ORDER BY created_at DESC, rowid DESC LIMIT ?1)",
            params![keep as i64],
        )
    })
}

/// 判重播种读（跨会话 10s 窗回填）：窗起点后入列条目的 (content_hash, created_at)
/// 全集（app 层按墙钟年龄折算后播种 Instant）。
pub fn recent_clip_hashes(db: &Db, since_iso: &str) -> Result<Vec<(String, String)>, DbError> {
    db.with_tx(|tx| {
        let mut stmt = tx.prepare(
            "SELECT content_hash, created_at FROM clips \
             WHERE created_at >= ?1 ORDER BY created_at DESC",
        )?;
        let rows = stmt
            .query_map(params![since_iso], |r| Ok((r.get(0)?, r.get(1)?)))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    })
}

/// blob 相对路径引用集读（blob 差集清理 · DEC-189 · design §2.6 blob 与条目同
/// 生命周期）：blob_rel_path / thumb_rel_path 两列非 NULL 全集（raw 与 thumb
/// 同收；行存活即路径受引用——本读即 DB 侧引用事实唯一源）。
pub fn list_blob_rel_paths(db: &Db) -> Result<Vec<String>, DbError> {
    db.with_tx(|tx| {
        let mut stmt = tx.prepare(
            "SELECT blob_rel_path FROM clips WHERE blob_rel_path IS NOT NULL \
             UNION ALL \
             SELECT thumb_rel_path FROM clips WHERE thumb_rel_path IS NOT NULL",
        )?;
        let rows = stmt
            .query_map([], |r| r.get::<_, String>(0))?
            .collect::<Result<Vec<_>, rusqlite::Error>>()?;
        Ok(rows)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::Db;
    use parchora_core::content::ContentType;
    use std::sync::Arc;

    fn fx_clip(hash: &str, text: Option<&str>, created: &str, pinned: bool) -> Clip {
        Clip {
            id: Uuid::new_v4(),
            content_type: ContentType::PlainText,
            text_content: text.map(String::from),
            content_hash: hash.into(),
            source_app: Some("com.example.source".into()),
            size: 42,
            is_pinned: pinned,
            blob_rel_path: None,
            thumb_rel_path: None,
            edited_at: None,
            created_at: created.into(),
        }
    }

    fn temp_db() -> Arc<Db> {
        // 目录名唯一性三级（DEC-189）：pid 与纳秒之外加进程内原子序号——纯纳秒
        // 拍在多测试线程同拍创建时存在碰撞窗口（实害：并发开同库报 DatabaseBusy
        // /"_meta already exists"）。
        static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let seq = SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!(
            "parchora-db-clips-test-{}-{}-{}",
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
    fn insert_and_list_roundtrip() {
        let db = temp_db();
        let a = fx_clip("hash-a", Some("甲甲甲"), "2026-10-21T08:00:00Z", false);
        insert_clip(&db, &a).unwrap();
        let b = fx_clip("hash-b", Some("乙乙乙"), "2026-10-21T09:00:00Z", true);
        insert_clip(&db, &b).unwrap();
        let listed = list_clips(&db).unwrap();
        assert_eq!(listed.len(), 2);
        assert_eq!(listed[0].content_hash, "hash-b", "created_at 倒序");
        assert_eq!(listed[0].id, b.id);
        assert!(listed[0].is_pinned);
        assert_eq!(listed[0].source_app.as_deref(), Some("com.example.source"));
        assert_eq!(listed[1].content_hash, "hash-a");
    }

    #[test]
    fn delete_by_id_and_unpinned_retention() {
        let db = temp_db();
        let old_unpinned = fx_clip("old", Some("旧"), "2026-10-01T00:00:00Z", false);
        let old_pinned = fx_clip("keep", Some("Pin 旧"), "2026-10-01T01:00:00Z", true);
        let fresh = fx_clip("new", Some("新"), "2026-10-21T10:00:00Z", false);
        for c in [&old_unpinned, &old_pinned, &fresh] {
            insert_clip(&db, c).unwrap();
        }
        // 截止在 Pin 行之前（保留）；未 Pin 旧行删除。
        let gone = delete_unpinned_before(&db, "2026-10-15T00:00:00Z").unwrap();
        assert_eq!(gone, 1);
        let listed = list_clips(&db).unwrap();
        assert_eq!(listed.len(), 2);
        assert!(listed.iter().any(|c| c.id == old_pinned.id));
        assert!(listed.iter().any(|c| c.id == fresh.id));
        // 单条删除（右键删除路径）。
        assert_eq!(delete_clip(&db, &fresh.id).unwrap(), 1);
        assert!(list_clips(&db).unwrap().iter().all(|c| c.id != fresh.id));
        // 幂等：再删同 id 计 0。
        assert_eq!(delete_clip(&db, &fresh.id).unwrap(), 0);
    }

    #[test]
    fn recent_hashes_window_read() {
        let db = temp_db();
        insert_clip(
            &db,
            &fx_clip("in-window", Some("窗内"), "2026-10-21T09:59:50Z", false),
        )
        .unwrap();
        insert_clip(
            &db,
            &fx_clip("out-window", Some("窗外"), "2026-10-21T08:00:00Z", false),
        )
        .unwrap();
        let rows = recent_clip_hashes(&db, "2026-10-21T09:59:40Z").unwrap();
        assert_eq!(
            rows,
            vec![("in-window".to_string(), "2026-10-21T09:59:50Z".to_string())]
        );
    }

    /// 引用集读（DEC-189 blob 差集清理）：两列非 NULL 全集，纯文本/None 行不入集。
    #[test]
    fn blob_rel_paths_read() {
        let db = temp_db();
        let mut a = fx_clip("a", Some("图片甲"), "2026-10-21T08:00:00Z", false);
        a.blob_rel_path = Some("blobs/id-a/raw".into());
        a.thumb_rel_path = Some("blobs/id-a/thumb".into());
        let b = fx_clip("b", Some("纯文本乙"), "2026-10-21T09:00:00Z", false);
        insert_clip(&db, &a).unwrap();
        insert_clip(&db, &b).unwrap();
        let mut refs = list_blob_rel_paths(&db).unwrap();
        refs.sort();
        assert_eq!(
            refs,
            vec!["blobs/id-a/raw".to_string(), "blobs/id-a/thumb".to_string()],
            "仅非 NULL 两列入集"
        );
    }

    /// 计数上限裁剪（REQ-CAPTURE-LIMIT · SCN-LIMIT-PIN-EXEMPT）：只裁未 Pin、
    /// 保序保留最新 N 条；Pin 超限不豁免计数口径不受影响（Pin 永不清理）。
    #[test]
    fn unpinned_over_limit_trims_oldest_keeps_pin() {
        let db = temp_db();
        // 4 条未 Pin（时间递增）＋ 1 条 Pin。
        let c1 = fx_clip("h1", Some("一"), "2026-10-21T08:00:00Z", false);
        let c2 = fx_clip("h2", Some("二"), "2026-10-21T09:00:00Z", false);
        let c3 = fx_clip("h3", Some("三"), "2026-10-21T10:00:00Z", false);
        let c4 = fx_clip("h4", Some("四"), "2026-10-21T11:00:00Z", false);
        let pin = fx_clip("hp", Some("Pin 标"), "2026-10-21T08:30:00Z", true);
        for c in [&c1, &c2, &c3, &c4, &pin] {
            insert_clip(&db, c).unwrap();
        }
        // 上限 2：保留最新两条（三/四）＋ Pin 豁免共 3 条。
        let gone = delete_unpinned_over_limit(&db, 2).unwrap();
        assert_eq!(gone, 2, "最旧两条未 Pin 被裁");
        let listed = list_clips(&db).unwrap();
        assert_eq!(listed.len(), 3);
        let ids: Vec<_> = listed.iter().map(|c| c.id).collect();
        assert!(ids.contains(&c3.id) && ids.contains(&c4.id), "最新两条保留");
        assert!(ids.contains(&pin.id), "Pin 条目恒豁免");
        // 上限 0 由调用侧语义承载（无限不触本口）；此处直接调用＝裁光。
        let gone = delete_unpinned_over_limit(&db, 0).unwrap();
        assert_eq!(gone, 2, "keep=0 应裁光全部未 Pin");
        assert!(list_clips(&db).unwrap().iter().all(|c| c.is_pinned));
    }

    #[test]
    fn dirty_content_type_rejected_by_check_hard_gate() {
        // 历史脏行防线双层：写层 CHECK 硬拦截（未知值无法入库）；读层
        // parse_db_value fail-soft 为纵深防御（值域与反解值域同源，正常不可达）。
        let db = temp_db();
        let err = db
            .with_tx(|tx| {
                tx.execute(
                    "INSERT INTO clips (id, content_type, text_content, content_hash, source_app, \
                     size, is_pinned, blob_rel_path, thumb_rel_path, edited_at, created_at) \
                     VALUES (?1,'dirtyType',?2,'dirt',NULL,0,0,NULL,NULL,NULL,'2026-10-21T09:00:00Z')",
                    params![Uuid::new_v4().to_string(), "脏行"],
                )?;
                Ok(())
            })
            .unwrap_err();
        assert!(matches!(err, DbError::Open(_)), "CHECK 拒写上浮: {err}");
        assert!(list_clips(&db).unwrap().is_empty(), "脏行未入库");
    }

    #[test]
    fn constraint_violations_surface_not_swallowed() {
        // 主键冲突（同 id 重插）→ 错误上浮，不静默。
        let db = temp_db();
        let a = fx_clip("dup", Some("重插"), "2026-10-21T08:00:00Z", false);
        insert_clip(&db, &a).unwrap();
        let same_id = Clip {
            content_hash: "other".into(),
            ..a.clone()
        };
        assert!(insert_clip(&db, &same_id).is_err());
    }
}
