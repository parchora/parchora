//! PG-A 拾记视图 + PG-A1 队列详情浮出层（UI-2 · TASK-502 CaptureView / F-CAP-1~6）。
//!
//! HTML 1:1（PG-A.html / PG-A1.html）：
//! - chips（类型/日期/密度）→ 计数行 → 条目行（无 hint/FAB · DEC-117 ⑫/⑥；无搜索框 · DEC-177）；
//! - A1 = 拾记内浮出列表：sec 头（队列详情·剩余 N + FIFO ⇄ LIFO）→ placed 行
//!   （growth 左条 + warm 底）→ 序号行 → 停止（pri）/ 清空（sec）→ hint。
//! - 图标（DEC-66)：行首类型 glyph → lucide（文本=type · 链接=link · 图片=image ·
//!   文件=file · Pin 徽标=pin）。
//! - 行单击 → Peek 直开（peek.rs；主列表 4 行
//!   承载，队列浮出层行不涉及）。
//! - 二下拉筛选（DEC-111 ②b 接线 · DEC-112）：
//!   chip_dd×2 → deferred 浮层（类型 8 类＋笔记/任务多选 · 日期四档）＋数据过滤，
//!   计数行「匹配 N 条」随生效条件更新（REQ-SEARCH-FILTERS / REQ-SEARCH-COUNT）。
//!
//! 剪贴板捕获接线（DEC-163 · DEC-183）：app 层 0.5s changeCount 轮询
//! 守护（REQ-CAPTURE-POLL）读得新文本（隐私标记拒收在 platform 桥面内 ·
//! REQ-CAPTURE-PRIVACY；自写板跳过经 `crate::clipstamp` ·
//! REQ-CAPTURE-SELF-SKIP/DEC-46；同内容 10s 判重在 app 守护面）→
//! `ingest_clip` 顶部插入（最新在前 · 捕获点墙钟随注入 · DEC-183）→
//! 行内时间显示经 `crate::walltime`（同日=时:分:秒 · 同年=月-日 时:分:秒 ·
//! 跨年=年-月-日 时:分:秒 · DEC-183）。

//! `sweep_clip` 按 app 层双阈值（保留时长天×1440/有效时长 · 分钟 · 配置档
//! clip_retention_days/clip_valid_min）清理：未处理超有效时长即清、非 Pin 超保留
//!
//!(Pin 豁免 · REQ-CAPTURE-LIMIT 精神）。「转为笔记/转为任务」即置 handled
//!
//!(未处理语义的真实承载 · 数据写库面随 TASK-502/300 实装）并上抛条目原文
//! → 宿主开「创建集锦」/「新建任务」面板预填（DEC-194）；「删除」即从
//! 数据源移除该条目并即时刷新（DEC-177 · 真实删除面随 TASK-300 实装）。
//!
//! 数据面（DEC-183 mock 清理）：列表初始为空，全部行来自真实捕获注入；无条
//! 目时显示空态文案。
//!
//! 演示宿主限定（偏差台账）：
//! - D-07 棋盘缩略块：无缩略数据时棋盘格纹占位回退（真图承载 DEC-188）。

use std::sync::Arc;
use std::time::{Duration, Instant};

use gpui::prelude::FluentBuilder as _;
use gpui::{
    deferred, div, img, px, relative, rgb, Animation, AnimationExt as _, ClickEvent, Context, Div,
    InteractiveElement as _, IntoElement as _, ObjectFit, ParentElement as _, Render,
    StatefulInteractiveElement as _, Styled, StyledImage as _, Window,
};

use crate::pages::peek::{self, PeekItem, PeekKind};
use crate::walltime::{self, Stamp};
use crate::{components as co, scale as s, tokens as t};

/// 拾记条目（捕获行由 app 守护经 `ingest_clip`/`ingest_row` 注入 · 列表初始为空
/// · DEC-183；TASK-300 回填种子经 `seed_rows` 承载）。
#[derive(Clone)]
pub struct ClipRow {
    pub icon: co::Icon,
    pub title: String,
    pub meta: Option<String>,
    pub thumb: bool,
    pub pinned: bool,
    pub selected: bool,
    /// 二下拉演示属性（DEC-112）：内容类型 / 捕捉日期档。
    pub kind: FilterKind,
    pub day: DayFilter,
    /// 加入时刻（DEC-163 捕获清理基准 · 捕获注入时刻；回填行按墙钟年龄回拨）。
    pub added_at: Instant,
    /// 捕获点墙钟快照（DEC-183 · 行时显示源；epoch=0 哨兵不显示）。
    pub wall: Stamp,
    /// 已处理（DEC-163 · 转为笔记/任务置位；未处理条目超有效时长即清）。
    pub handled: bool,
    /// DB 实体 id hex（TASK-300 持久化身份 · 右键删除的 DB 逆向动作承载）；
    /// None = 未持久化（内存暂态：测试直注/写失败待重试）。
    pub db_id: Option<String>,
    /// 已解码缩略图（SCN-CAPTURE-THUMB · DEC-188）：gpui 内部解码链承载的
    /// PNG 字节镜像（Arc 共享零拷贝克隆；None = 图面缺位回退棋盘占位）。
    pub thumb_img: Option<Arc<gpui::Image>>,
}

impl Default for ClipRow {
    fn default() -> Self {
        Self {
            icon: co::Icon::Type,
            title: String::new(),
            meta: None,
            thumb: false,
            pinned: false,
            selected: false,
            kind: FilterKind::PlainText,
            day: DayFilter::Today,
            added_at: Instant::now(),
            wall: Stamp::zero(),
            handled: false,
            db_id: None,
            thumb_img: None,
        }
    }
}

/// 类型筛选项：剪贴板 8 类对账 `parchora-core::ContentType`（REQ-CAPTURE-8TYPES ·
/// DB `clips.content_type` CHECK 值域）＋统一搜索扩展的笔记/任务（REQ-SEARCH-FILTERS）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FilterKind {
    PlainText,
    RichText,
    Html,
    Image,
    Url,
    FileUrl,
    Color,
    Unknown,
    Note,
    Task,
}

/// 队列按钮脉冲单半程时长（TASK-716 · design §11.2「~450ms × 2 次」——单段
/// 900ms 正弦两拍同型承载两半程，非 repeat；未撞首弹不包动画元素）。
const QUEUE_CHIP_PULSE_MS: u64 = 450;
/// 队列按钮脉冲升幅 px（chip 顶部带 ~6px 内边距余量——不触祖层裁剪）。
const QUEUE_CHIP_HOP_PX: f32 = 4.0;

/// 脉冲曲线（§11.2 y 位移脉冲）：单段 900ms 正弦两拍——t∈[0,1] → 两次起落
///（dock 弹跳同型：|sin(2πt)|）→ 0 落位终态，静止外观与基础态一致。
fn queue_chip_dy(t: f32) -> f32 {
    -QUEUE_CHIP_HOP_PX * (2.0 * std::f32::consts::PI * t).sin().abs()
}

/// 类型下拉序（8 类对齐 content.rs CONTENT_TYPES 序，笔记/任务殿后）。
const FILTER_KINDS: [FilterKind; 10] = [
    FilterKind::PlainText,
    FilterKind::RichText,
    FilterKind::Html,
    FilterKind::Image,
    FilterKind::Url,
    FilterKind::FileUrl,
    FilterKind::Color,
    FilterKind::Unknown,
    FilterKind::Note,
    FilterKind::Task,
];

impl FilterKind {
    /// 中文标注（PRD §4.1：L231 8 类英文枚举＋PG-A.html 行标题「文本/链接」＋
    /// L244「图片/文件/颜色/未知」；演示标注口径 · DEC-112）。
    fn label(self) -> &'static str {
        match self {
            Self::PlainText => "文本",
            Self::RichText => "富文本",
            Self::Html => "HTML",
            Self::Image => "图片",
            Self::Url => "链接",
            Self::FileUrl => "文件",
            Self::Color => "颜色",
            Self::Unknown => "未知",
            Self::Note => "笔记",
            Self::Task => "任务",
        }
    }
}

/// 日期筛选档位（All/Today/This Week/This Month 四档 · REQ-SEARCH-FILTERS/DES-ORG；
/// All=不过滤由 `Option::None` 表达）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DayFilter {
    Today,
    ThisWeek,
    ThisMonth,
}

impl DayFilter {
    fn label(self) -> &'static str {
        match self {
            Self::Today => "今天",
            Self::ThisWeek => "本周",
            Self::ThisMonth => "本月",
        }
    }
}

/// 当前打开的下拉（同一时刻至多一个 · DEC-112）。
#[derive(Clone, Copy, PartialEq, Eq)]
enum Dd {
    Kind,
    Day,
}

/// 行删除上抛通道（TASK-300 · 弱引用转发式 · TrashSink 同构）：参数＝行 db_id
/// hex（唯一身份）；宿主落 DB 删除。未注入＝无操作（演示页自持）。
pub type ClipDeleteSink = std::rc::Rc<dyn Fn(&str, &mut gpui::App)>;

/// 转写上抛通道（DEC-194 · ClipDeleteSink 同构弱引用转发式）：参数＝条目原文
///（ClipRow.title 即剪贴板文本 · peek_item 同源）；宿主开「创建集锦」/「新建
/// 任务」面板预填。未注入＝无操作（演示页自持）。
pub type ClipConvertSink = std::rc::Rc<dyn Fn(&str, &mut gpui::App)>;

/// 队列动作上抛（DEC-207 · TrashAction 同构 ui 局部动作型）：入队快照文本、
/// FIFO⇄LIFO 切换、停止、清空——队列单一事实源在 app 层 QueueHost（
/// capture::ClipQueue），动作经宿主 mutator 后全量回推镜像
///（set_queue_rows·set_disk_notes 同构）。未注入＝无操作。
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum QueueAction {
    /// 入队（快照文本＝拾记行载荷文本携带面）。
    Enqueue(String),
    /// FIFO⇄LIFO 切换（SCN-QUEUE-ORDER-*）。
    ToggleOrder,
    /// 停止队列（终态：队列不再持有；placed 补写随 TASK-301 引擎）。
    Stop,
    /// 清空队列（无残留）。
    Clear,
}
pub type QueueActionSink = std::rc::Rc<dyn Fn(&QueueAction, &mut gpui::App)>;

/// 队列条目显示镜像（DEC-207 · app 回推 · 消费序已排齐）：`id`＝
/// capture::QueueEntry.id（单调递增 · 同内容重复入队各自独立——稳定键）；
/// `text`＝入队快照文本。首行由浮出层以 placed 形态呈现、余行序号呈现。
#[derive(Clone)]
pub struct QueueRow {
    pub id: u64,
    pub text: String,
}

/// 二下拉 AND 组合过滤（纯函数 · REQ-SEARCH-FILTERS / SCN-SEARCH-AND-OR-COMBINE）：
/// 类型多选（空集=不限）、日期四档（None=全部）：全空时短路返回原列表；日期为
/// 窗径累积（今天 ⊂ 本周 ⊂ 本月）；返回原始下标（行绑定/右键操作按源对位）。
fn filter_rows(rows: &[ClipRow], kinds: &[FilterKind], day: Option<DayFilter>) -> Vec<usize> {
    if kinds.is_empty() && day.is_none() {
        return (0..rows.len()).collect();
    }
    rows.iter()
        .enumerate()
        .filter(|(_, r)| (kinds.is_empty() || kinds.contains(&r.kind)) && day_matches(day, r.day))
        .map(|(i, _)| i)
        .collect()
}

/// 日期窗径累积匹配：今天只命中今天捕捉；本周命中今天/本周；本月命中全部演示档位。
fn day_matches(selected: Option<DayFilter>, captured: DayFilter) -> bool {
    match selected {
        None => true,
        Some(DayFilter::Today) => captured == DayFilter::Today,
        Some(DayFilter::ThisWeek) => captured != DayFilter::ThisMonth,
        Some(DayFilter::ThisMonth) => true,
    }
}

