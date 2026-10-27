//! 承诺式投递状态机（TASK-301 · REQ-QUEUE-DETECT · design §2.3/§11.4）：
//! 队列上架承诺式剪贴板（惰性 data provider），外部应用索取数据即为粘贴
//! 信号。判定链（权威 = 规格「回填与队列」REQ-QUEUE-DETECT）：
//!
//! ```text
//! place ─→ Placed{entry} ─demand─→ Settling{settle_at(=索取时刻), replace}
//!                                       │ 350ms 结算窗（拍制到期兑现）
//!            ┌──────────────────────────┴──┐
//!      [非探查·SCN-QUEUE-CONSUME]      [探查·SCN-QUEUE-EXPLORE-BLOCKED]
//!         消耗：entry 流出宿主            重申承诺（板面替换为同条新承诺）
//!      （宿主摘除本体→下一拍 reconcile    不消耗、顺序不变、无事件
//!        自动上架后继）                  （探查不进提示层）
//! ```
//!
//! - **探查判定**：索取时刻距最近一次应用激活 <0.3s → 探查（onReplace＝重申
//!   同条承诺；对照 CB 原语言「Task{sleep(350); if justActivated …}」——判定以
//!   索取时刻为准、分支延后 350ms 兑现，语义同构）。
//! - **停止/清空**（SCN-QUEUE-STOP-FIXUP）：持板即以 placed 项**真实数据**
//!   写回（终态——板面解除承诺，用户后续 ⌘V 不再触发队列）；Idle 态无 fixup
//!   直出 NotHeld（事件是否发布由宿主 gate——SCN-QUEUE-NOTIFY 清空「曾运行」）。
//! - **抑制承载**（§11.4 S4 · TASK-718 验证面）：承诺板自带 TransientType
//!   隐私标记→capture 守护读板恒 Rejected（「skip_next_change 计数」的天然
//!   等价承载，无计数器同步风险）；真实写回经宿主 clipstamp 前置登记抑制。
//!   设计骨架 `Pasteboard::skip_next_change`（design §3.x）以此承载，登记
//!   偏差（无独立计数器）。
//! - **时间源**：`now: Instant` 参数逐拍注入（无隐式时钟——DEC-186 先例），
//!   结算窗基点＝桥承载的索取时刻（回调线程记录），拍延迟不侵蚀 350ms 语义。
//! - **350/300/0.1 拍制**：结算窗到期由宿主轮询兑现（QUEUE_POLL_SECS 0.1s 独立
//!   引擎拍），兑现点＝due 判定通过的首拍——总时限「索取→消耗」≤0.45s，
//!   CB 原 0.35s 精确制的拍制偏差登记 DEC。

use std::time::{Duration, Instant};

use super::queue::QueueEntry;

/// 结算延迟（SCN-QUEUE-CONSUME「350ms 结算」· design §2.3 定数唯一源）。
pub const SETTLE_DELAY_MS: u64 = 350;

/// 激活探查窗（SCN-QUEUE-EXPLORE-BLOCKED「0.3s」· design §2.3 定数唯一源）。
pub const ACTIVATION_WINDOW_MS: u64 = 300;

/// 停止/清空语义分型（SCN-QUEUE-STOP-FIXUP 两种触发同用 fixup；事件分型
/// 供 §11.2 节流表「队列停止/队列清空」两行各自提示）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StopKind {
    /// 停止（队列层停止钮 / Esc 中断——隐式停止全链）。
    Stop,
    /// 清空（队列层清空钮）。
    Clear,
}

