//! 队列提示编排（TASK-716 · design §11.2 定数口径 · C-14）：「弹一弹＋系统通
//! 知」双通道状态机。本层为纯逻辑——事件分型进、提示动作出，逐行按 §11.2 触
//! 发点表承载；平台面（Dock attention / UserNotifications / 面板动效注入）由
//! 宿主执行层装配（capture 侧无 gpui 依赖 · 铁律 2 分层单向不变量）。授权三态
//! 门：未决/拒绝 → 通知静默（首投丢失可接受 · §11.2 授权语义），「弹一弹」动
//! 效不受授权影响。

use crate::promise::QueueEventKind;

/// 队列通知标题（C5 隐私口径：内容只到「标题＋剩余条数」，不含剪贴板文本）。
pub const QUEUE_NOTIFY_TITLE: &str = "Parchora 队列";

/// 节流间隔下界（REQ-SETTINGS-QUEUE-NOTIFY-INTERVAL · 1~20 可配）。
pub const QUEUE_NOTIFY_INTERVAL_MIN: u64 = 1;
/// 节流间隔上界（REQ-SETTINGS-QUEUE-NOTIFY-INTERVAL · 1~20 可配）。
pub const QUEUE_NOTIFY_INTERVAL_MAX: u64 = 20;
/// 节流间隔默认值（DEC-48 定数 · N=5）。
pub const QUEUE_NOTIFY_INTERVAL_DEFAULT: u64 = 5;

/// 提示动作产出（§11.2 表逐行的执行面分型——宿主按能力域执行：Dock/通知走
/// platform 桥，面板 bounce 走面板注入链，授权走异步请求）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueuePromptAction {
    /// Dock 图标弹一弹（NSApplication.requestUserAttention Informational）。
    DockAttention,
    /// 面板队列按钮 bounce 一轮（gpui with_animation · §11.2 ①面板内动效）。
    PanelBounce,
    /// 向系统发起通知授权请求（仅首次需要发送时 · SCN-QUEUE-NOTIFY-AUTH-FIRST）。
    AuthorizeRequest,
    /// 系统通知投递（标题恒 `QUEUE_NOTIFY_TITLE`，body 携剩余/释放条数）。
    SystemNotify { body: String },
}

/// 队列提示编排状态机（C-14）：消费 `QueueEventKind` 四型事件，按 §11.2 触发
/// 点表产出提示动作。会话内存态——无持久化面（design §11.2「无新增持久化
/// 状态」）。
#[derive(Debug)]
pub struct QueueNotifier {
    /// 节流间隔 N（每 N 条消耗发一次进度通知 · 改值立即生效——set_interval）。
    interval: u64,
    /// 自上次进度通知后的累计消耗条数（节流计数 · 通知后清零重计）。
    consumed_since_notify: u64,
    /// 本轮运行累计消耗条数（运行边界＝队列变空；完成提示带出后复位）。
    consumed_total: u64,
    /// 授权请求已发起（首次需要送通知时一次 · SCN-QUEUE-NOTIFY-AUTH-FIRST）。
    auth_requested: bool,
    /// 最近一次 Dock 注意请求 id（通知点击路由撤回用——面板已可见无需再弹）。
    last_attention: Option<u64>,
}

impl Default for QueueNotifier {
    fn default() -> Self {
        Self::new(QUEUE_NOTIFY_INTERVAL_DEFAULT)
    }
}

impl QueueNotifier {
    /// 指定节流间隔起算（设置档回填承载——越界值钳回界内）。
    pub fn new(interval: u64) -> Self {
        let mut n = Self {
            interval,
            consumed_since_notify: 0,
            consumed_total: 0,
            auth_requested: false,
            last_attention: None,
        };
        n.set_interval(n.interval);
        n
    }

    /// 节流间隔读点（设置播种/持久化）。
    pub fn interval(&self) -> u64 {
        self.interval
    }

    /// 节流间隔改值（设置项变更加值立即生效 · REQ-SETTINGS-QUEUE-NOTIFY-
    /// INTERVAL：立即对后续队列消耗生效——不做任何延迟/重播种语义）。
    pub fn set_interval(&mut self, n: u64) {
        self.interval = n.clamp(QUEUE_NOTIFY_INTERVAL_MIN, QUEUE_NOTIFY_INTERVAL_MAX);
        // 改值不重置节流计数（立即生效＝以新阈值继续判，最小语义面）。
    }

    /// 记录最近一次 Dock 注意请求 id（宿主执行 DockAttention 动作后回填）。
    pub fn note_attention(&mut self, id: u64) {
        self.last_attention = Some(id);
    }

