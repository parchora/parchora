//! parchora-ui —— GPUI 视图与组件（M3/M4/M5 视图面）。
//!
//! 依赖边界（design §2.1）：唯一可依赖 gpui 的业务 crate（platform 为桥接层不做视图）。
//!
//! 当前推进：GUI 轮（对照 `页面设计/HTML/` 19 页 + parchora-shared.css 1:1 实现）。
//! - `tokens`（UI-0）：羊皮墨设计令牌唯一来源；
//! - `components`（UI-1）：shared.css 组件原语；
//! - `pages`（UI-2~6）：19 页视图；演示数据取自 HTML 设计稿同源示例；
//! - `dock`（DEC-67/76/78 · DEC-84）：7 向
//!   触发位置与停靠几何（可用区
//!   域边缘语义 + Dock 冲突自动切换；「顶部」无对应触发位（DEC-84/86/87）；
//!   收缩态 = 主面板整窗关闭全隐）；
//! - `walltime`：跨运行时一致的墙钟时间；
//! - `scale`（DEC-198）：分辨率/缩放档位 → 字号/间距/窗体几何档位表（每屏建档
//!   + 渲染期 TLS 栈接入）。
//! - 真实数据/输入引擎接线：TASK-300/502/504（ui 层仅承载展示与演示交互）。

pub mod clipstamp;
pub mod components;
pub mod dock;
pub mod icons;
pub mod pages;
pub mod scale;
pub mod tokens;
pub mod walltime;

use gpui::{rgb, App};

use crate::tokens as t;

/// 输入引擎装配（TASK-504）：
/// gpui-component 0.7.3 初始化（editor-spike 真机已验证同款惯例）＋羊皮墨主题
/// 色对齐——输入态文字/光标/选区/边框全部取自 DESIGN.md 17 色令牌（t::*），
/// 组件默认 zinc 白底主题不再以白底黑字污染编辑/输入区。app 层于 Application
/// ::run 首行调用（editor-spike 同源挂点）。
pub fn init(cx: &mut App) {
    gpui_component::init(cx);
    gpui_component::Theme::update(cx, |theme| {
        theme.foreground = rgb(t::PRIMARY).into(); // 主墨正文
        theme.muted_foreground = rgb(t::MUTED).into(); // 占位/次级文字
        theme.caret = rgb(t::INK_DEEP).into(); // 光标
        theme.selection = rgb(t::WARM).into(); // 选区底（DESIGN.md 选中底）
        theme.border = rgb(t::BORDER).into(); // 细边框
                                              // 滚动条装配（DEC-136 · 编辑区垂直滚动条主题面）：常驻模式
                                              //（溢出即显·无溢出隐藏在 gpui-base scrollbar 自证）；拇指/悬停色取羊皮
                                              // 墨令牌。策略仅落编辑区零连带。
        theme.scrollbar_mode = gpui_component::scroll::ScrollbarMode::Always;
        theme.colors.scrollbar_thumb = rgb(t::BORDER).into();
        theme.colors.scrollbar_thumb_hover = rgb(t::MUTED).into();
    });
    // md 引擎编辑键装配（DEC-150）：zorite
    // 宿主义务「bind_keys 于启动时调用一次」（vendor lib.rs Usage 段）——退格/
    // 方向键/选区/undo/redo/格式裹等编辑动作的键绑定按 zorite CONTEXT「Editor」
    // 注册；与 gpui_component::init（源码态 Textarea/标题/查找 Input 的绑定源）
    // 同位并立，缺此则混合态引擎仅可输入字符（macOS 特殊键无 key_char 兜底）。
    zorite_editor::bind_keys(cx);
    // Esc 通道接管（DEC-151）：bind_keys 的 escape→Dismiss（ctx「Editor」）在
    // action bubble 相位默认截停（vendor window.rs L6615），root on_key_down 的
    // Esc 关窗监听不可达——后追加同 context 绑定以宿主自有 EditorEscape 接管
    //（后注册优先，测试 md_escape_still_fires_sink_with_bindings 实证），语义仍
    // 落 EditorEvent::Escape 统一漏斗（DEC-150 首按即关窗不改）。
    // 待办创建面板 Esc 同构（DEC-165）：context「TaskCreate」自有 action 接管，
    // 与编辑窗「Editor」域互不相扰。
    cx.bind_keys([
        gpui::KeyBinding::new("escape", crate::pages::editor::EditorEscape, Some("Editor")),
        gpui::KeyBinding::new(
            "escape",
            crate::pages::tasks::TaskCreateEscape,
            Some("TaskCreate"),
        ),
    ]);
}
