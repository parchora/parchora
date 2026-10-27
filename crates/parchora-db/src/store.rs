//! `Db` 单根连接封装（连接 → 版本闸门 → PRAGMA 四件套 → 迁移 → 事务 / checkpoint）与
//! `RootStore` per-root 连接管理（design §2.10 第 3 点：同一时刻仅活跃根持连接，
//! **切根 = 先关旧后开新（字面序）**）。
//!
//! checkpoint 双时机（§2.6）：退出前 TRUNCATE / 空闲 5 分钟 PASSIVE——本 crate 只提供
//! 原子方法；接线已落 app 层（DEC-89 双时机 · start_db_idle_checkpoint 空闲臂 /
//! quit_checkpoint 退出臂，节流与生命周期管辖区）。

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};

use rusqlite::Connection;

use crate::error::DbError;
use crate::schema::{self, CURRENT_VERSION};

/// PRAGMA busy_timeout 毫秒（§2.6 四件套唯一口径：设置/断言/测试同源防漂移）。
const BUSY_TIMEOUT_MS: i64 = 5_000;

/// store 内部打开态。
#[derive(Debug)]
struct OpenRoot {
    root: PathBuf,
    db: Arc<Db>,
}

/// 单存储根的 SQLite 连接封装（单写者同构：Mutex<Connection>，§2.6）。
#[derive(Debug)]
pub struct Db {
    conn: Mutex<Connection>,
}

/// 存储根 → DB 文件路径（`<root>/data/parchora.db`——布局唯一口径，§2.6）。
pub fn db_file_path(root: &Path) -> PathBuf {
    root.join("data").join("parchora.db")
}

/// PRAGMA 断言失配错误构造（fail-closed 文案统一出口；通用码 SQLITE_ERROR + 全文）。
fn pragma_mismatch(name: &str, expected: &str, actual: &str) -> DbError {
    DbError::Open(rusqlite::Error::SqliteFailure(
        rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_ERROR),
        Some(format!("PRAGMA {name} 期望 {expected}，实得 {actual}")),
    ))
}

impl Db {
    /// 打开（或创建）`<root>/data/parchora.db`：建 `data/` 目录 → 打开连接 → 配置。
    pub fn open(root: &Path) -> Result<Arc<Db>, DbError> {
        std::fs::create_dir_all(root.join("data"))?;
        Self::configure(Connection::open(db_file_path(root))?, true)
    }

    /// 内存模式（测试 / 诊断专用）：不落盘；journal_mode 保持 memory（不适用 WAL 断言）。
    pub fn open_in_memory() -> Result<Arc<Db>, DbError> {
        Self::configure(Connection::open_in_memory()?, false)
    }

    /// 打开逻辑唯一实现（防双口径）：版本闸门先于 PRAGMA / 迁移（fail-closed，§3.2）
    /// → PRAGMA 四件套 → rusqlite_migration `to_latest` → 回读断言。
    fn configure(conn: Connection, verify_journal: bool) -> Result<Arc<Db>, DbError> {
        let mut conn = conn;
        let version: i64 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        if version > CURRENT_VERSION {
            return Err(DbError::SchemaTooNew {
                db_version: version,
                supported: CURRENT_VERSION,
            });
        }

        // PRAGMA 四件套（§2.6）：WAL · busy_timeout=BUSY_TIMEOUT_MS（同源唯一口径）· foreign_keys=ON ·
        // synchronous=NORMAL(1)。内存库保持 memory，跳过 journal_mode 赋值。
        if verify_journal {
            conn.pragma_update(None, "journal_mode", "WAL")?;
        }
        conn.pragma_update(None, "busy_timeout", BUSY_TIMEOUT_MS)?;
        conn.pragma_update(None, "foreign_keys", true)?;
        conn.pragma_update(None, "synchronous", 1)?;
        schema::migrations().to_latest(&mut conn)?;

        let db = Arc::new(Db {
            conn: Mutex::new(conn),
        });

        // 赋值后逐项回读断言（fail-closed：PRAGMA 静默失效即报错，禁止「以为开了」）。
        let mode = db.journal_mode()?;
        if verify_journal && mode != "wal" {
            return Err(pragma_mismatch("journal_mode", "wal", &mode));
        }
        let busy = db.busy_timeout_ms()?;
        if busy != BUSY_TIMEOUT_MS {
            return Err(pragma_mismatch(
                "busy_timeout",
                &BUSY_TIMEOUT_MS.to_string(),
                &busy.to_string(),
            ));
        }
        let fk = db.foreign_keys()?;
        if fk != 1 {
            return Err(pragma_mismatch("foreign_keys", "on(1)", &fk.to_string()));
        }
        let sync = db.synchronous()?;
        if sync != 1 {
            return Err(pragma_mismatch(
                "synchronous",
                "normal(1)",
                &sync.to_string(),
            ));
        }
        Ok(db)
    }

