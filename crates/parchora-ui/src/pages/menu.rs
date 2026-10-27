//! PG-MENU 菜单栏菜单（UI-6 · REQ-SETTINGS-STATUSBAR / REQ-URL-SCHEME）。
//!
//! HTML 1:1（PG-MENU.html）：夜读顶栏（monitor 图标 · macOS 顶栏 + feather pill 墨迹·图标）→
//! 菜单卡（5 项：呼出面板 / 开始·结束队列 / 存储根 chevron-right / 设置… / 退出）→ hint。
//! 300px 紧凑宽由 app 层窗口尺寸承载。图标语义见 DEC-66。

use gpui::{
    div, px, rgb, Context, Div, InteractiveElement as _, ParentElement as _, Render, Styled, Window,
};

use crate::{components as co, scale as s, tokens as t};

/// 菜单栏菜单视图（演示窗承载；真实 NSStatusItem 归 platform 层）。
pub struct MenuView;

impl MenuView {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self
    }
}

/// 单条菜单行：.row 变体（两端对齐；末行分隔线由调用方 `border_b_0` 收口）。
/// 菜单项为可点行（REQ-SETTINGS-STATUSBAR · PG-MENU）——悬停/指针态按
/// shared.css a.row 口径统一承载；动作接线随产品壳轮（NSStatusItem 菜单 defer）。
fn menu_row(label: &'static str, right: &'static str) -> Div {
    co::row()
        .justify_between()
        .cursor_pointer()
        .hover(co::hov_warm)
        .child(co::t_label(label))
        .child(co::meta(right))
}

impl Render for MenuView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let _s = crate::scale::enter(window, cx);
        let topbar = div()
            .flex()
            .justify_between()
            .items_center()
            .px(px(16.0))
            .py(px(8.0))
            .bg(rgb(t::NIGHT_BG))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.0))
                    .child(co::icon(co::Icon::Monitor, 12.0, t::NIGHT_INK))
                    .child(co::txt("macOS 顶栏", s::font(s::Lvl::Small), t::NIGHT_INK)),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(4.0))
                    .px(px(8.0))
                    .py(px(2.0))
                    .border_1()
                    .border_color(rgb(t::NIGHT_SAGE))
                    .rounded(px(t::RAD_SM))
                    .child(co::icon(co::Icon::Feather, 11.0, t::NIGHT_SAGE))
                    .child(co::txt(
                        "墨迹 · 图标",
                        s::font(s::Lvl::Small),
                        t::NIGHT_SAGE,
                    )),
            );

        let menu_card = div()
            .mx(px(20.0))
            .my(px(10.0))
            .border_1()
            .border_color(rgb(t::BORDER))
            .rounded(px(t::RAD_MD))
            .bg(rgb(t::SURFACE))
            .overflow_hidden()
            .flex()
            .flex_col()
            .child(menu_row("呼出面板", "⌃⇧Space"))
            .child(menu_row("开始 · 结束队列", "进行中 ⇄"))
            .child(
                co::row()
                    .justify_between()
                    .cursor_pointer()
                    .hover(co::hov_warm)
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(2.0))
                            .child(co::t_label("存储根"))
                            .child(co::icon(co::Icon::ChevronRight, 11.0, t::MUTED)),
                    )
                    .child(co::meta("子菜单（≥2 根时）"))
                    .border_b_0(),
            )
            .child(
                div().border_t_1().border_color(rgb(t::BORDER_SOFT)).child(
                    co::row()
                        .justify_between()
                        .cursor_pointer()
                        .hover(co::hov_warm)
                        .child(co::txt("设置…", s::font(s::Lvl::Tab), t::TERTIARY))
                        .child(co::meta("→ 设置窗")),
                ),
            )
            .child(menu_row("退出", "（逃生口之一）"));

        div()
            .flex_1()
            .min_h_0()
            .flex()
            .flex_col()
            .bg(rgb(t::SURFACE))
            .child(topbar)
            .child(menu_card)
            .child(co::hint(
                "菜单打开期间边缘检测暂停 · URL Scheme 3 命令 · 焦点交接 ≥0.15s",
            ))
    }
}
