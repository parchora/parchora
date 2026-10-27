//! `todos` 表 CRUD（TASK-300 · REQ-TASK-PERSIST 待办落库 · design §3.2 投影）。
//!
//! 形态口径：core::task::Todo 即投影唯一形态源。identifier（TASK-n）为 app↔DB
//! 的稳定外键（UNIQUE 列）；读写统一走 `Db::with_tx`（单写者事务封装 · §2.6）。
//! 读路径脏值（status/priority/bucket 反解失败、id 非法）fail-soft 跳过该行。

use rusqlite::params;

use crate::error::DbError;
use crate::store::Db;
use parchora_core::content::{Source, Uuid as CoreUuid};
use parchora_core::task::{Priority, ScheduleBucket, TaskStatus, Todo};

/// 单条事务写（创建漏斗 · 同事务写 identifier 与行本体）。
pub fn insert_todo(db: &Db, todo: &Todo) -> Result<(), DbError> {
    let (source_kind, source_ref_id, source_app, source_captured_at) = match &todo.source {
        None => (None, None, None, None),
        Some(Source::Clip {
            clip_id,
            source_app,
            captured_at,
        }) => (
            Some("clip"),
            Some(clip_id.to_string()),
            source_app.clone(),
            Some(captured_at.clone()),
        ),
        Some(Source::Note {
            ref_id,
            source_app,
            captured_at,
        }) => (
            Some("note"),
            Some(ref_id.to_string()),
            source_app.clone(),
            Some(captured_at.clone()),
        ),
    };
    db.with_tx(|tx| {
        tx.execute(
            "INSERT INTO todos (id, title, description, status, priority, schedule_bucket, \
             sort_order, identifier, project_id, source_kind, source_ref_id, source_app, \
             source_captured_at, created_at, modified_at, mark) \
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16)",
            params![
                todo.id.to_string(),
                todo.title,
                todo.description,
                todo.status.as_db_value(),
                todo.priority.as_db_value(),
                todo.schedule_bucket.as_db_value(),
                todo.sort_order,
                todo.identifier,
                todo.project_id.map(|p| p.to_string()),
                source_kind,
                source_ref_id,
                source_app,
                source_captured_at,
                todo.created_at,
                todo.modified_at,
                todo.mark,
            ],
        )?;
        Ok(())
    })
}

/// 全量读取（identifier 数值升序——TASK-n 创建序稳定承载；启动回填唯一入口）。
pub fn list_todos(db: &Db) -> Result<Vec<Todo>, DbError> {
    db.with_tx(|tx| {
        let mut stmt = tx.prepare(
            "SELECT id, title, description, status, priority, schedule_bucket, sort_order, \
             identifier, project_id, source_kind, source_ref_id, source_app, \
             source_captured_at, created_at, modified_at, mark \
             FROM todos ORDER BY CAST(substr(identifier, 6) AS INTEGER) ASC, identifier ASC",
        )?;
        let rows = stmt
            .query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(3)?,
                    r.get::<_, String>(4)?,
                    r.get::<_, String>(5)?,
                    r.get::<_, i64>(6)?,
                    r.get::<_, String>(7)?,
                    r.get::<_, Option<String>>(8)?,
                    r.get::<_, Option<String>>(9)?,
                    r.get::<_, Option<String>>(10)?,
                    r.get::<_, Option<String>>(11)?,
                    r.get::<_, Option<String>>(12)?,
                    r.get::<_, String>(13)?,
                    r.get::<_, String>(14)?,
                    r.get::<_, i64>(15)?,
                ))
            })?
            .collect::<Result<Vec<_>, rusqlite::Error>>()?;
        let mapped = rows
            .into_iter()
            .filter_map(
                |(
                    id,
                    title,
                    description,
                    status,
                    priority,
                    bucket,
                    sort_order,
                    identifier,
                    project_id,
                    source_kind,
                    source_ref_id,
                    source_app,
                    source_captured_at,
                    created_at,
                    modified_at,
                    mark_i64,
                )| {
                    let id = CoreUuid::parse_str(&id).ok()?;
                    let status = TaskStatus::parse_db_value(&status)?;
                    let priority = Priority::parse_db_value(&priority)?;
                    let schedule_bucket = ScheduleBucket::parse_db_value(&bucket)?;
                    let source = match source_kind.as_deref() {
                        None => None,
                        Some("clip") => Some(Source::Clip {
                            clip_id: CoreUuid::parse_str(source_ref_id.as_deref()?).ok()?,
                            source_app,
                            captured_at: source_captured_at.unwrap_or_default(),
                        }),
                        Some("note") => Some(Source::Note {
                            ref_id: CoreUuid::parse_str(source_ref_id.as_deref()?).ok()?,
                            source_app,
                            captured_at: source_captured_at.unwrap_or_default(),
                        }),
                        _ => return None, // 脏 source_kind → 跳行
                    };
                    Some(Todo {
                        id,
                        title,
                        description,
                        status,
                        priority,
                        schedule_bucket,
                        sort_order,
                        identifier,
                        project_id: project_id.and_then(|p| CoreUuid::parse_str(&p).ok()),
                        source,
                        mark: mark_i64.clamp(0, 6) as usize,
                        created_at,
                        modified_at,
                    })
                },
            )
            .collect();
        Ok(mapped)
    })
}

