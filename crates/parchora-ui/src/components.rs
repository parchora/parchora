//! 共享组件原语（UI-1）——`parchora-shared.css` 逐度量 1:1 落 GPUI。
//!
//! 每个函数对应 shared.css 一条规则块，注释给出 CSS 原值供走查对照。
//! 组件一律无状态、无回调：返回样式化 `Div`，事件由页面在返回值上追加
//! `.id(...).on_click(...)` 接线。
//!
//! 平台约束定形（偏差台账，交付报告汇总）：
//! - D-01 字距：gpui-fast 0.1.8 `TextStyle` 无 letter_spacing，CSS 字距（0.02–0.3em）不渲染；
//! - D-02 字体：Songti SC（UI 控件）/ Kaiti SC（内容区，`content_text`/`t_label`/`strike`
//!   承载）/ Menlo（源码）——DEC-66/68 定字体栈（覆盖 DESIGN.md 字型栈默认，捆绑
//!   标准变体族名）；字体未安装时 CoreText 落系统默认；
//! - D-03 划线：CSS line-through → 标题上覆 1px 线（rgba 主墨 48%）由本模块 `strike()` 承载；
//! - D-05 阴影：纸感环境环（多层 box-shadow）defer，不影响任何布局度量；
//! - 图标（DEC-66）：全部 lucide 线性图标（`crate::icons`，1.59.0 原文内联），
//!   本模块在此 re-export（`co::icon`/`co::Icon`），仅替换设计稿原 glyph 位。

use gpui::{
    anchored, deferred, div, px, relative, rgb, Div, FontWeight, Hsla, InteractiveElement as _,
    IntoElement, ParentElement, StyleRefinement, Styled,
};

/// hex u32 → Hsla（ Into<Hsla> 类型歧义场景的显式转换）。
pub fn h(hex: u32) -> Hsla {
    Hsla::from(rgb(hex))
}

// ──────────── 统一动效口径（shared.css :hover 全量 15 规则 × 按下为衍生口径） ────────────

/// 交互悬停/按下的统一样式闭包（single source of truth——页面不得散写内联 hover 值）。
/// 悬停依据：parchora-shared.css :hover 全量 15 规则（L24/46/52/61/84/86/88/93/96/
/// 101/130/153/159/164/171）；按下（:active）HTML 原型未定义——衍生口径 =
/// 「悬停底再加深一档」（DEC-111 登记），全部取自 DESIGN.md 17 色板、零新增色值。
/// 形态遵守 DESIGN.md 动效铁限：即时态切换，无弹跳无缩放（淡入族另经
/// `with_animation` + MOTION_FAST_MS 承载）；按下仅接已接线元素（on_click 存在者）。
/// 悬停·暖底（a.row / a.node / .ibtn / .fab / .tagc / .chip:hover → warm）。
pub fn hov_warm(s: StyleRefinement) -> StyleRefinement {
    s.bg(rgb(t::WARM))
}

/// 悬停·深一档（warm 基底件的衍生悬停 → border：btn_sec / chip_on）。
pub fn hov_border(s: StyleRefinement) -> StyleRefinement {
    s.bg(rgb(t::BORDER))
}

/// 悬停·主墨钮（.btn-pri:hover → press，shared.css L61 原值）。
pub fn hov_press(s: StyleRefinement) -> StyleRefinement {
    s.bg(rgb(t::PRESS))
}

/// 悬停·字色（a.tab / .crumb a:hover → primary；字色悬停需文本样式落在悬停元素自身）。
pub fn hov_ink(s: StyleRefinement) -> StyleRefinement {
    s.text_color(rgb(t::PRIMARY))
}

/// 悬停·夜读批量条链接（.batchbar a:hover → night-ink，shared.css L171 原值）。
pub fn hov_night_ink(s: StyleRefinement) -> StyleRefinement {
    s.text_color(rgb(t::NIGHT_INK))
}

/// 按下·一档加深（衍生：warm 悬停件按下 → border）。
pub fn act_border(s: StyleRefinement) -> StyleRefinement {
    s.bg(rgb(t::BORDER))
}

/// 按下·主墨钮（衍生：btn-pri 按下 press → ink-deep）。
pub fn act_ink_deep(s: StyleRefinement) -> StyleRefinement {
    s.bg(rgb(t::INK_DEEP))
}

/// 按下·暖底系（衍生：border 悬停件按下 → border 底 + primary 字提亮补足）。
pub fn act_border_ink(s: StyleRefinement) -> StyleRefinement {
    s.bg(rgb(t::BORDER)).text_color(rgb(t::PRIMARY))
}

