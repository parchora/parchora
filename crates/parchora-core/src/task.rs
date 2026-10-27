//! 任务模型与排序（TASK-102/103/108）。
//!
//! 锚点：REQ-TASK-STATUS-5 / REQ-TASK-PRIORITY-5 / REQ-TASK-SCHEDULE-BUCKET（五枚举与默认值）；
//! REQ-TASK-SORT + design §2.9（排序键 status_rank → priority_rank → schedule_rank → sort_order → id DESC，
//! 未知/缺失沉底，稳定排序）；REQ-TASK-COMMENTS；DEC-41（独立 DB 实体）/ DEC-43（排期桶，无日期字段）。

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 任务状态（REQ-TASK-STATUS-5）：todo / in_progress / backlog / done / cancelled；新任务默认 todo
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    #[default]
    Todo,
    InProgress,
    Backlog,
    Done,
    Cancelled,
}

impl TaskStatus {
    /// 与 DB CHECK 字面值对账（TASK-108 枚举取值全集断言）
    pub fn as_db_value(&self) -> &'static str {
        match self {
            TaskStatus::Todo => "todo",
            TaskStatus::InProgress => "in_progress",
            TaskStatus::Backlog => "backlog",
            TaskStatus::Done => "done",
            TaskStatus::Cancelled => "cancelled",
        }
    }

    /// DB CHECK 字面值反解（回填读路径 · 与 as_db_value 互逆；未匹配 → None）。
    pub fn parse_db_value(s: &str) -> Option<Self> {
        match s {
            "todo" => Some(Self::Todo),
            "in_progress" => Some(Self::InProgress),
            "backlog" => Some(Self::Backlog),
            "done" => Some(Self::Done),
            "cancelled" => Some(Self::Cancelled),
            _ => None,
        }
    }
}

/// 任务优先级（REQ-TASK-PRIORITY-5）：none / low / medium / high / urgent；默认 none。
///
/// 权威值域以规格为准（spec L107）；design §3.2 DDL 原文 `'normal'` 为 T7-R1 勘误项
/// （本批同步修剪 design.md 并登记 DEC-63）。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Priority {
    #[default]
    None,
    Low,
    Medium,
    High,
    Urgent,
}

impl Priority {
    /// 与 DB CHECK 字面值对账（TASK-108 枚举取值全集断言）
    pub fn as_db_value(&self) -> &'static str {
        match self {
            Priority::None => "none",
            Priority::Low => "low",
            Priority::Medium => "medium",
            Priority::High => "high",
            Priority::Urgent => "urgent",
        }
    }

    /// DB CHECK 字面值反解（回填读路径 · 与 as_db_value 互逆；未匹配 → None）。
    pub fn parse_db_value(s: &str) -> Option<Self> {
        match s {
            "none" => Some(Self::None),
            "low" => Some(Self::Low),
            "medium" => Some(Self::Medium),
            "high" => Some(Self::High),
            "urgent" => Some(Self::Urgent),
            _ => None,
        }
    }
}

/// 排期桶（REQ-TASK-SCHEDULE-BUCKET，DEC-43）：today / tomorrow / this_week / later / unscheduled；
/// 默认 unscheduled；任务无到期日期字段、无日期选择器
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScheduleBucket {
    Today,
    Tomorrow,
    ThisWeek,
    Later,
    #[default]
    Unscheduled,
}

impl ScheduleBucket {
    /// 与 DB CHECK 字面值对账（TASK-108 枚举取值全集断言）
    pub fn as_db_value(&self) -> &'static str {
        match self {
            ScheduleBucket::Today => "today",
            ScheduleBucket::Tomorrow => "tomorrow",
            ScheduleBucket::ThisWeek => "this_week",
            ScheduleBucket::Later => "later",
            ScheduleBucket::Unscheduled => "unscheduled",
        }
    }

    /// DB CHECK 字面值反解（回填读路径 · 与 as_db_value 互逆；未匹配 → None）。
    pub fn parse_db_value(s: &str) -> Option<Self> {
        match s {
            "today" => Some(Self::Today),
            "tomorrow" => Some(Self::Tomorrow),
            "this_week" => Some(Self::ThisWeek),
            "later" => Some(Self::Later),
            "unscheduled" => Some(Self::Unscheduled),
            _ => None,
        }
    }
}

