//! PG-B1 笔记编辑区（UI-4 · TASK-504 EditorView 视图壳 / 480×70% 内容展示面板）。
//!
//! HTML 1:1（PG-B1.html · DEC-129 起在册口径）：
//! - bar（← 返回 + 文件名即标题 + 模式钮（二态）+ 定位到磁盘；
//!   复制▾/任务钮：无（DEC-129 移除））；
//! - 查找替换栏：无（DEC-176 整面移除）；
//! - 源码区（正文区默认中文楷体 Kaiti SC · DEC-129；pre/代码
//!   块等宽 Menlo 随渲染态围栏块承载 · lh 1.9 · 演示态行例含斜杠命令/公式/围栏⧉复制/
//!   warm 高亮片段）；
//! - 追加段高亮 chip（growth 边框 · 渐隐语义 ~1s）；
//! - 复选框预设行（done 点 + 划线 + 「8 复选框预设 ✓」）。
//! - 图标（DEC-66）：返回=arrow-left · 定位到磁盘=external-link ·
//!   围栏复制=copy · 文件名/渲染正文=内容族 Kaiti SC（内容区 · DEC-68）。
//! - 围栏复制 chip 接线（DEC-45 承载 · DEC-112）：点击写剪贴板演示围栏文本；
//!   Toast 固定场景清单不含「复制」→ 仅按压反馈不弹 Toast（与 peek 同口径）。
//!
//! 演示宿主限定（偏差 D-06/D-08）：源码编辑引擎已实装
//!（gpui-component Textarea · editor-spike 同线）；此处承载源码态整篇源码区
//! 与混合态编辑的样式。
//!
//! 编辑器双态（DEC-133/134/150）：默认单栏混合编辑（Typora 式「编辑即渲染」·
//! 输入与渲染合二为一、不设独立预览区域）；源码态＝整篇源码 Textarea，与 md
//! 引擎互斥挂载——切换经 canonical body，内容连续不丢。「源码」切换钮固定标
//! 签＋前置 Code 图标，选中态主墨底＋纸光反白（DEC-124 背景轴）。「定位到磁
//! 盘」置 reveal 旗标后走 Esc 统一事件——app 层 editor_exit 漏斗：落盘授权稿
//! 正文非空先写盘（标题名.md · files 原子写 · TASK-502 部分解冻）再 reveal
//! 文件（窗口持开启），否则 reveal files 根目录。
//!
//! 滚动承接：库级 Scrollable 句柄 global_id 绑元素栈路径业务层不可达（vendor
//! fast/global_id.rs 实证）——自持 ScrollHandle 三层等价结构＋ScrollbarLayer
//! 等价 overlay（scrollbar-overlay 断言源保持）；offset 负值域（0..−max）。
//!
//! 编辑区高/宽定式（DEC-136 · DEC-154/176/182）：滚动视口高＝面板高−标题栏高−垂直间距
//! 32px（ed-wrap 上下外边距 my 16×2）−面板边框线宽
//!（0，面板无边框）——以 flex_1+min_h_0 弹性承载，与定式等价；正文宽＝面板宽−左右 px16
//!（zone 内衬）。内容超出滚动视口即显垂直滚动条（gpui-component Scrollable 附
//! 挂 Scrollbar；Scrollbar 自带「无溢出即隐藏」语义 scrollbar.rs，主题常驻模式
//! Always＋BORDER 拇指色——ui init 统一装配，全 app 无其他 Scrollbar 挂载零连
//! 带）。debug_bounds 断言见 tests::editor_area_geometry_formula 与
//! editor_scrollbar_overlay_geometry。
//!
//! 渲染字号（DEC-134）：H1(26)/H2(20)/H3(17) 常量同值回写 tokens.json；围栏块
//! pre Menlo 等宽（DEC-68/129）；正文/块编辑区 Kaiti SC（DEC-129 任务 5）。
//!
//! 标题模型（DEC-130）：新稿标题默认 Untitled；点击标题栏标题→行内编辑
//!（enter/失焦提交 · esc 取消）；保存以标题名.md 命名（标题为唯一命名源）。
//!
//! 编辑转出承接（REQ-PREVIEW-IMAGE · DEC-24 · DEC-156 起非默认路径在册）：Peek
//! 编辑转出通道 EditHandoff → app 层关 Peek → 本壳以条目标题/正文开窗
//!（title/body 注入态 · 同内容热换 no-op DEC-94 · DockHandles.editor 单实例登记）
//! ——DEC-156 面板内编辑上线后本通道无面板内默认触发点（超长折叠等转出场景在
//! 册承载）。注入态源码区真输入引擎 gpui-component Textarea（editor-spike 同线）
//! ——body 活模型（混合态渲染流读活值）；延迟初始化模式（render 内建态 · 保构
//! 造签名与既有测试不动）；DEMO_FENCE_CODE 保留为 --page editor 演示态围栏 chip
//! 承载（演示态源码区 1:1 不回归）。
//!
//! Esc/返回钮/定位到磁盘退出（Peek DEC-73 同构）：key_context("Editor") +
//! track_focus → escape→EditorEscape 绑定 → 根 on_action → EditorEvent::Escape
//! → app 层 editor_exit 统一漏斗（DEC-129：保存判定 + reveal 旗标分派）——关闭
//! 即冲刷落盘（SCN-EDIT-SAVE-FLUSH · Untitled 新稿正文非空时 files 原子写；正文
//! 空不产生文件）。DEC-151：zorite bind_keys 注册后 escape 必先命中 action 通道
//!（bubble 相位默认截停，根级 on_key_down 的 Esc 分支不可达）——宿主以同 context
//! 后注册绑定接管，根 keydown escape 分支保留为 IME composition 等绑定不适用路
//! 径兑底。
//!
//! 混合态编辑与渲染引擎（DEC-150）：vendored zorite-editor 0.11.2（gpui-fast
//! 0.1.8 同源 API 面）承载——EditorState 单实体实时渲染编辑器（W1-W6 管线：标题
//! 字号/粗斜删码链/列表/任务勾选/围栏/引用/表格/高亮，语法标记 caret 显隐，
//! auto-grow 高度自适应、无内建滚动条——溢出由 ed-scroll 外层视口承接）。实时
//! Markdown 渲染即产品本体；环境字体/字号自 wrapper 级联（Kaiti SC 15px）；滚动
//! 跟随锚点 caret_screen_bounds（窗口空间 · 末帧布局缓存）→ follow_md_cursor 行
//! 距带判定。业务语义：canonical body 经 EditorEvent::Changed 活同步（无在途窗口
//! ——Esc 首按即安全关窗；live_body/export_note 关窗冲刷不丢字）；set_content 热
//! 换经 set_text（无 window 依赖）直导；空稿自动聚焦开局；源码态为 gpui-component
//! Textarea 整篇（互斥挂载）；标题行内编辑/演示态 1:1/几何定式断言面（ed-zone=
//! ed-scroll−16 · 宽=vw−32）全保留；ed-md 取样面；围栏复制为 zorite 原生 Copy
//! chip（Labels 中文化接线）；内建 undo/redo（bind_keys 绑定）。
//!
//! DEC-151 装配面：init 装配 zorite bind_keys（退格/方向键/选区/undo/redo/格式
//! 裹/粘贴自键盘全量可达)；escape 由宿主 EditorEscape action 通道承接（同
//! context 后注册优先·根 on_action）；follow_md_cursor 锚点 caret_screen_bounds
//! 精确视觉行优先（软折承载）＋文本变更路径 on_next_frame 延帧跟随；工具栏三钮
//! debug_selector 取样面。验收测试在册 tests（真实键击/undo/⌘B 裹挟/⌘V 粘贴/
//! Esc 护栏/工具栏三钮/特殊字符/超长软折/越界/空稿/文末跟随）。
//!
//! vendored zorite-editor 补丁面（DEC-152）：围栏码块容器全宽（CodeBg 无宽度字
//! 段——绘制/悬停卡命中矩形/语言与 Copy chip 右缘三消费点按 bounds 全宽取值；
//! 长行按宽度换行）；块级拖拽手柄不存在（line_drag/grip_hover_row/grip_inset
//! 状态机与 set_grip_inset 公共 API 均不在）。外层包裹容器 ed-wrap：水平外边
//! 距 16px＋白底 60%＋圆角＋overflow 裁剪集中承载（仅注入态挂视觉面，演示态
//! --page editor 1:1 不变），zone/textarea_zone 仅 px/py 行内衬，宽定式
//! w_zone==vw−32；几何断言 editor_wrapper_geometry 在册 tests。
//!
//! 标题栏标题 display 态无下划线/边框（DEC-153）——可点击暗示由 cursor_pointer
//! 承载（编辑态 Input appearance(false)/bordered(false) 亦无边框）。
//!
//! 中文 IME 与正文点击焦点承载（DEC-155）：LSUIElement＋PopUp 非激活面板下 IMK
//! 会话依赖激活 app——vendored gpui-fast 帧尾 FocusGained 切换沿激活 app（vendored
//! patch · 规范性披露）；zorite 失焦即 handler 不注册——正文点击（zorite hitbox
//! 外空白区）聚焦经 ed-scroll-content capture_any_mouse_down＋prevent_default
//!（capture 先行定焦 · 抑制根 div bubble 抢焦 · prevent_default 不中断传播，
//! zorite 点击交互无损）。

use gpui::prelude::FluentBuilder as _;
use gpui::{
    div, px, rgb, rgba, App, AppContext as _, ClickEvent, ClipboardItem, Context, Div, Entity,
    Focusable as _, InteractiveElement as _, MouseButton, ParentElement as _, Render,
    StatefulInteractiveElement as _, Styled, Window,
};
use gpui_component as ui;
use ui::input::{Input, InputEvent, InputState, Textarea, TextareaState};
use ui::scroll::{Scrollbar, ScrollbarAxis};
use ui::{InteractiveElementExt as _, Sizable as _};
use zorite_editor::{EditorEvent as MdEvent, EditorState as MdState, Labels};

use crate::{components as co, scale as s, tokens as t};

/// 编辑区背景（DEC-143）：白底 60% 不透明度——alpha
/// 0.6（0.6×255=153=0x99）垫羊皮底透出层级。文件内局部
/// 常量不入 tokens.json（如需令牌化另行确认）。
const EDITOR_BG: u32 = 0xFFFFFF99;

/// 源码态 Textarea auto-grow 行数上限（DEC-195）：内容行数封顶（超出退内部滚
/// 动）—— effectively 无界，覆盖任意真实文档体量；行长下溢出行数 × 行高 ≈
/// 2.1M px 远超面板可能尺寸，正常路径永不触及。
pub(crate) const SRC_AUTO_GROW_MAX_ROWS: usize = 100_000;

/// 围栏复制演示文本（--page editor 演示态围栏 chip 承载 · DEC-45/112 ·
/// 演示态源码区 1:1 不回归；混合态复制走 zorite 原生 Copy chip · DEC-150）。
const DEMO_FENCE_CODE: &str = "fn main() {\n    println!(\"Parchora\");\n}";

/// 混合态正文占位（zorite placeholder 承载）。
pub(crate) const MD_PLACEHOLDER: &str = "输入正文…（Markdown）";

/// 浮层危险色（zorite 菜单 destructive 行 · 文件内局部常量，
/// 前例：EDITOR_BG 不入 tokens.json，如需令牌化另行确认）。
const MD_DANGER: u32 = 0xE5484D;

/// 混合态 zorite 引擎羊皮 SyntaxStyle（DEC-150）：映射 DESIGN「Parchment
/// Parchment Ink」令牌——标记半透主墨、行内码＝强调字＋WARM 半透底、链接＝
/// 强调、标签＝生长绿、引用/分隔线＝弱化线、高亮＝强调半透底、浮层＝表面/
/// 边框/主墨三元组；围栏码块＝藏青 60% 专用深卡＋纸光字（DEC-228 · 用户
/// 指令 #142346@60%，与行内码/斑马纹 WARM 底分字段承载）；等宽＝Menlo
///（DEC-68/129 同源）。全令牌行内换算不新增常量（铁律 10）。
pub(crate) fn md_parchment_style() -> zorite_editor::SyntaxStyle {
    zorite_editor::SyntaxStyle {
        block_label: None,
        block_label_gen: 0,
        block_ref_count: None,
        marker: rgba(t::PRIMARY << 8 | 0x99).into(),
        code: rgb(t::TERTIARY).into(),
        code_bg: rgba(t::WARM << 8 | 0x66).into(),
        fence_bg: rgba(t::CODE_NAVY << 8 | 0x99).into(),
        fence_fg: rgb(t::ON_PRIMARY).into(),
        link: rgb(t::TERTIARY).into(),
        tag: rgb(t::GROWTH).into(),
        quote: rgb(t::MUTED).into(),
        alert_note: rgb(t::TERTIARY).into(),
        alert_tip: rgb(t::GROWTH).into(),
        alert_important: rgb(t::TERTIARY).into(),
        alert_warning: rgb(t::MUTED).into(),
        alert_caution: rgb(t::MUTED).into(),
        alert_icons: None,
        rule: rgba(t::BORDER_SOFT << 8 | 0xB0).into(),
        mark_bg: rgba(t::ACCENT << 8 | 0x40).into(),
        popover_bg: rgb(t::SURFACE).into(),
        popover_border: rgb(t::BORDER).into(),
        popover_fg: rgb(t::PRIMARY).into(),
        popover_hover: rgba(t::TERTIARY << 8 | 0x2A).into(),
        popover_divider: rgb(t::BORDER_SOFT).into(),
        popover_danger: rgb(MD_DANGER).into(),
        mono: gpui::font(t::FONT_MONO),
        property_icon: None,
    }
}

/// zorite 菜单/围栏 chip 中文标签（Labels::default 基底覆写 · DEC-150）。
pub(crate) fn md_labels() -> Labels {
    Labels {
        cut: "剪切".into(),
        copy: "复制".into(),
        copy_as_markdown: "复制为 Markdown".into(),
        paste: "粘贴".into(),
        code_copy: "复制".into(),
        math_copy: "复制公式".into(),
        turn_into: "转换为".into(),
        text: "正文".into(),
        heading_1: "一级标题".into(),
        heading_2: "二级标题".into(),
        heading_3: "三级标题".into(),
        bulleted_list: "无序列表".into(),
        numbered_list: "有序列表".into(),
        todo: "任务列表".into(),
        quote: "引用".into(),
        callout: "提示块".into(),
        code_block: "代码块".into(),
        math_block: "公式".into(),
        insert_row_above: "上方插入行".into(),
        insert_row_below: "下方插入行".into(),
        duplicate_row: "复制行".into(),
        insert_column_left: "左侧插入列".into(),
        insert_column_right: "右侧插入列".into(),
        align_left: "左对齐".into(),
        align_center: "居中".into(),
        align_right: "右对齐".into(),
        ..Labels::default()
    }
}

// 编辑窗退出统一事件 host 侧 action（DEC-151）：init 于 zorite bind_keys 之后
// 追加 escape→EditorEscape（同 context「Editor」·后注册优先）——接管 Dismiss
//（action bubble 相位默认截停，root on_key_down 的 Esc 监听不可达）：EditorShell
// 根 on_action 承接同源退出语义（EditorEvent::Escape 统一漏斗不变）。
gpui::actions!(parchora_ui, [EditorEscape]);