    /// 事务封装（§2.6）：闭包 Ok → commit；Err 短路 → tx Drop 自动 ROLLBACK。
    /// 注：commit 自身失败不自动回滚（SQLite 语义，事务已消耗）——错误上浮由调用方裁决。
    /// 注：闭包内禁止嵌套调用同一 `Db` 的 `with_tx`（单写者 Mutex 同构 → 自我死锁）。
    pub fn with_tx<T>(
        &self,
        f: impl FnOnce(&rusqlite::Transaction) -> rusqlite::Result<T>,
    ) -> Result<T, DbError> {
        let conn = self.lock_conn()?;
        let tx = conn.unchecked_transaction()?;
        match f(&tx) {
            Ok(v) => {
                tx.commit()?;
                Ok(v)
            }
            // Err 短路：tx 在此分支尾 Drop（未 commit）→ 自动 ROLLBACK。
            Err(e) => Err(e.into()),
        }
    }

    /// 锁定连接（Poisoned 显式上浮——禁止静默重建连接掩盖现场）。
    fn lock_conn(&self) -> Result<MutexGuard<'_, Connection>, DbError> {
        self.conn.lock().map_err(|_| DbError::Poisoned)
    }

    /// 当前 `journal_mode`（诊断 / 测试）。
    pub fn journal_mode(&self) -> Result<String, DbError> {
        Ok(self
            .lock_conn()?
            .query_row("PRAGMA journal_mode", [], |r| r.get(0))?)
    }

    /// 当前 `busy_timeout`（ms）。
    pub fn busy_timeout_ms(&self) -> Result<i64, DbError> {
        Ok(self
            .lock_conn()?
            .query_row("PRAGMA busy_timeout", [], |r| r.get(0))?)
    }

    /// `foreign_keys` 开关（1 = ON）。
    pub fn foreign_keys(&self) -> Result<i64, DbError> {
        Ok(self
            .lock_conn()?
            .query_row("PRAGMA foreign_keys", [], |r| r.get(0))?)
    }

    /// `synchronous`（1 = NORMAL）。
    pub fn synchronous(&self) -> Result<i64, DbError> {
        Ok(self
            .lock_conn()?
            .query_row("PRAGMA synchronous", [], |r| r.get(0))?)
    }

    /// `user_version`（rusqlite_migration 版本表承载的 schema 版本）。
    pub fn user_version(&self) -> Result<i64, DbError> {
        Ok(self
            .lock_conn()?
            .query_row("PRAGMA user_version", [], |r| r.get(0))?)
    }

    /// `wal_checkpoint(TRUNCATE)`（退出前时机；退出臂接 parchora-app quit 链
    /// quit_checkpoint——DEC-89 双时机）。返回 `(busy, log_pages, checkpointed_pages)`。
    pub fn checkpoint_truncate(&self) -> Result<(i64, i64, i64), DbError> {
        Ok(self
            .lock_conn()?
            .query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |r| {
                Ok((r.get(0)?, r.get(1)?, r.get(2)?))
            })?)
    }

    /// `wal_checkpoint(PASSIVE)`（空闲 5 分钟时机；空闲臂接 parchora-app
    /// start_db_idle_checkpoint 5 分钟节流——DEC-89 双时机）。
    pub fn checkpoint_passive(&self) -> Result<(i64, i64, i64), DbError> {
        Ok(self
            .lock_conn()?
            .query_row("PRAGMA wal_checkpoint(PASSIVE)", [], |r| {
                Ok((r.get(0)?, r.get(1)?, r.get(2)?))
            })?)
    }
}