/// 状态五选一落库（右键「状态」· modified_at 同步戳 LWW 字段）。
pub fn set_todo_status(
    db: &Db,
    identifier: &str,
    status: TaskStatus,
    modified_at: &str,
) -> Result<usize, DbError> {
    db.with_tx(|tx| {
        tx.execute(
            "UPDATE todos SET status = ?2, modified_at = ?3 WHERE identifier = ?1",
            params![identifier, status.as_db_value(), modified_at],
        )
    })
}

/// 排期五桶落库（右键「排期」· modified_at 同步戳）。
pub fn set_todo_bucket(
    db: &Db,
    identifier: &str,
    bucket: ScheduleBucket,
    modified_at: &str,
) -> Result<usize, DbError> {
    db.with_tx(|tx| {
        tx.execute(
            "UPDATE todos SET schedule_bucket = ?2, modified_at = ?3 WHERE identifier = ?1",
            params![identifier, bucket.as_db_value(), modified_at],
        )
    })
}

/// 单条删除（未消费位：保留对称 API 与测试对账）。
pub fn delete_todo(db: &Db, id: &CoreUuid) -> Result<usize, DbError> {
    db.with_tx(|tx| tx.execute("DELETE FROM todos WHERE id = ?1", params![id.to_string()]))
}

/// identifier 单条删除（待办右键「删除」· TASK-300 逆向动作）：UI 行身份即
/// TASK-n；未知 identifier 计 0（幂等重删无行受累）。
pub fn delete_todo_by_identifier(db: &Db, identifier: &str) -> Result<usize, DbError> {
    db.with_tx(|tx| {
        tx.execute(
            "DELETE FROM todos WHERE identifier = ?1",
            params![identifier],
        )
    })
}

