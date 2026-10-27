//! 对话框与浮出物（UI-6 · sheet-dialog 组 · 19 页全承载 DEC-73）：
//! - DG-CONFIRM 二次确认（permanent / 清空回收站共用形态：对象清单 + 不可逆 + 落笔钮）
//!   ——叠层于 CollectionView（PG-B3 语境，confirm_open 状态机）。
//! - DG-CONFLICT 命名冲突四选一（--page conflict 独立演示窗；触发链随批量移动/恢复实装）。
//! - DG-EXT 外部变更挂起（--page ext 独立演示窗；dirty 冲突触发链随 notes/files 实装）。
//! - DG-PICKER 选择手记（--page picker 独立演示窗；⌘⌥K/右键触发链随 TASK-502 实装）。
//! - DG-ROOT 启动存储根选择器（--root 独立演示窗；askOnLaunch 判定在 TASK-204/402 层）。
//!
//! sheet-dialog 形态：surface · 顶 16 圆角 · p 20/24；落笔为唯一主操作。
//! 演示宿主承载口径：视图形态与 HTML stage 1:1，真实触发链在数据层实装后接宿主页
//! （批量移动 / 恢复被占 / F6 外部检测 / 右键·⌘⌥K）。

use gpui::{
    div, px, rgb, ClickEvent, Context, Div, InteractiveElement as _, ParentElement as _, Render,
    StatefulInteractiveElement as _, Styled, Window,
};

use crate::{components as co, pages::collection, scale as s, tokens as t};

/// DG-CONFIRM sheet（叠加在 CollectionView 之上的浮层；确认态由 CollectionView 持有）。
/// 语境动态（DEC-195）：Purge(题名)＝单件永久删除文案；Empty＝清空文案（项数取
/// 挂起时回收站条目数）；None＝演示语境 1:1 原文案。
pub fn confirm_sheet(
    v: &collection::CollectionView,
    cx: &mut Context<collection::CollectionView>,
) -> Div {
    let (heading, action) = match v.confirm_context() {
        Some(collection::TrashAction::Purge(title)) => {
            (format!("永久删除「{title}」？"), "永久删除")
        }
        Some(collection::TrashAction::Empty) => (
            format!("清空回收站（{} 项）？", v.trash_count()),
            "清空并永久删除",
        ),
        None => ("永久删除所选 2 项？".to_string(), "永久删除"),
    };
    let scrim = div()
        .absolute()
        .inset_0()
        .bg(gpui::rgba(t::INK_DEEP << 8 | 0x4D));
    let sheet = co::sheet()
        .relative()
        .m(px(20.0))
        .child(co::sheeth().child(co::txt_f(
            heading,
            s::font(s::Lvl::Small),
            t::PRIMARY,
            t::FONT_BODY,
            t::FONT_WEIGHT_REGULAR,
        )))
        .child(co::meta(
            "该操作不可逆，删除后无法恢复。回收站内条目保留 60 天后亦自动清空。",
        ))
        .child(
            div()
                .flex()
                .gap(px(10.0))
                .mt(px(14.0))
                .child(
                    co::btn_pri(action)
                        .id("dg-confirm-yes")
                        .on_click(cx.listener(
                            |this, _e: &ClickEvent, _w: &mut Window, cx: &mut Context<_>| {
                                this.close_confirm_ok(cx)
                            },
                        ))
                        .active(co::act_ink_deep),
                )
                .child(
                    co::btn_sec("取消（全部保留）")
                        .id("dg-confirm-no")
                        .on_click(cx.listener(
                            |this, _e: &ClickEvent, _w: &mut Window, cx: &mut Context<_>| {
                                this.confirm_open = false;
                                cx.notify();
                            },
                        ))
                        .active(co::act_border_ink),
                ),
        );

    div()
        .absolute()
        .left_0()
        .right_0()
        .bottom_0()
        .child(scrim)
        .child(sheet)
}

/// DG-ROOT 启动根选择器视图（独立演示窗承载；askOnLaunch 语义在 app 层）。
#[derive(Default)]
pub struct RootPicker {
    pub selected: usize,
}