pub use crate::icons::{icon, icon_inherited, Icon};
use crate::scale as s;
use crate::tokens as t;

// ───────────────────────── 文本 ─────────────────────────

/// 正文文本（UI 控件族；行高 1.95 = CSS body 继承值）。min_w_0（DEC-229）：
/// 作为 flex 行直接子项时解除 min-content 地板，约束宽下自动换行。
pub fn txt(label: impl Into<String>, size: f32, color: u32) -> Div {
    div()
        .min_w_0()
        .font_family(t::FONT_BODY)
        .font_weight(FontWeight(t::FONT_WEIGHT_LIGHT))
        .text_size(px(size))
        .line_height(relative(t::LH_BODY))
        .text_color(rgb(color))
        .child(label.into())
}

/// 内容文本：笔记/剪贴板/待办等具体内容展示区（内容族 Kaiti SC · DEC-66→DEC-68）。
/// min_w_0 同 txt（DEC-229）。
pub fn content_text(label: impl Into<String>, size: f32, color: u32) -> Div {
    div()
        .min_w_0()
        .font_family(t::FONT_CONTENT)
        .font_weight(FontWeight(t::FONT_WEIGHT_LIGHT))
        .text_size(px(size))
        .line_height(relative(t::LH_BODY))
        .text_color(rgb(color))
        .child(label.into())
}

/// 文本（全控：字重/行高覆盖）。
pub fn txt_f(
    label: impl Into<String>,
    size: f32,
    color: u32,
    font: &'static str,
    weight: f32,
) -> Div {
    div()
        .min_w_0()
        .font_family(font)
        .font_weight(FontWeight(weight))
        .text_size(px(size))
        .text_color(rgb(color))
        .child(label.into())
}

/// `.t` 行内标题：flex:1 · 13px · ls.02em（CSS .row .t；行标题=内容数据 → Kaiti SC · DEC-68）。
/// min_w_0（DEC-229）：flex_1 项在长标题下可收缩，正文在行内衬内自动换行
///（flex 主轴 min-content 地板＝整串不换行全宽，不归零则长串撑破行右缘）。
pub fn t_label(label: impl Into<String>) -> Div {
    div()
        .flex_1()
        .min_w_0()
        .font_family(t::FONT_CONTENT)
        .font_weight(FontWeight(t::FONT_WEIGHT_LIGHT))
        .text_size(s::px_of(s::Lvl::Tab))
        .line_height(relative(t::LH_BODY))
        .text_color(rgb(t::PRIMARY))
        .child(label.into())
}

/// 详情面板标题栏标题显示截断（DEC-235 用户指令 · DEC-237 阈值修订 12→20）：
/// 超过 20 字符 → 前 20 字符＋「......」字面（非单字符省略号）；字符按 char
/// 计（CJK 与拉丁同权）；≤20 原文不动。实装接线：editor 编辑/详情窗
/// 「ed-title」显示态与 peek 条幅标签（行内改名输入态承载全名不受此限）。
pub fn detail_title_truncate(s: &str) -> String {
    const DETAIL_TITLE_MAX_CHARS: usize = 20;
    const DETAIL_TITLE_AFFIX: &str = "......";
    match s.char_indices().nth(DETAIL_TITLE_MAX_CHARS).map(|(i, _)| i) {
        Some(idx) => {
            let mut head = s[..idx].to_string();
            head.push_str(DETAIL_TITLE_AFFIX);
            head
        }
        None => s.to_string(),
    }
}

/// `.meta` 次级文字：11px · muted · ls.06em。min_w_0 同 txt（DEC-229）。
pub fn meta(label: impl Into<String>) -> Div {
    div()
        .min_w_0()
        .font_family(t::FONT_BODY)
        .font_weight(FontWeight(t::FONT_WEIGHT_LIGHT))
        .text_size(px(s::font(s::Lvl::Meta)))
        .line_height(relative(t::LH_BODY))
        .text_color(rgb(t::MUTED))
        .child(label.into())
}

/// `.strike` 划线标题：48% 主墨上覆线（CSS line-through 的 D-03 模拟；内容数据 → Kaiti SC · DEC-68）。
/// min_w_0 同 t_label（DEC-229）。
pub fn strike(label: impl Into<String>) -> Div {
    div()
        .relative()
        .flex_1()
        .min_w_0()
        .font_family(t::FONT_CONTENT)
        .font_weight(FontWeight(t::FONT_WEIGHT_LIGHT))
        .text_size(s::px_of(s::Lvl::Tab))
        .line_height(relative(t::LH_BODY))
        .text_color(rgb(t::PRIMARY))
        .child(label.into())
        .child(
            div()
                .absolute()
                .left_0()
                .right_0()
                .h(px(1.0))
                .top(relative(0.48))
                .bg(gpui::rgba(t::PRIMARY << 8 | 0x7A)), // 主墨 48%（D-03）
        )
}

