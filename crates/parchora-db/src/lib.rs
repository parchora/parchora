//! parchora-db —— SQLite 结构化存储（schema v2：14 表 + 6 索引，design §3.2）。
//!
//! 锚点：design §2.6（连接与 PRAGMA 四件套 / 迁移 / 事务封装 / checkpoint 双时机）·
//! §2.10 第 3 点（per-root 独立 DB：切根 = 关连接 + 开新连接，字面序）· §3.2 DDL 唯一权威
//! （DEC-89：v1 表实数 13 · DEC-212：v2 追加 queue_entries）。
//!
//! 边界不变量（编译期强制，design §3.1）：本 crate 公共接口**不含 .md 正文**——
//! 对 notes 只存指针（uuid / path / tags / 时间戳 / 来源三元组），正文永不入 DB（C1）。
//!
//! 单写者模型（§2.6）：连接为 `Mutex<Connection>`（rusqlite `Connection: Send` 非 Sync），
//! 与「单写者 + DB 命令通道」同构——通道形态（mpsc）defer 至 TASK-202 首消费点（DEC-89）。

mod clips;
mod error;
mod queue;
mod schema;
mod store;
mod todos;

pub use clips::{
    delete_clip, delete_unpinned_before, delete_unpinned_over_limit, insert_clip,
    list_blob_rel_paths, list_clips, recent_clip_hashes,
};
pub use error::DbError;
pub use queue::{
    clear_queue_entries, delete_queue_entry, insert_queue_entry, list_queue_entries, QueueEntryRow,
};
pub use schema::CURRENT_VERSION;
pub use store::{db_file_path, Db, RootStore};
pub use todos::{
    delete_todo, delete_todo_by_identifier, insert_todo, list_todos, max_identifier_seq,
    set_todo_bucket, set_todo_status,
};
