//! 队列纯逻辑模型（F/CAP-4 · REQ-QUEUE-PLACE · TASK-301 数据面）：
//! 入队快照、FIFO/LIFO 消费序视图、重复内容独立条目、清空。承诺式投递
//! 状态机（settleDelay 350ms / activationWindow 0.3s / stop 补写 placed /
//! QueueEvent 发布）已随 TASK-301 实装于同 crate `promise.rs`——本文件
//! 仅承载入队/消费序数据面模型，引擎实装位以 promise.rs 为准。
//!
//! 快照口径（SCN-QUEUE-SNAPSHOT-ISOLATED）：条目持入队时的载荷拷贝，与
//! 拾记历史行无引用关系——历史上限清理/删除不波及队列条目。
//! 重复口径：入队恒新增独立条目，不判重不合并（同内容可重复独立入队）。

/// 消费序模式（REQ-QUEUE-PLACE · design §13.3 ④）：FIFO 默认——先入队先消耗；
/// LIFO 可切——后入队先消耗（SCN-QUEUE-ORDER-LIFO 首耗最新项）。
/// 模式持久在队列对象上（clear/stop 不复位——与用户切换意图同向）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum QueueOrder {
    #[default]
    Fifo,
    Lifo,
}

/// 队列条目（入队时快照）：`id`＝单调递增独立身份（同内容重复入队各自
/// 持 id——渲染以 id 为稳定元素键）；`text`＝入队时载荷文本（快照隔离）。
/// 二进制载荷（image 随 TASK-301 引擎 promise 载荷）不在本面。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueueEntry {
    pub id: u64,
    pub text: String,
}

/// 队列（逐条入队 → 按序逐条粘贴的工作面；多选入队承载随 TASK-502
/// 拾记多选机制接线）。数据面只管「持有什么 · 按什么序展示」；投递/
/// 消耗判定归引擎（TASK-301）。
#[derive(Debug, Clone, Default)]
pub struct ClipQueue {
    entries: Vec<QueueEntry>,
    order: QueueOrder,
    next_id: u64,
}