    /// 取走最近注意请求 id（点击路由撤回用 · 取走后清空）。
    pub fn take_attention(&mut self) -> Option<u64> {
        self.last_attention.take()
    }

    /// 最近注意请求 id 读点（非破坏 · 测试与路由撤回前置判断用）。
    pub fn last_attention_id(&self) -> Option<u64> {
        self.last_attention
    }

    /// 事件进、动作出（§11.2 表逐行 · 纯逻辑）：
    ///
    /// | 事件 | 弹一弹 | 系统通知 |
    /// |---|---|---|
    /// | 首次入队并投递（Placed） | ✅ Dock | ✅（含首次授权请求） |
    /// | 每消耗一条 | ✅ 仅面板按钮 | ❌（节流） |
    /// | 每 N 条消耗 | —— | ✅「队列进行中…还剩 k 条」 |
    /// | 停止（补写完成） | ✅ Dock | ✅「队列已停止，n 条已投递」 |
    /// | 清空 | ✅（一次性完成动效） | ✅（gate 由宿主承载） |
    ///
    /// 完成特例（SCN-QUEUE-NOTIFY-EMPTY-FINISH）：最后一条消耗（变空）→ 弹
    /// 一弹＋通知（完成提示），且运行计数复位起算。
    pub fn process(&mut self, ev: QueueEventKind, authorized: bool) -> Vec<QueuePromptAction> {
        let mut actions = Vec::new();
        match ev {
            QueueEventKind::Placed { remaining } => {
                // 授权请求时机：首次需要发送通知时（队列首次投递）一次。
                if !self.auth_requested {
                    self.auth_requested = true;
                    actions.push(QueuePromptAction::AuthorizeRequest);
                }
                actions.push(QueuePromptAction::DockAttention);
                // 未获授权 → 通知不呈现（静默禁用）；动效不受影响（上已发）。
                if authorized {
                    actions.push(QueuePromptAction::SystemNotify {
                        body: format!("队列开始投递，待粘贴 {remaining} 条"),
                    });
                }
            }
            QueueEventKind::Consumed { remaining } => {
                self.consumed_total += 1;
                // 每条消耗仅面板按钮 bounce（不弹 Dock · §11.2 表）。
                actions.push(QueuePromptAction::PanelBounce);
                if remaining == 0 {
                    // 完成特例：弹一弹＋通知（完成提示）——计数复位（下一轮
                    // 运行新起算）。
                    actions.push(QueuePromptAction::DockAttention);
                    if authorized {
                        let total = self.consumed_total;
                        actions.push(QueuePromptAction::SystemNotify {
                            body: format!("队列已完成，共投递 {total} 条"),
                        });
                    }
                    self.consumed_since_notify = 0;
                    self.consumed_total = 0;
                } else {
                    // 每 N 条消耗发一次进度通知（节流计数自上次通知后重新起算
                    // · SCN-QUEUE-NOTIFY-CONSUME）。
                    self.consumed_since_notify += 1;
                    if self.consumed_since_notify >= self.interval {
                        self.consumed_since_notify = 0;
                        if authorized {
                            actions.push(QueuePromptAction::SystemNotify {
                                body: format!("队列进行中…还剩 {remaining} 条"),
                            });
                        }
                    }
                }
            }
            QueueEventKind::Stopped { released } => {
                // 运行边界复位（R184 F-183-17）：停止即本轮运行止——计数清零，
                // 下一轮完成提示不带上轮余量（修复前 consumed_total 跨停止
                // 累加，停止后再开新轮的完成提示多报）。
                self.consumed_total = 0;
                self.consumed_since_notify = 0;
                actions.push(QueuePromptAction::DockAttention);
                if authorized {
                    actions.push(QueuePromptAction::SystemNotify {
                        body: format!("队列已停止，{released} 条已投递"),
                    });
                }
            }
            QueueEventKind::Cleared => {
                // 清空动效（一次性完成动效）＋通知（宿主 gate「队列曾运行过」
                // 已在事件面承载——gate 不过不进本层）。运行边界复位（R184
                // F-183-17）同 Stopped 口。
                self.consumed_total = 0;
                self.consumed_since_notify = 0;
                actions.push(QueuePromptAction::DockAttention);
                if authorized {
                    actions.push(QueuePromptAction::SystemNotify {
                        body: "队列已清空".to_string(),
                    });
                }
            }
        }
        actions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// §11.2 表逐行断言驱动器（授权态直入 · 纯逻辑无 mock 桥——动作分型即断言面）。
    fn placed(remaining: usize) -> QueueEventKind {
        QueueEventKind::Placed { remaining }
    }
    fn consumed(remaining: usize) -> QueueEventKind {
        QueueEventKind::Consumed { remaining }
    }

    /// 首次入队投递（SCN-QUEUE-NOTIFY-FIRST-ENQUEUE）：Dock 动效＋通知＋首次
    /// 授权请求；Placed 不动面板按钮（按钮 bounce 归「每条消耗」行）。
    #[test]
    fn placed_first_enqueue_dock_and_notify_with_auth_request() {
        let mut n = QueueNotifier::default();
        let acts = n.process(placed(3), true);
        assert_eq!(
            acts,
            vec![
                QueuePromptAction::AuthorizeRequest,
                QueuePromptAction::DockAttention,
                QueuePromptAction::SystemNotify {
                    body: "队列开始投递，待粘贴 3 条".into(),
                },
            ],
            "首投三件齐（授权请求＋Dock＋通知）"
        );
        // 二次 Placed 不再重复请求授权（SCN-QUEUE-NOTIFY-AUTH-FIRST 一次语义）。
        let acts = n.process(placed(2), true);
        assert!(
            !acts.contains(&QueuePromptAction::AuthorizeRequest),
            "授权请求仅首次"
        );
        assert!(
            !acts.contains(&QueuePromptAction::PanelBounce),
            "Placed 无按钮 bounce"
        );
    }

    /// 授权未决（首投授权进行中）→ 通知静默、Dock 动效照常；请求已发起。
    #[test]
    fn placed_pending_auth_is_silent_but_bounces() {
        let mut n = QueueNotifier::default();
        let acts = n.process(placed(4), false);
        assert!(
            !acts
                .iter()
                .any(|a| matches!(a, QueuePromptAction::SystemNotify { .. })),
            "未决授权不送通知（静默）"
        );
        assert!(
            acts.contains(&QueuePromptAction::DockAttention),
            "动效不受授权影响"
        );
        assert!(
            acts.contains(&QueuePromptAction::AuthorizeRequest),
            "授权请求仍发起"
        );
    }

    /// 每条消耗仅按钮 bounce（不弹 Dock · 不发通知）。
    #[test]
    fn consume_is_panel_bounce_only() {
        let mut n = QueueNotifier::default();
        n.set_interval(20);
        let acts = n.process(consumed(4), true);
        assert_eq!(
            acts,
            vec![QueuePromptAction::PanelBounce],
            "常规消耗仅按钮 bounce（间隔未到）"
        );
    }

    /// 停止/清空复位运行计数（R184 F-183-17）：运行边界＝停止/清空——复位后
    /// 新轮完成提示只计新轮消耗，不再累加旧轮（修复前跨边界累计多报）。
    #[test]
    fn stop_and_clear_reset_run_counters() {
        // Stopped 边界：老轮消耗 2 条後停止，复位，新轮第 1 条即终空。
        let mut n = QueueNotifier::default();
        n.set_interval(20);
        let _ = n.process(consumed(3), true);
        let _ = n.process(consumed(2), true);
        let stop = n.process(QueueEventKind::Stopped { released: 2 }, true);
        assert!(
            stop.contains(&QueuePromptAction::DockAttention),
            "停止仍弹一弹（复位不改变停止面）"
        );
        let acts = n.process(consumed(0), true);
        let total = acts.iter().rev().find_map(|a| match a {
            QueuePromptAction::SystemNotify { body } => Some(body.clone()),
            _ => None,
        });
        assert_eq!(
            total.as_deref(),
            Some("队列已完成，共投递 1 条"),
            "复位后完成提示只计新轮（修复前累加为 3 条）"
        );
        // Cleared 边界同款。
        let mut n = QueueNotifier::default();
        n.set_interval(20);
        let _ = n.process(consumed(5), true);
        let _ = n.process(QueueEventKind::Cleared, true);
        let acts = n.process(consumed(0), true);
        let total = acts.iter().rev().find_map(|a| match a {
            QueuePromptAction::SystemNotify { body } => Some(body.clone()),
            _ => None,
        });
        assert_eq!(
            total.as_deref(),
            Some("队列已完成，共投递 1 条"),
            "清空复位后只计新轮"
        );
    }

    /// 每 N 条消耗发进度通知（DEC-48 · N=5 默认；计数自上次通知后重计）。
    #[test]
    fn consume_every_nth_fires_progress_notify() {
        let mut n = QueueNotifier::default(); // N=5
        for i in (1..=4).rev() {
            let acts = n.process(consumed(i), true);
            assert!(
                !acts
                    .iter()
                    .any(|a| matches!(a, QueuePromptAction::SystemNotify { .. })),
                "前 4 条不发通知（剩余 {i}）"
            );
        }
        let acts = n.process(consumed(0), true);
        // 第 5 条即队列终空 → 完成特例行（非进度行）。
        assert!(
            acts.contains(&QueuePromptAction::DockAttention),
            "完成特例含弹一弹"
        );
        assert!(
            acts.iter()
                .any(|a| matches!(a, QueuePromptAction::SystemNotify { body }
                if body == "队列已完成，共投递 5 条" )),
            "完成提示带累计数"
        );
    }

    /// 进度节流计到 N 即发（间隔未终空态）：N=2 第二条即发；通知后计数重计。
    #[test]
    fn progress_throttle_recounts_after_notify() {
        let mut n = QueueNotifier::new(2);
        let acts = n.process(consumed(5), true);
        assert!(
            !acts
                .iter()
                .any(|a| matches!(a, QueuePromptAction::SystemNotify { .. })),
            "首条不发（计数 1 < 2）"
        );
        let acts = n.process(consumed(4), true);
        assert!(
            acts.iter()
                .any(|a| matches!(a, QueuePromptAction::SystemNotify { body }
                if body == "队列进行中…还剩 4 条" )),
            "第 2 条触发进度通知"
        );
        // 通知后计数重计：下一条不发。
        let acts = n.process(consumed(3), true);
        assert!(
            !acts
                .iter()
                .any(|a| matches!(a, QueuePromptAction::SystemNotify { .. })),
            "通知后计数重新起算"
        );
    }

    /// 停止补写完成（SCN-QUEUE-NOTIFY-STOP-FIXUP）：Dock＋通知（带释放条数）。
    #[test]
    fn stopped_fixup_notifies_released() {
        let mut n = QueueNotifier::default();
        let acts = n.process(QueueEventKind::Stopped { released: 3 }, true);
        assert!(acts.contains(&QueuePromptAction::DockAttention));
        assert!(acts
            .iter()
            .any(|a| matches!(a, QueuePromptAction::SystemNotify { body }
                if body == "队列已停止，3 条已投递" )));
    }

    /// 清空（宿主 gate 在事件面前已承载）：Dock＋通知。
    #[test]
    fn cleared_notifies_once() {
        let mut n = QueueNotifier::default();
        let acts = n.process(QueueEventKind::Cleared, true);
        assert!(acts.contains(&QueuePromptAction::DockAttention));
        assert!(acts
            .iter()
            .any(|a| matches!(a, QueuePromptAction::SystemNotify { body }
                if body == "队列已清空" )));
    }

    /// 拒绝授权（AUTH-FIRST 拒绝面）：通知全静默、动效照常（动效通道不依赖授权）。
    #[test]
    fn denied_auth_keeps_bounce_silent_notify() {
        let mut n = QueueNotifier::default();
        n.process(placed(3), false);
        let acts = n.process(consumed(2), false);
        assert_eq!(
            acts,
            vec![QueuePromptAction::PanelBounce],
            "拒绝态消耗仅动效"
        );
    }

    /// 运行边界复位：变空后计数清零（下一轮完成提示从 1 起算）。
    #[test]
    fn run_boundary_resets_counters() {
        let mut n = QueueNotifier::default();
        n.process(placed(1), true);
        n.process(consumed(0), true);
        n.process(placed(1), true);
        let acts = n.process(consumed(0), true);
        assert!(
            acts.iter()
                .any(|a| matches!(a, QueuePromptAction::SystemNotify { body }
                if body == "队列已完成，共投递 1 条" )),
            "复位后新一轮从 1 起算"
        );
    }

    /// 间隔改值立即生效（set_interval 钳界 · REQ-SETTINGS-QUEUE-NOTIFY-INTERVAL）。
    #[test]
    fn interval_set_clamps_and_applies_immediately() {
        let mut n = QueueNotifier::default();
        n.set_interval(25);
        assert_eq!(n.interval(), 20, "上界钳制");
        n.set_interval(0);
        assert_eq!(n.interval(), 1, "下界钳制");
        // 立即生效：间隔 1 → 任意消耗即触发进度通知。
        n.process(consumed(3), true);
        let acts = n.process(consumed(2), true);
        assert!(
            acts.iter()
                .any(|a| matches!(a, QueuePromptAction::SystemNotify { body }
                if body == "队列进行中…还剩 2 条" )),
            "改值后立即生效"
        );
    }

    /// 注意请求 id 登记/取走（点击路由撤回面）。
    #[test]
    fn attention_id_noted_and_taken_once() {
        let mut n = QueueNotifier::default();
        assert_eq!(n.take_attention(), None);
        n.note_attention(7);
        assert_eq!(n.take_attention(), Some(7));
        assert_eq!(n.take_attention(), None, "取走即清");
    }
}