// ───────────────────────── 外壳与导航 ─────────────────────────

/// `.bar`：顶栏容器 · p 12/16 · bsoft 底线（子元素由调用方装配）。
pub fn bar() -> Div {
    div()
        .flex()
        .items_center()
        .justify_between()
        .px(px(16.0))
        .py(px(12.0))
        .border_b_1()
        .border_color(rgb(t::BORDER_SOFT))
}

/// `.tabs` 容器：gap 20（shared.css）/ 主面板 14（v3 覆盖）· items-baseline。
pub fn tabs(gap: f32) -> Div {
    div().flex().items_center().gap(px(gap))
}

/// `.tab`：13px · 激活=加粗(500) + 字号 +2（15px）+ primary + 2px 下划线（::after）；
/// 未激活=muted（激活态字重阶梯上限 500，DESIGN.md 禁 600+）。
pub fn tab(label: impl Into<String>, on: bool) -> Div {
    let mut el = div()
        .font_family(t::FONT_BODY)
        .font_weight(FontWeight(t::FONT_WEIGHT_LIGHT))
        .text_size(px(s::font(s::Lvl::Tab)))
        .line_height(relative(1.2))
        .cursor_pointer()
        .hover(hov_ink);
    if on {
        // 选中=加粗(500) + 字号 +2 + 字色加深 + 2px 主墨下划线（tab-active-underline）
        el = el
            .font_weight(FontWeight(t::FONT_WEIGHT_MEDIUM))
            .text_size(px(s::font(s::Lvl::TabActive)))
            .text_color(rgb(t::PRIMARY))
            .child(label.into())
            .child(div().h(px(2.0)).w_full().mt(px(4.0)).bg(rgb(t::PRIMARY)));
    } else {
        el = el
            .text_color(rgb(t::MUTED))
            .child(label.into())
            .child(div().h(px(2.0)).w_full().mt(px(4.0)));
    }
    el
}

/// `.ibtn` 图标钮：border(border) · r6 · surface 底 · p 3/10 · 12px。
pub fn ibtn(label: impl Into<String>) -> Div {
    div()
        .px(px(10.0))
        .py(px(3.0))
        .border_1()
        .border_color(rgb(t::BORDER))
        .rounded(px(t::RAD_SM))
        .bg(rgb(t::SURFACE))
        .font_family(t::FONT_BODY)
        .font_weight(FontWeight(t::FONT_WEIGHT_LIGHT))
        .text_size(px(s::font(s::Lvl::Small)))
        .text_color(rgb(t::PRIMARY))
        .cursor_pointer()
        .hover(hov_warm)
        .child(label.into())
}

/// `.ibtn` 图标钮·纯图标变体：同盒度量，居中一枚 lucide 图标（DEC-66）。
pub fn icon_btn(i: Icon) -> Div {
    div()
        .flex()
        .items_center()
        .justify_center()
        .px(px(10.0))
        .py(px(3.0))
        .border_1()
        .border_color(rgb(t::BORDER))
        .rounded(px(t::RAD_SM))
        .bg(rgb(t::SURFACE))
        .cursor_pointer()
        .hover(hov_warm)
        .child(icon(i, 13.0, t::PRIMARY))
}

/// `.ibtn` 图标钮·图标+文字变体（前置 lucide 图标 + 12px 文字；颜色随态可变）。
/// 基线底色/悬停走 ibtn 三元组（纸面底 + warm 悬停），委托参数化变体保单一实现。
pub fn ibtn_icon_text(i: Icon, label: impl Into<String>, color: u32) -> Div {
    ibtn_icon_text_bg(i, label, color, t::SURFACE, t::WARM)
}

/// `.ibtn` 图标钮·图标+文字·背景参数化变体（DEC-124）：盒度量与前变体
/// 完全一致，仅背景底（bg）与悬停底（hover_bg）随选中态参数化。hover 由本组件
/// **单次注册**——vendor div.rs hover_style 为单槽（debug_assert「hover style
/// already set」），调用点禁止对本函数返回值再挂 .hover()。
pub fn ibtn_icon_text_bg(
    i: Icon,
    label: impl Into<String>,
    color: u32,
    bg: u32,
    hover_bg: u32,
) -> Div {
    div()
        .flex()
        .items_center()
        .gap(px(5.0))
        .px(px(10.0))
        .py(px(3.0))
        .border_1()
        .border_color(rgb(t::BORDER))
        .rounded(px(t::RAD_SM))
        .bg(rgb(bg))
        .cursor_pointer()
        .hover(move |s: StyleRefinement| s.bg(rgb(hover_bg)))
        .child(icon(i, 12.0, color))
        .child(txt_f(
            label,
            s::font(s::Lvl::Small),
            color,
            t::FONT_BODY,
            t::FONT_WEIGHT_LIGHT,
        ))
}

