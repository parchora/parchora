//! parchora-core —— 纯逻辑领域模型层（design §3.1 不变量 4：core 类型不含持久化类型）。
//!
//! 规格锚点：
//! - REQ-CAPTURE-8TYPES（8 类判定链）/ REQ-CAPTURE-PRIVACY（C5 三标记拒收）→ `classify`
//! - REQ-STORE-FILENAME（C6 拒绝式文件名）→ `filenames`
//! - REQ-TASK-SORT + design §2.9（排序键与沉底）→ `task`
//! - REQ-TASK-STATUS-5 / REQ-TASK-PRIORITY-5 / REQ-TASK-SCHEDULE-BUCKET / REQ-TASK-COMMENTS → `task`
//! - REQ-CONTENT-3STATE / REQ-TASK-FROM-CLIP（DEC-15 source）→ `content`
//! - REQ-HOTKEY-CONFLICT（两轮互比 + 7 固定项，作用域分组）→ `hotkeys`
//! - REQ-BOARD-COLOR（9 色按 (createdAt,id) 索引）→ `pinboard`
//! - C7 确定性收敛（design §2.10e）→ `survivor`

pub mod classify;
pub mod content;
pub mod filenames;
pub mod hash;
pub mod hotkeys;
pub mod pinboard;
pub mod survivor;
pub mod task;
pub mod timefmt;
