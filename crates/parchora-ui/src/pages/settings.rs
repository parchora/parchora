//! PG-SET 设置窗口（UI-6 · TASK-509 SettingsView / REQ-SETTINGS-*）。
//!
//! HTML 1:1（PG-SET.html）：430 独立窗 · 左导航 120px（10 分区 node 行）+ 右内容
//! （「分区名+设置」题注 · 6px 下距 + .sec 行组 p10）；面板分区含引导卡（tertiary 边）
//! 与 WAL 提示（neutral 底）。10 分区行集为 TASK-509 各分区要点承载。任一设置项
//! 变更即刻持久化（宿主 env 档 · DEC-85）+ hover 于设置窗面板之外 0.2s 自动保存
//! 并关窗（settings_watch 宿主承载；仅移出设置窗即关——即便光标位于主面板/
//! Peek 等其它在册 GUI 上，变更已即时落档）。
//!
//! 触发位置交互化（DEC-67 · DEC-84）：PG-SET
//! 「边缘触发侧」kv 行升级为「触发位置」7 选项 pill 组（选中=chip.on 暖底主墨；
//! 呈现序=ALL 常量），点选经 SettingsSink 上抛宿主 apply_trigger_pos（DockPrefs 全局
//! 落值 + 停靠窗关开重排，即时生效）并 Toast 确认；被 Dock 占用的缘 pill 置灰
//! 禁选（DockAvailability 全局 · 应用启动/屏帧变化落值 · REQ-PANEL-TRIGGER-POSITION
//! 可用性），冲突自动切换由宿主 resolve_conflict 承载。
//!
//! 输入监控引导卡交互化（DEC-81 · REQ-PANEL-LISTEN-GRACEFUL）：
//! 卡按 `ListenAuth` 全局授权态条件渲染（granted 不显示）；点击「打开系统设置」
//! Toast 即时反馈 + 上抛 `SettingsEvent::ListenAuthRequest`，宿主后台执行授权请求
//! （TCC 弹窗即引导入口）并回写授权态（授权到位卡即消隐，守护拍复探同口径）。
//!
//! 剪贴板历史保留时长（DEC-178 · 存储分区数值输入 · 天）：输入引擎延迟建＋
//! PressEnter/blur/设置窗关窗漏斗三口提交——合法且变更即生效（上抛
//! `SettingsEvent::ClipRetentionChanged` → 宿主落 ClipCapture + env 档即刻
//! 持久化），非法回退显示（不生效不落盘），等值 no-op。

use std::time::Duration;

/// ─── 镜像回退默认值（F-191-01 · 仅 SettingsView::new 构造兜底——宿主开窗
/// ─── 播种覆盖；取值锚定 app 层默认档 · DEC-163/188/48，改动须两仓同改）───
pub(crate) const DEFAULT_RETENTION_DAYS: u64 = 1;
pub(crate) const DEFAULT_CAPTURE_ENABLED: bool = true;
pub(crate) const DEFAULT_CAPTURE_LIMIT: u64 = 500;
pub(crate) const DEFAULT_QUEUE_NOTIFY_INTERVAL: u64 = 5;

use gpui::prelude::FluentBuilder as _;
use gpui::{
    div, img, px, rgb, App, AppContext as _, ClickEvent, Context, Div, Entity,
    InteractiveElement as _, ParentElement as _, Render, StatefulInteractiveElement as _, Styled,
    Window,
};
use gpui_component as ui;
use ui::input::{Input, InputEvent, InputState};

use crate::dock::{DockAvailability, DockPrefs, TriggerPosition};
use crate::{components as co, scale as s, tokens as t};

/// 10 分区（REQ-SETTINGS-SECTIONS，顺序=HTML 左导航）。
const SECTIONS: [&str; 10] = [
    "通用",
    "外观",
    "面板",
    "编辑器",
    "快捷键",
    "存储",
    "同步",
    "更新",
    "标签管理",
    "关于",
];

