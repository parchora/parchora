//! PG-PEEK 预览窗口（UI-4/6 · TASK-508 PeekView / REQ-PREVIEW-*）。
//!
//! HTML 1:1（PG-PEEK.html）：480×70% 内容展示面板 · bar
//! （文件名 + 复制 + 只读/编辑态「源码」切换钮 + 编辑/完成）→ 内容区（h3 标题 +
//! 可选正文段 + 图片棋盘卡 + 公式降级条）→ 底部导航行（↑↓ 同步 · Esc 级联第 2 级）。
//! 方角真窗语境下无 tertiary 左条（DEC-125）：2px 深褐全高通高贴外边缘
//! 呈现为「左侧黑色粗线」缺陷（根样式与 editor 详情窗一致）。
//!
//! 正文选择与复制（REQ-PREVIEW-SELECT-COPY）：跨段
//! 连续拖选 + 双击全段（SelectablePara 元素 · InteractiveText 同构）+ 复制钮 /
//! ⌘C；拖选移出窗口时悬停关闭暂缓、抬起补发（pending_close 状态机 · 生命周期
//! 不悬挂）。滚轮滚动合成伪 hover 退出由 app 层真光标守门承压（DEC-109）。
//!
//! 单击触发链（REQ-PREVIEW 单击口径）：
//! 三列表子视图行单击 → PeekEmitter（PreviewSink）→ PanelView（ShellEvent::
//! PeekShow）→ app 层开/原位更新 Peek 窗（单实例 set_item 内容直换）；Peek 窗
//! 自身关闭 = 指针移出 0.12s 对称宽限（app 层 arm_peek_close · peek_close_gen
//! 防争 · 光标回面板域内保持面板）或 Esc。
//! Esc 级联第 2 级退出：根 key_context + on_key_down → PeekEvent::Escape 上抛
//! （app 层只关 Peek，面板不动；§〇-4 / §十 · DEC-73）。
//!
//! 编辑转出（REQ-PREVIEW-IMAGE / SCN-PREVIEW-QUICKLOOK-EDIT-
//! HANDOFF · DEC-24）：「编辑」钮 / ⌘E → PeekEvent::EditHandoff 载条目上抛
//! （app 层关 Peek → 统一编辑器窗同址接手；Quick Look 自身不承载编辑）。编辑
//! 入口限定纯文本条目——演示口径数据驱动条目全为纯文本承载（真实内容类型判
//! 别器随 DB 接线 TASK-502/504 生效），静态演示态（item=None · --peek 1:1）
//! 按钮仅视觉承载不接线。
//!
//! item=None 渲染静态演示内容（--peek CLI 演示态，1:1 不回归）；item=Some 由
//! 单击条目驱动（演示正文多段占位，DB 接线 TASK-502/504 后真实）。真实 NSPanel
//! 非 key / 编辑器同渲染管线 / 缩放五通 defer（TASK-505，偏差台账登记）。
//!
//! 面板内编辑与 Markdown 阅读态（DEC-156）：
//! - 笔记（集锦）条目正文由 zorite-markdown 阅读视图 MarkdownView 承载
//!   （与编辑器同源渲染管线 · REQ-PREVIEW-PIPELINE——同 syntax 模块保证同文档
//!   同形态；渲染为 GPUI 原语零 HTML 注入面，URL 链接点击经 crate 内建
//!   is_safe_external_url 白名单（http/https/mailto）直开、危险协议 inert，
//!   wiki 链接无 handler 惰性；图片无 provider 降级可点击标签——与编辑器
//!   provider 未接线同口径）；Clip/Task 条目正文非文档语义（剪贴板原文/描述），
//!   保留纯文本段选择复制承载不变（REQ-PREVIEW-SELECT-COPY）。
//! - 只读态「源码」切换钮（DEC-158）：只读笔记条目 bar 增「源码」钮，渲染视图
//!   ↔ Markdown 源码文本互切（read_src 视图面标志 · 条目热换复位回默认渲染态；
//!   编辑态互斥切换仍由「源码」编辑钮 toggle_src_mode 承载；无「Peek · 非 key
//!   window」诊断标签，非 key 窗口行为特性不变）。
//! - 「编辑」钮/⌘E ＝面板内编辑态 toggle（默认路径；DEC-24 编辑转出非默认路径）：
//!   复用创建集锦编辑器（EditorShell）的 zorite 引擎装配三件套（MdState +
//!   Changed 活同步 + 外层自持滚动句柄光标跟随）与「源码」钮互斥挂载范式；
//!   退出编辑态（「完成」/⌘E/Esc）冲刷落盘——磁盘真实条目原地覆写
//!   （rewrite_note）+ 集锦列表同步（DEC-144 漏斗），演示/剪贴板条目不入盘
//!   （handoff「内容不入盘」同口径）；关窗路径（Esc/悬停宽限/级联/重泊/退出）
//!   经 app 层 close_peek 统一冲刷兜底（editor_exit 同构）。
//! - 保存失败面（DEC-159）：冲刷上抛失败（io 错误明文）→ bar 下错误条幅承载
//!   ＋失败条目寄存（pending_failed——关窗冲刷先重试）；软关窗路径（Esc/悬停
//!   宽限/单实例关 Peek）失败驻留不关窗（编辑态与在编缓冲原封，下次关窗重
//!   试）；强制关窗路径（级联/quit/换泊/编辑器转出）失败随宿主消亡（仅
//!   stderr 留档）。
//! - 编辑态挂起悬停收窗（DEC-94 Pin 同构）：编辑中指针移出不启动 0.12s 宽限，
//!   退出编辑时指针仍在窗外则照常补发关闭（生命周期不悬挂）；EditHandoff
//!   转出通道保留在册（无面板内触发点）。

use std::cell::Cell;
use std::rc::Rc;

use gpui::prelude::FluentBuilder as _;
use gpui::{
    div, point, px, relative, rgb, rgba, App, AppContext as _, Bounds, ClipboardItem, Context,
    CursorStyle, DispatchPhase, Div, Element, ElementId, Entity, Focusable as _, FontWeight,
    GlobalElementId, HighlightStyle, Hitbox, HitboxBehavior, HoverListenerMode, InspectorElementId,
    InteractiveElement as _, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent,
    ParentElement as _, Pixels, Render, ScrollHandle, StatefulInteractiveElement as _, Styled,
    StyledText, Subscription, Window,
};
use gpui_component::input::{Textarea, TextareaState};
use gpui_component::InteractiveElementExt as _;
use zorite_editor::{EditorEvent as MdEvent, EditorState as MdState};
use zorite_markdown::{
    Labels as MdLabels, MarkdownStyle, MarkdownView, SelectEvent, SelectHandler,
};

use crate::pages::editor::{self, md_labels, md_parchment_style, EditorEscape, MD_PLACEHOLDER};
use crate::{components as co, scale as s, tokens as t};

/// Peek 条目类别（三列表来源 · 单击承载）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PeekKind {
    /// 剪贴板条目（拾记）
    Clip,
    /// 知识条目（集锦）
    Note,
    /// 待办条目（待办）
    Task,
}

impl PeekKind {
    /// 详情 meta 前缀标签。
    pub const fn label(self) -> &'static str {
        match self {
            Self::Clip => "剪贴板内容",
            Self::Note => "笔记",
            Self::Task => "待办",
        }
    }
}

/// 单击预览条目（DEC-183：全部条目类别为真实数据承载——拾记行/待办行/磁盘笔
/// 记直通真实文本；静态演示态仅 --peek CLI 宿主）。
/// PartialEq = 同内容热换 no-op 判定（EdgeMark 内容热换协议第一律 · DEC-94）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PeekItem {
    /// 条目类别（详情 meta 前缀）
    pub kind: PeekKind,
    /// 标题（bar 文件名位 + h3；磁盘真实条目 = files 词干 = 落盘题名）
    pub title: String,
    /// 元信息（时间/来源/路径等；空则只显示类别标签）
    pub meta: String,
    /// 正文（Note 条目 = Markdown 源 · DEC-156 阅读态按文档语义渲染；Clip/Task
    /// = 纯文本原文）
    pub body: String,
    /// 磁盘真实条目（DEC-156 · 面板内编辑保存判据）：来自集锦磁盘扫描
    ///（collection::disk_note_item）；true ⇒ 面板内编辑退出/关窗冲刷时经保存漏
    /// 斗（save_peek_edit_at/rewrite_note · DEC-159）原地覆写；false ⇒
    /// 演示/剪贴板/待办条目内容不入盘（编辑窗 handoff「内容不入盘」同口径）。
    pub disk_real: bool,
}

/// 演示正文（DEC-183 · cfg(test) 限测试与 --peek 静态演示态调用面）：lead ＋
/// 10 段占位（保证 480~780 面板必然溢出 → 垂直滚动条口径可验）。
#[cfg(test)]
pub(crate) fn demo_body(lead: &str) -> String {
    let mut body = String::from(lead);
    for i in 0..10 {
        body.push_str(&format!(
            "\n\n第 {} 段（演示占位，DB 接线后为真实内容）：羊皮纸底、褐墨细字，正文 15px、行高 1.95，内容区以楷体呈现，未安装时回退系统默认（DEC-68）。",
            i + 1
        ));
    }
    body
}

/// 只读正文超长阈值（DEC-156 · REQ-PREVIEW 超长兜底）：笔记正文超过 200KB 时
/// 只读态按字符边界截断呈现（编辑态承载全文——zorite 引擎自理），免超长源于
/// 悬停面板逐帧重复解析。
const PEEK_MD_MAX_BYTES: usize = 200_000;
/// 保存失败条幅 io 错误明文预览截断宽（R184 建议 11 · DEC-159 64 字符承载）。
const SAVE_ERR_PREVIEW_CHARS: usize = 64;
/// Peek 阅读视图 Markdown 列表缩进步长（R184 建议 13 局部常量 · 现行 18px
/// 语义原样承载——仅字面量收敛，取值与视觉不变）。
const PEEK_LIST_INDENT_PTS: f32 = 18.0;

/// 只读正文超长截断（DEC-156）：> 阈值按字符边界剪切，返回（剪切后源，是否截
/// 断）；未超高零拷贝原样返回。解析异常兜底＝mdast 宽容解析（zorite-markdown
/// 语法层容错，非法语法按文本节点呈现不报错）。
fn truncated_read_body(src: &str) -> (&str, bool) {
    if src.len() <= PEEK_MD_MAX_BYTES {
        return (src, false);
    }
    let mut end = PEEK_MD_MAX_BYTES;
    while !src.is_char_boundary(end) {
        end -= 1;
    }
    (&src[..end], true)
}

/// 只读 Markdown 羊皮风格（DEC-156 · editor::md_parchment_style 同款令牌映射换
/// MarkdownStyle 承载）：主墨正文/标题、强调链接与行内码、WARM 半透码底、弱化
/// 引用与分隔线、生长绿标签、Menlo 等宽（DEC-68）；行高 1.45 与编辑引擎同比率
///（读-编辑两态切换不回流）；警示块取编辑器同语义令牌映射；围栏码块与编辑态
/// 同用藏青 60% 专用深卡＋纸光字（DEC-228，所见即所得）。
fn peek_md_style() -> MarkdownStyle {
    MarkdownStyle {
        text_color: rgb(t::PRIMARY).into(),
        text_size: px(s::font(s::Lvl::Body)),
        line_height: 1.45,
        heading_color: rgb(t::PRIMARY).into(),
        link_color: rgb(t::TERTIARY).into(),
        tag_color: rgb(t::GROWTH).into(),
        code_color: rgb(t::TERTIARY).into(),
        code_bg: rgba(t::WARM << 8 | 0x66).into(),
        fence_bg: rgba(t::CODE_NAVY << 8 | 0x99).into(),
        fence_fg: rgb(t::ON_PRIMARY).into(),
        muted_color: rgb(t::MUTED).into(),
        rule_color: rgba(t::BORDER_SOFT << 8 | 0xB0).into(),
        guide_color: rgba(t::BORDER_SOFT << 8 | 0x2A).into(),
        mark_bg: rgba(t::ACCENT << 8 | 0x40).into(),
        list_indent: px(PEEK_LIST_INDENT_PTS),
        mono_font: t::FONT_MONO.into(),
        alerts: zorite_markdown::AlertColors {
            note: rgb(t::TERTIARY).into(),
            tip: rgb(t::GROWTH).into(),
            important: rgb(t::TERTIARY).into(),
            warning: rgb(t::MUTED).into(),
            caution: rgb(t::MUTED).into(),
        },
        ..Default::default()
    }
}

/// 只读笔记正文视图（DEC-156 · REQ-PREVIEW-PIPELINE）：zorite-markdown
/// MarkdownView 阅读视图（与编辑器共享同 syntax 解析模块，同文档同形态）；URL
/// 链接点击 crate 内建 is_safe_external_url 白名单（http/https/mailto）直开、
/// 危险协议 inert，wiki 链接无 handler 惰性；渲染为 GPUI 原语零 HTML 注入面；
/// 图片无 provider 降级可点击标签（编辑器 provider 未接线同口径）。
fn read_md_view(src: &str) -> MarkdownView {
    MarkdownView::new("peek-md", src)
        .style(peek_md_style())
        .set_labels(MdLabels {
            code_copy: "复制".into(),
        })
}

/// 单击条目上抛通道（Rc：PanelView 一份 clone 注入三子视图 emitter）。
pub type PreviewSink = Rc<dyn Fn(&PeekItem, &mut App) + 'static>;

/// 子视图单击上抛器（三列表视图共用：单击条目 → PreviewSink → PanelView →
/// app 层开/原位更新统一详情窗；Default 供视图字段初始化）。
#[derive(Default)]
pub struct PeekEmitter {
    /// 事件上抛（宿主开/更新 Peek 窗）
    sink: Option<PreviewSink>,
}

impl PeekEmitter {
    /// 注入上抛通道（PanelView::new 统一转发 app 层）。
    pub fn set_sink(&mut self, sink: PreviewSink) {
        self.sink = Some(sink);
    }

    /// 单击条目上抛（app 层开窗/原位更新承载单实例口径）。
    pub fn show(&self, item: &PeekItem, cx: &mut App) {
        if let Some(f) = &self.sink {
            f(item, cx);
        }
    }
}

/// Peek 窗 → 宿主事件。
pub enum PeekEvent {
    /// 指针进入/离开预览窗（app 层维护对称宽限关闭）
    HoverChanged(bool),
    /// Esc 级联第 2 级退出（关 Peek，面板不动；§〇-4 / §十 · DEC-73）
    Escape,
    /// 编辑转出（REQ-PREVIEW-IMAGE · DEC-24）：承载条目移交统一
    /// 编辑器（app 层关 Peek → 编辑窗同址接手）。（编辑默认路径＝面板内承载
    /// · DEC-156；DEC-157 接线面板内唯一触发点＝超长截断条「在编辑器打开」
    /// 钮 · SCN-PREVIEW-QUICKLOOK-EDIT-HANDOFF 非默认路径在册承载）
    EditHandoff(PeekItem),
    /// 面板内编辑保存上抛（DEC-156）：退出编辑态（「完成」/⌘E/Esc）或
    /// 编辑中热换异题条目时承载已更新条目（live 正文 + disk_real 判据）；仅 body
    /// 有变时派发。app 层磁盘真实条目原地覆写落盘 + 集锦列表同步（DEC-144），
    /// 非磁盘条目不入盘（handoff 同口径）。
    SaveEdit(PeekItem),
    /// IME 组合态边沿（DEC-224 · R184 F-183-11 editor 窗镜像）：编辑混合态每帧
    /// 与 zorite 引擎 marked-text 存在性比对，翻转即上抛——`active` 为组合中
    ///（app 层即时降级键窗 NSWindowLevel 让候选浮层置顶）/ false 为组合结束或
    /// 引擎拆卸（恢复原 level）。内部通道，不改 Esc 漏斗语义。
    Composing { active: bool },
}

