//! PG-C / PG-C1 待办视图（UI-5 · TASK-506 BoardView / F-TSK-1~7）。
//!
//! HTML 1:1（PG-C.html；PG-C2 详情区无真实数据面不渲染，统一详情窗
//! peek.rs 承载 · TASK-506 数据面恢复点）：
//! - 分段控件（状态分组 ⇄ 排期分组，r6 边框盒内双 flex 半区，激活=warm 底）；
//! - 状态模式：待办置首（DEC-185）＋进行中 / 已搁置 / 已完成（划线行）/
//!   已取消保持相对序——行全部来自会话 created（右键状态/排期落行级字段 ·
//!   DEC-183）；空组不渲染（DEC-185）；
//! - 排期模式：今天/明天/本周/以后/未排期 五桶序（今天天然首位）同构（created
//!   按行 bucket 桶归组 · 空桶不渲染 DEC-185）；
//! - 勿忘：完成态行 = 9px 绿点 + 48% 划线（D-03 叠线）；
//! - 图标（DEC-66）：组头折叠符 ▾/▴ → chevron-down/up 随展开态翻转（DEC-185）·
//!   空态 · → circle-dashed ·「← 待办」
//!   → arrow-left。
//!
//! 演示宿主限定：⌘⌥1~5 / 拖拽排期 defer（TASK-506 实装）；单击任务行 →
//! 统一详情窗直开（peek.rs 单击直开）。
//!
//! 右键菜单接线（DEC-114 · DB 真链路仍 TASK-506）：任务行右键＝「状态→五选一」
//! +「排期→五桶」两节（✓ 标现值；点击真实落行级 status/bucket 字段，状态
//! 模式分组随值重排 · DEC-183）；菜单本体与消耗语义同 collect.rs。
//!
//! 待办创建面板真输入引擎（DEC-165 · 独立创建面板）：
//! 标题行换单行 InputState（面板窗内 ensure_title 延迟建——render 首帧建并聚焦）。
//! 「创建待办事项」独立窗（与「创建集锦」编辑窗同构 chrome · detail_window_opts
//! 可拖 480×70%）承载创建表单：FAB「新建任务」经 TaskSink 上抛
//! ShellEvent::QuickTask → 宿主开/激活面板窗（单实例幂等）；提交经
//! TaskCreateEvent 上抛宿主漏斗回灌 BoardView::ingest_created（TASK-<seq> 自增
//! · bucket 由面板携带 · 成功 Toast 列表面反馈）；空标题创建钮留页面板 Toast；
//! 返回＝保存并关闭（空稿 BackEmpty 关窗披露）；取消/Esc 纯丢弃关窗；楷体落点
//! ＝标题输入与字段值/chips（FONT_CONTENT · DEC-66/68 捆绑族名口径）。

use std::collections::HashMap;
use std::time::Duration;

use gpui::prelude::FluentBuilder as _;
use gpui::{
    deferred, div, px, relative, rgb, AppContext as _, ClickEvent, Context, Div, Entity,
    Focusable as _, InteractiveElement as _, IntoElement as _, ParentElement as _, Render,
    StatefulInteractiveElement as _, Styled, Window,
};
use gpui_component as ui;
use ui::input::{Input, InputState, Textarea, TextareaState};
use ui::scroll::{Scrollbar, ScrollbarAxis};
use ui::InteractiveElementExt as _;

use crate::pages::peek::{self, PeekItem, PeekKind};
use crate::{components as co, scale as s, tokens as t};

/// 排期五桶（REQ-TASK-SCHEDULE-BUCKET / SCN-TASK-BUCKET-GROUPS）。
const BUCKETS: [&str; 5] = ["今天", "明天", "本周", "以后", "未排期"];

/// 状态五枚举（REQ-TASK-STATUS-5 枚举序 · UI 组名口径 · DEC-114 右键五选一）。
const STATUSES: [&str; 5] = ["待办", "进行中", "已搁置", "已完成", "已取消"];

/// 优先级五值（REQ-TASK-PRIORITY-5 枚举序 none/low/medium/high/urgent · 中文
/// 映射 DEC-168；表单默认「无」＝spec DB 默认 none 同轴 · DEC-169 D-01）。
const PRIORITIES: [&str; 5] = ["无", "低", "中", "高", "紧急"];

/// 颜色标记七值（DEC-168 · hex 直给）；下标即互斥序，赤=0 为默认。
const MARKS: [(&str, u32); 7] = [
    ("赤", 0xFF6B6B),
    ("橙", 0xFFA94D),
    ("黄", 0xFFD93D),
    ("绿", 0x6BE585),
    ("青", 0x4DD9E8),
    ("蓝", 0x5B8CFF),
    ("紫", 0xB57BFF),
];

/// 行左优先级标识标签样式映射（DEC-232 纯函数承载 · DEC-233 恒显 ·
/// DEC-234 无级纯色化）：恒返回（优先级文本 Option, 底色＝该任务标签 mark
/// 七色对应值）——优先级=无 → 文本 None（标签只显示背景色=任务标签色点，
/// 不写字·用户指令）；有优先级 → 五值文本（墨黑字于 MARKS 亮色系底 ≥5.5:1 AA）。
fn pri_chip(priority: usize, mark: usize) -> (Option<&'static str>, u32) {
    (
        (priority > 0).then(|| PRIORITIES[priority % PRIORITIES.len()]),
        MARKS[mark % MARKS.len()].1,
    )
}

/// 创建面板标题栏按钮统一高（返回/取消/创建三钮同钉 ·「高度严格一致」的结构
/// 性保证——避免文本度量算术逐平台漂移）。
const TC_BAR_BTN_H: f32 = 28.0;

/// 创建面板单行控件盒高（DEC-173 堆叠后仅标题输入盒钉此值）：大于
/// gpui-component 单行 Input 固有高 32（Size::Medium = h_8）防裁字。
const TC_FIELD_H: f32 = 36.0;

/// 创建面板字段下拉（状态/优先级 · 单开互斥）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DdField {
    Status,
    Priority,
}

/// 组头折叠符（lucide chevron-down；自带水平 gap 供叠加说明文字）。
fn chev_down() -> Div {
    div()
        .flex()
        .items_center()
        .gap(px(3.0))
        .flex_none()
        .child(co::icon(co::Icon::ChevronDown, 11.0, t::MUTED))
}

/// 组头展开符（lucide chevron-up · DEC-185）：随展开态与折叠符成对翻转——展开
/// 态组头用之、折叠态仍 chev_down；同 chev_down 构造。
fn chev_up() -> Div {
    div()
        .flex()
        .items_center()
        .gap(px(3.0))
        .flex_none()
        .child(co::icon(co::Icon::ChevronUp, 11.0, t::MUTED))
}

/// 任务行（DEC-183：全部行＝会话创建产物；右键菜单落行级字段）。
#[derive(Default)]
struct TaskRow {
    title: String,
    id: String,
    meta_extra: Option<&'static str>,
    src: Option<(&'static str, &'static str)>, // (标签文案, 目标视图)
    /// 状态五选一（STATUSES 下标 · DEC-183；右键菜单落字段 · 分组随值重排）。
    status: usize,
    done: bool,
    bucket: usize,
    /// 优先级五值下标（PRIORITIES · DEC-232；DEC-233 标签恒显）。
    priority: usize,
    /// 颜色标记七值下标（MARKS · DEC-168；DEC-232 行左标签底色来源——
    /// 创建表单选择经 Submit/Created 随行持久化 todos.mark，重启回填不丢）。
    mark: usize,
}

/// 任务行 → Peek 条目（单击承载 · DEC-183）：标题/正文=任务题名，
/// meta=id＋排期/来源注记（会话内真实字段，无演示文案）。
fn peek_item(r: &TaskRow) -> PeekItem {
    let mut meta = r.id.clone();
    if let Some(m) = r.meta_extra {
        meta.push_str(&format!(" · {m}"));
    }
    if let Some((src, _)) = r.src {
        meta.push_str(&format!(" · {src}"));
    }
    PeekItem {
        kind: PeekKind::Task,
        title: r.title.clone(),
        meta,
        body: r.title.clone(),
        disk_real: false,
    }
}

/// 组内行排序（优先级高→无 · 同级创建序）：优先级降序稳定排序——同级保留
/// created 相对序（下标升序＝先创建在前）；行址＝created 下标不受重排影响。
fn sort_group_rows(rows: &mut [(usize, &TaskRow)]) {
    rows.sort_by_key(|(_, r)| std::cmp::Reverse(r.priority));
}

