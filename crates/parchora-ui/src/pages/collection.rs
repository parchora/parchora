//! PG-B 集锦视图（UI-3/UI-5 · TASK-503 CollectionView / 聚合首页范式）。
//!
//! HTML 1:1（PG-B.html 三态舞台 + PG-B2/PG-B3 子页）：
//! - L0 浏览首页（筛选工具条 → 标签栏（条件）→ 文件夹优先+文件列表；回收站
//!   入口在工具条右端 · DEC-117）；
//! - L1 下钻列表：面包屑 → 计数/排序 → 子文件夹置顶（chevron-right）→ 笔记行；
//! - 多选：mark-sel 行 + 渐隐层 + 目标选择器 .picker + .batchbar（night 反白）；
//! - Sub::Trash = PG-B3 回收站（集锦内页面，返回钮回 L0）。
//! - 图标（DEC-66）：行首/收藏夹/空态全量 lucide（速记箱=square-pen ·
//!   回收站=trash-2 · 最近加入=clock · 子文件夹=chevron-right ·
//!   笔记=file-text · 收藏夹=star）。
//!
//! 演示宿主限定：长按/marquee/拖拽重排 defer（TASK-503 实装）；「选择」入口以
//! L1 计数行右端「选择」ibtn 承载演示。
//!
//! 单击 Peek 接线（peek.rs）：L1 笔记行与首页「最近」文件行单击 → 统一详情窗
//! 直开；文件夹行单击＝进入该夹下钻（不开详情浮层 · 行数据面随 TASK-503 接线）。
//!
//! 视觉与交互口径：FAB 三层投影减淡；夹行单击＝下钻导航（不再双动作）；夹 icon
//! 统一 lucide folder；筛选下拉点外收起 out 挂浮层 wrapper（矩形＝面板体+起浮
//! 间隙，间隙下按不误收）；Pin 选中色＝t::PRIMARY（0x191923 · DEC-117 落地值 · 2026-10-23 四色重构随令牌层）。
//!
//! 右键菜单接线（DEC-114）：L1 笔记行六项（HTML 1:1）＝选择（真实进多选）/
//! 复制为纯文本/复制为 Markdown（真实写板 · 复制不弹 Toast · peek 先例）/
//! 复制为富文本/在 Finder 中显示/标记为任务（Toast 接 defer 口径，真实随
//! TASK-502/504）；文件夹行菜单仅「在 Finder 中显示」；PG-B3 右键＝恢复/
//! 永久删除（后者真实开 DG-CONFIRM）。菜单本体与消耗语义同 collect.rs
//!（deferred absolute + on_mouse_down_out 收起）。
//!
//! 磁盘真实面注入（DEC-144 · DEC-183 唯一数据源）：浏览列表＝宿主 app 层
//! 扫描 files 根（`scan_disk_notes_at`：仅根面普通 .md，跳过点前缀临时文件/
//! 子目录/非 md；mtime 降序新建置顶），经 `PanelView::set_disk_notes` 转发注入
//! `CollectionView::set_disk_notes`（唯一刷新入口、全量替换幂等、不触浏览态）。
//! 触发点四处：面板唤出（open_panel_docked）/ 编辑窗落盘三路（editor_exit 漏斗、
//! handoff 热换冲刷、open_editor_new 冲刷）。行单击 → 真实正文统一详情窗直开
//!（disk_note_item）；无条目时空态引导（FAB 新建笔记）。
//!
//! 数据面现状（DEC-183 mock 清理 · TASK-503/507 数据面恢复点）：浏览列表初始
//! 为空，全部行来自磁盘真实面注入；视图承载与交互机器保留：L1 下钻/多选
//! 当前无入口（夹行数据源未接线），行数据源清零；L1 行复制为 defer Toast；
//! 标签筛选保留（磁盘条目未打标计数 0 · 标签模型随 TASK-503/504 并轨）。

use std::time::Duration;

use gpui::prelude::FluentBuilder as _;
use gpui::{
    deferred, div, linear_color_stop, linear_gradient, px, relative, rgb, ClickEvent, Context, Div,
    InteractiveElement as _, IntoElement as _, ParentElement as _, Render,
    StatefulInteractiveElement as _, Styled, Window,
};

use crate::pages::peek::{self, PeekItem, PeekKind};
use crate::{components as co, scale as s, tokens as t};

use super::trash;

/// 当前子页（三层模型 + 集锦内回收站附属页）。
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Sub {
    /// L0 聚合首页
    Home,
    /// L1 下钻列表
    List,
    /// L1 多选态 + 目标选择器 + 批量条
    Multi,
    /// PG-B3 回收站
    Trash,
}

/// 笔记列表行（视图承载类型 · 真实数据源随 TASK-503 接线；初始空面，行右键/
/// 下钻机器留存）。
#[derive(Clone)]
struct NoteRow {
    icon: co::Icon,
    title: &'static str,
    meta: Option<&'static str>,
    tag_color: Option<u32>,
    folder: bool,
}

impl Default for NoteRow {
    fn default() -> Self {
        Self {
            icon: co::Icon::FileText,
            title: "",
            meta: None,
            tag_color: None,
            folder: false,
        }
    }
}

/// 集锦 FAB「新建笔记」→ 宿主上抛通道（ShellEvent::QuickNote 经
/// PanelView 弱引用转发；同 peek::PreviewSink 上抛模式）。
pub type NoteSink = std::rc::Rc<dyn Fn(&mut gpui::App)>;

/// 集锦首页笔记行「删除→回收站」上抛通道（DEC-164 · NoteSink 同模式 · 携题名
/// 词干）：宿主 rename 移 `.trash/` 并回灌列表（ShellEvent::TrashNote）。
pub type TrashSink = std::rc::Rc<dyn Fn(&str, &mut gpui::App)>;

/// 回收站物理删除动作（DEC-195 · 仅回收站域删除链用）：单件永久删除 / 清空。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TrashAction {
    /// 单件永久删除（行右键「永久删除」→ DG-CONFIRM 确认后 · 题名词干）。
    Purge(String),
    /// 清空回收站（页首「清空…」→ DG-CONFIRM 确认后）。
    Empty,
}

/// 回收站页删除动作上抛通道（TrashSink 同模式 · PanelView 装配注入）：宿主
/// defer 出借域后物理删除并回灌回收站列表。
pub type TrashActionSink = std::rc::Rc<dyn Fn(&TrashAction, &mut gpui::App)>;

/// 集锦浏览筛选态（工具条下拉两档）。
#[derive(Clone, Copy, PartialEq, Eq)]
enum HomeFilter {
    /// 最近加入（默认 · 磁盘面 mtime 降序与扫描序同源 · DEC-183）
    Recent,
    /// 标签（配 tag_sel 单选过滤 · 磁盘条目未打标 · 标签模型随 TASK-503/504）
    Tag,
}

/// 磁盘真实笔记（DEC-144 · app 层扫描 files 根注入）：PG-B 浏览列表在 DB 接线
///（TASK-503）前的真实数据面——「创建集锦」面板落盘（editor_exit/save 漏斗）
/// 后由宿主扫描 files 根，经 PanelView 转发注入，全量替换幂等（重复注入不
/// 增殖）。created=相对时距标注（扫描方 rel_time_label 求解）；字段仅承载展示
/// 面，排序（mtime 倒序）与噪声过滤（点前缀临时文件/非 .md/子目录）由扫描方
/// 负责。
pub struct DiskNote {
    /// 笔记题名（=文件名词干，与 saved_name 口径一致）
    pub title: String,
    /// 笔记正文（.md 全文原样）
    pub body: String,
    /// 创建时距标注（如「刚刚」「3 天前」）
    pub created: String,
}

/// 磁盘真实笔记行 → Peek 条目（DEC-144）：title/正文直通真实内容，meta 标注
/// 「磁盘真实」来源锚与创建时距（kind=Note 同款演示行）。
fn disk_note_item(d: &DiskNote) -> PeekItem {
    PeekItem {
        kind: PeekKind::Note,
        title: d.title.clone(),
        meta: format!("磁盘真实 · {}", d.created),
        body: d.body.clone(),
        disk_real: true,
    }
}

/// 磁盘行可见性门控（DEC-144）：标签模式且已选具体标签 → 未打标磁盘条目隐藏
///（演示条目带 tag 下标参与过滤，磁盘条目暂无标签承载——TASK-503/504 并轨后
/// 随标签模型接真实）。
fn disk_rows_visible(filter: HomeFilter, tag_sel: Option<usize>) -> bool {
    !(filter == HomeFilter::Tag && tag_sel.is_some())
}

/// 浏览首页笔记行右键菜单目标（DEC-164 · DEC-183）：磁盘
/// 真实行＝宿主真移 `.trash/`（行身份＝题名词干原值，开启时捕捉——磁盘面重扫
/// 重排不影响菜单存活期语义）。
#[derive(Clone)]
enum HomeCtx {
    Disk(String),
}

