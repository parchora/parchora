//! G1 帧遥测（gpui-fast 性能轮 · DEC-215 / perf skill 阶段 1）。
//!
//! 面向真实生产帧环路（macOS DisplayLink 信号 → Window::draw → present）的
//! 帧级测量管线：编译期挂 `frame-profiler` feature（gpui-fast `profiler`
//! 运行时采集面，vendor 既有 opt-in · 零 vendor 改动），运行期
//! `PARCHORA_FRAME_TELEMETRY=1` 激活。激活后限时窗运行：预热（等守护与
//! 首屏稳定）→ 开启 trace → 以固定频率对全部活动窗 `refresh()` 强制失效
//! （场景语义：窗口级强制失效帧 = 全量链路成本上界，与真实状态变化帧同
//! 环路）→ 采集 FrameTiming/PresentTiming 配对 → 输出 P50/P90/P95/P99 与
//! on-time/late 判定 → 自动退出进程（测量运行语义，配置档/存储随宿主 HOME）。
//!
//! 指标域：`draw` = Window::draw 全时长（request_layout/prepaint/paint 三相
//! 分列）；`work` = 帧 work_start（signal 与 dirty 较晚者）→ present 提交点，
//! 即 perf skill 阶段 1 口径的「帧工作时长」；判迟依据 vendor
//! FrameTiming::budget（刷新间隔下限 8.33ms · 未知取 16.67ms）。
//!
//! 环境变量（可选）：`PARCHORA_FRAME_TELEMETRY_WARMUP_S`（默认 2.0）·
//! `PARCHORA_FRAME_TELEMETRY_DURATION_S`（默认 8.0）·
//! `PARCHORA_FRAME_TELEMETRY_INVALIDATE_HZ`（默认 60.0；**显式 `0` ＝ passive
//! 观测模式**——不注入任何驱动拍，只数自然帧，供阶段 2 空闲静性判定）·
//! `PARCHORA_FRAME_TELEMETRY_SCENARIO`（场景轮 · DEC-217：`reveal`＝测量窗
//! 开窗直呼 `open_panel_docked`（转场帧 + 面板在场尾段）· `reveal-close`＝
//! 测量窗首拍 `close_panel`（收板转场 + 面板缺席稳态尾段 · 尾拍守卫真域）·
//! `scroll`＝按 `SCROLL_HZ` 注入合成滚轮（platform 真事件管线桥）· 仅
//! passive 模式承载场景动作·未知值按无场景处理并stderr 披露）。机器可读
//! 汇总：stderr 首尾 `[parchora-frame]` 行；JSON 单行居报告末行。

use std::collections::HashMap;
use std::time::Duration;

use gpui::profiler::{set_trace_enabled, FrameEvent, FrameTimingCollector, PresentTiming};
use gpui::{
    point, App, AppContext, MouseMoveEvent, PlatformInput, ScrollDelta, ScrollWheelEvent,
    TouchPhase,
};

/// 预热秒数（默认）：等启动守护/首帧渲染稳定后再开采集。
const TELEMETRY_WARMUP_S: f32 = 2.0;
/// 采集时长（默认）：预热后实际测量窗长。
const TELEMETRY_DURATION_S: f32 = 8.0;
/// 失效驱动频率（默认）：测量窗内逐拍对活动窗 refresh() 的频率。
const TELEMETRY_INVALIDATE_HZ: f32 = 60.0;

/// 滚动腿注入频率（场景轮 · DEC-217）：15Hz 滚轮拍——真实滚轮高频段代表性。
const SCROLL_HZ: f32 = 15.0;
/// 滚动腿每拍滚距（line 单位，负 = 向下）：1 行/拍 · 12s 窗总量 ≈180 行
/// ≪ 200 行种子列表的视口外余量（滚到底前不空转）。
const SCROLL_LINES_PER_TICK: i32 = -1;
/// 场景名（env 解析后以常量字面量比较 · 未知值按无场景处理并披露）。
const SCENARIO_REVEAL: &str = "reveal";
const SCENARIO_REVEAL_CLOSE: &str = "reveal-close";
const SCENARIO_SCROLL: &str = "scroll";

/// 帧遥测启动公告与报告的 stderr 前缀（区别于应用常规 `[parchora]` 前缀）。
const TELEMETRY_TAG: &str = "[parchora-frame]";

/// 读 env 秒数浮点（非法/缺失回默认），下限防零除。
fn env_secs(key: &str, default: f32) -> f32 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse::<f32>().ok())
        .filter(|v| *v > 0.0)
        .unwrap_or(default)
}