/// 待办分组数据准备（DEC-185 · 数据准备阶段过滤）：全部行＝created。状态模式
/// 归组序＝待办置首＋其余保持相对序（进行中/已搁置/已完成/已取消）；排期模式
/// ＝BUCKETS 固定序（今天天然首位）；组内按优先级高→无排序、同级创建序在前
/// （sort_group_rows）。空组一律不渲染（DEC-185）——返回仅含
/// 非空组的「（组名，组内行）」序；行＝（created 数据下标，引用），行 id 与
/// 右键菜单行址统一以 created 下标承载（DEC-183 行级字段落值口径）。
fn prepare_groups(
    created: &[TaskRow],
    schedule_mode: bool,
) -> Vec<(&'static str, Vec<(usize, &TaskRow)>)> {
    let mut groups: Vec<(&'static str, Vec<(usize, &TaskRow)>)> = Vec::new();
    if schedule_mode {
        for (b, name) in BUCKETS.iter().enumerate() {
            let mut rows: Vec<(usize, &TaskRow)> = created
                .iter()
                .enumerate()
                .filter(|(_, r)| r.bucket == b)
                .collect();
            if !rows.is_empty() {
                sort_group_rows(&mut rows);
                groups.push((name, rows));
            }
        }
    } else {
        // （组名， status 下标）序＝待办/进行中/已搁置/已完成/已取消（DEC-185
        // 「待办」置首 · SCN-TASK-SORT-RANK）。
        const STATUS_GROUPS: [(&str, usize); 5] = [
            ("待办", 0),
            ("进行中", 1),
            ("已搁置", 2),
            ("已完成", 3),
            ("已取消", 4),
        ];
        for &(name, st) in STATUS_GROUPS.iter() {
            let mut rows: Vec<(usize, &TaskRow)> = created
                .iter()
                .enumerate()
                .filter(|(_, r)| r.status == st)
                .collect();
            if !rows.is_empty() {
                sort_group_rows(&mut rows);
                groups.push((name, rows));
            }
        }
    }
    groups
}

/// 待办视图。
pub struct BoardView {
    schedule_mode: bool,
    /// 单击 Peek 上抛器（单击直开）
    emitter: peek::PeekEmitter,
    /// 待办 FAB「新建任务」→ 宿主上抛通道（DEC-165 · ShellEvent::QuickTask 经
    /// PanelView 弱引用转发；NoteSink 同模式）。
    task_open: Option<TaskSink>,
    /// 详情态演示 Toast
    toast: Option<String>,
    /// Toast 自动消失防争代次（gen 失效防争 · 与 app peek_close_gen 同模式）
    toast_gen: u64,
    /// 未决 Toast timer（gpui::Task 持于字段；防争由 toast_gen 代次承载 · DEC-72）
    toast_task: Option<gpui::Task<()>>,
    /// 主列表行右键菜单态（DEC-114)：下标＝created 数据下标（DEC-183 右键落
    /// 行级字段 · 分组随值重排）；元组二位＝任务 id（DEC-116 根挂载重建菜单
    /// Toast 文案需 who，开启时随态存）。
    row_ctx: Option<(usize, String)>,
    /// 右键菜单开启时的窗内点击坐标（DEC-116 点击位锚定；开启必写覆盖）。
    ctx_pos: Option<gpui::Point<gpui::Pixels>>,
    /// 会话内创建产物（TASK-300 DB 持久化承载 · Created 事件上抛宿主事务写，
    /// 重启经 seed_created 回填）：状态模式按行 status 分组、排期模式按 bucket
    /// 桶归组（DEC-183 真实化）。
    created: Vec<TaskRow>,
    /// 创建序号计数（TASK-1 起 · DEC-183）。
    next_seq: usize,
    /// 分组展开态（DEC-185 · D-04）：键＝组名（状态/排期两组名域不相交共用）；
    /// 显式值由 grp_toggle 落、缺省 grp_expanded 兜底（待办/今天默认展开 · 用户
    /// 指令）；实体常驻（shell 持有）→ 跨选项卡切换/数据更新自然保持（会话内
    /// 不写 DB · REQ-TASK-GROUP-COLLAPSE 随 DEC-185 同步改写）。
    expanded: HashMap<String, bool>,
    /// 待办 DB 事件上抛（TASK-300 · 宿主事务写唯一通道；未注 = 无操作）。
    /// 创建/状态/排期事件经 sink 上抛宿主落库——同步 DB 写不触窗口操作，
    /// 不触租借域（DEC-77 租借铁律豁免同 ShellEvent::PinChanged 同步段）。
    on_task_change: Option<TaskDbSink>,
}

impl BoardView {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            schedule_mode: false,
            emitter: peek::PeekEmitter::default(),
            task_open: None,
            toast: None,
            toast_gen: 0,
            toast_task: None,
            row_ctx: None,
            ctx_pos: None,
            created: Vec::new(),
            next_seq: 1,
            expanded: HashMap::new(),
            on_task_change: None,
        }
    }

    /// 单击 Peek 上抛通道注入（PanelView 装配统一转发 app 层）。
    pub fn set_peek_sink(&mut self, sink: peek::PreviewSink) {
        self.emitter.set_sink(sink);
    }

    /// FAB「新建任务」上抛通道注入（PanelView 装配 · DEC-165）。
    pub fn set_task_sink(&mut self, sink: TaskSink) {
        self.task_open = Some(sink);
    }

    /// FAB 点击上抛（通道缺省=无操作，演示页自持 · emit_note 同式）。
    fn emit_task_open(&self, cx: &mut Context<Self>) {
        if let Some(f) = self.task_open.clone() {
            f(cx);
        }
    }

    /// 待办 DB 事件通道注入（宿主装配位 · TASK-300 持久化链）。
    pub fn set_db_sink(&mut self, sink: TaskDbSink) {
        self.on_task_change = Some(sink);
    }

    /// DB 事件上抛（通道缺省无操作）；cx 即宿主 App 语境（同步 DB 写面）。
    fn fire_task_change(&self, ev: TaskDbEvent, cx: &mut Context<Self>) {
        if let Some(f) = self.on_task_change.clone() {
            f(&ev, cx);
        }
    }

    /// 回填种子（TASK-300 启动恢复 · 仅全新实体启用）：行集与序号整表替换
    ///（行序即 DB 读序 = identifier 数值序）；已存活实体（显隐复用）不得调用
    ///（整表替换会湮灭会话内存增量行）。行元组＝(id, 标题, 状态下标, 桶下标)。
    pub fn seed_created(
        &mut self,
        rows: impl IntoIterator<Item = (String, String, usize, usize, usize, usize)>,
        next_seq: usize,
        cx: &mut Context<Self>,
    ) {
        self.created = rows
            .into_iter()
            .map(|(id, title, status, bucket, priority, mark)| TaskRow {
                id,
                title,
                status,
                bucket,
                done: status == 3,
                priority,
                mark,
                ..Default::default()
            })
            .collect();
        self.next_seq = next_seq;
        cx.notify();
    }

    /// Esc 级联入口：主列表行菜单 → 不消化（shell.rs · DEC-73/114）。
    pub fn dismiss_overlay(&mut self, cx: &mut Context<Self>) -> bool {
        if self.row_ctx.take().is_some() {
            cx.notify();
            true
        } else {
            false
        }
    }

    /// 演示 Toast：1400ms 自动消失（UI-TOAST；新 Toast bump 代次 + timer 持于字段）。
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

    fn toggle_mode(&mut self, _e: &ClickEvent, _w: &mut Window, cx: &mut Context<Self>) {
        self.schedule_mode = !self.schedule_mode;
        cx.notify();
    }

    /// 任务行点击统一入口：统一详情窗（peek.rs 单击承载）。
    fn task_row_click(&mut self, item: &PeekItem, cx: &mut Context<Self>) {
        self.emitter.show(item, cx);
        cx.notify();
    }

    /// FAB「新建任务」点击（DEC-165）：上抛 ShellEvent::QuickTask，宿主开/激活
    /// 「创建待办事项」独立面板窗。
    fn fab_create(&mut self, _e: &ClickEvent, _w: &mut Window, cx: &mut Context<Self>) {
        self.emit_task_open(cx);
    }

    /// 创建面板提交回灌（DEC-165 · 单一入行权威面；提交漏斗 app 层转
    /// PanelView.ingest_created_task 承接）：TASK-<seq> 自增 + created 追加（bucket/
    /// 优先级/标签下标由面板携带 · DEC-232）+ 成功 Toast + DB 事件上抛（TASK-300 ·
    /// Created → 宿主事务写 todos 插行）——列表面反馈与刷新 = notify 重渲染。
    pub fn ingest_created(
        &mut self,
        title: String,
        bucket: usize,
        priority: usize,
        mark: usize,
        cx: &mut Context<Self>,
    ) {
        let id = format!("TASK-{}", self.next_seq);
        self.next_seq += 1;
        self.created.push(TaskRow {
            title: title.clone(),
            id: id.clone(),
            bucket,
            priority,
            mark,
            ..Default::default()
        });
        self.show_toast(format!("已创建 {id}"), cx);
        self.fire_task_change(
            TaskDbEvent::Created {
                identifier: id,
                title,
                bucket,
                priority,
                mark,
            },
            cx,
        );
    }

    /// 创建面板空稿收口披露（DEC-165 · 返回＝保存并关闭口径延续）：面板窗已关，
    /// Toast 落列表页提示（BackEmpty 漏斗落点）。
    pub fn task_create_closed_empty(&mut self, cx: &mut Context<Self>) {
        self.show_toast("已关闭 · 空标题未创建".into(), cx);
    }

    /// 待办行题名披露（DEC-186 · 根修测试断言读点）：app 层跨关板存续验证
    /// 需在窗亡后读行序；仅读不写，列序即创建序。
    pub fn board_titles(&self) -> Vec<String> {
        self.created.iter().map(|r| r.title.clone()).collect()
    }

    /// 创建序号读点（TASK-300 · 重启续接对账面）：next_seq 当前值（只读）。
    pub fn next_seq_snapshot(&self) -> usize {
        self.next_seq
    }

    // ── 右键菜单（DEC-114/116）──

    /// 主列表行菜单开（详情区开着时右键不可达——两层界面互斥）；记录点击窗内坐标
    /// 与任务 id（DEC-116 点击位锚定 · 根挂载重建需 who；created 行 id 动态 →
    /// String 承载）。
    fn ctx_open(
        &mut self,
        row: usize,
        who: impl Into<String>,
        pos: gpui::Point<gpui::Pixels>,
        cx: &mut Context<Self>,
    ) {
        self.row_ctx = Some((row, who.into()));
        self.ctx_pos = Some(pos);
        cx.notify();
    }

    /// 菜单项 · 状态五选一（DEC-183）：落行级 status 字段；完成态同步
    /// done 划线（状态/完成语义同源）；状态模式分组随值重排（notify 重渲染）；
    /// DB 事件上抛（TASK-300 · Status → 宿主 UPDATE）。
    /// 菜单项 · 状态五选一（DEC-183）：落行级 status 字段；完成态同步
    /// done 划线（状态/完成语义同源）；状态模式分组随值重排（notify 重渲染）；
    /// DB 事件上抛（TASK-300 · Status → 宿主 UPDATE；identifier 先取后触发
    /// ——行可变借用与 sink 不相交）。
    pub fn status_set(&mut self, row: usize, s: usize, who: &str, cx: &mut Context<Self>) {
        self.row_ctx = None;
        let identifier = self.created.get_mut(row).map(|r| {
            r.status = s;
            r.done = s == 3;
            r.id.clone()
        });
        if let Some(identifier) = identifier {
            self.fire_task_change(
                TaskDbEvent::Status {
                    identifier,
                    status: s,
                },
                cx,
            );
        }
        self.show_toast(
            format!("{who} 状态 → {}", STATUSES.get(s).copied().unwrap_or("?")),
            cx,
        );
    }

    /// 菜单项 · 排期五桶（DEC-183）：落行级 bucket 字段，排期模式按值
    /// 归组重排（notify 重渲染）；DB 事件上抛（TASK-300 · Bucket → 宿主 UPDATE）。
    pub fn bucket_set(&mut self, row: usize, b: usize, who: &str, cx: &mut Context<Self>) {
        self.row_ctx = None;
        let identifier = self.created.get_mut(row).map(|r| {
            r.bucket = b;
            r.id.clone()
        });
        if let Some(identifier) = identifier {
            self.fire_task_change(
                TaskDbEvent::Bucket {
                    identifier,
                    bucket: b,
                },
                cx,
            );
        }
        self.show_toast(
            format!("{who} 排期 → {}", BUCKETS.get(b).copied().unwrap_or("?")),
            cx,
        );
    }

    /// 菜单项 · 删除（DEC-195 · 拾记行删除同构）：行址＝created 数据下标（DEC-183
    /// 行级口径）；菜单点击即收层，删除后下标平移无碍；DB 事件上抛（TASK-300
    /// · Deleted → 宿主删行）。
    pub fn ctx_delete(&mut self, row: usize, cx: &mut Context<Self>) {
        self.row_ctx = None;
        let removed = if row < self.created.len() {
            Some(self.created.remove(row))
        } else {
            None
        };
        if let Some(r) = removed {
            self.fire_task_change(
                TaskDbEvent::Deleted {
                    identifier: r.id.clone(),
                },
                cx,
            );
            self.show_toast(format!("已删除 {}", r.id), cx);
        }
    }

    /// 任务行右键菜单本体（DEC-114/116）：状态五选一 head+5 行 + sep + 排期五桶 head+5 行，
    /// ✓ 标现值（DEC-183：读行级 status/bucket · 行＝created 数据下标）；
    /// out 包 wrapper 一次挂载（消耗语义同 collect.rs）；锚定＝点击位
    ///（DEC-116：视图根挂载 · unwrap 兕底不可达——开启必写坐标）。
    fn task_ctx_layer(&self, row: usize, who: &str, cx: &Context<Self>) -> gpui::AnyElement {
        let view = cx.weak_entity();
        let out = move |_: &gpui::MouseDownEvent, _: &mut gpui::Window, cx: &mut gpui::App| {
            let _ = view.update(cx, |v, cx| {
                if v.row_ctx.is_some() {
                    v.row_ctx = None;
                    cx.notify();
                }
            });
            cx.stop_propagation();
        };
        // listener 需 'static：who 先拷贝，进入每个菜单项 listener 时再克隆（Fn 多次
        // 调用不能移出 · 借传经 as_str）。
        let who = who.to_string();
        let st = self.created.get(row).map(|r| r.status).unwrap_or(0);
        let bk = self.created.get(row).map(|r| r.bucket).unwrap_or(4);
        let mut menu = co::ctx_menu().child(co::ctx_head("状态"));
        for (k, s) in STATUSES.iter().enumerate() {
            let checked = k == st;
            let who = who.clone();
            menu = menu.child(
                co::ctx_row(s.to_string(), None, checked)
                    .id(("ctx-c-st", k))
                    .on_click(cx.listener(
                        move |this, _e: &ClickEvent, _w: &mut Window, cx: &mut Context<Self>| {
                            this.status_set(row, k, who.as_str(), cx)
                        },
                    ))
                    .hover(co::hov_warm)
                    .active(co::act_border),
            );
        }
        menu = menu.child(co::ctx_sep()).child(co::ctx_head("排期"));
        for (b, bucket) in BUCKETS.iter().enumerate() {
            let checked = b == bk;
            let who = who.clone();
            menu = menu.child(
                co::ctx_row(bucket.to_string(), None, checked)
                    .id(("ctx-c-bk", b))
                    .on_click(cx.listener(
                        move |this, _e: &ClickEvent, _w: &mut Window, cx: &mut Context<Self>| {
                            this.bucket_set(row, b, who.as_str(), cx)
                        },
                    ))
                    .hover(co::hov_warm)
                    .active(co::act_border),
            );
        }
        menu = menu.child(co::ctx_sep()).child(
            co::ctx_row("删除", None, false)
                .id("ctx-c-del")
                // 测试取样面（DEC-195）：debug_bounds 按「task-menu-del」定位注入
                // 点击；release no-op 不入生产样式（DEC-137 同式）。
                .debug_selector(|| "task-menu-del".to_string())
                .on_click(cx.listener(
                    move |this, _e: &ClickEvent, _w: &mut Window, cx: &mut Context<Self>| {
                        this.ctx_delete(row, cx)
                    },
                ))
                .hover(co::hov_warm)
                .active(co::act_border),
        );
        co::ctx_anchor_at(
            self.ctx_pos.unwrap_or_default(),
            div().on_mouse_down_out(out).child(menu),
        )
    }

    // ── PG-C 主列表 ──

    /// 分组展开查询（DEC-185 · D-02/D-03）：显式值优先（grp_toggle 写入）；缺省
    /// ＝「待办」「今天」默认展开、其余折叠。
    fn grp_expanded(&self, name: &str) -> bool {
        self.expanded
            .get(name)
            .copied()
            .unwrap_or(matches!(name, "待办" | "今天"))
    }

    /// 分组折叠/展开切换（DEC-185 · D-05）：显式落值——跨重渲染/选项卡切
    /// 换/数据更新保持（实体常驻承载）；不写 DB（会话态边界不变）。
    fn grp_toggle(&mut self, name: &str, cx: &mut Context<Self>) {
        let next = !self.grp_expanded(name);
        self.expanded.insert(name.to_string(), next);
        cx.notify();
    }

    /// 分组头（DEC-185 · D-05 落点）：`co::grp` 装饰壳挂 id＋cursor_pointer
    /// ＋on_click——组头整行可点承载折叠切换（「点击右侧向下箭头切换」语义：
    /// ：箭头属组头右端子件、11px 级不可作独立稳定点击目标，整行命中含箭头
    /// 位）；chevron 随展开态翻转（展开＝chev_up · 折叠＝chev_down）；计数恒
    /// 现（空组已过滤 · DEC-185）。
    fn grp_head(
        &self,
        name: &'static str,
        n: usize,
        k: usize,
        cx: &mut Context<Self>,
    ) -> gpui::Stateful<Div> {
        let chev = if self.grp_expanded(name) {
            chev_up()
        } else {
            chev_down()
        };
        co::grp(format!("{name} {n}"), Some(chev))
            .id(("tc-grp", k))
            .debug_selector(move || format!("tc-grp-{}", k))
            .cursor_pointer()
            .on_click(cx.listener(
                move |this, _e: &ClickEvent, _w: &mut Window, cx: &mut Context<Self>| {
                    this.grp_toggle(name, cx)
                },
            ))
    }

    fn board_view(&self, cx: &mut Context<Self>) -> Div {
        // 分段控件（DEC-180 · 三处筛选栏内外边距统一：容器 padding px16/py6＝与
        // 拾记/集锦筛选栏同值同构；边框／圆角／半区 py6 等其他样式不动）。
        let seg = div()
            .flex()
            .px(px(16.0))
            .py(px(6.0))
            .border_1()
            .border_color(rgb(t::BORDER))
            .rounded(px(t::RAD_SM))
            .overflow_hidden()
            .child(
                div()
                    .flex_1()
                    .flex()
                    .items_center()
                    .justify_center()
                    .py(px(6.0))
                    .cursor_pointer()
                    .bg(if self.schedule_mode {
                        rgb(t::NEUTRAL)
                    } else {
                        rgb(t::WARM)
                    })
                    .child(co::txt(
                        "状态分组",
                        s::font(s::Lvl::Small),
                        if self.schedule_mode {
                            t::MUTED
                        } else {
                            t::PRIMARY
                        },
                    ))
                    .id("seg-status")
                    .on_click(cx.listener(Self::toggle_mode))
                    .active(co::act_border),
            )
            .child(
                div()
                    .flex_1()
                    .flex()
                    .items_center()
                    .justify_center()
                    .py(px(6.0))
                    .cursor_pointer()
                    .bg(if self.schedule_mode {
                        rgb(t::WARM)
                    } else {
                        rgb(t::NEUTRAL)
                    })
                    .child(co::txt(
                        "排期分组",
                        s::font(s::Lvl::Small),
                        if self.schedule_mode {
                            t::PRIMARY
                        } else {
                            t::MUTED
                        },
                    ))
                    .id("seg-schedule")
                    .on_click(cx.listener(Self::toggle_mode))
                    .active(co::act_border),
            );

        // 任务行构造（标题=内容数据 → 内容族 Kaiti SC · DEC-68）；右键=状态/排期菜单
        //（DEC-114 · menu_row=created 数据下标 · DEC-185 行址统一；None=无菜单）。
        let task_row = |r: &TaskRow,
                        i: usize,
                        menu_row: Option<usize>,
                        cx: &mut Context<Self>|
         -> gpui::Stateful<Div> {
            let base = co::row();
            let item = peek_item(r);
            let mut el = base
                .id(("task-row", i))
                // FAB 穿透测试取样面（DEC-148）：debug_bounds 按「task-row-<i>」定位
                // 注入压行；release no-op 不入生产样式（DEC-137 同式）。
                .debug_selector(move || format!("task-row-{}", i))
                .on_click(cx.listener(
                    move |this, _e: &ClickEvent, _w: &mut Window, cx: &mut Context<_>| {
                        this.task_row_click(&item, cx)
                    },
                ))
                .hover(co::hov_warm)
                .active(co::act_border);
            if let Some(row) = menu_row {
                let who = r.id.clone();
                el = el.on_aux_click(cx.listener(
                    move |this, e: &ClickEvent, _w: &mut Window, cx: &mut Context<_>| {
                        this.ctx_open(row, who.as_str(), e.position(), cx)
                    },
                ));
            }
            // 任务行主体＝单列流式（DEC-236 用户指令重构：原双列展示退役）。
            // 优先级 × 任务标签色合并为一枚带底色徽标，前置在标题最前方视觉
            // 上如同标题的首个字符——
            // ① 徽标高度＝紧随其后的首行行高（lh = Tab 字号 × 1.5 · 令牌
            //    ROW_TEXT_LH，随档位缩放同值同变）；
            // ② 与首行文字垂直居中同轴：title_line items_start ＋ 各前置件
            //    等高山轴（h=lh）——两行标题时对齐首行而非整块中心；
            // ③ 与标题紧邻：行内 gap 6px；
            // 有优先级＝文本胶囊（字=标题同字号 · 墨黑于亮色底 ≥5.5:1）；
            // 无优先级＝仅底色块（宽=字号 · 不写字 · DEC-234 单列化承载）。
            let f = s::font(s::Lvl::Tab);
            let lh = f * t::ROW_TEXT_LH;
            let (pri_label, pri_bg) = pri_chip(r.priority, r.mark);
            let badge = match pri_label {
                Some(label) => div()
                    .flex_none()
                    .debug_selector(move || format!("task-pri-{}", i))
                    .flex()
                    .items_center()
                    .px(px(6.0))
                    .h(px(lh))
                    .rounded_full()
                    .bg(rgb(pri_bg))
                    .child(co::txt(label, f, t::PRIMARY)),
                None => div()
                    .flex_none()
                    .debug_selector(move || format!("task-pri-{}", i))
                    .w(px(f))
                    .h(px(lh))
                    .rounded_full()
                    .bg(rgb(pri_bg)),
            };
            let title_line = div()
                .flex_1()
                // min_w_0（DEC-229）：长标题＋meta 多子项行内收纳。
                .min_w_0()
                .flex()
                .items_start()
                .gap(px(6.0))
                .child(badge);
            if r.done {
                // 完成态同流（DEC-236）：徽标 → 完成金点（前置等高山轴 h=lh
                // 内居中）→ 划线标题（≤2 行截断 · DEC-232）。
                el = el.child(
                    title_line
                        .child(
                            div()
                                .flex_none()
                                .h(px(lh))
                                .flex()
                                .items_center()
                                .child(co::dot(true)),
                        )
                        .child(
                            div().flex_1().min_w_0().child(
                                // min_w_0（DEC-229）：划线标题长串换行。
                                co::strike(r.title.clone())
                                    .line_height(relative(t::ROW_TEXT_LH))
                                    .line_clamp(t::ROW_CLAMP_TASK)
                                    .text_ellipsis(),
                            ),
                        ),
                );
            } else {
                // 未完成流（DEC-236）：徽标 → 标题（≤2 行截断）→ meta/来源
                // 后缀件（首行对齐随 items_start）。
                let title_line = title_line.child(
                    co::content_text(r.title.clone(), f, t::PRIMARY)
                        // 两行截断（DEC-232）；行号字段已随 DEC-232 移除。
                        .debug_selector(move || format!("task-title-{}", i))
                        .line_height(relative(t::ROW_TEXT_LH))
                        .line_clamp(t::ROW_CLAMP_TASK)
                        .text_ellipsis(),
                );
                // 后缀件（meta/来源）首行同轴（F-191-03）：等高山轴 h=lh 内
                // 居中——与徽标/完成点同法，两行标题行不再有顶端视差。
                let title_line = match r.meta_extra {
                    Some(m) => title_line.child(
                        div()
                            .flex_none()
                            .h(px(lh))
                            .flex()
                            .items_center()
                            .child(co::meta(format!(" · {m}"))),
                    ),
                    None => title_line,
                };
                let title_line = match r.src {
                    Some((label, _)) => title_line.child(
                        div().flex_none().h(px(lh)).flex().items_center().child(
                            co::txt(label.to_string(), s::font(s::Lvl::Meta), t::TERTIARY)
                                .id(("src-link", i)),
                        ),
                    ),
                    None => title_line,
                };
                el = el.child(title_line);
            }
            el.child(div().flex_none().child(co::tgo()))
        };

        let mut list = div().flex().flex_col();
        // 分组渲染（DEC-185 · 单循环收敛）：行全部来自 created，组序与空组过滤
        // 统一在 prepare_groups（数据准备阶段）；组头整行可点切换展开（DEC-20
        // 折叠交互 · chevron 随展开态翻转 DEC-185）；行 id 与右键菜单行址＝
        // created 数据下标（DEC-183 行级字段落值）；全空 → 列表级空态承接
        // board-empty；落库经 TASK-300 Created 事件宿主事务写（重启回填
        // seed_created）。
        let groups = prepare_groups(&self.created, self.schedule_mode);
        let groups_empty = groups.is_empty();
        for (k, (name, rows)) in groups.into_iter().enumerate() {
            list = list.child(self.grp_head(name, rows.len(), k, cx));
            if self.grp_expanded(name) {
                for (row_i, r) in rows {
                    list = list.child(task_row(r, row_i, Some(row_i), cx));
                }
            }
        }
        if groups_empty {
            // 全空（created 无任何非空组）→ 列表级空态承接 board-empty（PRD §3.4
            // 状态表 · 文案方向随 DEC-33 留评审）。
            list = list.child(co::empty_state(co::Icon::CircleDashed, "暂无任务"));
        }

        div()
            .flex_1()
            .min_h_0()
            .flex()
            .flex_col()
            .bg(rgb(t::NEUTRAL))
            .relative()
            .child(seg)
            .child(
                div()
                    .id("c-scroll")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .pb(px(76.0))
                    .child(list),
            )
            .child(
                div()
                    .absolute()
                    .right(px(t::FAB_DOCK))
                    .bottom(px(t::FAB_DOCK))
                    .child(
                        co::fab()
                            .id("fab-c")
                            .on_click(cx.listener(Self::fab_create))
                            .active(co::act_border),
                    ),
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
                        .child(co::toast(msg).id("toast-c")),
                )
            })
    }
}