impl RootPicker {
    pub fn view(&self, cx: &mut Context<Self>) -> Div {
        let lockup = div()
            .flex()
            .flex_col()
            .items_center()
            .pt(px(24.0))
            .pb(px(8.0))
            .child(co::txt_f(
                "PARCHORA",
                s::font(s::Lvl::Wordmark),
                t::PRIMARY,
                t::FONT_DISPLAY,
                t::FONT_WEIGHT_REGULAR,
            ))
            .child(co::meta("选择本次使用的存储根"));

        // 根选择 = 双行可点选 + 高亮随 self.selected 迁移（askOnLaunch 默认根 = ~/notes）。
        let root1 = {
            let base = if self.selected == 0 {
                co::sel_state(co::row())
            } else {
                co::row()
            };
            base.id("root-1")
                .hover(co::hov_warm)
                .active(co::act_border)
                .on_click(cx.listener(
                    |this, _e: &ClickEvent, _w: &mut Window, cx: &mut Context<Self>| {
                        this.selected = 0;
                        cx.notify();
                    },
                ))
                .child(
                    // min_w_0（DEC-229）：双行文本（路径 + meta）行项收纳，长串不撑破模态行。
                    div()
                        .flex_1()
                        .min_w_0()
                        .flex()
                        .flex_col()
                        .child(co::txt_f(
                            "~/notes",
                            13.0,
                            t::PRIMARY,
                            t::FONT_CONTENT,
                            t::FONT_WEIGHT_REGULAR,
                        ))
                        .child(co::meta("默认 · DB 指针 238 条 · 上次使用 10 分钟前")),
                )
        };
        let root2 = {
            let base = if self.selected == 1 {
                co::sel_state(co::row())
            } else {
                co::row()
            };
            base.id("root-2")
                .hover(co::hov_warm)
                .active(co::act_border)
                .on_click(cx.listener(
                    |this, _e: &ClickEvent, _w: &mut Window, cx: &mut Context<Self>| {
                        this.selected = 1;
                        cx.notify();
                    },
                ))
                .child(
                    // min_w_0（DEC-229）：同 root1 口径。
                    div()
                        .flex_1()
                        .min_w_0()
                        .flex()
                        .flex_col()
                        .child(co::txt_f(
                            "~/sync/parchora-files",
                            13.0,
                            t::PRIMARY,
                            t::FONT_CONTENT,
                            t::FONT_WEIGHT_REGULAR,
                        ))
                        .child(co::meta("同步盘根 · data 本机 / files 可同步 · 184 条")),
                )
        };

        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(rgb(t::SURFACE))
            .child(lockup)
            .child(
                div()
                    .px(px(24.0))
                    .pb(px(24.0))
                    .pt(px(8.0))
                    .flex()
                    .flex_col()
                    .child(root1)
                    .child(root2)
                    .child(
                        div()
                            .flex()
                            .gap(px(10.0))
                            .mt(px(14.0))
                            .child(co::btn_pri("打开所选根").id("root-open").flex_1())
                            .child(co::btn_sec("在设置中管理 →").id("root-manage")),
                    )
                    .child(co::meta(
                        "切换前先保存脏数据（.md 冲刷 + DB 提交）· 本选择仅本次会话生效",
                    )),
            )
    }
}

impl Render for RootPicker {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let _s = crate::scale::enter(window, cx);
        self.view(cx)
    }
}

/// DG-CONFLICT 命名冲突四选一视图（独立演示窗承载；HTML stage 1:1：
/// 变暗背景语境 + 底部落墨 sheet——sheeth 冲突双方 + 冲突计数行 + 六钮行 + 失败说明）。
/// 触发链（PG-B 批量移动重名 / PG-B3 恢复被占位）随数据层实装后接宿主页。
pub struct ConflictSheet;

impl ConflictSheet {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self
    }
}

impl Render for ConflictSheet {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let _s = crate::scale::enter(window, cx);
        // 底部落墨 sheet（left/right 15 · sheet 本体 surface · 16px 顶圆角）。
        let buttons = div()
            .flex()
            .flex_wrap()
            .gap(px(8.0))
            .mt(px(10.0))
            .mb(px(16.0))
            .child(co::btn_pri("保留两者"))
            .child(co::btn_sec("替换"))
            .child(co::btn_sec("跳过"))
            .child(co::btn_sec("全部保留两者"))
            .child(co::btn_sec("全部跳过"))
            .child(co::btn_sec("取消"));
        let sheet = co::sheet()
            .child(co::sheeth().child(co::txt_f(
                "「调研摘录.md」已存在于目标文件夹",
                s::font(s::Lvl::Small),
                t::PRIMARY,
                t::FONT_BODY,
                t::FONT_WEIGHT_REGULAR,
            )))
            .child(
                co::row()
                    .border_b_0()
                    .child(co::meta("冲突 1 / 3 · 本页逐个浮现，>1 时提供批量按钮")),
            )
            .child(buttons)
            .child(co::meta(
                "任何操作失败 → 中止本批移动（已迁移项不做回滚说明按 PRD 口径）",
            ));