// ───────────────────────── 行 ─────────────────────────

/// `.row` 行基座：flex · gap10 · p 11/16 · bsoft 底线（内容由调用方装配）。
pub fn row() -> Div {
    div()
        .flex()
        .items_center()
        .gap(px(10.0))
        .px(px(16.0))
        .py(px(11.0))
        .border_b_1()
        .border_color(rgb(t::BORDER_SOFT))
        // 组挂载（"row"）：供行内 tgo 的 group_hover 取最近包含组员（GroupHitboxes
        // DFS 栈顶语义 · vendor div.rs:4090）；仅样式归属，不变行本身视觉。
        .group("row")
}

/// 选中态三件套之一：`.row.sel` 暖底 + 左 2px 主墨条（inset box-shadow 等效）。
/// 在 row 返回值上叠加：`.bg(warm)` 后再布左条。
pub fn sel_state(div_: Div) -> Div {
    div_.bg(rgb(t::WARM)).child(
        div()
            .absolute()
            .left_0()
            .top_0()
            .bottom_0()
            .w(px(2.0))
            .bg(rgb(t::PRIMARY)),
    )
}

/// `.dot`：9px 圆 · done=点缀金实心（2026-10-23 四色重构 · growth 绿退役于 UI 完成态）。
pub fn dot(done: bool) -> Div {
    if done {
        div()
            .w(px(9.0))
            .h(px(9.0))
            .rounded_full()
            .flex_none()
            .bg(rgb(t::ACCENT))
    } else {
        div()
            .w(px(9.0))
            .h(px(9.0))
            .rounded_full()
            .flex_none()
            .border_1()
            .border_color(rgb(t::BORDER))
    }
}

/// `.tgo` ↗ 悬停预览指示 → lucide arrow-up-right（DEC-66）。
/// shared.css L51-52/153 原口径：行内基态 faint，opacity 0（只画不说话）；行悬停
/// 浮现 opacity 1、字色转 primary。经 row()/node_row() 基座 `.group("row")` 与本元素
/// `group_hover` 级联承载（GroupHitboxes DFS 栈顶取最近组员）；图标色走继承通道
/// `icon_inherited`，悬停 refinement 改 text_color 即级联到 svg（svg 涂装取
/// style.text.color）。
pub fn tgo() -> Div {
    div()
        .flex_none()
        .text_color(rgb(t::FAINT))
        .opacity(0.0)
        .group_hover("row", |s| s.text_color(rgb(t::PRIMARY)).opacity(1.0))
        .child(icon_inherited(Icon::ArrowUpRight, 11.0))
}

/// `.mark-sel` 多选勾选符 → lucide circle-dot（已选 tertiary）/ circle（未选 muted）。
pub fn mark_sel(selected: bool) -> Div {
    if selected {
        div()
            .flex_none()
            .child(icon(Icon::CircleDot, 13.0, t::TERTIARY))
    } else {
        div().flex_none().child(icon(Icon::Circle, 13.0, t::MUTED))
    }
}

// ───────────────────────── 列表结构件 ─────────────────────────

/// `.empty` 空态：p 56/16 居中 + lucide 图标 30px faint + 一句话。
pub fn empty_state(i: Icon, line: impl Into<String>) -> Div {
    div()
        .flex()
        .flex_col()
        .items_center()
        .px(px(16.0))
        .py(px(56.0))
        .gap(px(10.0))
        .child(icon(i, 30.0, t::FAINT))
        .child(txt(line, s::font(s::Lvl::Tab), t::MUTED))
}

/// `.count` 计数行：p 7/16 · 12px muted · bsoft 底线。
pub fn count_line(label: impl Into<String>) -> Div {
    div()
        .px(px(16.0))
        .py(px(7.0))
        .border_b_1()
        .border_color(rgb(t::BORDER_SOFT))
        .child(txt(label, s::font(s::Lvl::Small), t::MUTED))
}