impl ClipQueue {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            order: QueueOrder::Fifo,
            next_id: 1,
        }
    }

    /// 消费序模式（只读面）。
    pub fn order(&self) -> QueueOrder {
        self.order
    }

    /// 切换 FIFO⇄LIFO（design §13.3 ②）：单向翻转，返回切换后模式。
    pub fn toggle_order(&mut self) -> QueueOrder {
        self.order = match self.order {
            QueueOrder::Fifo => QueueOrder::Lifo,
            QueueOrder::Lifo => QueueOrder::Fifo,
        };
        self.order
    }

    /// 入队（快照拷贝）：内容恒入列——同内容重复＝独立条目（不判重不合并），
    /// 返回本条 id。
    pub fn enqueue(&mut self, text: &str) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.entries.push(QueueEntry {
            id,
            text: text.to_string(),
        });
        id
    }

    /// 消费序视图（design §13.3 ⑦）：首个＝下一消耗项（FIFO 最旧入队 /
    /// LIFO 最新入队），随后按剩余序（LIFO 整列倒置）。快照拷贝返回
    ///（幂等视图、不消费——消耗动作随引擎）。
    pub fn view(&self) -> Vec<QueueEntry> {
        let mut out = self.entries.clone();
        if self.order == QueueOrder::Lifo {
            out.reverse();
        }
        out
    }

    /// 按 id 摘除单条（TASK-301 消耗流）：引擎兑现消耗后宿主据其摘除
    ///（进行中条目不再依视图首定位——Settling 期 toggle 后 view 首变，消耗
    /// 对象仍为 placed 实体）。miss 返回 None（幂等）。
    pub fn remove_by_id(&mut self, id: u64) -> Option<QueueEntry> {
        let pos = self.entries.iter().position(|e| e.id == id)?;
        Some(self.entries.remove(pos))
    }

    /// 清空（SCN-QUEUE-STOP-FIXUP 终态口径：停止/清空后队列不持任何条目）；
    /// 返回清掉条数。消费序模式不复位。
    pub fn clear(&mut self) -> usize {
        let n = self.entries.len();
        self.entries.clear();
        n
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// 重复入队＝独立条目（REQ-QUEUE-PLACE）：同文本两次入队各自持 id 与快照，
/// 均在消费序内；FIFO 视图＝入队序。
#[test]
fn enqueue_duplicate_content_creates_independent_entries() {
    let mut q = ClipQueue::new();
    let a = q.enqueue("同文同本");
    let b = q.enqueue("同文同本");
    assert_ne!(a, b, "重复入队＝独立条目（各自 id）");
    assert_eq!(q.len(), 2, "两快照并存");
    assert_eq!(
        q.view(),
        vec![
            QueueEntry {
                id: a,
                text: "同文同本".into()
            },
            QueueEntry {
                id: b,
                text: "同文同本".into()
            },
        ],
        "FIFO 视图＝入队序"
    );
}

/// FIFO 视图序（SCN-QUEUE-ORDER-FIFO）：首个＝最先入队（下一消耗项）。
#[test]
fn view_fifo_order_oldest_first() {
    let mut q = ClipQueue::new();
    q.enqueue("甲");
    q.enqueue("乙");
    q.enqueue("丙");
    let view = q.view();
    let seq: Vec<&str> = view.iter().map(|e| e.text.as_str()).collect();
    assert_eq!(seq, ["甲", "乙", "丙"], "FIFO 视图＝入队序（首耗最旧）");
}

/// LIFO 视图序（SCN-QUEUE-ORDER-LIFO · §13.3 ⑦「整列倒置首耗最新项」）：
/// 切换后 A/B/C 顺序入队则首耗 C。
#[test]
fn view_lifo_order_newest_first() {
    let mut q = ClipQueue::new();
    q.enqueue("甲");
    q.enqueue("乙");
    q.enqueue("丙");
    q.toggle_order();
    assert_eq!(q.order(), QueueOrder::Lifo);
    let view = q.view();
    let seq: Vec<&str> = view.iter().map(|e| e.text.as_str()).collect();
    assert_eq!(seq, ["丙", "乙", "甲"], "LIFO 视图＝入队倒序（首耗最新）");
}

/// ⇄ 双向来回切换 + 模式跨 clear 保留（FIFO 默认）。
#[test]
fn toggle_order_round_trip_and_survives_clear() {
    let mut q = ClipQueue::new();
    assert_eq!(q.order(), QueueOrder::Fifo, "FIFO 默认");
    q.enqueue("甲");
    q.toggle_order();
    q.clear();
    assert_eq!(q.order(), QueueOrder::Lifo, "清空不复位模式");
    q.toggle_order();
    assert_eq!(q.order(), QueueOrder::Fifo);
}

/// 清空返回清掉条数（宿主侧对账承载）；清空后视图无残留。
#[test]
fn clear_returns_removed_count() {
    let mut q = ClipQueue::new();
    q.enqueue("甲");
    q.enqueue("乙");
    assert_eq!(q.clear(), 2);
    assert!(q.is_empty());
    assert_eq!(q.view(), Vec::<QueueEntry>::new(), "清空后视图无残留");
}

/// 快照隔离（SCN-QUEUE-SNAPSHOT-ISOLATED）：条目持入队时的拷贝值——入队后
/// 变更来源串不波及条目（值语义；无引用逃逸到拾记行）。
#[test]
fn enqueue_snapshots_payload() {
    let mut q = ClipQueue::new();
    let mut text = String::from("原始载荷");
    q.enqueue(&text);
    text.replace_range(.., "入队后被改的源面");
    assert_eq!(q.view()[0].text, "原始载荷", "条目持入队时快照");
}

/// 按 id 摘除（消耗流）：命中摘单条；重复 id 不存在（单调分配）；miss 幂等
/// None。LIFO 序下按 id 摘不受展示倒位影响（消耗对象=placed 实体）。
#[test]
fn remove_by_id_picks_exactly_one() {
    let mut q = ClipQueue::new();
    let a = q.enqueue("甲");
    let b = q.enqueue("乙");
    let c = q.enqueue("丙");
    assert_eq!(q.remove_by_id(b).map(|e| e.id), Some(b), "命中摘单条");
    assert_eq!(q.len(), 2, "其余保留");
    assert!(q.view().iter().any(|e| e.id == a), "甲仍在");
    assert!(q.view().iter().any(|e| e.id == c), "丙仍在");
    assert!(q.remove_by_id(999).is_none(), "miss 幂等 None");
    // FIFO 消耗语义对位：摘首=下一消耗项。
    assert_eq!(q.remove_by_id(a).map(|e| e.text), Some("甲".into()));
    assert_eq!(q.view()[0].id, c);
}