/// Peek 窗 → 宿主上抛通道。
pub type PeekSink = Box<dyn Fn(&PeekEvent, &mut App) + 'static>;

/// 关窗冲刷探测载体（DEC-159 · 只读不摘引擎）：app 层 close 冲刷前的预探——
/// edit ＝ 编辑在编时（含活正文条目，body 是否有变）；pending ＝ 前次保存失败
/// 寄存条目集（关窗冲刷逐项重试，成功逐项清除 · R184 F-183-08 Vec 化）。
pub struct PeekFlushProbe {
    /// 编辑在编探测（tear_down_edit 同取值面 · 非编辑态 None）
    pub edit: Option<(PeekItem, bool)>,
    /// 失败寄存条目集（mark_save_failed 逐次累加 · 多失败并存不覆写；
    /// 关窗冲刷重试源）
    pub pending: Vec<PeekItem>,
}

/// Peek 窗悬停会话态全局（PeekView 根 hover 写入 · app 层 MouseExitHide 抑制
/// 等读取）：指针在预览窗内时关闭路径转交 Peek 窗——预览可滚（SCN-PREVIEW-
/// SCROLL-IN-PREVIEW）。app 全局态为在册惯例（DEC-207：app 全局态＝
/// ClipCapture/PeekHover 惯例承载——R184 F-183-13 复核保留，非失控全局面）。
pub struct PeekHover {
    /// 指针是否在 Peek 窗内
    pub inside: bool,
}

impl gpui::Global for PeekHover {}

// ───────── 可选正文段（REQ-PREVIEW-SELECT-COPY）────

/// 段选择事件（SelectablePara 上抛 → PeekView 选择状态机）。
enum ParaSelEvent {
    /// 按下：单击记锚点（已钳字符边界）
    Down { para: usize, ix: usize },
    /// 双击全段选中（click_count ≥ 2）
    SelectAll { para: usize },
    /// 按住拖动更新选择头（跨段连续：会话内任一段命中均上抛；段号越界钳末段）
    Drag { para: usize, ix: usize },
    /// 抬起收口（无 hover 守卫——窗内任意位置抬起均送达，暂挂关闭必补发）
    Up,
}

/// 跨段连续选择状态（锚-头模型：按下记锚、拖动仅更新头；字节索引已钳字符边界，
/// 段序为 rendered_paras 下标 · REQ-PREVIEW-SELECT-COPY / DEC-109）。
#[derive(Clone, Debug, PartialEq, Eq)]
struct ParaSel {
    /// 锚点段（按下段）
    a_para: usize,
    /// 锚点字节（段内含）
    a_ix: usize,
    /// 头段（当前拖动点段）
    h_para: usize,
    /// 头字节（段内含）
    h_ix: usize,
}

/// 锚-头归一跨度（lo=(段,字节) → hi=(段,字节)）：头在锚前（反向拖动）则交换。
fn sel_span(sel: &ParaSel) -> (usize, usize, usize, usize) {
    if sel.h_para < sel.a_para || (sel.h_para == sel.a_para && sel.h_ix < sel.a_ix) {
        (sel.h_para, sel.h_ix, sel.a_para, sel.a_ix)
    } else {
        (sel.a_para, sel.a_ix, sel.h_para, sel.h_ix)
    }
}

/// 空选判定（塌缩：同段同字节——抬起收口清除依据）。
fn sel_collapsed(sel: &ParaSel) -> bool {
    sel.a_para == sel.h_para && sel.a_ix == sel.h_ix
}

/// 派生单段高亮区间（跨段连续选择 · 段序 para）：锚段=锚点→段尾（反向=0→锚
/// 点）、途经段=整段（0→len）、头段=头位→段尾（反向=0→头点，头位=段尾则空）；
/// 区间空 → None。
fn para_highlight(sel: &ParaSel, paras: &[String], para: usize) -> Option<std::ops::Range<usize>> {
    let (lo_p, lo_ix, hi_p, hi_ix) = sel_span(sel);
    if para < lo_p || para > hi_p {
        return None;
    }
    if lo_p == hi_p {
        return (lo_ix < hi_ix).then_some(lo_ix..hi_ix);
    }
    let len = paras.get(para).map_or(0, |s| s.len());
    let s = if para == lo_p { lo_ix.min(len) } else { 0 };
    let e = if para == hi_p { hi_ix.min(len) } else { len };
    (s < e).then_some(s..e)
}

/// 字节索引钳制到最近字符边界（CJK 多字节 · 高亮区间 char boundary 前置；
/// 越界取段末）。
fn clamp_char_boundary(s: &str, mut ix: usize) -> usize {
    if ix > s.len() {
        return s.len();
    }
    while !s.is_char_boundary(ix) {
        ix -= 1;
    }
    ix
}

/// 复制目标纯函数：选择区间优先（同段字节切片；跨段=锚段尾段 + 途经整段 +
/// 头段首段按段序以 \n 连接），无选择（含塌缩空选）→ 全段按行 join（可见即所
/// 得）；切片越界 / 空段落表 → None（复制动作静默）。
fn copy_target(paras: &[String], sel: Option<&ParaSel>) -> Option<String> {
    let Some(s) = sel.filter(|s| !sel_collapsed(s)) else {
        return (!paras.is_empty()).then(|| paras.join("\n"));
    };
    let (lo_p, lo_ix, hi_p, hi_ix) = sel_span(s);
    if lo_p == hi_p {
        return paras
            .get(lo_p)
            .and_then(|p| p.get(lo_ix..hi_ix))
            .map(str::to_string);
    }
    let mut parts: Vec<&str> = Vec::new();
    parts.push(paras.get(lo_p)?.get(lo_ix..)?);
    for p in paras.iter().take(hi_p).skip(lo_p + 1) {
        parts.push(p.as_str());
    }
    parts.push(paras.get(hi_p)?.get(..hi_ix)?);
    Some(parts.join("\n"))
}

/// 鼠标可选中正文段落元素（InteractiveText 同构 · vendor pristine 只读不改，
/// 镜像其 Element 形状）：拖选 / 双击全段 / IBeam 光标；正文样式由包裹容器
/// div 级联（StyledText 继承 window.text_style），元素自身零样式零子节点。
struct SelectablePara {
    id: ElementId,
    para: usize,
    text: StyledText,
    /// 拖选会话共享旗标（PeekView.selecting 的 Rc<Cell> 同源克隆）：任一段的
    /// Drag 守卫读此旗标 → 跨段连续上抛（DEC-109）
    drag_gate: Rc<Cell<bool>>,
    on_event: ParaSelSink,
}

/// 事件上抛通道（SelectablePara → PeekView · Rc 共享闭包）。
type ParaSelSink = Rc<dyn Fn(ParaSelEvent, &mut Window, &mut App)>;

/// 元素帧间状态：按下锚点索引（按下/抬起配对判据 · InteractiveText 同款）。
#[derive(Default)]
struct SelectableParaState {
    mouse_down_ix: Rc<Cell<Option<usize>>>,
}

impl SelectablePara {
    fn new(
        para: usize,
        text: StyledText,
        drag_gate: Rc<Cell<bool>>,
        on_event: ParaSelSink,
    ) -> Self {
        Self {
            id: ElementId::NamedInteger("peek-para".into(), para as u64),
            para,
            text,
            drag_gate,
            on_event,
        }
    }
}

impl Element for SelectablePara {
    type RequestLayoutState = ();
    type PrepaintState = Hitbox;