/// 设置 → 宿主事件（DEC-67：触发位置变更即时生效；DEC-81：输入监控授权请求；
/// DEC-178：剪贴板历史保留时长变更；DEC-188：捕获开关/历史上限档/排除应用——
/// 宿主落 ClipCapture + 即刻持久化）。
pub enum SettingsEvent {
    /// 触发位置变更（7 向 · DEC-84 · 单源 dock::TriggerPosition）
    TriggerPosChanged(TriggerPosition),
    /// 输入监控授权请求（引导卡点击；宿主后台执行授权请求并回写 ListenAuth）
    ListenAuthRequest,
    /// 剪贴板历史保留时长变更（天 · DEC-178 · 正整数；设置存储页数值输入提交）
    ClipRetentionChanged(u64),
    /// 剪贴板监控开关变更（SCN-CAPTURE-SWITCH-OFF · DEC-188 · 即时生效；
    /// 宿主 apply_capture_enabled 重播种 engine 基线）
    CaptureEnabledChanged(bool),
    /// 历史上限档变更（REQ-CAPTURE-LIMIT · DEC-188 · 五档即选即生效；
    /// 0=不限）
    CaptureLimitChanged(u64),
    /// 排除应用列表变更（REQ-CAPTURE-EXCLUDE · DEC-188 · 逗号分隔 bundleId；
    /// 归一化后上抛）
    CaptureExcludeChanged(String),
    /// 队列通知节流间隔变更（REQ-SETTINGS-QUEUE-NOTIFY-INTERVAL · DEC-48/209 ·
    /// 1~20 整数即选即生效）
    QueueNotifyIntervalChanged(u64),
}

/// 设置 → 宿主上抛通道（app 层 apply_trigger_pos）。
pub type SettingsSink = Box<dyn Fn(&SettingsEvent, &mut App) + 'static>;

/// 输入监控授权态全局（DEC-81 · REQ-PANEL-LISTEN-GRACEFUL）：app 层落值——启动
/// preflight 探测（只读不弹提示）、授权请求返回、守护拍复探变化时回写。设置引导卡
/// 按此条件渲染（granted = 不显示卡）。未初始化（纯 ui 语境/无宿主演示）视为
/// 已授权（避免误报卡）。
pub struct ListenAuth {
    pub listen_granted: bool,
}

impl gpui::Global for ListenAuth {}

/// 设置窗口视图。
pub struct SettingsView {
    section: usize,
    /// 演示 Toast（触发位置变更确认 · 1400ms 自动消失）
    toast: Option<String>,
    /// Toast 自动消失防争代次（gen 失效防争 · 与 app peek_close_gen 同模式）
    toast_gen: u64,
    /// 未决 Toast timer（gpui::Task 持于字段；防争由 toast_gen 代次承载 · DEC-72）
    toast_task: Option<gpui::Task<()>>,
    /// 设置 → 宿主事件
    on_settings_event: Option<SettingsSink>,
    /// 剪贴板历史保留时长（天 · DEC-178）：宿主 set_retention_days 开窗播种
    ///（单一事实源在 app 层 ClipCapture · 镜像仅承载输入初值）+ commit 生效回写。
    retention_days: u64,
    /// 保留时长数值输入引擎（存储分区渲染首帧延迟建 · tasks ensure_title 同式）
    retention_input: Option<Entity<InputState>>,
    /// 输入提交订阅（PressEnter/Blur → commit_retention；实体活期常驻不摘）
    retention_sub: Option<gpui::Subscription>,
    /// 剪贴板监控开关镜像（SCN-CAPTURE-SWITCH-OFF · DEC-188）：开窗播种 +
    /// 切换回写（单一事实源同 retention 口径）。
    capture_enabled: bool,
    /// 历史上限档镜像（REQ-CAPTURE-LIMIT · DEC-188 · 0=不限）。
    capture_limit: u64,
    /// 排除应用镜像（REQ-CAPTURE-EXCLUDE · DEC-188 · 逗号分隔 bundleId）。
    exclude_apps: String,
    /// 排除应用输入引擎（存储分区渲染首帧延迟建 · retention 同式）。
    exclude_input: Option<Entity<InputState>>,
    /// 排除应用提交订阅（PressEnter/Blur → commit_exclude）。
    exclude_sub: Option<gpui::Subscription>,
    /// 队列通知节流间隔镜像（REQ-SETTINGS-QUEUE-NOTIFY-INTERVAL · DEC-48/209 ·
    /// 宿主开窗播种 → 即选即生效回写；单一事实源在 app 层 QueueHost）。
    queue_notify_interval: u64,
}