/// 读失效驱动频率（专用解析 · 不走 env_secs）：仅此键区分档位——显式 `0`
/// ＝ passive 观测模式（不驱动，只数自然帧）；未置/非法回默认（G1 强制失效
/// 口径不变；负值按缺省处理）。
fn env_invalidate_hz() -> f32 {
    std::env::var("PARCHORA_FRAME_TELEMETRY_INVALIDATE_HZ")
        .ok()
        .and_then(|v| v.parse::<f32>().ok())
        .filter(|v| *v >= 0.0)
        .unwrap_or(TELEMETRY_INVALIDATE_HZ)
}

/// 启动入口（main run 闭包末尾挂点）：env 未置则零操作。
pub(crate) fn maybe_start(cx: &App) {
    if std::env::var("PARCHORA_FRAME_TELEMETRY").as_deref() != Ok("1") {
        return;
    }
    let warmup_s = env_secs("PARCHORA_FRAME_TELEMETRY_WARMUP_S", TELEMETRY_WARMUP_S);
    let duration_s = env_secs("PARCHORA_FRAME_TELEMETRY_DURATION_S", TELEMETRY_DURATION_S);
    let invalidate_hz = env_invalidate_hz();
    let passive = invalidate_hz == 0.0;
    let scenario = match std::env::var("PARCHORA_FRAME_TELEMETRY_SCENARIO").as_deref() {
        Ok(SCENARIO_REVEAL) => SCENARIO_REVEAL.to_string(),
        Ok(SCENARIO_REVEAL_CLOSE) => SCENARIO_REVEAL_CLOSE.to_string(),
        Ok(SCENARIO_SCROLL) => SCENARIO_SCROLL.to_string(),
        Ok(other) => {
            eprintln!(
                "{} 未知场景名 {other:?}（可用：reveal / reveal-close / scroll）——按无场景处理",
                TELEMETRY_TAG
            );
            String::new()
        }
        Err(_) => String::new(),
    };
    if passive {
        eprintln!(
            "{} 帧遥测启动（DEC-215 G1 → G2）：warmup={warmup_s:.1}s duration={duration_s:.1}s \
             · 模式=passive（仅观测自然帧·无驱动拍）{}",
            TELEMETRY_TAG,
            if scenario.is_empty() {
                String::new()
            } else {
                format!(" · scenario={scenario}")
            }
        );
    } else {
        eprintln!(
            "{} 帧遥测启动（DEC-215 G1）：warmup={warmup_s:.1}s duration={duration_s:.1}s \
             invalidate={invalidate_hz:.0}Hz · 场景=窗口级强制失效（全量链路成本上界）",
            TELEMETRY_TAG
        );
    }
    cx.spawn(async move |cx| {
        cx.background_executor()
            .timer(Duration::from_secs_f32(warmup_s))
            .await;
        // 先开 trace 再建 collector：trace 开启时清缓存并把 total_pushed 归零
        //（vendor profiler.rs clear_trace_buffers），collector 游标须后置才不漏帧。
        set_trace_enabled(true);
        let mut collector = FrameTimingCollector::new();
        let ticks;
        if passive {
            // passive（G2 空闲静性判定）：测量窗一次性挂定时，不注入任何驱动
            // 拍——帧发生只随真实失效源（守护 stamp/事件链/系统信号），窗内
            // 帧数即空闲真实帧率（判据：0~1 帧 = 静）。
            // 场景动作（DEC-217 场景轮）：scenario 指定的真实动作于测量窗
            // 开始时发生一次（open/close），滚动腿按固定频率注入合成滚轮
            // ——除此之外仍零驱动拍（自然管线观测）。
            ticks = 0;
            match scenario.as_str() {
                SCENARIO_REVEAL => {
                    // 唤出腿：生产同径直呼（同 open_panel_docked 调口）——
                    // 开窗转场帧（全树初建）+ 面板在场空闲尾段。
                    cx.update(super::open_panel_docked);
                    cx.background_executor()
                        .timer(Duration::from_secs_f32(duration_s))
                        .await;
                }
                SCENARIO_REVEAL_CLOSE => {
                    // 收板腿：首拍关面板——收板转场帧 + 面板缺席稳态尾段
                    // （reveal 尾拍守卫真域 · DEC-217 hygiene 实域验证）。
                    cx.update(super::close_panel);
                    cx.background_executor()
                        .timer(Duration::from_secs_f32(duration_s))
                        .await;
                }
                SCENARIO_SCROLL => {
                    // 滚动腿：15Hz 滚轮注入——经 gpui 公开 dispatch_event 入口
                    // （vendor 平台层事件回调汇入的同一入口，下游 hit-test/
                    // scroller 真管线同径；NSEvent 级合成被 AppKit 无窗号路由
                    // 私有语义阻断——探测证据链 DEC-217）。每拍先 MouseMove
                    // 定位悬停（滚轮落点 = 列表内容区，同真实滚轮前置）再注入
                    // 滚轮；派发形态照抄 vendor on_input 回调（window.rs
                    // L2059-2066）：handle.update(&mut AsyncApp) 单事件单事务
                    // ——无 AsyncApp::update 外层（该外层与租借嵌套是
                    // double-lease abort 差异点 · 崩溃证据链 DEC-217）。
                    let tick_secs = 1.0 / SCROLL_HZ;
                    let drives = (duration_s * SCROLL_HZ) as u32;
                    for _ in 0..drives {
                        cx.background_executor()
                            .timer(Duration::from_secs_f32(tick_secs))
                            .await;
                        // 拍预处理（独立 App 事务 · 无租借嵌套）：面板在场 +
                        // 靶点局部坐标。AsyncApp::update 直返闭包值（非 Result）。
                        let drive = cx.update(|cx| {
                            cx.global::<crate::DockHandles>().panel.and_then(|handle| {
                                scroll_target(cx)
                                    .map(|(sx, sy)| (handle, to_panel_local(cx, sx, sy)))
                            })
                        });
                        let Some((handle, local)) = drive else {
                            eprintln!("{} scroll 面板不在场，当拍跳过", TELEMETRY_TAG);
                            continue;
                        };
                        // 派发形态（租借对齐 · 崩溃证据链 DEC-217）：真实事件
                        // 平台回调经 AnyWindowHandle::update（不租根视图 ·
                        // window.rs L7593 闭包收 AnyView+Window+App）→ 窗域内
                        // dispatch_event → hover listener view.update 首租。
                        // 若外层再包 WindowHandle::update（先租根视图）→
                        // hover listener 二租 → double-lease abort（已实证）。
                        // 故此处用 App::update_window 同语义（any_handle），
                        // 与真实事件回调租借态完全同构。
                        // WindowHandle deref → AnyWindowHandle（#[deref] ·
                        // 同类型双方法：固有 update 走 update_window 不租视图）。
                        let any_handle: gpui::AnyWindowHandle = *handle;
                        cx.update(|cx| {
                            let _ = cx.update_window(any_handle, |_, window, cx| {
                                window.dispatch_event(
                                    PlatformInput::MouseMove(MouseMoveEvent {
                                        position: local,
                                        pressed_button: None,
                                        modifiers: Default::default(),
                                    }),
                                    cx,
                                );
                            });
                            let _ = cx.update_window(any_handle, |_, window, cx| {
                                window.dispatch_event(
                                    PlatformInput::ScrollWheel(ScrollWheelEvent {
                                        position: local,
                                        delta: ScrollDelta::Lines(point(
                                            0.0,
                                            SCROLL_LINES_PER_TICK as f32,
                                        )),
                                        touch_phase: TouchPhase::Moved,
                                        modifiers: Default::default(),
                                    }),
                                    cx,
                                );
                            });
                        });
                    }
                }
                _ => {
                    cx.background_executor()
                        .timer(Duration::from_secs_f32(duration_s))
                        .await;
                }
            }
        } else {
            let tick_secs = 1.0 / invalidate_hz;
            ticks = (duration_s * invalidate_hz) as u32;
            for _ in 0..ticks {
                cx.background_executor()
                    .timer(Duration::from_secs_f32(tick_secs))
                    .await;
                cx.update(|cx| {
                    for handle in cx.windows() {
                        let _ = handle.update(cx, |_, window, _| window.refresh());
                    }
                });
            }
        }
        let events = collector.collect_unseen();
        set_trace_enabled(false);
        report(&events, ticks, passive, &scenario);
        // 测量运行语义：报告落 stderr 即退出（免外杀；单实例锁随进程释放）。
        cx.update(|cx| cx.quit());
    })
    .detach();
}