    fn id(&self) -> Option<ElementId> {
        Some(self.id.clone())
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (gpui::LayoutId, ()) {
        self.text.request_layout(None, inspector_id, window, cx)
    }

    fn prepaint(
        &mut self,
        global_id: Option<&GlobalElementId>,
        inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        state: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) -> Hitbox {
        window.with_optional_element_state::<SelectableParaState, _>(
            global_id,
            |sel_state, window| {
                let sel_state = sel_state.map(|s| s.unwrap_or_default());
                self.text
                    .prepaint(None, inspector_id, bounds, state, window, cx);
                let hitbox = window.insert_hitbox(bounds, HitboxBehavior::Normal);
                (hitbox, sel_state)
            },
        )
    }

    fn paint(
        &mut self,
        global_id: Option<&GlobalElementId>,
        inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _state: &mut (),
        hitbox: &mut Hitbox,
        window: &mut Window,
        cx: &mut App,
    ) {
        let para = self.para;
        let text_layout = self.text.layout().clone();
        window.with_element_state::<SelectableParaState, _>(
            global_id.expect("selectable para carries an id"),
            |sel_state, window| {
                let sel_state = sel_state.unwrap_or_default();
                // 可选文本光标暗示（系统文本习惯 · IBeam）。
                window.set_cursor_style(CursorStyle::IBeam, hitbox);

                // 按下（恒注册）：单击记锚点 / 双击全段（click_count ≥ 2）；点击
                // 段内 padding 等出界点取 index_for_position 最近索引钳制。
                let on_event = self.on_event.clone();
                let down_hitbox = hitbox.clone();
                let layout = text_layout.clone();
                let down_ix = sel_state.mouse_down_ix.clone();
                window.on_mouse_event(
                    move |event: &MouseDownEvent, phase, window: &mut Window, cx: &mut App| {
                        if phase != DispatchPhase::Bubble || !down_hitbox.is_hovered(window) {
                            return;
                        }
                        let ix = match layout.index_for_position(event.position) {
                            Ok(ix) => ix,
                            Err(ix) => ix,
                        };
                        if event.click_count >= 2 {
                            on_event(ParaSelEvent::SelectAll { para }, window, cx);
                        } else {
                            on_event(ParaSelEvent::Down { para, ix }, window, cx);
                        }
                        down_ix.set(Some(ix));
                    },
                );

                // 抬起（恒注册 · 无 hover 守卫）：窗内任意位置抬起均收口——拖选
                // 移出窗口暂挂的 pending_close 必然于抬起补发（关闭不悬挂）；无按
                // 下锚点的杂散抬起幂等跳过。
                let on_event = self.on_event.clone();
                let up_ix = sel_state.mouse_down_ix.clone();
                window.on_mouse_event(
                    move |_: &MouseUpEvent, phase, window: &mut Window, cx: &mut App| {
                        if phase != DispatchPhase::Bubble {
                            return;
                        }
                        if up_ix.take().is_some() {
                            on_event(ParaSelEvent::Up, window, cx);
                        }
                    },
                );

                // 拖选：按住移动更新选择头（跨段连续 · DEC-109）：会话旗标共享
                // （drag_gate）取代按下段独占——任一段命中均上抛 (para, ix)，视图
                // 层钳段派生区间；段间隙/非段落区头冻结（出界即停）。
                let on_event = self.on_event.clone();
                let drag_hitbox = hitbox.clone();
                let drag_gate = self.drag_gate.clone();
                let layout = text_layout.clone();
                window.on_mouse_event(
                    move |event: &MouseMoveEvent, phase, window: &mut Window, cx: &mut App| {
                        if phase != DispatchPhase::Bubble
                            || !drag_gate.get()
                            || event.pressed_button.is_none()
                            || !drag_hitbox.is_hovered(window)
                        {
                            return;
                        }
                        let ix = match layout.index_for_position(event.position) {
                            Ok(ix) => ix,
                            Err(ix) => ix,
                        };
                        on_event(ParaSelEvent::Drag { para, ix }, window, cx);
                    },
                );

                self.text
                    .paint(None, inspector_id, bounds, &mut (), &mut (), window, cx);

                ((), sel_state)
            },
        );
    }
}

impl gpui::IntoElement for SelectablePara {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

/// Peek 预览视图（数据驱动 + 静态演示回退）。
/// item=None：PG-PEEK.html 静态演示内容（--peek CLI 演示态 1:1）；
/// item=Some：单击触发条目全量承载（标题 + 类别·meta + 正文段 + 内容区内滚）。
pub struct PeekView {
    /// 悬停条目（None = 静态演示内容）
    item: Option<PeekItem>,
    /// 窗内悬停事件通道（app 层；--peek 演示宿主不注入则无事件）
    on_peek_event: Option<PeekSink>,
    /// 键盘焦点柄（Esc 级联第 2 级退出承载 · DEC-73）
    focus: gpui::FocusHandle,
    /// 跨段连续选择（None = 无选择；锚-头模型，字节已钳字符边界 · DEC-109）
    selection: Option<ParaSel>,
    /// 拖选会话旗标（Rc<Cell> 与 SelectablePara.drag_gate 同源共享：元素 Drag 守
    /// 卫跨段读此旗标；Down/SelectAll 置位、Up/清态复位 · 单一事实源）
    selecting: Rc<Cell<bool>>,
    /// 双击全段进行中（抬起前 Drag 不扰动全段选择）
    select_all: bool,
    /// 悬停关闭暂挂（拖选中移出窗口 → 抬起后补发 HoverChanged(false)）
    pending_close: bool,
    /// DEC-184 md 渲染视图拖选会话（源字节锚/头）：vendored on_select 事件流
    /// 驱动；非 md 面（纯文本段/源码视图）不经此。None＝无会话。
    md_sel_anchor: Option<usize>,
    md_sel_head: Option<usize>,
    /// 本帧渲染视图的源文本（截断后 · DEC-184 选区切片与 ⌘C 的单一事实源）。
    md_view_source: Option<String>,
    /// 本帧渲染段落（选择 / 高亮 / 复制单一事实源 · 每帧 render 刷新）
    rendered_paras: Vec<String>,
    /// 面板内编辑态（DEC-156）：true = 正文由 zorite 引擎（混合态）或整篇
    /// Textarea（源码态）承载；false = 只读（Note → MarkdownView 阅读视图 /
    /// Clip·Task 与静态演示 → 纯文本段）。
    editing: bool,
    /// 编辑态内部模式（EditorShell hybrid 同构）：true = 混合态（zorite 编辑即
    /// 渲染 · 默认）、false = 源码态（互斥挂载）。
    hybrid: bool,
    /// 只读态「源码」视图开关（DEC-158）：true = 笔记正文以 Markdown 源码文本
    /// 呈现（与渲染视图互切）；false = 渲染视图（默认）。仅只读笔记条目承载，
    /// 条目热换复位回合渲染默认态。
    read_src: bool,
    /// 混合态 zorite 引擎（编辑态 render 首帧延迟建 · DEC-156）。
    md_editor: Option<Entity<MdState>>,
    /// md 引擎事件订阅保活（Changed→条目正文活同步 / SelectionChanged→滚动跟随；
    /// 编辑态摘除即弃）。
    md_sub: Option<Subscription>,
    /// md 引擎 observe 兑底（host 结构性编辑只 notify 不派发 Changed · DEC-150
    /// 承载同源）。编辑态摘除即弃。
    md_obs: Option<Subscription>,
    /// IME 组合态边沿记忆（DEC-224 · R184 F-183-11 editor 窗镜像）：render 每帧
    /// 与 markdown 引擎 marked-text 存在性比对，翻转即上抛 PeekEvent::Composing；
    /// None＝无引擎（只读/源码态），上抛复位沿。
    ime_composing: Option<bool>,
    /// 源码态整篇 Textarea 引擎（与 md 引擎互斥挂载 · EditorShell 同构）。
    src_textarea: Option<Entity<TextareaState>>,
    /// 编辑态外层滚动视口自持句柄（键控句柄跨上下文不可达 · vendor
    /// fast/global_id.rs 实证；光标跟随写入通道 · EditorShell ed_scroll 同构）。
    edit_scroll: ScrollHandle,
    /// 编辑后光标跟随待验旗标（DEC-151 同构）：文本变更路径（键入/回车/粘贴只发
    /// Changed、undo/redo 仅 notify）在陈旧布局下同步跟随会越界 no-op——置位后
    /// 由 render 头经 on_next_frame 于新布局生效后执行。
    md_follow_pending: bool,
    /// 编辑基线正文（DEC-156）：进入编辑时钉住进入前条目正文（编辑中 canonical
    /// 已被 Changed/observe 活同步为活值，不能作变化比对基线）；退出冲刷时对基
    /// 线比对 ⇒ body 有变才 SaveEdit/落盘（免无差别重写覆 mtime）。
    edit_base_body: Option<String>,
    /// 指针当前是否窗内（根 on_hover 实时维护 · 编辑态移出挂起，退出编辑补发
    /// 判定源 · DEC-156）。
    pointer_inside: bool,
    /// 保存失败条幅文案（DEC-159）：Some ＝ bar 下错误条幅承载（io 错误明文 ·
    /// 截断）；同题保存成功即清，窗亡即弃。
    save_error: Option<String>,
    /// 保存失败寄存条目集（DEC-159 · 不静默丢失 · R184 F-183-08 Vec 化）：冲刷
    /// 上抛失败时逐次寄存（含失败时活正文），关窗冲刷优先重试；同题保存成功
    /// 即移除该条（多失败并存不互相覆写，全清时撤条幅）。
    pending_failed: Vec<PeekItem>,
}

impl PeekView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            item: None,
            on_peek_event: None,
            focus: cx.focus_handle(),
            selection: None,
            selecting: Rc::new(Cell::new(false)),
            select_all: false,
            pending_close: false,
            md_sel_anchor: None,
            md_sel_head: None,
            md_view_source: None,
            rendered_paras: Vec::new(),
            editing: false,
            hybrid: true,
            read_src: false,
            md_editor: None,
            md_sub: None,
            md_obs: None,
            ime_composing: None,
            src_textarea: None,
            edit_scroll: ScrollHandle::new(),
            md_follow_pending: false,
            edit_base_body: None,
            pointer_inside: true,
            save_error: None,
            pending_failed: Vec::new(),
        }
    }

    /// 悬停驱动构造（app 层 open_peek_item 注入条目 · R184 建议 12：字段表
    /// 双份手写收敛为 ..Self::new(cx) 转发——新增字段不再需两处同步）。
    pub fn with_item(item: PeekItem, cx: &mut Context<Self>) -> Self {
        Self {
            item: Some(item),
            ..Self::new(cx)
        }
    }

    /// 键盘焦点柄（app 层开窗后 window.focus 定向；Esc 事件经根 key_context 分派）。
    pub fn focus_handle(&self) -> gpui::FocusHandle {
        self.focus.clone()
    }

    /// 原位更新条目（Peek 单实例：换行单击仅换内容不重开窗）。同内容 no-op
    /// （不 notify 不重绘 · EdgeMark 内容热换协议第一律 · DEC-94）；返回是否
    /// 发生变更（回归取证面）。
    ///
    /// 编辑态热换门（DEC-156）：同题（同条目重单击）——保留在编缓冲返回 no-op
    ///（编辑中逐键活同步使 body 已非行数据原值，内容等值判定在此不适用）；异题
    ///（切换预览目标）——先冲刷在编条目（tear_down_edit，body 有变即 SaveEdit
    /// 上抛）再换帧读态（新面＝同新鲜开立语义，不补发关闭——悬停会话未建立）。
    pub fn set_item(&mut self, item: PeekItem, cx: &mut Context<Self>) -> bool {
        if self.editing {
            if self.item.as_ref().map(|i| i.title.as_str()) == Some(item.title.as_str()) {
                return false;
            }
            if let Some((flushed, changed)) = self.tear_down_edit(cx) {
                if changed {
                    self.fire_peek_event(&PeekEvent::SaveEdit(flushed), cx);
                }
            }
        }
        if self.item.as_ref() == Some(&item) {
            return false;
        }
        self.item = Some(item);
        // 换内容清选择态：渲染段序已换，旧字节区间引用失效（防复制到旧文）。
        self.clear_selection_state();
        // 换内容回只读渲染视图默认态（DEC-158）：源码开关是视图面标志非条目态，
        // 不随条目携带；同内容 no-op 路径早退不触碰（在编缓冲保留语义同源）。
        self.read_src = false;
        cx.notify();
        true
    }

    /// 选择状态全清（换条目共用；拖选会话一并终止）。
    fn clear_selection_state(&mut self) {
        self.selection = None;
        self.selecting.set(false);
        self.select_all = false;
        self.pending_close = false;
        self.md_sel_anchor = None;
        self.md_sel_head = None;
        self.md_view_source = None;
    }

    /// 注入窗内悬停事件通道（app 层对称宽限关闭决策）。
    pub fn set_event_sink(&mut self, sink: PeekSink) {
        self.on_peek_event = Some(sink);
    }

    /// 面板内编辑进出统一入口（DEC-156「编辑」钮/⌘E/Esc 同源）：未编
    /// 辑 → 进入（态置位 + 清选择会话 + 滚动句柄回顶；引擎 render 首帧延迟建 ·
    /// 建后即聚焦——静态演示条目 item=None 不进编辑，--peek 1:1 按钮仅视觉承载）。
    /// 编辑中 → 冲刷退出：body 有变即 SaveEdit 上抛（app 层磁盘真实条目原地覆写
    /// 与集锦列表同步，非磁盘条目不入盘 · handoff 同口径）；指针已在窗外则照常
    /// 补发悬停关闭（编辑态挂起解除 · 生命周期不悬挂）。
    fn toggle_edit(&mut self, cx: &mut Context<Self>) {
        if self.editing {
            if let Some((item, changed)) = self.tear_down_edit(cx) {
                if changed {
                    self.fire_peek_event(&PeekEvent::SaveEdit(item), cx);
                }
            }
            if !self.pointer_inside {
                self.fire_peek_event(&PeekEvent::HoverChanged(false), cx);
            }
        } else if self.item.is_some() {
            self.editing = true;
            // 编辑基线钉住（DEC-156）：进入前正文＝变化比对基线（canonical 在编辑
            // 中被活同步，不能自证变化）。
            self.edit_base_body = self.item.as_ref().map(|i| i.body.clone());
            self.edit_scroll.set_offset(point(px(0.0), px(0.0)));
            self.clear_selection_state();
        }
        cx.notify();
    }

    /// 编辑态摘除（DEC-156）：取走活正文（混合态 md 引擎 / 源码态 Textarea 实时
    /// 值）入 canonical 条目并与取值前比对，摘除引擎与订阅回落只读。不派发任何
    /// 事件——事件侧（SaveEdit 上抛 / 悬停关闭补发）由调用方按场景闭环
    ///（toggle_edit 补发；条目热换与关窗冲刷不补发——换目标＝新面 / 窗亡无谓）。
    /// 返回（被冲刷条目，body 是否有变）；非编辑态 None。
    pub fn tear_down_edit(&mut self, cx: &mut Context<Self>) -> Option<(PeekItem, bool)> {
        if !self.editing {
            return None;
        }
        let live = if self.hybrid {
            self.md_editor
                .take()
                .map(|ed| ed.read(cx).text().to_string())
        } else {
            self.src_textarea
                .take()
                .map(|st| st.read(cx).value().to_string())
        };
        // 变化比对基线（DEC-156）：进入编辑时钉住的正文——canonical 已被活同步
        // 为活值（与 live 恒等），以此比对恒 false，不能自证变化。
        let changed = match (self.edit_base_body.as_deref(), live.as_deref()) {
            (Some(base), Some(live_body)) => base != live_body,
            _ => false,
        };
        self.edit_base_body = None;
        if let (Some(item), Some(live_body)) = (self.item.as_mut(), live) {
            item.body = live_body;
        }
        self.md_sub = None;
        self.md_obs = None;
        self.md_follow_pending = false;
        self.editing = false;
        self.hybrid = true;
        self.item.clone().map(|item| (item, changed))
    }

    /// 关窗冲刷探测（DEC-159 · 只读不摘引擎不置位失败面）：编辑在编 → 引擎活
    /// 值（混合态 md 引擎 / 源码态 Textarea，与 tear_down_edit 同取值面）对编辑
    /// 基线比对，承载（含活正文条目，是否有变）；未编辑 → None。pending ＝ 保存
    /// 失败寄存条目（app 层关窗冲刷先重试）。探测零副作用——失败驻留路径由
    /// app 层据此决定不关窗，引擎/基线/在编缓冲原封。
    pub fn flush_probe(&self, cx: &App) -> PeekFlushProbe {
        let edit = if self.editing {
            let live = if self.hybrid {
                self.md_editor
                    .as_ref()
                    .map(|ed| ed.read(cx).text().to_string())
            } else {
                self.src_textarea
                    .as_ref()
                    .map(|st| st.read(cx).value().to_string())
            };
            // 变化比对基线（DEC-156 同口径）：进入编辑时钉住的正文——在编
            // canonical 可能已被活同步为活值，不能自证变化。
            let changed = match (self.edit_base_body.as_deref(), live.as_deref()) {
                (Some(base), Some(live_body)) => base != live_body,
                _ => false,
            };
            match (self.item.as_ref(), live) {
                (Some(item), Some(live_body)) => {
                    let mut with_live = item.clone();
                    with_live.body = live_body;
                    Some((with_live, changed))
                }
                _ => None,
            }
        } else {
            None
        };
        PeekFlushProbe {
            edit,
            pending: self.pending_failed.clone(),
        }
    }

    /// 保存失败面承载（DEC-159 · app 层经窗柄回写）：置 bar 下错误条幅文案
    ///（io 错误明文 · SAVE_ERR_PREVIEW_CHARS 截断承载）+ 寄存条目入集（多失
    /// 败并存逐次累加不覆写 · R184 F-183-08；关窗冲刷先重试——编辑不静默丢
    /// 失）。软关驻留与冲刷上抛两条失败路共本口。
    pub fn mark_save_failed(&mut self, item: PeekItem, err: &str, cx: &mut Context<Self>) {
        let msg: String = err.chars().take(SAVE_ERR_PREVIEW_CHARS).collect();
        let ellipsis = if msg.chars().count() < err.chars().count() {
            "…"
        } else {
            ""
        };
        self.save_error = Some(format!("保存失败：{msg}{ellipsis}"));
        self.pending_failed.push(item);
        cx.notify();
    }

    /// 保存失败面清除（DEC-159 · F-183-08 Vec 化）：同题落盘成功后由 app 层回
    /// 写（保存漏斗 Saved 分支）——寄存集中移除同题条目；其余失败仍寄存则保
    /// 留条幅（非空即最近失败面未尽），全清时撤下（异题寄存保留——旧条目仍未
    /// 落盘，关窗冲刷继续重试）。
    pub fn clear_save_failed(&mut self, title: &str, cx: &mut Context<Self>) {
        if !self.pending_failed.iter().any(|p| p.title == title) {
            return;
        }
        self.pending_failed.retain(|p| p.title != title);
        if self.pending_failed.is_empty() {
            self.save_error = None;
        }
        cx.notify();
    }

    /// 源码/混合互斥切换（DEC-156 · EditorShell toggle_mode 同构去 pending 化：
    /// peek 端热换经编辑态门整题冲刷，无在途 pending 承载面）：body ← 当前引擎活
    /// 值（canonical 连续不丢）→ 摘引擎与订阅 → 翻转；新引擎 render 首帧自
    /// canonical 建立并聚焦。
    fn toggle_src_mode(&mut self, cx: &mut Context<Self>) {
        if self.hybrid {
            if let Some(ed) = self.md_editor.take() {
                if let Some(item) = self.item.as_mut() {
                    item.body = ed.read(cx).text().to_string();
                }
            }
            self.md_sub = None;
            self.md_obs = None;
        } else if let Some(st) = self.src_textarea.take() {
            if let Some(item) = self.item.as_mut() {
                item.body = st.read(cx).value().to_string();
            }
        }
        self.hybrid = !self.hybrid;
        cx.notify();
    }

    /// 只读态源码切换（DEC-158 · 渲染视图 ↔ Markdown 源码文本互切 · bar「源码」
    /// 钮）：仅只读笔记条目承载（守卫与 md_src 同口径——非编辑态 + kind=Note）；
    /// 静态演示 / Clip·Task 与编辑态 no-op（编辑态内互斥切换由 toggle_src_mode
    /// 承载 · 编辑进出不重置本旗标）。
    fn toggle_read_src(&mut self, cx: &mut Context<Self>) {
        if self.editing || !self.item.as_ref().is_some_and(|i| i.kind == PeekKind::Note) {
            return;
        }
        self.read_src = !self.read_src;
        // 选择区间随视图切换失效（DEC-183：源码段表与渲染段表异源——残留区间
        // 引用陈旧下标会误高亮/误复制，切换即清零同 set_item 热换口径）。
        self.clear_selection_state();
        cx.notify();
    }

    /// 编辑态 zorite 引擎延迟初始化（DEC-156 · EditorShell ensure_md_engine 同构）：
    /// MdState 预填 canonical 条目正文 + 羊皮 SyntaxStyle + 中文 Labels；进入编
    /// 辑即聚焦（显式编辑意图承接 · DEC-155 FocusGained 恰激活即 IME 可达）。
    /// Changed → 条目正文活同步 + 光标跟随待验；SelectionChanged → 滚动跟随；
    /// observe 兑底（host 结构性编辑只 notify 不派发 Changed · 哈希比对承载）。
    fn ensure_md_engine(&mut self, w: &mut Window, cx: &mut Context<Self>) {
        if !self.editing || !self.hybrid || self.item.is_none() || self.md_editor.is_some() {
            return; // 编辑态混合分支才建；建后即返防重复建环
        }
        let body = self
            .item
            .as_ref()
            .map(|i| i.body.clone())
            .unwrap_or_default();
        let ed = cx.new(|cx| {
            let mut st = MdState::new(w, cx)
                .with_text(body)
                .with_placeholder(MD_PLACEHOLDER);
            st.set_markdown_style(md_parchment_style(), cx);
            st.set_labels(md_labels(), cx);
            st.focus(w, cx); // 进入编辑即聚焦（DEC-155 FocusGained 恰激活承 IME）
            st
        });
        let sub = cx.subscribe(
            &ed,
            |this: &mut Self, ed: Entity<MdState>, ev: &MdEvent, cx: &mut Context<Self>| match ev {
                MdEvent::Changed => {
                    let text = ed.read(cx).text().to_string();
                    if let Some(item) = this.item.as_mut() {
                        item.body = text;
                    }
                    // 键入/回车/粘贴只发 Changed（无 SelectionChanged）：置跟随
                    // 待验旗标，render 头于新布局生效后执行（DEC-151 同构）。
                    this.md_follow_pending = true;
                    this.follow_md_cursor(cx);
                    cx.notify();
                }
                MdEvent::SelectionChanged => this.follow_md_cursor(cx),
                _ => {}
            },
        );
        let obs = cx.observe(&ed, |this, ed, cx| {
            let next = ed.read(cx).text().to_string();
            if this.item.as_ref().map(|i| i.body.as_str()) != Some(next.as_str()) {
                if let Some(item) = this.item.as_mut() {
                    item.body = next;
                }
                this.md_follow_pending = true;
                cx.notify();
            }
        });
        self.md_editor = Some(ed);
        self.md_sub = Some(sub);
        self.md_obs = Some(obs);
        cx.notify();
    }

    /// 编辑态外层滚动视口光标跟随（DEC-156 · EditorShell follow_md_cursor 同构
    /// 换自持句柄）：md 引擎 caret 行锚（caret_screen_bounds 精确视觉行优先 /
    /// offset_screen_top 回退逻辑行顶）→ 编辑滚动句柄补偿，镜像行距带判定；源
    /// 码态整篇 Textarea 由其内滚自理，不进本路径。
    fn follow_md_cursor(&mut self, cx: &mut Context<Self>) {
        let Some(ed) = self.md_editor.as_ref() else {
            return;
        };
        let (caret_top, lh) = {
            let e = ed.read(cx);
            match e.caret_screen_bounds() {
                Some(b) => (b.top(), b.size.height),
                None => {
                    let Some(top) = e.offset_screen_top(e.cursor()) else {
                        return; // 首帧未布局
                    };
                    let lh = e
                        .row_layout()
                        .first()
                        .map(|row| row.1)
                        .unwrap_or_else(|| px(s::font(s::Lvl::Body) * 2.0));
                    (top, lh)
                }
            }
        };
        let viewport = self.edit_scroll.bounds();
        let offset = self.edit_scroll.offset();
        // caret 行顶在滚动内容坐标中的位置（off ≤ 0：−off＝已滚动量）。
        let caret_y = f32::from(caret_top - viewport.top() - offset.y);
        // 行距带判定（行底越视口底 → 底对齐；行顶越顶 → 顶对齐）。
        let h = f32::from(viewport.size.height);
        let vis = caret_y + f32::from(offset.y);
        let mut new_off = f32::from(offset.y);
        if vis + f32::from(lh) > h {
            new_off = -(caret_y - h + f32::from(lh));
        } else if vis < 0.0 {
            new_off = -caret_y;
        }
        new_off = new_off.clamp(-f32::from(self.edit_scroll.max_offset().y), 0.0);
        if new_off != f32::from(offset.y) {
            self.edit_scroll.set_offset(point(offset.x, px(new_off)));
        }
    }

    /// 上抛窗内事件（生产链：on_hover/on_key_down listener 于 PeekView 租借态内
    /// 调用 → app sink 同实体分支必须 defer 出借域；app 层回归测试同构直达本口）。
    pub fn fire_peek_event(&self, ev: &PeekEvent, cx: &mut App) {
        if let Some(f) = &self.on_peek_event {
            f(ev, cx);
        }
    }

    /// 超长截断条转出请求（DEC-157）：EditHandoff 面板内唯一触发点（截断行
    /// 「在编辑器打开」钮）→ 上抛 app 层关 Peek 起统一编辑器窗编辑全文。仅数据
    /// 条目承载（item 为 Some）；静态演示无源不渲染截断行，本口同守卫 no-op。
    fn request_edit_handoff(&mut self, cx: &mut Context<Self>) {
        if let Some(item) = self.item.clone() {
            self.fire_peek_event(&PeekEvent::EditHandoff(item), cx);
        }
    }

    /// 根 hover 复合处理（on_hover listener 唯一入口）：全局同步 + 拖选/编辑挂
    /// 起判定 + 事件上抛。编辑中移出不启动消失宽限（DEC-156 · DEC-94 Pin 同构：
    /// 编辑语义优先于收窗，退出编辑时若指针仍在外补发）；拖选中移出 → pending
    /// _close 暂挂不立即上抛关闭，抬起必补发（窗口生命周期不悬挂）；回窗均清暂
    /// 挂；非挂起路径语义不变。
    fn on_hover_event(&mut self, hovered: bool, cx: &mut Context<Self>) {
        cx.set_global(PeekHover { inside: hovered });
        self.pointer_inside = hovered;
        let event = if hovered {
            self.pending_close = false;
            Some(PeekEvent::HoverChanged(true))
        } else if self.editing {
            // 面板内编辑中移出（DEC-156）：收窗挂起，退出编辑（toggle_edit）时
            // 若指针仍在外照常补发。
            None
        } else if self.selecting.get() {
            // 拖选/全段选择进行中移出：关闭上抛暂缓，抬起补发（on_para_up）。
            self.pending_close = true;
            None
        } else {
            Some(PeekEvent::HoverChanged(false))
        };
        if let Some(event) = event {
            self.fire_peek_event(&event, cx);
        }
    }

    /// 选择状态机入口（SelectablePara 事件回收）：按下记锚、拖动更新头、
    /// 双击全段、抬起收口（含暂挂关闭补发）。
    fn on_para_event(&mut self, event: ParaSelEvent, cx: &mut Context<Self>) {
        match event {
            ParaSelEvent::Down { para, ix } => {
                let (para, ix) = self.clamp_para_point(para, ix);
                self.selection = Some(ParaSel {
                    a_para: para,
                    a_ix: ix,
                    h_para: para,
                    h_ix: ix,
                });
                self.selecting.set(true);
                self.select_all = false;
                cx.notify();
            }
            ParaSelEvent::SelectAll { para } => {
                let (para, end) = self.clamp_para_point(para, usize::MAX);
                self.selection = Some(ParaSel {
                    a_para: para,
                    a_ix: 0,
                    h_para: para,
                    h_ix: end,
                });
                self.selecting.set(true);
                self.select_all = true;
                cx.notify();
            }
            ParaSelEvent::Drag { para, ix } => {
                if self.select_all || !self.selecting.get() {
                    return; // 双击全段态不受拖动扰动；无按下会话忽略
                }
                let Some(sel) = self.selection.as_ref() else {
                    return;
                };
                // 跨段连续（DEC-109）：锚固定于 selection，拖动仅移动头；段号越
                // 界防御钳末段。头未动 → 派生区间不变 → 免 notify 免重绘。
                let (h_para, h_ix) = self.clamp_para_point(para, ix);
                if sel.h_para == h_para && sel.h_ix == h_ix {
                    return;
                }
                self.selection = Some(ParaSel {
                    a_para: sel.a_para,
                    a_ix: sel.a_ix,
                    h_para,
                    h_ix,
                });
                cx.notify();
            }
            ParaSelEvent::Up => self.on_para_up(cx),
        }
    }

    /// 抬起收口：终态化选择（空选清除）、清暂挂并补发关闭事件。
    fn on_para_up(&mut self, cx: &mut Context<Self>) {
        self.select_all = false;
        let collapsed = self.selection.as_ref().is_some_and(sel_collapsed);
        if collapsed {
            self.selection = None;
        }
        self.selecting.set(false);
        if self.pending_close {
            self.pending_close = false;
            self.fire_peek_event(&PeekEvent::HoverChanged(false), cx);
        }
        cx.notify();
    }

    /// md 渲染视图拖选会话状态机（DEC-184 · vendored SelectEvent 驱动 · 与段
    /// 选择机器共用 selecting 旗标与 pending_close 补发语义）：按下记锚、拖动
    /// 更新头、抬起终态化（空选即清除）。
    fn on_md_sel_event(&mut self, ev: SelectEvent, cx: &mut Context<Self>) {
        match ev {
            SelectEvent::Down(src) => {
                self.md_sel_anchor = Some(src);
                self.md_sel_head = Some(src);
                self.selecting.set(true);
                self.select_all = false;
                cx.notify();
            }
            SelectEvent::Drag(src) => {
                if !self.selecting.get() || self.md_sel_anchor.is_none() {
                    return; // 无会话忽略（如段选会话期间滑入 md 视图的杂散 Drag）
                }
                if self.md_sel_head == Some(src) {
                    return; // 头未动 → 免 notify 免重绘
                }
                self.md_sel_head = Some(src);
                cx.notify();
            }
            SelectEvent::Up => self.on_md_sel_up(cx),
        }
    }

    /// md 拖选抬起收口（段选 on_para_up 同构）：空选清除、清旗标与暂挂补发。
    fn on_md_sel_up(&mut self, cx: &mut Context<Self>) {
        let collapsed = match (self.md_sel_anchor, self.md_sel_head) {
            (Some(a), Some(h)) => a == h,
            _ => true,
        };
        if collapsed {
            self.md_sel_anchor = None;
            self.md_sel_head = None;
        }
        self.selecting.set(false);
        if self.pending_close {
            self.pending_close = false;
            self.fire_peek_event(&PeekEvent::HoverChanged(false), cx);
        }
        cx.notify();
    }

    /// md 选区规范化区间（start ≤ end · 无会话 → None）。
    fn md_sel_range(&self) -> Option<std::ops::Range<usize>> {
        let (a, h) = (self.md_sel_anchor?, self.md_sel_head?);
        if a != h {
            if a < h { a..h } else { h..a }.into()
        } else {
            None
        }
    }

    /// md 选区切片（DEC-184 ⌘C/复制钮承载）：源文本边界钳制（多字节安全 ·
    /// clamp_char_boundary 同款回退）；无会话/空选/无源文本 → None。
    fn md_selection_slice(&self) -> Option<String> {
        let range = self.md_sel_range()?;
        let src = self.md_view_source.as_ref()?;
        let s = clamp_char_boundary(src, range.start.min(src.len()));
        let e = clamp_char_boundary(src, range.end.min(src.len()));
        (s < e).then(|| src[s..e].to_string())
    }

    /// 拖动点钳制：段号越界钳至首/末段（跨段拖出内容域即停），字节越界取段末
    /// 最近字符边界。
    fn clamp_para_point(&self, para: usize, ix: usize) -> (usize, usize) {
        if self.rendered_paras.is_empty() {
            return (0, 0);
        }
        let para = para.min(self.rendered_paras.len() - 1);
        let ix = clamp_char_boundary(&self.rendered_paras[para], ix);
        (para, ix)
    }

    /// 复制承载（复制钮 / ⌘C 单一口径 · DEC-183 选择区间优先化）：编辑态 → 条目
    /// 正文全文（Markdown 源；活值经 Changed 同步 · DEC-156）；拖选区间非空 →
    /// 选段字节切片（源码视图/纯文本段同机器）；笔记条目无选段 → 条目正文全文
    ///（阅读渲染视图不可选 · vendor 约束）；纯文本段 → 选择区间优先，无选择 →
    /// 全渲染段；无目标静默 no-op。
    fn copy_selection(&mut self, cx: &mut Context<Self>) {
        if self.editing {
            if let Some(item) = &self.item {
                // 写前登记（DEC-163 · REQ-CAPTURE-SELF-SKIP）：自写板不回捕获。
                crate::clipstamp::mark(&item.body);
                cx.write_to_clipboard(ClipboardItem::new_string(item.body.clone()));
            }
            return;
        }
        // 拖选区间优先（DEC-183 · DEC-184 扩展）：任一条目类别、选段非空即
        // 复制选段——md 渲染视图选区（源切片）优先于段选机器。
        if let Some(text) = self.md_selection_slice() {
            crate::clipstamp::mark(&text);
            cx.write_to_clipboard(ClipboardItem::new_string(text));
            return;
        }
        let paras = self.rendered_paras.clone();
        if let Some(text) = copy_target(&paras, self.selection.as_ref()) {
            crate::clipstamp::mark(&text);
            cx.write_to_clipboard(ClipboardItem::new_string(text));
            return;
        }
        if self.item.as_ref().is_some_and(|i| i.kind == PeekKind::Note) {
            if let Some(item) = &self.item {
                crate::clipstamp::mark(&item.body);
                cx.write_to_clipboard(ClipboardItem::new_string(item.body.clone()));
            }
            return;
        }
        if let Some(text) = copy_target(&paras, None) {
            crate::clipstamp::mark(&text);
            cx.write_to_clipboard(ClipboardItem::new_string(text));
        }
    }

    /// 可选正文段落（co::content_text 同款文本样式承载 SelectablePara）；
    /// 选中区间以 WARM 选中底高亮（tokens 既有 token · 零新增）。
    fn selectable_para(&self, para: usize, text: String, cx: &mut Context<Self>) -> Div {
        let selected = self
            .selection
            .as_ref()
            .and_then(|s| para_highlight(s, &self.rendered_paras, para));
        let styled = match selected {
            Some(range) => StyledText::new(text).with_highlights([(
                range,
                HighlightStyle {
                    background_color: Some(co::h(t::WARM)),
                    ..Default::default()
                },
            )]),
            None => StyledText::new(text),
        };
        let view = cx.entity();
        let on_event: ParaSelSink = Rc::new(
            move |event: ParaSelEvent, _window: &mut Window, cx: &mut App| {
                view.update(cx, |this, cx| this.on_para_event(event, cx));
            },
        );
        div()
            .font_family(t::FONT_CONTENT)
            .font_weight(FontWeight(t::FONT_WEIGHT_LIGHT))
            .text_size(s::px_of(s::Lvl::Tab))
            .line_height(relative(t::LH_BODY))
            .text_color(rgb(t::PRIMARY))
            .child(SelectablePara::new(
                para,
                styled,
                self.selecting.clone(),
                on_event,
            ))
    }

    /// 双色棋盘演示卡（D-07：1 行 ×8 双色条纹 · 每条 30px）。
    /// 源码视图可选正文（DEC-183 · REQ-PREVIEW-SELECT-COPY 拖选承载）：
    /// 敯型源文本逐字入可选段（SelectablePara 单段多行 · 同 mono 样式）——笔记
    /// 阅读渲染视图受 vendored zorite-markdown 选择能力约束不可选（规范 6 · 披
    /// 露在册），源码视图与 Clip·Task 纯文本段同享拖选/双击/⌘C 机器。
    fn src_selectable(&self, src: &str, cx: &mut Context<Self>) -> Div {
        self.selectable_para(0, src.to_string(), cx)
            .font_family(t::FONT_MONO)
            .text_size(px(s::font(s::Lvl::Small)))
            .text_color(rgb(t::PRIMARY))
            .line_height(relative(t::LH_BODY))
    }

    fn chess_card(&self) -> Div {
        let mut grid = div()
            .border_1()
            .border_color(rgb(t::BORDER_SOFT))
            .rounded(px(t::RAD_MD))
            .overflow_hidden()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap(px(6.0));
        let mut stripes = div().flex().w(px(240.0)).h(px(48.0));
        for i in 0..8 {
            let even = i % 2 == 0;
            stripes = stripes.child(div().flex_1().h_full().bg(rgb(if even {
                t::NEUTRAL
            } else {
                t::BORDER_SOFT
            })));
        }
        grid = grid
            .child(stripes)
            .child(co::meta("图片：fit ≤100% 初始 · 缩放上限 16x · 棋盘格恒定"))
            .child(co::meta(
                "五通道缩放：捏合 / ⌘滚轮 / ⌥滚轮 / ⌘= ⌘− ⌘0 / 双击",
            ));
        grid
    }
}