/// 集锦视图状态机（Home / L1 / 多选 / B2 / B3）。
pub struct CollectionView {
    sub: Sub,
    /// 面包屑路径（L1 态；根锚点=聚合视角名）
    path: Vec<&'static str>,
    /// 多选勾选（对应 rows 下标 · rows 真实源随 TASK-503 接线 · DEC-183）
    picked: [bool; 6],
    picker_open: bool,
    /// DG-CONFIRM 挂起（trash 清空/永久删除二态）
    pub(crate) confirm_open: bool,
    /// 单击 Peek 上抛器（单击直开）
    emitter: peek::PeekEmitter,
    rows: Vec<NoteRow>,
    /// L1 行右键菜单态（DEC-114）：当前开菜单的行下标（trash 模块闭包写回）。
    pub(crate) ctx: Option<usize>,
    /// 右键菜单开启时的窗内点击坐标（DEC-116 点击位锚定；右键菜单态共享——各子页
    /// 至多一菜单同帧存活，开启必写覆盖，关不清理害）。
    pub(crate) ctx_pos: Option<gpui::Point<gpui::Pixels>>,
    /// 浏览筛选态：模式与标签单选（None=未选=全部）。
    home_filter: HomeFilter,
    tag_sel: Option<usize>,
    /// 筛选工具条下拉浮层开合（Esc 级联承载）。
    filter_dd: bool,
    /// 集锦 FAB「新建笔记」→ 宿主上抛通道（ShellEvent::QuickNote）。
    note_sink: Option<NoteSink>,
    /// 演示 Toast 文案（1400ms 自动消失 · UI-TOAST；点击可提前关）
    toast: Option<String>,
    /// Toast 自动消失防争代次（gen 失效防争 · 与 app peek_close_gen 同模式）
    toast_gen: u64,
    /// 未决 Toast timer（gpui::Task 持于字段；防争由 toast_gen 代次承载 · DEC-72）
    toast_task: Option<gpui::Task<()>>,
    /// 磁盘真实笔记（DEC-144 · 宿主扫描 files 根注入 · 全量替换幂等；
    /// TASK-503/504 DB 接线后并轨）
    disk_notes: Vec<DiskNote>,
    /// 回收站真实条目（DEC-195 · 宿主扫描 files 同级 `.trash/` 注入 · DiskNote
    /// 同型复用：title＝文件名词干 / body＝正文 / created＝删除时距）。
    pub(crate) trash_notes: Vec<DiskNote>,
    /// 首页笔记行「删除→回收站」上抛通道（DEC-164 · PanelView 装配注入）。
    trash_sink: Option<TrashSink>,
    /// 回收站页物理删除动作上抛通道（DEC-195 · PanelView 装配注入）。
    trash_action_sink: Option<TrashActionSink>,
    /// 回收站行右键菜单态（DEC-195 · 至多一个 · 存题名词干；ctx_pos 共享——
    /// DEC-116 口径同首页菜单）。
    pub(crate) trash_ctx: Option<String>,
    /// DG-CONFIRM 挂起动作（DEC-195 · 与 confirm_open 同开合）：Purge(题名) /
    /// Empty；None＝纯收层（取消/级联后无宿主动作）。open 必写（显式优于隐式）。
    pub(crate) confirm_pend: Option<TrashAction>,
    /// 浏览首页笔记行右键菜单态（DEC-164 · 至多一个；ctx_pos 共享——各子页至
    /// 多一菜单同帧存活 · DEC-116 口径）。
    home_ctx: Option<HomeCtx>,
}