/// 承诺板注入面（生产 = app 装配 platform 承诺桥；测试 = 假板注入）。
/// 「真实写回」的抑制登记（ui clipstamp `mark`）由**宿主装配层前置**——
/// 本 trait 面（capture 侧）无 ui 依赖（铁律 2 分层单向不变量）。
pub trait PromiseBoard {
    /// 承诺上架：板面持文本惰性承诺＋TransientType 隐私标记（REQ-QUEUE-PLACE
    /// 快照 / SCN-QUEUE-CURRENT-HOST-ONLY 由桥承载）。false = 桥失败。
    fn declare_promise(&mut self, text: &str) -> bool;
    /// 真实数据写回（stop 补写终态；写板 changeCount 前进由宿主守护抑制）。
    fn write_real_text(&mut self, text: &str) -> bool;
    /// 索取信号读点（消费式）：Some(索取时刻)——时刻由桥在 provider 回调
    /// 线程记录（无拍延迟）。
    fn drain_demand(&mut self) -> Option<Instant>;
    /// 最近一次应用激活时刻（NSWorkspace didActivateApplication 事件槽）。
    fn last_activation(&self) -> Option<Instant>;
}

/// 引擎产出（宿主事件组装语义源）：
#[derive(Debug, Clone, PartialEq, Eq)]
enum PromiseState {
    /// 不持板（队列条目全在宿主本体的 view 内）。
    Idle,
    /// 承诺上架中：entry=进行中条目（入队快照拷贝）。placed_at 时刻戳已删
    ///（R184 F-183-22：声明性字段无消费点——grep 全量核验仅构造/搬运，
    /// 无比较无 duration 读取）。
    Placed { entry: QueueEntry },
    /// 结算窗挂起：replace=索取时刻的探查预判（SCN-QUEUE-EXPLORE-BLOCKED）。
    Settling {
        entry: QueueEntry,
        settle_at: Instant,
        replace: bool,
    },
}

/// 停止产出：NotHeld=无持板（无 fixup）；Released=持板释放（entry=补写对象
/// ·written=真实写回是否成功——写失败 stderr 由宿主按此披露）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StopOutcome {
    NotHeld,
    Released { entry: QueueEntry, written: bool },
}

/// 队列事件（TASK-301 · design §11.2 提示层输入）：投递/消耗/停止/清空
/// 四型——`QueueNotifier`（C-14）双通道编排消费（在役 `crate::notify`；
/// TASK-716）。探查替换（onReplace 重申）不发布事件——
/// 探查不进提示层（SCN-QUEUE-EXPLORE-BLOCKED 无提示语义）。
/// remaining = 动作后剩余待耗数（宿主 view 口径）；released = 停止/清空
/// 释放的条数（placed 项 ＋ 本体剩余——终态全清）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueueEventKind {
    /// 承诺上架（首次入/重申后继——「队列拥有剪贴板」态起点）。
    Placed { remaining: usize },
    /// 真实粘贴消耗一条（remaining=0 即队列终成——SCN-QUEUE-NOTIFY-EMPTY-）。
    Consumed { remaining: usize },
    /// 停止（补写完成口径随 attached）。
    Stopped { released: usize },
    /// 清空（宿主 gate「队列曾运行过」——空态清空不上报）。
    Cleared,
}

/// 队列承诺式投递状态机（引擎本体；队列数据本体在宿主 `ClipQueue`，引擎只
/// 持「进行中条目」快照）。单拍编排（place/note/fulfill/stop）由宿主守
/// 护拍驱动。
#[derive(Debug)]
pub struct QueueEngine {
    state: PromiseState,
}

impl Default for QueueEngine {
    fn default() -> Self {
        Self {
            state: PromiseState::Idle,
        }
    }
}

impl QueueEngine {
    pub fn new() -> Self {
        Self {
            state: PromiseState::Idle,
        }
    }

    /// 持板读点（Esc 中断检查 / fixup 对账用）：Placed/Settling → Some(entry)。
    pub fn holding(&self) -> Option<&QueueEntry> {
        match &self.state {
            PromiseState::Placed { entry, .. } | PromiseState::Settling { entry, .. } => {
                Some(entry)
            }
            PromiseState::Idle => None,
        }
    }

    /// 承诺上架：declare 成功 → Placed；失败 → 保持 Idle（宿主 stderr 披露，
    /// 下拍 reconcile 自然重试——真机桥类缺失等程序性异常自愈承载）。
    pub fn place_promise(&mut self, entry: &QueueEntry, board: &mut dyn PromiseBoard) -> bool {
        if !matches!(self.state, PromiseState::Idle) {
            return false; // 已持板/结算中——重申由宿主 reconcile 显式驱动
        }
        if !board.declare_promise(&entry.text) {
            return false;
        }
        self.state = PromiseState::Placed {
            entry: entry.clone(),
        };
        true
    }