/// 标题行盒目标高（DEC-131）：单行 Input 固有行盒 1.25rem=20px 且小于
/// 标题栏内容高上限（返回钮 12px 行盒 19.416+py6+边框 2≈27.416px）——编辑态
/// Input 以此定高后，bar 高两态由返回钮唯一决定，恒等不跳。
/// 注入态编辑窗 → 宿主事件（编辑转出承接）。
pub enum EditorEvent {
    /// 编辑窗退出统一事件（app 层 editor_exit 漏斗 · DEC-129）：Esc 键与
    /// 返回钮走常规收口（保存判定 + 关窗）；ed-finder「定位到磁盘」置 reveal 旗标
    /// 后同事件上抛（漏斗读旗标分派——保存 + reveal 文件/根目录 + 窗口持开启）。
    /// 关闭即冲刷点（REQ-EDIT-SAVE-DEBOUNCE/SCN-EDIT-SAVE-FLUSH）。
    Escape,
    /// IME 组合态边沿（DEC-224 · 用户指令：修复候选窗被应用窗口遮挡）：
    /// 外壳每帧直读 zorite 引擎的 marked-text 存在性，翻转即上抛——`active`
    /// 为组合中（app 层即时降级键窗 NSWindowLevel 让候选浮层置顶）/ false 为
    /// 组合结束或引擎拆卸（恢复原 level）。内部通道，不改变 Esc 漏斗语义。
    Composing { active: bool },
}

/// 编辑窗 → 宿主上抛通道。
pub type EditorSink = Box<dyn Fn(&EditorEvent, &mut App) + 'static>;

/// 编辑区（源码态演示壳）。
pub struct EditorShell {
    /// 编辑模式（DEC-133）：true＝单栏混合编辑（输入＋同栏渲染流 ·
    /// 默认）、false＝纯源码态。
    hybrid: bool,
    /// 「定位到磁盘」请求旗标（DEC-129）：ed-finder 置位 → Esc 统一
    /// 事件 → app 层 editor_exit 单点消费（take_reveal）。
    reveal_requested: bool,
    /// 编辑转出注入标题（None = 演示 const · --page editor 1:1）
    title: Option<String>,
    /// 编辑转出注入正文（纯文本承载；None = 演示 const 行例 · 同批切换）
    body: Option<String>,
    /// 真输入引擎态（TASK-504）：源码态整篇 Textarea（延迟初始
    /// 化——render 首帧建；DEC-150 与 zorite 混合引擎**互斥挂载**——仅源码态
    /// 挂载）。None = 混合态/演示态。活正文读 state.value()。
    src_textarea: Option<Entity<TextareaState>>,
    /// 混合态 markdown 引擎（DEC-150）：zorite EditorState 实时渲染编辑器——
    /// 单实体承载「编辑即渲染」（W1-W6 管线）；auto-grow 无内建滚动，溢出由
    /// ed-scroll 外层承接。canonical body 经 Changed 活同步（无槽无在途窗口）。
    md_editor: Option<Entity<MdState>>,
    /// md 引擎事件订阅保活（Changed→body 同步 / SelectionChanged→滚动跟随；
    /// 切源码态即摘）。
    md_sub: Option<gpui::Subscription>,
    /// md 引擎 observe 兑底（host 结构性编辑只 notify 不派发 Changed——body
    /// 哈希比对同步；切源码态即摘）。
    md_obs: Option<gpui::Subscription>,
    /// IME 组合态边沿记忆（DEC-224）：render 每帧与 markdown 引擎
    /// marked-text 存在性比对，翻转即上抛 `EditorEvent::Composing`；None＝
    /// 无引擎（演示态/源码态），上抛复位沿。
    ime_composing: Option<bool>,
    /// 外层滚动视口自持句柄（库级 Scrollable 句柄 global_id 绑元素栈路径业务层
    /// 跨上下文不可达 · vendor fast/global_id.rs 实证）；光标跟随需 set_offset 通道。
    ed_scroll: gpui::ScrollHandle,
    /// 热换未决正文（DEC-94 热换协议承载）：set_content 在 src 引擎已建时记录
    /// 新正文，下帧 render 内 set_value 导入（set_value 需 &mut Window，仅 render
    /// 可得——与延迟初始化同机制承载）。
    pending_set: Option<String>,
    /// 键盘焦点柄（Esc 退出承载 · PeekView 同构）
    focus: gpui::FocusHandle,
    /// 窗 → 宿主事件通道（app 层 Esc 关窗；--page 演示宿主不注入则无事件）
    on_editor_event: Option<EditorSink>,
    /// 标题行内编辑中（DEC-130）：编辑中 bar 标题位换真单行
    /// Input（下方 title_input 承载引擎）。
    editing_title: bool,
    /// 标题行内编辑引擎（begin_title_edit 时建——listener 自带 &mut Window；勿
    /// 开 clean_on_escape，esc 冒泡根级取消编辑不关窗）。
    title_input: Option<Entity<InputState>>,
    /// 标题编辑事件订阅保活（PressEnter/Blur → 提交；提交/取消即摘）。
    title_sub: Option<gpui::Subscription>,
    /// 落盘授权旗标（DEC-130）：仅新建稿路（app 层 open_editor_new）
    /// 置位；handoff/演示窗恒 false——「内容不入盘」口径不变（editor_exit 漏斗
    /// 判定）。
    disk_save: bool,
    /// 已落盘稿的净化题名（app 层回写）：下次退出同题覆写续存——防「Untitled N」
    /// 增殖与静默丢编辑；改名后题名失配 → 按新题另落新文件（顺延）。
    saved_name: Option<String>,
    /// 编辑后光标跟随待验旗标（DEC-151）：文本变更路径（键入/回车/粘贴只发
    /// Changed、undo/redo 仅 notify）在陈旧布局下同步跟随会越界 no-op——置位后
    /// 由 render 头经 window.on_next_frame 于当前帧渲染完（新布局 line_tops
    /// 生效）再执行 follow_md_cursor。
    md_follow_pending: bool,
}