impl SettingsView {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        // 镜像回退默认值语义常量（F-191-01 · P2.1 收敛）：仅构造回退兜底
        //（宿主开窗播种为单一事实源）；取值与 app 层默认档一致
        //（CLIP_RETENTION_DAYS_DEFAULT=1 / CLIP_LIMIT_DEFAULT=500 /
        // QUEUE 节流默认 N=5 · DEC-163/188/48）。
        Self {
            section: 2,
            toast: None,
            toast_gen: 0,
            toast_task: None,
            on_settings_event: None,
            retention_days: DEFAULT_RETENTION_DAYS,
            retention_input: None,
            retention_sub: None,
            capture_enabled: DEFAULT_CAPTURE_ENABLED,
            capture_limit: DEFAULT_CAPTURE_LIMIT,
            exclude_apps: String::new(),
            exclude_input: None,
            exclude_sub: None,
            queue_notify_interval: DEFAULT_QUEUE_NOTIFY_INTERVAL,
        } // 默认落「面板」分区（HTML 演示态）
    }

    /// Toast：1400ms 自动消失（UI-TOAST；新 Toast bump 代次 + timer 持于字段）。
    fn show_toast(&mut self, msg: String, cx: &mut Context<Self>) {
        self.toast = Some(msg);
        self.toast_gen += 1;
        let generation = self.toast_gen;
        self.toast_task = Some(cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(Duration::from_millis(t::TOAST_MS))
                .await;
            this.update(cx, |v, cx| {
                if v.toast_gen == generation {
                    v.toast = None;
                    cx.notify();
                }
            })
            .ok();
        }));
        cx.notify();
    }

    pub fn set_event_sink(&mut self, sink: SettingsSink) {
        self.on_settings_event = Some(sink);
    }

    /// 保留时长播种（DEC-178 · 宿主开窗期调用——输入引擎未建时仅落镜像，
    /// ensure_retention 建引擎自此读值；调用后再改镜像不回写已建引擎）。
    pub fn set_retention_days(&mut self, days: u64) {
        self.retention_days = days;
    }

    /// 捕获域三件播种（DEC-188 · 宿主开窗期调用 · 镜像仅承载初值）：监控开关 /
    /// 历史上限档 / 排除应用串。
    pub fn set_capture_enabled(&mut self, on: bool) {
        self.capture_enabled = on;
    }

    pub fn set_capture_limit(&mut self, limit: u64) {
        self.capture_limit = limit;
    }

    pub fn set_exclude_apps(&mut self, joined: String) {
        self.exclude_apps = joined;
    }

    /// 队列通知节流间隔播种（DEC-209 · 宿主开窗期调用）：通用分区 chips 行初值。
    pub fn set_queue_notify_interval(&mut self, interval: u64) {
        self.queue_notify_interval = interval;
    }

    /// 剪贴板历史保留时长行（DEC-178）：左 label（kv 同式）＋右数值输入框
    ///（64×24 · SURFACE 底 BORDER 边 · 引擎延迟建）＋单位「天」。
    fn retention_row(&self) -> Div {
        let input = self.retention_input.clone().map(|st| {
            div()
                .debug_selector(move || "set-retention-input".to_string())
                .w(px(64.0))
                .h(px(24.0))
                .px(px(6.0))
                .bg(rgb(t::SURFACE))
                .border_1()
                .border_color(rgb(t::BORDER))
                .rounded(px(t::RAD_SM))
                .overflow_hidden()
                .flex()
                .items_center()
                .child(Input::new(&st).appearance(false).bordered(false))
        });
        div()
            .flex()
            .justify_between()
            .child(co::txt(
                "剪贴板历史保留时长",
                s::font(s::Lvl::Small),
                t::PRIMARY,
            ))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.0))
                    .children(input)
                    .child(co::txt("天", s::font(s::Lvl::Small), t::MUTED)),
            )
    }

    /// 保留时长数值输入引擎延迟建（DEC-178 · 存储分区渲染首帧调用）：初值＝
    /// 当前生效天数镜像（set_retention_days 播种）；订阅 PressEnter/Blur 提交
    ///（editor 行内编辑同式 · subscribe_in 回调持 Window——非法回退 set_value 需）。
    fn ensure_retention(&mut self, w: &mut Window, cx: &mut Context<Self>) {
        if self.retention_input.is_some() {
            return;
        }
        let seed = self.retention_days.to_string();
        let st = cx.new(|cx| {
            let mut state = InputState::new(w, cx);
            state.set_placeholder("1", w, cx);
            state
        });
        st.update(cx, |state, cx| state.set_value(seed, w, cx));
        let sub = cx.subscribe_in(
            &st,
            w,
            |this: &mut Self,
             _st: &Entity<InputState>,
             ev: &InputEvent,
             w,
             cx: &mut Context<Self>| match ev {
                InputEvent::PressEnter { .. } | InputEvent::Blur => {
                    this.commit_retention(w, cx);
                }
                _ => {}
            },
        );
        self.retention_input = Some(st);
        self.retention_sub = Some(sub);
        cx.notify();
    }

    /// 保留时长提交（DEC-178 · enter/blur/设置窗关闭漏斗三口）：合法且变更 →
    /// 生效 + Toast + 上抛宿主（apply_clip_retention_days 落 ClipCapture + 即刻
    /// 持久化）；非法（非数值/0/空）→ 回退显示至当前生效值 + Toast（不生效不落
    /// 盘 · editor 行内编辑「非空才生效」同口径）；等值 no-op（不重写档）。pub 于
    /// 宿主关窗前漏斗（close_settings 开窗期未 blur/enter 的键入值不丢）。
    pub fn commit_retention(&mut self, w: &mut Window, cx: &mut Context<Self>) {
        let raw = self
            .retention_input
            .as_ref()
            .map(|st| st.read(cx).value().trim().to_string());
        let Some(raw) = raw else { return };
        match raw.parse::<u64>() {
            Ok(v) if v >= 1 => {
                if v != self.retention_days {
                    self.retention_days = v;
                    self.show_toast(format!("剪贴板历史保留时长 → {v} 天 · 即时生效"), cx);
                    self.emit(SettingsEvent::ClipRetentionChanged(v), cx);
                }
            }
            _ => {
                // 回退显示至当前生效值（set_value 触 Change 非订阅分支——无回环）。
                let cur = self.retention_days.to_string();
                if let Some(st) = self.retention_input.clone() {
                    st.update(cx, |state, cx| state.set_value(cur, w, cx));
                }
                self.show_toast("剪贴板历史保留时长需为正整数（天）· 已回退".to_string(), cx);
            }
        }
    }

    fn emit(&self, ev: SettingsEvent, cx: &mut App) {
        if let Some(f) = &self.on_settings_event {
            f(&ev, cx);
        }
    }

    /// 剪贴板监控开关行（SCN-CAPTURE-SWITCH-OFF · DEC-188）：左 label ＋右
    /// co::switch（可点击切换），切换即上抛宿主即时生效。
    fn capture_switch_row(&self, cx: &mut Context<Self>) -> Div {
        let on = self.capture_enabled;
        div()
            .flex()
            .justify_between()
            .child(co::txt("剪贴板监控", s::font(s::Lvl::Small), t::PRIMARY))
            .child(co::switch(on).id("set-cap-sw").on_click(cx.listener(
                move |this, _e: &ClickEvent, _w: &mut Window, cx: &mut Context<Self>| {
                    this.capture_enabled = !this.capture_enabled;
                    let now_on = this.capture_enabled;
                    this.show_toast(
                        if now_on {
                            "剪贴板监控已开启 · 即时生效".to_string()
                        } else {
                            "剪贴板监控已关闭 · 新复制不再入库".to_string()
                        },
                        cx,
                    );
                    this.emit(SettingsEvent::CaptureEnabledChanged(now_on), cx);
                },
            )))
    }

    /// 历史上限档行（REQ-CAPTURE-LIMIT · DEC-188）：五档 chip 组（100/500/1000/
    /// 5000/不限·选中=chip_on），点选即上抛宿主即时生效（触发位置 pill 同式）。
    fn limit_row(&self, cx: &mut Context<Self>) -> Div {
        const LIMIT_OPTIONS: [(u64, &str); 5] = [
            (100, "100"),
            (500, "500"),
            (1000, "1000"),
            (5000, "5000"),
            (0, "不限"),
        ];
        let mut chips = div().flex().flex_wrap().gap(px(6.0));
        for (i, (val, label)) in LIMIT_OPTIONS.iter().enumerate() {
            let base = if *val == self.capture_limit {
                co::chip_on(*label)
            } else {
                co::chip(*label)
            };
            chips = chips.child(
                base.id(("set-limit", i))
                    .on_click(cx.listener(
                        move |this, _e: &ClickEvent, _w: &mut Window, cx: &mut Context<Self>| {
                            this.capture_limit = *val;
                            let label_text = if *val == 0 {
                                "不限".to_string()
                            } else {
                                val.to_string()
                            };
                            this.show_toast(format!("历史上限 → {label_text} · 即时生效"), cx);
                            this.emit(SettingsEvent::CaptureLimitChanged(*val), cx);
                        },
                    ))
                    .active(co::act_border),
            );
        }
        div()
            .flex()
            .justify_between()
            .items_center()
            .child(co::txt("历史上限", s::font(s::Lvl::Small), t::PRIMARY))
            .child(chips)
    }

    /// 队列通知节流间隔行（REQ-SETTINGS-QUEUE-NOTIFY-INTERVAL · DEC-48/209）：
    /// 通用分区五档 chip 组（1/3/5/10/20 · 选中=chip_on）——1~20 越界不在预设
    /// 集内（设置项合法域仍为 1~20 全整数，档解析承载）；点选即上抛宿主即时
    /// 生效（历史上限行同式）。
    fn notify_interval_row(&self, cx: &mut Context<Self>) -> Div {
        const NOTIFY_STEPS: [(u64, &str); 5] =
            [(1, "1"), (3, "3"), (5, "5"), (10, "10"), (20, "20")];
        let mut chips = div().flex().flex_wrap().gap(px(6.0));
        for (i, (val, label)) in NOTIFY_STEPS.iter().enumerate() {
            let base = if *val == self.queue_notify_interval {
                co::chip_on(*label)
            } else {
                co::chip(*label)
            };
            chips = chips.child(
                base.id(("set-notify", i))
                    .on_click(cx.listener(
                        move |this, _e: &ClickEvent, _w: &mut Window, cx: &mut Context<Self>| {
                            this.queue_notify_interval = *val;
                            this.show_toast(format!("队列通知节流 → 每 {val} 条 · 即时生效"), cx);
                            this.emit(SettingsEvent::QueueNotifyIntervalChanged(*val), cx);
                        },
                    ))
                    .active(co::act_border),
            );
        }
        div()
            .flex()
            .justify_between()
            .items_center()
            .child(co::txt("队列通知节流", s::font(s::Lvl::Small), t::PRIMARY))
            .child(chips)
    }

    /// 排除应用行（REQ-CAPTURE-EXCLUDE · DEC-188）：左 label ＋右文本输入框
    ///（200×24 · 逗号分隔 bundleId）＋单位注记。
    fn exclude_row(&self) -> Div {
        let input = self.exclude_input.clone().map(|st| {
            div()
                .debug_selector(move || "set-exclude-input".to_string())
                .w(px(200.0))
                .h(px(24.0))
                .px(px(6.0))
                .bg(rgb(t::SURFACE))
                .border_1()
                .border_color(rgb(t::BORDER))
                .rounded(px(t::RAD_SM))
                .overflow_hidden()
                .flex()
                .items_center()
                .child(Input::new(&st).appearance(false).bordered(false))
        });
        div()
            .flex()
            .justify_between()
            .child(co::txt("排除应用", s::font(s::Lvl::Small), t::PRIMARY))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.0))
                    .children(input)
                    .child(co::txt(
                        "bundleId 逗号分隔",
                        s::font(s::Lvl::Small),
                        t::MUTED,
                    )),
            )
    }

    /// 排除应用输入引擎延迟建（DEC-188 · retention 同式）：初值＝当前生效列表。
    fn ensure_exclude(&mut self, w: &mut Window, cx: &mut Context<Self>) {
        if self.exclude_input.is_some() {
            return;
        }
        let seed = self.exclude_apps.clone();
        let st = cx.new(|cx| {
            let mut state = InputState::new(w, cx);
            state.set_placeholder("com.app.a, com.app.b", w, cx);
            state
        });
        st.update(cx, |state, cx| state.set_value(seed, w, cx));
        let sub = cx.subscribe_in(
            &st,
            w,
            |this: &mut Self,
             _st: &Entity<InputState>,
             ev: &InputEvent,
             w,
             cx: &mut Context<Self>| match ev {
                InputEvent::PressEnter { .. } | InputEvent::Blur => {
                    this.commit_exclude(w, cx);
                }
                _ => {}
            },
        );
        self.exclude_input = Some(st);
        self.exclude_sub = Some(sub);
        cx.notify();
    }

    /// 排除应用提交（DEC-188 · enter/blur/设置窗关窗漏斗三口）：去空格归一化
    ///（逗号分隔 trim + 去空项）且变更 → 生效 + Toast + 上抛宿主；等值 no-op。
    /// bundleId 字符集不作强校验（非法串由宿主端捕获判定链自然无效 · DEC-187
    /// env 键同口径）。
    pub fn commit_exclude(&mut self, w: &mut Window, cx: &mut Context<Self>) {
        let raw = self
            .exclude_input
            .as_ref()
            .map(|st| st.read(cx).value().trim().to_string());
        let Some(raw) = raw else { return };
        let normalized = raw
            .split(',')
            .map(|p| p.trim().to_string())
            .filter(|p| !p.is_empty())
            .collect::<Vec<_>>()
            .join(",");
        if normalized != self.exclude_apps {
            self.exclude_apps = normalized.clone();
            // 归一化显示回写（set_value 触 Change 非订阅分支——无回环）。
            if let Some(st) = self.exclude_input.clone() {
                st.update(cx, |state, cx| state.set_value(normalized.clone(), w, cx));
            }
            self.show_toast(
                format!(
                    "排除应用已更新（{} 项）· 即时生效",
                    normalized.split(',').filter(|s| !s.is_empty()).count()
                ),
                cx,
            );
            self.emit(SettingsEvent::CaptureExcludeChanged(normalized), cx);
        }
    }

    fn set_section(&mut self, i: usize, _e: &ClickEvent, _w: &mut Window, cx: &mut Context<Self>) {
        self.section = i;
        cx.notify();
    }

    /// 当前触发位置（DockPrefs 全局单源；未初始化时兜底取枚举默认——右侧 · DEC-76）。
    fn current_trigger(cx: &App) -> TriggerPosition {
        cx.try_global::<DockPrefs>()
            .map(|p| p.trigger)
            .unwrap_or_default()
    }

    /// 触发位置是否被 Dock 占用（DEC-84 · REQ-PANEL-TRIGGER-POSITION 可用性）：
    /// DockAvailability 全局读取（app 层启动/屏帧变化落值）；未初始化（纯 ui 语境/
    /// 无宿主演示）按全可用兜底；四角恒可用（DEC-84）。
    fn trigger_occupied(cx: &App, pos: TriggerPosition) -> bool {
        let avail = cx.try_global::<DockAvailability>();
        match pos {
            TriggerPosition::Left => avail.map(|a| a.occupied_left).unwrap_or(false),
            TriggerPosition::Right => avail.map(|a| a.occupied_right).unwrap_or(false),
            TriggerPosition::Bottom => avail.map(|a| a.occupied_bottom).unwrap_or(false),
            // 四角恒可用（DEC-84：角位无可用性条件）。
            TriggerPosition::TopLeft
            | TriggerPosition::TopRight
            | TriggerPosition::BottomLeft
            | TriggerPosition::BottomRight => false,
        }
    }

    /// 触发位置 7 选项 pill 组：选中=chip.on（暖底主墨），未选=chip（描边 muted）；
    /// 被 Dock 占用的缘置灰禁选（opacity 降 + 无点击处理器 · DEC-84 可用性）；
    /// 点选上抛宿主并 Toast 确认（DEC-67）。
    fn trigger_pills(&self, cx: &mut Context<Self>) -> Div {
        let current = Self::current_trigger(cx);
        let mut grid = div().flex().flex_wrap().gap(px(6.0));
        for (i, pos) in TriggerPosition::ALL.into_iter().enumerate() {
            let disabled = pos != current && Self::trigger_occupied(cx, pos);
            let base = if pos == current {
                co::chip_on(pos.label())
            } else {
                co::chip(pos.label())
            };
            let mut pill = base.id(("set-trg", i));
            if disabled {
                pill = pill.opacity(0.45);
            } else {
                pill = pill
                    .on_click(cx.listener(
                        move |this, _e: &ClickEvent, _w: &mut Window, cx: &mut Context<Self>| {
                            this.select_trigger(pos, cx)
                        },
                    ))
                    .active(co::act_border);
            }
            grid = grid.child(pill);
        }
        grid
    }

    fn select_trigger(&mut self, pos: TriggerPosition, cx: &mut Context<Self>) {
        let msg = format!("触发位置 → {} · 即时生效", pos.label());
        self.show_toast(msg, cx);
        self.emit(SettingsEvent::TriggerPosChanged(pos), cx);
    }

    /// 输入监控授权请求（引导卡点击 · DEC-81）：Toast 即时反馈 + 上抛宿主（宿主
    /// 后台执行 CGRequestListenEventAccess 并回写 ListenAuth · TCC 弹窗即引导入口）。
    fn request_listen_auth(&mut self, cx: &mut Context<Self>) {
        self.show_toast("已请求输入监控授权 · 请在系统弹窗中允许".to_string(), cx);
        self.emit(SettingsEvent::ListenAuthRequest, cx);
    }

    /// 行：说明 + 右值（12px 两端对齐，HTML sec 内条目布局）。
    fn kv(k: &'static str, v: &'static str) -> Div {
        div()
            .flex()
            .justify_between()
            .child(co::txt(k, s::font(s::Lvl::Small), t::PRIMARY))
            .child(co::txt(v, s::font(s::Lvl::Small), t::MUTED))
    }

    /// sec 盒内多条 kv 行（卡片 10px 内边距 · 行间 6px）。
    fn kv_sec(rows: &[(&'static str, &'static str)]) -> Div {
        let mut el = co::sec_box()
            .p(px(10.0))
            .mb(px(8.0))
            .flex()
            .flex_col()
            .gap(px(6.0));
        for (k, v) in rows {
            el = el.child(Self::kv(k, v));
        }
        el
    }

    fn section_body(&self, cx: &mut Context<Self>) -> Div {
        match self.section {
            // 通用分区（DEC-209）：队列通知节流间隔为真交互行；语言/开机启动/清除
            // 指标仍为规格占位 kv（后续任务承接面）。
            0 => {
                let mut el = co::sec_box()
                    .p(px(10.0))
                    .mb(px(8.0))
                    .flex()
                    .flex_col()
                    .gap(px(6.0));
                el = el
                    .child(Self::kv("语言", "简体中文 ▾（5 包，缺失 key 兜底）"))
                    .child(Self::kv("开机启动", "关 ▾（失败回滚 + 红提示）"))
                    .child(self.notify_interval_row(cx))
                    .child(Self::kv("清除本地指标", "清（仅本地 · DEC-33）"));
                el
            }
            1 => Self::kv_sec(&[
                ("主题", "跟随系统 ▾（System/Light/Dark 立即应用）"),
                ("材质", "不透明 ▾（标准双材质 × 6 色调）"),
                ("强调色", "墨 ▾（6 色调）"),
            ]),
            2 => {
                let listen_granted = cx
                    .try_global::<ListenAuth>()
                    .map(|a| a.listen_granted)
                    .unwrap_or(true);
                let mut el = div().flex().flex_col().child(
                    co::sec_box()
                        .p(px(10.0))
                        .mb(px(8.0))
                        .flex()
                        .flex_col()
                        .gap(px(6.0))
                        .child(co::txt("触发位置", s::font(s::Lvl::Small), t::PRIMARY))
                        .child(self.trigger_pills(cx))
                        .child(co::meta(
                            "被 Dock 占用的缘置灰禁选 · 冲突自动切换（顶缘系菜单栏恒定占用，无顶部位）",
                        ))
                        .child(Self::kv("触发延迟", "0.0s · 滑过立即（DEC-85）"))
                        .child(Self::kv("密度", "与主面板共享切换")),
                );
                // 引导卡条件渲染（DEC-81 · REQ-PANEL-LISTEN-GRACEFUL「设置页引导」）。
                if !listen_granted {
                    el = el.child(
                        div()
                            .border_1()
                            .border_color(rgb(t::TERTIARY))
                            .rounded(px(t::RAD_MD))
                            .bg(rgb(t::SURFACE))
                            .p(px(10.0))
                            .flex()
                            .flex_col()
                            .gap(px(4.0))
                            .child(co::txt_f(
                                "输入监控未授权 · 引导卡",
                                s::font(s::Lvl::Small),
                                t::TERTIARY,
                                t::FONT_BODY,
                                t::FONT_WEIGHT_REGULAR,
                            ))
                            .child(co::meta("边缘触发与剪贴板外监控需要此权限"))
                            .child(
                                co::btn_sec("打开系统设置")
                                    .id("set-perm-deeplink")
                                    .on_click(cx.listener(
                                    |this,
                                     _e: &ClickEvent,
                                     _w: &mut Window,
                                     cx: &mut Context<Self>| {
                                        this.request_listen_auth(cx);
                                    },
                                ))
                                .active(co::act_border_ink),
                            ),
                    );
                }
                el.child(
                    div()
                        .bg(rgb(t::NEUTRAL))
                        .rounded(px(t::RAD_MD))
                        .p(px(10.0))
                        .child(co::meta(
                            "WAL 风险提示（根位于同步盘目录时显示 · PRD 定稿文案）",
                        )),
                )
            }
            3 => Self::kv_sec(&[
                ("编辑器字体", "Songti SC ▾"),
                ("字号", "15（11–28pt）"),
                ("拼写策略", "关 ▾（四策略 · v1 未生效）"),
            ]),
            4 => Self::kv_sec(&[
                ("呼出面板", "⌃⇧Space（可改）"),
                ("快速回贴", "⌘⇧V（可改）"),
                ("本地键", "7 个可配 · 冲突纯提示不阻断"),
                ("注册失败", "红提示 + 重试"),
            ]),
            5 => {
                // 存储页（DEC-178/DEC-188）：kv 展示行＋捕获域四实值行——监控开关
                //（switch）· 历史上限档（chips）· 排除应用（文本输入）· 保留时长
                //（数值输入）（REQ-SETTINGS 存储分区承载 spec 明文）。
                Self::kv_sec(&[
                    ("存储根", "~/notes ▾（多根隔离 · 每根一库）"),
                    ("askOnLaunch", "关 ▾"),
                    ("导出 / 导入", "JSON 全量 · 后台进度可见"),
                ])
                .child(self.capture_switch_row(cx))
                .child(self.limit_row(cx))
                .child(self.exclude_row())
                .child(self.retention_row())
            }
            6 => Self::kv_sec(&[
                ("同步", "不内建（files 可入同步盘 · data 本机）"),
                ("data/ 排除引导", "在同步服务中排除 data/ 子目录"),
                ("备份", "JSON 导出（每根）"),
                ("Git", "files/ 友好布局（DB 不入 Git）"),
            ]),
            7 => Self::kv_sec(&[
                ("检查更新", "手动检查按钮（不受 24h 节流）"),
                ("自动更新", "24h 节流 · 下载可取消"),
                ("校验", "SHA256 · 失败提示 + 保留旧版"),
            ]),
            8 => {
                let mut el = co::sec_box()
                    .p(px(10.0))
                    .mb(px(8.0))
                    .flex()
                    .flex_col()
                    .gap(px(6.0));
                for (i, name) in t::TAG_NAMES.iter().enumerate() {
                    el = el.child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(6.0))
                            .child(
                                div()
                                    .w(px(7.0))
                                    .h(px(7.0))
                                    .rounded_full()
                                    .bg(rgb(t::tag_color(i))),
                            )
                            .child(co::txt(
                                name.to_string(),
                                s::font(s::Lvl::Small),
                                t::PRIMARY,
                            ))
                            .child(co::meta(format!(
                                "#{:06X}（hex 可改 · 非法拒绝）",
                                t::tag_color(i)
                            ))),
                    );
                }
                el
            }
            _ => {
                // 关于分区：题头彩色羊皮稿 logo（logo_color.png 终端统一口径）+
                // 三行 kv。图源 = host 注册的内嵌资产（app 层 ParchoraAssets
                //「parchora-logo-color.png」→ Resource::Embedded 解析 · gpui 内部
                // image 解码链）；纯 ui 测试语境无注册时 img 加载静默失败不阻断
                // 渲染（ImageCacheError 载体，不 panic）。正方形 1024px 源
                //（DEC-160）+ size 28pt ——比例 1:1 不拉伸不模糊。
                div()
                    .flex()
                    .flex_col()
                    .gap(px(6.0))
                    .mb(px(8.0))
                    .child(img("parchora-logo-color.png").size(px(28.0)).flex_none())
                    .child(Self::kv_sec(&[
                        ("版本", "0.1.0（alpha）"),
                        ("旧配置迁移", "自动 · 见「关于」说明"),
                        ("重置为默认", "逃生口（二次确认）"),
                    ]))
            }
        }
    }
}