impl Render for BoardView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        // 右键菜单视图根挂载（DEC-116 点击位锚定 · deferred anchored）：行菜单挂
        // board 根（单根承载）。
        let base = self.board_view(cx);
        match &self.row_ctx {
            Some((row, who)) => base.child(self.task_ctx_layer(*row, who, cx)),
            None => base,
        }
    }
}

// ───────────────────── 待办创建面板（DEC-165 · 独立窗） ─────────────────────

/// 待办创建面板 → 宿主事件（DEC-165）：提交入行 / 空稿收口披露 / 纯丢弃关窗。
#[derive(Clone, Debug)]
pub enum TaskCreateEvent {
    /// 创建/返回非空提交（面板守卫非空）：宿主入行（TASK-<seq> 语义在 BoardView
    /// ingest_created）+ 成功 Toast + 关窗。
    /// 提交（返回钮 · 保存并关闭 · DEC-232 创建钮移除）：优先级/颜色标记
    /// 下标随行（看板行左标签 + 落库双承载）。
    Submit {
        title: String,
        bucket: usize,
        priority: usize,
        mark: usize,
    },
    /// 返回钮空标题（保存并关闭口径延续）：宿主关窗 + 列表 Toast 披露。
    BackEmpty,
    /// 取消 / Esc：纯丢弃关窗（无 Toast）。
    Cancel,
}

/// 窗 → 宿主上抛通道（EditorSink 同型）。
pub type TaskCreateSink = Box<dyn Fn(&TaskCreateEvent, &mut gpui::App) + 'static>;

// 待办创建面板 Esc 通道（DEC-151 同构）：init 注册 escape→TaskCreateEscape
//（context「TaskCreate」），根 on_action 承接；keydown 分支为 IME composition
// 等绑定不适用路径的兑底。
gpui::actions!(parchora_ui, [TaskCreateEscape]);

/// 待办页 FAB「新建任务」→ 宿主转发通道（NoteSink 同型 · PanelView 弱引用转发）。
pub type TaskSink = std::rc::Rc<dyn Fn(&mut gpui::App)>;

/// 待办 DB 事件（TASK-300 持久化 · BoardView → 宿主事务写唯一通道）：创建落
/// todos 插行；状态/排期右键改值落 UPDATE（identifier 为稳定外键）。未注入=
/// 无操作（演示页自持）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskDbEvent {
    /// 创建（ingest_created 唯一权威点）：identifier TASK-n / 标题 / 排期桶
    /// 下标 / 优先级与标签下标（DEC-232 随行持久化 todos.priority/mark）。
    Created {
        identifier: String,
        title: String,
        bucket: usize,
        priority: usize,
        mark: usize,
    },
    /// 状态五选一（status_set）：STATUSES 下标（app 侧对账 core::TaskStatus）。
    Status { identifier: String, status: usize },
    /// 排期五桶（bucket_set）：BUCKETS 下标。
    Bucket { identifier: String, bucket: usize },
    /// 右键「删除」（ctx_delete）：宿主 DB 删行（identifier 外键逆向动作）。
    Deleted { identifier: String },
}

/// 待办 DB 事件上抛通道（ShellSink 同型 · 宿主事务写，不在租借域触窗）。
pub type TaskDbSink = std::rc::Rc<dyn Fn(&TaskDbEvent, &mut gpui::App)>;