/// 行 → Peek 条目（单击承载 · DEC-183）：标题=条目原文、正文=条目原文
///（捕获行 title 即剪贴板文本）、meta=行时格式串（显示层同源）。
fn peek_item(r: &ClipRow) -> PeekItem {
    let mut meta = String::new();
    if let Some(m) = &r.meta {
        meta.push_str(m);
        meta.push_str(" · ");
    }
    if !r.wall.is_zero() {
        let now = walltime::local_civil(walltime::now_epoch());
        meta.push_str(&walltime::fmt_between(&r.wall, &now));
    }
    PeekItem {
        kind: PeekKind::Clip,
        title: r.title.clone(),
        meta,
        body: r.title.clone(),
        disk_real: false,
    }
}

/// 拾记视图（含队列浮层态与演示 Toast）。
pub struct CaptureView {
    queue_open: bool,
    density_compact: bool,
    rows: Vec<ClipRow>,
    /// 二下拉筛选态（DEC-112）：当前开着的下拉 / 类型多选（空=不限）/ 日期。
    dropdown: Option<Dd>,
    sel_kinds: Vec<FilterKind>,
    sel_day: Option<DayFilter>,
    /// 单击 Peek 上抛器（单击直开）
    emitter: peek::PeekEmitter,
    /// 右键条目菜单态（DEC-114）：当前开菜单的行下标（至多一个）。
    ctx: Option<usize>,
    /// 右键菜单行身份快照（R184 F-183-02 防竞态）：开启时刻行三元组
    ///（DB 身份＋加入时刻＋题名）。菜单存活期行顶插使捕获下标过期——点击项
    /// 按身份重解析当前下标；未记录（宿主/测试直调）回落捕获下标。
    ctx_ident: Option<(Option<String>, Instant, String)>,
    /// 右键菜单开启时的窗内点击坐标（DEC-116 点击位锚定；开启必写，关不清理害）。
    ctx_pos: Option<gpui::Point<gpui::Pixels>>,
    /// 演示 Toast 文案（1400ms 自动消失 · UI-TOAST；点击可提前关）
    toast: Option<String>,
    /// Toast 自动消失防争代次（gen 失效防争 · 与 app peek_close_gen 同模式）
    toast_gen: u64,
    /// 未决 Toast timer（gpui::Task 持于字段；防争由 toast_gen 代次承载 · DEC-72）
    toast_task: Option<gpui::Task<()>>,
    /// 行删除上抛（TASK-300 持久化身份 · 弱引用转发与 TrashSink 同式）：右键
    /// 「删除」即以 db_id 上抛（None 行不上抛——无 DB 身份无逆向动作），宿主
    /// 落 DB 删除；本视图行移除仍在本方法内承载（单源）。未注 = 无操作。
    on_row_delete: Option<ClipDeleteSink>,
    /// 转写上抛（DEC-194 · set_delete_sink 同链接线）：「转为笔记/转为任务」
    /// 即置 handled + 条目原文上抛，宿主开编辑窗/创建面板预填。未注 = 无操作。
    on_clip_to_note: Option<ClipConvertSink>,
    on_clip_to_task: Option<ClipConvertSink>,
    /// 队列条目显示镜像（DEC-207 · app 层 QueueHost 回推 · 消费序已排齐；
    /// 视图不自持队列真值——单一事实源在宿主）。空＝队列无条目（入口 chip
    /// 随之隐没）。
    queue: Vec<QueueRow>,
    /// 队列动作上抛（DEC-207 · set_delete_sink 同链接线）：入队/切换/停止/
    /// 清空经宿主 mutator 后镜像回推。未注 = 无操作。
    on_queue_action: Option<QueueActionSink>,
    /// 队列按钮 bounce 序（TASK-716 · §11.2 面板动效通道）：每轮提示注入自增
    /// ——动画元素 id 随序变化触发重挂重播（一轮脉冲 ~450ms×2 非 repeat；
    /// 未撞首弹（0）不包 anim 元素）。会话内存态。
    queue_bounce_seq: u64,
}