impl Render for SettingsView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let _s = crate::scale::enter(window, cx);
        // 存储页承载保留时长与排除应用输入（DEC-178/DEC-188）：进入该分区首帧
        //建引擎（一次；切离/复返不重建不丢已键入值——引擎实体持于字段）。
        if self.section == 5 {
            self.ensure_retention(window, cx);
            self.ensure_exclude(window, cx);
        }

        // 左导航（120px · bsoft 右线 · surface 底）
        let mut nav = div()
            .w(px(120.0))
            .h_full()
            .border_r_1()
            .border_color(rgb(t::BORDER_SOFT))
            .bg(rgb(t::SURFACE))
            .flex()
            .flex_col();
        for (i, s) in SECTIONS.iter().enumerate() {
            nav = nav.child(
                co::node_row(s.to_string(), None, i == self.section)
                    .id(("set-nav", i))
                    .on_click(cx.listener(
                        move |this, e: &ClickEvent, w: &mut Window, cx: &mut Context<Self>| {
                            this.set_section(i, e, w, cx)
                        },
                    ))
                    .active(co::act_border),
            );
        }

        let title = SECTIONS.get(self.section).copied().unwrap_or("通用");
        let body = div()
            .flex_1()
            .min_h_0()
            // w_full：滚动容器内约束内容列宽度（否则 auto 宽被 pill 等子内容撑开，
            // flex_wrap 失效、长文案横向溢出被窗缘裁断）。
            .w_full()
            .p(px(14.0))
            .flex()
            .flex_col()
            // 题注 =「分区名+设置」；题注与首卡间距固定 6px。
            .child(co::meta(format!("{title}设置")).mb(px(6.0)))
            .child(self.section_body(cx));

        div()
            .size_full()
            .relative()
            .flex()
            .bg(rgb(t::NEUTRAL))
            .child(nav)
            .child(
                div()
                    .id("set-scroll")
                    .flex_1()
                    // min_w_0：row flex 子项默认被内容 min-width 下限撑破 grow 份额，
                    // 内容列溢出窗缘（pill 不折行、meta/右值裁断）。
                    .min_w_0()
                    .min_h_0()
                    .overflow_y_scroll()
                    .child(body),
            )
            .when_some(self.toast.clone(), |el, msg| {
                el.child(
                    div()
                        .absolute()
                        .bottom(px(24.0))
                        .left_0()
                        .right_0()
                        .flex()
                        .justify_center()
                        .child(co::toast(msg).id("toast-set")),
                )
            })
    }
}