/// identifier 序号极大值（next_seq 续接唯一权威；空表 → None）。
/// 解析口径：`TASK-` 之后整段 CAST Integer（脏 identifier → 0，滤除不误伤序号链）。
pub fn max_identifier_seq(db: &Db) -> Result<Option<u64>, DbError> {
    db.with_tx(|tx| {
        let v: Option<i64> = tx.query_row(
            "SELECT MAX(CAST(substr(identifier, 6) AS INTEGER)) FROM todos",
            [],
            |r| r.get(0),
        )?;
        Ok(v.filter(|n| *n >= 1).map(|n| n as u64))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::Db;
    use parchora_core::task::{Priority, ScheduleBucket, TaskStatus};
    use std::sync::Arc;

    fn temp_db() -> Arc<Db> {
        // 目录名唯一性三级（queue.rs 测试同口径：pid + 纳秒 + 进程内原子序号
        // · R184 建议 21 对齐——同拍多测试例不再可能撞目录）。
        static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let seq = SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!(
            "parchora-db-todos-test-{}-{}-{}",
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

    fn fx_todo(seq: u64, title: &str, bucket: ScheduleBucket) -> Todo {
        Todo {
            id: CoreUuid::new_v4(),
            title: title.into(),
            description: String::new(),
            status: TaskStatus::Todo,
            priority: Priority::None,
            schedule_bucket: bucket,
            sort_order: seq as i64,
            identifier: format!("TASK-{seq}"),
            project_id: None,
            source: None,
            mark: 0,
            created_at: "2026-10-21T08:00:00Z".into(),
            modified_at: "2026-10-21T08:00:00Z".into(),
        }
    }

    #[test]
    fn insert_and_list_roundtrip_ordered_by_identifier() {
        let db = temp_db();
        insert_todo(&db, &fx_todo(3, "丙", ScheduleBucket::Later)).unwrap();
        insert_todo(&db, &fx_todo(1, "甲", ScheduleBucket::Today)).unwrap();
        insert_todo(&db, &fx_todo(12, "乙", ScheduleBucket::Unscheduled)).unwrap();
        let listed = list_todos(&db).unwrap();
        let idents: Vec<&str> = listed.iter().map(|t| t.identifier.as_str()).collect();
        assert_eq!(
            idents,
            vec!["TASK-1", "TASK-3", "TASK-12"],
            "数值序非字典序"
        );
        assert_eq!(listed[0].title, "甲");
        assert_eq!(listed[0].status, TaskStatus::Todo);
        assert_eq!(listed[0].schedule_bucket, ScheduleBucket::Today);
        assert!(listed
            .iter()
            .all(|t| t.source.is_none() && t.project_id.is_none()));
    }

    #[test]
    fn todo_mark_roundtrip_clamped() {
        // DEC-232：todos.mark 持久化链——插入侧 0~6 下标往返保值；回读
        // clamp(0,6) 域归一。
        let db = temp_db();
        let mut t = fx_todo(1, "甲", ScheduleBucket::Today);
        t.mark = 5;
        insert_todo(&db, &t).unwrap();
        let listed = list_todos(&db).unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].mark, 5);
        assert_eq!(listed[0].identifier, "TASK-1");
    }

    #[test]
    fn max_identifier_seq_continues_across_sessions() {
        let db = temp_db();
        assert_eq!(max_identifier_seq(&db).unwrap(), None, "空表 None");
        for seq in [1u64, 2, 41] {
            insert_todo(&db, &fx_todo(seq, "任务", ScheduleBucket::Today)).unwrap();
        }
        assert_eq!(max_identifier_seq(&db).unwrap(), Some(41));
    }

    #[test]
    fn set_status_and_bucket_update_modified_at() {
        let db = temp_db();
        insert_todo(&db, &fx_todo(1, "甲", ScheduleBucket::Today)).unwrap();
        let n = set_todo_status(&db, "TASK-1", TaskStatus::Done, "2026-10-22T09:00:00Z").unwrap();
        assert_eq!(n, 1);
        let n = set_todo_bucket(
            &db,
            "TASK-1",
            ScheduleBucket::Tomorrow,
            "2026-10-22T10:00:00Z",
        )
        .unwrap();
        assert_eq!(n, 1);
        let listed = list_todos(&db).unwrap();
        assert_eq!(listed[0].status, TaskStatus::Done);
        assert_eq!(listed[0].schedule_bucket, ScheduleBucket::Tomorrow);
        assert_eq!(listed[0].modified_at, "2026-10-22T10:00:00Z");
        // 未知 identifier → 计 0（无行受累）。
        assert_eq!(
            set_todo_status(&db, "TASK-999", TaskStatus::Done, "2026-10-22T09:00:00Z").unwrap(),
            0
        );
    }

    #[test]
    fn unique_identifier_violation_surfaces() {
        let db = temp_db();
        insert_todo(&db, &fx_todo(1, "甲", ScheduleBucket::Today)).unwrap();
        let dup = Todo {
            id: CoreUuid::new_v4(),
            ..fx_todo(1, "重复序号", ScheduleBucket::Today)
        };
        assert!(
            insert_todo(&db, &dup).is_err(),
            "identifier UNIQUE 冲突上浮"
        );
    }

    #[test]
    fn delete_todo_by_id() {
        let db = temp_db();
        let t = fx_todo(1, "甲", ScheduleBucket::Today);
        insert_todo(&db, &t).unwrap();
        assert_eq!(delete_todo(&db, &t.id).unwrap(), 1);
        assert!(list_todos(&db).unwrap().is_empty());
    }

    #[test]
    fn delete_todo_by_identifier_idempotent_and_unknown_zero() {
        let db = temp_db();
        insert_todo(&db, &fx_todo(1, "甲", ScheduleBucket::Today)).unwrap();
        insert_todo(&db, &fx_todo(2, "乙", ScheduleBucket::Today)).unwrap();
        assert_eq!(delete_todo_by_identifier(&db, "TASK-1").unwrap(), 1);
        let idents: Vec<String> = list_todos(&db)
            .unwrap()
            .into_iter()
            .map(|t| t.identifier)
            .collect();
        assert_eq!(idents, vec!["TASK-2".to_string()], "仅删目标行");
        // 未知 identifier / 幂等重删：计 0 无行受累。
        assert_eq!(delete_todo_by_identifier(&db, "TASK-999").unwrap(), 0);
        assert_eq!(delete_todo_by_identifier(&db, "TASK-1").unwrap(), 0);
    }

    #[test]
    fn source_clip_roundtrip() {
        let db = temp_db();
        let mut t = fx_todo(7, "来源任务", ScheduleBucket::Today);
        t.source = Some(Source::Clip {
            clip_id: CoreUuid::nil(),
            source_app: Some("com.example".into()),
            captured_at: "2026-10-21T07:00:00Z".into(),
        });
        insert_todo(&db, &t).unwrap();
        let listed = list_todos(&db).unwrap();
        match &listed[0].source {
            Some(Source::Clip {
                clip_id,
                source_app,
                captured_at,
            }) => {
                assert_eq!(*clip_id, CoreUuid::nil());
                assert_eq!(source_app.as_deref(), Some("com.example"));
                assert_eq!(captured_at, "2026-10-21T07:00:00Z");
            }
            other => panic!("expected Clip source, got {other:?}"),
        }
    }
}