impl CaptureView {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            queue_open: false,
            density_compact: true,
            rows: Vec::new(),
            dropdown: None,
            sel_kinds: Vec::new(),
            sel_day: None,
            emitter: peek::PeekEmitter::default(),
            ctx: None,
            ctx_ident: None,
            ctx_pos: None,
            toast: None,
            toast_gen: 0,
            toast_task: None,
            on_row_delete: None,
            on_clip_to_note: None,
            on_clip_to_task: None,
            queue: Vec::new(),
            on_queue_action: None,
            queue_bounce_seq: 0,
        }
    }

    /// 队列按钮 bounce 触发（TASK-716 · PanelView::set_queue_bounce 转发点）：
    /// 序自增 + notify——chip 动画元素 id 随序变化重挂重播一轮脉冲。
    pub fn queue_chip_bounce(&mut self, cx: &mut Context<Self>) {
        self.queue_bounce_seq += 1;
        cx.notify();
    }

    /// 单击 Peek 上抛通道注入（PanelView 装配统一转发 app 层）。
    pub fn set_peek_sink(&mut self, sink: peek::PreviewSink) {
        self.emitter.set_sink(sink);
    }

    /// 剪贴板捕获注入（DEC-163 · app 守护转发 · REQ-CAPTURE-8TYPES 文本类）：
    /// 顶部插入（加入时间倒序 · 最新在前）；隐私/判重/自写三闸在
    /// app 守护面，此处无条件入列。`wall`＝捕获点墙钟 epoch 秒（DEC-183 ·
    /// 面板关闭期缓冲洗入时仍为捕获时刻 · 行时显示源）。
    pub fn ingest_clip(&mut self, text: String, at: Instant, wall: i64, cx: &mut Context<Self>) {
        self.rows.insert(
            0,
            ClipRow {
                icon: co::Icon::Type,
                title: text,
                kind: FilterKind::PlainText,
                day: DayFilter::Today,
                added_at: at,
                wall: walltime::local_civil(wall),
                ..Default::default()
            },
        );
        cx.notify();
    }

    /// 完整行注入（TASK-300 · 生产捕获/缓冲冲刷/回填复用同一位）：宿主组行
    ///（载荷映射 kind/icon/标题/db_id）→ 顶部插入 + notify（最新在前不变）。
    pub fn ingest_row(&mut self, row: ClipRow, cx: &mut Context<Self>) {
        self.rows.insert(0, row);
        cx.notify();
    }

    /// 回填种子（TASK-300 启动恢复 · 仅全新实体启用）：DB 读取行全集整表替换
    ///（顺序即 DB 读序 = 最新在顶）＋ notify；已存活实体（显隐复用）不得调用
    ///（整表替换会湮灭会话内存增量行）。
    pub fn seed_rows(&mut self, rows: Vec<ClipRow>, cx: &mut Context<Self>) {
        self.rows = rows;
        cx.notify();
    }

    /// 行删除通道注入（宿主装配位 · 弱引用转发式与 set_peek_sink 同链）。
    pub fn set_delete_sink(&mut self, sink: ClipDeleteSink) {
        self.on_row_delete = Some(sink);
    }

    /// 转为笔记通道注入（DEC-194 · PanelView 装配转发宿主；重复注入幂等）。
    pub fn set_clip_to_note_sink(&mut self, sink: ClipConvertSink) {
        self.on_clip_to_note = Some(sink);
    }

    /// 转为任务通道注入（DEC-194 · 同上）。
    pub fn set_clip_to_task_sink(&mut self, sink: ClipConvertSink) {
        self.on_clip_to_task = Some(sink);
    }

    /// 队列动作通道注入（DEC-207 · PanelView 装配转发宿主；重复注入幂等）。
    pub fn set_queue_action_sink(&mut self, sink: QueueActionSink) {
        self.on_queue_action = Some(sink);
    }

    /// 队列镜像注入（DEC-207 · app 层 QueueHost 回推 · set_disk_notes 同构：
    /// 全量替换幂等 · 唯一刷新口）：宿主在动作后/面板重开链统一回推
    ///（消费序已排齐 · 首项＝下一消耗项 placed）。
    pub fn set_queue_rows(&mut self, rows: Vec<QueueRow>, cx: &mut Context<Self>) {
        self.queue = rows;
        cx.notify();
    }

    /// 队列镜像披露（DEC-207 · clip_titles 同式仅读不写）：app 层回推验证
    /// 读点，列序即镜像呈现序（首项＝placed）。
    pub fn queue_texts(&self) -> Vec<String> {
        self.queue.iter().map(|r| r.text.clone()).collect()
    }

    /// 行「加入队列」（DEC-207 · 队列接线轮恢复入口 · design全集右键第四入）：
    /// 文本可粘贴类＝入队快照上拋（Toast「已加入队列 · 剩余 N」——N＝镜像存量
    /// +1 同拍可算）；image/unknown 行载荷不在文本面（占位题名非真实载荷）
    /// ——暂不入队，Toast 显示诚实差异面（承诺式引擎 TASK-301 承载二进制
    /// promise）。
    fn ctx_enqueue(&mut self, i: usize, cx: &mut Context<Self>) {
        let i = self.resolve_ctx().unwrap_or(i); // 防竞态（F-183-02，越界不再 panic）
        self.ctx_close(cx);
        let (kind, title) = match self.rows.get(i) {
            Some(r) => (r.kind, r.title.clone()),
            None => return,
        };
        if matches!(kind, FilterKind::Image | FilterKind::Unknown) {
            self.show_toast("图片/未知内容暂不入队".to_string(), cx);
            return;
        }
        if let Some(s) = &self.on_queue_action {
            s(&QueueAction::Enqueue(title), cx);
        }
        self.show_toast(format!("已加入队列 · 剩余 {}", self.queue.len() + 1), cx);
    }

    /// 拾记行题名披露（DEC-186 · 根修测试断言读点）：app 层跨关板存续验证
    /// 需在窗亡后读行序；仅读不写，列序即列表呈现序（最新在顶）。
    pub fn clip_titles(&self) -> Vec<String> {
        self.rows.iter().map(|r| r.title.clone()).collect()
    }

    /// 拾记行 DB 身份披露（DEC-188 · D-06 重试回填断言读点）：仅读不写。
    pub fn clip_db_ids(&self) -> Vec<Option<String>> {
        self.rows.iter().map(|r| r.db_id.clone()).collect()
    }

    /// 重试补写成功的 DB 身份回填（DEC-188 · D-06 收口）：按行加入时刻（捕获
    /// 时刻 Instant 同源克隆，Capture 逐行唯一）对位命中行置 db_id；命中行数
    /// 返回（app 层记账），有变更才 notify。
    pub fn update_db_ids(&mut self, hits: &[(Instant, String)], cx: &mut Context<Self>) -> usize {
        let mut changed = 0usize;
        for (at, id) in hits {
            for row in self.rows.iter_mut() {
                if row.added_at == *at && row.db_id.is_none() {
                    row.db_id = Some(id.clone());
                    changed += 1;
                }
            }
        }
        if changed > 0 {
            cx.notify();
        }
        changed
    }

    /// 拾记条目存活清理（DEC-163 谓词原文 · REQ-CAPTURE-LIMIT 精神）：清理谓词
    /// ＝(未处理 ∧ age≥有效时长) ∨ (非 Pin ∧ age≥存活时间)——Pin 豁免限存活窗
    /// 与计数上限两轴，未处理超「有效时长」Pin 不豁免（R184 F-183-14 文档对齐
    /// 谓词原文：旧注「Pin 豁免两窗」系漂移，实现未变）；两阈值分钟自 app 配置档；
    /// limit = 非 Pin 计数上限（DEC-188 档值·0=无限——超限从列表尾（最旧）裁
    /// 割非 Pin 行，Pin 恒豁免）。「转为笔记/任务」即置 handled（未处理语义的
    /// 唯一演示位）。返回移除数（app 层记账 · 有移除才 notify）。
    pub fn sweep_clip(
        &mut self,
        survive_min: u64,
        valid_min: u64,
        limit: u64,
        cx: &mut Context<Self>,
    ) -> usize {
        let valid = Duration::from_secs(valid_min * 60);
        let survive = Duration::from_secs(survive_min * 60);
        let now = Instant::now();
        let before = self.rows.len();
        self.rows.retain(|r| {
            let age = now.saturating_duration_since(r.added_at);
            let unhandled_expired = !r.handled && age >= valid;
            let survive_expired = !r.pinned && age >= survive;
            !(unhandled_expired || survive_expired)
        });
        // 计数上限裁剪（REQ-CAPTURE-LIMIT · limit>0 生效）：行序＝最新在顶，
        // 非 Pin 行按「最新→最旧」序保留前 limit 个，余下（最旧侧）逐位移除
        //（Pin 行恒豁免——下标定位删除，不做整段 truncate）。
        if limit > 0 {
            let unpinned_idx: Vec<usize> = self
                .rows
                .iter()
                .enumerate()
                .filter(|(_, r)| !r.pinned)
                .map(|(i, _)| i)
                .collect();
            if unpinned_idx.len() > limit as usize {
                let excess = unpinned_idx.len() - limit as usize;
                // 最旧侧 excess 个下标（升序倒出即降序移除，防位移）。
                for idx in unpinned_idx.iter().rev().take(excess).copied() {
                    self.rows.remove(idx);
                }
            }
        }
        let removed = before - self.rows.len();
        if removed > 0 {
            cx.notify();
        }
        removed
    }

    /// Esc 级联入口：先关右键菜单，再关三下拉浮层，再关 A1 队列层，均无则回报
    /// 不消化（shell.rs · DEC-73/112/114）。
    pub fn dismiss_overlay(&mut self, cx: &mut Context<Self>) -> bool {
        if self.ctx.is_some() {
            self.ctx = None;
            cx.notify();
            true
        } else if self.dropdown.is_some() {
            self.dropdown = None;
            cx.notify();
            true
        } else if self.queue_open {
            self.queue_open = false;
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

    pub fn open_queue(&mut self, cx: &mut Context<Self>) {
        self.dropdown = None; // 防浮层状态跨视图残留（DEC-112）
        self.queue_open = true;
        cx.notify();
    }

    fn toggle_density(&mut self, _e: &ClickEvent, _w: &mut Window, cx: &mut Context<Self>) {
        self.density_compact = !self.density_compact;
        cx.notify();
    }

    /// 三下拉开合（DEC-112）：再点同 chip 收起；点不同 chip 切换。
    /// （右键菜单开着时点击会被菜单 out 消耗并 stop 分派，本方法不可达 · DEC-114）
    fn toggle_dd(&mut self, d: Dd, cx: &mut Context<Self>) {
        self.dropdown = if self.dropdown == Some(d) {
            None
        } else {
            Some(d)
        };
        cx.notify();
    }

    /// 右键菜单开启（DEC-116 点击位锚定）：记录窗内点击坐标；开菜单即收三下拉
    /// （浮层互斥）。至多一个菜单存活——菜单面板 on_mouse_down_out 点外即收并 stop
    /// 本击，故到达本方法时菜单必然已关。
    fn ctx_open(&mut self, i: usize, pos: gpui::Point<gpui::Pixels>, cx: &mut Context<Self>) {
        self.dropdown = None;
        self.ctx = Some(i);
        // 行身份快照（R184 F-183-02）：菜单存活期顶插使捕获下标过期——点击项
        // 以（db_id, added_at, title）三元组重解析兑底。
        self.ctx_ident = self
            .rows
            .get(i)
            .map(|r| (r.db_id.clone(), r.added_at, r.title.clone()));
        self.ctx_pos = Some(pos);
        cx.notify();
    }

    /// 菜单行身份解析（R184 F-183-02）：命中快照 → 当前行下标；未记录（宿主
    /// /测试直调）或行面已无该身份（删除后菜单仍开）→ None（调用方回落捕获
    /// 下标）。added_at 为捕获唯一时刻，三元组命中即唯一行（DEC-163 行身份链）。
    fn resolve_ctx(&self) -> Option<usize> {
        let (db_id, added_at, title) = self.ctx_ident.as_ref()?;
        self.rows
            .iter()
            .position(|r| r.db_id == *db_id && r.added_at == *added_at && r.title == *title)
    }

    /// 右键菜单项执行后统一收起。
    fn ctx_close(&mut self, cx: &mut Context<Self>) {
        self.ctx = None;
        cx.notify();
    }

    /// 菜单项 · 转为笔记（F4a · DEC-194 真实承载）：即置 handled（DEC-163：
    /// 已处理＝未转化清单出清的演示承载）+ 条目原文经 sink 上抛——宿主开
    /// 「创建集锦」编辑窗预填正文，题名留待、后续落盘由编辑窗会话承载。反馈
    /// 载体＝新开面板（本页 toast 缺省）。
    fn ctx_to_note(&mut self, i: usize, cx: &mut Context<Self>) {
        let i = self.resolve_ctx().unwrap_or(i); // 防竞态（F-183-02）
        self.ctx_close(cx);
        let sink = self.on_clip_to_note.clone();
        if let Some(r) = self.rows.get_mut(i) {
            r.handled = true;
            if let Some(s) = &sink {
                s(&r.title, cx);
            }
        }
    }

    /// 菜单项 · 转为任务（F4b · DEC-194 真实承载）：handled 置位同 ctx_to_note
    ///（DEC-163）；条目原文经 sink 上抛——宿主开/激活「新建任务」面板预填
    ///「任务」多行域（题名不动），后续处理由用户定夺。
    fn ctx_to_task(&mut self, i: usize, cx: &mut Context<Self>) {
        let i = self.resolve_ctx().unwrap_or(i); // 防竞态（F-183-02）
        self.ctx_close(cx);
        let sink = self.on_clip_to_task.clone();
        if let Some(r) = self.rows.get_mut(i) {
            r.handled = true;
            if let Some(s) = &sink {
                s(&r.title, cx);
            }
        }
    }

    /// 菜单项 · 删除（DEC-177）：从数据源移除该条目并即时刷新（列表
    /// 每帧重渲即生效；i＝filter_rows 原始下标，过滤态同样按源对位）。pub 面向
    /// 菜单链与宿主侧持久化联动测试（TASK-300：删除上抛 db_id → 宿主 DB 删）。
    pub fn ctx_delete(&mut self, i: usize, cx: &mut Context<Self>) {
        let i = self.resolve_ctx().unwrap_or(i); // 防竞态（F-183-02）
        self.ctx_close(cx);
        if i < self.rows.len() {
            let row = self.rows.remove(i);
            let db_id = row.db_id.clone();
            let t = short_title(&row.title);
            if let (Some(id), Some(sink)) = (db_id.as_deref(), self.on_row_delete.clone()) {
                sink(id, cx);
            }
            self.show_toast(format!("已删除「{t}」"), cx);
        }
        cx.notify();
    }

    /// 类型多选切换：面板保持开放以便连续勾选（REQ-SEARCH-FILTERS 多选语义）。
    fn dd_kind_toggle(&mut self, k: FilterKind, cx: &mut Context<Self>) {
        if let Some(pos) = self.sel_kinds.iter().position(|&x| x == k) {
            self.sel_kinds.remove(pos);
        } else {
            self.sel_kinds.push(k);
        }
        cx.notify();
    }

    /// 日期单选：即应用并收起；「全部」选项承载清零（显式优于隐式）。
    fn dd_day_set(&mut self, d: Option<DayFilter>, cx: &mut Context<Self>) {
        self.sel_day = d;
        self.dropdown = None;
        cx.notify();
    }

    /// 停止队列（DEC-207）：动作上拋（宿主清 QueueHost→镜像回推）＋关层
    /// ＋Toast。placed 补写回板（SCN-QUEUE-STOP-FIXUP）随 TASK-301 承诺式
    /// 引擎接线——本面无 promise 持有板，无补写对象。
    fn close_queue(&mut self, _e: &ClickEvent, _w: &mut Window, cx: &mut Context<Self>) {
        if let Some(s) = &self.on_queue_action {
            s(&QueueAction::Stop, cx);
        }
        self.queue_open = false;
        self.show_toast("队列已停止".into(), cx);
    }

    /// 清空队列（DEC-207）：动作上拋（同上）＋关层＋Toast。
    fn clear_queue(&mut self, _e: &ClickEvent, _w: &mut Window, cx: &mut Context<Self>) {
        if let Some(s) = &self.on_queue_action {
            s(&QueueAction::Clear, cx);
        }
        self.queue_open = false;
        self.show_toast("队列已清空 · 不残留占位".into(), cx);
    }

    /// FIFO⇄LIFO 切换（DEC-207 · design §13.3 ②）：动作上拋——宿主翻转
    /// QueueHost 模式后镜像回推（视图不自持序模式）。
    fn queue_toggle_order(&mut self, _e: &ClickEvent, _w: &mut Window, cx: &mut Context<Self>) {
        if let Some(s) = &self.on_queue_action {
            s(&QueueAction::ToggleOrder, cx);
        }
    }

    /// 行点击统一入口：回贴演示（F1：写板随 capture crate 接线 · 既有）+ 单击直开
    /// 统一详情窗。
    fn row_activate(&mut self, i: usize, cx: &mut Context<Self>) {
        self.dropdown = None; // 点击行即收起下拉（DEC-112）
        self.ctx = None; // 菜单开时本击已被 out 消耗；防御复位（DEC-114）
        let (msg, item) = match self.rows.get(i) {
            Some(r) => (
                format!("已写回剪贴板 · {}", short_title(&r.title)),
                peek_item(r),
            ),
            None => return,
        };
        self.show_toast(msg, cx);
        self.emitter.show(&item, cx);
        cx.notify();
    }

    /// 单条目行：row（sel 三件套）+ 图标 + 预览（文本/meta/棋盘）+ tgo。
    /// 布局对照：文本行 meta 外挂（Pin 徽标=lucide pin）；链接/文件行 meta 内嵌；
    /// 图片行 t 内棋盘 + meta 外挂。右键 = 行锚定右键菜单（DEC-114）。
    /// 行时格式串（DEC-183）：哨兵不显示返 None；否则按日历日判三档格式。
    fn clip_time(r: &ClipRow, now: &Stamp) -> Option<String> {
        if r.wall.is_zero() {
            None
        } else {
            Some(walltime::fmt_between(&r.wall, now))
        }
    }

    fn clip_row(
        &self,
        i: usize,
        r: &ClipRow,
        now: &Stamp,
        cx: &Context<Self>,
    ) -> gpui::Stateful<Div> {
        let base = if r.selected {
            co::sel_state(co::row())
        } else {
            co::row()
        };
        let mut el = base
            .id(("clip-row", i))
            // 下拉选项穿透测试取样面（DEC-169 · D-04）：debug_bounds 按「clp-row-<i>」
            // 定位压行；release no-op 不入生产样式（home-disk-<k> 同式）。
            .debug_selector(move || format!("clp-row-{}", i))
            .on_click(cx.listener(
                move |this, _e: &ClickEvent, _w: &mut Window, cx: &mut Context<Self>| {
                    this.row_activate(i, cx)
                },
            ))
            .on_aux_click(cx.listener(
                move |this, e: &ClickEvent, _w: &mut Window, cx: &mut Context<Self>| {
                    this.ctx_open(i, e.position(), cx)
                },
            ))
            .hover(co::hov_warm)
            .active(co::act_border)
            .child(div().flex_none().child(co::icon(r.icon, 14.0, t::PRIMARY)));

        // 行时（DEC-183）：meta 槽位显示捕获时刻格式串；meta 既有
        // 值（真实来源注记 · 未来面）优先于行时拼接。
        let time_meta = Self::clip_time(r, now);
        if r.thumb {
            // 图片行：t 内棋盘块（flex_1 承载）+ 外挂 meta（行时拼接 · 徽标
            // 取样面 DEC-22x——长文字溢出回归的收纳判定承载）。
            let meta = match (&r.meta, &time_meta) {
                (Some(m), Some(t)) => format!("{m} · {t}"),
                (Some(m), None) => m.clone(),
                (None, Some(t)) => t.clone(),
                (None, None) => String::new(),
            };
            el = el
                .child(div().flex_1().child(self.thumb_block(&r.thumb_img)))
                .child(
                    co::meta(meta)
                        .id(("clip-row-badge", i))
                        .debug_selector(move || format!("clp-row-badge-{}", i)),
                );
        } else if r.kind == FilterKind::PlainText {
            // 文本行（捕获行 · DEC-183 真实面）：t 整行 + meta 外挂
            //（pinned = lucide pin 徽标；行时在非 Pin 槽位显示）。
            let badge = if r.pinned {
                div()
                    .flex_none()
                    .child(co::icon(co::Icon::Pin, 12.0, t::MUTED))
            } else {
                let meta_text = match (&r.meta, &time_meta) {
                    (Some(m), Some(t)) => format!("{m} · {t}"),
                    (Some(m), None) => m.clone(),
                    (None, Some(t)) => t.clone(),
                    (None, None) => String::new(),
                };
                co::meta(meta_text)
                    // 长文字溢出回归取样面（DEC-22x）：徽标收纳判定。
                    .debug_selector(move || format!("clp-row-badge-{}", i))
            };
            el = el
                .child(
                    co::t_label(r.title.clone())
                        // 三行截断（DEC-232）：文本 ≤3 行 · 行高 1.5em · 超出省略号。
                        .line_height(relative(t::ROW_TEXT_LH))
                        .line_clamp(t::ROW_CLAMP_COLLECT)
                        .text_ellipsis(),
                )
                .child(badge);
        } else {
            // 链接/文件行：t 内标题 + meta 子行（行时拼接）；min_w_0（DEC-229）：
            // 行 flex_1 项内长标题不撑破行（地板归零后子项文本换行）。
            let body = div().flex_1().min_w_0().flex().flex_col().child(
                co::content_text(r.title.clone(), s::font(s::Lvl::Tab), t::PRIMARY)
                    // 三行截断（DEC-232）：同文本行口径。
                    .line_height(relative(t::ROW_TEXT_LH))
                    .line_clamp(t::ROW_CLAMP_COLLECT)
                    .text_ellipsis(),
            );
            let body = match (&r.meta, &time_meta) {
                (Some(m), Some(t)) => body.child(co::meta(format!("{m} · {t}"))),
                (Some(m), None) => body.child(co::meta(m.clone())),
                (None, Some(t)) => body.child(co::meta(t.clone())),
                (None, None) => body,
            };
            el = el.child(body);
        }
        el.child(div().flex_none().child(co::tgo()))
    }

    /// 行右键菜单本体（DEC-114/116）：out 监听挂在外包 wrapper（含 4px 起浮间隙 ——
    /// 间隙下按不会误收菜单），点外收起 + stop 分派消耗本击（capture 先行于一切
    /// bubble/click 监听 —— 菜单开时点击行/chip 仅收菜单 · 头注口径）；菜单行点击 →
    /// 执行并收起。菜单行悬停/按下样式同 dd 行先例。锚定＝点击位（DEC-116：
    /// ctx_open 记录的窗内坐标；视图根挂載 · unwrap 兕底不可达——开启必写坐标）。
    fn ctx_layer(&self, i: usize, cx: &Context<Self>) -> gpui::AnyElement {
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
        let mut rows: Vec<gpui::Stateful<Div>> = Vec::new();
        // 四项（DEC-207 · design全集右键序「…加入队列，删除」对位）：转为笔记/
        // 转为任务置 handled（见 fns）；加入队列＝入队快照上拋（队列接线轮恢复
        // 入口）；删除＝从数据源移除该条目（原始下标对位）并即时刷新。
        let items: [&str; 4] = ["转为笔记", "转为任务", "加入队列", "删除"];
        for (k, label) in items.iter().enumerate() {
            rows.push(
                co::ctx_row(label.to_string(), None, false)
                    .id(("ctx-a", k))
                    .on_click(cx.listener(
                        move |this, _e: &ClickEvent, _w: &mut Window, cx: &mut Context<Self>| {
                            match k {
                                0 => this.ctx_to_note(i, cx),
                                1 => this.ctx_to_task(i, cx),
                                2 => this.ctx_enqueue(i, cx),
                                _ => this.ctx_delete(i, cx),
                            }
                        },
                    ))
                    .hover(co::hov_warm)
                    .active(co::act_border),
            );
        }
        let menu = rows.into_iter().fold(co::ctx_menu(), |m, r| m.child(r));
        co::ctx_anchor_at(
            self.ctx_pos.unwrap_or_default(),
            div().on_mouse_down_out(out).child(menu),
        )
    }

    /// 缩略块（SCN-CAPTURE-THUMB · DEC-188 真图承载）：120×64 · border · r6；
    /// 有缩略数据 → gpui 内部解码链 img（Contain 防拉伸溢出）；无数据（桥失败/
    /// 旧档无 thumb 文件）→ 棋盘占位（8 列交替双色 · 每列 15px）回退不变。
    fn thumb_block(&self, thumb: &Option<Arc<gpui::Image>>) -> Div {
        let frame = div()
            .w(px(120.0))
            .h(px(64.0))
            .border_1()
            .border_color(rgb(t::BORDER))
            .rounded(px(t::RAD_SM))
            .overflow_hidden()
            .flex()
            .flex_col();
        match thumb {
            Some(image) => div().child(
                frame.bg(rgb(t::SURFACE)).child(
                    img(image.clone())
                        .size_full()
                        .object_fit(ObjectFit::Contain),
                ),
            ),
            None => {
                let mut grid = frame;
                for row in 0..4 {
                    let mut line = div().flex().flex_1();
                    for col in 0..8 {
                        let even = (row + col) % 2 == 0;
                        let bg = if even { t::NEUTRAL } else { t::BORDER_SOFT };
                        line = line.child(div().flex_1().h_full().bg(rgb(bg)));
                    }
                    grid = grid.child(line);
                }
                div().child(grid)
            }
        }
    }

    /// 筛选行（PG-A ①）：二 chip_dd 接线（DEC-111 ②b · DEC-112）＋密度 co::switch
    ///（默认关=紧凑 / 开=宽松）＋同行强制 nowrap。下拉浮层宿主
    /// =relative 定高行：浮层用 deferred 延迟置顶绘制（盖过列表兄弟）＋absolute 显式
    /// top/left 定位。下拉开启期间整行包 out（任意外点击关菜单；
    /// chips/开关属界内不触发——同 chip toggle 语义保留；点行/空白仅收菜单不次级
    /// 触发，DEC-114 消耗语义）。
    fn filter_bar(&self, cx: &mut Context<Self>) -> Div {
        // 逐 chip 锚定（DEC-113）：各 chip 包 relative wrapper，浮层 deferred
        // absolute top=100% wrapper 高＝chip 下缘；先建浮层体再建 chips（顺序互斥借用）。
        let kind_layer = (self.dropdown == Some(Dd::Kind)).then(|| self.dd_anchor(Dd::Kind, cx));
        let day_layer = (self.dropdown == Some(Dd::Day)).then(|| self.dd_anchor(Dd::Day, cx));
        // 点外收起由 dd_anchor 内浮层 wrapper 的 out 承载（out 界域＝浮层 wrapper：
        // 若挂整行 wrapper——浮层点击位在行矩形外，capture 相先行收层 + stop 分派
        // 消耗本击，选项 click 永不触发；vendor on_mouse_down_out 语义取证）。
        // 三处筛选栏内外边距统一（DEC-180）：纵向内边距 6px——与集锦（collection.rs
        // home_view 工具条）同值同构（py6 自然高承接）。
        div()
            .px(px(16.0))
            .py(px(6.0))
            .flex()
            .items_center()
            .gap(px(6.0))
            .child(
                div()
                    .relative()
                    .child(
                        co::chip_dd("类型")
                            .id("dd-chip-kind")
                            .debug_selector(|| "clp-dd-chip-kind".to_string())
                            .on_click(cx.listener(
                                |this, _e: &ClickEvent, _w: &mut Window, cx: &mut Context<Self>| {
                                    this.toggle_dd(Dd::Kind, cx)
                                },
                            ))
                            .active(co::act_border),
                    )
                    .when_some(kind_layer, |el, layer| el.child(layer)),
            )
            .child(
                div()
                    .relative()
                    .child(
                        co::chip_dd("日期")
                            .id("dd-chip-day")
                            .on_click(cx.listener(
                                |this, _e: &ClickEvent, _w: &mut Window, cx: &mut Context<Self>| {
                                    this.toggle_dd(Dd::Day, cx)
                                },
                            ))
                            .active(co::act_border),
                    )
                    .when_some(day_layer, |el, layer| el.child(layer)),
            )
            .child(div().flex_1())
            // 队列入口标记（DEC-207 · §13.2 ⑦ collect-queue-active「入口标记
            // ＋剩余数」）：队列持条目时才呈现（空队列不渲染位）；点击 →
            // 队列详情浮出层。实装＝lucide copy glyph＋剩余数（设计稿 ⧉N 记法：
            // U+29C9 仅 Apple Symbols 在册 · UI 字体栈四族均无 · 文本直排 tofu
            // 风险——CoreText 探针实证 DEC-207）。
            .when(!self.queue.is_empty(), |bar| {
                // 队列按钮 bounce（TASK-716 · §11.2 面板动效）：序>0 时包 with_animation——
                // y 位移脉冲曲线（两半程 ~450ms×2 非 repeat · 实现注记：单段 900ms
                // 正弦两拍同形）；id 随序变化→重挂重播（未撞首弹不包）动画元素。
                let seq = self.queue_bounce_seq;
                let chip = div()
                    .id("chip-queue")
                    .debug_selector(|| "clp-chip-queue".to_string())
                    .flex()
                    .items_center()
                    .gap(px(4.0))
                    .px(px(10.0))
                    .py(px(1.0))
                    .border_1()
                    .border_color(rgb(t::BORDER))
                    .rounded_full()
                    .cursor_pointer()
                    .hover(co::hov_warm)
                    .on_click(cx.listener(
                        |this, _e: &ClickEvent, _w: &mut Window, cx: &mut Context<Self>| {
                            this.open_queue(cx)
                        },
                    ))
                    .child(co::icon(co::Icon::Copy, 10.0, t::MUTED))
                    .child(co::txt(
                        self.queue.len().to_string(),
                        s::font(s::Lvl::Meta),
                        t::MUTED,
                    ));
                bar.child(if seq == 0 {
                    chip.into_any_element()
                } else {
                    chip.with_animation(
                        ("chip-queue-bounce", seq as usize),
                        Animation::new(Duration::from_millis(QUEUE_CHIP_PULSE_MS * 2)),
                        |el, t| el.relative().top(px(queue_chip_dy(t))),
                    )
                    .into_any_element()
                })
            })
            .child(co::txt("宽松", s::font(s::Lvl::Meta), t::MUTED))
            .child(
                co::switch(!self.density_compact)
                    .id("sw-density")
                    .on_click(cx.listener(Self::toggle_density))
                    .active(co::act_border),
            )
    }

    /// 浮层锚（DEC-113）：deferred absolute，top=100% 宿主（chip wrapper）高＝chip
    /// 下缘；恒左对齐（二下拉均在近左缘 chips 下正对）；4px 起浮由面板体 mt 承载。
    /// 点外收起 out 挂浮层 wrapper：wrapper 矩形＝面板体＋起浮间隙 → 点面板内选项
    /// ＝界内不触发 out、bubble 相 click 正常；点面板外任意处＝capture 相收层＋stop
    /// 分派消耗本击（DEC-114 消耗口径，与右键菜单 ctx_layer wrapper 同形态）。
    fn dd_anchor(&self, d: Dd, cx: &mut Context<Self>) -> gpui::AnyElement {
        let view = cx.weak_entity();
        let out = move |_: &gpui::MouseDownEvent, _: &mut gpui::Window, cx: &mut gpui::App| {
            let _ = view.update(cx, |v, cx| {
                if v.dropdown.is_some() {
                    v.dropdown = None;
                    cx.notify();
                }
            });
            cx.stop_propagation();
        };
        let anchor = div().absolute().top(relative(1.0)).left(px(0.0));
        deferred(
            anchor
                // 鼠标隔离面（DEC-169 · D-04 · FAB/ctx 右键同款）：选项叠在 collect
                // 行带上而 hitbox 无遮挡时，点击会同合身后行击——block 截断 hover
                // 栈，滚轮经 should_handle_scroll 照常透传、浮层子树在前方不受影响。
                .block_mouse_except_scroll()
                .on_mouse_down_out(out)
                .child(self.dd_panel_body(d, cx)),
        )
        .into_any_element()
    }

    /// 浮层面板体（DEC-112）：头部复用 count_line；选项行=row＋左标注＋右「✓」。
    fn dd_panel_body(&self, d: Dd, cx: &mut Context<Self>) -> Div {
        let (header, options) = match d {
            Dd::Kind => self.dd_kind_panel(cx),
            Dd::Day => self.dd_day_panel(cx),
        };
        div()
            .mt(px(4.0)) // 4px 起浮（DEC-112 定数 · 逐 chip 锚定后由面板 mt 承载）
            .w(px(200.0))
            .border_1()
            .border_color(rgb(t::BORDER))
            .rounded(px(t::RAD_MD))
            .bg(rgb(t::SURFACE))
            .overflow_hidden()
            .flex()
            .flex_col()
            .child(header)
            .children(options)
    }

    /// 类型下拉：10 行多选，头部动态「不限/已选 N」；勾选后面板保持开放。
    fn dd_kind_panel(&self, cx: &mut Context<Self>) -> (Div, Vec<gpui::Stateful<Div>>) {
        let header = co::count_line(match self.sel_kinds.is_empty() {
            true => "类型筛选 · 不限".to_string(),
            false => format!("类型筛选 · 已选 {}", self.sel_kinds.len()),
        });
        let mut rows = Vec::new();
        for (i, &k) in FILTER_KINDS.iter().enumerate() {
            let on = self.sel_kinds.contains(&k);
            rows.push(
                co::row()
                    .justify_between()
                    .id(("dd-kind", i))
                    // 下拉选项穿透测试取样面（DEC-169 · D-04）：clp-kind-opt-<i>。
                    .debug_selector(move || format!("clp-kind-opt-{}", i))
                    .on_click(cx.listener(
                        move |this, _e: &ClickEvent, _w: &mut Window, cx: &mut Context<Self>| {
                            this.dd_kind_toggle(k, cx)
                        },
                    ))
                    .hover(co::hov_warm)
                    .active(co::act_border)
                    .child(co::txt(k.label(), s::font(s::Lvl::Small), t::PRIMARY))
                    .child(if on {
                        co::txt("✓", s::font(s::Lvl::Small), t::PRIMARY)
                    } else {
                        div()
                    }),
            );
        }
        (header, rows)
    }

    /// 日期下拉：全部＋三档（单选即收起）。
    fn dd_day_panel(&self, cx: &mut Context<Self>) -> (Div, Vec<gpui::Stateful<Div>>) {
        let header = co::count_line(format!(
            "日期筛选 · {}",
            self.sel_day.map(|d| d.label()).unwrap_or("全部")
        ));
        const DAY_OPTIONS: [(Option<DayFilter>, &str); 4] = [
            (None, "全部"),
            (Some(DayFilter::Today), "今天"),
            (Some(DayFilter::ThisWeek), "本周"),
            (Some(DayFilter::ThisMonth), "本月"),
        ];
        let mut rows = Vec::new();
        for (i, &(v, label)) in DAY_OPTIONS.iter().enumerate() {
            let on = self.sel_day == v;
            rows.push(
                co::row()
                    .justify_between()
                    .id(("dd-day", i))
                    .on_click(cx.listener(
                        move |this, _e: &ClickEvent, _w: &mut Window, cx: &mut Context<Self>| {
                            this.dd_day_set(v, cx)
                        },
                    ))
                    .hover(co::hov_warm)
                    .active(co::act_border)
                    .child(co::txt(label, s::font(s::Lvl::Small), t::PRIMARY))
                    .child(if on {
                        co::txt("✓", s::font(s::Lvl::Small), t::PRIMARY)
                    } else {
                        div()
                    }),
            );
        }
        (header, rows)
    }

    /// PG-A1 队列浮出层内容（fill 全替换呈现 · DEC-207 真数据承载）：全部行
    /// 来自 app 层 QueueHost 镜像（消费序已排齐——首项 placed＝下一消耗项，
    /// LIFO 整列倒置随回推；行文本不截断，超长随 q-list-scroll 纵向滚动）。
    fn queue_overlay(&self, cx: &mut Context<Self>) -> Div {
        let header = co::sheeth()
            .child(
                div()
                    .flex()
                    .gap(px(4.0))
                    .child(co::txt_f(
                        "队列详情 · 剩余 ",
                        s::font(s::Lvl::Small),
                        t::PRIMARY,
                        t::FONT_BODY,
                        t::FONT_WEIGHT_REGULAR,
                    ))
                    .child(co::txt_f(
                        self.queue.len().to_string(),
                        s::font(s::Lvl::Small),
                        t::TERTIARY,
                        t::FONT_BODY,
                        t::FONT_WEIGHT_LIGHT,
                    )),
            )
            // FIFO⇄LIFO 切换（design §13.3 ②）：点击上拋 ToggleOrder，宿主
            // 翻转后镜像回推（首项随之换面）。
            .child(
                co::meta("FIFO ⇄ LIFO")
                    .id("q-order")
                    .cursor_pointer()
                    .hover(co::hov_warm)
                    .on_click(cx.listener(Self::queue_toggle_order)),
            );

        // placed 行（first＝下一写板 · growth 左条 + warm 底）+ 序号行
        //（序号＝呈现位序 1 基，placed 无号）。
        let q_rows = self
            .queue
            .iter()
            .enumerate()
            .map(|(di, r)| {
                if di == 0 {
                    div()
                        .flex()
                        .items_center()
                        .gap(px(10.0))
                        .px(px(16.0))
                        .py(px(11.0))
                        .bg(rgb(t::WARM))
                        .relative()
                        .border_b_1()
                        .border_color(rgb(t::BORDER_SOFT))
                        .id(("q-placed", r.id))
                        .debug_selector(move || format!("a1-qrow-{}", r.id))
                        .child(
                            div()
                                .absolute()
                                .left_0()
                                .top_0()
                                .bottom_0()
                                .w(px(2.0))
                                .bg(rgb(t::ACCENT)),
                        )
                        .child(co::meta("下一写板"))
                        .child(co::t_label(r.text.clone()))
                } else {
                    co::row()
                        .id(("q-row", r.id))
                        .debug_selector(move || format!("a1-qrow-{}", r.id))
                        .child(co::meta((di + 1).to_string()))
                        .child(co::t_label(r.text.clone()))
                }
            })
            .collect::<Vec<gpui::Stateful<Div>>>();

        let actions = div()
            .flex()
            .gap(px(12.0))
            .px(px(16.0))
            .pt(px(14.0))
            .pb(px(6.0))
            .child(
                co::btn_pri("停止（补写 placed 项）")
                    .id("q-stop")
                    .flex_1()
                    .on_click(cx.listener(Self::close_queue))
                    .active(co::act_ink_deep),
            )
            .child(
                co::btn_sec("清空")
                    .id("q-clear")
                    .on_click(cx.listener(Self::clear_queue))
                    .active(co::act_border_ink),
            );

        // sec 盒（surface · r8 · p 20/24）内装队列列表——列表区 q-list-scroll
        // 有界滚动（REQ-PANEL-LIST-SCROLL · DEC-106）：条目超长时队列行纵向滚动，
        // sheeth 与操作行常驻不随滚。
        let sec = co::sec_box()
            .flex_1()
            .min_h_0()
            .flex()
            .flex_col()
            .child(header)
            .child(
                div()
                    .id("q-list-scroll")
                    .debug_selector(|| "a1-qlist".to_string())
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .children(q_rows),
            )
            .child(actions);

        div()
            .flex_1()
            .min_h_0()
            .flex()
            .flex_col()
            .bg(gpui::linear_gradient(
                180.0,
                gpui::linear_color_stop(co::h(t::SURFACE), 0.0),
                gpui::linear_color_stop(co::h(t::NEUTRAL), 1.0),
            ))
            .p(px(s::sp(t::SP_MD)))
            .child(sec)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .pt(px(10.0))
                    .child(co::hint("队列清空后不残留任何占位 · Esc 关闭本层回 拾记")),
            )
    }
}