/// `.search` 搜索框（静态承载，D-06 输入引擎 defer）：m 8/16 · border · r6 · p 6/12；
/// lucide 图标前置（HTML `.search` 原文序对齐：「🔍 打开即搜…⌘F」）；图标由调用方按语义传入（搜索=search · 新增=plus）；
/// 调用方追加的子元素经内部 flex_1 垫片靠右（⌘F 等快捷键标注位）。
pub fn search_box(placeholder: impl Into<String>, i: Icon) -> Div {
    div()
        .flex()
        .items_center()
        .gap(px(8.0))
        .mx(px(16.0))
        .my(px(8.0))
        .px(px(12.0))
        .py(px(6.0))
        .border_1()
        .border_color(rgb(t::BORDER))
        .rounded(px(t::RAD_SM))
        .child(icon(i, 14.0, t::MUTED))
        .child(txt(placeholder, s::font(s::Lvl::Tab), t::MUTED))
        .child(div().flex_1())
}

/// `.chip` 筛选小标：11px · border · pill · p 1/10 · muted。
pub fn chip(label: impl Into<String>) -> Div {
    div()
        .px(px(10.0))
        .py(px(1.0))
        .border_1()
        .border_color(rgb(t::BORDER))
        .rounded_full()
        .font_family(t::FONT_BODY)
        .font_weight(FontWeight(t::FONT_WEIGHT_LIGHT))
        .text_size(s::px_of(s::Lvl::Meta))
        .line_height(relative(t::LH_BODY))
        .text_color(rgb(t::MUTED))
        .cursor_pointer()
        .hover(hov_warm)
        .child(label.into())
}

/// `.chip·dd` 下拉 chip 变体：label + lucide chevron-down（设计稿「类型 ▾」glyph 位 · DEC-66）。
pub fn chip_dd(label: impl Into<String>) -> Div {
    div()
        .flex()
        .items_center()
        .gap(px(4.0))
        .px(px(10.0))
        .py(px(1.0))
        .border_1()
        .border_color(rgb(t::BORDER))
        .rounded_full()
        .cursor_pointer()
        .hover(hov_warm)
        .child(txt(label, s::font(s::Lvl::Meta), t::MUTED))
        .child(icon(Icon::ChevronDown, 10.0, t::MUTED))
}

/// `.chip.on`：激活 chip = warm 底 + primary 字（PG-A 筛选态）。
pub fn chip_on(label: impl Into<String>) -> Div {
    div()
        .px(px(10.0))
        .py(px(1.0))
        .border_1()
        .border_color(rgb(t::BORDER))
        .rounded_full()
        .bg(rgb(t::WARM))
        .cursor_pointer()
        .hover(hov_border)
        .font_family(t::FONT_BODY)
        .font_weight(FontWeight(t::FONT_WEIGHT_LIGHT))
        .text_size(s::px_of(s::Lvl::Meta))
        .text_color(rgb(t::PRIMARY))
        .child(label.into())
}

/// `.grp` 分组头：p 8/16 · warm 底 · 12px muted（右元素可选装配）。
pub fn grp(label: impl Into<String>, right: Option<impl IntoElement>) -> Div {
    let mut el = div()
        .flex()
        .items_center()
        .justify_between()
        .px(px(16.0))
        .py(px(8.0))
        .bg(rgb(t::WARM))
        .child(txt(label, s::font(s::Lvl::Small), t::MUTED));
    if let Some(r) = right {
        el = el.child(r);
    }
    el
}

/// `.node` 树节点：p 7/16/7/22 · 13px · swatch 9px 圆。
pub fn node_row(label: impl Into<String>, swatch: Option<u32>, selected: bool) -> Div {
    let label = label.into();
    let mut el = div()
        .flex()
        .items_center()
        .gap(px(8.0))
        .pl(px(22.0))
        .pr(px(16.0))
        .py(px(7.0))
        .border_b_1()
        .border_color(rgb(t::BORDER_SOFT))
        // a.node:hover → warm（shared.css L86）+ 组挂载供行内 tgo（同 row()）。
        .hover(hov_warm)
        .group("row")
        .child(txt(label.clone(), s::font(s::Lvl::Tab), t::PRIMARY));
    if let Some(s) = swatch {
        el = div()
            .flex()
            .items_center()
            .gap(px(8.0))
            .pl(px(22.0))
            .pr(px(16.0))
            .py(px(7.0))
            .border_b_1()
            .border_color(rgb(t::BORDER_SOFT))
            .hover(hov_warm)
            .group("row")
            .child(
                div()
                    .w(px(9.0))
                    .h(px(9.0))
                    .rounded_full()
                    .flex_none()
                    .bg(rgb(s)),
            )
            .child(txt(label.clone(), s::font(s::Lvl::Tab), t::PRIMARY));
    }
    if selected {
        el = sel_state(el);
    }
    el
}