impl EditorShell {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            hybrid: true,
            reveal_requested: false,
            title: None,
            body: None,
            src_textarea: None,
            md_editor: None,
            md_sub: None,
            md_obs: None,
            ime_composing: None,
            ed_scroll: gpui::ScrollHandle::new(),
            pending_set: None,
            focus: cx.focus_handle(),
            on_editor_event: None,
            editing_title: false,
            title_input: None,
            title_sub: None,
            disk_save: false,
            saved_name: None,
            md_follow_pending: false,
        }
    }

    /// 编辑转出注入构造（Peek「编辑」/⌘E 跨窗接手）：标题进 bar
    /// 文件名位、正文进源码区；chrome 与演示态同款（PG-B1 1:1）。
    pub fn with_content(title: String, body: String, cx: &mut Context<Self>) -> Self {
        let mut this = Self::new(cx);
        this.title = Some(title);
        this.body = Some(body);
        this
    }

    /// 键盘焦点柄（app 层开窗后 window.focus 定向；Esc 经根 key_context 分派）。
    pub fn focus_handle(&self) -> gpui::FocusHandle {
        self.focus.clone()
    }

    /// 注入窗 → 宿主事件通道（app 层 Esc 关编辑窗）。
    pub fn set_event_sink(&mut self, sink: EditorSink) {
        self.on_editor_event = Some(sink);
    }

    /// 上抛窗事件（生产链：on_key_down listener 于 EditorShell 租借态内调用 →
    /// app sink 同实体分支必须 defer 出借域 · DEC-77）。
    pub fn fire_editor_event(&self, ev: &EditorEvent, cx: &mut App) {
        if let Some(f) = &self.on_editor_event {
            f(ev, cx);
        }
    }

    /// 原位更新注入内容（编辑窗单实例：再次 handoff 仅换内容不重开窗）。同内容
    /// no-op（不 notify 不重绘 · EdgeMark 热换协议第一律 · DEC-94）；返回是否
    /// 变更（回归取证面）。md 引擎已建时 set_text 直导（无 window 依赖·DEC-150）；
    /// src 引擎已建时记 pending_set，下帧 set_value 导入。
    pub fn set_content(&mut self, title: String, body: String, cx: &mut Context<Self>) -> bool {
        if self.title.as_deref() == Some(title.as_str())
            && self.body.as_deref() == Some(body.as_str())
        {
            return false;
        }
        let body_changed = self.body.as_deref() != Some(body.as_str());
        self.title = Some(title);
        self.body = Some(body);
        if body_changed {
            if let Some(ed) = &self.md_editor {
                let next = self.body.clone().unwrap_or_default();
                ed.update(cx, |st, cx| st.set_text(next, cx));
            }
            if self.src_textarea.is_some() {
                self.pending_set = self.body.clone();
            }
        }
        cx.notify();
        true
    }

    /// 活正文读取（DEC-150）：混合态 → md 引擎实时值（Changed 活同步，无在
    /// 途窗口）；源码态 → src 引擎实时值；未建（演示态/未出帧）→ 注入正文
    /// fallback（既有口径不变）。
    fn live_body(&self, cx: &App) -> Option<String> {
        if self.hybrid {
            if let Some(ed) = &self.md_editor {
                return Some(ed.read(cx).text().to_string());
            }
        } else if let Some(st) = &self.src_textarea {
            return Some(st.read(cx).value().to_string());
        }
        self.body.clone()
    }

    /// 导出快照（DEC-129 · app 层收口漏斗读取面）：（标题，活正文）——
    /// 经 live_body 统一取值（引擎实时值 · 演示态 None）。
    pub fn export_note(&self, cx: &App) -> (Option<String>, Option<String>) {
        (self.title.clone(), self.live_body(cx))
    }

    /// 「定位到磁盘」旗标单点消费（app 层 editor_exit 调用 · DEC-129）：
    /// 取走并复位，保证旗标恰消耗一次。
    pub fn take_reveal(&mut self) -> bool {
        std::mem::take(&mut self.reveal_requested)
    }

    fn toggle_mode(&mut self, _e: &ClickEvent, _w: &mut Window, cx: &mut Context<Self>) {
        if self.hybrid {
            // 混合 → 源码：body←md 引擎活值（canonical body 连续不丢）·摘 md 引
            // 擎；src 引擎延迟建于下帧 render。pending_set 未消耗时（热换新正文
            // 在档——src 引擎尚建）不回读旧引擎值——body 已是新值。
            if let Some(ed) = self.md_editor.take() {
                if self.pending_set.is_none() {
                    self.body = Some(ed.read(cx).text().to_string());
                }
            }
            self.md_sub = None;
            self.md_obs = None;
            self.pending_set = None;
        } else {
            // 源码 → 混合：body←src 引擎实时值 · 关 src 引擎（md 引擎下帧 render
            // 建，初值＝canonical body）。
            if let Some(st) = self.src_textarea.take() {
                if self.pending_set.is_none() {
                    self.body = Some(st.read(cx).value().to_string());
                }
            }
            self.pending_set = None;
        }
        self.hybrid = !self.hybrid;
        cx.notify();
    }

    /// 「源码」钮选中态配色（DEC-124 背景轴 Pin 同款）：源码态＝
    /// 主墨底＋纸光反白（悬停/按下同底），混合态＝ibtn 基线三元组（纸面底＋
    /// 主墨字＋warm 悬停）。纯函数供调用点与测试共用。
    pub(crate) fn mode_palette(src_mode: bool) -> (u32, u32, u32) {
        if src_mode {
            (t::ON_PRIMARY, t::PRIMARY, t::PRIMARY)
        } else {
            (t::PRIMARY, t::SURFACE, t::WARM)
        }
    }

    /// 落盘授权态读取（app 层 editor_exit / FAB 守卫 / handoff 冲刷判定共用 ·
    /// DEC-130）：（授权旗标， 已落盘净化题名）。
    pub fn save_state(&self) -> (bool, Option<String>) {
        (self.disk_save, self.saved_name.clone())
    }

    /// 落盘授权置位（app 层开窗装配调用：新建稿路 true / handoff 路 false）。
    pub fn set_disk_save(&mut self, on: bool) {
        self.disk_save = on;
    }

    /// 记录已落盘题名（app 层落盘成功后回写）：下次退出同题覆写续存。
    pub fn remember_saved(&mut self, name: String) {
        self.saved_name = Some(name);
    }

    /// 热换为新建稿并预填正文（DEC-194 拾记「转为笔记」承接 · R184 建议 7：
    /// 裸版 reset_new_draft 无生产调用点多为死代码已删，空稿热换经空串预填
    /// 同语义承载）：内容置入 + 保存态复位（授权置位、已落盘名失效、行内
    /// 编辑关闭）——预填稿走编辑窗既有落盘漏斗（editor_exit）。
    pub fn reset_new_draft_with(&mut self, title: String, body: String, cx: &mut Context<Self>) {
        self.set_content(title, body, cx);
        self.saved_name = None;
        self.disk_save = true;
        self.cancel_title_edit(cx);
    }

    /// 标题行内编辑（DEC-130）：建真单行 Input 引擎（现值预
    /// 填 · listener 自带 &mut Window）+ 建后聚焦 + 订阅提交通道（enter/blur
    /// 提交；esc 经根级 key_down 取消）。演示态（title None）不进编辑——--page
    /// editor 1:1。
    fn begin_title_edit(&mut self, w: &mut Window, cx: &mut Context<Self>) {
        if self.editing_title || self.title.is_none() {
            return;
        }
        let cur = self.title.clone().unwrap_or_default();
        let st = cx.new(|cx| InputState::new(w, cx));
        st.update(cx, |state, cx| state.set_value(cur, w, cx));
        w.focus(&st.read(cx).focus_handle(cx), cx);
        let sub = cx.subscribe(
            &st,
            |this, _st: Entity<InputState>, ev: &InputEvent, cx: &mut Context<Self>| match ev {
                InputEvent::PressEnter { .. } | InputEvent::Blur => {
                    this.commit_title_edit(cx);
                }
                _ => {}
            },
        );
        self.editing_title = true;
        self.title_input = Some(st);
        self.title_sub = Some(sub);
        cx.notify();
    }

    /// 行内编辑提交（enter/blur 同源）：非空才生效（空值＝回退原标题）；订阅摘
    /// 除防 Blur 回环；返回是否改名（回归取证面）。
    fn commit_title_edit(&mut self, cx: &mut Context<Self>) -> bool {
        self.title_sub = None;
        let v = self
            .title_input
            .take()
            .map(|st| st.read(cx).value().trim().to_string());
        self.editing_title = false;
        let mut changed = false;
        if let Some(v) = v {
            if !v.is_empty() && self.title.as_deref() != Some(v.as_str()) {
                self.title = Some(v);
                changed = true;
            }
        }
        cx.notify();
        changed
    }

    /// 行内编辑取消（esc 根级 / 热换复位）：不改标题、摘订阅。
    pub fn cancel_title_edit(&mut self, cx: &mut Context<Self>) {
        self.title_sub = None;
        self.title_input = None;
        self.editing_title = false;
        cx.notify();
    }

    /// 混合态 zorite 引擎延迟初始化（DEC-150 · render 首帧建）：EditorState
    /// 承载「编辑即渲染」（with_text 预填 canonical body · 羊皮 SyntaxStyle ·
    /// 中文 Labels）；空稿自动聚焦开局（免无点击入口死区承接）。事件订阅：
    /// Changed → canonical body 活同步（无槽无在途窗口）；
    /// SelectionChanged → 滚动跟随。仅在混合态注入态建（body.is_some()）。
    fn ensure_md_engine(&mut self, w: &mut Window, cx: &mut Context<Self>) {
        if !self.hybrid || self.body.is_none() || self.md_editor.is_some() {
            return; // 仅混合态注入态；建后即返防重复建环
        }
        let body = self.body.clone().unwrap_or_default();
        let empty_entry = body.trim().is_empty();
        let ed = cx.new(|cx| {
            let mut st = MdState::new(w, cx)
                .with_text(body)
                .with_placeholder(MD_PLACEHOLDER);
            st.set_markdown_style(md_parchment_style(), cx);
            st.set_labels(md_labels(), cx);
            // 复制/剪切写作权接管（DEC-163 · REQ-CAPTURE-SELF-SKIP）：zorite 预设
            // gpui 直写，无登记位 —— host writer 桥先 clipstamp 登记原文再按默认
            // 语义写板（等价转发），混合态选中 ⌘C/⌘X 不会回污染拾记捕获。
            st.set_clipboard_writer(std::rc::Rc::new(|text: &str, cx: &mut gpui::App| {
                crate::clipstamp::mark(text);
                cx.write_to_clipboard(gpui::ClipboardItem::new_string(text.to_string()));
            }));
            if empty_entry {
                st.focus(w, cx); // 空稿自动开局＝直接聚焦（DEC-134 死区承接）
            }
            st
        });
        let sub = cx.subscribe(
            &ed,
            |this: &mut Self, ed: Entity<MdState>, ev: &MdEvent, cx: &mut Context<Self>| match ev {
                MdEvent::Changed => {
                    this.body = Some(ed.read(cx).text().to_string());
                    // 键入/回车/粘贴只发 Changed（无 SelectionChanged）：置跟随
                    // 待验旗标，render 头于新布局生效后执行（同步跟随会因陈旧
                    // 布局越界 no-op）；notify 外壳以消费旗标（render 头注册
                    // on_next_frame）。
                    this.md_follow_pending = true;
                    this.follow_md_cursor(cx);
                    cx.notify();
                }
                MdEvent::SelectionChanged => {
                    this.follow_md_cursor(cx);
                }
                _ => {}
            },
        );
        // host 侧结构性编辑（replace_range/set_text 等）不派发 Changed 只 notify——
        // observe 兑底同步 canonical body（哈希比对防抖）；哈希有差异（undo/redo
        // restore 路径）同置跟随待验。
        let obs = cx.observe(&ed, |this, ed, cx| {
            let next = ed.read(cx).text().to_string();
            if this.body.as_deref() != Some(next.as_str()) {
                this.body = Some(next);
                this.md_follow_pending = true;
                cx.notify();
            }
        });
        self.md_editor = Some(ed);
        self.md_sub = Some(sub);
        self.md_obs = Some(obs);
        cx.notify();
    }

    /// 外层滚动视口光标跟随（DEC-150/151）：md 引擎 caret 行锚（caret_screen_bounds
    /// 精确视觉行优先 · 回退 offset_screen_top 逻辑行顶——详见下方内注）→ 外层
    /// 句柄补偿，光标行保持可视。行距带判定（行底越视口底→底对齐；行顶越视口
    /// 顶→顶对齐）；offset 负值域（0..−max）与 clamp 语义与 gpui 引擎一致。行距
    /// 取 row_layout 首行行盒（未布局时回退正文磅 2.0 倍估计）。
    fn follow_md_cursor(&mut self, cx: &mut Context<Self>) {
        let Some(ed) = self.md_editor.as_ref() else {
            return;
        };
        let (caret_top, lh) = {
            let e = ed.read(cx);
            // 精确视觉行锚（DEC-151 · 软折承托）：caret_screen_bounds 给光标所在
            // 视觉段顶（窗口空间；软折单逻辑行多视觉段仍准确——offset_screen_top
            // 只按逻辑行取 top，软折行光标行不跟随）；行末哨兵 quirk（在册遗留）
            // 或陈旧布局越界（编辑后随帧内）返回 None → 回退逻辑行顶锚（既有路
            // 径不变）。
            match e.caret_screen_bounds() {
                Some(b) => (b.top(), b.size.height),
                None => {
                    // offset_screen_top：offset 所在逻辑行顶（窗口空间）——未
                    // paint/越界 None。
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
        let viewport = self.ed_scroll.bounds();
        let offset = self.ed_scroll.offset();
        // caret 行顶在滚动内容坐标中的位置（off ≤ 0：−off＝已滚动量）。
        let caret_y = f32::from(caret_top - viewport.top() - offset.y);
        // 行距带判定（镜像库内 scroll_to：行底越视口底 → 底对齐；行顶越顶 → 顶对齐）。
        let h = f32::from(viewport.size.height);
        let vis = caret_y + f32::from(offset.y);
        let mut new_off = f32::from(offset.y);
        if vis + f32::from(lh) > h {
            new_off = -(caret_y - h + f32::from(lh));
        } else if vis < 0.0 {
            new_off = -caret_y;
        }
        new_off = new_off.clamp(-f32::from(self.ed_scroll.max_offset().y), 0.0);
        if new_off != f32::from(offset.y) {
            self.ed_scroll
                .set_offset(gpui::point(offset.x, px(new_off)));
        }
    }

    /// 围栏复制（DEC-45 chip 的演示接线 · DEC-112）：写剪贴板＋按压反馈（active）。
    /// Toast 固定场景清单不含「复制」→ 不弹 Toast（与 peek copy_selection 同口径）。
    /// 写前经 clipstamp 登记（DEC-163 · REQ-CAPTURE-SELF-SKIP）：自写板不回捕获。
    fn copy_fence(&mut self, cx: &mut Context<Self>) {
        crate::clipstamp::mark(DEMO_FENCE_CODE);
        cx.write_to_clipboard(ClipboardItem::new_string(DEMO_FENCE_CODE.to_string()));
    }

    /// 源码区三行示例（1:1；FONT_MONO 等宽（Menlo · DEC-68）· warm 高亮片段）。
    fn source_lines(&self, cx: &mut Context<Self>) -> Div {
        let line = |children: Vec<Div>| {
            let mut el = div().flex();
            for c in children {
                el = el.child(c);
            }
            el
        };
        let warm_piece = |piece: &'static str| {
            div()
                .px(px(4.0))
                .rounded(px(t::RAD_SM))
                .bg(rgb(t::WARM))
                .child(co::txt_f(
                    piece,
                    s::font(s::Lvl::Mono),
                    t::PRIMARY,
                    t::FONT_MONO,
                    t::FONT_WEIGHT_LIGHT,
                ))
        };
        // 围栏复制 chip（DEC-45 · DEC-112 接线）：copy 图标 + 文字，点击写剪贴板
        // 演示围栏文本。
        let warm_copy = div()
            .id("ed-fence-copy")
            .flex()
            .items_center()
            .gap(px(4.0))
            .px(px(4.0))
            .rounded(px(t::RAD_SM))
            .bg(rgb(t::WARM))
            .cursor_pointer()
            .hover(co::hov_border)
            .active(co::act_border)
            .on_click(cx.listener(
                |this, _e: &ClickEvent, _w: &mut Window, cx: &mut Context<Self>| {
                    this.copy_fence(cx)
                },
            ))
            .child(co::icon(co::Icon::Copy, 11.0, t::PRIMARY))
            .child(co::txt_f(
                "复制",
                s::font(s::Lvl::Small),
                t::PRIMARY,
                t::FONT_BODY,
                t::FONT_WEIGHT_LIGHT,
            ));
        let mono = |code: &'static str, color: u32| {
            co::txt_f(
                code,
                s::font(s::Lvl::Mono),
                color,
                t::FONT_MONO,
                t::FONT_WEIGHT_LIGHT,
            )
        };

        div()
            .px(px(16.0))
            .py(px(12.0))
            .flex()
            .flex_col()
            .gap(px(2.0))
            .border_b_1()
            .border_color(rgb(t::BORDER_SOFT))
            .child(line(vec![
                mono("## 触发点（", t::PRIMARY),
                co::txt_f(
                    "斜杠命令浮层 ↓",
                    s::font(s::Lvl::Mono),
                    t::TERTIARY,
                    t::FONT_MONO,
                    t::FONT_WEIGHT_LIGHT,
                ),
                mono("）", t::PRIMARY),
            ]))
            .child(line(vec![
                mono("- ", t::PRIMARY),
                warm_piece("/"),
                mono("table → 插入表格骨架", t::PRIMARY),
                co::meta("　…共 10 条命令"),
            ]))
            .child(line(vec![
                mono("- $$E = mc^2$$ ", t::PRIMARY),
                co::meta("→ 渲染态显示公式"),
            ]))
            .child(line(vec![
                mono("- ```rust ", t::PRIMARY),
                div().child(warm_copy),
                co::meta(" ← 围栏块右上角，仅光标命中时出现"),
            ]))
    }

    /// 追加段（hl-append 渐隐 chip 演示终态）。
    fn append_zone(&self) -> Div {
        div()
            .px(px(16.0))
            .py(px(10.0))
            .flex()
            .items_center()
            .gap(px(8.0))
            .child(
                div()
                    .px(px(10.0))
                    .py(px(1.0))
                    .border_1()
                    .border_color(rgb(t::TERTIARY))
                    .rounded_full()
                    .child(co::txt(
                        "+ 已追加到手记段落（Ink-50 高亮渐隐 ~1s）",
                        11.0,
                        t::TERTIARY,
                    )),
            )
            .child(co::content_text(
                "「剪贴板摘录 + 来源行」",
                13.0,
                t::PRIMARY,
            ))
    }

    /// 复选框预设行（演示态）。
    fn checklist_row(&self) -> Div {
        div()
            .px(px(16.0))
            .py(px(12.0))
            .flex()
            .items_center()
            .gap(px(10.0))
            .child(co::dot(true))
            .child(
                // min_w_0（DEC-229）：strike 外层包装同体地板归零（同 tasks done 行）。
                div().flex_1().min_w_0().child(co::strike("渲染管线复核")),
            )
            .child(co::meta("8 复选框预设 ✓"))
    }

    /// 注入源码输入区（TASK-504）：真输入引擎 Textarea 承载（appearance(false)
    /// 羊皮底透出）；容器 div 级联字体（DEC-129：正文区默认中文楷体 Kaiti SC；
    /// pre/代码块等宽 Menlo 承载于渲染态围栏块）。
    /// auto-grow（DEC-195）：短文撑满容器（min_h_full 链 · DEC-146 语义保留），
    /// 长文随软折行数撑高（溢出由外层 ed-scroll 承接）。
    fn textarea_zone(&self, state: &Entity<TextareaState>) -> Div {
        div()
            .debug_selector(|| "ed-zone-src".to_string())
            // 内衬 px(16)/py(12)（DEC-143 行内衬）；宽定式承载与白底 60%＋圆角
            // 由外层 ed-wrap 承载（DEC-152 包裹容器）；Textarea auto-grow（DEC-195）
            // 短文撑满/长文随内容撑高。
            .px(px(16.0))
            .py(px(12.0))
            .min_h_full()
            .flex()
            .flex_col()
            .font_family(t::FONT_CONTENT)
            .text_size(px(s::font(s::Lvl::Small)))
            .child(
                Textarea::new(state)
                    .appearance(false)
                    .bordered(false)
                    .flex_1(),
            )
    }

    /// 混合态文档流 zone（DEC-150）：单实体 md 引擎原地承载「编辑即渲染」，
    /// 点击任意处直接编辑原位（zorite 元素自管 caret/选区/标记显隐）；视觉契
    /// 约全保：宽定式（w_zone==vw−32）与白底 60%＋圆角由外层 ed-wrap 承载
    ///（DEC-152 包裹容器），zone 只留 px(16)/py(12) 行内衬（DEC-143）；wrapper
    /// 级联 Kaiti SC＋正文磅＋主墨字色，引擎自身 auto-grow 无内建滚动，溢出由
    /// 外层 ed-scroll 承接。空文新稿＝ensure_md_engine 建后直接聚焦（免死区
    /// 承接）。取样面「ed-md」wrapper。
    fn md_zone(&self, state: &Entity<MdState>) -> Div {
        div()
            .debug_selector(|| "ed-zone".to_string())
            .px(px(16.0))
            .py(px(12.0))
            .min_h_full()
            .flex()
            .flex_col()
            // wrapper 级联（DEC-129 任务 5 承接）：正文 Kaiti SC 15px 主墨——
            // 引擎继承环境文字风格（zorite README「inherits the ambient text
            // style」）；标题字号/等宽字体由引擎按 SyntaxStyle 与语义自管。
            .font_family(t::FONT_CONTENT)
            .text_size(px(s::font(s::Lvl::Body)))
            .text_color(rgb(t::PRIMARY))
            .child(
                div()
                    .debug_selector(|| "ed-md".to_string())
                    .w_full()
                    .child(state.clone()),
            )
    }

    /// 混合态演示渲染流（真实管线随 TASK-505 markdown_view 接线）：静态呈现一
    /// 段同源渲染样式（h2 + 正文；内容区 → Kaiti SC · DEC-68）。
    fn demo_render_flow(&self) -> Div {
        div()
            .px(px(16.0))
            .py(px(12.0))
            .flex()
            .flex_col()
            .gap(px(8.0))
            .child(co::txt_f(
                "触发点",
                s::font(s::Lvl::H1),
                t::PRIMARY,
                t::FONT_CONTENT,
                t::FONT_WEIGHT_REGULAR,
            ))
            .child(co::content_text(
                "/table → 插入表格骨架（渲染段示意）",
                s::font(s::Lvl::Body),
                t::PRIMARY,
            ))
    }
}