/// 独立「创建待办事项」面板（DEC-165）：chrome 与「创建集锦」编辑窗同构（bar
/// 返回钮 + 标题 + 提示 · SURFACE 壳
/// · NEUTRAL 内容底）。数据流：提交经 TaskCreateEvent 上抛宿主（app 层转
/// PanelView.board 的 ingest_created——入行/序号/Toast 权威在 BoardView）；空标题
/// 创建钮留页面板 Toast（自持三件套 · BoardView show_toast 同式）；返回＝保存并
/// 关闭（空稿 BackEmpty 披露口径）；取消/Esc 纯丢弃。
///
/// 楷体落点（本面板）：标题 Input 与字段值/chips 文字走 FONT_CONTENT
/// 环境继承（wrapper font_family 级联 · zorite md_zone 同式；Input 内部若显式
/// 覆写需组件库 Theme 全局通道——真机验证腿，DEC-78 色通道口径不混用）。
pub struct TaskCreateView {
    /// 标题输入真引擎（延迟初始化——render 首帧建并聚焦
    /// 「开面板即入输入态」；勿开 clean_on_escape，Esc 冒泡根级关窗）。
    title_input: Option<Entity<InputState>>,
    /// 排期桶（Submit 契约字段 · DEC-177「任务时间」行由面板 chips 选择 ·
    /// 默认 0=今天）。
    bucket: usize,
    /// 状态选择（状态五值下标 · 默认待办=0 · DEC-168 真下拉受控字段）。
    status: usize,
    /// 优先级选择（优先级五值下标 · 默认无=0 与 spec none 同轴 · DEC-168 真下拉受控字段 ＋ DEC-169 D-01）。
    priority: usize,
    /// 颜色标记选择（七色下标 · 默认赤=0 · DEC-168）。
    mark: usize,
    /// 开启中的字段下拉（状态/优先级单开互斥 · collect toggle_dd 同型）。
    dd_open: Option<DdField>,
    /// 任务正文多行引擎（DEC-168 · render 首帧建不聚焦——初始
    /// 输入态归标题 · Textarea 自动生长 + 外层滚动 EditorShell src 同式）。
    task_input: Option<Entity<TextareaState>>,
    /// 任务域预填待落值（DEC-194 拾记「转为任务」承接）：引擎建前暂存，
    /// ensure_task 首帧建引擎即落值；None＝无预填。
    task_prefill: Option<String>,
    /// 任务域滚动句柄（tc-task-scroll 视口自持 · 跟随/断言通道）。
    task_scroll: gpui::ScrollHandle,
    /// 面板 Toast（空标题留页提示 · 1400ms 自动消失）。
    toast: Option<String>,
    toast_gen: u64,
    /// 未决 Toast timer（gen 防争 · DEC-72 同式）。
    toast_task: Option<gpui::Task<()>>,
    /// 键盘焦点柄（Esc 经根 key_context 分派 · EditorShell 同构）。
    focus: gpui::FocusHandle,
    /// 窗 → 宿主事件通道。
    on_event: Option<TaskCreateSink>,
}

