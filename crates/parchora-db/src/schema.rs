//! Schema v1→v2：DDL 唯一权威 = design §3.2。
//!
//! 计数：表实数 **14** + 索引 6（v1 13 表 = DEC-89；v2 追加 `queue_entries` = DEC-212；
//! v3 追加 `todos.mark` 单列 = DEC-232 用户授权的标签色持久化链）。
//!
//! v1 建表序按 FK 依赖排布（被引用表先行）：`_meta` → notes(+idx) → folders →
//! trash_meta → projects → todos(+idx×2) → labels → todos_labels → comments(+idx)
//! → clips(+idx×2) → clips_labels → pinboards → pinboard_entries。
//! v2 为追加式增量单元（无 FK 无索引，行生命周期短、恒走全表读）。

use rusqlite_migration::{Migrations, M};

/// 当前支持的最高 schema 版本（v3；`user_version` 超前即 fail-closed）。
pub const CURRENT_VERSION: i64 = 3;

/// v1 全量 DDL（§3.2 逐句转写；首个迁移单元 `M::up`——v1 无历史版本路径可增量）。
pub(crate) const SCHEMA_V1_SQL: &str = r#"
-- 0. 应用级元数据（§2.8：identifier_counter 同事务递增）
CREATE TABLE _meta ( key TEXT PRIMARY KEY, value TEXT NOT NULL );

-- 1. notes 指针表（DEC-41：正文永不入库 C1；tags 为 7 色标签名 JSON 数组）
CREATE TABLE notes (
  uuid TEXT PRIMARY KEY,
  path TEXT NOT NULL UNIQUE,          -- 相对 <files>/（/ 分隔，跨平台稳定）
  created_at TEXT NOT NULL,           -- ISO8601（沿 EM savedAt 语义）
  modified_at TEXT NOT NULL,
  saved_at TEXT NOT NULL,             -- 指纹对账 1s 阈值基准
  tags TEXT NOT NULL DEFAULT '[]',
  source_clip_id TEXT, source_app TEXT, source_captured_at TEXT  -- DEC-15，可空；有意无 DB 级 FK（clip 可被历史上限清理）
);
CREATE INDEX idx_notes_saved_at ON notes(saved_at);

-- 2. 文件夹
CREATE TABLE folders ( path TEXT PRIMARY KEY, color TEXT );

-- 3. 回收站元数据（正文仍在 files/.trash/，此处为指针）
CREATE TABLE trash_meta (
  uuid TEXT PRIMARY KEY,
  filename TEXT NOT NULL,             -- <uuid>_<原名>.md
  original_path TEXT NOT NULL,
  trashed_at TEXT NOT NULL, created_at TEXT, modified_at TEXT, tags TEXT NOT NULL DEFAULT '[]'
);

-- 4. 项目
CREATE TABLE projects (
  id TEXT PRIMARY KEY, name TEXT NOT NULL, icon TEXT, description TEXT,
  sort_order INTEGER NOT NULL DEFAULT 0, created_at TEXT NOT NULL
);

-- 5. todos（DEC-41 独立实体；DEC-43 排期桶；T7-R1 勘误：priority 权威值域 'medium'）
CREATE TABLE todos (
  id TEXT PRIMARY KEY,
  title TEXT NOT NULL, description TEXT NOT NULL DEFAULT '',
  status TEXT NOT NULL CHECK (status IN ('todo','in_progress','backlog','done','cancelled')),
  priority TEXT NOT NULL CHECK (priority IN ('urgent','high','medium','low','none')),
  schedule_bucket TEXT NOT NULL DEFAULT 'unscheduled'
    CHECK (schedule_bucket IN ('today','tomorrow','this_week','later','unscheduled')),
  sort_order INTEGER NOT NULL DEFAULT 0,
  identifier TEXT NOT NULL UNIQUE,    -- TASK-n；_meta 计数器同事务分配（§2.8）
  project_id TEXT REFERENCES projects(id) ON DELETE SET NULL,
  source_kind TEXT CHECK (source_kind IN ('clip','note')),  -- 'note' = Note→Task 派生路径；无 DB 级 FK（孤儿由启动引用校验剔除）
  source_ref_id TEXT, source_app TEXT, source_captured_at TEXT,
  created_at TEXT NOT NULL, modified_at TEXT NOT NULL
);
CREATE INDEX idx_todos_board  ON todos(status, priority, sort_order);
CREATE INDEX idx_todos_bucket ON todos(schedule_bucket);