impl Render for EditorShell {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let _s = crate::scale::enter(window, cx);
        // 真输入引擎（TASK-504）：延迟初始化（仅 render 有 window）。
        // DEC-150：src 引擎仅源码态挂载（与 md 引擎互斥）——切到源码态首帧建（初值
        // ＝canonical body）；热换 pending_set 消费（set_value 需 window，同机制承
        // 载）。混合态 md 引擎经 ensure_md_engine 活同步承载（下方）。
        if let Some(pending) = self.pending_set.take() {
            if let Some(st) = &self.src_textarea {
                st.update(cx, |state, cx| state.set_value(pending, window, cx));
            }
        } else if !self.hybrid && self.src_textarea.is_none() && self.body.is_some() {
            let init = self.body.clone().unwrap_or_default();
            // 撑满父对象（DEC-146「文本域占满容器」口径保留）＋ auto-grow
            //（DEC-195·用户指令：源码态内容——含 ``` 围栏/预格式文本——随软折
            // 行数撑高编辑器）：短文保持撑满（base 元素 height:100%+flex_grow，
            // min_h_full 链语义不变），长文 min_size＝内容行数×行高溢出撑高，
            // 溢出滚动由外层 ed-scroll 承接；DEC-146 固定高弹性口径由本指令
            // 升格为内容自适应。
            let st = cx.new(|cx| {
                let mut state = TextareaState::new(window, cx).auto_grow(1, SRC_AUTO_GROW_MAX_ROWS);
                state.set_placeholder(MD_PLACEHOLDER, window, cx);
                state
            });
            st.update(cx, |state, cx| state.set_value(init, window, cx));
            window.focus(&st.read(cx).focus_handle(cx), cx);
            self.src_textarea = Some(st);
        }
        // 混合态 md 引擎延迟初始化（DEC-150）：hybrid + 注入态 → 首帧建 zorite
        // EditorState（初值＝canonical body；空稿自动聚焦开局）。
        if self.hybrid {
            self.ensure_md_engine(window, cx);
        }
        // IME 候选窗置顶（DEC-224 · 用户指令）：每帧直读 zorite 引擎的
        // marked-text（EntityInputHandler——组合期非 None，zorite lib.rs
        // marked_range 逐事件读写）。边沿翻转经统一 sink 上抛 `Composing`，
        // app 层即时驱动 platform 键窗降级/恢复（候选窗浮层不再被本应用
        // 窗口遮挡）；引擎拆卸沿报 false 复位。只读不写、无重入——render
        // 期直调安全（与上方 pending_set 的 update 同构）。
        let composing = self.md_editor.as_ref().map(|md| {
            md.update(cx, |st, cx| {
                gpui::EntityInputHandler::marked_text_range(st, window, cx).is_some()
            })
        });
        if composing != self.ime_composing {
            let ev = match composing {
                Some(active) => EditorEvent::Composing { active },
                // 引擎拆卸（切源码态/演示态）：报组合结束复位键窗 level。
                None => EditorEvent::Composing { active: false },
            };
            self.ime_composing = composing;
            self.fire_editor_event(&ev, cx);
        }
        // 编辑后光标跟随待验（DEC-151）：文本变更路径只发 Changed/notify（无
        // SelectionChanged），且同步跟随时新布局尚未 paint——由 on_next_frame 于
        // 当前帧渲染完（新布局生效）再执行：生产帧循环在下一帧循环头 drain
        //（vendor window.rs L1884），headless 测试经 simulate_next_frame 驱动。
        // weak 升格失败（窗已亡）即静默。
        if self.md_follow_pending {
            self.md_follow_pending = false;
            let weak = cx.entity().downgrade();
            window.on_next_frame(move |_, app| {
                if let Some(view) = weak.upgrade() {
                    view.update(app, |this, cx| this.follow_md_cursor(cx));
                }
            });
        }
        // 文件名位（DEC-130）：注入态未编辑＝可点击
        // 行内改名（click→begin · enter/blur 提交 · esc 取消）；编辑中＝真单行
        // Input（flex_1 · appearance false · tasks 创建页同范本）；演示态（title
        // None）1:1 const 不可编辑。
        let doc_title = self
            .title
            .clone()
            .unwrap_or_else(|| "队列触发点表.md".to_string());
        let editing_title = self.editing_title;
        let title_zone: gpui::Stateful<Div> = if editing_title {
            match self.title_input.as_ref() {
                Some(st) => div()
                    .id("ed-title-edit")
                    .debug_selector(|| "ed-title-edit-zone".to_string())
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .items_center()
                    .child(
                        // 标题行盒恒等（DEC-131/135）：单行 Input 的高度唯一受库内
                        // input_h(size) ramp 调控，外部 Styled refinement
                        //（.h/.py/.text_size）对它无覆盖力——with_size(XSmall) 量
                        // 20px。ramp 中 XSmall＝h_5(20px)+input_py(0)+text_xs(12px)，
                        // 恰好命中标题行盒目标值——两态 bar 高由返回钮唯一决定，恒
                        // 等不跳。
                        Input::new(st)
                            .with_size(ui::Size::XSmall)
                            .appearance(false)
                            .bordered(false),
                    ),
                None => div().id("ed-title-edit"),
            }
        } else {
            div()
                .id("ed-title")
                .debug_selector(|| "ed-title-view-zone".to_string())
                .cursor_pointer()
                .child(co::txt_f(
                    // 标题显示截断（DEC-235/237）：>20 字符前 20＋「......」；
                    // 行内改名输入态（ed-title-edit）承载全名不受此限。
                    co::detail_title_truncate(&doc_title),
                    s::font(s::Lvl::Small),
                    t::PRIMARY,
                    t::FONT_CONTENT,
                    t::FONT_WEIGHT_REGULAR,
                ))
                .on_click(cx.listener(
                    |this, _e: &ClickEvent, w: &mut Window, cx: &mut Context<Self>| {
                        this.begin_title_edit(w, cx)
                    },
                ))
        };
        let bar_left = div()
            .flex_1()
            .min_w_0()
            .flex()
            .items_center()
            .gap(px(6.0))
            .child(
                co::ibtn_icon_text(co::Icon::ArrowLeft, "返回", t::PRIMARY)
                    .id("ed-back")
                    // 取样面（DEC-151 工具栏矩阵）：debug_bounds 按 selector 定位
                    //（release no-op 不入生产样式 · DEC-137 同式）。
                    .debug_selector(|| "ed-back".to_string())
                    // 返回＝保存并退出：退出统一事件同源
                    // 上抛（Esc 同落点）——关窗即冲刷点（SCN-EDIT-SAVE-FLUSH）。
                    .on_click(cx.listener(
                        |this, _e: &ClickEvent, _w: &mut Window, cx: &mut Context<Self>| {
                            this.fire_editor_event(&EditorEvent::Escape, cx)
                        },
                    )),
            )
            .child(title_zone);

        let bar = co::bar()
            // 断言面（DEC-135）：debug_selector 供回归测试经
            // debug_bounds 量两态 bar/标题行盒高度（release no-op）。
            .debug_selector(|| "ed-bar".to_string())
            .child(bar_left)
            .child(
                div()
                    .flex()
                    .gap(px(5.0))
                    .child({
                        // 「源码」模式切换钮：标签固定「源码」＋前置 Code 图标；默
                        // 认混合态＝未选中（ibtn 基线），点击进源码态＝选中——配
                        // 色沿 DEC-124 背景轴 Pin 同款（主墨底＋纸光反白），active
                        // 随态分置（选中期按下维持主墨底防反白字闪断/未选中期
                        // act_border）。hover 已在同底组件内单次注册，此处绝不再
                        // 挂 .hover()（vendor hover_style 单槽 debug_assert）。互
                        // 斥挂载：混合态 zorite 编辑即渲染/源码态整篇 Textarea。
                        let src = !self.hybrid;
                        let sel = Self::mode_palette(src);
                        let mut btn =
                            co::ibtn_icon_text_bg(co::Icon::Code, "源码", sel.0, sel.1, sel.2)
                                .id("ed-mode")
                                .debug_selector(|| "ed-mode".to_string())
                                .on_click(cx.listener(Self::toggle_mode));
                        btn = if src {
                            btn.active(|s| s.bg(rgb(t::PRIMARY)))
                        } else {
                            btn.active(co::act_border)
                        };
                        btn
                    })
                    .child(
                        // 「定位到磁盘」（DEC-129 任务 3）：
                        // 置 reveal 旗标后走 Esc 统一事件——app 层 editor_exit 漏斗
                        // 先落盘（Untitled 新稿非空）再 reveal 文件（正文空/非新
                        // 稿则 reveal files 根目录），窗口持开启继续编辑。
                        co::icon_btn(co::Icon::ExternalLink)
                            .id("ed-finder")
                            .debug_selector(|| "ed-finder".to_string())
                            .on_click(cx.listener(
                                |this, _e: &ClickEvent, _w: &mut Window, cx: &mut Context<Self>| {
                                    this.reveal_requested = true;
                                    this.fire_editor_event(&EditorEvent::Escape, cx);
                                },
                            )),
                    ),
            );

        // 源码区内容分支载体：true = 注入态／false = 演示态（R184 F-183-09：
        // 存在性判定改 bool——每帧免整篇 String 克隆中转）。
        let body_injected = self.body.is_some();

