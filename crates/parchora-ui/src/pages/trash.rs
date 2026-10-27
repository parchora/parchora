//! PG-B3 回收站（UI-5 · REQ-STORE-TRASH*：60 天保留 / 只读 / 二次确认）。
//!
//! HTML 1:1（PG-B3.html）：bar（← 集锦 · 回收站 · 清空…tertiary）→ 计数行
//! → 行列表（真实条目 → read-only 面语境）→ hint。
//! 图标（DEC-66）：返回=arrow-left · 文件夹行 📁 → folder · 空态墨痕 ◌ → circle-dashed。
//!
//! 真数据实装（DEC-195）：宿主 app 层扫描 files 同级 `.trash/`（仅普通 .md ·
//! mtime 倒序），经 `PanelView::set_trash_notes` → `CollectionView::set_trash_notes`
//! 注入（唯一刷新入口 · 全量替换幂等 · 镜像集锦首页磁盘面口径）。触发点：面板
//! 唤出、首页「删除→回收站」真移后、单件永久删除后、清空后。计数行显示真实 N。
//!
//! 行交互（DEC-183 · DEC-114/116 机器保留）：单击 → 预览（统一详情窗 · 非
//! 磁盘条目不入盘不复活）；右键 → 恢复（在册 defer Toast 口径，真链路随
//! TASK-50x）/ 永久删除（真实开 DG-CONFIRM · 确认后宿主物理删除）；「清空…」
//! → DG-CONFIRM（Empty 挂起 · 确认后物理清空）。锚定＝点击位（DEC-116）。
//!
//! 行右键菜单接线（DEC-114/116 · 状态由 trash_ctx 承载 · DB 真链路仍 TASK-50x）：
//! 恢复=Toast defer 口径；永久删除=真实开 DG-CONFIRM；菜单本体同 collect.rs；
//! 锚定＝点击位（DEC-116：窗内点击坐标 · 视图根挂载）。

use gpui::prelude::FluentBuilder as _;
use gpui::{
    div, rgb, ClickEvent, Context, Div, InteractiveElement as _, ParentElement as _,
    StatefulInteractiveElement as _, Styled, Window,
};

use crate::pages::peek::{PeekItem, PeekKind};
use crate::{components as co, pages::collection, scale as s, tokens as t};