/// 存储根 → 活跃连接管理（§2.10 第 3 点）：同一时刻仅活跃根持连接；
/// **切根 = 先关旧后开新（字面序）**——开新失败时旧连接已关、store 留空态，可重试；
/// 切根期仍被持有的旧 `Arc<Db>` 句柄自然存活至引用释放（app 层须避免切换与写并发交错）。
#[derive(Debug)]
pub struct RootStore {
    active: Mutex<Option<OpenRoot>>,
    /// 事件时序日志（仅测试构建）：「close 在 open 前」字面序断言用。
    #[cfg(test)]
    events: Mutex<Vec<&'static str>>,
}

impl RootStore {
    /// 空态构造（未打开任何根）。
    pub fn new() -> Self {
        Self {
            active: Mutex::new(None),
            #[cfg(test)]
            events: Mutex::new(Vec::new()),
        }
    }

    /// 切换活跃根：① 取下旧根（连接 Drop = 关）→ ② 打开新根（字面序 §2.10.3）。
    pub fn switch_to(&self, root: &Path) -> Result<(), DbError> {
        let mut guard = self.active.lock().map_err(|_| DbError::Poisoned)?;
        if let Some(old) = guard.take() {
            #[cfg(test)]
            {
                self.log("close_active_root");
            }
            drop(old);
        }
        let db = Db::open(root)?;
        #[cfg(test)]
        {
            self.log("open_new_root");
        }
        *guard = Some(OpenRoot {
            root: root.to_path_buf(),
            db,
        });
        Ok(())
    }

    /// 当前活跃根（无连接 → `None`；快照语义，调用方自担与切根的时序）。
    pub fn active_root(&self) -> Result<Option<PathBuf>, DbError> {
        let guard = self.active.lock().map_err(|_| DbError::Poisoned)?;
        Ok(guard.as_ref().map(|r| r.root.clone()))
    }

    /// 活跃根连接句柄（共享同一单写者连接；切根语义见结构体文档注）。
    pub fn active_db(&self) -> Result<Arc<Db>, DbError> {
        let guard = self.active.lock().map_err(|_| DbError::Poisoned)?;
        guard
            .as_ref()
            .map(|r| r.db.clone())
            .ok_or(DbError::NoActiveRoot)
    }

    /// 在活跃根连接上执行事务（无活跃根 → `NoActiveRoot`）。
    pub fn with_tx<T>(
        &self,
        f: impl FnOnce(&rusqlite::Transaction) -> rusqlite::Result<T>,
    ) -> Result<T, DbError> {
        self.active_db()?.with_tx(f)
    }

    #[cfg(test)]
    fn log(&self, what: &'static str) {
        if let Ok(mut e) = self.events.lock() {
            e.push(what);
        }
    }
}

impl Default for RootStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
impl RootStore {
    /// 事件时序日志快照（测试专用）。
    pub fn logged_events(&self) -> Vec<&'static str> {
        self.events.lock().expect("events 锁未中毒").clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{insert_queue_entry, list_queue_entries};

    /// sqlite_master 表名清单（字典序；排除 sqlite 内部表）。
    fn table_names(conn: &Connection) -> Vec<String> {
        let mut stmt = conn
            .prepare(
                "SELECT name FROM sqlite_master \
                 WHERE type = 'table' AND name NOT LIKE 'sqlite_%' ORDER BY name",
            )
            .unwrap();
        stmt.query_map([], |r| r.get::<_, String>(0))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
    }

    /// 表名全集期望（字典序；schema.rs `SCHEMA_TABLES` 唯一源）。
    fn expected_tables() -> Vec<String> {
        let mut v: Vec<String> = schema::SCHEMA_TABLES
            .iter()
            .map(|s| s.to_string())
            .collect();
        v.sort();
        v
    }