pub fn short_title(s: &str) -> String {
    let chars: Vec<char> = s.chars().take(12).collect();
    let mut out: String = chars.into_iter().collect();
    if s.chars().count() > 12 {
        out.push('…');
    }
    out
}

impl Render for CaptureView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        // ── A1 队列态：fill 全替换 ──
        if self.queue_open {
            let overlay = self.queue_overlay(cx);
            return div()
                .size_full()
                .relative()
                .flex()
                .flex_col()
                .child(overlay)
                .when_some(self.toast.clone(), |el, msg| {
                    el.child(
                        div()
                            .absolute()
                            .bottom(px(24.0))
                            .left_0()
                            .right_0()
                            .flex()
                            .justify_center()
                            .child(co::toast(msg).id("toast-a1").on_click(cx.listener(
                                |this, _e: &ClickEvent, _w: &mut Window, cx: &mut Context<Self>| {
                                    this.toast = None;
                                    cx.notify();
                                },
                            ))),
                    )
                });
        }

        // ── PG-A 正常态 ──
        // 二下拉 AND 过滤（DEC-112）：计数行随生效条
        // 件更新（REQ-SEARCH-COUNT · HTML「匹配 N 条」文案）。
        let density = self.density_compact;
        let filtered = filter_rows(&self.rows, &self.sel_kinds, self.sel_day);
        // 行时显示基准（DEC-183）：今日本地字段帧头取一次（无逐行 FFI）。
        let now = walltime::local_civil(walltime::now_epoch());
        let mut list = div()
            .flex()
            .flex_col()
            .child(co::count_line(format!("匹配 {} 条", filtered.len())));
        for &orig in &filtered {
            let mut row = self.clip_row(orig, &self.rows[orig], &now, cx);
            if !density {
                row = row.py(px(16.0)); // 完整密度：56pt 卡片感（行高加大）
            }
            list = list.child(row);
        }
        if self.rows.is_empty() {
            // 空列表态（DEC-183 · 初始空面引导）。
            list = list.child(div().p(px(24.0)).flex().justify_center().child(co::txt(
                "还没有条目 · 复制任意文本后自动出现在这里",
                s::font(s::Lvl::Small),
                t::MUTED,
            )));
        } else if filtered.is_empty() {
            // collect-empty-filter 方向文案（PRD §4.1 空态 · REQ-SEARCH-FILTERS 空结果）
            list = list.child(div().p(px(24.0)).flex().justify_center().child(co::txt(
                "没有匹配的条目，试试放宽筛选",
                s::font(s::Lvl::Small),
                t::MUTED,
            )));
        }

        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(rgb(t::NEUTRAL))
            .relative()
            .child(self.filter_bar(cx))
            .child(
                div()
                    .id("cap-scroll")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .child(list),
            )
            .when(self.ctx.is_some(), |el| {
                // 右键菜单视图根挂载（DEC-116 点击位锚定 · deferred anchored）：
                // 祖先链无滚动容器，绘制于开启瞬间的光标位、不随滚动行漂移。
                el.child(self.ctx_layer(self.ctx.unwrap(), cx))
            })
            .when_some(self.toast.clone(), |el, msg| {
                el.child(
                    div()
                        .absolute()
                        .bottom(px(24.0))
                        .left_0()
                        .right_0()
                        .flex()
                        .justify_center()
                        .child(co::toast(msg).id("toast-a").on_click(cx.listener(
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
    // cx.new（TestAppContext）经 AppContext trait 承载 —— 隔离在测试模块内引入
    //（顶层不再引入：生产面无 cx.new 用点，避免未用导入告警）。
    use gpui::AppContext as _;

    /// 测试夹具行（DEC-183 · 过滤/清理语义对账自建面）：文本
    ///（Pin·今天）/ 链接（本周）/ 图片（本月·棋盘）/ 文件四档覆盖维度。
    fn fx_rows() -> Vec<ClipRow> {
        vec![
            ClipRow {
                title: "文本条目甲".to_string(),
                pinned: true,
                ..Default::default()
            },
            ClipRow {
                title: "链接条目乙".to_string(),
                kind: FilterKind::Url,
                day: DayFilter::ThisWeek,
                ..Default::default()
            },
            ClipRow {
                title: "图片条目丙".to_string(),
                thumb: true,
                kind: FilterKind::Image,
                day: DayFilter::ThisMonth,
                ..Default::default()
            },
            ClipRow {
                title: "文件条目丁.md".to_string(),
                kind: FilterKind::FileUrl,
                ..Default::default()
            },
        ]
    }

    // 行单击 → 统一详情窗上抛 + 选中态保留。
    #[gpui::test]
    async fn row_click_emits_preview_and_keeps_selection(cx: &mut gpui::TestAppContext) {
        let titles: std::rc::Rc<std::cell::RefCell<Vec<String>>> = Default::default();
        let view = cx.new(|cx| {
            let mut v = CaptureView::new(cx);
            let titles = titles.clone();
            v.set_peek_sink(std::rc::Rc::new(
                move |item: &PeekItem, _: &mut gpui::App| {
                    titles.borrow_mut().push(item.title.clone());
                },
            ));
            v.rows = fx_rows();
            v.rows[1].selected = true;
            v
        });
        let first_selected = view.update(cx, |v, cx| {
            v.row_activate(0, cx); // 文本行（fx 甲）
            v.rows[1].selected
        });
        assert_eq!(*titles.borrow(), vec!["文本条目甲".to_string()]);
        assert!(first_selected); // 选中高亮保留：点击不改变选中态
    }

    // ── 二下拉过滤纯函数（DEC-112 · REQ-SEARCH-FILTERS 语义对账）──

    #[test]
    fn filter_rows_no_filters_returns_all() {
        let rows = fx_rows();
        assert_eq!(filter_rows(&rows, &[], None), vec![0, 1, 2, 3]);
    }

    #[test]
    fn filter_rows_kind_multi_select() {
        // SCN-SEARCH-TYPE-MULTI-SELECT 同构：勾选 文本+图片 → 排除其余类型。
        let rows = fx_rows();
        assert_eq!(
            filter_rows(&rows, &[FilterKind::PlainText, FilterKind::Image], None),
            vec![0, 2]
        );
        // 夹具未覆盖的类型（笔记）→ 空结果。
        assert_eq!(
            filter_rows(&rows, &[FilterKind::Note], None),
            Vec::<usize>::new()
        );
    }

    #[test]
    fn filter_rows_day_cumulative() {
        // 窗径累积：今天 ⊂ 本周 ⊂ 本月（夹具：甲今天/乙本周/丙本月/丁今天）。
        let rows = fx_rows();
        assert_eq!(filter_rows(&rows, &[], Some(DayFilter::Today)), vec![0, 3]);
        assert_eq!(
            filter_rows(&rows, &[], Some(DayFilter::ThisWeek)),
            vec![0, 1, 3]
        );
        assert_eq!(
            filter_rows(&rows, &[], Some(DayFilter::ThisMonth)),
            vec![0, 1, 2, 3]
        );
    }

    #[test]
    fn filter_rows_and_combo() {
        // SCN-SEARCH-AND-OR-COMBINE 同构：类型+日期 AND 组合。
        let rows = fx_rows();
        assert_eq!(
            filter_rows(&rows, &[FilterKind::PlainText], Some(DayFilter::Today)),
            vec![0]
        );
        // 类型/日期互斥 → 空结果（AND 不放宽）。
        assert_eq!(
            filter_rows(&rows, &[FilterKind::Image], Some(DayFilter::Today)),
            Vec::<usize>::new()
        );
    }

    #[test]
    fn filter_kinds_cover_8_types_plus_note_task() {
        // 对账 REQ-CAPTURE-8TYPES 8 类＋REQ-SEARCH-FILTERS 笔记/任务；标注非空。
        assert_eq!(FILTER_KINDS.len(), 10);
        for k in FILTER_KINDS {
            assert!(!k.label().is_empty());
        }
    }

    // 行时显示（DEC-183）：捕获行墙钟快照 → 三档格式串（同日/同年/跨年）。
    #[test]
    fn clip_time_formats_three_windows() {
        let now = Stamp {
            epoch: 100,
            y: 2026,
            mo: 10,
            d: 22,
            h: 9,
            mi: 0,
            s: 0,
        };
        let same_day = Stamp {
            epoch: 60,
            y: 2026,
            mo: 10,
            d: 22,
            h: 8,
            mi: 30,
            s: 5,
        };
        assert_eq!(
            walltime::fmt_between(&same_day, &now),
            "08:30:05",
            "同日 → 时:分:秒"
        );
        let prev_day = Stamp {
            epoch: 50,
            y: 2026,
            mo: 10,
            d: 21,
            h: 8,
            mi: 30,
            s: 5,
        };
        assert_eq!(
            walltime::fmt_between(&prev_day, &now),
            "10-21 08:30:05",
            "同年异日 → 月-日 时:分:秒"
        );
        let last_year = Stamp {
            epoch: 40,
            y: 2025,
            mo: 1,
            d: 2,
            h: 8,
            mi: 30,
            s: 5,
        };
        assert_eq!(
            walltime::fmt_between(&last_year, &now),
            "2025-01-02 08:30:05",
            "跨年 → 年-月-日 时:分:秒"
        );
    }

    // ── 剪贴板捕获接线（DEC-163 · REQ-CAPTURE-* 视图侧承载）──

    #[gpui::test]
    async fn ingest_clip_prepends_capture_row(cx: &mut gpui::TestAppContext) {
        let view = cx.new(|cx| {
            let mut v = CaptureView::new(cx);
            v.ingest_clip(
                "剪贴板新捕获的文本行".to_string(),
                std::time::Instant::now(),
                crate::walltime::now_epoch(),
                cx,
            );
            v
        });
        let (len, top) = view.update(cx, |v, _cx| {
            let r0 = &v.rows[0];
            (
                v.rows.len(),
                (
                    r0.title.clone(),
                    r0.kind,
                    r0.day,
                    r0.pinned,
                    r0.handled,
                    r0.wall.is_zero(),
                ),
            )
        });
        assert_eq!(len, 1, "空列表首条入列（最新在前）");
        assert_eq!(top.0, "剪贴板新捕获的文本行");
        assert_eq!(top.1, FilterKind::PlainText); // 8 类对账：文本类
        assert_eq!(top.2, DayFilter::Today); // 今天档命中日期筛选
        assert!(!top.3 && !top.4, "捕获行默认未 Pin 未处理");
        assert!(!top.5, "捕获点墙钟快照已记录（DEC-183 行时显示源）");
    }

    #[gpui::test]
    async fn sweep_clip_cleans_unhandled_after_valid_window(cx: &mut gpui::TestAppContext) {
        let view = cx.new(|cx| {
            let mut v = CaptureView::new(cx);
            // 已过有效时长（默认 24h）+ 1 分钟：未处理 → 清；Pin 夹具行豁免存活窗。
            let stale = std::time::Instant::now()
                .checked_sub(Duration::from_secs(24 * 3600 + 60))
                .unwrap();
            v.rows = fx_rows();
            v.ingest_clip("过期未处理条目".to_string(), stale, 0, cx);
            v
        });
        let removed = view.update(cx, |v, cx| v.sweep_clip(7 * 24 * 60, 24 * 60, 0, cx));
        view.update(cx, |v, _cx| {
            assert_eq!(removed, 1, "仅过期未处理行被清");
            assert!(!v.rows.iter().any(|r| r.title == "过期未处理条目"));
            assert!(
                v.rows.iter().any(|r| r.title == "文本条目甲"),
                "Pin 行豁免存活窗"
            );
        });
    }

    #[gpui::test]
    async fn sweep_clip_survive_cap_exempts_pin_and_handled_kept(cx: &mut gpui::TestAppContext) {
        let view = cx.new(|cx| {
            let mut v = CaptureView::new(cx);
            v.rows = fx_rows();
            let old = std::time::Instant::now()
                .checked_sub(Duration::from_secs(90))
                .unwrap();
            // 已处理行：超有效时长（valid=1min）但未达存活窗（survive=2min）→ 留。
            v.ingest_clip("已处理陈条".to_string(), old, 0, cx);
            v.rows[0].handled = true;
            // 未 Pin 未处理旧行：超有效窗即清。
            v.ingest_clip("未处理陈条".to_string(), old, 0, cx);
            v
        });
        view.update(cx, |v, cx| {
            // valid=1min：两条均过期；handled 行由存活窗（90s<2min）保住；
            // 未 Pin 未处理行被清。Pin 夹具行未达存活窗 → 留。
            let removed = v.sweep_clip(2, 1, 0, cx);
            assert_eq!(removed, 1, "仅未 Pin 未处理过期行被清");
            assert!(
                v.rows.iter().any(|r| r.title == "已处理陈条"),
                "handled 行存活窗内保留"
            );
            assert!(v.rows.iter().any(|r| r.title == "文本条目甲"), "Pin 豁免");
        });
    }

    // 计数上限裁剪（DEC-188 · REQ-CAPTURE-LIMIT）：非 Pin 行超限从最旧侧裁
    //（最新在顶），Pin 行恒豁免（下标定位删除不做整段 truncate）。
    #[gpui::test]
    async fn sweep_clip_count_limit_trims_unpinned_keeps_pin(cx: &mut gpui::TestAppContext) {
        let view = cx.new(|cx| {
            let mut v = CaptureView::new(cx);
            // 顶（最新）→ 尾（最旧）：甲 Pin / 乙丙丁未 Pin。
            v.rows = vec![
                ClipRow {
                    title: "最新Pin甲".into(),
                    pinned: true,
                    ..Default::default()
                },
                ClipRow {
                    title: "未Pin乙".into(),
                    ..Default::default()
                },
                ClipRow {
                    title: "未Pin丙".into(),
                    ..Default::default()
                },
                ClipRow {
                    title: "未Pin丁".into(),
                    ..Default::default()
                },
            ];
            v
        });
        view.update(cx, |v, cx| {
            let removed = v.sweep_clip(100 * 365 * 24 * 60, 100 * 365 * 24 * 60, 2, cx);
            assert_eq!(removed, 1, "limit=2 只裁最旧一条未 Pin");
            let titles: Vec<String> = v.rows.iter().map(|r| r.title.clone()).collect();
            assert_eq!(
                titles,
                vec![
                    "最新Pin甲".to_string(),
                    "未Pin乙".to_string(),
                    "未Pin丙".to_string()
                ],
                "未 Pin 保最新两条 · Pin 恒豁免（尾部夹杂 Pin 不被连带裁）"
            );
            // limit=0（无限）→ 不裁。
            let removed = v.sweep_clip(100 * 365 * 24 * 60, 100 * 365 * 24 * 60, 0, cx);
            assert_eq!(removed, 0, "limit=0 无限档不触发裁剪");
        });
    }

    // 重试补写成功回填（DEC-188 · D-06）：按加入时刻对位置 db_id；已带身份行
    // 不重复覆盖。
    #[gpui::test]
    async fn update_db_ids_backfills_by_added_at(cx: &mut gpui::TestAppContext) {
        let at_a = std::time::Instant::now();
        let at_b = std::time::Instant::now();
        let view = cx.new(|_cx| {
            let mut v = CaptureView::new(_cx);
            v.rows = vec![
                ClipRow {
                    title: "重试甲".into(),
                    added_at: at_a,
                    ..Default::default()
                },
                ClipRow {
                    title: "已落库乙".into(),
                    added_at: at_b,
                    db_id: Some("seeded".into()),
                    ..Default::default()
                },
            ];
            v
        });
        let changed = view.update(cx, |v, cx| {
            v.update_db_ids(
                &[(at_a, "id-a".to_string()), (at_b, "id-b".to_string())],
                cx,
            )
        });
        view.update(cx, |v, _cx| {
            assert_eq!(changed, 1, "仅未带身份行被回填");
            assert_eq!(v.rows[0].db_id.as_deref(), Some("id-a"));
            assert_eq!(v.rows[1].db_id.as_deref(), Some("seeded"), "已带身份不覆盖");
        });
    }

    // 右键菜单防竞态（R184 F-183-02 Prove-It）：菜单开启记录行身份 → 顶插使
    // 捕获下标过期 → 菜单项以旧下标调用仍按身份解析命中原行（不误标新行）。
    #[gpui::test]
    async fn ctx_menu_identity_resolves_after_top_insert(cx: &mut gpui::TestAppContext) {
        let at_a = std::time::Instant::now();
        let view = cx.new(|cx| {
            let mut v = CaptureView::new(cx);
            v.ingest_clip("原行甲".to_string(), at_a, 0, cx);
            v
        });
        view.update(cx, |v, cx| {
            v.ctx_open(0, gpui::point(gpui::px(20.0), gpui::px(30.0)), cx);
            // 顶插新行 → 「原行甲」由 0 移到 1（捕获下标 0 现指新行）。
            v.ingest_clip("插行乙".to_string(), std::time::Instant::now(), 0, cx);
            v.ctx_to_note(0, cx);
            assert!(!v.rows[0].handled, "顶插新行不得被旧下标误标");
            assert!(v.rows[1].handled, "身份解析命中原行（防竞态 · F-183-02）");
            assert_eq!(v.rows[1].title, "原行甲", "命中的正是右键那行");
            assert!(v.ctx.is_none(), "菜单执行后统一收起");
        });
    }

    #[gpui::test]
    async fn ctx_to_note_marks_row_handled(cx: &mut gpui::TestAppContext) {
        let view = cx.new(|cx| {
            let mut v = CaptureView::new(cx);
            v.ingest_clip("待转化条目".to_string(), std::time::Instant::now(), 0, cx);
            v.ctx_to_note(0, cx);
            v
        });
        view.update(cx, |v, _cx| {
            assert!(
                v.rows[0].handled,
                "转为笔记即置已处理（DEC-163 清理谓词消费面）"
            );
            assert!(v.ctx.is_none(), "菜单执行后统一收起");
            assert!(
                v.toast.is_none(),
                "转换无 Toast（DEC-194：反馈载体＝新开面板）"
            );
        });
    }

    // 右键菜单（DEC-194）：转为笔记/转为任务——条目原文经双通道上抛（宿主
    // 开窗预填承载）+ 即置 handled；handled 同置不分支于通道注入与否。
    #[gpui::test]
    async fn ctx_convert_emits_clip_via_sinks(cx: &mut gpui::TestAppContext) {
        let noted: std::rc::Rc<std::cell::RefCell<Vec<String>>> = Default::default();
        let tasked: std::rc::Rc<std::cell::RefCell<Vec<String>>> = Default::default();
        let view = cx.new(|cx| {
            let mut v = CaptureView::new(cx);
            v.set_clip_to_note_sink({
                let noted = noted.clone();
                std::rc::Rc::new(move |content: &str, _: &mut gpui::App| {
                    noted.borrow_mut().push(content.to_string());
                })
            });
            v.set_clip_to_task_sink({
                let tasked = tasked.clone();
                std::rc::Rc::new(move |content: &str, _: &mut gpui::App| {
                    tasked.borrow_mut().push(content.to_string());
                })
            });
            v.ingest_clip("转写原文甲".to_string(), std::time::Instant::now(), 0, cx);
            v.ctx_to_note(0, cx);
            v.ingest_clip("转写原文乙".to_string(), std::time::Instant::now(), 1, cx);
            v.ctx_to_task(0, cx);
            v
        });
        view.update(cx, |v, _cx| {
            assert_eq!(
                noted.borrow().as_slice(),
                ["转写原文甲"],
                "转为笔记上抛条目原文（宿主预填正文承载）"
            );
            assert_eq!(
                tasked.borrow().as_slice(),
                ["转写原文乙"],
                "转为任务上抛条目原文（宿主预填任务域承载）"
            );
            assert!(
                v.rows.iter().all(|r| r.handled),
                "两条目均即置已处理（DEC-163）"
            );
            assert!(v.toast.is_none(), "转写面无 Toast（DEC-194）");
        });
    }

    // 菜单项 · 删除（DEC-177）：从数据源移除该条目 + 菜单收起 + 原始
    // 下标序保持（filter_rows 返回源下标 → 行绑定/右键按源对位）。
    #[gpui::test]
    async fn ctx_delete_removes_row(cx: &mut gpui::TestAppContext) {
        let view = cx.new(|cx| {
            let mut v = CaptureView::new(cx);
            v.ingest_clip("待删除条目".to_string(), std::time::Instant::now(), 0, cx);
            v.ctx_open(0, gpui::point(gpui::px(20.0), gpui::px(30.0)), cx);
            v
        });
        let removed_title = view.update(cx, |v, cx| {
            let t = v.rows[0].title.clone();
            v.ctx_delete(0, cx);
            t
        });
        view.update(cx, |v, _cx| {
            assert_eq!(v.rows.len(), 0, "删除后列表清空（初始空面 · 无种子行）");
            assert!(
                v.rows.iter().all(|r| r.title != removed_title),
                "被删条目不得残留"
            );
            assert!(v.ctx.is_none(), "删除后菜单收起");
        });
    }

    // 右键菜单（DEC-114）：Esc 级联首位消化；菜单收起后继续透传（不重复关浮层）。
    #[gpui::test]
    async fn ctx_menu_dismiss_cascade_first(cx: &mut gpui::TestAppContext) {
        let view = cx.new(|cx| {
            let mut v = CaptureView::new(cx);
            v.ctx_open(0, gpui::point(gpui::px(20.0), gpui::px(30.0)), cx);
            v
        });
        let consumed_then_plain = view.update(cx, |v, cx| {
            let first = v.dismiss_overlay(cx); // 菜单 → 消化
            let second = v.dismiss_overlay(cx); // 无浮层 → 不消化
            (first, second)
        });
        assert_eq!(consumed_then_plain, (true, false));
    }

    // ctx_open 开启即收三下拉（浮层互斥 · DEC-114）。
    #[gpui::test]
    async fn ctx_open_closes_dropdown(cx: &mut gpui::TestAppContext) {
        let view = cx.new(|cx| {
            let mut v = CaptureView::new(cx);
            v.dropdown = Some(Dd::Kind);
            v.ctx_open(2, gpui::point(gpui::px(40.0), gpui::px(50.0)), cx);
            v
        });
        let (menu_open, dd_closed, pos_set, consumed) = view.update(cx, |v, cx| {
            (
                v.ctx == Some(2),
                v.dropdown.is_none(),
                v.ctx_pos.is_some(),
                v.dismiss_overlay(cx),
            )
        });
        assert!(menu_open && dd_closed && pos_set && consumed); // 互斥收下拉 + 点击位已记录 + 菜单被 Esc 首位消化
        assert!(!view.update(cx, |v, cx| v.dismiss_overlay(cx))); // 再次无浮层 → 不消化
    }

    // 类型下拉选项点击穿透回归（DEC-169 · D-04 · DEC-167 同式真派发）：浮层叠在
    // collect 行带之上，选项真实左击仅落值（多选开态保持）——身后行不得同合
    // 触发 row_activate（回贴 Toast ＋ peek 上抛）。几何前置自校验防空转。
    #[gpui::test]
    async fn kind_dd_option_click_no_passthrough_to_rows(cx: &mut gpui::TestAppContext) {
        use std::ops::Deref as _;
        let titles: std::rc::Rc<std::cell::RefCell<Vec<String>>> = Default::default();
        let window = cx.open_window(gpui::size(gpui::px(900.0), gpui::px(560.0)), |_w, cx| {
            crate::init(cx);
            let titles = titles.clone();
            let mut v = CaptureView::new(cx);
            v.set_peek_sink(std::rc::Rc::new(
                move |item: &PeekItem, _: &mut gpui::App| {
                    titles.borrow_mut().push(item.title.clone());
                },
            ));
            v.rows = fx_rows(); // 下拉选项压行几何前提（DEC-183 空面后自建夹具）
            v
        });
        let mut cx = gpui::VisualTestContext::from_window(*window.deref(), cx);
        cx.run_until_parked();
        // 真实左击「类型」chip 开层。
        let chip = cx
            .debug_bounds("clp-dd-chip-kind")
            .unwrap_or_else(|| panic!("类型 chip 未录制（debug_selector 失效）"));
        cx.simulate_click(chip.center(), Default::default());
        cx.run_until_parked();
        let open = window
            .update(&mut cx, |v, _w, _cx| v.dropdown == Some(Dd::Kind))
            .unwrap();
        assert!(open, "真实点击 chip 应开启类型下拉");
        // 几何前置自校验：某选项点击位须恰好有 collect 行压在下方（穿透面成立）。
        let row_n = window.update(&mut cx, |v, _w, _cx| v.rows.len()).unwrap();
        let found = (0..FILTER_KINDS.len())
            .find_map(|o| {
                let ob =
                    cx.debug_bounds(Box::leak(format!("clp-kind-opt-{}", o).into_boxed_str()))?;
                (0..row_n).find_map(|k| {
                    cx.debug_bounds(Box::leak(format!("clp-row-{}", k).into_boxed_str()))
                        .filter(|r| r.contains(&ob.center()))
                        .map(|_| ())
                })?;
                Some((o, ob))
            })
            .unwrap_or_else(|| panic!("应有下拉选项压在 collect 行上（几何前置失效）"));
        // 选项真实左击：穿透即误触发身后行（peek 上抛置位 ⇒ 红）。
        cx.simulate_click(found.1.center(), Default::default());
        cx.run_until_parked();
        assert!(
            titles.borrow().is_empty(),
            "选项点击不得穿透误触发身后 collect 行（peek 实得 {:?}）",
            titles.borrow()
        );
        let (sel, open_still) = window
            .update(&mut cx, |v, _w, _cx| {
                (v.sel_kinds.clone(), v.dropdown == Some(Dd::Kind))
            })
            .unwrap();
        assert_eq!(
            sel,
            vec![FILTER_KINDS[found.0]],
            "选项点击应落值（多选 toggle 加选）"
        );
        assert!(open_still, "多选语义勾选后面板保持开放");
        let toast = window
            .update(&mut cx, |v, _w, _cx| v.toast.clone())
            .unwrap();
        assert!(toast.is_none(), "回贴 Toast 不得同合触发（穿透即置位）");
    }

    // ── 队列接线（DEC-207 · A1 队列接线轮）──

    // 入队动作上抛（REQ-QUEUE-PLACE · 拾记行右键第四入）：快照文本携带面
    // ＋ Toast 剩余数同拍可算（镜像存量 +1）。
    #[gpui::test]
    async fn ctx_enqueue_emits_action_and_toast(cx: &mut gpui::TestAppContext) {
        let queued: std::rc::Rc<std::cell::RefCell<Vec<QueueAction>>> = Default::default();
        let view = cx.new(|cx| {
            let mut v = CaptureView::new(cx);
            v.set_queue_action_sink({
                let queued = queued.clone();
                std::rc::Rc::new(move |a: &QueueAction, _: &mut gpui::App| {
                    queued.borrow_mut().push(a.clone());
                })
            });
            v.ingest_clip("队列快照原文".to_string(), std::time::Instant::now(), 0, cx);
            v
        });
        view.update(cx, |v, cx| v.ctx_enqueue(0, cx));
        view.update(cx, |v, _cx| {
            assert_eq!(
                queued.borrow().as_slice(),
                [QueueAction::Enqueue("队列快照原文".into())],
                "入队快照＝行载荷文本"
            );
            assert_eq!(
                v.toast.as_deref(),
                Some("已加入队列 · 剩余 1"),
                "入队反馈 Toast（镜像存量+1）"
            );
        });
    }

    // 二进制/未知载荷行暂不入队（占位题名非真实载荷——TASK-301 承诺式引擎面）：
    // 不上抛动作、Toast 诚实差异面。
    #[gpui::test]
    async fn ctx_enqueue_non_text_kind_defers(cx: &mut gpui::TestAppContext) {
        let queued: std::rc::Rc<std::cell::RefCell<Vec<QueueAction>>> = Default::default();
        let view = cx.new(|cx| {
            let mut v = CaptureView::new(cx);
            v.set_queue_action_sink({
                let queued = queued.clone();
                std::rc::Rc::new(move |a: &QueueAction, _: &mut gpui::App| {
                    queued.borrow_mut().push(a.clone());
                })
            });
            v.ingest_row(
                ClipRow {
                    title: "(图片)".into(),
                    kind: FilterKind::Image,
                    thumb: true,
                    ..Default::default()
                },
                cx,
            );
            v
        });
        view.update(cx, |v, cx| v.ctx_enqueue(0, cx));
        view.update(cx, |v, _cx| {
            assert!(queued.borrow().is_empty(), "图片行不上抛入队动作");
            assert_eq!(
                v.toast.as_deref(),
                Some("图片/未知内容暂不入队"),
                "诚实差异面 Toast（引擎 defer 登记）"
            );
        });
    }

    // 停止/清空动作上抛（终态：动作出镜 · 视图关层；镜像由宿主回推承载）。
    // placed 补写（SCN-QUEUE-STOP-FIXUP）随 TASK-301 引擎——两动作同终态。
    #[gpui::test]
    async fn queue_stop_and_clear_fire_sinks_and_close(cx: &mut gpui::TestAppContext) {
        use std::ops::Deref as _;
        let queued: std::rc::Rc<std::cell::RefCell<Vec<QueueAction>>> = Default::default();
        let window = cx.open_window(gpui::size(gpui::px(900.0), gpui::px(560.0)), |_w, cx| {
            crate::init(cx);
            let queued_sink = queued.clone();
            let mut v = CaptureView::new(cx);
            v.set_queue_action_sink(std::rc::Rc::new(
                move |a: &QueueAction, _: &mut gpui::App| {
                    queued_sink.borrow_mut().push(a.clone());
                },
            ));
            v.set_queue_rows(
                vec![
                    QueueRow {
                        id: 1,
                        text: "甲".into(),
                    },
                    QueueRow {
                        id: 2,
                        text: "乙".into(),
                    },
                ],
                cx,
            );
            v.open_queue(cx);
            v
        });
        let mut cx = gpui::VisualTestContext::from_window(*window.deref(), cx);
        cx.run_until_parked();
        window
            .update(&mut cx, |v, w, _cx| {
                v.close_queue(&ClickEvent::default(), w, _cx);
            })
            .unwrap();
        window
            .update(&mut cx, |v, _w, _cx| {
                assert_eq!(queued.borrow().as_slice(), [QueueAction::Stop], "停止上抛");
                assert!(!v.queue_open, "停止后层关（回拾记）");
                assert_eq!(v.queue.len(), 2, "镜像由宿主回推承载（视图不自清）");
            })
            .unwrap();
        window
            .update(&mut cx, |v, w, _cx| {
                v.clear_queue(&ClickEvent::default(), w, _cx);
            })
            .unwrap();
        window
            .update(&mut cx, |v, _w, _cx| {
                assert_eq!(
                    queued.borrow().as_slice(),
                    [QueueAction::Stop, QueueAction::Clear],
                    "清空上抛"
                );
                assert!(!v.queue_open, "清空后层关");
            })
            .unwrap();
    }

    // Esc 关层回拾记（§13.3 ②）：队列条目保留在镜像（入口标记在册可重开）；
    // 消化返回 true（dismiss 级联序 collect ctx→dropdown→queue · DEC-114）。
    #[gpui::test]
    async fn queue_dismiss_keeps_mirror(cx: &mut gpui::TestAppContext) {
        let view = cx.new(|cx| {
            let mut v = CaptureView::new(cx);
            v.set_queue_rows(
                vec![QueueRow {
                    id: 1,
                    text: "甲".into(),
                }],
                cx,
            );
            v.open_queue(cx);
            v
        });
        let (closed, kept) = view.update(cx, |v, cx| {
            let closed = v.dismiss_overlay(cx);
            (closed, v.queue.len())
        });
        assert!(closed, "队列层在 dismiss 级联位上消化");
        assert_eq!(kept, 1, "Esc 仅关层——队列条目保留（可重开）");
    }

    // FIFO⇄LIFO 切换上抛（design §13.3 ②）：动作出镜，序随宿主镜像回推。
    #[gpui::test]
    async fn queue_toggle_order_fires_sink(cx: &mut gpui::TestAppContext) {
        use std::ops::Deref as _;
        let queued: std::rc::Rc<std::cell::RefCell<Vec<QueueAction>>> = Default::default();
        let window = cx.open_window(gpui::size(gpui::px(900.0), gpui::px(560.0)), |_w, cx| {
            crate::init(cx);
            let queued_sink = queued.clone();
            let mut v = CaptureView::new(cx);
            v.set_queue_action_sink(std::rc::Rc::new(
                move |a: &QueueAction, _: &mut gpui::App| {
                    queued_sink.borrow_mut().push(a.clone());
                },
            ));
            v
        });
        let mut cx = gpui::VisualTestContext::from_window(*window.deref(), cx);
        cx.run_until_parked();
        window
            .update(&mut cx, |v, w, _cx| {
                v.queue_toggle_order(&ClickEvent::default(), w, _cx);
            })
            .unwrap();
        window
            .update(&mut cx, |_v, _w, _cx| {
                assert_eq!(
                    queued.borrow().as_slice(),
                    [QueueAction::ToggleOrder],
                    "切换上抛"
                );
            })
            .unwrap();
    }

    // 队列按钮 bounce（TASK-716 · §11.2 面板动效通道）：注入即序自增＋重渲
    //（动画元素重挂由 id 随序承载，未撞首弹不包）；脉冲曲线端点落位。
    #[gpui::test]
    async fn queue_chip_bounce_increments_and_curve_lands(cx: &mut gpui::TestAppContext) {
        let view = cx.new(|cx| {
            let mut v = CaptureView::new(cx);
            v.set_queue_rows(
                vec![QueueRow {
                    id: 1,
                    text: "甲".into(),
                }],
                cx,
            );
            v
        });
        view.update(cx, |v, cx| {
            assert_eq!(v.queue_bounce_seq, 0, "未撞首弹序为 0");
            v.queue_chip_bounce(cx);
            v.queue_chip_bounce(cx);
            assert_eq!(v.queue_bounce_seq, 2, "每轮注入序自增");
        });
        // 曲线端点落位（起点/终点≈0 · 中点=升幅）：静止外观与基础态一致。
        assert!(queue_chip_dy(0.0).abs() < 1e-4);
        assert!(queue_chip_dy(1.0).abs() < 1e-4);
        assert_eq!(queue_chip_dy(0.25), -QUEUE_CHIP_HOP_PX, "第一次中点达升幅");
    }

    // 入口标记＋剩余数（collect-queue-active · §13.2 ⑦）：镜像有存量才呈现
    //（空队列不渲染位），点击开队列详情层并呈现 placed 真数据。
    #[gpui::test]
    async fn queue_chip_appears_only_when_nonempty(cx: &mut gpui::TestAppContext) {
        use std::ops::Deref as _;
        let window = cx.open_window(gpui::size(gpui::px(900.0), gpui::px(560.0)), |_w, cx| {
            crate::init(cx);
            CaptureView::new(cx)
        });
        let mut cx = gpui::VisualTestContext::from_window(*window.deref(), cx);
        cx.run_until_parked();
        assert!(
            cx.debug_bounds("clp-chip-queue").is_none(),
            "空队列不渲染入口位"
        );
        window
            .update(&mut cx, |v, _w, cx| {
                // 宿主回推同式注入两快照（id 与序对位由 app 层保障）。
                v.set_queue_rows(
                    vec![
                        QueueRow {
                            id: 1,
                            text: "甲".into(),
                        },
                        QueueRow {
                            id: 2,
                            text: "乙".into(),
                        },
                    ],
                    cx,
                );
            })
            .unwrap();
        cx.run_until_parked();
        assert!(
            cx.debug_bounds("clp-chip-queue").is_some(),
            "队列持条目 → 入口标记呈现"
        );
        let chip = cx
            .debug_bounds("clp-chip-queue")
            .unwrap_or_else(|| panic!("入口标记未录制"));
        cx.simulate_click(chip.center(), Default::default());
        cx.run_until_parked();
        assert!(
            window.update(&mut cx, |v, _w, _cx| v.queue_open).unwrap(),
            "点击入口标记开队列详情层"
        );
        cx.run_until_parked();
        assert!(
            cx.debug_bounds("a1-qrow-1").is_some(),
            "placed 行真实呈现（镜像首项快照）"
        );
    }

    // 列表条目长文字溢出统一修复（DEC-229）：极长标题行（不可断中文串/URL/
    // 长词序列）不得撑出行右缘——文字应在可用宽度内换行（行高增长），meta
    // 徽标右缘收纳在行内衬内（右留白＝左内衬 16px 同值）。
    #[gpui::test]
    async fn long_title_rows_wrap_within_row(cx: &mut gpui::TestAppContext) {
        use std::ops::Deref as _;
        let long_cjk = "连续中文串超长测试".repeat(12);
        let long_url = "https://example.com/some/very/long/path/segments/joined/without/spaces-and-hyphens-endless-token-oooooooooooooooooooooooooo".to_string();
        let long_words = "alpha beta gamma delta epsilon zeta eta theta iota kappa".to_string();
        let window = cx.open_window(gpui::size(gpui::px(420.0), gpui::px(560.0)), |_w, cx| {
            crate::init(cx);
            let mut v = CaptureView::new(cx);
            v.rows = vec![
                ClipRow {
                    title: "短标题".to_string(),
                    ..Default::default()
                },
                ClipRow {
                    title: long_cjk.clone(),
                    meta: Some("01/02".to_string()),
                    ..Default::default()
                },
                ClipRow {
                    title: long_url.clone(),
                    kind: FilterKind::Url,
                    ..Default::default()
                },
                ClipRow {
                    title: long_words.clone(),
                    meta: Some("01/02".to_string()),
                    ..Default::default()
                },
            ];
            v
        });
        let mut cx = gpui::VisualTestContext::from_window(*window.deref(), cx);
        cx.run_until_parked();
        let short = cx
            .debug_bounds("clp-row-0")
            .unwrap_or_else(|| panic!("短标题行未录制"));
        let cjk = cx
            .debug_bounds("clp-row-1")
            .unwrap_or_else(|| panic!("长中文串行未录制"));
        let url = cx
            .debug_bounds("clp-row-2")
            .unwrap_or_else(|| panic!("长 URL 行未录制"));
        let words = cx
            .debug_bounds("clp-row-3")
            .unwrap_or_else(|| panic!("长词序列行未录制"));
        // ① 换行判据：长标题行随换行增高（短标题行为单行基准）。
        assert!(
            cjk.size.height > short.size.height,
            "长中文串行应换行增高（cjk={cjk:?} short={short:?}）"
        );
        assert!(
            url.size.height > short.size.height,
            "长 URL 行应换行增高（url={url:?} short={short:?}）"
        );
        assert!(
            words.size.height > short.size.height,
            "长词序列行应换行增高（words={words:?} short={short:?}）"
        );
        // ② 徽标收纳判据：meta 徽标右缘不越行右缘（右内衬 16px 以内）。
        for (k, row) in [(1usize, cjk), (3usize, words)] {
            let badge = cx
                .debug_bounds(Box::leak(format!("clp-row-badge-{k}").into_boxed_str()))
                .unwrap_or_else(|| panic!("徽标 {k} 未录制"));
            assert!(
                badge.left() <= row.right(),
                "徽标 {k} 应收纳在行内（badge={badge:?} row={row:?}）"
            );
            assert!(
                badge.right() <= row.right(),
                "徽标 {k} 右缘不得越行右缘（badge={badge:?} row={row:?}）"
            );
        }
    }

    // 拾记行文本三行截断（DEC-232）：超三行文本在第三行封顶省略号截断——
    // 行高增幅＝恰好两线差（≤2*行高+容差），不会无限换行撑破列表。
    #[gpui::test]
    async fn clip_row_text_clamps_three_lines(cx: &mut gpui::TestAppContext) {
        use std::ops::Deref as _;
        let long_text = "拾记文本超长截断测试串逐行计高。".repeat(30);
        let window = cx.open_window(gpui::size(gpui::px(420.0), gpui::px(560.0)), |_w, cx| {
            crate::init(cx);
            let mut v = CaptureView::new(cx);
            v.rows = vec![
                ClipRow {
                    title: "短句".to_string(),
                    ..Default::default()
                },
                ClipRow {
                    title: long_text.clone(),
                    meta: Some("01/02".to_string()),
                    ..Default::default()
                },
            ];
            v
        });
        let mut cx = gpui::VisualTestContext::from_window(*window.deref(), cx);
        cx.run_until_parked();
        let short = cx
            .debug_bounds("clp-row-0")
            .unwrap_or_else(|| panic!("短句行未录制"));
        let long = cx
            .debug_bounds("clp-row-1")
            .unwrap_or_else(|| panic!("长文本行未录制"));
        let line = short.size.height - gpui::px(23.0); // py 11×2 + 发丝线 1
        let diff = long.size.height - short.size.height;
        assert!(
            diff > line * 0.85 && diff <= line * 2.0 + gpui::px(8.0),
            "长文本应三行封顶（diff={diff:?} line={line:?}）"
        );
        // 徽标（meta 行）收纳在行内——meta 不被截断文本挤出。
        let badge = cx
            .debug_bounds("clp-row-badge-1")
            .unwrap_or_else(|| panic!("徽标未录制"));
        assert!(
            badge.right() <= long.right() && badge.left() >= long.left(),
            "徽标应在行内（badge={badge:?} row={long:?}）"
        );
    }
}