/// 任务（DEC-41 独立 DB 实体；`todos` 表投影）。
///
/// 无日期字段（DEC-43）；source 为可选来源（DEC-15 clip / v1.3a note 两态，无来源省略）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Todo {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub status: TaskStatus,
    pub priority: Priority,
    pub schedule_bucket: ScheduleBucket,
    /// ⌘⌥↑/⌘⌥↓ 移动排序写入（REQ-MOVE-KEYS）
    pub sort_order: i64,
    /// `TASK-n` 标识符（design §2.8 同事务分配）
    pub identifier: String,
    pub project_id: Option<Uuid>,
    pub source: Option<crate::content::Source>,
    /// 颜色标记七值下标（DEC-168 MARKS 同轴 0~6 · 默认赤=0；DEC-232 行左
    /// 标签底色承载——DB todos.mark 列 · schema v3）。serde 兼容兜底：不含
    /// mark 字段的历史导出档反解为默认赤（F-191-02 · M14 导入导出前向兼容）。
    #[serde(default)]
    pub mark: usize,
    pub created_at: String,
    pub modified_at: String,
}

/// 任务动态（REQ-TASK-COMMENTS：时间升序只读流；编辑不留痕/删除见 REQ-COMMENT-EDIT-DELETE）
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Comment {
    pub id: Uuid,
    pub todo_id: Uuid,
    pub content: String,
    pub created_at: String,
}

/// 排序投影行（design §3.4 `sort_tasks(tasks: &mut [TaskView])`）。
///
/// `Option` 承载「未知/缺失」：DB 脏值无法解析为枚举或字段缺失时为 `None`，一律沉底。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TaskView {
    pub id: Uuid,
    pub status: Option<TaskStatus>,
    pub priority: Option<Priority>,
    pub schedule_bucket: Option<ScheduleBucket>,
    pub sort_order: i64,
}

/// status_rank：in_progress(0) < todo(1) < backlog(2) < done(3) < cancelled(4)；
/// 未知/缺失 → usize::MAX 沉底（design §2.9，等价 ORDER BY CASE 补 ELSE）
pub fn status_rank(status: &Option<TaskStatus>) -> usize {
    match status {
        Some(TaskStatus::InProgress) => 0,
        Some(TaskStatus::Todo) => 1,
        Some(TaskStatus::Backlog) => 2,
        Some(TaskStatus::Done) => 3,
        Some(TaskStatus::Cancelled) => 4,
        None => usize::MAX,
    }
}

/// priority_rank：urgent(0) < high(1) < medium(2) < low(3) < none(4)；未知/缺失沉底
pub fn priority_rank(priority: &Option<Priority>) -> usize {
    match priority {
        Some(Priority::Urgent) => 0,
        Some(Priority::High) => 1,
        Some(Priority::Medium) => 2,
        Some(Priority::Low) => 3,
        Some(Priority::None) => 4,
        None => usize::MAX,
    }
}

/// schedule_rank：today(0) < tomorrow(1) < this_week(2) < later(3) < unscheduled(4)（DEC-43）；
/// 未知/缺失仍显式沉底（design §2.9：DB CHECK 拦脏值，内存态兜底）
pub fn schedule_rank(bucket: &Option<ScheduleBucket>) -> usize {
    match bucket {
        Some(ScheduleBucket::Today) => 0,
        Some(ScheduleBucket::Tomorrow) => 1,
        Some(ScheduleBucket::ThisWeek) => 2,
        Some(ScheduleBucket::Later) => 3,
        Some(ScheduleBucket::Unscheduled) => 4,
        None => usize::MAX,
    }
}