/// `.crumb` 面包屑：p 7/12 · 12px · sep=lucide chevron-right faint · cur=primary。
/// `parts`: (text, is_current)。
pub fn crumb(parts: Vec<(String, bool)>) -> Div {
    let mut el = div()
        .flex()
        .items_center()
        .gap(px(6.0))
        .px(px(12.0))
        .py(px(7.0))
        .border_b_1()
        .border_color(rgb(t::BORDER_SOFT));
    for (i, (text, cur)) in parts.into_iter().enumerate() {
        if i > 0 {
            el = el.child(
                div()
                    .flex_none()
                    .child(icon(Icon::ChevronRight, 10.0, t::FAINT)),
            );
        }
        if cur {
            el = el.child(txt(text, s::font(s::Lvl::Small), t::PRIMARY));
        } else {
            // .crumb a:hover → primary（shared.css L46）：字色悬停落在文本元素自身
            // （txt 自带 text_color，同元素 refinement 可合并级联）。
            el = el.child(
                txt(text, s::font(s::Lvl::Small), t::MUTED)
                    .cursor_pointer()
                    .hover(hov_ink),
            );
        }
    }
    el
}

// ───────────────────────── 按钮与浮钮 ─────────────────────────

/// `.btn-pri` 落笔钮：primary 实心 · surface 字 · 13px · p 11/22（每屏仅一枚）。
pub fn btn_pri(label: impl Into<String>) -> Div {
    div()
        .px(px(22.0))
        .py(px(11.0))
        .rounded(px(t::RAD_SM))
        .bg(rgb(t::PRIMARY))
        .font_family(t::FONT_BODY)
        .font_weight(FontWeight(t::FONT_WEIGHT_LIGHT))
        .text_size(px(s::font(s::Lvl::Tab)))
        .text_color(rgb(t::ON_PRIMARY))
        .cursor_pointer()
        .hover(hov_press)
        .child(label.into())
}

/// `.btn-sec` 次级钮：warm 底 · muted 字 · 12px · p 8/16。
pub fn btn_sec(label: impl Into<String>) -> Div {
    div()
        .px(px(16.0))
        .py(px(8.0))
        .rounded(px(t::RAD_SM))
        .bg(rgb(t::WARM))
        .font_family(t::FONT_BODY)
        .font_weight(FontWeight(t::FONT_WEIGHT_LIGHT))
        .text_size(px(s::font(s::Lvl::Small)))
        .text_color(rgb(t::MUTED))
        .cursor_pointer()
        .hover(hov_border)
        .child(label.into())
}

/// `.btn-sec` 次级钮·图标+文字变体（lucide plus + label；DEC-66）。
pub fn btn_sec_icon(i: Icon, label: impl Into<String>) -> Div {
    div()
        .flex()
        .items_center()
        .gap(px(6.0))
        .px(px(16.0))
        .py(px(8.0))
        .rounded(px(t::RAD_SM))
        .bg(rgb(t::WARM))
        .cursor_pointer()
        .hover(hov_border)
        .child(icon(i, 12.0, t::MUTED))
        .child(txt_f(
            label,
            s::font(s::Lvl::Small),
            t::MUTED,
            t::FONT_BODY,
            t::FONT_WEIGHT_LIGHT,
        ))
}

/// `.fab` 右下浮钮：54px 圆 · surface · 1px border · lucide plus 22px（fab-dock 右下 16px）；
/// 三层投影（CSS 等效
/// `box-shadow:0 0 1px rgba(0,0,0,.4),0 0 2px rgba(0,0,0,.3),0 0 6px rgba(0,0,0,.2)`）；
/// 悬浮遮挡（DEC-148）：阻断点击/hover 向身后列表穿透，滚轮仍透身后列表。
pub fn fab() -> Div {
    div()
        .w(px(s::sp(t::FAB)))
        .h(px(s::sp(t::FAB)))
        .rounded_full()
        .border_1()
        .border_color(rgb(t::BORDER))
        .bg(rgb(t::SURFACE))
        .flex()
        .items_center()
        .justify_center()
        .cursor_pointer()
        .hover(hov_warm)
        // FAB 悬浮于列表上方：gpui 命中测试默认把其下方 hitbox 一并计入 hover 栈，
        // 点击会传到身后列表行误触发 on_click——block_mouse_except_scroll 在 FAB 处
        // 截断 hover 栈（下方行不合成 click），滚轮经 should_handle_scroll 照常滚动
        // 身后列表（既有滚动行为不变）；「fab」selector 供点击穿透回归测试定位
        //（release no-op 不入生产样式 · DEC-137 同式）。
        .block_mouse_except_scroll()
        .debug_selector(|| "fab".to_string())
        .shadow(vec![
            gpui::BoxShadow::new(px(0.0), px(0.0), gpui::hsla(0.0, 0.0, 0.0, 0.4))
                .blur_radius(px(1.0)),
            gpui::BoxShadow::new(px(0.0), px(0.0), gpui::hsla(0.0, 0.0, 0.0, 0.3))
                .blur_radius(px(2.0)),
            gpui::BoxShadow::new(px(0.0), px(0.0), gpui::hsla(0.0, 0.0, 0.0, 0.2))
                .blur_radius(px(6.0)),
        ])
        .child(icon(Icon::Plus, 22.0, t::PRIMARY))
}

