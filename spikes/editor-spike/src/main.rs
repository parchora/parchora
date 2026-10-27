//! editor-spike 真机 demo（gpui-fast 线 + gpui-component 编辑器 + vendored park_offscreen 原语）。
//!
//! 用法：
//!   cargo run                    —— 交互模式：窗口常驻展示多行编辑器（Textarea）
//!   cargo run -- --selftest      —— 自测模式：1s 停放屏外 → 2s 后还原 → 1.5s 后断言并退出 PASS/FAIL
//!   cargo run -- --no-park       —— bisect 模式：三段 timer 链不碰窗口（验证调度栈健康度）
//!
//! 自测判定：不崩溃 + 全程日志 + 退出码；停放/还原的视觉行为由真机肉眼复核
//! （窗口消失又原位重现），运行时腿在 10.15.7 实机由 scripts/catalina-check.sh 再验。
//!
//! 启动惯例取自 gpui-fast-platform 官方（application() → Application::run）与
//! gpui-component 惯例（cx.open_window + gpui_component::init）。vendored gpui-fast
//! 的 lib 名即 `gpui`，与 gpui-component内部的 `extern crate ... as gpui` 语义一致。

use std::io::Write as _;
use std::time::Duration;

use gpui::{
    App, AppContext, Bounds, Context, Entity, Global, Render, Window, WindowBounds, WindowOptions,
    div, prelude::*, px, rgb, size,
};
use gpui_component as ui;
use ui::input::{Textarea, TextareaState};

/// 编辑器 demo 视图：顶部说明 + 多行文本域（gpui-component Textarea）。
struct EditorDemo {
    textarea: Entity<TextareaState>,
}

impl EditorDemo {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let textarea = cx.new(|cx| {
            let mut state = TextareaState::new(window, cx);
            state.set_placeholder(
                "Parchora editor spike —— gpui-fast 线真机 demo",
                window,
                cx,
            );
            state
        });
        Self { textarea }
    }
}

impl Render for EditorDemo {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .bg(rgb(0xf7f4ee)) // Parchment 底（DESIGN.md 纸面色示意）
            .p_6()
            .flex()
            .flex_col()
            .gap_3()
            .child(
                div()
                    .text_lg()
                    .text_color(rgb(0x1c1e21))
                    .child("Parchora · editor-spike 真机 demo（gpui-fast 0.1.8 线）"),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(0x5d6470))
                    .child("关闭窗口退出 · 交互模式；--selftest 停放/还原自测"),
            )
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .child(Textarea::new(&self.textarea)),
            )
    }
}

/// 自测全局状态：记录已执行的 park/unpark 结果，供结论步断言。
struct SelftestState {
    parked: bool,
    unparked: bool,
}

impl Global for SelftestState {}

const WINDOW_SIZE: (f32, f32) = (640., 420.);

fn main() {
    let arg = std::env::args().nth(1);
    let selftest = matches!(arg.as_deref(), Some("--selftest") | Some("--no-park"));
    // [D-05 bisect] --no-park：三段 timer 链不碰窗口，定位 park 与唤醒链的关系
    let no_park = matches!(arg.as_deref(), Some("--no-park"));

    let app = gpui_platform::application();
    app.run(move |cx: &mut App| {
        // gpui-component 初始化（官方惯例）
        ui::init(cx);

        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                None,
                size(px(WINDOW_SIZE.0), px(WINDOW_SIZE.1)),
                cx,
            ))),
            ..Default::default()
        };

        let handle = match cx.open_window(options, |window, cx| {
            cx.new(|cx| EditorDemo::new(window, cx))
        }) {
            Ok(handle) => handle,
            Err(err) => {
                eprintln!("[SELFTEST] FAIL —— 窗口创建失败: {err}");
                std::process::exit(1);
            }
        };

        if !selftest {
            return; // 交互模式：窗口常驻，关闭即退出
        }

        // --selftest：有界自动化真机测试
        let t0 = std::time::Instant::now();
        let log = move |msg: &str| println!("[SELFTEST][{:>7.3}s] {}", t0.elapsed().as_secs_f64(), msg);
        cx.set_global(SelftestState {
            parked: false,
            unparked: false,
        });
        log("app 启动，窗口已创建（640x420）");

        let executor = cx.background_executor().clone();
        cx.spawn(async move |cx: &mut gpui::AsyncApp| {
            // t+1s：停放屏外（窗口应从屏幕消失）
            log("进入 task，等待 timer1(1s)...");
            executor.timer(Duration::from_secs(1)).await;
            log("timer1 唤醒");
            let parked = if no_park {
                log("[bisect] --no-park：跳过 PARK");
                true
            } else {
                handle
                    .update(cx, |_view, window, _cx| {
                        println!("[SELFTEST] PARK —— 停放屏外 (-32000, -31000)");
                        window.park_offscreen(true);
                    })
                    .is_ok()
            };
            if !parked {
                eprintln!("[SELFTEST] WARN —— PARK 更新失败（窗口可能已被关闭）");
            }
            log("PARK update 返回，写 set_global #1");
            cx.update(|cx| {
                cx.set_global(SelftestState {
                    parked,
                    unparked: false,
                })
            });
            log("set_global #1 完成，等待 timer2(2s)...");

            // t+3s：还原（窗口应原位恢复）
            executor.timer(Duration::from_secs(2)).await;
            log("timer2 唤醒");
            let unparked = if no_park {
                log("[bisect] --no-park：跳过 UNPARK");
                true
            } else {
                handle
                    .update(cx, |_view, window, _cx| {
                        println!("[SELFTEST] UNPARK —— setFrame:display: 全量还原");
                        window.park_offscreen(false);
                    })
                    .is_ok()
            };
            if !unparked {
                eprintln!("[SELFTEST] WARN —— UNPARK 更新失败（窗口可能已被关闭）");
            }
            log("UNPARK update 返回，写 set_global #2");
            cx.update(|cx| cx.set_global(SelftestState { parked, unparked }));
            log("set_global #2 完成，等待 timer3(1.5s)...");

            // t+4.5s：结论输出并退出
            executor.timer(Duration::from_millis(1500)).await;
            log("timer3 唤醒，输出结论");
            let (parked, unparked) = cx.update(|cx| {
                let s = cx.try_global::<SelftestState>();
                (
                    s.map(|x| x.parked).unwrap_or(false),
                    s.map(|x| x.unparked).unwrap_or(false),
                )
            });
            if parked && unparked {
                println!(
                    "[SELFTEST] PASS —— park/unpark 两步均落地，退出码 0（窗口行为请肉眼复核：消失又原位重现）"
                );
                let _ = std::io::stdout().flush();
                cx.update(|cx| cx.quit());
            } else {
                eprintln!("[SELFTEST] FAIL —— parked={parked} unparked={unparked}");
                let _ = std::io::stdout().flush();
                let _ = std::io::stderr().flush();
                cx.update(|cx| cx.quit());
                std::process::exit(1);
            }
        })
        .detach();
    })
}