/// REQ-TASK-SORT / design §2.9：排序键 (status_rank, priority_rank, schedule_rank, sort_order, id DESC)。
///
/// Rust `sort_by` 为稳定排序（REQ-TASK-SORT「排序必须稳定」）；id DESC 作平手 tiebreaker。
pub fn sort_tasks(tasks: &mut [TaskView]) {
    tasks.sort_by(|a, b| {
        status_rank(&a.status)
            .cmp(&status_rank(&b.status))
            .then_with(|| priority_rank(&a.priority).cmp(&priority_rank(&b.priority)))
            .then_with(|| schedule_rank(&a.schedule_bucket).cmp(&schedule_rank(&b.schedule_bucket)))
            .then_with(|| a.sort_order.cmp(&b.sort_order))
            .then_with(|| b.id.cmp(&a.id)) // id DESC（design §2.9 Reverse(id) 等价）
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    const CHECK_STATUSES: [&str; 5] = ["todo", "in_progress", "backlog", "done", "cancelled"];
    const CHECK_PRIORITIES: [&str; 5] = ["none", "low", "medium", "high", "urgent"];
    const CHECK_BUCKETS: [&str; 5] = ["today", "tomorrow", "this_week", "later", "unscheduled"];

    /// 字面值反解互逆全覆盖（回填读路径契约：as_db_value ↔ parse_db_value）。
    #[test]
    fn db_value_parse_roundtrip_all_variants() {
        for s in CHECK_STATUSES {
            let parsed = TaskStatus::parse_db_value(s).expect(s);
            assert_eq!(parsed.as_db_value(), s);
        }
        for s in CHECK_PRIORITIES {
            let parsed = Priority::parse_db_value(s).expect(s);
            assert_eq!(parsed.as_db_value(), s);
        }
        for s in CHECK_BUCKETS {
            let parsed = ScheduleBucket::parse_db_value(s).expect(s);
            assert_eq!(parsed.as_db_value(), s);
        }
        assert_eq!(TaskStatus::parse_db_value("bogus"), None);
        assert_eq!(Priority::parse_db_value("Bogus"), None);
        assert_eq!(ScheduleBucket::parse_db_value(""), None);
    }

    fn v(
        id: u8,
        status: Option<TaskStatus>,
        priority: Option<Priority>,
        bucket: Option<ScheduleBucket>,
        sort_order: i64,
    ) -> TaskView {
        TaskView {
            id: Uuid::from_u128(id as u128),
            status,
            priority,
            schedule_bucket: bucket,
            sort_order,
        }
    }

    #[test]
    fn status_enum_covers_db_check_value_set() {
        let all = [
            TaskStatus::Todo,
            TaskStatus::InProgress,
            TaskStatus::Backlog,
            TaskStatus::Done,
            TaskStatus::Cancelled,
        ];
        for (i, s) in all.iter().enumerate() {
            assert_eq!(s.as_db_value(), CHECK_STATUSES[i]);
            assert_eq!(
                serde_json::to_string(s).unwrap(),
                format!("\"{}\"", CHECK_STATUSES[i])
            );
        }
    }

    #[test]
    fn priority_enum_covers_db_check_value_set() {
        let all = [
            Priority::None,
            Priority::Low,
            Priority::Medium,
            Priority::High,
            Priority::Urgent,
        ];
        for (i, p) in all.iter().enumerate() {
            assert_eq!(p.as_db_value(), CHECK_PRIORITIES[i]);
            assert_eq!(
                serde_json::to_string(p).unwrap(),
                format!("\"{}\"", CHECK_PRIORITIES[i])
            );
        }
        // T7-R1：权威值域为 medium（design DDL 'normal' 已勘误，DEC-63）
        assert!(CHECK_PRIORITIES.contains(&"medium"));
        assert!(!CHECK_PRIORITIES.contains(&"normal"));
    }

    #[test]
    fn schedule_bucket_enum_covers_db_check_value_set() {
        let all = [
            ScheduleBucket::Today,
            ScheduleBucket::Tomorrow,
            ScheduleBucket::ThisWeek,
            ScheduleBucket::Later,
            ScheduleBucket::Unscheduled,
        ];
        for (i, b) in all.iter().enumerate() {
            assert_eq!(b.as_db_value(), CHECK_BUCKETS[i]);
            assert_eq!(
                serde_json::to_string(b).unwrap(),
                format!("\"{}\"", CHECK_BUCKETS[i])
            );
        }
    }

    #[test]
    fn defaults_match_spec() {
        // REQ-TASK-STATUS-5 / PRIORITY-5 / SCHEDULE-BUCKET 默认值
        assert_eq!(TaskStatus::default(), TaskStatus::Todo);
        assert_eq!(Priority::default(), Priority::None);
        assert_eq!(ScheduleBucket::default(), ScheduleBucket::Unscheduled);
    }

    #[test]
    fn group_order_is_in_progress_todo_backlog_done_cancelled() {
        // SCN-TASK-SORT-RANK：分组顺序断言
        let mut list = vec![
            v(1, Some(TaskStatus::Done), Some(Priority::None), None, 0),
            v(2, Some(TaskStatus::Todo), Some(Priority::None), None, 0),
            v(
                3,
                Some(TaskStatus::Cancelled),
                Some(Priority::None),
                None,
                0,
            ),
            v(
                4,
                Some(TaskStatus::InProgress),
                Some(Priority::None),
                None,
                0,
            ),
            v(5, Some(TaskStatus::Backlog), Some(Priority::None), None, 0),
        ];
        sort_tasks(&mut list);
        let statuses: Vec<TaskStatus> = list.iter().filter_map(|t| t.status).collect();
        assert_eq!(
            statuses,
            vec![
                TaskStatus::InProgress,
                TaskStatus::Todo,
                TaskStatus::Backlog,
                TaskStatus::Done,
                TaskStatus::Cancelled
            ]
        );
    }

    #[test]
    fn secondary_keys_priority_then_bucket_then_sort_order_then_id_desc() {
        let mut list = vec![
            // 同 (status=todo, priority=none, bucket=unscheduled)：sort_order 决序
            v(10, Some(TaskStatus::Todo), Some(Priority::None), None, 2),
            v(11, Some(TaskStatus::Todo), Some(Priority::None), None, 1),
            // 同 (status=todo)：priority 决序（urgent 在前）
            v(20, Some(TaskStatus::Todo), Some(Priority::Urgent), None, 9),
            v(21, Some(TaskStatus::Todo), Some(Priority::Low), None, 9),
        ];
        sort_tasks(&mut list);
        assert_eq!(list[0].id, Uuid::from_u128(20)); // urgent
        assert_eq!(list[1].id, Uuid::from_u128(21)); // low
        assert_eq!(list[2].id, Uuid::from_u128(11)); // sort_order 1
        assert_eq!(list[3].id, Uuid::from_u128(10)); // sort_order 2

        // 同 (status, priority, sort_order)：schedule_rank 决序
        let mut list = vec![
            v(
                30,
                Some(TaskStatus::Todo),
                Some(Priority::None),
                Some(ScheduleBucket::Later),
                5,
            ),
            v(
                31,
                Some(TaskStatus::Todo),
                Some(Priority::None),
                Some(ScheduleBucket::Today),
                5,
            ),
        ];
        sort_tasks(&mut list);
        assert_eq!(list[0].id, Uuid::from_u128(31)); // today < later
    }

    #[test]
    fn id_desc_is_tiebreaker() {
        let mut list = vec![
            v(40, Some(TaskStatus::Todo), Some(Priority::None), None, 0),
            v(41, Some(TaskStatus::Todo), Some(Priority::None), None, 0),
        ];
        sort_tasks(&mut list);
        // id DESC：大 id 在前
        assert_eq!(list[0].id, Uuid::from_u128(41));
        assert_eq!(list[1].id, Uuid::from_u128(40));
    }

    #[test]
    fn dirty_data_sinks_to_bottom_and_not_lost() {
        // SCN-TASK-SORT-DIRTY-SINKS：未知/缺失 status/priority/bucket 沉底且不丢失
        let mut list = vec![
            v(50, Some(TaskStatus::Done), Some(Priority::None), None, 0),
            v(51, None, None, None, 0),                   // status 缺失
            v(52, Some(TaskStatus::Todo), None, None, 0), // priority 缺失
            v(53, Some(TaskStatus::Todo), Some(Priority::High), None, 0),
            v(54, Some(TaskStatus::Todo), Some(Priority::None), None, 0), // bucket 缺失
            v(
                55,
                Some(TaskStatus::Todo),
                Some(Priority::None),
                Some(ScheduleBucket::Today),
                0,
            ),
        ];
        sort_tasks(&mut list);
        // 正常数据在前：53(urgent 高优先) → 55(有桶) → 54(无桶) → 52(优先级缺失沉底)
        assert_eq!(list[0].id, Uuid::from_u128(53));
        assert_eq!(list[1].id, Uuid::from_u128(55));
        assert_eq!(list[2].id, Uuid::from_u128(54));
        assert_eq!(list[3].id, Uuid::from_u128(52));
        // status 缺失=全局沉底（最后）
        assert_eq!(list[5].id, Uuid::from_u128(51));
        // 不丢失：原 6 条全在
        assert_eq!(list.len(), 6);
    }

    #[test]
    fn sort_is_deterministic_across_calls() {
        // REQ-TASK-SORT「排序必须稳定」的承载：sort_by 稳定算法 + 五键含 id DESC 对互异 id
        // 构成全序（等键全等的唯一情形是重复投影行，同样保持输入序）——以双次排序一致性佐证。
        let mut list = vec![
            v(60, Some(TaskStatus::Todo), None, None, 0),
            v(61, Some(TaskStatus::Todo), None, None, 0),
            v(62, Some(TaskStatus::Todo), None, None, 0),
        ];
        sort_tasks(&mut list);
        let first = list.clone();
        sort_tasks(&mut list);
        assert_eq!(list, first);
        // 除 id 外全等时由 id DESC 定序（大 id 在前）
        assert_eq!(list[0].id, Uuid::from_u128(62));
        assert_eq!(list[2].id, Uuid::from_u128(60));
    }

    #[test]
    fn todo_source_two_kinds_roundtrip() {
        // TASK-108：source_kind 两态 roundtrip 断言（clip / note）+ 无来源省略
        let base = Todo {
            id: Uuid::nil(),
            title: "整理速记".into(),
            description: String::new(),
            status: TaskStatus::default(),
            priority: Priority::default(),
            schedule_bucket: ScheduleBucket::default(),
            sort_order: 0,
            identifier: "TASK-7".into(),
            project_id: None,
            source: None,
            mark: 0,
            created_at: "2026-10-17T08:00:00Z".into(),
            modified_at: "2026-10-17T08:00:00Z".into(),
        };
        let back: Todo = serde_json::from_str(&serde_json::to_string(&base).unwrap()).unwrap();
        assert_eq!(back, base);
        assert!(back.source.is_none()); // DEC-15：无来源省略

        let with_clip = Todo {
            source: Some(crate::content::Source::Clip {
                clip_id: Uuid::nil(),
                source_app: Some("Safari".into()),
                captured_at: "2026-10-17T07:00:00Z".into(),
            }),
            ..base.clone()
        };
        let back: Todo = serde_json::from_str(&serde_json::to_string(&with_clip).unwrap()).unwrap();
        assert!(matches!(
            back.source,
            Some(crate::content::Source::Clip { .. })
        ));

        let with_note = Todo {
            source: Some(crate::content::Source::Note {
                ref_id: Uuid::nil(),
                source_app: None,
                captured_at: "2026-10-17T07:00:00Z".into(),
            }),
            ..base.clone()
        };
        let back: Todo = serde_json::from_str(&serde_json::to_string(&with_note).unwrap()).unwrap();
        assert!(matches!(
            back.source,
            Some(crate::content::Source::Note { .. })
        ));
    }

    #[test]
    fn comment_roundtrip() {
        let c = Comment {
            id: Uuid::nil(),
            todo_id: Uuid::nil(),
            content: "已排到今天".into(),
            created_at: "2026-10-17T08:00:00Z".into(),
        };
        let back: Comment = serde_json::from_str(&serde_json::to_string(&c).unwrap()).unwrap();
        assert_eq!(back, c);
    }
}