/// `.switch` 开关（拾记密度切换承载）：34×20 轨道
/// · r-full · 14px 旋钮（2px 内距）；开=warm 轨+主墨描边+主墨旋钮右位（宽松模式）
/// / 关=羊皮底轨+borders 细边+muted 旋钮左位（紧凑默认）。
pub fn switch(on: bool) -> Div {
    let (rail_bg, rail_border, knob) = if on {
        (t::WARM, t::PRIMARY, t::PRIMARY)
    } else {
        (t::NEUTRAL, t::BORDER, t::MUTED)
    };
    div()
        .flex_none()
        .w(px(34.0))
        .h(px(20.0))
        .rounded_full()
        .border_1()
        .border_color(rgb(rail_border))
        .bg(rgb(rail_bg))
        .cursor_pointer()
        .relative()
        .child(
            div()
                .absolute()
                .top(px(2.0))
                .left(if on { px(16.0) } else { px(2.0) })
                .w(px(14.0))
                .h(px(14.0))
                .rounded_full()
                .bg(rgb(knob)),
        )
}

// ───────────────────────── 浮层 ─────────────────────────

/// `.sheet` 落墨浮层内体：surface · 顶 16 圆角 · p 20/24（落墨为唯一主操作）。
pub fn sheet() -> Div {
    div()
        .bg(rgb(t::SURFACE))
        .rounded_t(px(t::RAD_LG))
        .px(px(24.0))
        .py(px(20.0))
}

/// `.sheeth` 浮层头：两侧对齐 · mb 12（子元素调用方装配）。
pub fn sheeth() -> Div {
    div().flex().items_center().justify_between().mb(px(12.0))
}

/// `.toast`：ink-deep 底 · surface 字 · r6 · p 9/20 · 12px · 0.93（1400ms 单发语义由页面承载）。
pub fn toast(label: impl Into<String>) -> Div {
    div()
        .px(px(20.0))
        .py(px(9.0))
        .rounded(px(t::RAD_SM))
        .bg(rgb(t::INK_DEEP))
        .opacity(0.93)
        .child(txt(label, s::font(s::Lvl::Small), t::SURFACE))
}

/// `.hint` 底部提示条：p 16 居中 · 12px muted · bsoft 顶线。
pub fn hint(label: impl Into<String>) -> Div {
    div()
        .p(px(16.0))
        .flex()
        .flex_col()
        .items_center()
        .border_t_1()
        .border_color(rgb(t::BORDER_SOFT))
        .child(txt(label, s::font(s::Lvl::Small), t::MUTED))
}

/// `.hint` 带墨痕变体 → lucide 图标 22px faint 置上（PG-A ✑=square-pen / 回收站 ◌=circle-dashed）。
pub fn hint_marked(i: Icon, line: impl Into<String>) -> Div {
    div()
        .p(px(16.0))
        .flex()
        .flex_col()
        .items_center()
        .gap(px(6.0))
        .border_t_1()
        .border_color(rgb(t::BORDER_SOFT))
        .child(icon(i, 22.0, t::FAINT))
        .child(txt(line, s::font(s::Lvl::Small), t::MUTED))
}

/// `.sec` 内容盒：surface · r8 · p 20/24（设置/演示说明用）。
pub fn sec_box() -> Div {
    div()
        .bg(rgb(t::SURFACE))
        .rounded(px(t::RAD_MD))
        .px(px(s::sp(t::SP_LG)))
        .py(px(20.0))
}

// ───────────────────────── 集锦 v4 组件 ─────────────────────────

