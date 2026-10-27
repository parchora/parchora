//! zorite-spike runtime smoke: open a real window, render EditorState (zorite)
//! on the vendored gpui-fast line with a parchment-light SyntaxStyle, log
//! events to stderr. Killed by the runner after ~15s => survival = PASS.

use gpui::{
    prelude::*, App, AppContext, Bounds, Context, Entity, Focusable, InteractiveElement,
    IntoElement, ParentElement, Render, ScrollHandle, SharedString, StatefulInteractiveElement,
    Styled, Subscription, Window, WindowBounds, WindowOptions, div, font, hsla, px, rgb, size,
};
use zorite_editor::{EditorEvent, EditorState, SyntaxStyle};

struct Smoke {
    editor: Entity<EditorState>,
    _sub: Subscription,
    scroll: ScrollHandle,
}

impl Render for Smoke {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .bg(rgb(0xf7f4ee))
            .text_color(rgb(0x1c1e21))
            .text_size(px(15.))
            .child(
                div()
                    .id("spike-scroll")
                    .size_full()
                    .overflow_y_scroll()
                    .track_scroll(&self.scroll)
                    .p(px(24.))
                    .child(
                        div()
                            .bg(rgb(0xfffdf8))
                            .border_1()
                            .border_color(rgb(0xd8d2c4))
                            .rounded(px(6.))
                            .p(px(16.))
                            .child(self.editor.clone()),
                    ),
            )
    }
}

fn parchment_style() -> SyntaxStyle {
    SyntaxStyle {
        block_label: None,
        block_label_gen: 0,
        block_ref_count: None,
        marker: hsla(0.08, 0.08, 0.45, 0.45),
        code: hsla(0.02, 0.62, 0.42, 1.0),
        code_bg: hsla(0.1, 0.35, 0.93, 0.6),
        link: hsla(0.55, 0.6, 0.42, 1.0),
        tag: hsla(0.32, 0.42, 0.38, 1.0),
        quote: hsla(0.08, 0.08, 0.42, 1.0),
        alert_note: hsla(0.55, 0.6, 0.42, 1.0),
        alert_tip: hsla(0.36, 0.45, 0.4, 1.0),
        alert_important: hsla(0.74, 0.6, 0.45, 1.0),
        alert_warning: hsla(0.12, 0.6, 0.42, 1.0),
        alert_caution: hsla(0.02, 0.65, 0.45, 1.0),
        alert_icons: None,
        rule: hsla(0.08, 0.08, 0.7, 0.4),
        mark_bg: hsla(0.12, 0.75, 0.85, 0.55),
        popover_bg: hsla(0.1, 0.2, 0.98, 1.0),
        popover_border: hsla(0.08, 0.15, 0.8, 1.0),
        popover_fg: hsla(0.08, 0.1, 0.2, 1.0),
        popover_hover: hsla(0.55, 0.5, 0.85, 0.3),
        popover_divider: hsla(0.08, 0.08, 0.85, 1.0),
        popover_danger: gpui::rgb(0xE5484D).into(),
        mono: font("Menlo"),
        property_icon: None,
    }
}

fn main() {
    gpui_platform::application().run(|cx: &mut App| {
        eprintln!("[ZSPIKE] app started");
        zorite_editor::bind_keys(cx);

        let bounds = Bounds::centered(None, size(px(760.), px(520.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |window, cx| {
                eprintln!("[ZSPIKE] window opened");
                let text = "# Parchora spike\n\n**bold**, *italic*, `code`, a \
                            [link](https://example.com) and a #tag.\n\n- item one\n- item two\n\n\
                            ```rust\nfn main() {}\n```\n\n> a quote line\n\n- [x] done task\n- [ ] pending task\n";
                let editor = cx.new(|cx| {
                    EditorState::new(window, cx)
                        .with_placeholder("Type here…")
                        .with_text(text)
                });
                editor.update(cx, |ed, cx| {
                    ed.set_markdown_style(parchment_style(), cx);
                });
                window.focus(&editor.read(cx).focus_handle(cx), cx);

                let handle = editor.clone();
                cx.new(|cx| {
                    let sub = cx.subscribe(
                        &handle,
                        |_smoke: &mut Smoke, _editor, event: &EditorEvent, _cx| {
                            match event {
                                EditorEvent::Changed => eprintln!("[ZSPIKE] changed"),
                                EditorEvent::OpenLink(src) => {
                                    eprintln!("[ZSPIKE] open link {src}")
                                }
                                _ => {}
                            }
                        },
                    );
                    Smoke {
                        editor: handle.clone(),
                        _sub: sub,
                        scroll: ScrollHandle::new(),
                    }
                })
            },
        )
        .expect("[ZSPIKE] window failed");
        cx.activate(true);
        let _ = SharedString::from("alive");
    });
}