impl TaskCreateView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            title_input: None,
            bucket: 0,
            status: 0,
            priority: 0,
            mark: 0,
            dd_open: None,
            task_input: None,
            task_prefill: None,
            task_scroll: gpui::ScrollHandle::new(),
            toast: None,
            toast_gen: 0,
            toast_task: None,
            focus: cx.focus_handle(),
            on_event: None,
        }
    }

    /// 键盘焦点柄（app 层开窗后 window.focus 定向 · EditorShell 同构）。
    pub fn focus_handle(&self) -> gpui::FocusHandle {
        self.focus.clone()
    }

    /// 面板 Toast 只读快照（DEC-166 失败驻留：宿主侧断言披露面）。
    pub fn toast(&self) -> Option<String> {
        self.toast.clone()
    }

    /// 注入窗 → 宿主事件通道（app 层提交/收口漏斗）。
    pub fn set_event_sink(&mut self, sink: TaskCreateSink) {
        self.on_event = Some(sink);
    }

    /// 「任务」域预填（DEC-194 拾记「转为任务」承接）：引擎已建 → set_value
    /// 直导（仅任务域，题名/排期/状态/优先级/标记不触）；引擎未建（开窗构造
    /// 期）→ 暂存待落值，ensure_task 首帧建引擎即导入。预填为显式转换动作
    /// ——已有内容让位；题名不动（已键入标题不被冲）。
    pub fn prefill_task(&mut self, content: String, w: &mut Window, cx: &mut Context<Self>) {
        match self.task_input.as_ref() {
            Some(st) => st.update(cx, |state, cx| state.set_value(content, w, cx)),
            None => self.task_prefill = Some(content),
        }
        cx.notify();
    }

    /// 上抛窗事件（生产链：listener 于本实体租借态内调用 → app sink 必须 defer
    /// 出借域 · DEC-77 同 EditorShell fire 口径）。
    pub fn fire(&self, ev: &TaskCreateEvent, cx: &mut gpui::App) {
        if let Some(f) = &self.on_event {
            f(ev, cx);
        }
    }

    /// 标题输入引擎延迟建（render 首帧调用；首次建即聚焦——开面板即入输入态）。
    fn ensure_title(&mut self, w: &mut Window, cx: &mut Context<Self>) {
        if self.title_input.is_some() {
            return;
        }
        let st = cx.new(|cx| {
            let mut state = InputState::new(w, cx);
            state.set_placeholder("标题…", w, cx);
            state
        });
        w.focus(&st.read(cx).focus_handle(cx), cx);
        self.title_input = Some(st);
        cx.notify();
    }

    /// 引擎活标题（trim · title_input None＝未建态与空标题同守卫）。
    fn draft_title(&self, cx: &Context<Self>) -> String {
        self.title_input
            .as_ref()
            .map(|st| st.read(cx).value().trim().to_string())
            .unwrap_or_default()
    }

    /// 返回（保存并关闭 · DEC-232 创建钮移除后为唯一提交入口；宿主漏斗 =
    /// 入行 + 关窗 + 主界面恢复幂等）：非空 = Submit；
    /// 空标题 = BackEmpty（宿主关窗 + 列表披露 Toast）。
    fn back(&mut self, cx: &mut Context<Self>) {
        let title = self.draft_title(cx);
        if title.is_empty() {
            self.fire(&TaskCreateEvent::BackEmpty, cx);
            return;
        }
        self.fire(
            &TaskCreateEvent::Submit {
                title,
                bucket: self.bucket,
                priority: self.priority,
                mark: self.mark,
            },
            cx,
        );
    }

    /// 取消（纯丢弃关窗 · 无 Toast；Esc 同源语义）。
    fn cancel(&mut self, cx: &mut Context<Self>) {
        self.fire(&TaskCreateEvent::Cancel, cx);
    }

    /// 字段下拉开合（互斥单开 · collect toggle_dd 同型 · DEC-168）。
    fn toggle_dd(&mut self, f: DdField, cx: &mut Context<Self>) {
        self.dd_open = if self.dd_open == Some(f) {
            None
        } else {
            Some(f)
        };
        cx.notify();
    }

    /// 下拉选项落值（单选即收 · collect dd_day_set 同型）。
    fn dd_pick(&mut self, f: DdField, i: usize, cx: &mut Context<Self>) {
        match f {
            DdField::Status => self.status = i,
            DdField::Priority => self.priority = i,
        }
        self.dd_open = None;
        cx.notify();
    }

    /// 任务时间桶选择（DEC-177「任务时间」行 · 互斥单选 · pick_mark 同口径）。
    fn pick(&mut self, b: usize, cx: &mut Context<Self>) {
        self.bucket = b;
        cx.notify();
    }

    /// 颜色标记选择（互斥单选 · 排期 chips 同口径）。
    fn pick_mark(&mut self, i: usize, cx: &mut Context<Self>) {
        self.mark = i;
        cx.notify();
    }

    /// 任务正文引擎延迟建（DEC-168 · render 首帧调用不聚焦——初始输入态归
    /// 标题；Textarea 自动生长，滚动由外层容口承接 · EditorShell src 同式）。
    /// 预填待落值（task_prefill）随建即导（DEC-194）。
    fn ensure_task(&mut self, w: &mut Window, cx: &mut Context<Self>) {
        if self.task_input.is_some() {
            return;
        }
        let st = cx.new(|cx| {
            let mut state = TextareaState::new(w, cx);
            state.set_placeholder("任务内容…", w, cx);
            state
        });
        if let Some(text) = self.task_prefill.take() {
            st.update(cx, |state, cx| state.set_value(text, w, cx));
        }
        self.task_input = Some(st);
        cx.notify();
    }

    /// 字段下拉控件（DEC-170 · DEC-173 堆叠：label 由块原语 field_blk 承载；
    /// 触发器填满所在块 flex_1＋relative wrapper）：触发器＝现值
    /// ＋chev（楷体）；开态挂 deferred 浮层（左对齐锚下方 · DEC-174·与
    /// 触发器左缘对齐）；out 收层＋stop 消耗本击（collect dd_anchor 同构 out 口径
    /// ——锚向域外维持右缘不动）。选项点击置值即收（单选）。
    fn dd_control(
        &self,
        field: DdField,
        options: &[&str],
        value: usize,
        cx: &mut Context<Self>,
    ) -> Div {
        let trigger_id = match field {
            DdField::Status => "tc-dd-status",
            DdField::Priority => "tc-dd-pri",
        };
        let open = self.dd_open == Some(field);
        let trigger = div()
            .id(trigger_id)
            .debug_selector(move || format!("{}-trg", trigger_id))
            .flex_1()
            .flex()
            .items_center()
            .gap(px(4.0))
            .cursor_pointer()
            .font_family(t::FONT_CONTENT)
            .on_click(cx.listener(
                move |this, _e: &ClickEvent, _w: &mut Window, cx: &mut Context<Self>| {
                    this.toggle_dd(field, cx)
                },
            ))
            .child(co::txt_f(
                options.get(value).copied().unwrap_or("?").to_string(),
                s::font(s::Lvl::Meta),
                t::MUTED,
                t::FONT_CONTENT,
                t::FONT_WEIGHT_LIGHT,
            ))
            .child(chev_down());
        let mut wrap = div().relative().flex_1().min_w_0().flex().child(trigger);
        if open {
            wrap = wrap.child(self.dd_layer(field, options, value, cx));
        }
        wrap
    }

    /// 下拉浮层（collect dd_anchor/panel_body 同构 out 口径 · 左对齐锚定
    /// 〔DEC-174·与触发器左缘对齐〕；选项行 = co::row
    /// ＋左标注＋右「✓」；out wrapper矩形＝浮层＋起浮间隙 → 点选项=界内不触 out）。
    fn dd_layer(
        &self,
        field: DdField,
        options: &[&str],
        value: usize,
        cx: &mut Context<Self>,
    ) -> gpui::AnyElement {
        let view = cx.weak_entity();
        let out = move |_: &gpui::MouseDownEvent, _: &mut gpui::Window, cx: &mut gpui::App| {
            let _ = view.update(cx, |v, cx| {
                if v.dd_open.is_some() {
                    v.dd_open = None;
                    cx.notify();
                }
            });
            cx.stop_propagation();
        };
        let opt_prefix = match field {
            DdField::Status => "tc-dd-status",
            DdField::Priority => "tc-dd-pri",
        };
        let mut rows = Vec::new();
        for (i, opt) in options.iter().enumerate() {
            let on = i == value;
            rows.push(
                co::row()
                    .justify_between()
                    .id((opt_prefix, i))
                    .debug_selector(move || format!("{}-opt-{}", opt_prefix, i))
                    .on_click(cx.listener(
                        move |this, _e: &ClickEvent, _w: &mut Window, cx: &mut Context<Self>| {
                            this.dd_pick(field, i, cx)
                        },
                    ))
                    .hover(co::hov_warm)
                    .active(co::act_border)
                    .child(co::txt(opt.to_string(), s::font(s::Lvl::Small), t::PRIMARY))
                    .child(if on {
                        co::txt("✓", s::font(s::Lvl::Small), t::PRIMARY)
                    } else {
                        div()
                    }),
            );
        }
        deferred(
            div()
                .absolute()
                .top(relative(1.0))
                .left(px(0.0))
                // 鼠标隔离面（DEC-167 FAB/ctx 同款 · 新链创建时即挂）：浮层叠
                // 在下方可点行（优先级触发器/任务区）之上而 hitbox 无遮挡时，选项
                // 点击会同合身后行击——block 挂 wrapper 截断 hover 栈，滚轮照常透
                // 传、浮层子树（行 hover/点击）在前方不受影响。
                .block_mouse_except_scroll()
                .on_mouse_down_out(out)
                .child(
                    div()
                        .mt(px(4.0))
                        .w(px(200.0))
                        .border_1()
                        .border_color(rgb(t::BORDER))
                        .rounded(px(t::RAD_MD))
                        .bg(rgb(t::SURFACE))
                        .overflow_hidden()
                        .flex()
                        .flex_col()
                        // 选项字楷体（DEC-165「下拉选项字体统一楷体」）。
                        .font_family(t::FONT_CONTENT)
                        .children(rows),
                ),
        )
        .into_any_element()
    }

    /// 标签 chip（DEC-170「标签」命名；DEC-177 选中视觉与任务
    /// 时间 chips 统一＝创建钮同色变量 bg PRIMARY ＋ 字 ON_PRIMARY）：色点＋楷体
    /// 标签；互斥单选 · 默认赤=0；未选中＝白底（纸白 SURFACE · 体系无纯白令牌先例）。
    fn mark_chip(&self, i: usize, cx: &mut Context<Self>) -> gpui::Stateful<Div> {
        let (label, hex) = MARKS[i];
        let mut el = div()
            .id(("tc-mark", i))
            .debug_selector(move || format!("tc-mark-{}", i))
            .flex()
            .items_center()
            .gap(px(4.0))
            .px(px(10.0))
            .py(px(1.0))
            .border_1()
            .border_color(rgb(t::BORDER))
            .rounded_full()
            .bg(rgb(t::SURFACE))
            .cursor_pointer()
            .font_family(t::FONT_CONTENT)
            .font_weight(gpui::FontWeight(t::FONT_WEIGHT_LIGHT))
            .text_size(s::px_of(s::Lvl::Meta))
            .text_color(rgb(t::MUTED))
            .child(div().w(px(8.0)).h(px(8.0)).rounded_full().bg(rgb(hex)))
            .child(label)
            .on_click(cx.listener(
                move |this, _e: &ClickEvent, _w: &mut Window, cx: &mut Context<Self>| {
                    this.pick_mark(i, cx)
                },
            ))
            .hover(co::hov_border)
            .active(co::act_border);
        if i == self.mark {
            el = el.bg(rgb(t::PRIMARY)).text_color(rgb(t::ON_PRIMARY));
        }
        el
    }

    /// 提交失败就地披露（DEC-166 失败驻留）：主面板亡（None/句柄失效）时宿主
    /// 漏斗回落调用——面板驻留不关窗，Toast 明示可重试（编辑不静默丢）。
    pub fn notify_submit_failed(&mut self, cx: &mut Context<Self>) {
        self.show_toast("主面板已关闭 · 未入列（重开后可重试）".into(), cx);
    }

    /// 面板 Toast（空标题留页提示 · 1400ms 自动消失；新 Toast bump 代次 + timer
    /// 持于字段 · BoardView show_toast 同式 DEC-72）。
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

    /// 表单块原语（DEC-173「label 上置 · 控件下发」堆叠布局）：块内
    /// label↔控件纵向间距 SP_SM，块间间距由 form 容器 gap SP_MD 统一承责；label
    /// 宋体 12 muted（UI 族照旧 · DEC-66 字体栈分工不混用）。
    fn field_blk(label: &'static str, control: impl gpui::IntoElement) -> Div {
        div()
            .flex()
            .flex_col()
            .gap(px(s::sp(t::SP_SM)))
            .child(co::txt(label, s::font(s::Lvl::Small), t::MUTED))
            .child(control)
    }

    /// 面板字段块降级面：值为内容族楷体（输入框/下拉选项字体统一）。
    fn field_blk_kai(label: &'static str, value: &'static str) -> Div {
        Self::field_blk(
            label,
            div()
                .flex()
                .items_center()
                .gap(px(4.0))
                .child(co::txt_f(
                    value,
                    s::font(s::Lvl::Meta),
                    t::MUTED,
                    t::FONT_CONTENT,
                    t::FONT_WEIGHT_LIGHT,
                ))
                .child(chev_down()),
        )
    }

    fn create_body(&self, cx: &mut Context<Self>) -> Div {
        // 标题栏（DEC-168 · DEC-232 创建钮移除＝返回即保存）：返回钮＋标题
        // 在左中、取消钮在右——justify_between 分布保持标题居中位；两钮统一
        // 钉 TC_BAR_BTN_H（「高度严格一致」· 结构性保证）；按钮风格/样式保持。
        let bar = co::bar()
            .child(
                co::ibtn_icon_text(co::Icon::ArrowLeft, "返回", t::PRIMARY)
                    .id("tc-panel-back")
                    .flex()
                    .items_center()
                    .justify_center()
                    .py(px(0.0))
                    .h(px(TC_BAR_BTN_H))
                    // 返回＝保存并关闭：back 分层语义不变。
                    .on_click(cx.listener(
                        |this, _e: &ClickEvent, _w: &mut Window, cx: &mut Context<Self>| {
                            this.back(cx)
                        },
                    ))
                    .active(co::act_border),
            )
            .child(co::txt_f(
                "新建任务",
                s::font(s::Lvl::Small),
                t::PRIMARY,
                t::FONT_BODY,
                t::FONT_WEIGHT_REGULAR,
            ))
            .child(
                div().flex().items_center().gap(px(8.0)).child(
                    co::btn_sec("取消")
                        .id("tc-panel-cancel")
                        .flex()
                        .items_center()
                        .justify_center()
                        .py(px(0.0))
                        .h(px(TC_BAR_BTN_H))
                        .on_click(cx.listener(
                            |this, _e: &ClickEvent, _w: &mut Window, cx: &mut Context<Self>| {
                                this.cancel(cx)
                            },
                        ))
                        .active(co::act_border_ink),
                ),
            );

        // 第一块：任务时间（DEC-177 · bucket 契约不变，Submit 携面板选择入看板
        // 行）：五桶互斥单选 · 默认
        // 今天=0 · 楷体选项字；未选中＝白底（SURFACE），选中＝标题栏「创建」钮
        // 同色变量（bg PRIMARY ＋ 字 ON_PRIMARY · 与标签 chips 选中态统一）。
        let mut sched_row = div().flex().flex_wrap().items_center().gap(px(4.0));
        for (i, b) in BUCKETS.iter().enumerate() {
            let mut chip = co::chip(b.to_string())
                .id(("tc-bucket", i))
                .debug_selector(move || format!("tc-bucket-{}", i))
                .on_click(cx.listener(
                    move |this, _e: &ClickEvent, _w: &mut Window, cx: &mut Context<Self>| {
                        this.pick(i, cx)
                    },
                ))
                .active(co::act_border)
                .font_family(t::FONT_CONTENT)
                .bg(rgb(t::SURFACE));
            if i == self.bucket {
                chip = chip.bg(rgb(t::PRIMARY)).text_color(rgb(t::ON_PRIMARY));
            }
            sched_row = sched_row.child(chip);
        }
        let sched_blk = Self::field_blk("任务时间", sched_row);

        // 第二块：标题（DEC-173 label 上置堆叠，白底单行文本盒 · 延迟引
        // 擎；块列内子件默认拉满宽，盒不钉 flex_1/min_w_0）。
        let title_blk = match &self.title_input {
            Some(st) => Self::field_blk(
                "标题",
                div()
                    .h(px(TC_FIELD_H))
                    .flex()
                    .items_center()
                    .bg(rgb(t::SURFACE))
                    .border_1()
                    .border_color(rgb(t::BORDER))
                    .rounded(px(t::RAD_SM))
                    .overflow_hidden()
                    .font_family(t::FONT_CONTENT)
                    .child(Input::new(st).appearance(false).bordered(false)),
            ),
            // ensure_title 于 render 头先行建态——None 仅构造初帧前存在，静态
            // degrade 承载同详情字段块。
            None => Self::field_blk_kai("标题", "标题…"),
        };

        // 第三块：标签独立整行（DEC-179 · 七色互斥单选 · 默认赤=0 · 未选中白底随
        // mark_chip；全行可容七枚 chips，窄窗 flex_wrap 折行承接）。
        let mut marks_row = div().flex().flex_wrap().items_center().gap(px(4.0));
        for i in 0..MARKS.len() {
            marks_row = marks_row.child(self.mark_chip(i, cx));
        }
        let tags_blk = Self::field_blk("标签", marks_row);

        // 第四块：状态｜优先级同行两列（DEC-179；左列＝状态、右
        // 列＝优先级 · 皆真下拉 DEC-168）：两列 flex_1 等宽 · 列间距 SP_MD ·
        // min_w_0 自适应（label 上置堆叠随 field_blk）。
        let status_pri_row = div()
            .flex()
            .items_start()
            .gap(px(s::sp(t::SP_MD)))
            .child(
                Self::field_blk(
                    "状态",
                    self.dd_control(DdField::Status, &STATUSES, self.status, cx),
                )
                .flex_1()
                .min_w_0(),
            )
            .child(
                Self::field_blk(
                    "优先级",
                    self.dd_control(DdField::Priority, &PRIORITIES, self.priority, cx),
                )
                .flex_1()
                .min_w_0(),
            );

        // 末块：任务文本域（DEC-168 延迟引擎；DEC-173 label 上置·块占表单余高——
        // Textarea 自动生长、溢出滚动由外层视口承接（EditorShell src 同式：
        // track_scroll 视口 ＋ Scrollbar overlay）；楷体环境继承。
        let task_blk = match &self.task_input {
            Some(st) => {
                let handle = self.task_scroll.clone();
                div()
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .flex_col()
                    .gap(px(s::sp(t::SP_SM)))
                    .child(co::txt("任务", s::font(s::Lvl::Small), t::MUTED))
                    .child(
                        div()
                            .debug_selector(|| "tc-task-wrap".to_string())
                            .flex_1()
                            .min_h_0()
                            .min_w_0()
                            .flex()
                            .flex_col()
                            .bg(rgb(t::SURFACE))
                            .border_1()
                            .border_color(rgb(t::BORDER))
                            .rounded(px(t::RAD_SM))
                            .overflow_hidden()
                            .child(
                                div()
                                    .id("tc-task-scroll")
                                    .flex_1()
                                    .min_h_0()
                                    .relative()
                                    .overflow_hidden()
                                    .child(
                                        div()
                                            .id("tc-task-scroll-area")
                                            .size_full()
                                            .flex()
                                            .flex_col()
                                            .track_scroll(&handle)
                                            .overflow_y_scroll()
                                            .lock_scroll_axis()
                                            .child(
                                                div()
                                                    .debug_selector(|| "tc-task-zone".to_string())
                                                    // 内边距 0（DEC-177）。
                                                    .min_h_full()
                                                    .flex()
                                                    .flex_col()
                                                    .font_family(t::FONT_CONTENT)
                                                    .text_size(px(s::font(s::Lvl::Small)))
                                                    .child(
                                                        Textarea::new(st)
                                                            .appearance(false)
                                                            .bordered(false)
                                                            .flex_1(),
                                                    ),
                                            ),
                                    )
                                    .child(
                                        div().absolute().inset_0().child(
                                            Scrollbar::new(&handle)
                                                .id("tc-task-sb")
                                                .axis(ScrollbarAxis::Vertical)
                                                .viewport_from_layout(),
                                        ),
                                    ),
                            ),
                    )
            }
            None => Self::field_blk_kai("任务", "任务内容…"),
        };

        // 表单卡（DEC-170 统一背景与 16px 四边外边距 · DEC-173 堆叠块序 ·
        // DEC-177 任务时间行 · DEC-179 标签整行＋状态｜优先级同行两列）：块间
        // 垂直间距 SP_MD 统一（块内 label↔控件 SP_SM 随 field_blk）。
        let form = div()
            .debug_selector(|| "tc-form".to_string())
            .flex_1()
            .min_h_0()
            .m(px(16.0))
            .p(px(s::sp(t::SP_SM)))
            .bg(rgb(t::WARM))
            .rounded(px(t::RAD_MD))
            .flex()
            .flex_col()
            .gap(px(s::sp(t::SP_MD)))
            .child(sched_blk)
            .child(title_blk)
            .child(tags_blk)
            .child(status_pri_row)
            .child(task_blk);

        div()
            .debug_selector(|| "tc-body".to_string())
            .flex_1()
            .min_h_0()
            .relative()
            .flex()
            .flex_col()
            .bg(rgb(t::NEUTRAL))
            .child(bar)
            .child(form)
            .when_some(self.toast.clone(), |el, msg| {
                el.child(
                    div()
                        .absolute()
                        .bottom(px(24.0))
                        .left_0()
                        .right_0()
                        .flex()
                        .justify_center()
                        .child(co::toast(msg).id("toast-tc")),
                )
            })
    }
}