impl Render for PeekView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let _s = crate::scale::enter(window, cx);
        // 编辑态引擎延迟初始化（DEC-156 · render 头 = EditorShell 同构）：源码态
        // 建整篇 Textarea（canonical 初值 + 占位 + 聚焦），混合态经
        // ensure_md_engine 建 zorite 引擎（进编辑即聚焦）；编辑后光标跟随待验经
        // on_next_frame 延帧（DEC-151 同构）。只读态零建（MarkdownView 无态元素）。
        if self.editing {
            if !self.hybrid && self.src_textarea.is_none() && self.item.is_some() {
                let init = self
                    .item
                    .as_ref()
                    .map(|i| i.body.clone())
                    .unwrap_or_default();
                let st = cx.new(|cx| {
                    let mut state =
                        TextareaState::new(window, cx).auto_grow(1, editor::SRC_AUTO_GROW_MAX_ROWS);
                    state.set_placeholder(MD_PLACEHOLDER, window, cx);
                    state
                });
                st.update(cx, |state, cx| state.set_value(init, window, cx));
                window.focus(&st.read(cx).focus_handle(cx), cx);
                self.src_textarea = Some(st);
            }
            if self.hybrid {
                self.ensure_md_engine(window, cx);
            }
            if self.md_follow_pending {
                self.md_follow_pending = false;
                let weak = cx.entity().downgrade();
                window.on_next_frame(move |_, app| {
                    if let Some(view) = weak.upgrade() {
                        view.update(app, |this, cx| this.follow_md_cursor(cx));
                    }
                });
            }
        }
        // IME 候选窗置顶（DEC-224 · R184 F-183-11 editor 窗镜像三件之三）：每帧
        // 直读 zorite 引擎 marked-text 存在性（引擎仅编辑混合态在场——拆卸/
        // 退出编辑沿报 None，翻转即上抛 Composing 复位键窗 level）。只读不写、
        // 无重入——render 期直调安全（editor 同构）。
        let composing = self.md_editor.as_ref().map(|md| {
            md.update(cx, |st, cx| {
                gpui::EntityInputHandler::marked_text_range(st, window, cx).is_some()
            })
        });
        if composing != self.ime_composing {
            let ev = match composing {
                Some(active) => PeekEvent::Composing { active },
                // 引擎拆卸（退出编辑/切源码态）：报组合结束复位键窗 level。
                None => PeekEvent::Composing { active: false },
            };
            self.ime_composing = composing;
            self.fire_peek_event(&ev, cx);
        }
        // 静态演示（--peek 1:1）保留（DEC-183：棋盘卡/公式降级条仅静态演示态
        // 承载）；笔记条目阅读态 = Markdown 阅读视图（DEC-156），其余 = 纯文本段；
        // 编辑态 = 自持滚动文档流。
        let (bar_label, title, meta_line, show_extras) = match &self.item {
            Some(item) => (
                item.title.clone(),
                item.title.clone(),
                if item.meta.is_empty() {
                    item.kind.label().to_string()
                } else {
                    format!("{} · {}", item.kind.label(), item.meta)
                },
                false,
            ),
            None => (
                "羊皮纸视觉系统推导.md".to_string(),
                "羊皮纸视觉系统推导".to_string(),
                String::new(),
                true,
            ),
        };
        let editing = self.editing;
        // 笔记条目阅读态源（DEC-156）：超长兑底截断（>200KB 字符边界剪切 · 编辑态
        // 承载全文）；Clip/Task/静态演示不进此路（非文档语义保留纯文本段）。
        let md_src = match self.item.as_ref() {
            // 借用一次截断（R184 F-183-10）：直借条目正文判边界剪切，免整篇
            // clone 中转（原 body.clone() 后再 to_string 双次拷贝收敛为截断
            // 结果一次成串）；超长兑底截断语义不变（编辑态承载全文）。
            Some(i) if !editing && i.kind == PeekKind::Note => {
                let (src, truncated) = truncated_read_body(&i.body);
                Some((src.to_string(), truncated))
            }
            _ => None,
        };
        // 只读纯文本段路径（静态演示 / Clip·Task）：正文行拆段；编辑/笔记阅读态
        // 段表清空——选择复制承载转向条目正文复制（copy_selection · DEC-156）。
        let paragraphs: Vec<String> = if md_src.is_some() {
            Vec::new()
        } else if let Some(item) = &self.item {
            if editing {
                Vec::new()
            } else {
                item.body
                    .lines()
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
            }
        } else if editing {
            Vec::new() // 不可达（item=None 不进编辑 · enter 门），防御承载
        } else {
            vec!["渲染与编辑器同一管线：GFM + 代码高亮 + LaTeX。".to_string()]
        };
        // 本帧渲染段落 = 选择 / 复制 / 高亮单一事实源（选择字节区间与本帧渲染
        // 字符串同序同源；条目热换经 set_item 清选择防陈旧区间）。
        self.rendered_paras = paragraphs.clone();

        let bar = co::bar()
            .child(co::txt_f(
                // 标题显示截断（DEC-235/237 阈值 20）：同编辑窗标题栏口径。
                co::detail_title_truncate(&bar_label),
                s::font(s::Lvl::Small),
                t::PRIMARY,
                t::FONT_CONTENT,
                t::FONT_WEIGHT_REGULAR,
            ))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.0))
                    .child(
                        co::ibtn("复制")
                            .id("peek-copy")
                            .on_click(cx.listener(
                                |this,
                                 _ev: &gpui::ClickEvent,
                                 _w: &mut Window,
                                 cx: &mut Context<Self>| {
                                    this.copy_selection(cx);
                                },
                            ))
                            .active(co::act_border),
                    )
                    // 只读态「源码」切换钮（DEC-158）：仅只读笔记条目挂载（md_src
                    // 有值同口径）；渲染视图＝未选中（ibtn 基线），源码视图＝选中
                    //（主墨底＋纸光反白 · DEC-124 背景轴同款 · 三色与编辑态源码钮
                    // 同 palette）；hover 已在组件内单次注册，不另挂。
                    .when(!editing && md_src.is_some(), |el| {
                        let read_src = self.read_src;
                        let sel = editor::EditorShell::mode_palette(read_src);
                        let mut btn =
                            co::ibtn_icon_text_bg(co::Icon::Code, "源码", sel.0, sel.1, sel.2)
                                .id("peek-read-src")
                                .on_click(cx.listener(
                                    |this,
                                     _ev: &gpui::ClickEvent,
                                     _w: &mut Window,
                                     cx: &mut Context<Self>| {
                                        this.toggle_read_src(cx);
                                    },
                                ));
                        btn = if read_src {
                            btn.active(|s| s.bg(rgb(t::PRIMARY)))
                        } else {
                            btn.active(co::act_border)
                        };
                        el.child(btn)
                    })
                    // 「源码」模式切换钮（DEC-156 · EditorShell 同构）：仅编辑态挂
                    // 挂载；混合态＝未选中（ibtn 基线），源码态＝选中（主墨底＋纸光
                    // 反白 · DEC-124 背景轴同款）；hover 已在组件内单次注册，不另挂。
                    .when(editing, |el| {
                        let src = !self.hybrid;
                        let sel = editor::EditorShell::mode_palette(src);
                        let mut btn =
                            co::ibtn_icon_text_bg(co::Icon::Code, "源码", sel.0, sel.1, sel.2)
                                .id("peek-mode")
                                .on_click(cx.listener(
                                    |this,
                                     _ev: &gpui::ClickEvent,
                                     _w: &mut Window,
                                     cx: &mut Context<Self>| {
                                        this.toggle_src_mode(cx);
                                    },
                                ));
                        btn = if src {
                            btn.active(|s| s.bg(rgb(t::PRIMARY)))
                        } else {
                            btn.active(co::act_border)
                        };
                        el.child(btn)
                    })
                    .child(
                        // 编辑进出 toggle（DEC-156）：编辑中＝「完成」（退出并保存）。
                        co::ibtn(if editing { "完成" } else { "编辑" })
                            .id("peek-edit")
                            .on_click(cx.listener(
                                |this,
                                 _ev: &gpui::ClickEvent,
                                 _w: &mut Window,
                                 cx: &mut Context<Self>| {
                                    this.toggle_edit(cx);
                                },
                            ))
                            .active(co::act_border),
                    ),
            );

        // 内容二分支（DEC-156）：编辑态＝自持滚动文档流；只读态＝既有键控滚动
        //（Note 走 MarkdownView，其余纯文本段保持不变）。
        let content = if editing {
            // ── 编辑态（EditorShell ed-scroll 同构）：自持滚动句柄承载文档流（标
            // 题/元信息随文滚动 · 单一滚动域）；正文点击聚焦补偿（DEC-155 同源缺
            // 陷的面板内预防：根 div track_focus 的 mousedown 自动聚焦会夺 zorite
            // 焦点 → handler 不注册 → IME 废）。
            div()
                .id("peek-edit-scroll")
                .flex_1()
                .min_h_0()
                .relative()
                .overflow_hidden()
                .child(
                    div()
                        .id("peek-edit-area")
                        .size_full()
                        .flex()
                        .flex_col()
                        .track_scroll(&self.edit_scroll)
                        .overflow_y_scroll()
                        .lock_scroll_axis()
                        .child(
                            div()
                                .id("peek-edit-content")
                                .flex_none()
                                .h_auto()
                                .min_h_full()
                                .flex()
                                .flex_col()
                                .gap(px(10.0))
                                .px(px(16.0))
                                .pt(px(16.0))
                                .pb(px(16.0))
                                .capture_any_mouse_down(cx.listener(
                                    |this, ev: &MouseDownEvent, w, cx: &mut Context<Self>| {
                                        if ev.button != MouseButton::Left {
                                            return;
                                        }
                                        if this.hybrid {
                                            if let Some(ed) = this.md_editor.clone() {
                                                let fh = ed.read(cx).focus_handle(cx);
                                                w.focus(&fh, cx);
                                            }
                                        } else if let Some(st) = this.src_textarea.clone() {
                                            w.focus(&st.read(cx).focus_handle(cx), cx);
                                        }
                                        w.prevent_default();
                                    },
                                ))
                                .child(co::txt_f(
                                    title,
                                    s::font(s::Lvl::H2),
                                    t::PRIMARY,
                                    t::FONT_CONTENT,
                                    t::FONT_WEIGHT_LIGHT,
                                ))
                                .when(!meta_line.is_empty(), |el| {
                                    el.child(co::meta(meta_line.clone()))
                                })
                                .when(self.hybrid, |el| {
                                    let ed = self
                                        .md_editor
                                        .as_ref()
                                        .expect("编辑态混合分支 md 引擎应于 render 头部初始化");
                                    el.child(
                                        div()
                                            .debug_selector(|| "peek-md-edit".to_string())
                                            .w_full()
                                            .min_w_0()
                                            // wrapper 级联（EditorShell md_zone 同构）：
                                            // 正文 Kaiti SC 15px 主墨——引擎继承环境文
                                            // 字风格（zorite README）。
                                            .font_family(t::FONT_CONTENT)
                                            .text_size(px(s::font(s::Lvl::Body)))
                                            .text_color(rgb(t::PRIMARY))
                                            .child(ed.clone()),
                                    )
                                })
                                .when(!self.hybrid, |el| {
                                    let st = self.src_textarea.as_ref().expect(
                                        "编辑态源码分支 textarea 引擎应于 render 头部初始化",
                                    );
                                    el.child(
                                        div()
                                            .debug_selector(|| "peek-src-edit".to_string())
                                            .flex_1()
                                            .min_h_0()
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
                                    )
                                }),
                        ),
                )
        } else {
            // ── 只读态（原口径 + Note 走 MarkdownView · DEC-156）。
            let mut read = div()
                .id("peek-scroll")
                .flex_1()
                .min_h_0()
                .overflow_y_scroll()
                .p(px(16.0))
                .flex()
                .flex_col()
                .gap(px(10.0))
                .child(co::txt_f(
                    title,
                    s::font(s::Lvl::H2),
                    t::PRIMARY,
                    t::FONT_CONTENT,
                    t::FONT_WEIGHT_LIGHT,
                ));
            // meta 行：仅数据驱动条目显示（静态演示 1:1 不回归——原稿无 meta 行）。
            if !meta_line.is_empty() {
                read = read.child(co::meta(meta_line.clone()));
            }
            match md_src {
                Some((src, truncated)) => {
                    // 「源码」只读切换（DEC-158 ＋ DEC-183 拖选承载）：源码视图＝单段
                    // SelectablePara 源文本承载（多行 \n 同源折行 — 空行/行高节奏
                    // 不变，选段字节切片与源逐字一致 · DEC-109 拖选机器复用）；渲
                    // 染视图＝既有 MarkdownView。选择/复制事实源：本帧段表＝[src]。
                    if self.read_src {
                        self.rendered_paras = vec![src.clone()];
                        read = read.child(
                            div()
                                .debug_selector(|| "peek-src-read".to_string())
                                .w_full()
                                .min_w_0()
                                .font_family(t::FONT_MONO)
                                .text_size(px(s::font(s::Lvl::Small)))
                                .text_color(rgb(t::PRIMARY))
                                .line_height(relative(t::LH_BODY))
                                // 克隆仅在源码视图渲染路径（截断兑底条仍需借用 src）。
                                .child(self.src_selectable(&src, cx)),
                        );
                    } else {
                        // md 渲染视图（DEC-184 可选文本）：vendored on_select 事件
                        // 流锚定本实体状态机；selection 高亮由 vendored 层按源
                        // 区间投影。本帧源文本缓存（截断后 src）＋段表清空（段选
                        // 机器事实源归位，⌘C 走 md 切片优先分支）。
                        self.md_view_source = Some(src.clone());
                        self.rendered_paras = Vec::new();
                        let view = cx.entity();
                        let handler: SelectHandler =
                            Rc::new(move |ev: SelectEvent, _window: &mut Window, cx: &mut App| {
                                view.update(cx, |this, cx| this.on_md_sel_event(ev, cx));
                            });
                        read = read.child(
                            div()
                                .debug_selector(|| "peek-md-read".to_string())
                                .w_full()
                                .min_w_0()
                                .font_family(t::FONT_CONTENT)
                                .child(
                                    read_md_view(&src)
                                        .on_select(handler)
                                        .selection(self.md_sel_range()),
                                ),
                        );
                    }
                    // 超长兑底（REQ-PREVIEW 超长兜底 · DEC-156）：截断行＝提示 +
                    // 「在编辑器打开」转出钮（DEC-157：EditHandoff 面板内唯一触发
                    // 点 · SCN-PREVIEW-QUICKLOOK-EDIT-HANDOFF 非默认路径承载）；
                    // 空内容＝阅读引导占位（md 视图空渲染）。
                    if truncated {
                        read = read.child(
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .gap(px(10.0))
                                // 截断阈值文案由常量派生（R184 建议 10 · SI kB=1000B
                                // ——200_000 B = 200 kB），字面量不再可漂移。
                                .child(co::meta(format!(
                                    "正文超长已截断（>{}KB）· 编辑查看全文",
                                    PEEK_MD_MAX_BYTES / 1000
                                )))
                                .child(
                                    co::ibtn("在编辑器打开")
                                        .id("peek-handoff")
                                        .on_click(cx.listener(
                                            |this,
                                             _ev: &gpui::ClickEvent,
                                             _w: &mut Window,
                                             cx: &mut Context<Self>| {
                                                this.request_edit_handoff(cx);
                                            },
                                        ))
                                        .active(co::act_border),
                                ),
                        );
                    } else if src.is_empty() {
                        read = read.child(co::meta("空笔记 · 编辑写入正文"));
                    }
                }
                None => {
                    for (para, p) in paragraphs.into_iter().enumerate() {
                        read = read.child(self.selectable_para(para, p, cx));
                    }
                }
            }
            if show_extras {
                read = read.child(self.chess_card()).child(
                    div()
                        .bg(rgb(t::WARM))
                        .rounded(px(t::RAD_SM))
                        .px(px(12.0))
                        .py(px(8.0))
                        .child(co::txt_f(
                            "公式失败=整体灰底源码 +「公式未渲染」角标（同编辑器降级终态）",
                            s::font(s::Lvl::Small),
                            t::MUTED,
                            t::FONT_MONO,
                            t::FONT_WEIGHT_LIGHT,
                        )),
                );
            }
            read
        };

        div()
            .id("peek-root")
            .key_context("Peek")
            .track_focus(&self.focus)
            .flex_1()
            .min_h_0()
            .flex()
            .flex_col()
            .bg(rgb(t::SURFACE))
            // 方角真窗语境下无 tertiary 左条（DEC-125）：2px 深褐全高通高贴外
            // 边缘＋贴窗影呈现为「左侧黑色粗线」，根样式与 editor 详情窗
            //（方角细节窗 · 无左条）一致。
            // 键盘模态独立（REQ-PREVIEW-SCROLL-STABILITY · DEC-107）：gpui 默认键盘
            // 输入抑制 hover——预览窗内任意键入（Esc/⌘C/方向键滚动）会伪翻转
            // on_hover(false) → arm_peek_close 收 Peek 并级联收面板。显隐判定只认
            // 「鼠标真离开」：模态过滤关闭，键盘滚动/键入不触显隐。
            .hover_listener_mode(HoverListenerMode::InputModalityIndependent)
            // 窗内悬停会话态（PeekHover 全局实时写 + app 层对称宽限事件上抛）：
            // 指针在预览窗内 → 悬停关闭挂起，预览可滚不被打断（REQ-PREVIEW-NOKEY）；
            // 拖选移出暂缓、抬起补发（on_hover_event 统一状态机）。
            .on_hover(cx.listener(
                |this, hovered: &bool, _w: &mut Window, cx: &mut Context<Self>| {
                    this.on_hover_event(*hovered, cx);
                },
            ))
            // Esc action 通道（DEC-156 · DEC-151 同构）：编辑态 zorite 聚焦后
            // escape→EditorEscape 绑定（parchora_ui::init 全局注册 · Editor
            // context 后注册优先接管 Dismiss）在 action 相位默认截停，下方
            // keydown 分支不可达——此处承接编辑态退出（完成并保存）。
            .on_action(cx.listener(
                |this, _: &EditorEscape, _w: &mut Window, cx: &mut Context<Self>| {
                    if this.editing {
                        this.toggle_edit(cx);
                    }
                    cx.stop_propagation();
                },
            ))
            // Esc：编辑态＝完成并保存（面板内编辑退出 · DEC-156）；只读态＝级联
            // 第 2 级退出——只关 Peek，面板不动（§〇-4 / §十 · DEC-73）；zc：复制
            // 选择（与复制钮同口 · copy_selection 单一承载，编辑态由 zorite 内建
            // Copy 接管，不在此口）；⌘E：编辑进出 toggle（DEC-156）。
            .on_key_down(cx.listener(
                |this, ev: &gpui::KeyDownEvent, _w: &mut Window, cx: &mut Context<Self>| {
                    if ev.keystroke.key == "escape" {
                        if this.editing {
                            this.toggle_edit(cx);
                        } else {
                            this.fire_peek_event(&PeekEvent::Escape, cx);
                        }
                        cx.stop_propagation();
                    } else if ev.keystroke.key == "c"
                        && ev.keystroke.modifiers.platform
                        && !this.editing
                    {
                        this.copy_selection(cx);
                        cx.stop_propagation();
                    } else if ev.keystroke.key == "e" && ev.keystroke.modifiers.platform {
                        this.toggle_edit(cx);
                        cx.stop_propagation();
                    }
                },
            ))
            .child(bar)
            // 保存失败条幅（DEC-159）：bar 与内容区之间承载 io 失败错误明文（
            // 暖底醒目 + 设计红棕文字 · 十七色板内既有色），窗保留期间常驻。
            .when(self.save_error.is_some(), |el| {
                el.child(
                    div()
                        .id("peek-save-error")
                        .flex()
                        .items_center()
                        .bg(rgb(t::WARM))
                        .px(px(16.0))
                        .py(px(8.0))
                        .child(co::txt_f(
                            self.save_error.clone().unwrap_or_default(),
                            s::font(s::Lvl::Small),
                            t::TAG_COLORS[0],
                            t::FONT_BODY,
                            t::FONT_WEIGHT_REGULAR,
                        )),
                )
            })
            .child(content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn peek_kind_labels_cover_three_sources() {
        assert_eq!(PeekKind::Clip.label(), "剪贴板内容");
        assert_eq!(PeekKind::Note.label(), "笔记");
        assert_eq!(PeekKind::Task.label(), "待办");
    }

    #[test]
    fn demo_body_fills_scrollbar_overflow() {
        let body = demo_body("《队列触发点表》：羊皮纸墨面同源演示正文。");
        assert!(body.starts_with("《队列触发点表》：羊皮纸墨面同源演示正文。"));
        // ≥10 段占位：480~780 面板内容区必然溢出（垂直滚动条口径）。
        assert!(body.matches("演示占位").count() >= 10);
    }

    #[gpui::test]
    async fn peek_view_construction_carries_entry(cx: &mut gpui::TestAppContext) {
        let item = PeekItem {
            kind: PeekKind::Task,
            title: "回收站文案评审".into(),
            meta: "TASK-14 · 今天".into(),
            body: demo_body("任务描述演示正文。"),
            disk_real: false,
        };
        let view = cx.new(|cx| PeekView::with_item(item.clone(), cx));
        assert!(view.update(cx, |v, _| v.item.is_some()));
        let empty = cx.new(PeekView::new);
        assert!(empty.update(cx, |v, _| v.item.is_none()));
        // with_item 与 set_item 同源（Clone 语义在事件转发链承载）。
        assert_eq!(item.kind, PeekKind::Task);
    }

    #[gpui::test]
    async fn set_item_same_content_is_no_op(cx: &mut gpui::TestAppContext) {
        // 同内容 no-op：返回 false 不 notify 不重绘（EdgeMark 热换协议第一律 ·
        // DEC-94）；换内容返回 true 且现场条目同步为新内容（原位换帧语义）。
        let view = cx.new(|cx| {
            let mut v = PeekView::new(cx);
            assert!(v.set_item(demo_item("甲"), cx), "首次注入应变更");
            v
        });
        view.update(cx, |v, cx| {
            assert!(!v.set_item(demo_item("甲"), cx), "同内容 no-op");
            assert!(v.set_item(demo_item("乙"), cx), "换内容应变更");
            assert_eq!(
                v.item.as_ref().map(|i| i.title.as_str()),
                Some("乙"),
                "现场条目已原位换帧"
            );
        });
    }

    fn demo_item(title: &str) -> PeekItem {
        PeekItem {
            kind: PeekKind::Note,
            title: title.to_string(),
            meta: "演示".to_string(),
            body: "正文".to_string(),
            disk_real: false,
        }
    }

    #[gpui::test]
    async fn md_sel_state_machine_down_drag_up_slice(cx: &mut gpui::TestAppContext) {
        // DEC-184 md 渲染选区状态机：按下记锚 → 拖动移头 → 抬起终态化（空选
        // 清除）；⌘C 切片走 md_selection_slice（源边界钳制）。
        let view = cx.new(|cx| PeekView::with_item(demo_item("md 选态"), cx));
        view.update(cx, |view, cx| {
            // 人造源文本缓存（render 接线在真机帧才跑，测试直注字段 · 状态机
            // 事件注入同构 hover_pending_close 先例）。
            view.md_view_source = Some("abcdef".to_string());
            assert!(view.md_sel_range().is_none(), "无会话空区间");
            view.on_md_sel_event(SelectEvent::Down(2), cx);
            assert!(view.selecting.get(), "按下即进入拖选态（与段选共用旗标）");
            view.on_md_sel_event(SelectEvent::Drag(7), cx);
            assert_eq!(view.md_sel_range(), Some(2..7), "拖动移头（末越界原样）");
            // 逆拖（头小于锚）：区间规范化 start ≤ end。
            view.on_md_sel_event(SelectEvent::Drag(0), cx);
            assert_eq!(view.md_sel_range(), Some(0..2), "逆拖规范化");
        });
        view.update(cx, |view, _cx| {
            let slice = view.md_selection_slice().unwrap();
            assert_eq!(slice, "ab", "切片按源字节区间（多字节钳制后 0.2）");
        });
        view.update(cx, |view, cx| {
            view.on_md_sel_event(SelectEvent::Up, cx);
            assert!(!view.selecting.get(), "抬起退出拖选态");
            assert_eq!(
                view.md_sel_range(),
                Some(0..2),
                "非塌缩（逆拖）选区保留供 ⌘C"
            );
        });
    }

    #[gpui::test]
    async fn md_sel_same_point_up_clears_and_drag_ignored_without_session(
        cx: &mut gpui::TestAppContext,
    ) {
        // 塌缩会话（单点按下后抬起）→ 选区清空；无会话 Drag 忽略（守卫）。
        let view = cx.new(|cx| PeekView::with_item(demo_item("md 选态二"), cx));
        view.update(cx, |view, cx| {
            view.md_view_source = Some("abcdef".to_string());
            view.on_md_sel_event(SelectEvent::Down(3), cx);
            view.on_md_sel_event(SelectEvent::Up, cx);
            assert!(view.md_sel_range().is_none(), "单点塌缩抬起即清");
            assert!(!view.selecting.get(), "旗标复位");
            view.on_md_sel_event(SelectEvent::Drag(5), cx);
            assert!(view.md_sel_range().is_none(), "无会话 Drag 忽略");
        });
    }

    #[gpui::test]
    async fn flush_probe_non_editing_and_pending_passthrough(cx: &mut gpui::TestAppContext) {
        // 冲刷探测（DEC-159 · 只读不摘引擎）：非编辑态 edit None（无论条目是否
        // 在场）；pending 寄存集透传给 app 层裁定；多失败并存不覆写（R184
        // F-183-08）；同题清除 / 异题保留。
        let view = cx.new(|cx| PeekView::with_item(demo_item("探测条目"), cx));
        view.update(cx, |v, cx| {
            let probe = v.flush_probe(cx);
            assert!(probe.edit.is_none(), "非编辑态探测应为空");
            assert!(probe.pending.is_empty(), "无寄存探测 pending 应空");
            v.mark_save_failed(demo_item("寄存条目"), "权限拒绝（os error 13）", cx);
            assert!(v.save_error.is_some(), "失败面文案已置");
            assert_eq!(v.pending_failed.len(), 1, "失败条目已寄存");
            // 多失败并存：后置 mark 不覆写前置（F-183-08 核心面）。
            v.mark_save_failed(demo_item("别条乙"), "磁盘满（os error 28）", cx);
            v.mark_save_failed(demo_item("寄存条目"), "重试再败（os error 13）", cx);
            assert_eq!(v.pending_failed.len(), 3, "多失败并存逐次累加");
            let probe = v.flush_probe(cx);
            assert!(probe.edit.is_none(), "非编辑态 edit 仍空");
            assert_eq!(
                probe.pending.first().map(|i| i.title.as_str()),
                Some("寄存条目"),
                "寄存透传（关窗冲刷重试源）"
            );
            // 同题清除（同题多条一并移除）/ 异题保留；全清前条幅不撤。
            v.clear_save_failed("不存在条", cx);
            assert_eq!(v.pending_failed.len(), 3, "异题清除无效果");
            v.clear_save_failed("寄存条目", cx);
            assert_eq!(v.pending_failed.len(), 1, "同题移除（两条同名一并出清）");
            assert!(v.save_error.is_some(), "仍有寄存条幅保留（其余未落盘）");
            v.clear_save_failed("别条乙", cx);
            assert!(v.pending_failed.is_empty(), "同题保存成功后寄存集清空");
            assert!(v.save_error.is_none(), "条幅随全清撤下");
        });
    }

    #[gpui::test]
    async fn flush_probe_editing_without_engine_reports_no_change(cx: &mut gpui::TestAppContext) {
        // 编辑在编探测（DEC-159）：进编辑但引擎未建（无活值）→ edit None（无
        // 键入无冲刷需求，与 tear_down_edit 取值面一致）；引擎/基线/缓冲零副作用。
        let view = cx.new(|cx| PeekView::with_item(demo_item("编辑探测"), cx));
        view.update(cx, |v, cx| {
            v.toggle_edit(cx);
            assert!(v.editing, "前置：已进编辑态");
            assert!(v.md_editor.is_none(), "前置：引擎未建（render 首帧延迟建）");
            let probe = v.flush_probe(cx);
            assert!(probe.edit.is_none(), "无活值可探——不可误报变更");
            assert!(v.pending_failed.is_empty(), "探测零置位");
            assert!(v.editing, "探测不摘引擎不改态");
        });
    }

    #[gpui::test]
    async fn toggle_edit_static_demo_noops_and_enters_with_item(cx: &mut gpui::TestAppContext) {
        // 面板内编辑进出（DEC-156）：数据驱动条目 toggle 进编辑态；静态演示态
        //（item=None · --peek 1:1）静默 no-op 不进编辑。
        let view = cx.new(|cx| PeekView::with_item(demo_item("羊皮纸视觉系统推写"), cx));
        view.update(cx, |v, cx| {
            v.toggle_edit(cx);
            assert!(v.editing, "有条目应进编辑态");
            assert!(v.hybrid, "进编辑默认混合态");
        });
        view.update(cx, |v, cx| {
            v.toggle_edit(cx);
            assert!(!v.editing, "再 toggle 回只读态");
        });
        let empty = cx.new(PeekView::new);
        empty.update(cx, |v, cx| {
            v.toggle_edit(cx);
            assert!(!v.editing, "静态演示态应 no-op 不进编辑");
        });
    }

    #[gpui::test]
    async fn truncated_strip_handoff_unique_trigger_fires_edit_handoff(
        cx: &mut gpui::TestAppContext,
    ) {
        // 截断条「在编辑器打开」钮（DEC-157）：EditHandoff 面板内唯一触发点——
        // 数据条目上抛 app 层转出（关 Peek → 统一编辑器窗）；静态演示（item=
        // None · 截断行不渲染）本口同守卫 no-op。
        let events: std::rc::Rc<std::cell::RefCell<Vec<&'static str>>> = Default::default();
        let over = PeekItem {
            kind: PeekKind::Note,
            title: "超长稿".to_string(),
            meta: "演示".to_string(),
            body: "abc".repeat(PEEK_MD_MAX_BYTES / 3 + 1),
            disk_real: false,
        };
        let view = cx.new(|cx| {
            let mut view = PeekView::with_item(over, cx);
            let events = events.clone();
            view.set_event_sink(Box::new(move |ev: &PeekEvent, _cx: &mut App| {
                if let PeekEvent::EditHandoff(_) = ev {
                    events.borrow_mut().push("handoff");
                }
            }));
            view
        });
        view.update(cx, |v, cx| {
            assert!(
                truncated_read_body(&v.item.as_ref().unwrap().body).1,
                ">200KB 应触发截断"
            );
            v.request_edit_handoff(cx);
            assert_eq!(
                events.borrow().as_slice(),
                ["handoff"],
                "截断条转出上抛 EditHandoff"
            );
        });
        let demo = cx.new(PeekView::new);
        demo.update(cx, |v, cx| {
            v.request_edit_handoff(cx);
            assert_eq!(
                events.borrow().as_slice(),
                ["handoff"],
                "静态演示无源 no-op（上抛未追加）"
            );
        });
    }

    #[gpui::test]
    async fn md_engine_live_sync_and_flush(cx: &mut gpui::TestAppContext) {
        // 编辑态 zorite 引擎承载链（DEC-156 · EditorShell md_flow 同构）：进编辑
        // 首帧 render 自动建引擎并聚焦；replace_range（host 结构性编辑，仅 notify
        // 不派发 Changed）→ observe 兑底活同步条目正文；tear_down_edit 取走活正
        // 文且回报 body 有变；末帧再退出（无变化）→ SaveEdit 不再上抛。
        let events: std::rc::Rc<std::cell::RefCell<Vec<PeekItem>>> = Default::default();
        let (view, cx) = cx.add_window_view(|_w, cx| {
            crate::init(cx);
            let mut view = PeekView::with_item(demo_item("活同步稿"), cx);
            let events = events.clone();
            view.set_event_sink(Box::new(move |ev: &PeekEvent, _cx: &mut App| {
                if let PeekEvent::SaveEdit(item) = ev {
                    events.borrow_mut().push(item.clone());
                }
            }));
            view
        });
        view.update(cx, |v, cx| v.toggle_edit(cx));
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
        });
        cx.run_until_parked();
        view.update(cx, |v, _cx| {
            assert!(v.md_editor.is_some(), "进编辑首帧应自动建 md 引擎");
        });
        // host 结构性编辑仿真：replace_range 只 notify → observe 兑底同步。
        cx.update(|_window, app| {
            let ed = view.read(app).md_editor.clone().unwrap();
            ed.update(app, |st, cx| st.replace_range(0..0, "首段文字", cx));
        });
        cx.run_until_parked();
        view.update(cx, |v, _cx| {
            assert_eq!(
                v.item.as_ref().map(|i| i.body.as_str()),
                Some("首段文字正文"),
                "结构性编辑 observe 兑底入正体（前插 · 原正文保留）"
            );
        });
        // 撤出：tear_down_edit 取走活正文，body 有变 = true。
        view.update(cx, |v, cx| {
            let (item, changed) = v.tear_down_edit(cx).expect("编辑态应可冲刷");
            assert!(changed, "正文有变应回报");
            assert!(v.md_editor.is_none() && !v.editing, "引擎已摘回落只读");
            assert_eq!(item.body, "首段文字正文", "冲刷条目携带活正文");
        });
        // 无变化退出：不再回报 changed（未变更不冲刷）。
        view.update(cx, |v, cx| {
            v.toggle_edit(cx);
            assert!(v.editing);
        });
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
        });
        cx.run_until_parked();
        view.update(cx, |v, cx| {
            let (_, changed) = v.tear_down_edit(cx).expect("编辑态应可冲刷");
            assert!(!changed, "无变化冲刷应回报无变");
        });
    }

    #[gpui::test]
    async fn set_item_edit_gate_same_title_keeps_other_title_flushes(
        cx: &mut gpui::TestAppContext,
    ) {
        // 编辑态热换门（DEC-156）：同题重单击保留在编缓冲（不冲不换）；异题冲刷
        // 在编条目（SaveEdit 上抛）后换帧读态。
        let events: std::rc::Rc<std::cell::RefCell<Vec<PeekItem>>> = Default::default();
        let (view, cx) = cx.add_window_view(|_w, cx| {
            crate::init(cx);
            let mut view = PeekView::with_item(demo_item("甲"), cx);
            let events = events.clone();
            view.set_event_sink(Box::new(move |ev: &PeekEvent, _cx: &mut App| {
                if let PeekEvent::SaveEdit(item) = ev {
                    events.borrow_mut().push(item.clone());
                }
            }));
            view
        });
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
        });
        cx.run_until_parked();
        view.update(cx, |v, cx| {
            v.toggle_edit(cx);
        });
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
        });
        cx.run_until_parked();
        // 在编缓冲仿真：引擎内编辑 → observe 兑底入条目。
        cx.update(|_window, app| {
            let ed = view.read(app).md_editor.clone().unwrap();
            ed.update(app, |st, cx| st.replace_range(0..0, "编辑中", cx));
        });
        cx.run_until_parked();
        view.update(cx, |v, cx| {
            assert!(
                !v.set_item(demo_item("甲"), cx),
                "同题重单击应 no-op 保留缓冲"
            );
            assert!(v.editing, "编辑态保持");
            assert_eq!(
                v.item.as_ref().map(|i| i.body.as_str()),
                Some("编辑中正文"),
                "在编缓冲未被行数据原值覆盖"
            );
            assert!(v.set_item(demo_item("乙"), cx), "异题应冲刷换帧");
            assert!(!v.editing, "异题冲刷后回落只读");
            assert_eq!(
                v.item.as_ref().map(|i| i.title.as_str()),
                Some("乙"),
                "现场条目已切换"
            );
        });
        assert_eq!(
            events.borrow().first().map(|i| i.title.as_str()),
            Some("甲"),
            "异题冲刷应上抛在编条目"
        );
    }

    #[gpui::test]
    async fn src_mode_toggle_preserves_canonical_body(cx: &mut gpui::TestAppContext) {
        // 源码/混合互斥切换（DEC-156 · EditorShell toggle_mode 同构）：body ← 引
        // 擎活值连续不丢；回混合后引擎自 canonical 重建。
        let (view, cx) = cx.add_window_view(|_w, cx| {
            crate::init(cx);
            PeekView::with_item(demo_item("源码稿"), cx)
        });
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
        });
        cx.run_until_parked();
        view.update(cx, |v, cx| {
            v.toggle_edit(cx);
            assert!(v.hybrid);
        });
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
        });
        cx.run_until_parked();
        // 混合态引擎内编辑 → 活同步。
        cx.update(|_window, app| {
            let ed = view.read(app).md_editor.clone().unwrap();
            ed.update(app, |st, cx| {
                let end = st.text().len();
                st.replace_range(end..end, "源码前写入", cx);
                cx.emit(MdEvent::Changed);
            });
        });
        cx.run_until_parked();
        view.update(cx, |v, cx| {
            v.toggle_src_mode(cx);
            assert!(!v.hybrid, "转源码态");
            assert!(v.md_editor.is_none(), "混合引擎已摘");
        });
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
        });
        cx.run_until_parked();
        view.update(cx, |v, _cx| {
            assert!(v.src_textarea.is_some(), "源码态首帧建 Textarea");
            let st = v.src_textarea.as_ref().unwrap();
            let val = st.read(_cx).value().to_string();
            assert_eq!(
                val, "正文源码前写入",
                "源码引擎自 canonical 正文建立（混合态活值不丢）"
            );
        });
        // 回混合态：md 引擎自 canonical 重建。
        view.update(cx, |v, cx| {
            v.toggle_src_mode(cx);
            assert!(v.hybrid, "回混合态");
        });
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
        });
        cx.run_until_parked();
        cx.update(|_window, app| {
            let ed = view.read(app).md_editor.clone().unwrap();
            assert_eq!(
                ed.read(app).text(),
                "正文源码前写入",
                "回混合自 canonical 重建"
            );
        });
    }

    #[gpui::test]
    async fn hover_close_suspended_while_editing_and_resumed_after(cx: &mut gpui::TestAppContext) {
        // 编辑态挂起悬停收窗（DEC-156 · DEC-94 Pin 同构）：编辑中移出不发关闭；
        // 退出编辑时指针仍在窗外则补发；指针在内不补发。
        let events: std::rc::Rc<std::cell::RefCell<Vec<&'static str>>> = Default::default();
        let view = cx.new(|cx| {
            let mut view = PeekView::with_item(demo_item("挂起稿"), cx);
            let events = events.clone();
            view.set_event_sink(Box::new(move |ev: &PeekEvent, _cx: &mut App| match ev {
                PeekEvent::HoverChanged(true) => events.borrow_mut().push("in"),
                PeekEvent::HoverChanged(false) => events.borrow_mut().push("out"),
                PeekEvent::Escape => events.borrow_mut().push("esc"),
                PeekEvent::EditHandoff(_) => events.borrow_mut().push("handoff"),
                PeekEvent::SaveEdit(_) => events.borrow_mut().push("save"),
                PeekEvent::Composing { .. } => {} // IME 内部通道（测试面不采集）
            }));
            view
        });
        view.update(cx, |v, cx| {
            v.toggle_edit(cx);
            assert!(v.editing);
            v.on_hover_event(false, cx);
            assert!(
                events.borrow().is_empty(),
                "编辑中移出不发关闭（编辑态挂起）"
            );
            v.on_hover_event(true, cx); // 回窗（正常进事件）
            assert_eq!(*events.borrow(), vec!["in"]);
            v.toggle_edit(cx); // 指针在内退出：不补发、无 SaveEdit（无变化）
            assert!(!v.editing);
            assert_eq!(*events.borrow(), vec!["in"]);
            v.on_hover_event(false, cx); // 非编辑态照常直发
            assert_eq!(*events.borrow(), vec!["in", "out"]);
        });
        // 复位后编辑态退出（指针在外）补发：
        view.update(cx, |v, cx| {
            v.toggle_edit(cx); // 进编辑
            v.on_hover_event(true, cx);
            v.on_hover_event(false, cx); // 编辑中移出 → 挂起（不发）
            assert_eq!(*events.borrow(), vec!["in", "out", "in"].as_slice());
            v.toggle_edit(cx); // 退出编辑：指针仍在外 → 补发 out
            assert_eq!(*events.borrow(), vec!["in", "out", "in", "out"].as_slice());
        });
    }

    #[gpui::test]
    async fn peek_window_hover_event_and_global_flag(cx: &mut gpui::TestAppContext) {
        let events: std::rc::Rc<std::cell::RefCell<Vec<bool>>> = Default::default();
        let (_view, visual_cx) = cx.add_window_view(|_window, _cx| {
            let mut view = PeekView::with_item(demo_item("羊皮纸视觉系统推导"), _cx);
            let events = events.clone();
            view.set_event_sink(Box::new(move |ev: &PeekEvent, _cx: &mut App| {
                if let PeekEvent::HoverChanged(h) = ev {
                    events.borrow_mut().push(*h);
                }
                // Escape：本测试只验悬停事件通道，Esc 通道由 app 层 close_peek 承载。
            }));
            view
        });
        // 窗内点（root 全窗承接，测试窗最大化到测试显示区）→ 进入事件 + 全局置位。
        visual_cx.simulate_mouse_move(
            gpui::point(gpui::px(300.0), gpui::px(300.0)),
            None,
            Default::default(),
        );
        visual_cx.run_until_parked();
        assert_eq!(*events.borrow(), vec![true]);
        let inside = visual_cx.read(|cx| cx.try_global::<PeekHover>().is_some_and(|h| h.inside));
        assert!(inside);
        // 窗内再次移动：悬停态未变 → 不重复发事件。
        visual_cx.simulate_mouse_move(
            gpui::point(gpui::px(310.0), gpui::px(310.0)),
            None,
            Default::default(),
        );
        visual_cx.run_until_parked();
        assert_eq!(*events.borrow(), vec![true]);
    }

    #[gpui::test]
    async fn keyboard_modality_does_not_synthesize_hover_exit(cx: &mut gpui::TestAppContext) {
        // REQ-PREVIEW-SCROLL-STABILITY（DEC-107）：预览窗内键入（方向键滚动 / Esc /
        // ⌘C）不得被 gpui 键盘模态 hover 抑制语义伪翻转成「移出」→ arm_peek_close
        // 收 Peek 并级联收面板。根 hover_listener_mode=InputModalityIndependent：
        // hover 只认真移出与几何变化；键盘事件后悬停保持；真移出 (MouseExited)
        // 照常上抛 false（控制组防过修）。
        use gpui::{InputEvent as _, VisualContext as _};
        let events: std::rc::Rc<std::cell::RefCell<Vec<bool>>> = Default::default();
        let (_view, visual_cx) = cx.add_window_view(|_window, _cx| {
            let mut view = PeekView::with_item(demo_item("羊皮纸视觉系统推导"), _cx);
            let events = events.clone();
            view.set_event_sink(Box::new(move |ev: &PeekEvent, _cx: &mut App| {
                if let PeekEvent::HoverChanged(h) = ev {
                    events.borrow_mut().push(*h);
                }
            }));
            view
        });
        // 悬停会话建立（进入事件）。
        visual_cx.simulate_mouse_move(
            gpui::point(gpui::px(300.0), gpui::px(300.0)),
            None,
            Default::default(),
        );
        visual_cx.run_until_parked();
        assert_eq!(*events.borrow(), vec![true]);
        // 键盘滚动语义键入：模态翻 Keyboard + refresh 重绘——伪「移出」不得发生。
        visual_cx.simulate_keystrokes("down");
        visual_cx.run_until_parked();
        assert_eq!(*events.borrow(), vec![true]);
        // 字符键入同口径（任意键均不触显隐）。
        visual_cx.simulate_keystrokes("a");
        visual_cx.run_until_parked();
        assert_eq!(*events.borrow(), vec![true]);
        let handle = visual_cx.window_handle();
        // 控制组：真移出（MouseExited 平台事件）仍上抛 false。
        handle
            .update(&mut visual_cx.cx, |_, window, cx| {
                window.dispatch_event(
                    gpui::MouseExitEvent {
                        position: gpui::point(gpui::px(0.0), gpui::px(0.0)),
                        ..Default::default()
                    }
                    .to_platform_input(),
                    cx,
                );
            })
            .unwrap();
        visual_cx.run_until_parked();
        assert_eq!(*events.borrow(), vec![true, false]);
    }

    #[test]
    fn copy_target_prefers_selection_and_joins_paragraphs() {
        let paras = vec!["你好世界".to_string(), "second".to_string()];
        // 同段选区：CJK 字节边界 0..3 = 首字符「你」（3 字节 UTF-8）。
        assert_eq!(
            copy_target(
                &paras,
                Some(&ParaSel {
                    a_para: 0,
                    a_ix: 0,
                    h_para: 0,
                    h_ix: 3
                })
            ),
            Some("你".to_string())
        );
        // 同段中段区间 3..9 = 「好世」（字节切片不劈裂字符）。
        assert_eq!(
            copy_target(
                &paras,
                Some(&ParaSel {
                    a_para: 0,
                    a_ix: 3,
                    h_para: 0,
                    h_ix: 9
                })
            ),
            Some("好世".to_string())
        );
        // 跨段选区（DEC-109）：锚段尾段 + 头段首段按段序 \n 连接。
        assert_eq!(
            copy_target(
                &paras,
                Some(&ParaSel {
                    a_para: 0,
                    a_ix: 3,
                    h_para: 1,
                    h_ix: 3
                })
            ),
            Some("好世界\nsec".to_string())
        );
        // 反向拖动归一：锚在头后 → 同一内容。
        assert_eq!(
            copy_target(
                &paras,
                Some(&ParaSel {
                    a_para: 1,
                    a_ix: 3,
                    h_para: 0,
                    h_ix: 6
                })
            ),
            Some("世界\nsec".to_string())
        );
        // 越界区间防御：头字节超段长 → None（不截断不恐慌）。
        assert_eq!(
            copy_target(
                &paras,
                Some(&ParaSel {
                    a_para: 0,
                    a_ix: 0,
                    h_para: 0,
                    h_ix: 99
                })
            ),
            None
        );
        // 塌缩空选 → 视同无选择（全段拼接）。
        assert_eq!(
            copy_target(
                &paras,
                Some(&ParaSel {
                    a_para: 0,
                    a_ix: 3,
                    h_para: 0,
                    h_ix: 3
                })
            ),
            Some("你好世界\nsecond".to_string())
        );
        // 无选择 → 全段按行 join（可见即所得）。
        assert_eq!(
            copy_target(&paras, None),
            Some("你好世界\nsecond".to_string())
        );
        // 空段落表 → None（无复制目标，复制动作静默）。
        assert_eq!(copy_target(&[], None), None);
    }

    #[test]
    fn para_highlight_derives_span_ranges() {
        // 派生高亮纯函数（DEC-109）：锚段尾段 / 途经整段 / 头段区间 / 越界段
        // None / 反向归一 / 同段区间 / 塌缩 None。
        let paras = vec!["甲段".to_string(), "乙段".to_string(), "丙段".to_string()];
        let fwd = ParaSel {
            a_para: 0,
            a_ix: 3,
            h_para: 2,
            h_ix: 3,
        };
        assert_eq!(para_highlight(&fwd, &paras, 0), Some(3..6), "锚段尾段");
        assert_eq!(para_highlight(&fwd, &paras, 1), Some(0..6), "途经段整段");
        assert_eq!(para_highlight(&fwd, &paras, 2), Some(0..3), "头段首段");
        assert_eq!(para_highlight(&fwd, &paras, 3), None, "越界段");
        let rev = ParaSel {
            a_para: 2,
            a_ix: 3,
            h_para: 0,
            h_ix: 0,
        };
        assert_eq!(
            para_highlight(&rev, &paras, 0),
            Some(0..6),
            "反向头段=头位→段尾（头在段首即整段）"
        );
        assert_eq!(
            para_highlight(&rev, &paras, 1),
            Some(0..6),
            "反向途经段整段"
        );
        assert_eq!(
            para_highlight(&rev, &paras, 2),
            Some(0..3),
            "反向锚段=段首→锚点"
        );
        let rev_tail = ParaSel {
            a_para: 2,
            a_ix: 3,
            h_para: 0,
            h_ix: 6,
        };
        assert_eq!(
            para_highlight(&rev_tail, &paras, 0),
            None,
            "反向头位=段尾无字节贡献（空区间）"
        );
        let same = ParaSel {
            a_para: 1,
            a_ix: 0,
            h_para: 1,
            h_ix: 6,
        };
        assert_eq!(para_highlight(&same, &paras, 1), Some(0..6), "同段全段");
        let collapsed = ParaSel {
            a_para: 1,
            a_ix: 3,
            h_para: 1,
            h_ix: 3,
        };
        assert_eq!(para_highlight(&collapsed, &paras, 1), None, "塌缩无高亮");
    }

    #[gpui::test]
    async fn hover_pending_close_defers_until_para_lift(cx: &mut gpui::TestAppContext) {
        // 暂缓-补发状态机（app 层零改动回归面）：拖选中移出窗口不立即上抛关闭，
        // 抬起补发 HoverChanged(false)（对称宽限照常收口 · 生命周期不悬挂）。
        let events: std::rc::Rc<std::cell::RefCell<Vec<&'static str>>> = Default::default();
        let view = cx.new(|cx| {
            let mut view = PeekView::with_item(demo_item("羊皮纸视觉系统推导"), cx);
            let events = events.clone();
            view.set_event_sink(Box::new(move |ev: &PeekEvent, _cx: &mut App| match ev {
                PeekEvent::HoverChanged(true) => events.borrow_mut().push("in"),
                PeekEvent::HoverChanged(false) => events.borrow_mut().push("out"),
                PeekEvent::Escape => events.borrow_mut().push("esc"),
                PeekEvent::EditHandoff(_) => events.borrow_mut().push("edit"),
                PeekEvent::SaveEdit(_) => events.borrow_mut().push("save"),
                PeekEvent::Composing { .. } => {} // IME 内部通道（测试面不采集）
            }));
            view
        });
        view.update(cx, |view, cx| {
            view.on_para_event(ParaSelEvent::Down { para: 0, ix: 0 }, cx);
            assert!(view.selecting.get(), "按下即进入拖选态");
            view.on_hover_event(false, cx);
            assert!(view.pending_close, "拖选中移出 → 暂挂");
        });
        assert!(events.borrow().is_empty(), "暂挂期间不上抛关闭事件");
        view.update(cx, |view, cx| {
            view.on_para_event(ParaSelEvent::Up, cx);
            assert!(!view.pending_close, "抬起清暂挂");
        });
        assert_eq!(
            events.borrow().as_slice(),
            ["out"],
            "抬起补发关闭上抛（app 层对称宽限照常收口）"
        );
        // 后续移出恢复正常直发路径（暂挂已清）。
        view.update(cx, |view, cx| view.on_hover_event(false, cx));
        assert_eq!(events.borrow().as_slice(), ["out", "out"]);
        // 进入路径不受影响（即时上抛）。
        view.update(cx, |view, cx| view.on_hover_event(true, cx));
        assert_eq!(events.borrow().as_slice(), ["out", "out", "in"]);
    }

    #[gpui::test]
    async fn set_item_change_clears_selection_state(cx: &mut gpui::TestAppContext) {
        // 换内容清选择态：旧字节区间引用失效（防复制到旧文 · 单一事实源）；跨段
        // 头更新随查（DEC-109）。
        let view = cx.new(|cx| PeekView::with_item(demo_item("甲"), cx));
        view.update(cx, |view, cx| {
            view.rendered_paras = vec!["段落文本".to_string(), "第二段落".to_string()];
            view.on_para_event(ParaSelEvent::Down { para: 0, ix: 0 }, cx);
            assert!(view.selection.is_some(), "按下后有选择态");
            view.on_para_event(ParaSelEvent::Drag { para: 1, ix: 6 }, cx);
            assert_eq!(
                view.selection,
                Some(ParaSel {
                    a_para: 0,
                    a_ix: 0,
                    h_para: 1,
                    h_ix: 6
                }),
                "跨段拖选更新选择头（6 = 「第二」两字节界）"
            );
            assert!(view.set_item(demo_item("乙"), cx), "换内容应变更");
            assert!(view.selection.is_none(), "换条目清选择");
            assert!(
                !view.selecting.get() && !view.select_all && !view.pending_close,
                "拖选会话一并终止"
            );
        });
    }

    #[gpui::test]
    async fn drag_selection_spans_paragraphs_and_clamps(cx: &mut gpui::TestAppContext) {
        // 跨段连续拖选状态机（DEC-109 · REQ-PREVIEW-SELECT-COPY）：锚-头跨段更
        // 新、头未动免更新、段号越界钳末段、反向拖动对称、塌缩抬起清选择。
        let view = cx.new(|cx| PeekView::with_item(demo_item("甲"), cx));
        view.update(cx, |view, cx| {
            view.rendered_paras = vec![
                "甲段文字".to_string(),
                "乙段文字".to_string(),
                "丙段".to_string(),
            ];
            view.on_para_event(ParaSelEvent::Down { para: 0, ix: 3 }, cx);
            assert!(view.selecting.get(), "按下进入拖选会话");
            view.on_para_event(ParaSelEvent::Drag { para: 2, ix: 3 }, cx);
            assert_eq!(
                view.selection,
                Some(ParaSel {
                    a_para: 0,
                    a_ix: 3,
                    h_para: 2,
                    h_ix: 3
                }),
                "跨段跳进：锚固定头更新"
            );
            view.on_para_event(ParaSelEvent::Drag { para: 2, ix: 3 }, cx);
            assert_eq!(
                view.selection,
                Some(ParaSel {
                    a_para: 0,
                    a_ix: 3,
                    h_para: 2,
                    h_ix: 3
                }),
                "头未动免更新（同值守门 · 不重复 notify）"
            );
            view.on_para_event(ParaSelEvent::Drag { para: 9, ix: 1 }, cx);
            assert_eq!(
                view.selection,
                Some(ParaSel {
                    a_para: 0,
                    a_ix: 3,
                    h_para: 2,
                    h_ix: 0
                }),
                "段号越界防御钳末段（字节钳字符边界）"
            );
            view.on_para_event(ParaSelEvent::Drag { para: 0, ix: 0 }, cx);
            assert_eq!(
                view.selection,
                Some(ParaSel {
                    a_para: 0,
                    a_ix: 3,
                    h_para: 0,
                    h_ix: 0
                }),
                "反向拖动对称（头回锚段段首）"
            );
            view.on_para_event(ParaSelEvent::Up, cx);
            assert!(
                view.selection.is_some() && !view.selecting.get(),
                "非塌缩抬起保留选择并退出会话"
            );
            view.on_para_event(ParaSelEvent::Down { para: 1, ix: 0 }, cx);
            view.on_para_event(ParaSelEvent::Up, cx);
            assert!(view.selection.is_none(), "塌缩抬起清选择");
        });
    }

    #[gpui::test]
    async fn wheel_freeze_flips_hover_under_still_pointer(cx: &mut gpui::TestAppContext) {
        // 滚轮冻结诊断链回归（REQ-PREVIEW-SCROLL-STABILITY · DEC-110）：真机症状
        // 「内容面板快速滚轮 → 应用隐藏」的机制锚点。vendor hover_freeze 在本仓
        // 测试编译域为启用态（cfg!(test) 仅在 vendor 自测编译时关闭，parchora-ui
        // 依赖编译下恒为 false → enabled）；滚轮事件于分发前被记录（vendor
        // input.rs before_dispatch 对任意 ScrollWheel 无条件 note_wheel），冻结
        // 窗口内（100ms · 指针未动）的绘制帧 hit test 被置空 → 根 on_hover 伪
        // 翻转 false → HoverChanged(false) 上抛。app 层显隐守护域（对称宽限收口）
        // 依此承载判定；本测试钉住机制存在性，防 vendor 静默变更漂移（DEC-110 D1）。
        let events: std::rc::Rc<std::cell::RefCell<Vec<bool>>> = Default::default();
        let (_view, visual_cx) = cx.add_window_view(|_window, _cx| {
            let item = PeekItem {
                kind: PeekKind::Note,
                title: "滚轮冻结诊断".to_string(),
                meta: "演示".to_string(),
                // 正文 ≥10 段溢出：滚轮真实驱动有界滚动；滚轮 position 与指针
                // 同位，满足冻结判据 position == window.mouse_position。
                body: demo_body("《滚轮冻结诊断》：羊皮纸墨面同源演示正文。"),
                disk_real: false,
            };
            let mut view = PeekView::with_item(item, _cx);
            let events = events.clone();
            view.set_event_sink(Box::new(move |ev: &PeekEvent, _cx: &mut App| {
                if let PeekEvent::HoverChanged(h) = ev {
                    events.borrow_mut().push(*h);
                }
            }));
            view
        });
        // 悬停会话建立（进入事件）。
        visual_cx.simulate_mouse_move(
            gpui::point(gpui::px(300.0), gpui::px(300.0)),
            None,
            Default::default(),
        );
        visual_cx.run_until_parked();
        assert_eq!(*events.borrow(), vec![true]);
        // 同位滚轮注入冻结（note_wheel 定时器 110ms 不在断言前到期；frozen 以
        // 真实时钟 100ms 判定，测试序列紧凑即可）。
        visual_cx.simulate_event(gpui::ScrollWheelEvent {
            position: gpui::point(gpui::px(300.0), gpui::px(300.0)),
            delta: gpui::ScrollDelta::Pixels(gpui::point(gpui::px(0.0), gpui::px(-600.0))),
            ..Default::default()
        });
        // 冻结窗口内强制出帧（Window::draw 与 ArenaClearNeeded::clear 均为 pub）：
        // paint 期 mouse_hit_test 置空 → 伪翻转 → false 上抛。
        visual_cx.update(|window, cx| {
            window.draw(cx).clear(cx);
        });
        visual_cx.run_until_parked();
        assert_eq!(
            *events.borrow(),
            vec![true, false],
            "滚轮冻结伪翻转 = 诊断链事实（收口语义由 app 层宽限承载，非 ui 层）"
        );
    }

    #[gpui::test]
    async fn read_src_toggle_note_only_flips_and_guards(cx: &mut gpui::TestAppContext) {
        // 只读态「源码」切换钮（DEC-158）：笔记条目 toggle 翻转/回翻；编辑态、
        // 静态演示（item=None）与 Clip 守卫 no-op（编辑态互斥切换由源码钮承载；
        // 编辑进出不重置只读源码视图旗标）。
        let view = cx.new(|cx| {
            let mut v = PeekView::with_item(demo_item("源码切换稿"), cx);
            assert!(!v.read_src, "默认渲染视图");
            v.toggle_read_src(cx);
            assert!(v.read_src, "toggle 进源码视图");
            v.toggle_read_src(cx);
            assert!(!v.read_src, "再 toggle 回渲染视图");
            v
        });
        view.update(cx, |v, cx| {
            v.toggle_read_src(cx);
            assert!(v.read_src, "前置：源码视图");
            v.toggle_edit(cx);
            assert!(v.editing, "进编辑态");
            v.toggle_read_src(cx);
            assert!(
                v.read_src,
                "编辑态只读源码切换 no-op（toggle_src_mode 承载）"
            );
            v.toggle_edit(cx);
            assert!(!v.editing, "退出编辑态");
            assert!(v.read_src, "编辑进出不重置只读源码视图旗标");
            v.toggle_read_src(cx);
            assert!(!v.read_src, "回渲染视图");
        });
        let clip = cx.new(|cx| {
            let mut v = PeekView::with_item(
                PeekItem {
                    kind: PeekKind::Clip,
                    title: "剪贴板条目".into(),
                    meta: "演示".into(),
                    body: "正文".into(),
                    disk_real: false,
                },
                cx,
            );
            v.toggle_read_src(cx);
            assert!(
                !v.read_src,
                "Clip 条目无钮源码切换 no-op（非文档语义保留纯文本段）"
            );
            v
        });
        clip.update(cx, |v, _| assert!(!v.read_src));
        let empty = cx.new(PeekView::new);
        empty.update(cx, |v, cx| {
            v.toggle_read_src(cx);
            assert!(!v.read_src, "静态演示态无钮源码切换 no-op");
        });
    }

    // 源码视图拖选承载（DEC-183 · REQ-PREVIEW-SELECT-COPY 补全）：渲染段表＝
    // [src] 后选择区间字节切片与源逐字一致；视图切换（read_src toggle）清选择
    //（陈旧区间引用防护）。
    #[gpui::test]
    async fn src_view_selection_slices_and_toggle_clears(cx: &mut gpui::TestAppContext) {
        let body = "第一行\n\n第二行\n第三行";
        let view = cx.new(|cx| {
            let item = PeekItem {
                kind: PeekKind::Note,
                title: "选择稿".into(),
                meta: "磁盘真实 · 刚刚".into(),
                body: body.to_string(),
                disk_real: true,
            };
            let mut v = PeekView::with_item(item, cx);
            v.toggle_read_src(cx);
            assert!(v.read_src, "前置：源码视图");
            v
        });
        view.update(cx, |v, cx| {
            // 渲染段表（render 头部置定）：源码视图 → [src] 单段；测试直置同值。
            v.rendered_paras = vec![body.to_string()];
            // 拖选等价：首段字节区间 0..17（第一行=9B·\n=2B·第二=6B；跨行
            // 字节切片与源逐字一致，CJK 多字节安全）。
            v.on_para_event(ParaSelEvent::Down { para: 0, ix: 0 }, cx);
            v.on_para_event(ParaSelEvent::Drag { para: 0, ix: 17 }, cx);
            v.editing = false;
            v.copy_selection(cx); // 选择区间优先 → ⌘C = 选段字节切片
        });
        let got = cx
            .read_from_clipboard()
            .and_then(|c| c.text())
            .unwrap_or_default();
        assert_eq!(got, "第一行\n\n第二");
        view.update(cx, |v, cx| {
            v.toggle_read_src(cx); // 回渲染视图 → 选择清零
            assert!(v.selection.is_none(), "视图切换清选择（陈旧区间防护）");
        });
    }

    #[gpui::test]
    async fn set_item_content_change_resets_read_src(cx: &mut gpui::TestAppContext) {
        // 条目热换复位（DEC-158）：异内容换帧回渲染视图默认态；同内容 no-op 路
        // 径早退不触碰旗标（在编缓冲保留语义同源）。
        let view = cx.new(|cx| {
            let mut v = PeekView::with_item(demo_item("甲"), cx);
            v.toggle_read_src(cx);
            v
        });
        view.update(cx, |v, _| assert!(v.read_src, "前置：源码视图"));
        view.update(cx, |v, cx| {
            assert!(!v.set_item(demo_item("甲"), cx), "同内容 no-op");
            assert!(v.read_src, "同内容热换保留源码视图");
            assert!(v.set_item(demo_item("乙"), cx), "换内容应变更");
        });
        view.update(cx, |v, _| {
            assert!(!v.read_src, "异内容热换复位回渲染视图默认态");
        });
    }
}