-- 6. 标签与关联（多对多；labels 为主表，task / clip 各自关联）
CREATE TABLE labels ( id TEXT PRIMARY KEY, name TEXT NOT NULL UNIQUE, color TEXT );
CREATE TABLE todos_labels (
  todo_id TEXT NOT NULL REFERENCES todos(id)   ON DELETE CASCADE,
  label_id TEXT NOT NULL REFERENCES labels(id) ON DELETE CASCADE,
  PRIMARY KEY (todo_id, label_id)
);
CREATE TABLE clips_labels (
  clip_id TEXT NOT NULL REFERENCES clips(id)   ON DELETE CASCADE,
  label_id TEXT NOT NULL REFERENCES labels(id) ON DELETE CASCADE,
  PRIMARY KEY (clip_id, label_id)
);

-- 7. 动态（comments）
CREATE TABLE comments (
  id TEXT PRIMARY KEY,
  todo_id TEXT NOT NULL REFERENCES todos(id) ON DELETE CASCADE,
  content TEXT NOT NULL,
  created_at TEXT NOT NULL
);
CREATE INDEX idx_comments_todo ON comments(todo_id, created_at);

-- 8. clips（8 类内容；替换 history.jsonl）
CREATE TABLE clips (
  id TEXT PRIMARY KEY,
  content_type TEXT NOT NULL CHECK (content_type IN
    ('plainText','richText','html','image','url','fileURL','color','unknown')),
  text_content TEXT,                  -- 文本类内容；二进制类为 NULL（正文在 blobs）
  content_hash TEXT NOT NULL,         -- sha256 hex（10s 去重键）
  source_app TEXT,
  size INTEGER NOT NULL DEFAULT 0,
  is_pinned INTEGER NOT NULL DEFAULT 0,
  blob_rel_path TEXT,                 -- 相对 data/blobs/；NULL = 纯文本条目
  thumb_rel_path TEXT,                -- 仅 image 类
  edited_at TEXT,                     -- 用户就地编辑文本时间（LWW 字段）
  created_at TEXT NOT NULL
);
CREATE INDEX idx_clips_created ON clips(created_at DESC);
CREATE INDEX idx_clips_dedupe  ON clips(content_hash, created_at);

-- 9. 收集板与跨类型条目（kind 决定 ref_id 解释：DB 行 vs 文件指针；ref_id 有意无 FK——
--    note 类可被外部文件操作删除，孤儿由启动引用校验剔除，REQ-BOARD-ORPHAN-RECONCILE）
CREATE TABLE pinboards (
  id TEXT PRIMARY KEY, name TEXT NOT NULL,
  sort_order INTEGER NOT NULL DEFAULT 0, created_at TEXT NOT NULL
);
CREATE TABLE pinboard_entries (
  pinboard_id TEXT NOT NULL REFERENCES pinboards(id) ON DELETE CASCADE,
  kind TEXT NOT NULL CHECK (kind IN ('clip','note','task')),
  ref_id TEXT NOT NULL,
  added_at TEXT NOT NULL,
  sort_order INTEGER NOT NULL DEFAULT 0,
  PRIMARY KEY (pinboard_id, ref_id)
);
"#;

/// v2 增量 DDL（DEC-212：承诺式队列跨会话持久 · 每动作事务即写）。
/// 无 FK 无索引——行随消费/清空删除、条数小，读写恒走全表按 `ord` 排序。
pub(crate) const SCHEMA_V2_SQL: &str = r#"
CREATE TABLE queue_entries (
  uuid TEXT PRIMARY KEY,
  text TEXT NOT NULL,
  ord INTEGER NOT NULL,               -- 入队序（单调递增；FIFO/LIFO 为消费序，不入列）
  created_at TEXT NOT NULL            -- ISO8601
);
"#;

/// v3 单列增量（DEC-232 · 用户授权「任务自身标签色持久化链」）：`todos.mark`
/// ＝颜色标记七值下标 0~6（DEC-168 MARKS 同轴；默认赤=0 与创建面板默认同值）。
/// 七色值集不入 DB CHECK（UI 常量域），下标域由插入侧 clamp 承载。
pub(crate) const SCHEMA_V3_SQL: &str = r#"
ALTER TABLE todos ADD COLUMN mark INTEGER NOT NULL DEFAULT 0;
"#;

/// 表名全集（v1 13 表 + v2 1 表；DDL 序；测试按字典序精确断言全集，防多建 / 漏建；仅测试构建使用）。
#[cfg(test)]
pub(crate) const SCHEMA_TABLES: [&str; 14] = [
    "_meta",
    "notes",
    "folders",
    "trash_meta",
    "projects",
    "todos",
    "labels",
    "todos_labels",
    "comments",
    "clips",
    "clips_labels",
    "pinboards",
    "pinboard_entries",
    "queue_entries",
];

/// 迁移集（rusqlite_migration 版本由 `PRAGMA user_version` 驱动，§2.6；v1→v2 增量）。
pub(crate) fn migrations() -> Migrations<'static> {
    Migrations::new(vec![
        M::up(SCHEMA_V1_SQL),
        M::up(SCHEMA_V2_SQL),
        M::up(SCHEMA_V3_SQL),
    ])
}