    /// 重申承诺（Placed 态专用 · SCN-QUEUE-ORDER-LIFO 切换收敛）：把进行中
    /// 条目替换为新序首项并重新上架（板面 changeCount 前进→守护 Rejected
    /// 吞掉）。非 Placed 态 / 桥失败 → false（宿主不动作，下拍 reconcile
    /// 依 placed.id≠view 首项再收敛）。Settling 态不重申（结算窗内物理板
    /// 不得被换——避免「重申覆盖未结算承诺」的信号错乱）。
    pub fn replace_promise(&mut self, entry: &QueueEntry, board: &mut dyn PromiseBoard) -> bool {
        if !matches!(self.state, PromiseState::Placed { .. }) {
            return false;
        }
        if !board.declare_promise(&entry.text) {
            return false;
        }
        self.state = PromiseState::Placed {
            entry: entry.clone(),
        };
        true
    }

    /// 索取信号转结算：Placed 态吸收一枚 demand → Settling（replace=索取
    /// 时刻探查预判）。Settling 态重复 demand 恒 false（「一次 placement 只
    /// 认第一次索取」——350ms 结算窗内同粘贴多类型不重复触发）。
    pub fn note_demand(&mut self, board: &mut dyn PromiseBoard) -> bool {
        let PromiseState::Placed { entry, .. } = &self.state else {
            return false;
        };
        let Some(demand_at) = board.drain_demand() else {
            return false;
        };
        // 探查判定（以索取时刻为基准点 · CB 同构）：激活时刻不得晚于索取
        //（时钟异常序（la > demand_at）保守判非探查，防 duration_since 回退 panic）。
        let replace = match board.last_activation() {
            Some(la) if la <= demand_at => {
                demand_at.duration_since(la) < Duration::from_millis(ACTIVATION_WINDOW_MS)
            }
            _ => false,
        };
        self.state = PromiseState::Settling {
            entry: entry.clone(),
            settle_at: demand_at,
            replace,
        };
        true
    }

    /// 结算窗到期判定（宿主拍驱动：due 首拍兑现）。
    pub fn settle_due(&self, now: Instant) -> bool {
        match &self.state {
            PromiseState::Settling { settle_at, .. } => {
                now.duration_since(*settle_at) >= Duration::from_millis(SETTLE_DELAY_MS)
            }
            _ => false,
        }
    }

    /// 结算兑现（settle_due 首拍后调用）：
    /// - **非探查** → 消耗返回 `Some(entry)`（宿主据其摘除队列本体→组成
    ///   Consumed 事件→引擎转 Idle，下拍 reconcile 自动上架后继）；
    /// - **探查** → 引擎内重申承诺（板面替换为同条新承诺——不消耗顺序不
    ///   变）返回 `None`；重申失败 → 转 Idle（下拍 reconcile 对 view 首项重新
    ///   上架，自愈同位）。
    pub fn fulfill_settle(&mut self, board: &mut dyn PromiseBoard) -> Option<QueueEntry> {
        let PromiseState::Settling { entry, replace, .. } = &self.state else {
            return None;
        };
        if !*replace {
            let PromiseState::Settling { entry, .. } =
                std::mem::replace(&mut self.state, PromiseState::Idle)
            else {
                unreachable!("分支前置已核状态");
            };
            return Some(entry);
        }
        let entry = entry.clone();
        if !board.declare_promise(&entry.text) {
            // 重申失败：板面已非本队列承诺（探查读取真值仍在板）——转 Idle
            //（条目保留宿主本体，下拍 reconcile 重试上架同条）。
            self.state = PromiseState::Idle;
            return None;
        }
        self.state = PromiseState::Placed { entry };
        None
    }