impl Render for TaskCreateView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let _s = crate::scale::enter(window, cx);
        // 标题引擎延迟建（DEC-78 src_textarea 同式 · 保 new 构造签名与窗测不动）；
        // 首建即聚焦（开面板即入输入态）。
        self.ensure_title(window, cx);
        // 任务正文引擎延迟建（DEC-168 · 同式不聚焦——初始输入态归标题）。
        self.ensure_task(window, cx);

        div()
            .size_full()
            // Esc 退出（EditorShell DEC-151 同构）：根 key_context + track_focus →
            // Cancel 上抛宿主关窗（独立任务窗）；action 通道（init 注册绑定）+
            // keydown 兑底（IME composition 等绑定不适用路径）。
            .key_context("TaskCreate")
            .track_focus(&self.focus)
            .on_action(cx.listener(
                |this, _: &TaskCreateEscape, _w: &mut Window, cx: &mut Context<Self>| {
                    // 下拉开 → Esc 只收层（级联末位 · DEC-114）；否则关窗。
                    if this.dd_open.take().is_some() {
                        cx.notify();
                    } else {
                        this.cancel(cx);
                    }
                    cx.stop_propagation();
                },
            ))
            .on_key_down(cx.listener(
                |this, ev: &gpui::KeyDownEvent, _w: &mut Window, cx: &mut Context<Self>| {
                    if ev.keystroke.key == "escape" {
                        if this.dd_open.take().is_some() {
                            cx.notify();
                        } else {
                            this.cancel(cx);
                        }
                        cx.stop_propagation();
                    }
                },
            ))
            .flex()
            .flex_col()
            .bg(rgb(t::SURFACE))
            .child(self.create_body(cx))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 任务行单击 → 统一详情窗上抛（详情=Peek 承载）。
    #[gpui::test]
    async fn task_click_emits_preview(cx: &mut gpui::TestAppContext) {
        let titles: std::rc::Rc<std::cell::RefCell<Vec<String>>> = Default::default();
        let view = cx.new(|cx| {
            let mut v = BoardView::new(cx);
            let titles = titles.clone();
            v.set_peek_sink(std::rc::Rc::new(
                move |item: &PeekItem, _: &mut gpui::App| {
                    titles.borrow_mut().push(item.title.clone());
                },
            ));
            v
        });
        let title = view.update(cx, |v, cx| {
            let item = peek_item(&TaskRow {
                title: "评审会准备".into(),
                id: "TASK-1".into(),
                ..Default::default()
            });
            let title = item.title.clone();
            v.task_row_click(&item, cx);
            title
        });
        assert_eq!(title, "评审会准备");
        assert_eq!(*titles.borrow(), vec!["评审会准备".to_string()]);
    }

    // FAB 点击穿透（DEC-148 · 真实鼠标路径）：小窗 + 48 条 created 压行保证任务
    // 列表溢出、fab-c 下方被可点击行覆盖。点击 FAB 圆心只上抛开创建面板事件
    //（task sink 计数 · DEC-165 面板窗）+ 无误上抛详情；
    // 下方行不得被穿透误触发；对照：FAB 上缘 30px（遮挡面之外、压行带内）
    // 任务行点击照常上抛详情（防过修）。
    #[gpui::test]
    async fn fab_click_no_passthrough_to_task_rows(cx: &mut gpui::TestAppContext) {
        use std::ops::Deref as _;
        let titles: std::rc::Rc<std::cell::RefCell<Vec<String>>> = Default::default();
        let opened: std::rc::Rc<std::cell::RefCell<usize>> = Default::default();
        let window = cx.open_window(gpui::size(gpui::px(900.0), gpui::px(640.0)), |_w, cx| {
            crate::init(cx);
            let titles = titles.clone();
            let opened = opened.clone();
            let mut v = BoardView::new(cx);
            v.created = (0..48)
                .map(|i| TaskRow {
                    title: format!("穿透压行 {}", i),
                    id: format!("TASK-P{}", i),
                    bucket: 0,
                    ..Default::default()
                })
                .collect();
            v.set_peek_sink(std::rc::Rc::new(
                move |item: &PeekItem, _: &mut gpui::App| {
                    titles.borrow_mut().push(item.title.clone());
                },
            ));
            v.set_task_sink(std::rc::Rc::new(move |_: &mut gpui::App| {
                *opened.borrow_mut() += 1;
            }));
            v
        });
        let mut cx = gpui::VisualTestContext::from_window(*window.deref(), cx);
        cx.run_until_parked();
        let fab = cx
            .debug_bounds("fab")
            .unwrap_or_else(|| panic!("fab 未录制（debug_selector 未生效）"));
        // 前置自校验：注入压行带确有行压在 FAB 区域之下（几何失效即拒测，防空转）。
        let under = (0..70).any(|k| {
            cx.debug_bounds(Box::leak(format!("task-row-{k}").into_boxed_str()))
                .is_some_and(|r| r.intersects(&fab))
        });
        assert!(under, "注入压行应有行覆盖 fab-c 区域（几何前置失效）");
        // 对照组先打：遮挡面之外（完整位于 FAB 上方）的任务行真实点击 → 详情
        // 上抛照常（详情=Peek 上抛）。
        let ctrl = (0..70)
            .find_map(|k| {
                cx.debug_bounds(Box::leak(format!("task-row-{k}").into_boxed_str()))
                    .filter(|r| r.origin.y + r.size.height < fab.origin.y - gpui::px(8.0))
                    .map(|r| r.center())
            })
            .unwrap_or_else(|| panic!("应有完整位于 FAB 上方的任务行（对照组前置失效）"));
        cx.simulate_click(ctrl, Default::default());
        let peek_n = window
            .update(&mut cx, |_v, _w, _cx| titles.borrow().len())
            .unwrap();
        assert_eq!(peek_n, 1, "对照组行应恰好上抛一次详情（实得 {}）", peek_n);
        // 清计数后出帧重绘——统一详情窗为独立窗（视图不替换），FAB 断言在
        // 看板态下直接进行（无两层界面切换）。
        titles.borrow_mut().clear();
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
        });
        cx.run_until_parked();
        let fab2 = cx
            .debug_bounds("fab")
            .unwrap_or_else(|| panic!("fab 未重录（注入压行态应保持看板）"));
        // FAB 圆心：只上抛开创建面板事件（task sink 计数）；穿透若在，压行会再
        // 上抛一条详情——断言双向不漂移。
        cx.simulate_click(fab2.center(), Default::default());
        let (opened_n, peek_n) = window
            .update(&mut cx, |_v, _w, _cx| {
                (*opened.borrow(), titles.borrow().len())
            })
            .unwrap();
        assert_eq!(opened_n, 1, "FAB 单击应恰好上抛一次开面板事件");
        assert_eq!(peek_n, 0, "FAB 下方压行不得被穿透误触发（实得 {}）", peek_n);
    }

    // 行左优先级标签与标题截断（DEC-232）：优先级>0 行渲染「task-pri-<i>」
    // 标签面（DEC-233 恒显）；长标题行两行截断（行高增幅恰好一线差而非全量
    // 换行行数）；无优先级行不渲染标签且标题单行基准。
    #[gpui::test]
    async fn task_row_priority_chip_and_two_line_clamp(cx: &mut gpui::TestAppContext) {
        use std::ops::Deref as _;
        let long_title = "待办标题超长截断测试串".repeat(30);
        let window = cx.open_window(gpui::size(gpui::px(900.0), gpui::px(640.0)), |_w, cx| {
            crate::init(cx);
            let mut v = BoardView::new(cx);
            v.created = vec![
                TaskRow {
                    title: "短标题".into(),
                    id: "TASK-1".into(),
                    bucket: 0,
                    ..Default::default()
                },
                TaskRow {
                    title: long_title.clone(),
                    id: "TASK-2".into(),
                    bucket: 0,
                    ..Default::default()
                },
                TaskRow {
                    title: "高优先级绿标签行".into(),
                    id: "TASK-3".into(),
                    bucket: 0,
                    priority: 3,
                    mark: 3,
                    ..Default::default()
                },
            ];
            v
        });
        let mut cx = gpui::VisualTestContext::from_window(*window.deref(), cx);
        cx.run_until_parked();
        // ① 两行截断判据（DEC-233 改判——行基高由恒显标签驱动，行级差分
        // 不再干净）：直接对标题文本元素断言——长标题恰为 2 行（增量＝
        // 单行基准），不出现三行及以上。
        let t_short = cx
            .debug_bounds("task-title-0")
            .unwrap_or_else(|| panic!("短标题文本未录制"));
        let t_long = cx
            .debug_bounds("task-title-1")
            .unwrap_or_else(|| panic!("长标题文本未录制"));
        let tdiff = t_long.size.height - t_short.size.height;
        assert!(
            tdiff > t_short.size.height * 0.85,
            "长标题应换行至两行（diff={tdiff:?} base={:?}",
            t_short.size.height
        );
        assert!(
            tdiff <= t_short.size.height * 1.2,
            "长标题应两行封顶（diff={tdiff:?} base={:?}",
            t_short.size.height
        );
        // ② 优先级标签判据（DEC-233 恒显）：每行都渲染标签。
        let chip = cx
            .debug_bounds("task-pri-2")
            .unwrap_or_else(|| panic!("优先级标签未录制"));
        assert!(
            cx.debug_bounds("task-pri-0").is_some(),
            "无优先级行应恒显「无」标签（DEC-233）"
        );
        let row3 = cx
            .debug_bounds("task-row-2")
            .unwrap_or_else(|| panic!("高优先级行未录制"));
        assert!(
            chip.left() >= row3.left() && chip.right() <= row3.right(),
            "标签应收纳在行内左缘（chip={chip:?} row={row3:?}）"
        );
        // ③ 等高判据（DEC-236 单列流式）：徽标高＝首行行高（＝单行标题
        // 文本元素高，lh 令牌同源）。
        let badge0 = cx
            .debug_bounds("task-pri-0")
            .unwrap_or_else(|| panic!("无级块未录制"));
        assert!(
            (badge0.size.height - t_short.size.height).abs() <= gpui::px(1.0),
            "徽标高应与首行文字一致（badge={badge0:?} title={t_short:?}）"
        );
        // ④ 首行同轴判据（DEC-236）：两行标题内徽标中心＝首行行中心
        //（items_start＋等高山轴——不对齐整块中心）。
        let first_center = t_long.origin.y + t_short.size.height / 2.0;
        let badge1 = cx
            .debug_bounds("task-pri-1")
            .unwrap_or_else(|| panic!("两行行标签未录制"));
        assert!(
            (badge1.origin.y + badge1.size.height / 2.0 - first_center).abs() <= gpui::px(1.0),
            "徽标应对齐首行中心（badge={badge1:?} first_center={first_center}）"
        );
        // ⑤ 紧邻判据（DEC-236）：有级行徽标右缘 ↔ 文字左缘＝行内 gap 6。
        let t2 = cx
            .debug_bounds("task-title-2")
            .unwrap_or_else(|| panic!("有级行标题未录制"));
        assert!(
            ((t2.origin.x - chip.origin.x - chip.size.width) - gpui::px(6.0)).abs()
                <= gpui::px(1.0),
            "徽标应紧邻标题（gap 实测 {:?}",
            (t2.origin.x - chip.origin.x - chip.size.width)
        );
    }

    // 优先级标签样式映射（DEC-232 纯函数 · DEC-233 恒显）：恒返回
    //（PRIORITIES[priority], MARKS[mark]）（域归一 %5 / %7）。
    #[test]
    fn pri_chip_mapping() {
        // 无优先级 → 仅背景色点（文本 None · DEC-234）。
        assert_eq!(pri_chip(0, 0), (None, 0xFF6B6B));
        assert_eq!(pri_chip(3, 3), (Some("高"), 0x6BE585));
        assert_eq!(pri_chip(4, 6), (Some("紧急"), 0xB57BFF));
        assert_eq!(pri_chip(99, 9).0, Some("紧急")); // 域外下标归一不 panic（99%5=4）
    }

    // 右键菜单（DEC-114 · DEC-183）：状态/排期赋值真实落行级字段（status
    // 同步 done 划线 · 分组重排 by status/bucket）。
    #[gpui::test]
    async fn status_and_bucket_set_on_created_rows(cx: &mut gpui::TestAppContext) {
        let view = cx.new(|cx| {
            let mut v = BoardView::new(cx);
            v.ingest_created("评审会准备".to_string(), 0, 0, 0, cx);
            v.ingest_created("第二项".to_string(), 1, 0, 0, cx);
            v
        });
        view.update(cx, |v, cx| v.status_set(0, 3, "TASK-1", cx));
        let (st, done, ctx_closed) = view.update(cx, |v, _| {
            (v.created[0].status, v.created[0].done, v.row_ctx.is_none())
        });
        assert_eq!(st, 3);
        assert!(done, "完成态 ⇔ done 划线（status/完成语义同源）");
        assert!(ctx_closed);
        view.update(cx, |v, cx| v.bucket_set(1, 0, "TASK-2", cx));
        let bk = view.update(cx, |v, _| v.created[1].bucket);
        assert_eq!(bk, 0, "行级 bucket 落值 · 排期模式随值归组");
        // 回填不落组：状态仍为待办（0）。
        assert_eq!(view.update(cx, |v, _| v.created[1].status), 0);
    }

    #[gpui::test]
    async fn ctx_delete_removes_row_and_emits_deleted_event(cx: &mut gpui::TestAppContext) {
        let events: std::rc::Rc<std::cell::RefCell<Vec<TaskDbEvent>>> = Default::default();
        let view = cx.new(|cx| {
            let mut v = BoardView::new(cx);
            let events = events.clone();
            v.set_db_sink(std::rc::Rc::new(
                move |ev: &TaskDbEvent, _: &mut gpui::App| {
                    events.borrow_mut().push(ev.clone());
                },
            ));
            v.ingest_created("评审会准备".to_string(), 0, 0, 0, cx);
            v.ingest_created("第二项".to_string(), 1, 0, 0, cx);
            v
        });
        view.update(cx, |v, cx| v.ctx_delete(0, cx));
        let (titles, ctx_closed, toast) = view.update(cx, |v, _cx| {
            (v.board_titles(), v.row_ctx.is_none(), v.toast.clone())
        });
        assert_eq!(titles, vec!["第二项".to_string()], "目标行移除·余行序保持");
        assert!(ctx_closed, "菜单即收");
        assert_eq!(
            toast.as_deref(),
            Some("已删除 TASK-1"),
            "删除 Toast·行序号语义"
        );
        let (n_events, last) = {
            let evs = events.borrow();
            (evs.len(), evs.last().cloned())
        };
        assert_eq!(n_events, 3, "创建两拍 + 删除一拍");
        assert_eq!(
            last,
            Some(TaskDbEvent::Deleted {
                identifier: "TASK-1".into()
            }),
            "Deleted 上抛·identifier 外键逆向动作"
        );
        // 越界行址幂等无害：不上抛不再触发 sink（事件数不变）。
        view.update(cx, |v, cx| v.ctx_delete(9, cx));
        assert_eq!(events.borrow().len(), 3, "越界行址不上抛");
    }

    // 右键菜单（DEC-114）：Esc 级联：行菜单首位消化。
    #[gpui::test]
    async fn tasks_dismiss_cascade(cx: &mut gpui::TestAppContext) {
        let view = cx.new(BoardView::new);
        let seq = view.update(cx, |v, cx| {
            let nothing = v.dismiss_overlay(cx); // 无浮层 → 不消化
            v.ctx_open(1, "TASK-2", gpui::point(gpui::px(14.0), gpui::px(16.0)), cx); // 行菜单开
            let consumed = v.dismiss_overlay(cx); // 行菜单级联消化
            v.ctx_open(0, "TASK-1", gpui::point(gpui::px(14.0), gpui::px(16.0)), cx);
            let again = v.dismiss_overlay(cx);
            (nothing, consumed, again)
        });
        assert_eq!(seq, (false, true, true));
    }

    // 创建面板事件流（DEC-165）：非空创建/返回 →
    // Submit（bucket 默认今天 0 · DEC-168）；空标题创建钮留页提示
    //（面板 Toast · 不出事件，无事件即无宿主入行无序号推进）；返回空标题 →
    // BackEmpty；取消 → Cancel 纯丢弃。键入值以 set_value 通道承载（InputState
    // 同引擎写路径；真机 IME 为人工腿）。
    #[gpui::test]
    async fn task_create_panel_event_flow(cx: &mut gpui::TestAppContext) {
        let (view, cx) = cx.add_window_view(|_w, cx| {
            // 面板渲染真 Input：组件库 Theme 全局在测试域同需注册
            //（wrapper 每帧读 cx.theme() · 无 init 则 panic）。
            crate::init(cx);
            TaskCreateView::new(cx)
        });
        let events: std::rc::Rc<std::cell::RefCell<Vec<TaskCreateEvent>>> = Default::default();
        view.update(cx, |v, _| {
            let events = events.clone();
            v.set_event_sink(Box::new(move |ev: &TaskCreateEvent, _: &mut gpui::App| {
                events.borrow_mut().push(ev.clone());
            }));
        });
        // 标题引擎建（render 首帧同源 ensure_title 通道）+ 初值空。
        cx.update(|window, app| {
            view.update(app, |v, cx| v.ensure_title(window, cx));
        });
        let ready = view.update(cx, |v, cx| {
            v.title_input
                .as_ref()
                .map(|st| st.read(cx).value().is_empty())
                .unwrap_or(false)
        });
        assert!(ready, "真输入引擎应已建且初值空");
        let set_title =
            |view: &Entity<TaskCreateView>, cx: &mut gpui::VisualTestContext, s: &str| {
                cx.update(|window, app| {
                    view.update(app, |v, cx| {
                        if let Some(st) = &v.title_input {
                            st.update(cx, |state, scx| state.set_value(s, window, scx));
                        }
                    });
                });
            };
        set_title(&view, cx, "验收任务甲");
        // 非空返回 → Submit{title, bucket=默认今天 0, priority/mark=默认 0}
        //（DEC-232 创建钮移除＝返回唯一提交入口；窗关闭在宿主漏斗，测试域
        // 仅断言事件面）。
        view.update(cx, |v, cx| {
            v.priority = 2;
            v.mark = 5;
            v.back(cx);
        });
        let ok = match events.borrow().last() {
            Some(TaskCreateEvent::Submit {
                title,
                bucket,
                priority,
                mark,
            }) => title == "验收任务甲" && *bucket == 0 && *priority == 2 && *mark == 5,
            other => panic!("非空返回应上抛 Submit（实得 {:?}）", other),
        };
        assert!(ok, "Submit 应携标题/默认桶/优先级/标签（DEC-232）");
        // 空标题守卫（创建钮留页）已随 DEC-232 创建钮移除退役——空标题由
        // BackEmpty 口径承载（下一断言）。
        set_title(&view, cx, "");
        // 返回空标题 → BackEmpty（宿主关窗 + 列表披露口径）。
        view.update(cx, |v, cx| v.back(cx));
        assert!(
            matches!(events.borrow().last(), Some(TaskCreateEvent::BackEmpty)),
            "返回空标题应上抛 BackEmpty"
        );
        // 返回非空 = Submit 同源（保存并关闭口径）。
        set_title(&view, cx, "验收任务乙");
        view.update(cx, |v, cx| {
            v.priority = 0;
            v.mark = 3;
            v.back(cx);
        });
        let ok2 = match events.borrow().last() {
            Some(TaskCreateEvent::Submit {
                title,
                bucket,
                priority,
                mark,
            }) => title == "验收任务乙" && *bucket == 0 && *priority == 0 && *mark == 3,
            other => panic!("返回非空应同源 Submit（实得 {:?}）", other),
        };
        assert!(ok2, "返回提交应携标题/默认桶/优先级/标签（DEC-232）");
        // 取消：纯丢弃 Cancel。
        view.update(cx, |v, cx| v.cancel(cx));
        assert!(
            matches!(events.borrow().last(), Some(TaskCreateEvent::Cancel)),
            "取消应上抛 Cancel"
        );
    }

    // 创建面板提交回灌（DEC-165）：ingest_created 单一入行权威面——TASK-<seq> 起
    // 序自增 + 面板携带 bucket 直落字段（chips 真实落 bucket 通道）+ 成功 Toast
    // 列表面反馈；空稿收口披露（BackEmpty 漏斗落点）独立 Toast 面。
    #[gpui::test]
    async fn board_ingest_created_appends_row(cx: &mut gpui::TestAppContext) {
        let view = cx.new(BoardView::new);
        view.update(cx, |v, cx| {
            v.ingest_created("验收任务甲".into(), 4, 0, 0, cx)
        });
        let (created_n, id, next_seq, bucket, toast) = view.update(cx, |v, _| {
            (
                v.created.len(),
                v.created[0].id.clone(),
                v.next_seq,
                v.created[0].bucket,
                v.toast.clone(),
            )
        });
        assert_eq!(created_n, 1, "入行数");
        assert_eq!(id, "TASK-1", "序号自 TASK-1 起");
        assert_eq!(next_seq, 2, "序号已推进");
        assert_eq!(bucket, 4, "面板携带桶落行");
        assert!(toast.is_some(), "成功 Toast（列表面反馈）");
        // 面板桶选择非默认值亦直落字段。
        view.update(cx, |v, cx| v.ingest_created("任务乙".into(), 1, 0, 0, cx));
        let (created_n, bk, seq) = view.update(cx, |v, _| {
            (v.created.len(), v.created[1].bucket, v.next_seq)
        });
        assert_eq!((created_n, bk, seq), (2, 1, 3));
        // 空稿收口披露（BackEmpty 漏斗落点 · Toast 独立面）。
        view.update(cx, |v, cx| v.task_create_closed_empty(cx));
        let toast = view.update(cx, |v, _| v.toast.clone());
        assert!(toast.is_some(), "空稿收口披露 Toast");
    }

    // 提交失败就地披露（DEC-166 失败驻留 · 宿主漏斗回落）：面板自持 Toast 直落
    //（不渲染 Input · 无需 crate::init；文案锚主面板重开口径）。
    #[gpui::test]
    async fn task_create_notify_submit_failed_shows_toast(cx: &mut gpui::TestAppContext) {
        let view = cx.new(TaskCreateView::new);
        view.update(cx, |v, cx| v.notify_submit_failed(cx));
        let toast = view.update(cx, |v, _| v.toast.clone());
        assert_eq!(
            toast.as_deref(),
            Some("主面板已关闭 · 未入列（重开后可重试）"),
            "失败驻留 Toast 文案"
        );
    }

    // 新表单字段语义（DEC-168 · 方法级）：默认值（排期今天 0/状态待办 0/优先级
    // 无 0（DEC-169 D-01）/颜色赤 0）；字段下拉单开互斥、选项落值即收；颜色标记互斥单选。
    #[gpui::test]
    async fn task_create_form_fields_defaults_and_dd(cx: &mut gpui::TestAppContext) {
        let (view, cx) = cx.add_window_view(|_w, cx| {
            crate::init(cx);
            TaskCreateView::new(cx)
        });
        let d0 = view.update(cx, |v, _| {
            (v.bucket, v.status, v.priority, v.mark, v.dd_open)
        });
        assert_eq!(
            d0,
            (0, 0, 0, 0, None),
            "默认：排期今天/状态待办/优先级无/颜色赤/无下拉（DEC-168 行序 ＋ DEC-169 D-01 默认）"
        );
        // 状态 dropdown开 → 再开优先级应单开互斥置空状态。
        view.update(cx, |v, cx| v.toggle_dd(DdField::Status, cx));
        let opened = view.update(cx, |v, _| v.dd_open == Some(DdField::Status));
        assert!(opened, "状态下拉应开启");
        view.update(cx, |v, cx| v.toggle_dd(DdField::Priority, cx));
        let mutual = view.update(cx, |v, _| v.dd_open == Some(DdField::Priority));
        assert!(mutual, "开优先级应互斥收状态");
        view.update(cx, |v, cx| v.dd_pick(DdField::Priority, 3, cx));
        let (pri, closed) = view.update(cx, |v, _| (v.priority, v.dd_open.is_none()));
        assert_eq!(pri, 3, "选项落值（高=3）");
        assert!(closed, "单选即收");
        // 回切状态：选已取消 4。
        view.update(cx, |v, cx| v.toggle_dd(DdField::Status, cx));
        view.update(cx, |v, cx| v.dd_pick(DdField::Status, 4, cx));
        let (st, closed2) = view.update(cx, |v, _| (v.status, v.dd_open.is_none()));
        assert_eq!((st, closed2), (4, true), "状态切换与收层");
        // 颜色标记互斥：橙 → 青。
        view.update(cx, |v, cx| v.pick_mark(1, cx));
        view.update(cx, |v, cx| v.pick_mark(4, cx));
        let mark = view.update(cx, |v, _| v.mark);
        assert_eq!(mark, 4, "颜色标记最后选中生效");
        // 任务时间桶互斥单选（DEC-177）：今天 0 → 选「本周」2。
        view.update(cx, |v, cx| v.pick(2, cx));
        let bucket = view.update(cx, |v, _| v.bucket);
        assert_eq!(bucket, 2, "任务时间最后选中生效");
    }

    // 表单卡与下拉真派发（DEC-170 · 真实鼠标路径）：①几何＝表单卡四边外边距
    // 均为 16px（tc-body/tc-form 取样面）＋任务包装容器占行余高
    //（填至卡内底，底差＝卡内边距 SP_SM）且任务区在容器内不开天窗；②点触发
    // 器开层 → 点选项落值即收；③开层后再点触发器＝out 消耗收层（collect 同型）。
    #[gpui::test]
    async fn task_create_dd_dispatch_and_task_zone_geometry(cx: &mut gpui::TestAppContext) {
        use std::ops::Deref as _;
        let window = cx.open_window(gpui::size(gpui::px(480.0), gpui::px(640.0)), |_w, cx| {
            crate::init(cx);
            TaskCreateView::new(cx)
        });
        let mut cx = gpui::VisualTestContext::from_window(*window.deref(), cx);
        cx.run_until_parked();
        // ① 表单卡四边外边距 16px（tc-body/tc-form 取样面）＋任务包装容器填至
        // 卡内底（底差＝卡内边距 SP_SM）且占行余高。
        let (body, form) = (
            cx.debug_bounds("tc-body").expect("tc-body 未录制"),
            cx.debug_bounds("tc-form").expect("tc-form 未录制"),
        );
        let body_left = f32::from(body.origin.x);
        let body_right = f32::from(body.origin.x + body.size.width);
        let body_top = f32::from(body.origin.y);
        let body_bottom = f32::from(body.origin.y + body.size.height);
        let form_left = f32::from(form.origin.x);
        let form_right = f32::from(form.origin.x + form.size.width);
        let form_top = f32::from(form.origin.y);
        let form_bottom = f32::from(form.origin.y + form.size.height);
        assert!(
            (form_left - body_left - 16.0).abs() < 0.5
                && (body_right - form_right - 16.0).abs() < 0.5
                && (body_bottom - form_bottom - 16.0).abs() < 0.5
                && form_top - body_top >= 16.0,
            "表单卡四边外边距应为 16px（左差 {} 右差 {} 底差 {} 顶差 {}）",
            form_left - body_left,
            body_right - form_right,
            body_bottom - form_bottom,
            form_top - body_top,
        );
        let wrap = cx
            .debug_bounds("tc-task-wrap")
            .expect("tc-task-wrap 未录制");
        let wrap_top = f32::from(wrap.origin.y);
        let wrap_bottom = f32::from(wrap.origin.y + wrap.size.height);
        let wrap_h = f32::from(wrap.size.height);
        assert!(
            (form_bottom - wrap_bottom - t::SP_SM).abs() < 0.5,
            "任务包装容器应填至表单卡内底（底差＝卡内边距 {}，实得 {}）",
            t::SP_SM,
            form_bottom - wrap_bottom
        );
        assert!(
            wrap_top > form_top && wrap_h > 80.0,
            "任务包装容器应占据表单卡剩余可用空间（实得 {}）",
            wrap_h
        );
        let zone = cx
            .debug_bounds("tc-task-zone")
            .expect("tc-task-zone 未录制");
        assert!(
            f32::from(zone.origin.y) >= wrap_top
                && f32::from(zone.origin.y + zone.size.height) <= wrap_bottom + 0.5,
            "任务区不得超出包装容器（zone={}, wrap={}）",
            zone,
            wrap
        );
        // ② 状态触发器真点击 → 层开 → 选项「已取消」真点击 → 落值即收。
        let trg = cx
            .debug_bounds("tc-dd-status-trg")
            .unwrap_or_else(|| panic!("tc-dd-status-trg 未录制"));
        cx.simulate_click(trg.center(), Default::default());
        cx.run_until_parked();
        let open = window
            .update(&mut cx, |v, _w, _cx| v.dd_open == Some(DdField::Status))
            .unwrap();
        assert!(open, "真点击触发器应开状态下拉");
        let opt = cx
            .debug_bounds("tc-dd-status-opt-4")
            .unwrap_or_else(|| panic!("状态选项 tc-dd-status-opt-4 未录制"));
        cx.simulate_click(opt.center(), Default::default());
        cx.run_until_parked();
        let (st, closed) = window
            .update(&mut cx, |v, _w, _cx| (v.status, v.dd_open.is_none()))
            .unwrap();
        assert_eq!(st, 4, "点击选项「已取消」应落值");
        assert!(closed, "选项点击后应收层");
        // ③ 优先级触发器开层后再点触发器自身：out 消耗收层（开态不再 toggle）。
        let trg2 = cx
            .debug_bounds("tc-dd-pri-trg")
            .unwrap_or_else(|| panic!("tc-dd-pri-trg 未录制"));
        cx.simulate_click(trg2.center(), Default::default());
        cx.run_until_parked();
        let pri_open = window
            .update(&mut cx, |v, _w, _cx| v.dd_open == Some(DdField::Priority))
            .unwrap();
        assert!(pri_open, "优先级下拉应开启");
        cx.simulate_click(trg2.center(), Default::default());
        cx.run_until_parked();
        let pri_closed = window
            .update(&mut cx, |v, _w, _cx| v.dd_open.is_none())
            .unwrap();
        assert!(pri_closed, "再点触发器应被 out 消耗收层（collect 同型）");
    }

    // 分组数据准备（DEC-185 · 方法级）：状态模式组序＝待办置首＋其余相对序；
    // 排期模式 BUCKETS 序（今天天然首位）；组内排序＝优先级高→无、同级创建序
    // （sort_group_rows）；空组归零过滤（数据准备阶段）仅非空组入序；
    // 行址＝created 数据下标。
    #[gpui::test]
    async fn board_groups_filter_and_order(cx: &mut gpui::TestAppContext) {
        let view = cx.new(|cx| {
            let mut v = BoardView::new(cx);
            // 下标即行址：0 待办·低 · 1 已完成·紧急 · 2 进行中·低 · 3 未排期桶·高 ·
            // 4 今天桶·中；行 3/4 未写 status ⇒ 默认 0（待办）——状态分组只看
            // status、与 bucket 无关。
            v.created = vec![
                TaskRow {
                    id: "TASK-A".into(),
                    status: 0,
                    priority: 1,
                    ..Default::default()
                },
                TaskRow {
                    id: "TASK-B".into(),
                    status: 3,
                    priority: 4,
                    ..Default::default()
                },
                TaskRow {
                    id: "TASK-C".into(),
                    status: 1,
                    priority: 1,
                    ..Default::default()
                },
                TaskRow {
                    id: "TASK-D".into(),
                    bucket: 4,
                    priority: 3,
                    ..Default::default()
                },
                TaskRow {
                    id: "TASK-E".into(),
                    bucket: 0,
                    priority: 2,
                    ..Default::default()
                },
            ];
            v
        });
        let (st, sk) = view.update(cx, |v, _| {
            let st = prepare_groups(&v.created, false)
                .into_iter()
                .map(|(name, rows)| {
                    (
                        name,
                        rows.into_iter().map(|(i, _)| i).collect::<Vec<usize>>(),
                    )
                })
                .collect::<Vec<(&'static str, Vec<usize>)>>();
            let sk = prepare_groups(&v.created, true)
                .into_iter()
                .map(|(name, rows)| {
                    (
                        name,
                        rows.into_iter().map(|(i, _)| i).collect::<Vec<usize>>(),
                    )
                })
                .collect::<Vec<(&'static str, Vec<usize>)>>();
            (st, sk)
        });
        assert_eq!(
            st,
            vec![
                // 组内优先级高→无：行 3 高 · 行 4 中 · 行 0 低（created 序同级在前）。
                ("待办", vec![3_usize, 4_usize, 0_usize]),
                ("进行中", vec![2_usize]),
                ("已完成", vec![1_usize]),
            ],
            "状态模式：待办置首＋组内优先级降序（同级创建序）＋空组过滤（已搁置/已取消 空 → 不渲染）"
        );
        assert_eq!(
            sk,
            vec![
                // 组内优先级高→无：行 1 紧急 · 行 4 中 · 行 0/2 同级低（创建序 0 在 2 前）。
                ("今天", vec![1_usize, 4_usize, 0_usize, 2_usize]),
                ("未排期", vec![3_usize])
            ],
            "排期模式：BUCKETS 序（今天首位）＋组内优先级降序（同级创建序）＋空桶过滤（行未写 bucket ⇒ 默认今天）"
        );
        // 空库 → 零组（列表级空态承载 board-empty）。
        let view2 = cx.new(BoardView::new);
        let zero = view2.update(cx, |v, _| {
            prepare_groups(&v.created, false).is_empty()
                && prepare_groups(&v.created, true).is_empty()
        });
        assert!(zero, "空库两模式均零组");
    }

    // 分组展开缺省与显式持久（DEC-185 · D-02/D-03/D-04 方法级）：缺省＝待办/今
    // 天展开、其余全折叠；grp_toggle 显式落值——覆盖缺省、数据更新后保持；再次
    // 切换回收起；缺省展开组（待办）亦可折叠为显式值。
    #[gpui::test]
    async fn board_group_expand_defaults_and_persistence(cx: &mut gpui::TestAppContext) {
        let view = cx.new(BoardView::new);
        let d = view.update(cx, |v, _| {
            [
                ("待办", v.grp_expanded("待办")),
                ("进行中", v.grp_expanded("进行中")),
                ("已搁置", v.grp_expanded("已搁置")),
                ("已完成", v.grp_expanded("已完成")),
                ("已取消", v.grp_expanded("已取消")),
                ("今天", v.grp_expanded("今天")),
                ("明天", v.grp_expanded("明天")),
                ("本周", v.grp_expanded("本周")),
                ("以后", v.grp_expanded("以后")),
                ("未排期", v.grp_expanded("未排期")),
            ]
        });
        assert_eq!(
            d,
            [
                ("待办", true),
                ("进行中", false),
                ("已搁置", false),
                ("已完成", false),
                ("已取消", false),
                ("今天", true),
                ("明天", false),
                ("本周", false),
                ("以后", false),
                ("未排期", false),
            ],
            "缺省：待办/今天展开、其余折叠（DEC-185 D-02/D-03）"
        );
        // 显式切换持久：已完成 展开 → 数据更新保持 → 再切换回收起。
        view.update(cx, |v, cx| v.grp_toggle("已完成", cx));
        let e1 = view.update(cx, |v, _| v.grp_expanded("已完成"));
        assert!(e1, "显式切换应覆盖缺省（已完成 → 展开）");
        view.update(cx, |v, cx| {
            v.ingest_created("数据更新扰动".into(), 0, 0, 0, cx)
        });
        let e2 = view.update(cx, |v, _| v.grp_expanded("已完成"));
        assert!(e2, "数据更新后显式态保持");
        view.update(cx, |v, cx| v.grp_toggle("已完成", cx));
        let e3 = view.update(cx, |v, _| v.grp_expanded("已完成"));
        assert!(!e3, "再次切换回收起且持久");
        // 缺省展开组（待办）折叠为显式值——不被缺省兑底回翻。
        view.update(cx, |v, cx| v.grp_toggle("待办", cx));
        let d1 = view.update(cx, |v, _| v.grp_expanded("待办"));
        assert!(!d1, "待办显式折叠覆盖缺省展开");
    }

    // 分组头真实点击切换（DEC-185 · D-05 · 真实鼠标路径）：组头整行
    //（含右侧箭头位）on_click → 展开态翻转、行显隐随之；头点击不误上抛详情
    //（与行单击分流）；排期模式同构（mode 切换后同验今天桶）。
    #[gpui::test]
    async fn board_group_header_click_toggles(cx: &mut gpui::TestAppContext) {
        use std::ops::Deref as _;
        let titles: std::rc::Rc<std::cell::RefCell<Vec<String>>> = Default::default();
        let window = cx.open_window(gpui::size(gpui::px(900.0), gpui::px(640.0)), |_w, cx| {
            crate::init(cx);
            let titles = titles.clone();
            let mut v = BoardView::new(cx);
            v.created = vec![
                TaskRow {
                    title: "待办甲".into(),
                    id: "TASK-1".into(),
                    ..Default::default()
                },
                TaskRow {
                    title: "进行中乙".into(),
                    id: "TASK-2".into(),
                    status: 1,
                    ..Default::default()
                },
            ];
            v.set_peek_sink(std::rc::Rc::new(
                move |item: &PeekItem, _: &mut gpui::App| {
                    titles.borrow_mut().push(item.title.clone());
                },
            ));
            v
        });
        let mut cx = gpui::VisualTestContext::from_window(*window.deref(), cx);
        cx.run_until_parked();
        // 缺省：待办（tc-grp-0）展开行 0 可见；进行中（tc-grp-1）折叠行 1 隐藏。
        assert!(
            cx.debug_bounds("task-row-0").is_some(),
            "待办缺省展开 → 行 0 可见"
        );
        assert!(
            cx.debug_bounds("task-row-1").is_none(),
            "进行中缺省折叠 → 行 1 隐藏"
        );
        // 点「进行中」组头 → 展开 → 行 1 现身；头点击不上抛详情。
        let h = cx.debug_bounds("tc-grp-1").expect("进行中组头未录制");
        cx.simulate_click(h.center(), Default::default());
        cx.run_until_parked();
        assert!(
            cx.debug_bounds("task-row-1").is_some(),
            "点击组头 → 进行中展开 → 行 1 现身"
        );
        let peek_n = window
            .update(&mut cx, |_v, _w, _cx| titles.borrow().len())
            .unwrap();
        assert_eq!(peek_n, 0, "组头点击不得误上抛详情");
        // 再点「进行中」组头 → 收起 → 行 1 隐藏。
        let h = cx.debug_bounds("tc-grp-1").expect("进行中组头复发（重绘）");
        cx.simulate_click(h.center(), Default::default());
        cx.run_until_parked();
        assert!(
            cx.debug_bounds("task-row-1").is_none(),
            "再点组头 → 进行中收起 → 行 1 隐藏"
        );
        // 再点「待办」组头（缺省展开组）→ 收起 → 行 0 隐藏；再点恢复。
        let h0 = cx.debug_bounds("tc-grp-0").expect("待办组头未录制");
        cx.simulate_click(h0.center(), Default::default());
        cx.run_until_parked();
        assert!(
            cx.debug_bounds("task-row-0").is_none(),
            "待办收起 → 行 0 隐藏"
        );
        // 点「进行中」组头头重开 → 行 1 现身（折叠态可逆 · 收尾态还原）。
        let h = cx.debug_bounds("tc-grp-1").expect("进行中组头复发录制");
        cx.simulate_click(h.center(), Default::default());
        cx.run_until_parked();
        assert!(
            cx.debug_bounds("task-row-1").is_some(),
            "进行中重开 → 行 1 再现（折叠可逆）"
        );
    }
}