    /// 纳秒级独立临时目录（避免 tempfile 依赖——规范 9：零新增第三方）。
    fn temp_root(tag: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("系统时钟正常")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("parchora-db-test-{tag}-{nanos}"));
        std::fs::create_dir_all(&dir).expect("创建临时目录");
        dir
    }

    /// ① 新鲜建库：表清单 14 张精确断言 + 幂等（二次打开不迁移不出错）。
    #[test]
    fn fresh_open_creates_schema_and_is_idempotent() {
        let root = temp_root("fresh");
        let db = Db::open(&root).unwrap();
        assert_eq!(db.user_version().unwrap(), CURRENT_VERSION);
        drop(db);

        let db2 = Db::open(&root).unwrap();
        assert_eq!(db2.user_version().unwrap(), CURRENT_VERSION);
        let names: Vec<String> = db2
            .with_tx(|tx| -> rusqlite::Result<Vec<String>> {
                let mut stmt = tx.prepare(
                    "SELECT name FROM sqlite_master \
                     WHERE type = 'table' AND name NOT LIKE 'sqlite_%' ORDER BY name",
                )?;
                let rows = stmt.query_map([], |r| r.get::<_, String>(0))?;
                rows.collect()
            })
            .unwrap();
        assert_eq!(names, expected_tables());
    }

    /// ② PRAGMA 四件套（持久库）逐项回读断言。
    #[test]
    fn pragma_quartet_applied_on_file_db() {
        let root = temp_root("pragma");
        let db = Db::open(&root).unwrap();
        assert_eq!(db.journal_mode().unwrap(), "wal");
        assert_eq!(db.busy_timeout_ms().unwrap(), BUSY_TIMEOUT_MS);
        assert_eq!(db.foreign_keys().unwrap(), 1);
        assert_eq!(db.synchronous().unwrap(), 1);
    }

    /// ③ 版本闸门：user_version 超前 → fail-closed 拒绝打开（应用过旧）。
    #[test]
    fn too_new_schema_rejects_fail_closed() {
        let root = temp_root("toonew");
        drop(Db::open(&root).unwrap()); // 先建当前版库

        // 裸连接把 user_version 抬到 99（模拟未来版本产物）
        let raw = Connection::open(db_file_path(&root)).unwrap();
        raw.execute_batch("PRAGMA user_version = 99;").unwrap();
        drop(raw);

        let err = Db::open(&root).unwrap_err();
        assert!(
            matches!(
                err,
                DbError::SchemaTooNew {
                    db_version: 99,
                    supported: CURRENT_VERSION
                }
            ),
            "实得 {err:?}"
        );
    }

    /// ④ 事务封装：Err 短路回滚 / Ok 提交，双向。
    #[test]
    fn with_tx_commits_and_rolls_back() {
        let db = Db::open_in_memory().unwrap();

        // 回滚腿：人为业务失败信号 → 未提交行不可见
        let err = db
            .with_tx(|tx| -> rusqlite::Result<()> {
                tx.execute(
                    "INSERT INTO _meta(key, value) VALUES ('identifier_counter', '5')",
                    [],
                )?;
                Err(rusqlite::Error::InvalidQuery)
            })
            .unwrap_err();
        assert!(
            matches!(err, DbError::Open(rusqlite::Error::InvalidQuery)),
            "实得 {err:?}"
        );
        let n: i64 = db
            .with_tx(|tx| {
                tx.query_row(
                    "SELECT COUNT(*) FROM _meta WHERE key = 'identifier_counter'",
                    [],
                    |r| r.get(0),
                )
            })
            .unwrap();
        assert_eq!(n, 0);

        // 提交腿：Ok → commit → 可读
        db.with_tx(|tx| {
            tx.execute(
                "INSERT INTO _meta(key, value) VALUES ('identifier_counter', '7')",
                [],
            )
        })
        .unwrap();
        let v: String = db
            .with_tx(|tx| {
                tx.query_row(
                    "SELECT value FROM _meta WHERE key = 'identifier_counter'",
                    [],
                    |r| r.get(0),
                )
            })
            .unwrap();
        assert_eq!(v, "7");
    }

    /// ⑤ kill -9 型中断注入（mem::forget 代理——不跑 Drop、无 commit / 无回滚）：
    /// 未提交行不可见、user_version 无损、表结构完好（WAL / 日志恢复一致性）。
    /// 注：被 forget 的连接仅泄漏于测试进程内存（DEC-89 口径在案）。
    #[test]
    fn uncommitted_transaction_vanishes_like_kill9() {
        let root = temp_root("kill9");
        std::fs::create_dir_all(root.join("data")).unwrap();

        // 裸连接建库到 v1（不经 Db::open——便于对裸连接 mem::forget）
        let mut cn1 = Connection::open(db_file_path(&root)).unwrap();
        cn1.query_row("PRAGMA journal_mode=WAL", [], |r| r.get::<_, String>(0))
            .unwrap();
        schema::migrations().to_latest(&mut cn1).unwrap();

        // 未提交事务：立即型写入（已过 WAL 写入路径，未 commit）
        cn1.execute_batch(
            "BEGIN IMMEDIATE; \
             INSERT INTO _meta(key, value) VALUES ('identifier_counter', '7');",
        )
        .unwrap();

        // 进程死亡代理：不 commit / 不 close——forget 跳过 Drop（回滚由此不会发生）
        std::mem::forget(cn1);

        // 新裸「读」连接验证恢复一致性（读路径在 WAL 下不与被遗忘的写锁冲突）
        let cn2 = Connection::open(db_file_path(&root)).unwrap();
        let n: i64 = cn2
            .query_row(
                "SELECT COUNT(*) FROM _meta WHERE key = 'identifier_counter'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(n, 0);
        let v: i64 = cn2
            .query_row("PRAGMA user_version", [], |r| r.get(0))
            .unwrap();
        assert_eq!(v, CURRENT_VERSION);
        assert_eq!(table_names(&cn2), expected_tables());
    }

    /// ⑤b v1→v2 增量迁移（DEC-212）：存量 v1 库打开后抬到 v2、queue_entries
    /// 就位、v1 数据无损（迁移只建新表不动旧行）。
    #[test]
    fn v1_to_v2_migration_preserves_data_and_adds_queue_entries() {
        let root = temp_root("migrate");
        std::fs::create_dir_all(root.join("data")).unwrap();
        let db_path = db_file_path(&root);

        // 裸连接手工建 v1 库（v1 全量 DDL + user_version=1）
        let raw = Connection::open(&db_path).unwrap();
        raw.execute_batch(schema::SCHEMA_V1_SQL).unwrap();
        raw.execute_batch("PRAGMA user_version = 1;").unwrap();
        raw.execute_batch("INSERT INTO _meta(key, value) VALUES ('identifier_counter', '9');")
            .unwrap();
        drop(raw);

        // 常规打开：迁移至 v2
        let db = Db::open(&root).unwrap();
        assert_eq!(db.user_version().unwrap(), CURRENT_VERSION);
        let names: Vec<String> = db
            .with_tx(|tx| -> rusqlite::Result<Vec<String>> {
                let mut stmt = tx.prepare(
                    "SELECT name FROM sqlite_master \
                     WHERE type = 'table' AND name NOT LIKE 'sqlite_%' ORDER BY name",
                )?;
                let rows = stmt.query_map([], |r| r.get::<_, String>(0))?;
                rows.collect()
            })
            .unwrap();
        assert_eq!(names, expected_tables());
        // v1 数据无损
        let v: String = db
            .with_tx(|tx| {
                tx.query_row(
                    "SELECT value FROM _meta WHERE key = 'identifier_counter'",
                    [],
                    |r| r.get(0),
                )
            })
            .unwrap();
        assert_eq!(v, "9");
        // v2 表可用（读写打通）
        insert_queue_entry(&db, "mig-1", "甲", 1, "2026-10-23T09:58:00Z").unwrap();
        let rows = list_queue_entries(&db).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].text, "甲");
    }

    /// ⑥ RootStore 双根隔离 + 切换时序（先关后开 §2.10.3 字面序）+ 原根持久性。
    #[test]
    fn root_store_switches_close_before_open_and_isolates() {
        let ra = temp_root("root-a");
        let rb = temp_root("root-b");
        let store = RootStore::new();

        assert!(store.active_root().unwrap().is_none());
        store.switch_to(&ra).unwrap();
        assert_eq!(store.active_root().unwrap().as_deref(), Some(ra.as_path()));
        store
            .with_tx(|tx| {
                tx.execute(
                    "INSERT INTO _meta(key, value) VALUES ('which', 'root-a')",
                    [],
                )
            })
            .unwrap();

        store.switch_to(&rb).unwrap();
        assert_eq!(store.active_root().unwrap().as_deref(), Some(rb.as_path()));

        // 时序断言：open → close → open（第二次切换先关旧后开新）
        assert_eq!(
            store.logged_events(),
            ["open_new_root", "close_active_root", "open_new_root"]
        );

        // 双根隔离：root-b 无 root-a 的数据
        let n_a: i64 = store
            .with_tx(|tx| {
                tx.query_row(
                    "SELECT COUNT(*) FROM _meta WHERE key = 'which' AND value = 'root-a'",
                    [],
                    |r| r.get(0),
                )
            })
            .unwrap();
        assert_eq!(n_a, 0);

        // root-a 数据仍在原库（per-root 持久性）
        let db_a = Db::open(&ra).unwrap();
        let v: String = db_a
            .with_tx(|tx| {
                tx.query_row("SELECT value FROM _meta WHERE key = 'which'", [], |r| {
                    r.get(0)
                })
            })
            .unwrap();
        assert_eq!(v, "root-a");
    }

    /// ⑦ 内存模式全套：版本 / 表结构 / 事务往返 / checkpoint 无害；
    /// journal_mode == "memory"（内存库不适 WAL，断言口径按 §2.6 持久库区分）。
    #[test]
    fn in_memory_db_full_suite() {
        let db = Db::open_in_memory().unwrap();
        assert_eq!(db.user_version().unwrap(), CURRENT_VERSION);
        assert_eq!(db.journal_mode().unwrap(), "memory");
        assert_eq!(db.busy_timeout_ms().unwrap(), BUSY_TIMEOUT_MS);
        assert_eq!(db.foreign_keys().unwrap(), 1);
        assert_eq!(db.synchronous().unwrap(), 1);

        db.with_tx(|tx| tx.execute("INSERT INTO _meta(key, value) VALUES ('k', 'v')", []))
            .unwrap();
        let v: String = db
            .with_tx(|tx| tx.query_row("SELECT value FROM _meta WHERE key = 'k'", [], |r| r.get(0)))
            .unwrap();
        assert_eq!(v, "v");

        // checkpoint 在内存库无害（返回三元组行即可）
        let _ = db.checkpoint_passive().unwrap();
    }

    /// ⑧ 完整性约束下沉：FK 生效（引用不存在项目 → 拒绝）+ CHECK 拒收
    /// （priority='normal' 为 T7-R1 勘误前旧值，权威值域 medium）。
    #[test]
    fn fk_and_check_constraints_enforced() {
        let db = Db::open_in_memory().unwrap();

        // FK 腿（结构化域 FK + 级联策略，§3.2）
        let err = db
            .with_tx(|tx| -> rusqlite::Result<()> {
                tx.execute(
                    "INSERT INTO todos \
                     (id, title, status, priority, identifier, project_id, created_at, modified_at) \
                     VALUES ('t1', '标题', 'todo', 'low', 'TASK-1', 'no-such-project', '', '')",
                    [],
                )?;
                Ok(())
            })
            .unwrap_err();
        match err {
            DbError::Open(rusqlite::Error::SqliteFailure(_, Some(msg))) => {
                assert!(msg.contains("FOREIGN KEY"), "FK 违规信息异常: {msg}");
            }
            other => panic!("期望 FK 违规，实得 {other:?}"),
        }

        // CHECK 腿（priority 旧值 'normal' 拒收）
        let err = db
            .with_tx(|tx| -> rusqlite::Result<()> {
                tx.execute(
                    "INSERT INTO todos \
                     (id, title, status, priority, identifier, created_at, modified_at) \
                     VALUES ('t2', '标题', 'todo', 'normal', 'TASK-2', '', '')",
                    [],
                )?;
                Ok(())
            })
            .unwrap_err();
        match err {
            DbError::Open(rusqlite::Error::SqliteFailure(_, Some(msg))) => {
                assert!(msg.contains("CHECK"), "CHECK 违规信息异常: {msg}");
            }
            other => panic!("期望 CHECK 违规，实得 {other:?}"),
        }
    }
}