/// 一次配对统计（单窗单帧）：draw 全时长（含三相）+ work（signal→present）。
struct FrameSample {
    window_id: u64,
    draw: Duration,
    work: Option<Duration>,
    budget: Duration,
    missed_refreshes: Option<u64>,
    refresh_interval: Option<Duration>,
    invalidations: u64,
    phases: gpui::profiler::DrawPhases,
}

/// FrameEvent 流 → Draw/Present 逐窗配对样本。缓冲按发生序 push（Draw 先于
/// Present）；同窗以最新未配对 Draw 与邻近 Present 配对，无 Present 的 Draw
/// 记未配对帧（draw 仍有数，work 记 None）。
fn pair_events(events: &[FrameEvent]) -> (Vec<FrameSample>, u64) {
    let mut samples = Vec::new();
    let mut pending: HashMap<u64, gpui::profiler::FrameTiming> = HashMap::new();
    let mut unpaired_draws: u64 = 0;
    for event in events {
        match *event {
            FrameEvent::Draw(timing) => {
                if let Some(prev) = pending.insert(timing.window_id.as_u64(), timing) {
                    // 上一 Draw 未等到 Present 即来新 Draw：保留其 draw 统计。
                    unpaired_draws += 1;
                    samples.push(sample_from_draw(&prev, None));
                }
            }
            FrameEvent::Present(present) => {
                if let Some(timing) = pending.remove(&present.window_id.as_u64()) {
                    samples.push(sample_from_draw(&timing, Some(&present)));
                }
            }
        }
    }
    for (_, timing) in pending.drain() {
        unpaired_draws += 1;
        samples.push(sample_from_draw(&timing, None));
    }
    (samples, unpaired_draws)
}