        div()
            .size_full()
            .relative()
            .overflow_hidden()
            .flex()
            .flex_col()
            .bg(rgb(t::NEUTRAL))
            .child(
                div()
                    .p(px(16.0))
                    .child(co::meta("背景：PG-B 整理 / PG-B3 恢复 原位（变暗示意）")),
            )
            .child(
                div()
                    .absolute()
                    .left(px(15.0))
                    .right(px(15.0))
                    .bottom_0()
                    .child(sheet),
            )
    }
}

/// DG-EXT 外部变更冲突挂起视图（独立演示窗承载；HTML stage 1:1：tertiary 警示 grp 头
/// + 说明段 + 双方对照 sec（本地 dirty / 磁盘外部版本）+ 保留本地（pri）/重载磁盘（sec）
/// + 尾注）。触发链（面板可见时 mtime 对账 dirty 分支）随 notes/files 实装后接宿主页。
pub struct ExtConflictView;

impl ExtConflictView {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self
    }
}

impl Render for ExtConflictView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let _s = crate::scale::enter(window, cx);
        let contrast_box = div()
            .my(px(12.0))
            .bg(rgb(t::NEUTRAL))
            .rounded(px(t::RAD_MD))
            .px(px(24.0))
            .py(px(20.0))
            .flex()
            .flex_col()
            .gap(px(2.0))
            .child(co::meta("本地：未保存的编辑（dirty）"))
            .child(co::meta("磁盘：mtime 变化 · 外部版本"));

        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(rgb(t::NEUTRAL))
            .child(
                // 警示 grp 头（tertiary · 字距 defer D-01）。
                div()
                    .flex()
                    .items_center()
                    .px(px(16.0))
                    .py(px(8.0))
                    .bg(rgb(t::WARM))
                    .child(co::txt_f(
                        "⚠ 外部变更 · note-ext-conflict",
                        s::font(s::Lvl::Small),
                        t::TERTIARY,
                        t::FONT_BODY,
                        t::FONT_WEIGHT_LIGHT,
                    )),
            )
            .child(
                div()
                    .px(px(24.0))
                    .py(px(20.0))
                    .flex()
                    .flex_col()
                    .child(co::content_text(
                        "磁盘上的「调研摘录.md」已被其他程序修改，当前编辑区存在未保存改动。",
                        13.0,
                        t::PRIMARY,
                    ))
                    .child(contrast_box)
                    .child(
                        div()
                            .flex()
                            .gap(px(10.0))
                            .child(co::btn_pri("保留本地"))
                            .child(co::btn_sec("重载磁盘版本")),
                    )
                    .child(
                        div()
                            .mt(px(12.0))
                            .child(co::meta("不静默覆盖任一方 · 挂起期间可取消操作回到编辑")),
                    ),
            )
    }
}

/// DG-PICKER 选择手记浮层视图（独立演示窗承载；HTML stage 1:1：bar（加入到手记 + ⌘⌥K）
/// → 打开即搜 → grp 最近加入 ≤5 → 三行（首行 sel 态）→ 新建手记落笔钮 → 守卫 hint）。
/// 触发链（⌘⌥K · DEC-163）随
/// TASK-502 实装后接 PG-A。
pub struct PickerSheet;

impl PickerSheet {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self
    }
}

impl Render for PickerSheet {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let _s = crate::scale::enter(window, cx);
        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(rgb(t::NEUTRAL))
            .child(
                co::bar()
                    .child(co::txt_f(
                        "加入到手记",
                        s::font(s::Lvl::Small),
                        t::PRIMARY,
                        t::FONT_BODY,
                        t::FONT_WEIGHT_REGULAR,
                    ))
                    .child(co::meta("⌘⌥K")),
            )
            .child(co::search_box(
                "打开即搜（150ms 防抖 · 虚拟列表）",
                co::Icon::Search,
            ))
            .child(co::grp("最近加入 · ≤5", Some(co::meta("会话记忆目标层"))))
            .child(
                co::sel_state(co::row())
                    .hover(co::hov_warm)
                    .child(co::t_label("调研摘录"))
                    .child(co::meta("设计笔记")),
            )
            .child(
                co::row()
                    .hover(co::hov_warm)
                    .child(co::t_label("速记 · 待整理名单"))
                    .child(co::meta("速记")),
            )
            .child(
                co::row()
                    .hover(co::hov_warm)
                    .border_b_0()
                    .child(co::t_label("羊皮纸视觉系统推导")),
            )
            .child(
                div()
                    .p(px(12.0))
                    .flex()
                    .justify_center()
                    .child(co::btn_pri("新建手记")),
            )
            .child(co::hint(
                "守卫：重复加入 → 提示｜空内容 / 不支持类型 → 不入列｜目标被外部删除 → Toast「《XX》已不存在」+ 重弹本层 · Esc 关闭回 拾记",
            ))
    }
}