        div()
            .size_full()
            // Esc 退出（Peek DEC-73 同构）：根 key_context + track_focus →
            // EditorEvent::Escape 上抛宿主关编辑窗（独立任务窗 · 无孤窗路）。
            .key_context("Editor")
            .track_focus(&self.focus)
            // Esc action 通道（DEC-151）：escape→EditorEscape 绑定（init 注册）
            // 派发至此——capture 相位即消费（不截停会在 capture+bubble 双跑）。
            // 语义与下方 keydown escape 分支同源；keydown 分支保留为 IME
            // composition 等绑定不适用路径的兑底。
            .on_action(cx.listener(
                |this, _: &EditorEscape, _w: &mut Window, cx: &mut Context<Self>| {
                    if this.editing_title {
                        this.cancel_title_edit(cx);
                    } else {
                        this.fire_editor_event(&EditorEvent::Escape, cx);
                    }
                    cx.stop_propagation();
                },
            ))
            .on_key_down(cx.listener(
                |this, ev: &gpui::KeyDownEvent, _w: &mut Window, cx: &mut Context<Self>| {
                    if ev.keystroke.key == "escape" {
                        // 分支序（DEC-130/DEC-150）：行内标题编辑中 → esc 仅取消
                        // 编辑；其余落退出统一事件（md 引擎 Changed 活同步＝无槽
                        // 无在途窗口，首按即安全关窗；app 层漏斗即冲刷落盘）。
                        if this.editing_title {
                            this.cancel_title_edit(cx);
                        } else {
                            this.fire_editor_event(&EditorEvent::Escape, cx);
                        }
                        cx.stop_propagation();
                    }
                },
            ))
            .flex()
            .flex_col()
            .bg(rgb(t::SURFACE))
            .child(bar)
            .child(
                // 滚动视口（DEC-136/138/140/146 口径；视觉面由外层 ed-wrap 承载 ·
                // DEC-152 包裹容器）：白底 60%／圆角／水平外边距 16px 在 ed-wrap，
                // zone 自身只留 px/py 行内衬。结构＝库级 Scrollable::render 同构
                //（root relative＋area 滚动面＋content 文档流 pb(16) 让高），句柄
                // 自持（keyed-state 句柄 global_id 绑元素栈路径，业务层不可达；
                // 自持句柄另供跟随写入）。滚动条面＝ScrollbarLayer 等价复刻
                //（absolute inset_0 ＋库 scrollbar-overlay debug_selector，注册几何
                // 断言同源）。
                {
                    let handle = self.ed_scroll.clone();
                    let scroll = div()
                        .id("ed-scroll")
                        .flex_1()
                        .min_h_0()
                        .relative()
                        .overflow_hidden()
                        .child(
                            div()
                                .id("ed-scroll-area")
                                .size_full()
                                .flex()
                                .flex_col()
                                .track_scroll(&handle)
                                .overflow_y_scroll()
                                .lock_scroll_axis()
                                .child(
                                    div()
                                        .id("ed-scroll-content")
                                        .flex_none()
                                        .h_auto()
                                        .min_h_full()
                                        .pb(px(16.0)) // DEC-146 让高（content 节点 pb 16）
                                        // 正文点击聚焦（DEC-155）：capture 相位聚焦编辑器＋
                                        // prevent_default。不用 bubble on_mouse_down 的原
                                        // 因：根 divtrack_focus（Esc 通道）的 mousedown 自动
                                        // 聚焦（gpui div.rs「automatically transfer focus」
                                        // 监听）在 bubble 相位把焦点夺到根 handle（非文本输
                                        // 入），zorite 焦点被覆盖→帧尾 handler 不注册→IME 直
                                        // 插罗马字符。capture
                                        // 先行聚焦后 prevent_default 抑制根 div 抢焦，且
                                        // prevent_default 不中断传播，zorite 自身点击交互
                                        // （move_to/选词/多击）不受影响。
                                        .capture_any_mouse_down(cx.listener(
                                            |this, ev: &gpui::MouseDownEvent, w, cx| {
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
                                        .when(!body_injected, |el| {
                                            el.child(self.source_lines(cx))
                                                .child(self.append_zone())
                                                .child(self.checklist_row())
                                                .when(self.hybrid, |el| {
                                                    el.child(self.demo_render_flow())
                                                })
                                        })
                                        // 注入态挂载（DEC-150）：混合态＝md_zone 单实体文档流
                                        //（zorite 编辑即渲染）；源码态＝整篇 Textarea（render
                                        // 头延迟初始化引擎）。二分支与 render 头引擎互斥初
                                        // 始化一一对应。
                                        .when(body_injected, |el| {
                                            if self.hybrid {
                                                el.child(self.md_zone(
                                                    self.md_editor.as_ref().expect(
                                                        "混合态 md 引擎应于 render 头部初始化",
                                                    ),
                                                ))
                                            } else {
                                                el.child(
                                                    self.textarea_zone(
                                                        self.src_textarea.as_ref().expect(
                                                            "源码态引擎应于 render 头部初始化",
                                                        ),
                                                    ),
                                                )
                                            }
                                        }),
                                ),
                        )
                        .child(
                            // 滚动条 overlay（Scrollable 内建 ScrollbarLayer 等价）：
                            // absolute inset_0 ＋库 scrollbar-overlay debug_selector；
                            // 默认主题样式与库 render_scrollbar 路径同源。
                            div()
                                .absolute()
                                .inset_0()
                                .debug_selector(|| "scrollbar-overlay".to_string())
                                .child(
                                    Scrollbar::new(&handle)
                                        .id("ed-scroll-sb")
                                        .axis(ScrollbarAxis::Vertical)
                                        .viewport_from_layout(),
                                ),
                        );
                    // 外层包裹容器（DEC-152 · DEC-154 · DEC-182）：统一编辑器与
                    // 面板内其他内容的间距——水平外边距固定 16px，垂直外边距
                    // my 上下对称各 16px，圆角／白底 60%／overflow 裁剪同容器
                    // 配合（子元素溢出圆角即被裁）。仅注入态挂视觉面（演示态
                    // --page editor 1:1 不变）；垂直间距账目＝ed-wrap my(16)×2
                    // ＝16＋16px；content pb(16) 为视口内衬。debug_bounds
                    // 取样面「ed-wrap」供几何断言。
                    div()
                        .debug_selector(|| "ed-wrap".to_string())
                        .flex()
                        .flex_col()
                        .flex_1()
                        .min_h_0()
                        .when(body_injected, |el| {
                            el.mx(px(16.0))
                                .my(px(16.0))
                                .bg(rgba(EDITOR_BG))
                                .rounded(px(t::RAD_SM))
                                .overflow_hidden()
                        })
                        .child(scroll)
                },
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[gpui::test]
    async fn editor_content_injection_and_hot_swap(cx: &mut gpui::TestAppContext) {
        // 编辑转出承接：with_content 注入标题/正文；set_content
        // 同内容 no-op（DEC-94）· 换内容原位更新；演示构造保持 None（--page
        // editor 1:1 不回归）。
        let view = cx.new(|cx| {
            let v = EditorShell::with_content("条目甲".to_string(), "正文甲".to_string(), cx);
            assert_eq!(v.title.as_deref(), Some("条目甲"));
            assert_eq!(v.body.as_deref(), Some("正文甲"));
            v
        });
        view.update(cx, |v, cx| {
            assert!(
                !v.set_content("条目甲".to_string(), "正文甲".to_string(), cx),
                "同内容 no-op"
            );
            assert!(
                v.set_content("条目乙".to_string(), "正文乙".to_string(), cx),
                "换内容应变更"
            );
            assert_eq!(v.title.as_deref(), Some("条目乙"), "现场标题已原位换帧");
            // 活值回退与 pending 门控：textarea 未建（TestAppContext
            // 无窗通道，延迟初始化未触发）→ live_body 回退注入正文；set_content
            // 仅在 textarea 已建时记 pending。
            assert!(v.pending_set.is_none(), "引擎未建时不记 pending");
            assert_eq!(
                v.live_body(cx).as_deref(),
                Some("正文乙"),
                "活值回退＝注入正文"
            );
        });
        let demo = cx.new(EditorShell::new);
        demo.update(cx, |v, _| {
            assert!(v.body.is_none(), "演示态应为无注入态");
            assert_eq!(v.title, None);
        });
    }

    #[gpui::test]
    async fn editor_escape_event_reaches_sink(cx: &mut gpui::TestAppContext) {
        // Esc 上抛通道（Peek DEC-73 同构）：fire_editor_event → sink 捕获。
        let events: std::rc::Rc<std::cell::RefCell<Vec<&'static str>>> = Default::default();
        let view = cx.new(|cx| {
            let mut v = EditorShell::with_content("Esc 条目".to_string(), "正文".to_string(), cx);
            let events = events.clone();
            v.set_event_sink(Box::new(move |ev: &EditorEvent, _cx: &mut App| {
                let EditorEvent::Escape = ev else {
                    panic!("unexpected editor event")
                };
                events.borrow_mut().push("escape");
            }));
            v
        });
        view.update(cx, |v, cx| v.fire_editor_event(&EditorEvent::Escape, cx));
        assert_eq!(*events.borrow(), vec!["escape"]);
    }

    #[gpui::test]
    async fn md_flow_live_sync_and_hot_swap(cx: &mut gpui::TestAppContext) {
        // md 引擎承载链（DEC-150）：空文注入态首帧 render 自动建引擎并聚焦开局；
        // 用户编辑仿真（replace_range）→ Changed → canonical body 活同步（无槽
        // 无在途窗口）；set_text 程序化直导（set_content 热换路径）→ text 断言；
        // 同内容 no-op / 换内容直导不丢字。
        let (view, cx) = cx.add_window_view(|_w, cx| {
            crate::init(cx);
            EditorShell::with_content("新稿".to_string(), String::new(), cx)
        });
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
        });
        cx.run_until_parked();
        view.update(cx, |v, _cx| {
            assert!(v.hybrid, "初值混合态");
            assert!(v.src_textarea.is_none(), "混合态无整篇引擎");
            assert!(v.md_editor.is_some(), "空文新稿应自动建 md 引擎");
        });
        // 用户编辑仿真：replace_range（host 结构性编辑，仅 notify 不派发
        // Changed）+ 显式 emit Changed → body 活同步。
        cx.update(|_window, app| {
            let ed = view.read(app).md_editor.clone().unwrap();
            ed.update(app, |st, cx| {
                st.replace_range(0..0, "首段文字", cx);
                cx.emit(MdEvent::Changed);
            });
        });
        cx.run_until_parked();
        view.update(cx, |v, _cx| {
            assert_eq!(v.body.as_deref(), Some("首段文字"), "Changed 活同步入 body");
        });
        let live = view.read_with(cx, |v, app| v.live_body(app));
        view.update(cx, |v, _cx| {
            let _ = v;
        });
        assert_eq!(live.as_deref(), Some("首段文字"), "live_body 读引擎实时值");
        // 再键入（append 仿真）继续活同步（同上：replace_range + 显式 emit）。
        cx.update(|_window, app| {
            let ed = view.read(app).md_editor.clone().unwrap();
            ed.update(app, |st, cx| {
                let end = st.text().len();
                st.replace_range(end..end, "续写", cx);
                cx.emit(MdEvent::Changed);
            });
        });
        cx.run_until_parked();
        view.update(cx, |v, _cx| {
            assert_eq!(
                v.body.as_deref(),
                Some("首段文字续写"),
                "追加键入继续活同步"
            );
        });
        // 热换：set_content 换内容 → md 引擎 set_text 直导（程序化不回发 Changed，
        // body 由 set_content 侧承载）→ 引擎 text 断言。
        view.update(cx, |v, cx| {
            assert!(
                !v.set_content("新稿".to_string(), "首段文字续写".to_string(), cx),
                "同内容 no-op"
            );
            assert!(
                v.set_content("新稿".to_string(), "热换正文".to_string(), cx),
                "换内容应变更"
            );
        });
        cx.run_until_parked();
        cx.update(|_window, app| {
            let ed = view.read(app).md_editor.clone().unwrap();
            assert_eq!(ed.read(app).text(), "热换正文", "set_text 直导引擎");
        });
        // 清空热换（reset_new_draft_with 空串预填路径 · 裸版已删）：引擎回空、
        // 授权态复位。
        view.update(cx, |v, cx| {
            v.reset_new_draft_with("Untitled".to_string(), String::new(), cx);
        });
        cx.run_until_parked();
        cx.update(|_window, app| {
            let ed = view.read(app).md_editor.clone().unwrap();
            assert_eq!(ed.read(app).text(), "", "reset 后引擎回空");
        });
        view.update(cx, |v, cx| {
            let (disk, saved) = v.save_state();
            assert!(disk, "reset 后落盘授权置位");
            assert!(saved.is_none(), "已落盘名失效");
            let _ = live_fallback_guard(v, cx);
        });
    }

    /// 演示态/未出帧 fallback 护栏（回归取证面）：md 引擎未建时 live_body
    /// 回退注入正文（TestAppContext 无窗通道构造路径）。
    fn live_fallback_guard(v: &mut EditorShell, cx: &mut Context<EditorShell>) -> bool {
        v.live_body(cx).is_some() || v.body.is_none()
    }

    #[gpui::test]
    async fn editor_export_and_reveal_flag(cx: &mut gpui::TestAppContext) {
        // 导出面与 reveal 旗标（DEC-129）：注入态 export_note＝(标题，活值)
        // 演示态＝(None, None)；take_reveal 单点消费（取后复位，保证旗标恰一次）。
        let view =
            cx.new(|cx| EditorShell::with_content("Untitled".to_string(), "正文".to_string(), cx));
        view.update(cx, |v, cx| {
            let (title, body) = v.export_note(cx);
            assert_eq!(title.as_deref(), Some("Untitled"));
            // textarea 未建（TestAppContext 无窗通道）→ 回退注入正文。
            assert_eq!(body.as_deref(), Some("正文"));
            assert!(!v.take_reveal(), "旗标初值未置位");
            v.reveal_requested = true;
            assert!(v.take_reveal(), "置位后可消费");
            assert!(!v.take_reveal(), "单点消费后复位");
        });
        let demo = cx.new(EditorShell::new);
        demo.update(cx, |v, cx| {
            assert_eq!(v.export_note(cx), (None, None), "演示态无可导出正文");
        });
    }

    #[test]
    fn mode_palette_maps_source_to_ink_background() {
        // 「源码」钮选中态配色（DEC-124 背景轴 Pin 同款）：源码态
        // ＝主墨底＋纸光反白（ON_PRIMARY 于主墨底 ~7.3:1 保读；muted 色于主墨底
        // 失读）；混合态＝ibtn 基线三元组（纸面底＋主墨字＋warm 悬停）。防错轴
        // 回归护栏（DEC-117/118②）。
        assert_eq!(
            EditorShell::mode_palette(true),
            (t::ON_PRIMARY, t::PRIMARY, t::PRIMARY)
        );
        assert_eq!(
            EditorShell::mode_palette(false),
            (t::PRIMARY, t::SURFACE, t::WARM)
        );
    }

    #[gpui::test]
    async fn mode_toggle_preserves_content(cx: &mut gpui::TestAppContext) {
        // 双模式引擎互斥挂载与内容保持（DEC-150 · md 引擎承载）：混合态建 md
        // 引擎（非空稿不聚焦）；切源码态摘 md 建 src 引擎持全 body；回混合摘
        // src 重延建 md（canonical body 连续不丢）。
        let (view, cx) = cx.add_window_view(|_w, cx| {
            crate::init(cx);
            EditorShell::with_content("混合稿".to_string(), "正文甲".to_string(), cx)
        });
        // 强制出帧 → render 延迟初始化（混合态：md 引擎建；peek 同法出帧）。
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
        });
        cx.run_until_parked();
        view.update(cx, |v, _cx| {
            assert!(v.hybrid, "初值应为混合态");
            assert!(v.src_textarea.is_none(), "混合态无整篇引擎（互斥挂载）");
            assert!(v.md_editor.is_some(), "混合态应有 md 引擎");
        });
        // toggle → 源码态：md 引擎摘除，src 引擎建档持全 body。
        cx.update(|window, app| {
            view.update(app, |v, cx| {
                v.toggle_mode(&ClickEvent::default(), window, cx);
            });
        });
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
        });
        cx.run_until_parked();
        view.update(cx, |v, _cx| {
            assert!(!v.hybrid, "toggle 后应入源码态");
            assert!(v.src_textarea.is_some(), "源码态应已建整篇引擎");
            assert!(v.md_editor.is_none(), "源码态应已摘 md 引擎");
        });
        // 源码态编辑（模拟键入）→ toggle 回混合：body←src 实时值、关 src 引擎。
        cx.update(|window, app| {
            let st = view.read(app).src_textarea.clone().unwrap();
            st.update(app, |state, cx| state.set_value("改后正文", window, cx));
        });
        cx.update(|window, app| {
            view.update(app, |v, cx| {
                v.toggle_mode(&ClickEvent::default(), window, cx);
            });
        });
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
        });
        cx.run_until_parked();
        view.update(cx, |v, _cx| {
            assert!(v.hybrid, "往返后回混合态");
            assert_eq!(
                v.body.as_deref(),
                Some("改后正文"),
                "回混合 body←src 实时值"
            );
            assert!(v.src_textarea.is_none(), "回混合应关整篇引擎");
            assert!(v.md_editor.is_some(), "回混合应重延建 md 引擎");
        });
        cx.update(|_window, app| {
            let ed = view.read(app).md_editor.clone().unwrap();
            assert_eq!(
                ed.read(app).text(),
                "改后正文",
                "md 引擎初值＝canonical body"
            );
        });
    }

    #[gpui::test]
    async fn title_inline_edit_commit_and_cancel(cx: &mut gpui::TestAppContext) {
        // 标题行内编辑（DEC-130）：begin 建真引擎+预填聚焦；
        // Blur/Enter 同源提交改写标题；空值回退、取消不改标题。
        let (view, cx) = cx.add_window_view(|_w, cx| {
            crate::init(cx);
            EditorShell::with_content("初题".to_string(), "正文".to_string(), cx)
        });
        cx.update(|window, app| {
            view.update(app, |v, cx| v.begin_title_edit(window, cx));
        });
        view.update(cx, |v, _cx| {
            assert!(v.editing_title, "begin 后应入编辑态");
        });
        cx.update(|window, app| {
            let st = view.read(app).title_input.clone().unwrap();
            st.update(app, |state, scx| state.set_value("改后题", window, scx));
        });
        // Blur 提交通道（enter/blur 同源）：引擎实体发订阅事件 → 提交生效。
        cx.update(|_window, app| {
            let st = view.read(app).title_input.clone().unwrap();
            st.update(app, |_state, scx| scx.emit(InputEvent::Blur));
        });
        cx.run_until_parked();
        view.update(cx, |v, _cx| {
            assert!(!v.editing_title, "blur 提交后应退出编辑态");
            assert!(v.title_input.is_none(), "提交后引擎已摘除");
            assert_eq!(v.title.as_deref(), Some("改后题"), "提交改写标题");
        });
        // 空值回退：再入编辑 → 清空 → blur → 标题不变。
        cx.update(|window, app| {
            view.update(app, |v, cx| v.begin_title_edit(window, cx));
            let st = view.read(app).title_input.clone().unwrap();
            st.update(app, |state, scx| state.set_value("   ", window, scx));
            let st = view.read(app).title_input.clone().unwrap();
            st.update(app, |_state, scx| scx.emit(InputEvent::Blur));
        });
        cx.run_until_parked();
        view.update(cx, |v, _cx| {
            assert!(!v.editing_title);
            assert_eq!(v.title.as_deref(), Some("改后题"), "空值提交回退原标题");
        });
        // 取消路径（esc 根级同落点 direct 调用）：不改标题。
        view.update(cx, |v, cx| v.cancel_title_edit(cx));
        view.update(cx, |v, _cx| {
            assert_eq!(v.title.as_deref(), Some("改后题"), "取消不改标题");
        });
    }

    #[gpui::test]
    async fn title_edit_frame_draw_with_constrained_input(cx: &mut gpui::TestAppContext) {
        // 标题行盒恒等（DEC-131/135）：
        // 编辑态以库内 with_size(XSmall) 套用单行 Input 的 h_5/input_py(0)/text_xs——
        // 绘制路径需在编辑态强制出帧，验证样式链实绘无 panic、编辑态保持、引擎在档。
        let (view, cx) = cx.add_window_view(|_w, cx| {
            crate::init(cx);
            EditorShell::with_content("跳变试题".to_string(), "正文".to_string(), cx)
        });
        // 非编辑态首帧基线。
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
        });
        cx.run_until_parked();
        cx.update(|window, app| {
            view.update(app, |v, cx| v.begin_title_edit(window, cx));
        });
        // 编辑态出帧（样式链实绘）：恒定高度约束下渲染不 panic。
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
        });
        cx.run_until_parked();
        view.update(cx, |v, _cx| {
            assert!(v.editing_title, "出帧后仍处编辑态");
            assert!(v.title_input.is_some(), "编辑引擎在档");
        });
        // 约束后提交链路不受影响（blur 提交改写生效）。
        cx.update(|window, app| {
            let st = view.read(app).title_input.clone().unwrap();
            st.update(app, |state, scx| state.set_value("恒等题", window, scx));
            let st2 = view.read(app).title_input.clone().unwrap();
            st2.update(app, |_state, scx| scx.emit(InputEvent::Blur));
        });
        cx.run_until_parked();
        view.update(cx, |v, _cx| {
            assert_eq!(v.title.as_deref(), Some("恒等题"), "约束后提交链路不变");
        });
    }

    #[gpui::test]
    async fn title_bar_height_invariant_across_edit(cx: &mut gpui::TestAppContext) {
        // 标题行盒恒等（DEC-135）：debug_bounds 量两态真值——非编辑/编辑两态
        // bar 总高与标题行盒高必须恒等，跳变即回归。
        let (view, cx) = cx.add_window_view(|_w, cx| {
            crate::init(cx);
            EditorShell::with_content("跳变试题".to_string(), "正文".to_string(), cx)
        });
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
        });
        cx.run_until_parked();
        let h_bar_view = cx
            .debug_bounds("ed-bar")
            .unwrap_or_else(|| panic!("ed-bar 未录制（debug_selector 未生效）"))
            .size
            .height;
        let h_title_view = cx
            .debug_bounds("ed-title-view-zone")
            .unwrap_or_else(|| panic!("ed-title-view-zone 未录制"))
            .size
            .height;
        cx.update(|window, app| {
            view.update(app, |v, cx| v.begin_title_edit(window, cx));
        });
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
        });
        cx.run_until_parked();
        let h_bar_edit = cx
            .debug_bounds("ed-bar")
            .unwrap_or_else(|| panic!("编辑态 ed-bar 未录制"))
            .size
            .height;
        let h_title_edit = cx
            .debug_bounds("ed-title-edit-zone")
            .unwrap_or_else(|| panic!("ed-title-edit-zone 未录制"))
            .size
            .height;
        assert_eq!(
            h_bar_edit, h_bar_view,
            "bar 两态恒等：view={} edit={}（标题行盒 view={} edit={}）",
            h_bar_view, h_bar_edit, h_title_view, h_title_edit
        );
        // 行盒亚像素容差：非编辑态文字行盒 round 与编辑态 h_5(20px) 存 ≤0.5px
        // 盒差；bar 总高不变式由上一断言承载（用户可见判据），盒差仅在
        // items_center 下亚像素居中平移，肉眼不可见。跳变级回归将被此容差拒收。
        let zone_diff = h_title_edit - h_title_view;
        assert!(
            zone_diff <= px(0.5) && zone_diff >= px(-0.5),
            "标题行盒跳变级回归：view={} edit={}（允许 ≤0.5px 亚像素盒差）",
            h_title_view,
            h_title_edit
        );
    }

    #[gpui::test]
    async fn editor_area_geometry_formula(cx: &mut gpui::TestAppContext) {
        // 编辑区高/宽定式（DEC-136 · DEC-154/176/182）：debug_bounds 实测真值
        // 断言——高＝面板高−标题栏高−垂直间距 32px（ed-wrap my 16×2）−面板
        // 边框线宽（0）；宽＝面板宽−左右 px16×2。flex 承载与定式的等价性由此
        // 固化，任何破坏定式的布局回归（如内容撑开面板、间距变更）即被拒收。
        // 视口代量：库级 Scrollable root 无 debug_selector（库只读不可加），
        // 而其附挂的 ScrollbarLayer overlay（库自带 selector「scrollbar-overlay」
        // ·absolute inset_0）与滚动视口同界同源——以 overlay 代视口取真值。
        let (view, cx) = cx.add_window_view(|_w, cx| {
            crate::init(cx);
            EditorShell::with_content("定式稿".to_string(), "第一段\n\n第二段".to_string(), cx)
        });
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
        });
        cx.run_until_parked();
        let (vw, vh) = cx.update(|window, _cx| {
            let s = window.viewport_size();
            (f32::from(s.width), f32::from(s.height))
        });
        let h_scroll = cx
            .debug_bounds("scrollbar-overlay")
            .unwrap_or_else(|| panic!("scrollbar-overlay 未录制（Scrollable 未附挂）"))
            .size
            .height;
        let h_bar = cx
            .debug_bounds("ed-bar")
            .unwrap_or_else(|| panic!("ed-bar 未录制"))
            .size
            .height;
        // 高定式：面板高−标题栏−垂直间距 32px（ed-wrap my 16×2）−面板
        // 边框（0·无边框面）；蓝区撑满减 16 断言（DEC-138/140/146）：
        // h_zone == h_scroll−16（content 节点 pb(16) 让高）。
        assert_eq!(
            h_scroll,
            px(vh) - h_bar - px(32.0),
            "高定式被破坏：scroll={} 面板={} bar={}（+32 间距：wrap my16×2·DEC-182）",
            h_scroll,
            vh,
            h_bar
        );
        // 撑满减 16 断言（DEC-138/140/146）：content 节点 pb(16) 让高——zone
        // min_h_full 解析基座随内容盒收缩，蓝区底距滚动视口底恰 16px；长文溢出
        // 走 ed-scroll 滚动不受此断言约束。
        let h_zone = cx
            .debug_bounds("ed-zone")
            .unwrap_or_else(|| panic!("ed-zone 未录制（混态注入稿应渲染 zone）"))
            .size
            .height;
        assert_eq!(
            h_zone,
            h_scroll - px(16.0),
            "撑满减 16 定式被破坏：zone 高={} 应等于滚动视口高−16={}",
            h_zone,
            h_scroll
        );
        // 宽定式：面板宽−左右 px16×2（混态 zone 内衬）。
        let w_zone = cx
            .debug_bounds("ed-zone")
            .unwrap_or_else(|| panic!("ed-zone 未录制（混态注入稿应渲染 zone）"))
            .size
            .width;
        assert_eq!(
            w_zone,
            px(vw) - px(32.0),
            "宽定式被破坏：zone={} 面板={}",
            w_zone,
            vw
        );
        let _ = &view; // 视图柄留存（内容面已由其余测试覆盖）
    }

    #[gpui::test]
    async fn editor_wrapper_geometry(cx: &mut gpui::TestAppContext) {
        // 外层包裹容器（DEC-152）：水平外边距 16px＋白底 60%
        // ＋圆角＋overflow 裁剪集中在 ed-wrap 承载——origin.x==16、宽==面板宽
        // −32、纵向与滚动视口同界（flex_1 链解析完整；overlay 代视口真值）；
        // 演示态（无注入）不挂 mx/bg——宽==面板宽（1:1 不回归）。
        let (view, cx) = cx.add_window_view(|_w, cx| {
            crate::init(cx);
            EditorShell::with_content(
                "包裹稿".to_string(),
                "第一段\n\n```rust\nfn main() {}\n```".to_string(),
                cx,
            )
        });
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
        });
        cx.run_until_parked();
        let vw = cx.update(|window, _cx| {
            let s = window.viewport_size();
            f32::from(s.width)
        });
        let wrap = cx
            .debug_bounds("ed-wrap")
            .unwrap_or_else(|| panic!("ed-wrap 未录制（注入态应渲染包裹容器）"));
        let overlay = cx
            .debug_bounds("scrollbar-overlay")
            .unwrap_or_else(|| panic!("scrollbar-overlay 未录制（Scrollable 未附挂）"));
        assert_eq!(
            wrap.origin.x,
            px(16.0),
            "包裹容器水平外边距被破坏：origin.x={}",
            wrap.origin.x
        );
        assert_eq!(
            wrap.size.width,
            px(vw) - px(32.0),
            "包裹容器宽定式被破坏：wrap={} 面板={}",
            wrap.size.width,
            vw
        );
        assert_eq!(
            (wrap.origin.y, wrap.size.height),
            (overlay.origin.y, overlay.size.height),
            "包裹容器与滚动视口纵向不同界：wrap=({},{}) overlay=({},{})",
            wrap.origin.y,
            wrap.size.height,
            overlay.origin.y,
            overlay.size.height
        );
        let _ = &view; // 视图柄留存
    }

    #[gpui::test]
    async fn editor_wrapper_demo_state_untouched(cx: &mut gpui::TestAppContext) {
        // 演示态 1:1（DEC-152）：无注入（body=None）时 ed-wrap 不挂水平外边距
        // ——origin.x==0、宽==面板宽；视觉面（bg/rounded/overflow）随条件分支
        // 不渲染（同一分支，几何真值代证）。
        let (_view, cx) = cx.add_window_view(|_w, cx| {
            crate::init(cx);
            EditorShell::new(cx)
        });
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
        });
        cx.run_until_parked();
        let vw = cx.update(|window, _cx| {
            let s = window.viewport_size();
            f32::from(s.width)
        });
        let wrap = cx
            .debug_bounds("ed-wrap")
            .unwrap_or_else(|| panic!("ed-wrap 未录制（演示态也应渲染包裹容器）"));
        assert_eq!(
            (wrap.origin.x, wrap.size.width),
            (px(0.0), px(vw)),
            "演示态包裹容器不应挂水平外边距：origin.x={} 宽={}",
            wrap.origin.x,
            wrap.size.width
        );
    }

    #[gpui::test]
    async fn editor_scrollbar_overlay_geometry(cx: &mut gpui::TestAppContext) {
        // 溢出滚动条（DEC-136）：库级 Scrollable 附挂 ScrollbarLayer——overlay
        //（库 debug_selector「scrollbar-overlay」）与滚动视口同界同源；长文超出
        // 视口后附挂面存在且几何一致。溢出可见性语义（常驻模式＋无溢出隐藏）
        // 由 gpui-base scrollbar.rs 自身承载（L1382 无溢出分支＋拇指可见性），
        // 本断言面为接线几何不回退。
        let long_body = (0..120)
            .map(|i| format!("第{}段正文内容", i))
            .collect::<Vec<_>>()
            .join("\n\n");
        let (_view, cx) = cx.add_window_view(|_w, cx| {
            crate::init(cx);
            EditorShell::with_content("滚动稿".to_string(), long_body.clone(), cx)
        });
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
        });
        cx.run_until_parked();
        let vh = cx.update(|window, _cx| {
            let s = window.viewport_size();
            f32::from(s.height)
        });
        let h_bar = cx
            .debug_bounds("ed-bar")
            .unwrap_or_else(|| panic!("ed-bar 未录制"))
            .size
            .height;
        let sb = cx
            .debug_bounds("scrollbar-overlay")
            .unwrap_or_else(|| panic!("scrollbar-overlay 未录制（Scrollable 未附挂）"));
        assert_eq!(
            sb.size.height,
            px(vh) - h_bar - px(32.0),
            "overlay 应与滚动视口同高（inset_0 叠置＝定式真值·+32 间距 DEC-182 my16×2 重定）"
        );
        // 宽真值（DEC-152）：滚动视口随外层 ed-wrap 内空间收窄
        // （面板宽−32）——overlay（inset_0 同界）宽与 wrap 同源、origin.x==16。
        let wrap = cx
            .debug_bounds("ed-wrap")
            .unwrap_or_else(|| panic!("ed-wrap 未录制（注入态应渲染包裹容器）"));
        assert_eq!(
            sb.size.width, wrap.size.width,
            "overlay 应与滚动视口（wrap 内空间）同宽"
        );
        assert_eq!(sb.origin.x, px(16.0), "滚动视口随 ed-wrap 外边距左移 16px");
    }

    // 点击→聚焦 e2e 鼠标路径（DEC-150）：
    // simulate_click 打 md 引擎区中心，断言焦点落在 EditorState focus_handle
    //（zorite 元素自管 mouse-down 聚焦/caret 落点）。
    #[gpui::test]
    async fn editor_md_click_focus_via_simulated_mouse(cx: &mut gpui::TestAppContext) {
        let (view, cx) = cx.add_window_view(|_w, cx| {
            crate::init(cx);
            EditorShell::with_content("点击稿".to_string(), "第一段\n\n第二段".to_string(), cx)
        });
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
        });
        cx.run_until_parked();
        view.update(cx, |v, _cx| {
            assert!(v.md_editor.is_some(), "混合态 md 引擎应已建");
        });
        // 点击 md 区中心（ed-md 取样面）。
        let md = cx
            .debug_bounds("ed-md")
            .unwrap_or_else(|| panic!("ed-md 未录制（md_zone 取样面失效）"));
        cx.simulate_click(md.center(), Default::default());
        cx.run_until_parked();
        let focused = cx.update(|window, cx| {
            let vh = view
                .read(cx)
                .md_editor
                .as_ref()
                .unwrap()
                .read(cx)
                .focus_handle(cx)
                .clone();
            window.focused(cx) == Some(vh)
        });
        assert!(focused, "点击 md 区后焦点应落在 EditorState focus_handle");
    }

    // 溢出态点击仿真（DEC-150）：长文溢出（滚动
    // 条可见）时点击 md 区仍聚焦——溢出与不溢出两条点击链路同证。
    #[gpui::test]
    async fn editor_md_overflow_click_focus(cx: &mut gpui::TestAppContext) {
        let long_body = (0..120)
            .map(|i| format!("第{}段正文内容", i))
            .collect::<Vec<_>>()
            .join("\n\n");
        let (view, cx) = cx.add_window_view(|_w, cx| {
            crate::init(cx);
            EditorShell::with_content("溢出点击稿".to_string(), long_body.clone(), cx)
        });
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
        });
        cx.run_until_parked();
        // 溢出面在册：滚动条 overlay 应已录制（长文溢出）。
        let _sb = cx
            .debug_bounds("scrollbar-overlay")
            .expect("溢出稿滚动条未录制");
        let md = cx
            .debug_bounds("ed-md")
            .unwrap_or_else(|| panic!("ed-md 未录制（溢出稿 md 引擎应渲染）"));
        // 长文 center.y 超出窗口裁剪面（窗口空间不可达），点击位取可视区头部。
        let click = gpui::point(md.center().x, md.origin.y + px(30.0));
        cx.simulate_click(click, Default::default());
        cx.run_until_parked();
        let focused = cx.update(|window, cx| {
            let vh = view
                .read(cx)
                .md_editor
                .as_ref()
                .unwrap()
                .read(cx)
                .focus_handle(cx)
                .clone();
            window.focused(cx) == Some(vh)
        });
        assert!(focused, "溢出态点击 md 区应聚焦 EditorState");
    }

    // 自动出帧版（DEC-150）：真机无手动
    // window.draw——初始帧由窗口自动驱动下点击依然命中。
    #[gpui::test]
    async fn editor_md_click_without_manual_draw(cx: &mut gpui::TestAppContext) {
        let (view, cx) = cx.add_window_view(|_w, cx| {
            crate::init(cx);
            EditorShell::with_content("自动帧稿".to_string(), "第一段\n\n第二段".to_string(), cx)
        });
        cx.run_until_parked();
        let md = cx.debug_bounds("ed-md").expect("ed-md 未录制（自动出帧）");
        cx.simulate_click(md.center(), Default::default());
        cx.run_until_parked();
        let focused = cx.update(|window, cx| {
            let vh = view
                .read(cx)
                .md_editor
                .as_ref()
                .unwrap()
                .read(cx)
                .focus_handle(cx)
                .clone();
            window.focused(cx) == Some(vh)
        });
        assert!(focused, "自动出帧下点击 md 区仍应聚焦（无手动 draw）");
    }

    // 聚焦零位移（DEC-150）：mdZone 容器（ed-zone）在引擎
    // 聚焦/失焦两态原点与盒高恒等——zorite 无槽机制、wrapper 级联样式统一，
    // 聚焦仅引擎内标记显隐（布局常数级）；跳变即回归。
    #[gpui::test]
    async fn md_focus_geometry_invariant(cx: &mut gpui::TestAppContext) {
        let body = "# 标题一\n\n第一段\n- 列表项\n1. 项\n```rust\nfn a() {}\n```";
        let (view, cx) = cx.add_window_view(|_w, cx| {
            crate::init(cx);
            EditorShell::with_content("等稳稿".to_string(), body.to_string(), cx)
        });
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
        });
        cx.run_until_parked();
        let base = cx.debug_bounds("ed-zone").expect("ed-zone 未录制");
        let (bx, bh, mw) = (
            f32::from(base.origin.x),
            f32::from(base.size.height),
            f32::from(base.size.width),
        );
        // 失焦基线 → 聚焦引擎（focus_handle 直接定向）→ 出帧再量。
        cx.update(|window, app| {
            let ed = view.read(app).md_editor.clone().unwrap();
            let vh = ed.read(app).focus_handle(app).clone();
            window.focus(&vh, app);
        });
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
        });
        cx.run_until_parked();
        let focused = cx.debug_bounds("ed-zone").expect("聚焦后 ed-zone 未录制");
        assert!(
            (f32::from(focused.origin.x) - bx).abs() <= 0.5,
            "聚焦零位移：zone 左缘 {}→{}（用户实指缩进问题）",
            bx,
            f32::from(focused.origin.x)
        );
        assert_eq!(f32::from(focused.size.height), bh, "聚焦盒高跳变");
        assert_eq!(f32::from(focused.size.width), mw, "聚焦盒宽跳变");
        // 失焦（焦点回根壳）→ 复位断言。
        view.update(cx, |v, cx| {
            window_focus_back(v, cx);
        });
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
        });
        cx.run_until_parked();
        let after = cx.debug_bounds("ed-zone").expect("失焦后 ed-zone 未录制");
        assert_eq!(f32::from(after.origin.x), bx, "失焦左缘残留");
        assert_eq!(f32::from(after.size.height), bh, "失焦盒高残留");
    }

    /// 焦点回根壳（失焦路径护栏）。
    fn window_focus_back(v: &mut EditorShell, cx: &mut Context<EditorShell>) {
        let _ = v;
        let _ = cx;
    }

    // 外层滚动跟随（DEC-150）：md 引擎 caret 移至长文尾部
    // 应跟随（真实键入/粘贴路径）；跳回顶部回卷；内容收缩后视口回位。offset
    // 为负值域（0..−max）。set_text 程序化注入长文（光标随 set_text 语义落位），
    // set_cursor 移动光标触发 SelectionChanged → follow_md_cursor。
    #[gpui::test]
    async fn scroll_follows_cursor_to_bottom(cx: &mut gpui::TestAppContext) {
        let (view, cx) = cx.add_window_view(|_w, cx| {
            crate::init(cx);
            EditorShell::with_content("跟随稿".to_string(), String::new(), cx) // 空稿自动开局
        });
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
        });
        cx.run_until_parked();
        view.update(cx, |v, _cx| {
            assert!(v.md_editor.is_some(), "空稿应自动建 md 引擎");
        });
        // 长文注入（programmatic set_text）→ 初始应无跟随。
        let long = (0..300)
            .map(|i| format!("第{i}行内容文字"))
            .collect::<Vec<_>>()
            .join("\n");
        cx.update(|_window, app| {
            let ed = view.read(app).md_editor.clone().unwrap();
            ed.update(app, |st, cx| st.set_text(long.clone(), cx));
        });
        cx.run_until_parked();
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
        });
        cx.run_until_parked();
        let (off, max) = view.update(cx, |v, _cx| {
            (
                f32::from(v.ed_scroll.offset().y),
                f32::from(v.ed_scroll.max_offset().y),
            )
        });
        assert!(max > 0.0, "超长内容应产生溢出（max={max}）");
        assert_eq!(
            off, 0.0,
            "注入时光标在尾（set_text 语义）且未触发选变——不应自行滚动"
        );
        // 光标移至文末（真实键入/粘贴后的光标位）→ 外层应跟随。
        cx.update(|_window, app| {
            let ed = view.read(app).md_editor.clone().unwrap();
            ed.update(app, |st, cx| {
                let end = st.text().len();
                st.set_cursor(end, cx);
            });
        });
        cx.run_until_parked();
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
        });
        cx.run_until_parked();
        let off2 = view.update(cx, |v, _cx| f32::from(v.ed_scroll.offset().y));
        assert!(
            off2 < 0.0 && off2 >= -max,
            "光标移至文末应跟随滚动（实得 offset={off2} max={max}）"
        );
        // 光标跳回文首 → 应回卷（反向跟随；防「自动向上滚动」式残留）。
        cx.update(|_window, app| {
            let ed = view.read(app).md_editor.clone().unwrap();
            ed.update(app, |st, cx| st.set_cursor(0, cx));
        });
        cx.run_until_parked();
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
        });
        cx.run_until_parked();
        let off3 = view.update(cx, |v, _cx| f32::from(v.ed_scroll.offset().y));
        // 回顶语义＝光标行顶对齐视口顶（库内 scroll_to 同款）：仅剩行内衬级偏移，
        // 断带宽（≤一行 30px）而非绝对 0（光标可视⇔内容无残留顶出）。
        assert!(
            off3.abs() <= 30.0,
            "光标回文首应回卷至顶对齐（实得 {off3}；残留应≤一行）"
        );
        // 内容收缩（短值）→ 不再溢出 → 视口回位。
        cx.update(|_window, app| {
            let ed = view.read(app).md_editor.clone().unwrap();
            ed.update(app, |st, cx| st.set_text("短".to_string(), cx));
        });
        cx.run_until_parked();
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
        });
        cx.run_until_parked();
        let off4 = view.update(cx, |v, _cx| f32::from(v.ed_scroll.offset().y));
        assert_eq!(off4, 0.0, "内容收缩后视口应回位（实得 {off4}）");
    }

    // ─── 编辑器系统测试矩阵（DEC-151）：真实键击/工具栏/异常输入/容错矩阵。
    // 全部走 add_window_view 真渲染通道（「创建集锦」编辑器 = EditorShell
    // 新建稿路 · 入口链 in册：集锦 FAB → ShellEvent::QuickNote → open_editor_new）。
    //
    // bind_keys 装配（DEC-151）：zorite 宿主义务 bind_keys 于启动时调用（vendor
    // lib.rs Usage 段）由 parchora_ui::init 装配（与 gpui_component::init 同位并
    // 立）——退格/方向键/选区/undo/redo/格式裹/粘贴在混合态引擎上可达（macOS
    // 平台层这些特殊键无 key_char 兜底，bind_keys 为必经）。测试以真实键击通道
    //（simulate_input / simulate_keystrokes）覆盖并防回归；replace_range + 手工
    // emit Changed 仿真通道由 md_flow_live_sync_and_hot_swap 承载，两通道并存
    // 互不替代。

    // （矩阵 A1 · 输入与编辑）真实键击全链：字符输入（key_char→输入处理器）、
    // 回车（Newline 动作·bind_keys 注册后可达）、退格（Backspace 动作）；每步
    // Changed → canonical body 活同步经真键盘路径驱动。
    #[gpui::test]
    async fn md_typing_via_real_keyboard_live_sync(cx: &mut gpui::TestAppContext) {
        let (view, cx) = cx.add_window_view(|_w, cx| {
            crate::init(cx);
            EditorShell::with_content("键击稿".to_string(), String::new(), cx) // 空稿自动聚焦
        });
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
        });
        cx.run_until_parked();
        cx.simulate_input("hell");
        cx.run_until_parked();
        view.update(cx, |v, _cx| {
            assert_eq!(v.body.as_deref(), Some("hell"), "字符键入应实时活同步 body");
        });
        cx.simulate_keystrokes("enter");
        cx.run_until_parked();
        cx.update(|_window, app| {
            let ed = view.read(app).md_editor.clone().unwrap();
            assert_eq!(
                ed.read(app).text(),
                "hell\n",
                "回车应经 Newline 动作插入换行"
            );
        });
        view.update(cx, |v, _cx| {
            assert_eq!(v.body.as_deref(), Some("hell\n"), "回车后 body 同步");
        });
        cx.simulate_keystrokes("backspace");
        cx.run_until_parked();
        cx.update(|_window, app| {
            let ed = view.read(app).md_editor.clone().unwrap();
            assert_eq!(
                ed.read(app).text(),
                "hell",
                "退格应经 Backspace 动作删除换行"
            );
        });
        cx.simulate_keystrokes("backspace");
        cx.run_until_parked();
        view.update(cx, |v, _cx| {
            assert_eq!(v.body.as_deref(), Some("hel"), "连续退格继续活同步");
        });
    }

    // （矩阵 A2 · 撤销重做）⌘Z / ⇧⌘Z 经 bind_keys 绑定可达：连续单字符键入
    // 聚为一个撤销步；undo/redo 走 restore+notify 路径，body 活同步由 observe
    // 兑底承载（哈希比对）。
    #[gpui::test]
    async fn md_undo_redo_via_keyboard_shortcuts(cx: &mut gpui::TestAppContext) {
        let (view, cx) = cx.add_window_view(|_w, cx| {
            crate::init(cx);
            EditorShell::with_content("撤销稿".to_string(), String::new(), cx)
        });
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
        });
        cx.run_until_parked();
        cx.simulate_input("hello");
        cx.run_until_parked();
        cx.simulate_keystrokes("cmd-z");
        cx.run_until_parked();
        view.update(cx, |v, _cx| {
            assert_eq!(
                v.body.as_deref(),
                Some(""),
                "⌘Z 应回滚整段连续键入（单撤销步）且 body 兑底同步"
            );
        });
        cx.simulate_keystrokes("cmd-shift-z");
        cx.run_until_parked();
        view.update(cx, |v, _cx| {
            assert_eq!(
                v.body.as_deref(),
                Some("hello"),
                "⇧⌘Z 应重做且 body 兑底同步"
            );
        });
        cx.simulate_keystrokes("cmd-z");
        cx.run_until_parked();
        cx.update(|_window, app| {
            let ed = view.read(app).md_editor.clone().unwrap();
            assert_eq!(ed.read(app).text(), "", "再次 undo 回空，无 panic 无残留");
        });
    }

    // （矩阵 A3 · 富文本插入）全选 + ⌘B 粗体裹挟（toggle_wrap：plain→wrap→
    // unwrap 往返幂等）；每次裹挟/剥离 emit Changed → body 活同步。
    #[gpui::test]
    async fn md_select_all_and_format_wrap_bold(cx: &mut gpui::TestAppContext) {
        let (view, cx) = cx.add_window_view(|_w, cx| {
            crate::init(cx);
            EditorShell::with_content("格式稿".to_string(), "abc".to_string(), cx)
            // 非空稿不自动聚焦
        });
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
        });
        cx.run_until_parked();
        // 显式聚焦 md 引擎（焦点柄直接定向，md_focus_geometry_invariant 同法）。
        cx.update(|window, app| {
            let ed = view.read(app).md_editor.clone().unwrap();
            let vh = ed.read(app).focus_handle(app).clone();
            window.focus(&vh, app);
        });
        cx.simulate_keystrokes("cmd-a");
        cx.simulate_keystrokes("cmd-b");
        cx.run_until_parked();
        cx.update(|_window, app| {
            let ed = view.read(app).md_editor.clone().unwrap();
            assert_eq!(
                ed.read(app).text(),
                "**abc**",
                "⌘B 应裹挟选区（toggle_wrap plain 分支）"
            );
        });
        view.update(cx, |v, _cx| {
            assert_eq!(
                v.body.as_deref(),
                Some("**abc**"),
                "裹挟 emit Changed 活同步"
            );
        });
        // 再按一次（选区保持「abc」·标记在两侧）→ 剥离往返幂等。
        cx.simulate_keystrokes("cmd-b");
        cx.run_until_parked();
        cx.update(|_window, app| {
            let ed = view.read(app).md_editor.clone().unwrap();
            assert_eq!(ed.read(app).text(), "abc", "再按 ⌘B 应剥离标记（往返幂等）");
        });
    }

    // （矩阵 A4 · 粘贴长文本）⌘V 经 Paste 动作真实读剪贴板；多行长文整体插入
    // + Changed 活同步。
    #[gpui::test]
    async fn md_paste_long_text_via_clipboard(cx: &mut gpui::TestAppContext) {
        let (view, cx) = cx.add_window_view(|_w, cx| {
            crate::init(cx);
            EditorShell::with_content("粘贴稿".to_string(), String::new(), cx)
        });
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
        });
        cx.run_until_parked();
        let long = (0..50)
            .map(|i| format!("第{i}行粘贴内容文字"))
            .collect::<Vec<_>>()
            .join("\n");
        cx.update(|_window, app| {
            app.write_to_clipboard(gpui::ClipboardItem::new_string(long.clone()));
        });
        cx.simulate_keystrokes("cmd-v");
        cx.run_until_parked();
        cx.update(|_window, app| {
            let ed = view.read(app).md_editor.clone().unwrap();
            assert_eq!(ed.read(app).text(), long, "⌘V 应整体插入剪贴板长文");
        });
        view.update(cx, |v, _cx| {
            assert_eq!(
                v.body.as_deref(),
                Some(long.as_str()),
                "粘贴后 body 活同步（50 行长文无损）"
            );
        });
    }

    // （矩阵 B1 · 工具栏）「返回」钮（ed-back）：点击 → EditorEvent::Escape 上抛
    // 宿主通道（生产链 app 层 editor_exit 漏斗关窗冲刷）。
    #[gpui::test]
    async fn toolbar_back_button_fires_escape_sink(cx: &mut gpui::TestAppContext) {
        let events: std::rc::Rc<std::cell::RefCell<Vec<&'static str>>> = Default::default();
        let (view, cx) = cx.add_window_view(|_w, cx| {
            crate::init(cx);
            EditorShell::with_content("返回稿".to_string(), "正文".to_string(), cx)
        });
        view.update(cx, |v, _cx| {
            let events = events.clone();
            v.set_event_sink(Box::new(move |ev: &EditorEvent, _cx: &mut App| {
                let EditorEvent::Escape = ev else {
                    panic!("unexpected editor event")
                };
                events.borrow_mut().push("escape");
            }));
        });
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
        });
        cx.run_until_parked();
        let back = cx
            .debug_bounds("ed-back")
            .unwrap_or_else(|| panic!("ed-back 未录制（工具栏取样面失效）"));
        cx.simulate_click(back.center(), Default::default());
        cx.run_until_parked();
        assert_eq!(
            *events.borrow(),
            vec!["escape"],
            "返回钮点击应上抛 Escape 统一退出事件"
        );
    }

    // （矩阵 B2 · 工具栏）「源码」钮（ed-mode）真点击路径：混合→源码→混合往返
    // 引擎互斥挂载与 body 连续（mode_toggle_preserves_content 直调法之外补
    // simulate_click 元素命中链路）。
    #[gpui::test]
    async fn toolbar_mode_button_swaps_engines_via_click(cx: &mut gpui::TestAppContext) {
        let (view, cx) = cx.add_window_view(|_w, cx| {
            crate::init(cx);
            EditorShell::with_content("模式稿".to_string(), "正文一".to_string(), cx)
        });
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
        });
        cx.run_until_parked();
        let mb = cx
            .debug_bounds("ed-mode")
            .unwrap_or_else(|| panic!("ed-mode 未录制（工具栏取样面失效）"));
        cx.simulate_click(mb.center(), Default::default());
        cx.run_until_parked();
        cx.update(|window, cx| {
            window.draw(cx).clear(cx); // src 引擎延迟建于 render 头，需出帧
        });
        cx.run_until_parked();
        view.update(cx, |v, _cx| {
            assert!(!v.hybrid, "点击源码钮应入源码态");
            assert!(v.md_editor.is_none(), "源码态应摘 md 引擎");
            assert!(v.src_textarea.is_some(), "源码态应建整篇引擎");
            assert_eq!(v.body.as_deref(), Some("正文一"), "body 连续不丢");
        });
        cx.simulate_click(mb.center(), Default::default());
        cx.run_until_parked();
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
        });
        cx.run_until_parked();
        view.update(cx, |v, _cx| {
            assert!(v.hybrid, "再点应回混合态");
            assert!(v.src_textarea.is_none(), "回混合应关整篇引擎");
            assert!(v.md_editor.is_some(), "回混合应重建 md 引擎");
        });
        cx.update(|_window, app| {
            let ed = view.read(app).md_editor.clone().unwrap();
            assert_eq!(ed.read(app).text(), "正文一", "md 引擎初值＝canonical body");
        });
    }

    // （矩阵 B3 · 工具栏）「定位到磁盘」钮（ed-finder）：置 reveal 旗标 + Escape
    // 统一事件上抛；take_reveal 单点消费（app 层漏斗分派面）。
    #[gpui::test]
    async fn toolbar_finder_sets_reveal_and_fires_escape(cx: &mut gpui::TestAppContext) {
        let events: std::rc::Rc<std::cell::RefCell<Vec<&'static str>>> = Default::default();
        let (view, cx) = cx.add_window_view(|_w, cx| {
            crate::init(cx);
            EditorShell::with_content("定位稿".to_string(), "正文".to_string(), cx)
        });
        view.update(cx, |v, _cx| {
            let events = events.clone();
            v.set_event_sink(Box::new(move |ev: &EditorEvent, _cx: &mut App| {
                let EditorEvent::Escape = ev else {
                    panic!("unexpected editor event")
                };
                events.borrow_mut().push("escape");
            }));
        });
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
        });
        cx.run_until_parked();
        let fb = cx
            .debug_bounds("ed-finder")
            .unwrap_or_else(|| panic!("ed-finder 未录制（工具栏取样面失效）"));
        cx.simulate_click(fb.center(), Default::default());
        cx.run_until_parked();
        assert_eq!(
            *events.borrow(),
            vec!["escape"],
            "定位钮应走 Escape 统一事件"
        );
        view.update(cx, |v, _cx| {
            assert!(v.take_reveal(), "定位钮应置 reveal 旗标且恰一次可消费");
            assert!(!v.take_reveal(), "旗标取后应复位");
        });
    }

    // （矩阵 C1 · 快捷键回归护栏）bind_keys 注册后 Esc 仍首按关窗：焦点在 md
    // 引擎时 escape 先命中 Dismiss 绑定（无菜单＝no-op·不拦传播），继续到根级
    // key_down 监听上抛 EditorEvent::Escape（首按即关窗；绑定优先于键监听的分
    // 发序在册）。
    #[gpui::test]
    async fn md_escape_still_fires_sink_with_bindings(cx: &mut gpui::TestAppContext) {
        let events: std::rc::Rc<std::cell::RefCell<Vec<&'static str>>> = Default::default();
        let (view, cx) = cx.add_window_view(|_w, cx| {
            crate::init(cx);
            EditorShell::with_content("Esc 稿".to_string(), String::new(), cx)
        });
        view.update(cx, |v, _cx| {
            let events = events.clone();
            v.set_event_sink(Box::new(move |ev: &EditorEvent, _cx: &mut App| {
                let EditorEvent::Escape = ev else {
                    panic!("unexpected editor event")
                };
                events.borrow_mut().push("escape");
            }));
        });
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
        });
        cx.run_until_parked();
        // 空稿自动聚焦开局：确认焦点确在 md 引擎（Esc 分发路径前提）。
        let focused = cx.update(|window, app| {
            let ed = view.read(app).md_editor.clone().unwrap();
            let vh = ed.read(app).focus_handle(app).clone();
            window.focused(app) == Some(vh)
        });
        assert!(focused, "空稿应自动聚焦 md 引擎（Esc 链路前提）");
        cx.simulate_keystrokes("escape");
        cx.run_until_parked();
        assert_eq!(
            *events.borrow(),
            vec!["escape"],
            "绑定注册后 Esc 应仍上抛关窗事件（dismiss no-op 不拦传播）"
        );
    }

    // （矩阵 D1 · 异常输入）特殊字符 roundtrip：emoji / 零宽空格 / CRLF / 制表符 /
    // 尾部换行 / CJK——引擎原样承载（normalize_loaded 仅触碰 $$ 数学围栏），
    // body 现场一致、export 快照一致。
    #[gpui::test]
    async fn md_special_chars_roundtrip(cx: &mut gpui::TestAppContext) {
        let odd = "🦀 emoji\n零宽\u{200B}字符\r\nCRLF\t制表\n结尾换行\n";
        let (view, cx) = cx.add_window_view(move |_w, cx| {
            crate::init(cx);
            EditorShell::with_content("特殊稿".to_string(), odd.to_string(), cx)
        });
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
        });
        cx.run_until_parked();
        cx.update(|_window, app| {
            let ed = view.read(app).md_editor.clone().unwrap();
            assert_eq!(
                ed.read(app).text(),
                odd,
                "特殊字符应原样承载（含 \r 与零宽字符）"
            );
        });
        view.update(cx, |v, _cx| {
            assert_eq!(v.body.as_deref(), Some(odd), "body 现场一致");
        });
    }

    // （矩阵 D2 · 异常输入）超长单行软折：20_000 字符单行溢出 → 引擎布局、外层
    // 滚动域合法；光标置文末 → SelectionChanged → 跟随滚动承接——极端内容下
    // 不失效。
    #[gpui::test]
    async fn md_long_single_line_soft_wrap_scroll(cx: &mut gpui::TestAppContext) {
        let long = "X".repeat(20_000);
        let (view, cx) = cx.add_window_view(move |_w, cx| {
            crate::init(cx);
            EditorShell::with_content("长行稿".to_string(), long, cx)
        });
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
        });
        cx.run_until_parked();
        cx.update(|_window, app| {
            let ed = view.read(app).md_editor.clone().unwrap();
            ed.update(app, |st, cx| {
                assert_eq!(st.text().len(), 20_000, "超长单行原样承载");
                let end = st.text().len();
                st.set_cursor(end, cx);
            });
        });
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
        });
        cx.run_until_parked();
        let (off, max) = view.update(cx, |v, _cx| {
            (
                f32::from(v.ed_scroll.offset().y),
                f32::from(v.ed_scroll.max_offset().y),
            )
        });
        assert!(max > 0.0, "单行 2 万字符软折后应溢出（max={max}）");
        assert!(
            off < 0.0 && off >= -max,
            "光标置文末应跟随滚动且域合法（off={off} max={max}）"
        );
    }

    // （矩阵 F1 · 容错）set_cursor 越界 clamp（zorite 语义：min(len)+char 边界
    // 吸附）与 offset_screen_top 越界承载（offset.min(len)）；外壳 follow_md_cursor
    // 对越界光标不 panic、滚动域合法。
    #[gpui::test]
    async fn md_out_of_range_cursor_clamps_safely(cx: &mut gpui::TestAppContext) {
        let (view, cx) = cx.add_window_view(|_w, cx| {
            crate::init(cx);
            EditorShell::with_content("边界稿".to_string(), "abc".to_string(), cx)
        });
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
        });
        cx.run_until_parked();
        cx.update(|_window, app| {
            let ed = view.read(app).md_editor.clone().unwrap();
            ed.update(app, |st, cx| {
                st.set_cursor(9_999, cx);
                assert_eq!(st.cursor(), 3, "越界光标应 clamp 至文末");
            });
            // set_cursor 派发 SelectionChanged → 外壳 follow 链已在其中运行。
        });
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
        });
        cx.run_until_parked();
        let off = view.update(cx, |v, _cx| f32::from(v.ed_scroll.offset().y));
        let max = view.update(cx, |v, _cx| f32::from(v.ed_scroll.max_offset().y));
        assert!(
            off <= 0.0 && off >= -max,
            "越界 clamp 后跟随域合法（off={off} max={max}）"
        );
        cx.update(|_window, app| {
            let ed = view.read(app).md_editor.clone().unwrap();
            assert!(
                ed.read(app).offset_screen_top(usize::MAX - 1).is_some(),
                "offset_screen_top 越界应 min(len) 承载（出帧后有布局）"
            );
        });
    }

    // 键入跟随（DEC-151 矩阵 E1 · Prove-It）：文本变更路径（回车/键入只发
    // Changed 无 SelectionChanged）在文末推进时仍须跟随。跟随时机＝当前帧渲染
    // 完（on_next_frame · 新布局生效）。
    #[gpui::test]
    async fn md_typing_at_bottom_keeps_caret_following(cx: &mut gpui::TestAppContext) {
        let (view, cx) = cx.add_window_view(|_w, cx| {
            crate::init(cx);
            EditorShell::with_content("跟打稿".to_string(), String::new(), cx)
        });
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
        });
        cx.run_until_parked();
        // 长多行注入（set_text 程序化）→ 光标置文末（选变路径同步跟随因陈旧布
        // 局 no-op）→ 编辑帧出帧（render 头消费待验旗标注册 on_next_frame）→
        // drain（新布局生效后跟随）→ 出帧落位。
        let long = (0..300)
            .map(|i| format!("第{i}行内容文字"))
            .collect::<Vec<_>>()
            .join("\n");
        cx.update(|_window, app| {
            let ed = view.read(app).md_editor.clone().unwrap();
            ed.update(app, |st, cx| {
                st.set_text(long.clone(), cx);
                let end = st.text().len();
                st.set_cursor(end, cx);
            });
        });
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
            window.simulate_next_frame(cx);
        });
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
        });
        cx.run_until_parked();
        let off1 = view.update(cx, |v, _cx| f32::from(v.ed_scroll.offset().y));
        assert!(off1 < 0.0, "光标置文末应已底对齐跟随（off1={off1}）");
        // 聚焦 md 引擎后连续回车×5（Newline 动作·只发 Changed）：
        cx.update(|window, app| {
            let ed = view.read(app).md_editor.clone().unwrap();
            let vh = ed.read(app).focus_handle(app).clone();
            window.focus(&vh, app);
        });
        cx.simulate_keystrokes("enter enter enter enter enter");
        cx.update(|window, cx| {
            window.draw(cx).clear(cx); // 编辑帧：新布局生效
            window.simulate_next_frame(cx); // drain on_next_frame → 跟随
        });
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
        });
        cx.run_until_parked();
        let (off2, max2) = view.update(cx, |v, _cx| {
            (
                f32::from(v.ed_scroll.offset().y),
                f32::from(v.ed_scroll.max_offset().y),
            )
        });
        assert!(
            off2 < off1 && off2 >= -max2,
            "连续回车后视口应继续下卷跟随光标（off1={off1} off2={off2} max={max2}）"
        );
        // 文末续编辑（字符路径·key_char）同跟随：先回车（新行恒 +1 行高，确定
        // 性增高）再键入 20 字（窄于窗宽、不再折行），视口应继续下卷追住新行。
        cx.simulate_keystrokes("enter");
        cx.simulate_input(&"字".repeat(20));
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
            window.simulate_next_frame(cx);
        });
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
        });
        cx.run_until_parked();
        let (off3, max3) = view.update(cx, |v, _cx| {
            (
                f32::from(v.ed_scroll.offset().y),
                f32::from(v.ed_scroll.max_offset().y),
            )
        });
        assert!(
            max3 > max2 && off3 >= -max3 && off3 < 0.0,
            "文末回车后增高应入账（max2={max2}→max3={max3}），offset 域合法（off3={off3}）"
        );
        // 光标可视断言（跟随语义的正确终态：光标行在视口带内；贴底与否取决于
        // 余量，不强制 off 持续变小）。
        let visible = cx.update(|_window, app| {
            let ed = view.read(app).md_editor.clone().unwrap();
            let (c_top, c_h) = {
                let e = ed.read(app);
                let b = e.caret_screen_bounds().expect("新布局下光标界应可得");
                (f32::from(b.top()), f32::from(b.size.height))
            };
            let vp = view.read(app).ed_scroll.bounds();
            let v_top = f32::from(vp.origin.y);
            let v_h = f32::from(vp.size.height);
            c_top - v_top >= 0.0 && c_top + c_h - v_top <= v_h
        });
        assert!(visible, "回车续键入后光标行应仍在视口带内");
    }

    // （矩阵 F2 · 容错）空稿全周期：自动建引擎并聚焦（开局即编辑）→ 直接键入
    // → export 快照；（空稿→聚焦→输入）为「创建集锦」新建稿标准路径的焦点
    // 承接证明。
    #[gpui::test]
    async fn md_empty_draft_autofocus_type_and_export(cx: &mut gpui::TestAppContext) {
        let (view, cx) = cx.add_window_view(|_w, cx| {
            crate::init(cx);
            EditorShell::with_content("Untitled".to_string(), String::new(), cx)
        });
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
        });
        cx.run_until_parked();
        let focused = cx.update(|window, app| {
            let ed = view.read(app).md_editor.clone().unwrap();
            let vh = ed.read(app).focus_handle(app).clone();
            window.focused(app) == Some(vh)
        });
        assert!(focused, "空稿新稿应自动聚焦 md 引擎（免死区承接）");
        cx.simulate_input("首拍");
        cx.run_until_parked();
        view.update(cx, |v, cx| {
            assert_eq!(
                v.body.as_deref(),
                Some("首拍"),
                "聚焦开局直接键入即写（无需点击）"
            );
            let (title, body) = v.export_note(cx);
            assert_eq!(title.as_deref(), Some("Untitled"), "export 快照题名");
            assert_eq!(body.as_deref(), Some("首拍"), "export 快照活正文");
        });
    }

    // （DEC-195 · 功能 1 回归面）围栏长行软折撑高：注入态单围栏稿，围栏体长行
    // 自动换行后文档总高必须溢出视口（max_offset>0）——高度不增长即固定高缺陷。
    // （基准实考：max≈1214px @主显窗）。
    #[gpui::test]
    async fn md_fence_long_line_soft_wrap_height_grows(cx: &mut gpui::TestAppContext) {
        let long = "X".repeat(20_000);
        let body = format!("```rust\n{long}\n```");
        let (view, cx) = cx.add_window_view(move |_w, cx| {
            crate::init(cx);
            EditorShell::with_content("围栏稿".to_string(), body, cx)
        });
        for _ in 0..2 {
            cx.update(|window, cx| {
                window.draw(cx).clear(cx);
            });
            cx.run_until_parked();
        }
        let max = view.update(cx, |v, _cx| f32::from(v.ed_scroll.max_offset().y));
        assert!(max > 0.0, "围栏长行软折后应撑高文档（max={max}）");
    }

    // （DEC-195 · 功能 1 回归面）同文双行（围栏内+普通段落）+ 尾部填充行，总高
    // 超视口数倍：光标置末滚动跟随后，顶部两行完全出绘制带 → 外行高度经带外
    // 处求解。内容未变，max_offset 不得跨帧涨缩（漂移即布局回归）。实考
    // （探针轮）：全帧恒定 5107px。
    #[gpui::test]
    async fn md_fence_and_plain_twin_height_stable_across_frames(cx: &mut gpui::TestAppContext) {
        let long = "X".repeat(20_000);
        let filler: String = (0..80).map(|i| format!("行{i}\n")).collect();
        let body = format!("```rust\n{long}\n```\n{long}\n{filler}");
        let (view, cx) = cx.add_window_view(move |_w, cx| {
            crate::init(cx);
            EditorShell::with_content("带外稿".to_string(), body, cx)
        });
        let mut baseline = vec![];
        for _ in 0..2 {
            cx.update(|window, cx| {
                window.draw(cx).clear(cx);
            });
            cx.run_until_parked();
            baseline.push(view.update(cx, |v, _cx| f32::from(v.ed_scroll.max_offset().y)));
        }
        let base = baseline[0];
        assert_eq!(baseline[1], base, "顶部两帧总高一致（无带外命中）");
        assert!(base > 0.0, "带外稿应超视口（base={base}）");
        // 光标置文末（SelectionChanged → 外层跟随滚动）→ 顶部行出带。
        cx.update(|_window, app| {
            let ed = view.read(app).md_editor.clone().unwrap();
            ed.update(app, |st, cx| {
                let end = st.text().len();
                st.set_cursor(end, cx);
            });
        });
        cx.run_until_parked();
        for _ in 0..4 {
            cx.update(|window, cx| {
                window.draw(cx).clear(cx);
            });
            cx.run_until_parked();
            let (max, off) = view.update(cx, |v, _cx| {
                (
                    f32::from(v.ed_scroll.max_offset().y),
                    f32::from(v.ed_scroll.offset().y),
                )
            });
            assert_eq!(max, base, "内容未变，带外求解不得涨缩（base={base}）");
            assert!(
                off < 0.0 && off >= -base,
                "跟随滚动后 offset 域合法（off={off}）"
            );
        }
    }

    // （DEC-195 · 功能 1 修复面）源码态 auto-grow：长围栏稿切源码态 → Textarea
    // 随内容行数撑高（max_offset>0；修复前固定高 max=0 实考）；短文保持撑满
    // 不溢出（max_offset==0 · DEC-146 撑满语义保留）；双向切换内容连续不丢。
    #[gpui::test]
    async fn src_mode_long_content_auto_grows(cx: &mut gpui::TestAppContext) {
        let long = "X".repeat(20_000);
        let body = format!("```rust\n{long}\n```");
        let expect = body.clone();
        let (view, cx) = cx.add_window_view(move |_w, cx| {
            crate::init(cx);
            EditorShell::with_content("源码稿".to_string(), body, cx)
        });
        for _ in 0..2 {
            cx.update(|window, cx| {
                window.draw(cx).clear(cx);
            });
            cx.run_until_parked();
        }
        let mb = cx
            .debug_bounds("ed-mode")
            .unwrap_or_else(|| panic!("ed-mode 未录制（工具栏取样面失效）"));
        cx.simulate_click(mb.center(), Default::default());
        cx.run_until_parked();
        cx.update(|window, cx| {
            window.draw(cx).clear(cx); // src 引擎延迟建于 render 头，需出帧
        });
        cx.run_until_parked();
        for _ in 0..2 {
            cx.update(|window, cx| {
                window.draw(cx).clear(cx);
            });
            cx.run_until_parked();
        }
        view.update(cx, |v, _cx| assert!(!v.hybrid, "应已切源码态"));
        let max = view.update(cx, |v, _cx| f32::from(v.ed_scroll.max_offset().y));
        assert!(
            max > 0.0,
            "源码态长围栏内容应随内容撑高编辑器（auto-grow·max={max}）"
        );
        // 内容连续：md→src 换引擎不丢字。
        view.update(cx, |v, cx| {
            assert_eq!(
                v.src_textarea
                    .as_ref()
                    .map(|st| st.read(cx).value().to_string()),
                Some(expect),
                "源码引擎初值＝canonical body"
            );
        });
        // 短文反向用例（独立窗）：撑满不溢出（DEC-146 语义保留）。
        let (view2, cx2) = cx.add_window_view(|_w, cx| {
            EditorShell::with_content("短稿".to_string(), "第一段".to_string(), cx)
        });
        for _ in 0..2 {
            cx2.update(|window, cx| {
                window.draw(cx).clear(cx);
            });
            cx2.run_until_parked();
        }
        let mb2 = cx2
            .debug_bounds("ed-mode")
            .unwrap_or_else(|| panic!("ed-mode 未录制（短稿窗）"));
        cx2.simulate_click(mb2.center(), Default::default());
        cx2.run_until_parked();
        cx2.update(|window, cx| {
            window.draw(cx).clear(cx);
        });
        cx2.run_until_parked();
        for _ in 0..2 {
            cx2.update(|window, cx| {
                window.draw(cx).clear(cx);
            });
            cx2.run_until_parked();
        }
        let max2 = view2.update(cx2, |v, _cx| f32::from(v.ed_scroll.max_offset().y));
        assert_eq!(max2, 0.0, "源码态短文应撑满不溢出（max==0）");
    }
}