/// PG-B3 视图（面板内页 · DEC-183 · DEC-195 携视图读点两参——Context 无 deref
/// 与全仓模式一致）。
pub fn view(
    v: &mut collection::CollectionView,
    cx: &mut Context<collection::CollectionView>,
) -> Div {
    let rows = v.trash_rows_view();
    let bar = co::bar()
        .child(
            co::ibtn_icon_text(co::Icon::ArrowLeft, "集锦", t::PRIMARY)
                .id("tr-back")
                .on_click(cx.listener(
                    |this, e: &ClickEvent, w: &mut Window, cx: &mut Context<_>| {
                        this.go_home(e, w, cx)
                    },
                ))
                .active(co::act_border),
        )
        .child(co::txt_f(
            "回收站",
            s::font(s::Lvl::Small),
            t::PRIMARY,
            t::FONT_BODY,
            t::FONT_WEIGHT_REGULAR,
        ))
        .child(
            co::ibtn("清空…")
                .id("tr-empty")
                .text_color(rgb(t::TERTIARY))
                .on_click(cx.listener(
                    |this, _e: &ClickEvent, _w: &mut Window, cx: &mut Context<_>| {
                        this.open_confirm(cx)
                    },
                ))
                .active(co::act_border),
        );

    let count = co::count_line(format!("{} 项 · 已删除条目保留 60 天", rows.len()));

    // 列表（DEC-195 · 真实条目行）：单击 → 预览（非磁盘条目不落盘）；右键 →
    // 恢复（defer）/ 永久删除（DG-CONFIRM）。
    let mut list = div().flex().flex_col();
    for (k, d) in rows.iter().enumerate() {
        let title = d.title.clone();
        let item = PeekItem {
            kind: PeekKind::Note,
            title: title.clone(),
            meta: format!("回收站 · {}", d.created),
            body: d.body.clone(),
            disk_real: false,
        };
        list = list.child(
            co::row()
                .id(("tr-row", k))
                // 回收站行回归取样面（DEC-195）：debug_bounds 按「tr-row-<k>」
                // 定位注入点击；release no-op 不入生产样式（DEC-137 同式）。
                .debug_selector(move || format!("tr-row-{}", k))
                .on_click(cx.listener(
                    move |this, e: &ClickEvent, w: &mut Window, cx: &mut Context<_>| {
                        this.trash_row_preview(&item, e, w, cx)
                    },
                ))
                .on_aux_click(cx.listener(
                    move |this, ev: &ClickEvent, _w: &mut Window, cx: &mut Context<_>| {
                        this.trash_ctx_open(title.clone(), ev.position(), cx)
                    },
                ))
                .hover(co::hov_warm)
                .active(co::act_border)
                .child(
                    div()
                        .flex_none()
                        .child(co::icon(co::Icon::FileText, 12.0, t::MUTED)),
                )
                .child(co::t_label(d.title.clone()))
                .child(co::meta(d.created.clone()))
                .child(div().flex_none().child(co::tgo())),
        );
    }
    // 空态（DEC-195 · 真实条目域）。
    if rows.is_empty() {
        list = list.child(co::empty_state(
            co::Icon::CircleDashed,
            "回收站没有条目 · 删除的笔记会出现在这里",
        ));
    }

    div()
        .flex_1()
        .min_h_0()
        .flex()
        .flex_col()
        .bg(rgb(t::NEUTRAL))
        .relative() // Toast 锚定域（DEC-114 集锦 Toast 常驻底部）
        .child(bar)
        .child(count)
        .child(
            div()
                .id("tr-scroll")
                .flex_1()
                .min_h_0()
                .overflow_y_scroll()
                .child(list),
        )
        // 行右键菜单挂载（DEC-195 · ctx_anchor_at 窗级 anchored · DEC-116 口径
        // 同首页菜单）；DG-CONFIRM 挂起时菜单已收层（purge 路即 take）。
        .when(v.trash_menu_open(), |el| el.child(trash_ctx_layer(v, cx)))
        .child(co::hint_marked(
            co::Icon::CircleDashed,
            "打开条目 = 预览 · 右键：恢复 / 永久删除（经 DG-CONFIRM）· 清空＝页首「清空…」",
        ))
}

/// 回收站行右键菜单本体（DEC-195 · DEC-114/116）：恢复（defer Toast）＋ 永久删除
///（真实开 DG-CONFIRM）；out 包 wrapper 同 home_ctx_layer 形态；锚定＝点击位
///（ctx_anchor_at 窗级 anchored）。unwrap 兕底不可达——开启必写坐标。
fn trash_ctx_layer(
    v: &collection::CollectionView,
    cx: &mut Context<collection::CollectionView>,
) -> gpui::AnyElement {
    let view = cx.weak_entity();
    let out = move |_: &gpui::MouseDownEvent, _: &mut gpui::Window, cx: &mut gpui::App| {
        let _ = view.update(cx, |v, cx| {
            if v.trash_ctx.is_some() {
                v.trash_ctx = None;
                cx.notify();
            }
        });
        cx.stop_propagation();
    };
    let menu = co::ctx_menu()
        .child(
            co::ctx_row("恢复", None, false)
                .id("ctx-t-restore")
                .on_click(cx.listener(
                    |this, _e: &ClickEvent, _w: &mut Window, cx: &mut Context<_>| {
                        this.trash_ctx_restore(cx)
                    },
                ))
                .hover(co::hov_warm)
                .active(co::act_border),
        )
        .child(co::ctx_sep())
        .child(
            co::ctx_row("永久删除", None, false)
                .id("ctx-t-purge")
                // 测试取样面（DEC-195）：debug_bounds 按「tr-menu-purge」定位
                // 注入点击；release no-op 不入生产样式（DEC-137 同式）。
                .debug_selector(|| "tr-menu-purge".to_string())
                .on_click(cx.listener(
                    |this, _e: &ClickEvent, _w: &mut Window, cx: &mut Context<_>| {
                        this.trash_ctx_purge(cx)
                    },
                ))
                .hover(co::hov_warm)
                .active(co::act_border),
        );
    co::ctx_anchor_at(
        v.trash_menu_anchor(),
        div().on_mouse_down_out(out).child(menu),
    )
}