    /// 停止/清空（SCN-QUEUE-STOP-FIXUP）：持板 → placed 真实数据写回（写
    /// 抑制由宿主 clipstamp 前置登记）并转 Idle，返回 Released；Idle 态无
    /// fixup 直出 NotHeld（队列本体清空与事件 gate 均在宿主承载）。
    pub fn stop(&mut self, board: &mut dyn PromiseBoard, kind: StopKind) -> StopOutcome {
        let Some(entry) = self.holding().cloned() else {
            return StopOutcome::NotHeld;
        };
        let _ = kind; // fixup 动作同型；分型只影响宿主事件与提示语
        let written = board.write_real_text(&entry.text);
        self.state = PromiseState::Idle;
        StopOutcome::Released { entry, written }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 假板（时序全族测试注入）：declare/write 记调用序列；demand/activation 直给。
    struct FakeBoard {
        declare_calls: Vec<String>,
        write_calls: Vec<String>,
        demand: Vec<Instant>,
        activation: Option<Instant>,
        /// declare 可控失败（桥异常路径）。
        declare_ok: bool,
    }

    impl FakeBoard {
        fn new() -> Self {
            Self {
                declare_calls: Vec::new(),
                write_calls: Vec::new(),
                demand: Vec::new(),
                activation: None,
                declare_ok: true,
            }
        }
    }

    impl PromiseBoard for FakeBoard {
        fn declare_promise(&mut self, text: &str) -> bool {
            self.declare_calls.push(text.to_string());
            self.declare_ok
        }
        fn write_real_text(&mut self, text: &str) -> bool {
            self.write_calls.push(text.to_string());
            true
        }
        fn drain_demand(&mut self) -> Option<Instant> {
            if self.demand.is_empty() {
                None
            } else {
                Some(self.demand.remove(0))
            }
        }
        fn last_activation(&self) -> Option<Instant> {
            self.activation
        }
    }

    fn entry(id: u64, text: &str) -> QueueEntry {
        QueueEntry {
            id,
            text: text.to_string(),
        }
    }

    /// 常量定数（design §2.3 官方出处——350ms 结算 / 0.3s 探查窗）：防改值回归。
    #[test]
    fn timing_constants_match_design() {
        assert_eq!(SETTLE_DELAY_MS, 350, "SCN-QUEUE-CONSUME 350ms 结算");
        assert_eq!(
            ACTIVATION_WINDOW_MS, 300,
            "SCN-QUEUE-EXPLORE-BLOCKED 0.3s 探查窗"
        );
    }

    /// place 成功 → 持板 + 版面 declare 调用（SCN-QUEUE-CONSUME 前置）；
    /// declare 失败（桥异常）→ 保持 Idle 不假持板。
    #[test]
    fn place_promise_declares_or_refuses() {
        let mut eng = QueueEngine::new();
        let mut b = FakeBoard::new();
        assert!(eng.place_promise(&entry(1, "甲"), &mut b));
        assert_eq!(b.declare_calls, ["甲"], "上架声明一次");
        assert_eq!(eng.holding().map(|e| e.id), Some(1), "Placed 持板");
        // 已持板重 place 拒绝（重申走 replace_promise 专用面）。
        assert!(!eng.place_promise(&entry(2, "乙"), &mut b));
        // 桥失败：不置假态（下拍 reconcile 重试承载）。
        let mut eng2 = QueueEngine::new();
        let mut bad = FakeBoard {
            declare_ok: false,
            ..FakeBoard::new()
        };
        assert!(!eng2.place_promise(&entry(3, "丙"), &mut bad));
        assert!(eng2.holding().is_none(), "上架失败不假持板");
    }

    /// 非探查全链（SCN-QUEUE-CONSUME）：place → demand → 350ms 内未到期 →
    /// 窗满首拍兑现 → 消耗流出 → 引擎 Idle 允许后继上架。
    #[test]
    fn demand_settles_consumes_after_window() {
        let mut eng = QueueEngine::new();
        let mut b = FakeBoard::new();
        assert!(eng.place_promise(&entry(1, "甲"), &mut b));
        let t0 = Instant::now();
        // demand 事件（t0 时刻索取）。
        b.demand.push(t0);
        assert!(eng.note_demand(&mut b), "Settling 转入");
        // 结算窗内不兑现。
        assert!(!eng.settle_due(t0 + Duration::from_millis(349)));
        // 窗满首拍兑现 → entry 流出（宿主摘本体）。
        assert!(eng.settle_due(t0 + Duration::from_millis(SETTLE_DELAY_MS)));
        assert_eq!(eng.fulfill_settle(&mut b), Some(entry(1, "甲")), "消耗流出");
        assert!(eng.holding().is_none(), "Idle（下拍 reconcile 上架后继）");
        // 后继可再上架（promote 承载面）。
        assert!(eng.place_promise(&entry(2, "乙"), &mut b));
        assert_eq!(b.declare_calls, ["甲", "乙"]);
    }

    /// 探查窗内索取（SCN-QUEUE-EXPLORE-BLOCKED）：0.3s 内激活 → onReplace＝
    /// 重申承诺（declare 第二次）；不消耗、顺序不变、无 entry 流出。
    #[test]
    fn activation_window_demand_replaces_without_consume() {
        let mut eng = QueueEngine::new();
        let mut b = FakeBoard::new();
        assert!(eng.place_promise(&entry(7, "甲"), &mut b));
        let t0 = Instant::now();
        b.activation = Some(t0 - Duration::from_millis(100)); // 0.1s 前激活
        b.demand.push(t0);
        assert!(eng.note_demand(&mut b));
        assert!(eng.settle_due(t0 + Duration::from_millis(SETTLE_DELAY_MS)));
        assert_eq!(eng.fulfill_settle(&mut b), None, "探查不消耗");
        assert_eq!(b.declare_calls.len(), 2, "重申承诺（板面替换为同条新承诺）");
        assert!(b.write_calls.is_empty(), "重申不写真值（承诺仍在役）");
        assert_eq!(eng.holding().map(|e| e.id), Some(7), "顺序不变");
    }

    /// 窗界外激活（400ms 前）→ 正常消耗；时钟异常序（激活晚于索取）保守
    /// 判非探查。
    #[test]
    fn activation_outside_window_or_inverted_clock_consumes() {
        let mut eng = QueueEngine::new();
        let mut b = FakeBoard::new();
        assert!(eng.place_promise(&entry(1, "甲"), &mut b));
        let t0 = Instant::now();
        b.activation = Some(t0 - Duration::from_millis(400)); // 0.4s 前激活 → 窗外
        b.demand.push(t0);
        assert!(eng.note_demand(&mut b));
        assert!(eng.settle_due(t0 + Duration::from_millis(SETTLE_DELAY_MS)));
        assert_eq!(
            eng.fulfill_settle(&mut b),
            Some(entry(1, "甲")),
            "窗外激活 = 真实粘贴消耗"
        );
        // 异常序（激活时刻晚于索取时刻）→ 保守非探查。
        let mut eng2 = QueueEngine::new();
        let mut b2 = FakeBoard::new();
        assert!(eng2.place_promise(&entry(1, "甲"), &mut b2));
        let t1 = Instant::now();
        b2.activation = Some(t1 + Duration::from_millis(50)); // 激活在索取之后
        b2.demand.push(t1);
        assert!(eng2.note_demand(&mut b2));
        assert!(eng2.settle_due(t1 + Duration::from_millis(SETTLE_DELAY_MS)));
        assert_eq!(
            eng2.fulfill_settle(&mut b2),
            Some(entry(1, "甲")),
            "保守消耗"
        );
    }

    /// 一次 placement 只认第一次索取：Settling 态重复 demand 不重触发。
    #[test]
    fn repeated_demand_within_settling_is_ignored() {
        let mut eng = QueueEngine::new();
        let mut b = FakeBoard::new();
        assert!(eng.place_promise(&entry(1, "甲"), &mut b));
        let t0 = Instant::now();
        b.demand.push(t0);
        assert!(eng.note_demand(&mut b));
        // 结算窗内第二枚 demand（同粘贴的另一类型）→ 拒收不重结算。
        b.demand.push(t0 + Duration::from_millis(10));
        assert!(!eng.note_demand(&mut b), "一次 placement 只认第一次索取");
        assert!(!b.demand.is_empty(), "未吸收留在槽内自愈");
        assert!(eng.settle_due(t0 + Duration::from_millis(SETTLE_DELAY_MS)));
        assert_eq!(eng.fulfill_settle(&mut b), Some(entry(1, "甲")));
    }

    /// 停止补写（SCN-QUEUE-STOP-FIXUP）：持板 stop → placed 真实数据写回；
    /// Idle stop → NotHeld 无写板（终态直出，无 fixup 对象）。
    #[test]
    fn stop_fixes_up_holding_board_real_data() {
        let mut eng = QueueEngine::new();
        let mut b = FakeBoard::new();
        assert!(eng.place_promise(&entry(1, "甲"), &mut b));
        let outcome = eng.stop(&mut b, StopKind::Stop);
        assert_eq!(
            outcome,
            StopOutcome::Released {
                entry: entry(1, "甲"),
                written: true
            },
            "持板释放 + 写回承载"
        );
        assert_eq!(b.write_calls, ["甲"], "placed 项真实数据写回");
        assert!(eng.holding().is_none(), "终态 Idle");
        // Idle 态 stop：NotHeld（无 fixup 对象 直出）。
        let outcome2 = eng.stop(&mut b, StopKind::Clear);
        assert_eq!(outcome2, StopOutcome::NotHeld);
        assert!(b.write_calls.len() == 1, "Idle 清空不写板");
    }

    /// Settling 态 stop：placed 项同样补写（结算窗挂起≠板面已失去承诺——
    /// 未消耗前仍持板）。
    #[test]
    fn stop_during_settling_still_fixes_up() {
        let mut eng = QueueEngine::new();
        let mut b = FakeBoard::new();
        assert!(eng.place_promise(&entry(1, "甲"), &mut b));
        b.demand.push(Instant::now());
        assert!(eng.note_demand(&mut b));
        assert_eq!(eng.holding().map(|e| e.id), Some(1), "Settling 仍持板");
        let outcome = eng.stop(&mut b, StopKind::Stop);
        assert!(
            matches!(outcome, StopOutcome::Released { .. }),
            "挂起期同样 fixup"
        );
        assert_eq!(b.write_calls, ["甲"]);
    }

    /// toggle 收敛（replace_promise）：Placed 态换序首项重申；Settling 态
    /// 拒绝（窗口内物理板不得被换）；失败保持原 placed 不假换序。
    #[test]
    fn replace_promise_converges_toggle_in_placed_only() {
        let mut eng = QueueEngine::new();
        let mut b = FakeBoard::new();
        assert!(eng.place_promise(&entry(1, "甲"), &mut b));
        // LIFO 切换 → 新首项乙 → 重申。
        assert!(eng.replace_promise(&entry(3, "丙"), &mut b));
        assert_eq!(b.declare_calls, ["甲", "丙"], "重申一次（换序后新首项）");
        assert_eq!(eng.holding().map(|e| e.id), Some(3), "进行中条目已收敛");
        // Settling 态拒绝重申（物理板不得在结算窗内被换）。
        let t0 = Instant::now();
        b.demand.push(t0);
        assert!(eng.note_demand(&mut b));
        assert!(
            !eng.replace_promise(&entry(2, "乙"), &mut b),
            "Settling 不重申"
        );
        assert_eq!(b.declare_calls.len(), 2, "未追加 declare");
        assert_eq!(eng.holding().map(|e| e.id), Some(3), "保持原条目待结算");
        // 上架失败：不假换序（declare 失败 → 保持原条目）。
        let mut b2 = FakeBoard::new();
        let mut eng2 = QueueEngine::new();
        assert!(eng2.place_promise(&entry(1, "甲"), &mut b2), "先正常上架");
        b2.declare_ok = false; // 后续 declare 全失败
        b2.declare_calls.clear();
        assert!(!eng2.replace_promise(&entry(2, "乙"), &mut b2), "换序失败");
        assert_eq!(b2.declare_calls, ["乙"], "失败调用已尝试并记于册");
        assert_eq!(eng2.holding().map(|e| e.id), Some(1), "换序失败保持原条目");
    }
}