/// `.tagc` 标签入口卡：surface · bsoft 边 · r6 · p 7/10 · t-dot 7px（DEC-117）。
pub fn tagc(label: impl Into<String>, color: u32, num: impl Into<String>) -> Div {
    div()
        .flex()
        .items_center()
        .gap(px(6.0))
        .min_w_0()
        .border_1()
        .border_color(rgb(t::BORDER_SOFT))
        .bg(rgb(t::SURFACE))
        .rounded(px(t::RAD_SM))
        .px(px(10.0))
        .py(px(7.0))
        .overflow_hidden()
        .hover(hov_warm)
        .child(
            div()
                .size(px(7.0))
                .rounded_full()
                .flex_none()
                .bg(rgb(color)),
        )
        .child(
            div()
                .min_w_0()
                .truncate()
                .child(txt(label, s::font(s::Lvl::Meta), t::PRIMARY)),
        )
        .child(
            div()
                .flex_none()
                .ml_auto()
                .child(txt(num, s::font(s::Lvl::Meta), t::MUTED)),
        )
}

/// `.picker` 目标选择器浮层内体：surface · r16 · p 14/16/12。
pub fn picker() -> Div {
    div()
        .bg(rgb(t::SURFACE))
        .rounded(px(t::RAD_LG))
        .pl(px(16.0))
        .pr(px(16.0))
        .pt(px(14.0))
        .pb(px(12.0))
}

// ─────────── 右键上下文菜单（DEC-114 · 各页右键操作 1:1 落地） ───────────
// 视觉与交互同构 chip 下拉浮层（DEC-112/113）：面板内 deferred absolute 绘制
//（vendor 弹层正解 · AnchoredPopup 在 macOS 被拒），w200 · border · SURFACE ·
// RAD_MD，行为＝anchor 4px 起浮 + 行 hover warm / 按下加深 + on_mouse_down_out
// 点外收起 + Esc 级联先行消化。

/// 菜单面板体：w200 与 dd_panel 同宽（面板 320 视口内右键浮层统一宽）。
pub fn ctx_menu() -> Div {
    div()
        .mt(px(4.0)) // 4px 起浮（与 dd_panel 同口径 · DEC-112 定数）
        .w(px(200.0))
        .border_1()
        .border_color(rgb(t::BORDER))
        .rounded(px(t::RAD_MD))
        .bg(rgb(t::SURFACE))
        .overflow_hidden()
        .flex()
        .flex_col()
}

/// 菜单行：row 基座 + 左标签 12px + 可选右缀（快捷键 meta / ✓ 现值）。
pub fn ctx_row(label: impl Into<String>, trail: Option<String>, checked: bool) -> Div {
    let right = if checked {
        div().child(txt("✓", s::font(s::Lvl::Small), t::PRIMARY))
    } else {
        match trail {
            Some(s) => div().child(meta(s)),
            None => div(),
        }
    };
    row()
        .justify_between()
        .py(px(7.0))
        .child(txt(label, s::font(s::Lvl::Small), t::PRIMARY))
        .child(right)
}

/// 菜单分组小标题（meta 12px 右缀对齐 · 状态/排期等节头）。
pub fn ctx_head(label: impl Into<String>) -> Div {
    div()
        .px(px(16.0))
        .py(px(6.0))
        .bg(rgb(t::NEUTRAL))
        .child(meta(label))
}

/// 菜单分隔线（1px 全宽 · BORDER_SOFT）。
pub fn ctx_sep() -> Div {
    div().h(px(1.0)).bg(rgb(t::BORDER_SOFT))
}

/// 点击位上下文菜单锚（DEC-116 · native context menu 语义）：deferred 包 anchored，
/// 以窗内点击坐标锚定（ClickEvent::position），anchored 内建视口防溢出（默认
/// SwitchAnchor 翻边 + 边缘钳制）。宿主＝各页视图根（祖先链无滚动容器 →
/// element_offset=0，菜单固定绘制于打开瞬间的光标位、不随滚动行漂移——native
/// 菜单同理；滚动仅换行、菜单原位待 out/Esc/项选收起）。
///
/// 浮层 wrapper 统一阻断鼠标穿透（DEC-167 · FAB DEC-148 同款正解）：菜单叠在
/// 页面行之上而 hitbox 无遮挡时，菜单项点击会同时合成身后行的 on_click（开详情
/// /切选中误触发）。block 在挂载点截断 hover 栈——身后 hitbox 不再 is_hovered
/// （不合成 click），滚轮经 should_handle_scroll 照常透传，菜单自身子树（行
/// hover/点击）在前方不受影响。挂 wrapper 而非菜单体：一并覆盖 ctx_menu 的
/// mt(4px) 起浮条带（死区点击同样阻断）。全部右键菜单同源收敛（tasks /
/// collect / collection / trash）。
pub fn ctx_anchor_at(pos: gpui::Point<gpui::Pixels>, body: Div) -> gpui::AnyElement {
    let anchor = anchored().position(pos);
    deferred(anchor.child(body.block_mouse_except_scroll())).into_any_element()
}