impl CollectionView {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            sub: Sub::Home,
            path: vec![],
            picked: [false; 6],
            picker_open: false,
            confirm_open: false,
            emitter: peek::PeekEmitter::default(),
            rows: Vec::new(),
            ctx: None,
            ctx_pos: None,
            home_filter: HomeFilter::Recent,
            tag_sel: None,
            filter_dd: false,
            note_sink: None,
            toast: None,
            toast_gen: 0,
            toast_task: None,
            disk_notes: Vec::new(),
            trash_notes: Vec::new(),
            trash_sink: None,
            trash_action_sink: None,
            trash_ctx: None,
            confirm_pend: None,
            home_ctx: None,
        }
    }

    /// 单击 Peek 上抛通道注入（PanelView 装配统一转发 app 层）。
    pub fn set_peek_sink(&mut self, sink: peek::PreviewSink) {
        self.emitter.set_sink(sink);
    }

    /// FAB「新建笔记」上抛通道注入（PanelView 装配）。
    pub fn set_note_sink(&mut self, sink: NoteSink) {
        self.note_sink = Some(sink);
    }

    /// 首页笔记行删除上抛通道注入（PanelView 装配 · DEC-164）。
    pub fn set_trash_sink(&mut self, sink: TrashSink) {
        self.trash_sink = Some(sink);
    }

    /// 回收站页删除动作上抛通道注入（PanelView 装配 · DEC-195）。
    pub fn set_trash_action_sink(&mut self, sink: TrashActionSink) {
        self.trash_action_sink = Some(sink);
    }

    /// 回收站真实条目注入（DEC-195 · 唯一刷新入口）：宿主扫描 files 同级
    /// `.trash/` 后全量替换注入（镜像 set_disk_notes 口径：mtime 倒序由扫描方
    /// 保证；重复注入全量替换不增殖；不触浏览态）。
    pub fn set_trash_notes(&mut self, notes: Vec<DiskNote>, cx: &mut Context<Self>) {
        self.trash_notes = notes;
        cx.notify();
    }

    // 回收站页跨模块读点（DEC-195 · trash.rs 渲染域只用；字段私有纪律同 L1）：
    /// 行集只读视图。
    pub(crate) fn trash_rows_view(&self) -> &[DiskNote] {
        &self.trash_notes
    }

    /// 行右键菜单存活态（挂载门控）。
    pub(crate) fn trash_menu_open(&self) -> bool {
        self.trash_ctx.is_some()
    }

    /// 菜单锚点（点击位 · DEC-116；None＝兑底原点，开启必写坐标）。
    pub(crate) fn trash_menu_anchor(&self) -> gpui::Point<gpui::Pixels> {
        self.ctx_pos.unwrap_or_default()
    }

    /// DG-CONFIRM 挂起动作读点（dialogs.rs 动态文案）。
    pub(crate) fn confirm_context(&self) -> Option<&TrashAction> {
        self.confirm_pend.as_ref()
    }

    /// 挂起语境下的待删项数（清空确认文案「N 项」）。
    pub(crate) fn trash_count(&self) -> usize {
        self.trash_notes.len()
    }

    /// 磁盘真实笔记注入（DEC-144 · 唯一刷新入口）：app 层扫描 files 根后全量
    /// 替换注入——mtime 倒序序由扫描方保证，此处不重排。重复注入（同内容幂等）
    /// 全量替换不增殖；不触浏览态（sub/path/home_filter/tag_sel/ctx/toast 原样
    /// · 列表刷新不破坏既有交互状态）。
    pub fn set_disk_notes(&mut self, notes: Vec<DiskNote>, cx: &mut Context<Self>) {
        self.disk_notes = notes;
        cx.notify();
    }

    /// FAB 点击上抛（首页/下钻态同款；通道缺省=无操作，演示页自持）。
    fn emit_note(&self, cx: &mut Context<Self>) {
        if let Some(f) = self.note_sink.clone() {
            f(cx);
        }
    }

    /// 工具条下拉开合。
    fn toggle_filter_dd(&mut self, _e: &ClickEvent, _w: &mut Window, cx: &mut Context<Self>) {
        self.filter_dd = !self.filter_dd;
        cx.notify();
    }

    /// 下拉选项单选：即应用并收起；切回「最近加入」时标签单选清零（显式优于隐式）。
    fn home_filter_set(&mut self, f: HomeFilter, cx: &mut Context<Self>) {
        self.home_filter = f;
        self.filter_dd = false;
        if f == HomeFilter::Recent {
            self.tag_sel = None;
        }
        cx.notify();
    }

    /// 标签单选（再点同标签=取消回全部 · 显式优于隐式）。
    fn toggle_tag_sel(
        &mut self,
        i: usize,
        _e: &ClickEvent,
        _w: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.tag_sel = if self.tag_sel == Some(i) {
            None
        } else {
            Some(i)
        };
        cx.notify();
    }

    /// 回收站页开入（DEC-195 · 子页切换菜单不跨页存活）：首页菜单/回收站菜单
    /// 统一收层（浮层仅随属页挂载，但状态不残留——回页不重现陈旧浮层）。
    pub fn open_trash(&mut self, cx: &mut Context<Self>) {
        self.sub = Sub::Trash;
        self.home_ctx = None;
        self.trash_ctx = None;
        cx.notify();
    }

    /// 清空回收站 → DG-CONFIRM（UI-6 对话框层承载）：挂起动作＝Empty（DEC-195
    /// · 确认钮 close_confirm_ok 真派发）。
    pub fn open_confirm(&mut self, cx: &mut Context<Self>) {
        self.confirm_pend = Some(TrashAction::Empty);
        self.confirm_open = true;
        cx.notify();
    }

    /// DG-CONFIRM 确认（DEC-195 真落）：挂起动作上抛宿主 defer 出借域后物理删除
    ///（单件/清空）并回灌回收站列表；失败＝stderr 自证＋行不消（可重试，同
    /// trash_note 惯例）。Toast 随拍披露（行数取挂起时快照）。
    pub fn close_confirm_ok(&mut self, cx: &mut Context<Self>) {
        self.confirm_open = false;
        match self.confirm_pend.take() {
            Some(TrashAction::Purge(title)) => {
                if let Some(f) = self.trash_action_sink.clone() {
                    f(&TrashAction::Purge(title.clone()), cx);
                }
                self.show_toast(format!("已永久删除「{title}」"), cx);
            }
            Some(TrashAction::Empty) => {
                let n = self.trash_notes.len();
                if let Some(f) = self.trash_action_sink.clone() {
                    f(&TrashAction::Empty, cx);
                }
                self.show_toast(format!("回收站已清空（{n} 项）"), cx);
            }
            None => {}
        }
        cx.notify();
    }

    /// Esc 级联入口：DG-CONFIRM → 各右键菜单 → 筛选下拉 → 不消化（shell.rs ·
    /// DEC-73/114）。
    pub fn dismiss_overlay(&mut self, cx: &mut Context<Self>) -> bool {
        if self.confirm_open {
            self.confirm_open = false;
            cx.notify();
            true
        } else if self.ctx.take().is_some()
            || self.home_ctx.take().is_some()
            || self.trash_ctx.take().is_some()
        {
            // 菜单态同级；先到先消化（各菜单至多一个，sub 页间互斥不会同存）。
            cx.notify();
            true
        } else if self.filter_dd {
            self.filter_dd = false;
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

    fn go_l1(&mut self, path: Vec<&'static str>, cx: &mut Context<Self>) {
        self.sub = Sub::List;
        // 下钻承载保留（TASK-503 数据面恢复点）：夹行当前
        // 无真实数据源，下钻列表为空态；入口随真实文件夹模型接线复生。
        self.rows = Vec::new();
        self.path = path;
        self.ctx = None; // 菜单不跨页存活（DEC-114）
        self.home_ctx = None; // 首页笔记行菜单同口径（DEC-164）
        self.trash_ctx = None; // 回收站行菜单同口径（DEC-195）
        cx.notify();
    }

    pub(crate) fn go_home(&mut self, _e: &ClickEvent, _w: &mut Window, cx: &mut Context<Self>) {
        self.sub = Sub::Home;
        self.picker_open = false;
        self.filter_dd = false; // 筛选下拉不跨页存活（同 DEC-114 菜单不漂移口径）
        self.ctx = None; // 菜单不跨页存活（DEC-114）
        self.home_ctx = None; // 首页笔记行菜单同口径（DEC-164）
        self.trash_ctx = None; // 回收站行菜单同口径（DEC-195）
        cx.notify();
    }

    fn enter_multi(&mut self, _e: &ClickEvent, _w: &mut Window, cx: &mut Context<Self>) {
        self.sub = Sub::Multi;
        self.picker_open = false;
        self.ctx = None; // 菜单项「选择」真实进多选（DEC-114）：开菜单即收
        cx.notify();
    }

    fn exit_multi(&mut self, _e: &ClickEvent, _w: &mut Window, cx: &mut Context<Self>) {
        self.sub = Sub::List;
        self.picker_open = false;
        self.ctx = None;
        cx.notify();
    }

    fn toggle_pick(&mut self, i: usize, _e: &ClickEvent, _w: &mut Window, cx: &mut Context<Self>) {
        if let Some(p) = self.picked.get_mut(i) {
            *p = !*p;
        }
        cx.notify();
    }

    fn toggle_picker(&mut self, _e: &ClickEvent, _w: &mut Window, cx: &mut Context<Self>) {
        self.picker_open = !self.picker_open;
        cx.notify();
    }

    // ── 右键菜单（DEC-114）──

    /// L1 行右键菜单开合（至多一个 · DEC-116 记录点击窗内坐标）。
    fn ctx_open(&mut self, i: usize, pos: gpui::Point<gpui::Pixels>, cx: &mut Context<Self>) {
        self.ctx = Some(i);
        self.ctx_pos = Some(pos);
        cx.notify();
    }

    fn ctx_close(&mut self, cx: &mut Context<Self>) {
        self.ctx = None;
        cx.notify();
    }

    /// 菜单项 · 复制为纯文本（DEC-183 defer 口径：行数据源当前为空，真实
    /// 内容复制随 TASK-503 行模型接线复生；复制不触写板）。
    fn ctx_copy_plain(&mut self, _i: usize, cx: &mut Context<Self>) {
        self.ctx_close(cx);
        self.show_toast("复制为纯文本 · 行数据面随 TASK-503 接线".to_string(), cx);
    }

    /// 菜单项 · 复制为 Markdown（defer 口径同上；md 组装随 TASK-503 接线复生）。
    fn ctx_copy_md(&mut self, _i: usize, cx: &mut Context<Self>) {
        self.ctx_close(cx);
        self.show_toast("复制为 Markdown · 行数据面随 TASK-503 接线".to_string(), cx);
    }

    /// 菜单项 · defer Toast（复制富文本/Finder/标记为任务：转换与真链路随接线）。
    fn ctx_defer(&mut self, msg: &str, cx: &mut Context<Self>) {
        self.ctx_close(cx);
        self.show_toast(msg.to_string(), cx);
    }

    /// 菜单项 · 标记为任务（F4b 同口径；真实落库随 TASK-502 接线）。
    fn ctx_mark_task(&mut self, i: usize, cx: &mut Context<Self>) {
        self.ctx_close(cx);
        let t = self
            .rows
            .get(i)
            .map(|r| r.title.to_string())
            .unwrap_or_default();
        self.show_toast(format!("转为任务「{t}」· 落待办（TASK-502 后真实）"), cx);
    }

    /// L1 行右键菜单本体（DEC-114/116 · HTML PG-B 笔记行 1:1）：out 包 wrapper 同
    /// collect.rs 形态；笔记行六项 / 文件夹行仅「在 Finder 中显示」。锚定＝点击位
    /// （DEC-116：视图根挂載 · unwrap 兕底不可达——开启必写坐标）。
    fn ctx_layer(&self, i: usize, folder: bool, cx: &Context<Self>) -> gpui::AnyElement {
        let view = cx.weak_entity();
        let out = move |_: &gpui::MouseDownEvent, _: &mut gpui::Window, cx: &mut gpui::App| {
            let _ = view.update(cx, |v, cx| {
                if v.ctx.is_some() {
                    v.ctx = None;
                    cx.notify();
                }
            });
            cx.stop_propagation();
        };
        let mut menu = co::ctx_menu();
        if folder {
            menu = menu.child(self.finder_row(cx));
        } else {
            menu = menu
                .child(
                    co::ctx_row("选择", None, false)
                        .id("ctx-b-0")
                        .on_click(cx.listener(Self::enter_multi)) // 真实进多选
                        .hover(co::hov_warm)
                        .active(co::act_border),
                )
                .child(
                    co::ctx_row("复制为纯文本", None, false)
                        .id("ctx-b-1")
                        .on_click(cx.listener(
                            move |this,
                                  _e: &ClickEvent,
                                  _w: &mut Window,
                                  cx: &mut Context<Self>| {
                                this.ctx_copy_plain(i, cx)
                            },
                        ))
                        .hover(co::hov_warm)
                        .active(co::act_border),
                )
                .child(
                    co::ctx_row("复制为 Markdown", None, false)
                        .id("ctx-b-2")
                        .on_click(cx.listener(
                            move |this,
                                  _e: &ClickEvent,
                                  _w: &mut Window,
                                  cx: &mut Context<Self>| {
                                this.ctx_copy_md(i, cx)
                            },
                        ))
                        .hover(co::hov_warm)
                        .active(co::act_border),
                )
                .child(
                    co::ctx_row("复制为富文本", None, false)
                        .id("ctx-b-3")
                        .on_click(cx.listener(
                            |this, _e: &ClickEvent, _w: &mut Window, cx: &mut Context<Self>| {
                                this.ctx_defer("复制为富文本 · RTF 转换随 TASK-504 后真实", cx)
                            },
                        ))
                        .hover(co::hov_warm)
                        .active(co::act_border),
                )
                .child(self.finder_row(cx))
                .child(
                    co::ctx_row("标记为任务", None, false)
                        .id("ctx-b-5")
                        .on_click(cx.listener(
                            move |this,
                                  _e: &ClickEvent,
                                  _w: &mut Window,
                                  cx: &mut Context<Self>| {
                                this.ctx_mark_task(i, cx)
                            },
                        ))
                        .hover(co::hov_warm)
                        .active(co::act_border),
                );
        }
        co::ctx_anchor_at(
            self.ctx_pos.unwrap_or_default(),
            div().on_mouse_down_out(out).child(menu),
        )
    }

    /// 菜单公共行 · 在 Finder 中显示（Toast 接 defer 口径 · platform 接线后真实）。
    fn finder_row(&self, cx: &Context<Self>) -> gpui::Stateful<Div> {
        co::ctx_row("在 Finder 中显示", None, false)
            .id("ctx-b-fd")
            .on_click(cx.listener(
                |this, _e: &ClickEvent, _w: &mut Window, cx: &mut Context<Self>| {
                    this.ctx_defer("在 Finder 中显示 · platform 接线后真实", cx)
                },
            ))
            .hover(co::hov_warm)
            .active(co::act_border)
    }

    // ── 浏览首页笔记行右键菜单（DEC-164）──

    /// 首页笔记行右键菜单开合（至多一个 · DEC-116 记录点击窗内坐标）。
    fn home_ctx_open(
        &mut self,
        target: HomeCtx,
        pos: gpui::Point<gpui::Pixels>,
        cx: &mut Context<Self>,
    ) {
        self.home_ctx = Some(target);
        self.ctx_pos = Some(pos);
        cx.notify();
    }

    /// 菜单项 · 删除→回收站（DEC-164 · DEC-183）：磁盘真实行经
    /// 宿主真移 `.trash/`（失败 stderr 自证 · 列表不动＝可重试）。home_ctx 开启
    /// 时必 Some（out/级联/项点击三路收层先行）。
    fn home_ctx_delete(&mut self, cx: &mut Context<Self>) {
        if let Some(HomeCtx::Disk(title)) = self.home_ctx.take() {
            if let Some(f) = self.trash_sink.clone() {
                f(&title, cx);
            }
        }
        cx.notify();
    }

    /// 首页笔记行右键菜单本体（DEC-164）：单项「删除」（仅含删除一
    /// 项）；out 包 wrapper 同 ctx_layer 形态；锚定＝点击位（ctx_anchor_at 窗级
    /// anchored · 绘制于开启瞬间光标位不随滚动行漂移）。
    fn home_ctx_layer(&self, cx: &Context<Self>) -> gpui::AnyElement {
        let view = cx.weak_entity();
        let out = move |_: &gpui::MouseDownEvent, _: &mut gpui::Window, cx: &mut gpui::App| {
            let _ = view.update(cx, |v, cx| {
                if v.home_ctx.is_some() {
                    v.home_ctx = None;
                    cx.notify();
                }
            });
            cx.stop_propagation();
        };
        let menu = co::ctx_menu().child(
            co::ctx_row("删除", None, false)
                .id("ctx-h-del")
                // 点击穿透回归取样面（DEC-167 · home-disk/FAB 同式）：debug_bounds
                // 按「home-menu-del」定位菜单项注入点击；release no-op 不入生产
                // 样式（DEC-137 同式）。
                .debug_selector(|| "home-menu-del".to_string())
                .on_click(cx.listener(
                    |this, _e: &ClickEvent, _w: &mut Window, cx: &mut Context<Self>| {
                        this.home_ctx_delete(cx)
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

    // ── 回收站页（DEC-195 · 真数据实装 · 行点击/右键菜单/物理删除确认链）──

    /// 回收站行单击 → 预览（统一详情窗）：disk_real=false（回收条目非根面稿，
    /// 详情窗编辑保存路对非磁盘条目跳写 · 不会复活根面同名文件）。
    pub(crate) fn trash_row_preview(
        &self,
        item: &PeekItem,
        _e: &ClickEvent,
        _w: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.emitter.show(item, cx);
    }

    /// 回收站行右键菜单开合（DEC-195 · 至多一个 · ctx_pos 共享 DEC-116 口径）。
    pub(crate) fn trash_ctx_open(
        &mut self,
        title: String,
        pos: gpui::Point<gpui::Pixels>,
        cx: &mut Context<Self>,
    ) {
        self.trash_ctx = Some(title);
        self.ctx_pos = Some(pos);
        cx.notify();
    }

    /// 菜单项 · 恢复（在册 defer 口径维持 DEC-195）：Toast 披露，真链路随
    /// TASK-50x 接线复生（移回根面·同名顺延）。
    pub(crate) fn trash_ctx_restore(&mut self, cx: &mut Context<Self>) {
        if let Some(title) = self.trash_ctx.take() {
            self.show_toast(format!("恢复「{title}」· 链路随 TASK-50x 接线真实"), cx);
        }
        cx.notify();
    }

    /// 菜单项 · 永久删除 → DG-CONFIRM（Purge(题名) 挂起 · 在册口径 DEC-195）：
    /// 二次确认后 close_confirm_ok 真派发宿主物理删除。
    pub(crate) fn trash_ctx_purge(&mut self, cx: &mut Context<Self>) {
        if let Some(title) = self.trash_ctx.take() {
            self.confirm_pend = Some(TrashAction::Purge(title));
            self.confirm_open = true;
        }
        cx.notify();
    }

    // ── L0 浏览首页 ──

    /// 工具条下拉浮层锚（DEC-113 同款：deferred absolute · top=chip 下缘 · 左对齐）。
    /// 点外收起 out 挂浮层 wrapper（wrapper 矩形＝面板体 + 起浮
    /// 间隙 → 点面板内选项＝界内 capture 判定不触发 out，bubble 相 click 正常；
    /// 点面板外任意处＝capture 相收层 + stop 分派消耗本击，DEC-114 口径；与
    /// 右键菜单 ctx_layer wrapper 同形态）。面板体 mt(4) 在 wrapper 界内，间隙
    /// 下按不误收（同 ctx wrapper 口径）。
    fn filter_dd_anchor(&self, cx: &Context<Self>) -> gpui::AnyElement {
        const OPTIONS: [(HomeFilter, &str); 2] =
            [(HomeFilter::Recent, "最近加入"), (HomeFilter::Tag, "标签")];
        let view = cx.weak_entity();
        let out = move |_: &gpui::MouseDownEvent, _: &mut gpui::Window, cx: &mut gpui::App| {
            let _ = view.update(cx, |v, cx| {
                if v.filter_dd {
                    v.filter_dd = false;
                    cx.notify();
                }
            });
            cx.stop_propagation();
        };
        let mut rows = Vec::new();
        for (i, (f, label)) in OPTIONS.iter().enumerate() {
            let on = *f == self.home_filter;
            rows.push(
                co::row()
                    .justify_between()
                    .id(("flt-opt", i))
                    // 下拉选项穿透测试取样面（DEC-169 · D-04）：home-flt-opt-<i>。
                    .debug_selector(move || format!("home-flt-opt-{}", i))
                    .on_click(cx.listener(move |this, _e, _w, cx| this.home_filter_set(*f, cx)))
                    .hover(co::hov_warm)
                    .active(co::act_border)
                    .child(co::txt(
                        label.to_string(),
                        s::font(s::Lvl::Small),
                        t::PRIMARY,
                    ))
                    .child(if on {
                        co::txt("✓", s::font(s::Lvl::Small), t::PRIMARY)
                    } else {
                        div()
                    }),
            );
        }
        let panel = div()
            .mt(px(4.0))
            .w(px(160.0))
            .border_1()
            .border_color(rgb(t::BORDER))
            .rounded(px(t::RAD_MD))
            .bg(rgb(t::SURFACE))
            .overflow_hidden()
            .flex()
            .flex_col()
            .children(rows);
        let anchor = div()
            .absolute()
            .top(relative(1.0))
            .left(px(0.0))
            // 鼠标隔离面（DEC-169 · D-04 · FAB/ctx 右键同款）：选项叠在首页行上而
            // hitbox 无遮挡时，点击会同合身后夹行击（误下钻）——block 截断 hover
            // 栈，滚轮经 should_handle_scroll 照常透传、浮层子树在前方不受影响。
            .block_mouse_except_scroll()
            .on_mouse_down_out(out)
            .child(panel);
        deferred(anchor).into_any_element()
    }

    fn home_view(&self, cx: &mut Context<Self>) -> Div {
        // ① 筛选工具条：左端「最近加入/标签」chip 下拉（deferred 锚定 DEC-113
        // 同款）· 右端回收站（数量角标 · DEC-117）。下拉开启期整行包 out
        //（任意外点击关菜单 · 消耗语义同 DEC-114）。
        let dd_layer = self.filter_dd.then(|| self.filter_dd_anchor(cx));
        let filter_label = match self.home_filter {
            HomeFilter::Recent => "最近加入",
            HomeFilter::Tag => "标签",
        };
        // 点外收起由 filter_dd_anchor 内浮层 wrapper 的 out 承载（out 界域＝浮层
        // wrapper：若挂工具条行——浮层点击位在其矩形外，capture 相先行收层 + stop
        // 分派消耗本击，选项 click 永不触发；详见 vendor on_mouse_down_out 语义取证）。
        let toolbar = div()
            .px(px(16.0))
            .py(px(6.0))
            .flex()
            .items_center()
            .gap(px(6.0))
            .child(
                div()
                    .relative()
                    .child(
                        co::chip_dd(filter_label)
                            .id("flt-chip")
                            .debug_selector(|| "home-flt-chip".to_string())
                            .on_click(cx.listener(Self::toggle_filter_dd))
                            .active(co::act_border),
                    )
                    .when_some(dd_layer, |el, layer| el.child(layer)),
            )
            .child(div().flex_1())
            .child(
                div().relative().child(
                    co::icon_btn(co::Icon::Trash2)
                        .id("btn-trash-tb")
                        .on_click(cx.listener(|this, _e, _w, cx| {
                            this.filter_dd = false;
                            this.open_trash(cx)
                        }))
                        .active(co::act_border),
                ),
            );
        // ② 标签栏（标签模式下显示）：单选按钮，样式=既有 tagc 标签卡
        //（选中=warm 底+主墨描边；再点同标签=取消回全部）。计数＝0（DEC-183：
        // 磁盘条目未打标 · 标签模型随 TASK-503/504 并轨接真实）。
        let tag_counts = [0usize; 7];
        let tag_bar = (self.home_filter == HomeFilter::Tag).then(|| {
            let mut bar = div()
                .px(px(16.0))
                .pt(px(2.0))
                .pb(px(8.0))
                .flex()
                .flex_wrap()
                .gap(px(6.0));
            for (i, name) in t::TAG_NAMES.iter().enumerate() {
                let selected = self.tag_sel == Some(i);
                bar = bar.child(
                    div().w(px(80.0)).child(
                        co::tagc(name.to_string(), t::tag_color(i), tag_counts[i].to_string())
                            .id(("tag-btn", i))
                            .when(selected, |el| {
                                el.bg(rgb(t::WARM)).border_color(rgb(t::PRIMARY))
                            })
                            .on_click(cx.listener(
                                move |this,
                                      e: &ClickEvent,
                                      w: &mut Window,
                                      cx: &mut Context<Self>| {
                                    this.toggle_tag_sel(i, e, w, cx)
                                },
                            ))
                            .active(co::act_border),
                    ),
                );
            }
            bar
        });

        // ③ 列表（DEC-183 · 磁盘真实笔记行 · DEC-144）：
        // 标签模式已选具体标签 → 未打标磁盘条目隐藏（disk_rows_visible 门控）。
        let mut list = div().flex().flex_col();
        if disk_rows_visible(self.home_filter, self.tag_sel) {
            for (k, d) in self.disk_notes.iter().enumerate() {
                let item = disk_note_item(d);
                let title = d.title.clone();
                list = list.child(
                    co::row()
                        .id(("home-disk", k))
                        // FAB 穿透测试取样面（DEC-148）：debug_bounds 按「home-disk-<k>」定位注入压行；release no-op 不入生产样式（DEC-137 同式）。
                        .debug_selector(move || format!("home-disk-{}", k))
                        .on_click(cx.listener(move |this, _e, _w, cx| this.emitter.show(&item, cx)))
                        .on_aux_click(cx.listener(
                            move |this,
                                  ev: &ClickEvent,
                                  _w: &mut Window,
                                  cx: &mut Context<Self>| {
                                this.home_ctx_open(HomeCtx::Disk(title.clone()), ev.position(), cx)
                            },
                        ))
                        .hover(co::hov_warm)
                        .active(co::act_border)
                        .child(div().flex_none().child(co::icon(
                            co::Icon::FileText,
                            12.0,
                            t::MUTED,
                        )))
                        .child(co::t_label(d.title.clone()))
                        .child(co::meta(d.created.clone()))
                        .child(div().flex_none().child(co::tgo())),
                );
            }
        }
        // 空态（DEC-183）：Recent 无磁盘条目 / Tag 选定无打标条目双口径。
        if self.disk_notes.is_empty() || !disk_rows_visible(self.home_filter, self.tag_sel) {
            let msg = if self.home_filter == HomeFilter::Tag && self.tag_sel.is_some() {
                "该标签暂无内容，试试取消标签筛选"
            } else {
                "还没有笔记 · 点右下角 ＋ 新建"
            };
            list = list.child(co::empty_state(co::Icon::CircleDashed, msg));
        }
        div()
            .flex_1()
            .min_h_0()
            .flex()
            .flex_col()
            .child(toolbar)
            .when_some(tag_bar, |el, bar| el.child(bar))
            .child(
                div()
                    .id("b-home-scroll")
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
                            .id("fab-b")
                            .on_click(cx.listener(|this, _e, _w, cx| this.emit_note(cx)))
                            .active(co::act_border),
                    ),
            )
            .when(self.home_ctx.is_some(), |el| {
                // 首页笔记行右键菜单挂载（DEC-164 · ctx_anchor_at 窗级 anchored）：
                // 绘制于开启瞬间的光标位、不随滚动行漂移（DEC-116 同口径）。
                el.child(self.home_ctx_layer(cx))
            })
    }

    // ── L1 下钻列表 ──
    fn l1_view(&self, cx: &mut Context<Self>) -> Div {
        let parts: Vec<(String, bool)> = std::iter::once(("集锦首页".to_string(), false))
            .chain(
                self.path
                    .iter()
                    .enumerate()
                    .map(|(i, s)| ((*s).to_string(), i + 1 == self.path.len())),
            )
            .collect();
        let crumb = co::crumb(parts)
            .id("b-crumb")
            .on_click(cx.listener(Self::go_home))
            .active(co::act_border);

        let count_row = div()
            .flex()
            .items_center()
            .justify_between()
            .border_b_1()
            .border_color(rgb(t::BORDER_SOFT))
            .child(co::count_line(format!(
                "{} 篇 · 子文件夹 {} · 排序 名称",
                self.rows.iter().filter(|r| !r.folder).count(),
                self.rows.iter().filter(|r| r.folder).count()
            )))
            .child(
                co::ibtn("选择")
                    .id("btn-select")
                    .mr(px(16.0))
                    .on_click(cx.listener(Self::enter_multi))
                    .active(co::act_border),
            );

        let mut list = div().flex().flex_col();
        for (i, r) in self.rows.iter().enumerate() {
            let folder = r.folder;
            // 行单击（DEC-18x）：夹行＝下钻（重渲染子项）；笔记行 → defer Toast
            //（行数据面当前为空 · TASK-503 接线后统一详情窗直开复生）。行右键 →
            // 菜单（DEC-114）。
            let title = r.title; // &'static：提升出循环供 'static 监听器捕获
            let mut row = co::row()
                .id(("b-row", i))
                .on_click(cx.listener(move |this, _e, _w, cx| {
                    if folder {
                        let mut p = this.path.clone();
                        p.push(title);
                        this.go_l1(p, cx);
                    } else {
                        this.show_toast(format!("打开「{title}」· 行数据面随 TASK-503 接线"), cx);
                    }
                }))
                .on_aux_click(cx.listener(
                    move |this, e: &ClickEvent, _w: &mut Window, cx: &mut Context<Self>| {
                        this.ctx_open(i, e.position(), cx)
                    },
                ))
                .hover(co::hov_warm)
                .active(co::act_border)
                .child(div().flex_none().child(co::icon(r.icon, 12.0, t::MUTED)))
                .child(co::t_label(r.title));
            if let Some(c) = r.tag_color {
                row = row.child(div().w(px(9.0)).h(px(9.0)).rounded_full().bg(rgb(c)));
            }
            if let Some(m) = r.meta {
                row = row.child(co::meta(m));
            }
            if !folder {
                row = row.child(div().flex_none().child(co::tgo()));
            }
            list = list.child(row);
        }
        if self.rows.is_empty() {
            // 空夹（收藏夹 · L0 meta「长按文件夹加入」同源口径）：不虚构子项，示空态。
            list = list.child(co::empty_state(
                co::Icon::Folder,
                "文件夹为空 · 长按文件夹加入",
            ));
        }

        div()
            .flex_1()
            .min_h_0()
            .flex()
            .flex_col()
            .relative()
            .child(crumb)
            .child(count_row)
            .child(
                div()
                    .id("b-l1-scroll")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .child(list),
            )
            .child(
                div()
                    .absolute()
                    .right(px(t::FAB_DOCK))
                    .bottom(px(t::FAB_DOCK))
                    .child(
                        co::fab()
                            .id("fab-b1")
                            .on_click(cx.listener(|this, _e, _w, cx| this.emit_note(cx)))
                            .active(co::act_border),
                    ),
            )
            .when(self.ctx.is_some(), |el| {
                // 右键菜单视图根挂载（DEC-116 点击位锚定 · deferred anchored）：
                // 祖先链无滚动容器，绘制于开启瞬间的光标位、不随滚动行漂移。
                let folder = self.rows.get(self.ctx.unwrap()).is_some_and(|r| r.folder);
                el.child(self.ctx_layer(self.ctx.unwrap(), folder, cx))
            })
    }

    // ── 多选 + 目标选择器 + 批量条 ──
    fn multi_view(&self, cx: &mut Context<Self>) -> Div {
        let parts: Vec<(String, bool)> = std::iter::once(("集锦首页".to_string(), false))
            .chain(self.path.iter().map(|s| ((*s).to_string(), true)))
            .collect();
        let crumb = co::crumb(parts)
            .id("b-crumb-m")
            .on_click(cx.listener(Self::go_home))
            .active(co::act_border);

        let mut list = div().flex().flex_col();
        for (i, r) in self.rows.iter().enumerate() {
            let picked = self.picked.get(i).copied().unwrap_or(false);
            let row = if picked {
                co::sel_state(co::row())
            } else {
                co::row()
            };
            list = list.child(
                row.id(("m-row", i))
                    .on_click(cx.listener(
                        move |this, e: &ClickEvent, w: &mut Window, cx: &mut Context<Self>| {
                            this.toggle_pick(i, e, w, cx)
                        },
                    ))
                    .hover(co::hov_warm)
                    .active(co::act_border)
                    .child(co::mark_sel(picked))
                    .child(co::t_label(r.title))
                    .child(div().flex_1()),
            );
        }

        // 底部渐隐层（110px，表面色向下显影）
        let fade = div()
            .absolute()
            .left_0()
            .right_0()
            .bottom_0()
            .h(px(110.0))
            .bg(linear_gradient(
                180.0,
                linear_color_stop(co::h(t::SURFACE).alpha(0.0), 0.0),
                linear_color_stop(co::h(t::SURFACE), 1.0),
            ));

        // picker 浮层（移动到… 目标选择器）
        let picker = co::picker()
            .child(
                co::sheeth()
                    .child(co::txt_f(
                        "移动 2 项到…",
                        s::font(s::Lvl::Small),
                        t::PRIMARY,
                        t::FONT_BODY,
                        t::FONT_WEIGHT_REGULAR,
                    ))
                    .child(co::meta("最近目标：速记箱 · 调研摘录")),
            )
            .child(
                co::row()
                    .id("p-fav")
                    .hover(co::hov_warm)
                    .child(
                        div()
                            .flex_none()
                            .child(co::icon(co::Icon::Star, 13.0, t::PRIMARY)),
                    )
                    .child(co::t_label("收藏夹")),
            )
            .child(
                co::row()
                    .id("p-dir")
                    .hover(co::hov_warm)
                    .child(
                        div()
                            .w(px(7.0))
                            .h(px(7.0))
                            .rounded_full()
                            .bg(rgb(t::ACCENT)),
                    )
                    .child(co::t_label("设计笔记"))
                    .child(
                        div()
                            .flex_none()
                            .flex()
                            .items_center()
                            .gap(px(2.0))
                            .child(co::icon(co::Icon::ChevronRight, 10.0, t::MUTED))
                            .child(co::meta("下级")),
                    ),
            )
            .child(
                co::row()
                    .id("p-inbox")
                    .hover(co::hov_warm)
                    .child(
                        div()
                            .w(px(7.0))
                            .h(px(7.0))
                            .rounded_full()
                            .bg(rgb(t::TERTIARY)),
                    )
                    .child(co::t_label("速记箱")),
            )
            .child(
                co::row()
                    .id("p-research")
                    .hover(co::hov_warm)
                    .child(
                        div()
                            .w(px(7.0))
                            .h(px(7.0))
                            .rounded_full()
                            .bg(rgb(t::SECONDARY)),
                    )
                    .child(co::t_label("调研摘录")),
            )
            .child(
                div()
                    .flex()
                    .gap(px(10.0))
                    .mt(px(12.0))
                    .child(co::btn_pri("移动到所选").id("p-confirm").flex_1())
                    .child(co::btn_sec_icon(co::Icon::Plus, "新建文件夹").id("p-newdir")),
            );

        // batchbar（night-reverse-sheet）
        let batchbar = div()
            .id("batchbar")
            .absolute()
            .left_0()
            .right_0()
            .bottom_0()
            .flex()
            .items_center()
            .gap(px(14.0))
            .px(px(14.0))
            .py(px(10.0))
            .bg(rgb(t::NIGHT_BG))
            .child(
                div()
                    .flex_1()
                    .child(co::txt("已选 2 项", s::font(s::Lvl::Small), t::NIGHT_INK)),
            )
            .child(
                co::txt("移动到…", s::font(s::Lvl::Small), t::NIGHT_SAGE)
                    .id("bb-move")
                    .cursor_pointer()
                    .hover(co::hov_night_ink)
                    .on_click(cx.listener(Self::toggle_picker)),
            )
            .child(
                co::txt("打标签", s::font(s::Lvl::Small), t::NIGHT_SAGE)
                    .cursor_pointer()
                    .hover(co::hov_night_ink),
            )
            .child(
                co::txt("删除", s::font(s::Lvl::Small), t::NIGHT_SAGE)
                    .cursor_pointer()
                    .hover(co::hov_night_ink),
            );

        div()
            .flex_1()
            .min_h_0()
            .flex()
            .flex_col()
            .relative()
            .overflow_hidden()
            .child(crumb)
            .child(
                div()
                    .id("b-multi-scroll")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .pb(px(120.0))
                    .child(list),
            )
            .child(fade)
            .when(self.picker_open, |el| {
                el.child(
                    div()
                        .absolute()
                        .left(px(10.0))
                        .right(px(10.0))
                        .bottom(px(54.0))
                        .child(picker),
                )
            })
            .child(batchbar)
            .child(
                co::ibtn("完成")
                    .id("multi-done")
                    .absolute()
                    .left(px(14.0))
                    .bottom(px(54.0))
                    .on_click(cx.listener(Self::exit_multi))
                    .active(co::act_border),
            )
    }
}

impl Render for CollectionView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let base = match self.sub {
            Sub::Home => self.home_view(cx),
            Sub::List => self.l1_view(cx),
            Sub::Multi => self.multi_view(cx),
            Sub::Trash => trash::view(self, cx),
        };
        let base = if self.confirm_open {
            base.child(super::dialogs::confirm_sheet(self, cx))
        } else {
            base
        };
        base.when_some(self.toast.clone(), |el, msg| {
            el.child(
                div()
                    .absolute()
                    .bottom(px(24.0))
                    .left_0()
                    .right_0()
                    .flex()
                    .justify_center()
                    .child(co::toast(msg).id("toast-b").on_click(cx.listener(
                        |this, _e: &ClickEvent, _w: &mut Window, cx: &mut Context<Self>| {
                            this.toast = None;
                            cx.notify();
                        },
                    ))),
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::AppContext as _;

    // 笔记行单击 → 统一详情窗上抛（emitter 直达单测）。
    // FAB「新建笔记」上抛：通道注入后单击承载（emit_note 直达单测）。
    #[gpui::test]
    async fn fab_note_emits_via_sink(cx: &mut gpui::TestAppContext) {
        let fired: std::rc::Rc<std::cell::Cell<bool>> = Default::default();
        let view = cx.new(|cx| {
            let mut v = CollectionView::new(cx);
            let fired = fired.clone();
            v.set_note_sink(std::rc::Rc::new(move |_: &mut gpui::App| {
                fired.set(true);
            }));
            v
        });
        view.update(cx, |v, cx| v.emit_note(cx));
        assert!(fired.get());
    }

    // FAB 点击穿透（DEC-148 · 真实鼠标路径）：小窗 + 48 条磁盘压行保证列表溢出、
    // fab-b 下方被可点击行覆盖（行带 ~2100px，几何容差千像素级）。点击 FAB 圆心只
    // 上抛 QuickNote（notes=1），下方行不得误触发（peek 空、未下钻）；对照：FAB
    // 上缘 30px（遮挡面之外、压行带内）行点击照常上抛（防过修）。
    #[gpui::test]
    async fn fab_click_no_passthrough_to_rows(cx: &mut gpui::TestAppContext) {
        use std::ops::Deref as _;
        let notes: std::rc::Rc<std::cell::Cell<u32>> = Default::default();
        let titles: std::rc::Rc<std::cell::RefCell<Vec<String>>> = Default::default();
        let window = cx.open_window(gpui::size(gpui::px(900.0), gpui::px(560.0)), |_w, cx| {
            crate::init(cx);
            let notes = notes.clone();
            let titles = titles.clone();
            let mut v = CollectionView::new(cx);
            v.disk_notes = (0..48)
                .map(|i| DiskNote {
                    title: format!("穿透压行 {}", i),
                    body: String::new(),
                    created: "刚刚".to_string(),
                })
                .collect();
            v.set_note_sink(std::rc::Rc::new(move |_: &mut gpui::App| {
                notes.set(notes.get() + 1);
            }));
            v.set_peek_sink(std::rc::Rc::new(
                move |item: &PeekItem, _: &mut gpui::App| {
                    titles.borrow_mut().push(item.title.clone());
                },
            ));
            v
        });
        let mut cx = gpui::VisualTestContext::from_window(*window.deref(), cx);
        cx.run_until_parked();
        let fab = cx
            .debug_bounds("fab")
            .unwrap_or_else(|| panic!("fab 未录制（debug_selector 未生效）"));
        // 前置自校验：注入压行带确有行压在 FAB 区域之下（几何失效即拒测，防空转）。
        let under = (0..48).any(|k| {
            cx.debug_bounds(Box::leak(format!("home-disk-{k}").into_boxed_str()))
                .is_some_and(|r| r.intersects(&fab))
        });
        assert!(under, "注入压行应有行覆盖 fab-b 区域（几何前置失效）");
        // 对照组先打：遮挡面之外（完整位于 FAB 上方）的行真实点击 → 详情上抛照常。
        let ctrl = (0..48)
            .find_map(|k| {
                cx.debug_bounds(Box::leak(format!("home-disk-{k}").into_boxed_str()))
                    .filter(|r| r.origin.y + r.size.height < fab.origin.y - gpui::px(8.0))
                    .map(|r| r.center())
            })
            .unwrap_or_else(|| panic!("应有完整位于 FAB 上方的压行（对照组前置失效）"));
        cx.simulate_click(ctrl, Default::default());
        assert_eq!(
            titles.borrow().len(),
            1,
            "遮挡面之外的行点击应照常上抛（对照组·实得 {:?}）",
            titles.borrow()
        );
        // FAB 圆心：只开新建笔记（notes 恰 +1）；穿透若在，压行会再上抛一条
        // （titles 增殖）且 path 可能被夹行下钻改写——断言双向不漂移。
        cx.simulate_click(fab.center(), Default::default());
        assert_eq!(notes.get(), 1, "FAB 单击应恰好上抛一次 QuickNote");
        assert_eq!(
            titles.borrow().len(),
            1,
            "FAB 下方压行不得被穿透误触发（实得 {:?}）",
            titles.borrow()
        );
        let path = window.update(&mut cx, |v, _w, _cx| v.path.clone()).unwrap();
        assert_eq!(path, Vec::<String>::new(), "路径不得漂移（初始空）");
    }

    // 右键菜单（DEC-114）：Esc 级联首位消化（confirm 优先于菜单）。
    #[gpui::test]
    async fn collection_dismiss_cascade(cx: &mut gpui::TestAppContext) {
        let view = cx.new(|cx| {
            let mut v = CollectionView::new(cx);
            v.ctx_open(0, gpui::point(gpui::px(20.0), gpui::px(30.0)), cx);
            v
        });
        let seq = view.update(cx, |v, cx| {
            let menu = v.dismiss_overlay(cx); // 菜单 → 消化
            let plain = v.dismiss_overlay(cx); // 无浮层 → 不消化
            (menu, plain)
        });
        assert_eq!(seq, (true, false));
        let seq2 = view.update(cx, |v, cx| {
            v.confirm_open = true;
            let confirm = v.dismiss_overlay(cx); // DG-CONFIRM 优先
            (confirm, v.confirm_open)
        });
        assert_eq!(seq2, (true, false));
    }

    // 磁盘真实面注入（DEC-144）：全量替换幂等 —— 重复注入不增殖、后注覆盖前注，
    // 且不触浏览态（sub/筛选/下拉/标签单选原样）。
    #[gpui::test]
    async fn disk_notes_resync_replaces_without_duplication(cx: &mut gpui::TestAppContext) {
        let view = cx.new(CollectionView::new);
        view.update(cx, |v, cx| {
            v.set_disk_notes(
                vec![
                    DiskNote {
                        title: "甲".into(),
                        body: "一".into(),
                        created: "刚刚".into(),
                    },
                    DiskNote {
                        title: "乙".into(),
                        body: "二".into(),
                        created: "1 天前".into(),
                    },
                ],
                cx,
            );
            assert_eq!(v.disk_notes.len(), 2);
            v.set_disk_notes(
                vec![DiskNote {
                    title: "丙".into(),
                    body: "三".into(),
                    created: "刚刚".into(),
                }],
                cx,
            );
            assert_eq!(v.disk_notes.len(), 1, "重复注入=全量替换不增殖");
            assert_eq!(v.disk_notes[0].title, "丙");
            // 浏览态不破坏：子视图/筛选/下拉/标签单选保持基线（matches! 直判
            // · Sub/HomeFilter 演示枚举不引 Debug 派生 面）。
            assert!(matches!(v.sub, Sub::Home));
            assert!(matches!(v.home_filter, HomeFilter::Recent));
            assert!(v.tag_sel.is_none());
            assert!(!v.filter_dd);
        });
    }

    // 磁盘行 → Peek 条目映射（DEC-144）：真实正文直通（title/created/meta/body）。
    #[test]
    fn disk_note_item_maps_real_content() {
        let d = DiskNote {
            title: "会议记录".into(),
            body: "## 议程\n- 决议甲".into(),
            created: "3 天前".into(),
        };
        let item = disk_note_item(&d);
        assert!(matches!(item.kind, PeekKind::Note));
        assert_eq!(item.title, "会议记录");
        assert_eq!(item.body, "## 议程\n- 决议甲");
        assert!(item.meta.contains("3 天前") && item.meta.contains("磁盘真实"));
    }

    // 磁盘行可见性门控（DEC-144）：标签模式已选具体标签 → 未打标磁盘条目隐藏。
    #[test]
    fn disk_rows_visible_tag_filter_hides_untagged() {
        assert!(disk_rows_visible(HomeFilter::Recent, None));
        assert!(disk_rows_visible(HomeFilter::Tag, None)); // 标签模式未选具体标签=全部
        assert!(!disk_rows_visible(HomeFilter::Tag, Some(0)));
        assert!(!disk_rows_visible(HomeFilter::Tag, Some(3)));
    }

    // 首页笔记行右键菜单（DEC-164）：磁盘真实行「删除」→ 宿主上抛题名 + 菜单收
    // 起（无 Toast——成功面＝行随回灌消失）。
    #[gpui::test]
    async fn home_ctx_disk_delete_emits_trash_sink(cx: &mut gpui::TestAppContext) {
        let trashed: std::rc::Rc<std::cell::RefCell<Vec<String>>> = Default::default();
        let view = cx.new(|cx| {
            let mut v = CollectionView::new(cx);
            let trashed = trashed.clone();
            v.set_trash_sink(std::rc::Rc::new(move |title: &str, _: &mut gpui::App| {
                trashed.borrow_mut().push(title.to_string());
            }));
            v
        });
        view.update(cx, |v, cx| {
            v.home_ctx_open(
                HomeCtx::Disk("会议记录".into()),
                gpui::point(gpui::px(30.0), gpui::px(40.0)),
                cx,
            );
            assert!(v.home_ctx.is_some(), "开菜单即置态");
            v.home_ctx_delete(cx);
        });
        assert_eq!(*trashed.borrow(), vec!["会议记录".to_string()]);
        let state = view.update(cx, |v, _cx| (v.home_ctx.is_none(), v.toast.is_none()));
        assert!(state.0 && state.1, "菜单收起且磁盘行不弹 Toast");
    }

    // 首页笔记行右键菜单（DEC-164）：Esc 级联首位消化 + 菜单不跨页存活（下钻
    // 即收 · DEC-114 同口径）。
    #[gpui::test]
    async fn home_ctx_menu_dismisses_via_cascade_and_nav(cx: &mut gpui::TestAppContext) {
        let view = cx.new(|cx| {
            let mut v = CollectionView::new(cx);
            v.home_ctx_open(
                HomeCtx::Disk("演示目标甲".into()),
                gpui::point(gpui::px(10.0), gpui::px(20.0)),
                cx,
            );
            v
        });
        let seq = view.update(cx, |v, cx| {
            let menu = v.dismiss_overlay(cx); // 菜单 → 消化
            let plain = v.dismiss_overlay(cx); // 无浮层 → 不消化
            (menu, plain)
        });
        assert_eq!(seq, (true, false));
        let gone = view.update(cx, |v, cx| {
            v.home_ctx_open(
                HomeCtx::Disk("甲".into()),
                gpui::point(gpui::px(1.0), gpui::px(2.0)),
                cx,
            );
            assert!(v.home_ctx.is_some());
            v.go_l1(vec!["设计笔记"], cx);
            v.home_ctx.is_some()
        });
        assert!(!gone, "下钻后首页菜单不存活");
    }

    // 首页笔记行右键菜单点击穿透回归（DEC-167 · 真实鼠标路径 · FAB DEC-148 同式）：
    // 真实右键开菜单（aux 合成），菜单叠在磁盘行带上；对照组先打（菜单未开）行
    // 点击 peek 上抛照常；随后左点「删除」项（点击位同时压在身后磁盘行上）——
    // ctx_anchor_at 统一 block_mouse_except_scroll 应使菜单项恰好上抛回收站一次、
    // 身后行不合成 click（peek 零增殖）。几何前置自校验防空转。
    #[gpui::test]
    async fn home_ctx_menu_item_click_no_passthrough_to_rows(cx: &mut gpui::TestAppContext) {
        use std::ops::Deref as _;
        let titles: std::rc::Rc<std::cell::RefCell<Vec<String>>> = Default::default();
        let trashed: std::rc::Rc<std::cell::RefCell<Vec<String>>> = Default::default();
        let window = cx.open_window(gpui::size(gpui::px(900.0), gpui::px(560.0)), |_w, cx| {
            crate::init(cx);
            let titles = titles.clone();
            let trashed = trashed.clone();
            let mut v = CollectionView::new(cx);
            v.disk_notes = (0..48)
                .map(|i| DiskNote {
                    title: format!("穿透压行 {}", i),
                    body: String::new(),
                    created: "刚刚".to_string(),
                })
                .collect();
            v.set_peek_sink(std::rc::Rc::new(
                move |item: &PeekItem, _: &mut gpui::App| {
                    titles.borrow_mut().push(item.title.clone());
                },
            ));
            v.set_trash_sink(std::rc::Rc::new(move |title: &str, _: &mut gpui::App| {
                trashed.borrow_mut().push(title.to_string());
            }));
            v
        });
        let view = window.root(cx).unwrap();
        let mut cx = gpui::VisualTestContext::from_window(*window.deref(), cx);
        cx.run_until_parked();
        // 取一条全高落在窗口中带的行作锚（出锚下不翻边、行带不被窗缘裁切）。
        let anchor = (0..48)
            .find_map(|k| {
                cx.debug_bounds(Box::leak(format!("home-disk-{}", k).into_boxed_str()))
                    .filter(|r| {
                        r.origin.y > gpui::px(60.0) && r.origin.y + r.size.height < gpui::px(460.0)
                    })
                    .map(|r| (k, r))
            })
            .unwrap_or_else(|| panic!("应有全高落在窗口中带的磁盘行（几何前置失效）"));
        // 对照组：菜单未开，行真实左击 → peek 上抛照常（行为基线对照）。
        cx.simulate_click(anchor.1.center(), Default::default());
        assert_eq!(
            titles.borrow().len(),
            1,
            "菜单未开时行点击应照常上抛（对照组·实得 {:?}）",
            titles.borrow()
        );
        // 真实右键开菜单：锚定该行中心，出帧后取样「删除」项。
        view.update(&mut cx, |v, cx| {
            v.home_ctx_open(
                HomeCtx::Disk(format!("穿透压行 {}", anchor.0)),
                anchor.1.center(),
                cx,
            );
        });
        cx.run_until_parked();
        let del = cx
            .debug_bounds("home-menu-del")
            .unwrap_or_else(|| panic!("菜单删除项未录制（debug_selector 失效）"));
        // 几何前置自校验：删除项点击位必须同时有磁盘行压在下方（穿透面成立）。
        let under = (0..48).any(|k| {
            cx.debug_bounds(Box::leak(format!("home-disk-{}", k).into_boxed_str()))
                .is_some_and(|r| r.contains(&del.center()))
        });
        assert!(under, "删除项点击位应有磁盘行压在下方（几何前置失效）");
        // 菜单项真实左击：恰好上抛回收站一次；身后行不得被穿透误触发（peek 不增殖）。
        cx.simulate_click(del.center(), Default::default());
        assert_eq!(
            *trashed.borrow(),
            vec![format!("穿透压行 {}", anchor.0)],
            "删除项点击应恰好上抛一次回收站"
        );
        assert_eq!(
            titles.borrow().len(),
            1,
            "菜单项点击不得穿透误触发身后行（实得 {:?}）",
            titles.borrow()
        );
        assert!(
            view.update(&mut cx, |v, _cx| v.home_ctx.is_none()),
            "项选后菜单应收起"
        );
    }

    // 筛选下拉选项点击穿透回归（DEC-169 · D-04 · 集锦首页 · DEC-167 同式真派发）：
    // 下拉浮层叠在首页条目行（夹组置顶）之上，点选项切筛选——身后夹行不得同合
    // 触发下钻（go_l1 改写 path）。对照组：dd 未开时磁盘行点击 peek 照常（防过修）。
    #[gpui::test]
    async fn home_filter_dd_option_click_no_passthrough_to_rows(cx: &mut gpui::TestAppContext) {
        use std::ops::Deref as _;
        let titles: std::rc::Rc<std::cell::RefCell<Vec<String>>> = Default::default();
        let window = cx.open_window(gpui::size(gpui::px(900.0), gpui::px(560.0)), |_w, cx| {
            crate::init(cx);
            let titles = titles.clone();
            let mut v = CollectionView::new(cx);
            v.disk_notes = (0..48)
                .map(|i| DiskNote {
                    title: format!("穿透压行 {}", i),
                    body: String::new(),
                    created: "刚刚".to_string(),
                })
                .collect();
            v.set_peek_sink(std::rc::Rc::new(
                move |item: &PeekItem, _: &mut gpui::App| {
                    titles.borrow_mut().push(item.title.clone());
                },
            ));
            v
        });
        let mut cx = gpui::VisualTestContext::from_window(*window.deref(), cx);
        cx.run_until_parked();
        // path 基线：下拉选项下压行面＝磁盘行带，穿透面
        // 以「path 未改写」为牙面。
        let path0 = window.update(&mut cx, |v, _w, _cx| v.path.clone()).unwrap();
        // 对照组（防过修）：dd 未开，磁盘行真实左击 → peek 照常上抛一次。
        let ctrl = (0..48)
            .find_map(|k| {
                cx.debug_bounds(Box::leak(format!("home-disk-{}", k).into_boxed_str()))
                    .filter(|r| {
                        r.origin.y > gpui::px(60.0) && r.origin.y + r.size.height < gpui::px(460.0)
                    })
                    .map(|r| r.center())
            })
            .unwrap_or_else(|| panic!("应有全高落在窗口中带的磁盘行（几何前置失效）"));
        cx.simulate_click(ctrl, Default::default());
        assert_eq!(
            titles.borrow().len(),
            1,
            "对照组行点击应照常上抛（实得 {:?}）",
            titles.borrow()
        );
        // 真实左击筛选 chip 开层。
        let chip = cx
            .debug_bounds("home-flt-chip")
            .unwrap_or_else(|| panic!("筛选 chip 未录制（debug_selector 失效）"));
        cx.simulate_click(chip.center(), Default::default());
        cx.run_until_parked();
        let open = window.update(&mut cx, |v, _w, _cx| v.filter_dd).unwrap();
        assert!(open, "真实点击 chip 应开启筛选下拉");
        // 几何前置自校验：某选项点击位须恰好有首页可点行（磁盘行带）压在下方。
        let found = (0..2usize)
            .find_map(|o| {
                let ob =
                    cx.debug_bounds(Box::leak(format!("home-flt-opt-{}", o).into_boxed_str()))?;
                (0..48).find_map(|k| {
                    cx.debug_bounds(Box::leak(format!("home-disk-{}", k).into_boxed_str()))
                        .filter(|r| r.contains(&ob.center()))
                        .map(|_| ())
                })?;
                Some((o, ob))
            })
            .unwrap_or_else(|| panic!("应有筛选选项压在首页行上（几何前置失效）"));
        // 选项真实左击：穿透＝磁盘行同合误触发（peek 增殖 ⇒ 红；path 不得改写）。
        cx.simulate_click(found.1.center(), Default::default());
        cx.run_until_parked();
        let st = window
            .update(&mut cx, |v, _w, _cx| {
                (v.home_filter, v.filter_dd, v.path.clone())
            })
            .unwrap();
        assert!(
            st.2 == path0,
            "选项点击不得穿透夹行误下钻（path 被改写 ⤍ {:?}）",
            st.2
        );
        assert_eq!(
            titles.borrow().len(),
            1,
            "peek 计数不得变动（穿透面外无源）"
        );
        let expected = if found.0 == 0 {
            HomeFilter::Recent
        } else {
            HomeFilter::Tag
        };
        assert!(
            st.0 == expected,
            "选项点击应切换筛选态（第 {} 项）",
            found.0
        );
        assert!(!st.1, "选项选中后下拉应收起");
    }

    // 回收站真实条目注入（DEC-195）：全量替换幂等，不触浏览态（镜像磁盘面
    // 口径）。
    #[gpui::test]
    async fn trash_notes_resync_replaces_without_duplication(cx: &mut gpui::TestAppContext) {
        let view = cx.new(CollectionView::new);
        view.update(cx, |v, cx| {
            v.set_trash_notes(
                vec![DiskNote {
                    title: "甲".into(),
                    body: "一".into(),
                    created: "刚刚".into(),
                }],
                cx,
            );
            assert_eq!(v.trash_rows_view().len(), 1);
            v.set_trash_notes(vec![], cx);
            assert_eq!(v.trash_rows_view().len(), 0, "重复注入=全量替换不增殖");
        });
    }

    // 回收站清除链（DEC-195 · 永久删除）：行右键 → 永久删除 → DG-CONFIRM 开启
    //（Purge 挂起·菜单收）→ 确认 → 动作上抛一次；取取消路径不上抛。
    #[gpui::test]
    async fn trash_purge_confirm_emits_action(cx: &mut gpui::TestAppContext) {
        let acts: std::rc::Rc<std::cell::RefCell<Vec<TrashAction>>> = Default::default();
        let view = cx.new(|cx| {
            let mut v = CollectionView::new(cx);
            let acts = acts.clone();
            v.set_trash_action_sink(std::rc::Rc::new(
                move |act: &TrashAction, _: &mut gpui::App| {
                    acts.borrow_mut().push(act.clone());
                },
            ));
            v
        });
        view.update(cx, |v, cx| {
            v.trash_ctx_open(
                "会议记录".into(),
                gpui::point(gpui::px(20.0), gpui::px(30.0)),
                cx,
            );
            v.trash_ctx_purge(cx);
        });
        view.update(cx, |v, _cx| {
            assert!(v.confirm_open, "永久删除应 DG-CONFIRM 二次确认");
            assert!(v.trash_ctx.is_none(), "菜单点项即收");
        });
        // 取消：不上抛，层收，挂起清。
        view.update(cx, |v, cx| {
            v.confirm_open = false;
            v.confirm_pend = None; // 取消臂（dialogs 取消钮同式）
            cx.notify();
        });
        assert!(acts.borrow().is_empty(), "取消不上抛");
        // 确认：Purge 上抛一次。
        view.update(cx, |v, cx| {
            v.confirm_pend = Some(TrashAction::Purge("会议记录".into()));
            v.confirm_open = true;
            v.close_confirm_ok(cx);
        });
        assert_eq!(
            *acts.borrow(),
            vec![TrashAction::Purge("会议记录".into())],
            "确认后物理删除动作恰好上抛一次"
        );
        let (open, toast, pend) = view.update(cx, |v, _cx| {
            (v.confirm_open, v.toast.clone(), v.confirm_pend.is_none())
        });
        assert!(!open && pend, "确认后层收且挂起清零");
        assert_eq!(toast.as_deref(), Some("已永久删除「会议记录」"));
    }

    // 回收站清空链（DEC-195）：页首「清空…」→ Empty 挂起 → 确认 → 动作上抛+
    // Toast 带项数快照。
    #[gpui::test]
    async fn trash_empty_confirm_emits_action(cx: &mut gpui::TestAppContext) {
        let acts: std::rc::Rc<std::cell::RefCell<Vec<TrashAction>>> = Default::default();
        let view = cx.new(|cx| {
            let mut v = CollectionView::new(cx);
            let acts = acts.clone();
            v.set_trash_action_sink(std::rc::Rc::new(
                move |act: &TrashAction, _: &mut gpui::App| {
                    acts.borrow_mut().push(act.clone());
                },
            ));
            v.set_trash_notes(
                vec![DiskNote {
                    title: "甲".into(),
                    body: "一".into(),
                    created: "刚刚".into(),
                }],
                cx,
            );
            v
        });
        view.update(cx, |v, cx| {
            v.open_confirm(cx); // 页首「清空…」入口同款
            assert!(matches!(v.confirm_context(), Some(TrashAction::Empty)));
            v.close_confirm_ok(cx);
        });
        assert_eq!(*acts.borrow(), vec![TrashAction::Empty], "清空动作上抛");
        let toast = view.update(cx, |v, _cx| v.toast.clone());
        assert_eq!(toast.as_deref(), Some("回收站已清空（1 项）"), "项数快照");
    }

    // 回收站恢复行（DEC-195 · 在册 defer 口径）：菜单「恢复」→ Toast 披露+
    // 菜单收，不上抛动作。
    #[gpui::test]
    async fn trash_restore_is_deferred_toast(cx: &mut gpui::TestAppContext) {
        let acts: std::rc::Rc<std::cell::RefCell<Vec<TrashAction>>> = Default::default();
        let view = cx.new(|cx| {
            let mut v = CollectionView::new(cx);
            let acts = acts.clone();
            v.set_trash_action_sink(std::rc::Rc::new(
                move |act: &TrashAction, _: &mut gpui::App| {
                    acts.borrow_mut().push(act.clone());
                },
            ));
            v
        });
        view.update(cx, |v, cx| {
            v.trash_ctx_open(
                "会议记录".into(),
                gpui::point(gpui::px(20.0), gpui::px(30.0)),
                cx,
            );
            v.trash_ctx_restore(cx);
        });
        let (toast, ctx_closed) =
            view.update(cx, |v, _cx| (v.toast.clone(), v.trash_ctx.is_none()));
        assert!(ctx_closed, "菜单点项即收");
        assert!(
            toast.unwrap_or_default().starts_with("恢复「会议记录」"),
            "恢复 defer Toast 披露"
        );
        assert!(acts.borrow().is_empty(), "恢复 defer 不上抛动作");
    }
}