fn sample_from_draw(
    timing: &gpui::profiler::FrameTiming,
    present: Option<&PresentTiming>,
) -> FrameSample {
    let work = present.and_then(|p| timing.work(p.present_start));
    let missed = present.and_then(|p| timing.missed_refreshes(p.present_start));
    FrameSample {
        window_id: timing.window_id.as_u64(),
        draw: timing.draw_duration(),
        work,
        budget: timing.budget(),
        missed_refreshes: missed,
        refresh_interval: timing.refresh_interval,
        invalidations: timing.invalidations,
        phases: timing.phases,
    }
}

/// 有序样本 → 百分位（最近秩法，空集回零）。
fn percentile(sorted: &[Duration], p: f32) -> Duration {
    if sorted.is_empty() {
        return Duration::ZERO;
    }
    let idx = (((p / 100.0) * sorted.len() as f32).ceil() as usize)
        .saturating_sub(1)
        .min(sorted.len() - 1);
    sorted[idx]
}

/// 排序辅助：升序 Duration 序列。
fn sorted_asc(values: &[Duration]) -> Vec<Duration> {
    let mut v = values.to_vec();
    v.sort_unstable();
    v
}

/// 滚动腿靶点（面板内容区中心）：与 open_panel_docked 同源放置链——
/// trigger_pos + screen_rect + main_panel_size + ui dock::panel_bounds 求面板
/// 屏幕矩形（Quartz top-left 同源坐标系）→ 中心点。纯几何，不读窗柄。
fn scroll_target(cx: &App) -> Option<(f32, f32)> {
    let pos = super::trigger_pos(cx);
    let (x, y, w, h) =
        parchora_ui::dock::panel_bounds(pos, super::screen_rect(cx), super::main_panel_size(cx));
    Some((x + w / 2.0, y + h / 2.0))
}

/// 全局点（Quartz top-left）→ 面板窗局部点（gpui top-left 口径）：
/// dispatch_event 的 position 须为本窗局部坐标——与 open_panel_docked 装配
/// 同源换算（to_panel_screen_local 语义：全局减探针屏原点 + 窗位差）。面板
/// 窗缘以同源放置链反推（几何与实际窗位一致 · 面板静止期成立；拖动中的窗
/// 不在场景域）。
fn to_panel_local(cx: &App, sx: f32, sy: f32) -> gpui::Point<gpui::Pixels> {
    let (px_, py_, _, _) = parchora_ui::dock::panel_bounds(
        super::trigger_pos(cx),
        super::screen_rect(cx),
        super::main_panel_size(cx),
    );
    gpui::point(gpui::px(sx - px_), gpui::px(sy - py_))
}

/// 聚合报告（stderr 人类可读 + 末行机器可读 JSON）；模式随行标注（G2 passive
/// 空闲判定：帧数=0 即窗内零自然帧）。
fn report(events: &[FrameEvent], ticks: u32, passive: bool, scenario: &str) {
    let (samples, unpaired) = pair_events(events);
    let draws: Vec<Duration> = samples.iter().map(|s| s.draw).collect();
    let works: Vec<Duration> = samples.iter().filter_map(|s| s.work).collect();
    let mut on_time: u64 = 0;
    let mut late: u64 = 0;
    let mut missed_total: u64 = 0;
    let mut phases = (Duration::ZERO, Duration::ZERO, Duration::ZERO);
    let mut invalidations_total: u64 = 0;
    let mut budgets = std::collections::BTreeSet::new();
    let mut refresh_intervals = std::collections::BTreeSet::new();
    let mut per_window: HashMap<u64, u64> = HashMap::new();
    for s in &samples {
        invalidations_total += s.invalidations;
        *per_window.entry(s.window_id).or_default() += 1;
        if let Some(work) = s.work {
            if work > s.budget {
                late += 1;
            } else {
                on_time += 1;
            }
            missed_total += s.missed_refreshes.unwrap_or(0);
        }
        budgets.insert(s.budget);
        if let Some(i) = s.refresh_interval {
            refresh_intervals.insert(i);
        }
        phases.0 += s.phases.request_layout;
        phases.1 += s.phases.prepaint;
        phases.2 += s.phases.paint;
    }
    let d = sorted_asc(&draws);
    let w = sorted_asc(&works);
    let p = |v: &[Duration], q: f32| percentile(v, q).as_secs_f64() * 1000.0;
    eprintln!(
        "{} 模式={}{} 采样窗={} 驱动拍={} 帧数={}（Draw={} · Present={} · 未配对Draw={}）· 失效合计={}",
        TELEMETRY_TAG,
        if passive { "passive(仅观测)" } else { "driven" },
        if scenario.is_empty() {
            String::new()
        } else {
            format!("+scenario={scenario}")
        },
        per_window.len(),
        ticks,
        samples.len(),
        samples.len(),
        samples.len() - unpaired as usize,
        unpaired,
        invalidations_total
    );
    eprintln!(
        "{} draw P50={:.2} P90={:.2} P95={:.2} P99={:.2} max={:.2} ms（全窗合并 n={}; 相={:.2}/{:.2}/{:.2} ms）",
        TELEMETRY_TAG,
        p(&d, 50.0),
        p(&d, 90.0),
        p(&d, 95.0),
        p(&d, 99.0),
        d.last().map(|v| v.as_secs_f64() * 1000.0).unwrap_or(0.0),
        d.len(),
        phases.0.as_secs_f64() * 1_000.0,
        phases.1.as_secs_f64() * 1_000.0,
        phases.2.as_secs_f64() * 1_000.0,
    );
    eprintln!(
        "{} work P50={:.2} P90={:.2} P95={:.2} P99={:.2} ms（可测帧 n={} / {}）· \
         on-time={} late={}（budget={:?}）· missed_refreshes={}",
        TELEMETRY_TAG,
        p(&w, 50.0),
        p(&w, 90.0),
        p(&w, 95.0),
        p(&w, 99.0),
        w.len(),
        samples.len(),
        on_time,
        late,
        budgets.iter().next(),
        missed_total
    );
    // 机器可读汇总（单行 JSON · 基线固档用）。
    let budget_ms = budgets
        .iter()
        .next()
        .map(|b| b.as_secs_f64() * 1000.0)
        .unwrap_or(0.0);
    eprintln!(
        "{} {{\"mode\":\"{}\",\"scenario\":\"{}\",\"frames\":{},\"draw_p50_ms\":{:.3},\"draw_p95_ms\":{:.3},\"draw_p99_ms\":{:.3},\"work_p50_ms\":{:.3},\"work_p95_ms\":{:.3},\"work_p99_ms\":{:.3},\"on_time\":{},\"late\":{},\"missed_refreshes\":{},\"refresh_interval_ms\":{:.3},\"budget_ms\":{:.3}}}",
        TELEMETRY_TAG,
        if passive { "passive" } else { "driven" },
        scenario,
        samples.len(),
        p(&d, 50.0),
        p(&d, 95.0),
        p(&d, 99.0),
        p(&w, 50.0),
        p(&w, 95.0),
        p(&w, 99.0),
        on_time,
        late,
        missed_total,
        refresh_intervals
            .iter()
            .next()
            .map(|i| i.as_secs_f64() * 1000.0)
            .unwrap_or(0.0),
        budget_ms,
    );
}
