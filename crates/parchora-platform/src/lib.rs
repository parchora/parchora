//! parchora-platform —— 平台桥接（unsafe 集中营 · design.md §2.1 M6）。
//!
//! 边界：objc2 系依赖独占（NSPanel/NSStatusItem/NSPasteboard/CGEventTap/Carbon
//! 热键）；不做视图。全部桥接为纯 C FFI（零第三方依赖 · 规范 9）。
//!
//! ## cursor 模块（唤出感知承载 · DEC-79）
//!
//! `cursor_position()` 以 CGEvent 只读 API 取光标全局坐标（top-left 原点 · y 向下 ·
//! 逻辑点坐标，与本仓 dock 几何坐标系同源）。选型依据：gpui-fast 窗口级 hover 对
//! 「非 key + 非激活 NSPanel」收不到 mouseMoved（events.rs 不映射 NSMouseEntered、
//! PopUp 分支不设 setAcceptsMouseMovedEvents——真机实证：key 面板 hover 通、
//! 非 key 热区死），窗口 hover 机制承载不了收缩态唤出；CGEventTap 全局监听
//! （REQ-PANEL-EDGE-TRIGGER 真语义）见 listen/tap 模块（DEC-81 · 需辅助功能
//! 权限）；CGEvent 位置读探针无权限、无注入、纯查询，是未授权期的 poll 降级
//! 承载（真机实证：CGEvent(source: nil).location 读回与 warp 一致）。
//! 多显示器真原点 defer DEC-76 口径不变（单屏机完全正确）。
//!
//! ## listen / display / tap 模块（DEC-81 · 真语义光标源与授权探测）
//!
//! - `listen_access_granted()` / `request_listen_access()`：输入监控授权显式探测
//!   （REQ-PANEL-LISTEN-GRACEFUL）。preflight 纯只读不弹系统提示（启动/守护拍
//!   安全）；request 可能弹 TCC 系统弹窗，**仅限设置引导卡点击路径调用，启动
//!   禁止自动调用**（防 TCC 弹窗劫持首启）。
//! - `display_bounds_list()`：全部活动显示器 bounds（Quartz top-left 真原点 ·
//!   y 向下），供内缘判定消费真几何——与 vendor PlatformDisplay::bounds origin
//!   恒 (0,0) 的差异即为 DEC-76 副屏真原点 defer 的收口载体。
//! - `spawn_mouse_tap()`：CGEventTap **只听监听**（ListenOnly · mouseMoved + 左右
//!   拖拽位掩码），真语义光标源；仅线程装配失败返回 None——tap 创建失败（含未
//!   授权 CGEventTapCreate 返 NULL）时句柄非 None 但线程即刻退出且永不回调，
//!   降级以 app 层 preflight 复探 + 槽新鲜度判定承载（注释按实现口径 · DEC-79/81/82）。
//!   句柄 Drop = 停 runloop → join；tap 线程收尾顺序：先 CFMachPortInvalidate
//!   再回收 handler Box（防晚到回调解引用悬垂）；kCGEventTapDisabledByTimeout
//!   防御性 re-enable。坐标系与 cursor_position 同源（CGEventGetLocation）。
//!
//! ## screens 模块（DEC-84 · 可用区域边缘语义）
//!
//! `screen_areas()`：逐屏返回全帧（NSScreen.frame）与可用区域（NSScreen.visibleFrame
//! ——排除菜单栏/Dock 占用后的可用区域，REQ-PANEL-EDGE-TRIGGER「可用区域边缘」
//! 定义源）双 rect（Quartz top-left 原点 · y 向下 · 与 cursor_position/display_bounds
//! 同源坐标系）。桥接为纯 C FFI（libobjc objc_msgSend 系统库 · 零第三方 crate ·
//! 规范 9）；macOS 27 为 ARM64——objc_msgSend 直接承载结构返回（x8 sret 寄存器转发
//! ·objc_msgSend_stret 于 arm64 不适用），x86_64 不适用（登记局限）。生产调用点均在
//! gpui 主线程（AppKit 主线程惯例）；NSScreen.frame 为逻辑点，与 CGDisplayBounds
//! 同尺寸同位（换算后逐屏比对可复核）。
//!
//! ## status-item 模块（状态栏图标 + 点击交互）
//!
//! `create_status_item()`：NSStatusItem 常驻图标（variable length · NSStatusBar
//! systemStatusBar 主线程装配），NSApp activation policy = Accessory（无 Dock 图标
//! ——AGENTS.md 技术栈表 LSUIElement 语义 · REQ-SETTINGS-STATUSBAR「菜单栏常驻」
//! 演示宿主承载，5 项内容菜单 defer 产品壳轮）；图标 = SF Symbol（macOS 11+ ·
//! respondsToSelector 门槛防未识别 selector 崩溃）→ title 字形 fallback（10.15 级）。
//! `status_click_take()`：点击旗标排水——click IMP（动态构造目标类 · class_addMethod
//! types "v@:@"）仅置位 AtomicBool，app 层 0.05s 守护排水 toggle 主面板（演示
//! 窗形态仅登记）。目标对象裸指针静态持有（setTarget: 弱引用不 retain，进程
//! 生命周期保活防悬垂）。
//!
//! ## child-window 模块（拖动跟随）
//!
//! `attach_child_window(parent_rect, child_rect)`：面板↔Peek AppKit 子窗挂接
//! ——枚举 `[NSApp windows]`（本进程窗口，含 NSPanel；无 CGWindowList 权限面），
//! NS 底左原点 frame 换算 Quartz top-left（screen_areas 同源式），按实界 bounds
//! 匹配（分量容差 2pt）定位父子窗后 `[parent addChildWindow:child ordered:1]`
//! （NSWindowAbove）。AppKit 原生保持子窗相对偏移与 z 序：拖动主面板全程零抖
//! 跟随；Peek 关窗自动解除关联。失败返回 false（调用方独立窗降级，不阻断）。
//!
//! ## app-icon 模块（设置应用图标=品牌 logo）
//!
//! `create_app_icon(png)`：内嵌 PNG（tools/render_logo.swift 经 AppKit/CoreSVG
//! 直读 SVG 离线渲染产物 · include_bytes 零运行时文件依赖）→ NSData
//! dataWithBytes:length:（拷贝字节）→ NSImage initWithData: → NSApp
//! setApplicationIconImage:（主线程专用 · 同 create_status_item 风格：任一环节
//! 失败 stderr 落档不阻断启动）。Accessory 激活策略下无 Dock
//! 图标，图标可见面 = 通知横幅/NSAlert 等程序化场景；NSImage 对象 +1 引用刻意
//! 不释放（单次装配持有于 NSApp · 同 STATUS_TARGET 保活先例）。

use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicU8, AtomicUsize, Ordering};
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Instant;

use parchora_core::classify::{PasteboardKind, PasteboardTypeSnapshot};

/// 光标全局坐标（主屏 top-left 原点 · y 向下 · 逻辑点）。读失败/无事件源 → None。
/// 纯 C FFI（CoreGraphics 已在链接图中，零新增依赖 · 规范 9）；CGEventCreate(nil)
/// + CGEventGetLocation 属只读查询，非事件注入（与 CGEventPost 不同，不经 TCC）。
pub fn cursor_position() -> Option<(f32, f32)> {
    #[repr(C)]
    struct CGPoint {
        x: f64,
        y: f64,
    }
    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        fn CGEventCreate(source: *const std::ffi::c_void) -> *mut std::ffi::c_void;
        fn CGEventGetLocation(event: *const std::ffi::c_void) -> CGPoint;
        fn CFRelease(cf: *mut std::ffi::c_void);
    }
    unsafe {
        let event = CGEventCreate(std::ptr::null());
        if event.is_null() {
            return None;
        }
        let point = CGEventGetLocation(event);
        CFRelease(event);
        Some((point.x as f32, point.y as f32))
    }
}

/// 输入监控授权预检（只读 · 不弹系统提示 · REQ-PANEL-LISTEN-GRACEFUL 启动探测）。
/// 启动/守护拍均可安全调用（CGPreflightListenEventAccess 纯状态查询）。
pub fn listen_access_granted() -> bool {
    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        // 返回型 Apple `Boolean`(unsigned char)：与 Rust bool 同 1 字节 ABI 且 Apple
        // 恒返回 0/1，bool 映射安全（同口径三处：preflight / request / tap enable）。
        fn CGPreflightListenEventAccess() -> bool;
    }
    unsafe { CGPreflightListenEventAccess() }
}

/// 请求输入监控授权（可能弹 TCC 系统弹窗 · REQ-PANEL-LISTEN-GRACEFUL 设置页引导
/// 入口）。**仅限设置引导卡点击路径调用；启动禁止自动调用**（防 TCC 弹窗劫持）。
/// 返回调用时点的授权态（弹窗未决 = false；用户在系统设置授权后由 preflight 复探）。
pub fn request_listen_access() -> bool {
    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        fn CGRequestListenEventAccess() -> bool; // Boolean 映射安全（见 preflight 注）
    }
    unsafe { CGRequestListenEventAccess() }
}

/// 全部活动显示器的 bounds（Quartz top-left 真原点 · y 向下 · 逻辑点 ·
/// (x, y, w, h) 四元组）。内缘判定（REQ-PANEL-EDGE-TRIGGER
/// SCN-PANEL-INTERIOR-SEAM）的真几何源；最多取 8 屏（截断登记口径，桌面场景
/// 远够）。调用失败/无显示器 → 空表（调用方按全外缘处置）。
pub fn display_bounds_list() -> Vec<(f32, f32, f32, f32)> {
    #[repr(C)]
    struct CGPoint {
        x: f64,
        y: f64,
    }
    #[repr(C)]
    struct CGSize {
        width: f64,
        height: f64,
    }
    #[repr(C)]
    struct CGRect {
        origin: CGPoint,
        size: CGSize,
    }
    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        fn CGGetActiveDisplayList(
            max_displays: u32,
            active_displays: *mut u32,
            display_count: *mut u32,
        ) -> i32;
        fn CGDisplayBounds(display: u32) -> CGRect;
    }
    let mut ids = [0u32; 8];
    let mut count: u32 = 0;
    unsafe {
        if CGGetActiveDisplayList(8, ids.as_mut_ptr(), &mut count) != 0 {
            return Vec::new();
        }
        ids[..count.min(8) as usize]
            .iter()
            .map(|&id| {
                let b = CGDisplayBounds(id);
                (
                    b.origin.x as f32,
                    b.origin.y as f32,
                    b.size.width as f32,
                    b.size.height as f32,
                )
            })
            .collect()
    }
}

// ───── CGEventTap 只听监听（DEC-81 · REQ-PANEL-EDGE-TRIGGER 真语义光标源）─────

/// 事件类型位（CGEventType 枚举值 · kCGEventLeftMouseDown=1 / kCGEventLeftMouseUp=2 /
/// kCGEventMouseMoved=5 / 左拖=6 / 右拖=7 —— NX_* 映射经 CGEventTypes.h，
/// IOLLEvent.h 数值定义查证）。
const EVENT_LEFT_MOUSE_DOWN: u32 = 1;
const EVENT_LEFT_MOUSE_UP: u32 = 2;
const EVENT_MOUSE_MOVED: u32 = 5;
const EVENT_LEFT_MOUSE_DRAGGED: u32 = 6;
const EVENT_RIGHT_MOUSE_DRAGGED: u32 = 7;
/// 只听位掩码：左键按下（点击面板外隐藏路径 · DEC-94）+ 左键释放（拖动释放即
/// 吸附信号源 · DEC-210）+ 鼠标移动 + 左右拖拽（REQ-PANEL-EDGE-TRIGGER「全局
/// 鼠标移动事件」）。
const MOUSE_EVENT_MASK: u32 = (1 << EVENT_LEFT_MOUSE_DOWN)
    | (1 << EVENT_LEFT_MOUSE_UP)
    | (1 << EVENT_MOUSE_MOVED)
    | (1 << EVENT_LEFT_MOUSE_DRAGGED)
    | (1 << EVENT_RIGHT_MOUSE_DRAGGED);
/// CGEventTap 位点常量：会话级 tap / 头插入 / 只听（不拦截不修改）。
const KCG_SESSION_EVENT_TAP: u32 = 1;
const KCG_HEAD_INSERT_EVENT_TAP: u32 = 0;
const KCG_EVENT_TAP_OPTION_LISTEN_ONLY: u32 = 1;
/// 系统超时禁用/用户禁用事件类型（防御性 re-enable 场景）。
const KCG_EVENT_TAP_DISABLED_BY_TIMEOUT: u32 = 0xFFFF_FFFE;
const KCG_EVENT_TAP_DISABLED_BY_USER_INPUT: u32 = 0xFFFF_FFFF;

#[repr(C)]
struct CGPoint {
    x: f64,
    y: f64,
}

type CGEventTapCallbackFn =
    Option<unsafe extern "C" fn(*mut c_void, u32, *mut c_void, *mut c_void) -> *mut c_void>;

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGEventTapCreate(
        tap: u32,
        place: u32,
        options: u32,
        events_of_interest: u64, // CGEventMask 实型 uint64_t（头文件口径 · 原型对齐）
        callback: CGEventTapCallbackFn,
        user_info: *mut c_void,
    ) -> *mut c_void;
    fn CGEventTapEnable(tap: *mut c_void, enable: bool); // enable：Boolean 映射安全（见 preflight 注）
    fn CFMachPortInvalidate(port: *mut c_void);
    fn CGEventGetLocation(event: *const c_void) -> CGPoint;
}

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFMachPortCreateRunLoopSource(
        allocator: *const c_void,
        port: *mut c_void,
        order: i64,
    ) -> *mut c_void;
    fn CFRunLoopGetCurrent() -> *mut c_void;
    fn CFRunLoopAddSource(runloop: *mut c_void, source: *mut c_void, mode: *mut c_void);
    fn CFRunLoopSourceInvalidate(source: *mut c_void);
    fn CFRunLoopRun();
    fn CFRunLoopStop(runloop: *mut c_void);
    fn CFRelease(cf: *mut c_void);
    static kCFRunLoopDefaultMode: *mut c_void;
}

/// tap 回调共享态：handler（互斥包 Box 防回调重入）+ tap 端口（防御性 re-enable）
/// + 收尾旗标（句柄 Drop 与 tap 线程装载竞态关断 · R184 F-183-05）。
struct TapState {
    handler: Mutex<Box<dyn FnMut(f32, f32) + Send>>,
    /// CFMachPortRef 以 usize 存储（raw 指针跨线程不可 Send/Sync）；0 = 未创建完。
    tap_port: AtomicUsize,
    /// 收尾请求（sticky 一次性）：句柄 Drop 先置位——Drop 落在「tap 已建、runloop
    /// 槽未写」竞态窗口时，装载检查据此关断，CFRunLoopRun 不再起跑（槽无值
    /// 路径的 Stop 缺口收口）。
    shutdown: Arc<AtomicBool>,
}

/// 首事件一次性诊断闩（真机判读 tap 是否真收到事件 · DEC-81）。
static TAP_FIRST_EVENT: AtomicBool = AtomicBool::new(false);

/// 左键按下旗标（tap 置位 · app 层排水消费 · 点击面板外隐藏路径 DEC-94）：tap 内
/// 仅登记「会话内发生过左键按下」，点击在保持域内外由 app 层按当下光标坐标判定
/// （保持域 = 面板 ∪ 详情窗 ∪ 邻接间隙条 ∪ 触发热区 ∪ 设置窗）。
static TAP_CLICK: AtomicBool = AtomicBool::new(false);

/// 左键释放旗标（tap 置位 · app 层排水消费 · 拖动释放即吸附 DEC-210）：拖曳窗口
/// 移动结束的**键态真信号**（替代「连续两拍读数一致」的释放近似——慢拖长停顿
/// /暂停不再有单拍误判面）。锁存无时间语义（不过期），由 app 层 0.05s 守护拍
/// 排水；tap 未运行（未授权/演示宿主）时恒 false——测试与未授权降级路径零污染。
static TAP_MOUSE_UP: AtomicBool = AtomicBool::new(false);

/// 鼠标只听 tap 回调（runloop 线程执行 ListenOnly · 系统转发事件流）：鼠标移动/拖拽
/// → handler(全局坐标)；左键按下/释放 → 锁存登记（不调 handler）；禁用类事件 →
/// 防御性 re-enable；其余类型忽略。userinfo 指向 Box::into_raw的 TapState，生存期由
/// tap 收尾纪律保证（invalidate 先于回收）。
unsafe extern "C" fn tap_callback(
    _proxy: *mut c_void,
    type_: u32,
    event: *mut c_void,
    user_info: *mut c_void,
) -> *mut c_void {
    if type_ == KCG_EVENT_TAP_DISABLED_BY_TIMEOUT || type_ == KCG_EVENT_TAP_DISABLED_BY_USER_INPUT {
        if let Some(state) = (user_info as *const TapState).as_ref() {
            let port = state.tap_port.load(Ordering::Acquire);
            if port != 0 {
                CGEventTapEnable(port as *mut c_void, true);
            }
        }
        return event;
    }
    if matches!(
        type_,
        EVENT_LEFT_MOUSE_DOWN
            | EVENT_MOUSE_MOVED
            | EVENT_LEFT_MOUSE_DRAGGED
            | EVENT_RIGHT_MOUSE_DRAGGED
    ) {
        if let Some(state) = (user_info as *const TapState).as_ref() {
            // try_lock：回调用锁竞争时跳过本拍（下一事件即修正，丢失无害）。
            if let Ok(mut handler) = state.handler.try_lock() {
                let point = CGEventGetLocation(event);
                if !TAP_FIRST_EVENT.swap(true, Ordering::Relaxed) {
                    eprintln!("[parchora] tap 收到首个鼠标事件（真语义光标源激活 · DEC-81）");
                }
                handler(point.x as f32, point.y as f32);
            }
        }
    }
    if type_ == EVENT_LEFT_MOUSE_DOWN {
        // 左键按下登记（app 层 click-outside 排水消费 · DEC-94；内存序同
        // STATUS_CLICK：Release 写 / AcqRel 排水）。
        TAP_CLICK.store(true, Ordering::Release);
    }
    if type_ == EVENT_LEFT_MOUSE_UP {
        // 左键释放登记（app 层释放即吸附守护排水消费 · DEC-210；内存序同上）。
        TAP_MOUSE_UP.store(true, Ordering::Release);
    }
    event
}

/// tap 线程主（创建 tap → 装 runloop 源 → 运行 → 收尾）。收尾纪律（DEC-81 ·
/// 防悬垂）：CFRunLoopSourceInvalidate → CFMachPortInvalidate（此后不再有任何
/// 回调进入本线程）→ release → 回收 handler Box。创建失败路径同样回收。
unsafe fn tap_thread_main(state: *mut TapState, slot: Arc<Mutex<Option<usize>>>) {
    eprintln!("[parchora] tap 线程启动（CGEventTapCreate · DEC-81 诊断）");
    let tap = CGEventTapCreate(
        KCG_SESSION_EVENT_TAP,
        KCG_HEAD_INSERT_EVENT_TAP,
        KCG_EVENT_TAP_OPTION_LISTEN_ONLY,
        MOUSE_EVENT_MASK as u64,
        Some(tap_callback),
        state as *mut c_void,
    );
    if tap.is_null() {
        // 未授权（TCC）/创建失败：回收 handler，无 tap 生命周期（调用方收 None）。
        eprintln!("[parchora] CGEventTapCreate 返 NULL（未授权/策略拒绝 · DEC-81 诊断）");
        drop(Box::from_raw(state));
        return;
    }
    (*state).tap_port.store(tap as usize, Ordering::Release);
    let source = CFMachPortCreateRunLoopSource(std::ptr::null(), tap, 0);
    if source.is_null() {
        eprintln!("[parchora] CFMachPortCreateRunLoopSource 返 NULL（DEC-81 诊断）");
        CFMachPortInvalidate(tap);
        CFRelease(tap);
        drop(Box::from_raw(state));
        return;
    }
    let runloop = CFRunLoopGetCurrent();
    CFRunLoopAddSource(runloop, source, kCFRunLoopDefaultMode);
    // Drop 竞态关断检查（R184 F-183-05 · sticky）：句柄 Drop 先置 shutdown——
    // 落在槽写窗口竞态时此处命中即 Stop（CFRunLoopStop 先于 Run 标记 sticky，
    // Run 立即返回），线程照走下方收尾链回收。
    if (*state).shutdown.load(Ordering::SeqCst) {
        CFRunLoopStop(runloop);
    }
    if let Ok(mut s) = slot.lock() {
        *s = Some(runloop as usize);
    }
    CFRunLoopRun(); // 阻塞至 CFRunLoopStop（句柄 Drop 停 tap）或源失效
    eprintln!("[parchora] tap runloop 退出（CFRunLoopRun 返回 · DEC-81 诊断）");
    // 收尾：先摘槽（Drop 端不再重复 Stop），invalidate 链，最后回收 handler。
    if let Ok(mut s) = slot.lock() {
        *s = None;
    }
    CFRunLoopSourceInvalidate(source);
    CFMachPortInvalidate(tap);
    CFRelease(source);
    CFRelease(tap);
    drop(Box::from_raw(state));
}

/// Send 包装（*mut TapState 跨线程移交 tap 线程独占消费：创建失败即回收 /
/// 收尾回收；回调侧经 userinfo 裸指针访问，生存期由「invalidate 先于回收」
/// 纪律保证 · DEC-81）。仅在 spawn_mouse_tap 内构造一次。
#[derive(Clone, Copy)]
struct SendStatePtr(*mut TapState);
unsafe impl Send for SendStatePtr {}

impl SendStatePtr {
    /// 方法接收者按整体捕获（阻断 edition 2021 闭包字段级捕获绕过 Send 包装）。
    fn into_raw(self) -> *mut TapState {
        self.0
    }
}

/// 鼠标只听 tap 句柄（Drop = 停 runloop → join 回收线程）。进程生命周期持有；
/// 单测试可显式 drop 验证回收。Send + Sync（线程句柄与槽均为同步原语）。
pub struct MouseTapHandle {
    thread: Option<std::thread::JoinHandle<()>>,
    runloop: Arc<Mutex<Option<usize>>>,
    /// 收尾旗标共享（TapState.shutdown 同 Arc）：Drop 先置位再查槽——装载竞态
    /// 窗口（tap 已建、槽未写）不漏关（R184 F-183-05）。
    shutdown: Arc<AtomicBool>,
}

impl Drop for MouseTapHandle {
    fn drop(&mut self) {
        // 收尾旗标先行（sticky 一次性）：竞态窗口（tap 已建、runloop 槽未写）由
        // tap 线程装载检查消费；已运行路径此旗标无害（下方槽 Stop 照常）。
        self.shutdown.store(true, Ordering::SeqCst);
        // 槽锁中毒（临界区无 panic 点 · 理论路径）时跳过 Stop——join 等线程自身收尾。
        let runloop_ref = self.runloop.lock().ok().and_then(|s| *s);
        if let Some(runloop_ref) = runloop_ref {
            unsafe { CFRunLoopStop(runloop_ref as *mut c_void) };
        }
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

/// 装配 CGEventTap 只听监听（DEC-81 · REQ-PANEL-EDGE-TRIGGER 真语义光标源）：
/// 每个鼠标移动/拖拽事件回调 `handler(全局坐标)`（top-left 原点 · y 向下 · 与
/// `cursor_position` 同源坐标系）。
///
/// 返回 None = **线程装配失败**；tap 未授权 / CGEventTapCreate 返 NULL 时返回
/// Some 但 tap 线程即刻退出且永不回调（创建失败路径回收 handler）——授权态与
/// 光标活性由调用方 preflight 复探 + 槽新鲜度判定承载（REQ-PANEL-LISTEN-GRACEFUL
/// 降级语义 · app 层 DEC-82）；**不自动弹系统
/// 授权提示**（授权请求仅经设置引导卡点击路径）。
pub fn spawn_mouse_tap(
    handler: Box<dyn FnMut(f32, f32) + Send + 'static>,
) -> Option<MouseTapHandle> {
    let shutdown = Arc::new(AtomicBool::new(false));
    let state = SendStatePtr(Box::into_raw(Box::new(TapState {
        handler: Mutex::new(handler),
        tap_port: AtomicUsize::new(0),
        shutdown: Arc::clone(&shutdown),
    })));
    let runloop_slot = Arc::new(Mutex::new(Option::<usize>::None));
    let slot_for_thread = Arc::clone(&runloop_slot);
    let thread = match std::thread::Builder::new()
        .name("parchora-mouse-tap".into())
        .spawn(move || unsafe { tap_thread_main(state.into_raw(), slot_for_thread) })
    {
        Ok(thread) => thread,
        Err(_) => {
            // 线程创建失败：回收 state（tap 尚未创建，无并发风险）。
            unsafe { drop(Box::from_raw(state.into_raw())) };
            return None;
        }
    };
    Some(MouseTapHandle {
        thread: Some(thread),
        runloop: runloop_slot,
        shutdown,
    })
}

/// 左键按下旗标排水（DEC-94 · REQ-PANEL-HIDE「点击面板外部隐藏」信号源）：消耗式
/// 读一次置位（swap(false) 语义同 `status_click_take`）；tap 未运行（未授权/演示
/// 宿主）时恒 false——测试与未授权降级路径零污染。点击内外域判定由 app 层承载。
pub fn tap_click_take() -> bool {
    TAP_CLICK.swap(false, Ordering::AcqRel)
}

/// 左键释放旗标排水（DEC-210 · 拖动释放即吸附信号源）：消耗式读一次置位
///（swap(false) 语义同 `tap_click_take`）；tap 未运行（未授权/演示宿主）时恒
/// false——未授权降级路径释放吸附由 app 层 0.25s 吸附守护兜底（DEC-78 口径）。
pub fn tap_mouse_up_take() -> bool {
    TAP_MOUSE_UP.swap(false, Ordering::AcqRel)
}

// ───── NSScreen 屏帧桥接（DEC-84 · 可用区域边缘语义）─────

/// 单屏几何：全帧（NSScreen.frame）与可用区域（NSScreen.visibleFrame——排除
/// 菜单栏/Dock 占用 · REQ-PANEL-EDGE-TRIGGER「可用区域」）双 rect。Quartz
/// top-left 原点 · y 向下 · 逻辑点 (x, y, w, h)，与 cursor_position/同仓 display
/// 模块坐标系同源（app 层停靠几何可直接消费）。
/// `id` = deviceDescription NSScreenNumber（CGDirectDisplayID，解析失败哨兵 0
/// ——真实显示 id 恒非 0，app 侧身份匹配天然不命中即安全回退）：与 gpui
/// `DisplayId`（MacDisplay::id 经 `From<DisplayId> for u64` 展开同值）精确配对,
/// 支撑 app 层窗全局坐标缝（DEC-99）按显示身份查回本屏全局原点。
pub struct ScreenArea {
    pub id: u32,
    pub full: (f32, f32, f32, f32),
    pub usable: (f32, f32, f32, f32),
}

/// ObjC 桥接用 CGRect 同构（NSScreen.frame/visibleFrame 返回 · NS 坐标 · 底左
/// 原点 y 向上）。布局 = {{x,y},{w,h}} 四连续 f64，平铺字段等价（同构复用）。
#[repr(C)]
struct NsRect {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

// libobjc 系统库直接桥接（零第三方 crate · 规范 9）：objc_msgSend 按「同符号
// 多签名声明」（Apple 文档认可的消息发送变通；ARM64 结构返回由 x8 寄存器承载
// · objc_msgSend_stret 不适用，x86_64 不在本仓目标域 · 登记局限）。 rustc
// clashing_extern_declarations 系对「同符号不同签名」的保守提示，按上述文档
// 语义放行。仅本节消费，不对外暴露。
#[allow(clashing_extern_declarations)]
#[link(name = "objc")]
extern "C" {
    fn objc_getClass(name: *const std::ffi::c_char) -> *mut c_void;
    fn sel_registerName(name: *const std::ffi::c_char) -> *mut c_void;
    #[link_name = "objc_msgSend"]
    fn msg_send_id(receiver: *mut c_void, sel: *mut c_void) -> *mut c_void;
    #[link_name = "objc_msgSend"]
    fn msg_send_usize(receiver: *mut c_void, sel: *mut c_void) -> usize;
    #[link_name = "objc_msgSend"]
    fn msg_send_id_idx(receiver: *mut c_void, sel: *mut c_void, idx: usize) -> *mut c_void;
    #[link_name = "objc_msgSend"]
    fn msg_send_rect(receiver: *mut c_void, sel: *mut c_void) -> NsRect;
    // receiver+sel+id 三参变体（objectForKey: / stringWithUTF8String: ·
    // 同符号多签名声明同语义放行 · 仅 screen_areas 消费）。
    #[link_name = "objc_msgSend"]
    fn msg_send_id_obj(receiver: *mut c_void, sel: *mut c_void, arg: *mut c_void) -> *mut c_void;
    // addChildWindow:ordered: 双参 void 变体（子窗挂接；同符号多签名
    // 声明同语义放行 · 仅 attach_child_window 消费）。
    #[link_name = "objc_msgSend"]
    fn msg_send_void_id_isize(
        receiver: *mut c_void,
        sel: *mut c_void,
        arg1: *mut c_void,
        arg2: isize,
    );
    // respondsToSelector: BOOL 返回 + 单 id 参变体（承诺板本机化 sel 防御；
    // 同符号多签名声明同语义放行 · 仅承诺桥消费）。
    #[link_name = "objc_msgSend"]
    fn msg_send_bool_arg(receiver: *mut c_void, sel: *mut c_void, arg1: *mut c_void) -> bool;
    // prepareForNewContents: usize 参 usize 返回变体（返回新 changeCount 基
    // 线；同符号多签名声明同语义放行 · 仅承诺桥消费）。
    #[link_name = "objc_msgSend"]
    fn msg_send_usize_arg(receiver: *mut c_void, sel: *mut c_void, arg1: usize) -> usize;
}

/// 逐屏全帧 + 可用区域 + 显示身份（DEC-84 · DEC-99）：
/// `[[NSScreen screens] objectAtIndex:i]` 取 frame/visibleFrame 双 rect 与
/// deviceDescription NSScreenNumber（CGDirectDisplayID，失败哨兵 0），经主屏
/// 锚定换算到 Quartz top-left 坐标（NS 全局坐标底左原点 y 向上 → cg_y =
/// primary_h − (ns_y + ns_h)；主屏 = NS 原点 (0,0) 屏，兜底取首屏）。类查找失败
/// / 数组为空 / 无显示器 → 空表（调用方按「全帧当可用」兜底。
/// NSScreenNumber 仅作身份配对，不参与坐标换算）。
/// 生产调用点均在 gpui 主线程（AppKit 主线程惯例）。
pub fn screen_areas() -> Vec<ScreenArea> {
    unsafe {
        let cls = objc_getClass(c"NSScreen".as_ptr());
        if cls.is_null() {
            return Vec::new();
        }
        let arr = msg_send_id(cls, sel_registerName(c"screens".as_ptr()));
        if arr.is_null() {
            return Vec::new();
        }
        let count = msg_send_usize(arr, sel_registerName(c"count".as_ptr()));
        let sel_at = sel_registerName(c"objectAtIndex:".as_ptr());
        let sel_frame = sel_registerName(c"frame".as_ptr());
        let sel_visible = sel_registerName(c"visibleFrame".as_ptr());
        let sel_device = sel_registerName(c"deviceDescription".as_ptr());
        let sel_key = sel_registerName(c"objectForKey:".as_ptr());
        let sel_uint = sel_registerName(c"unsignedIntegerValue".as_ptr());
        // NSScreenNumber 常量键（stringWithUTF8String:，主线程 autorelease 池
        // 托管；NSString 类缺失 → 哨兵空键 id 恒 0）。
        let str_cls = objc_getClass(c"NSString".as_ptr());
        let num_key = if str_cls.is_null() {
            std::ptr::null_mut()
        } else {
            msg_send_id_obj(
                str_cls,
                sel_registerName(c"stringWithUTF8String:".as_ptr()),
                c"NSScreenNumber".as_ptr() as *mut c_void,
            )
        };
        let mut frames: Vec<(NsRect, NsRect, u32)> = Vec::new();
        for i in 0..count {
            let screen = msg_send_id_idx(arr, sel_at, i);
            if screen.is_null() {
                continue;
            }
            let id = if num_key.is_null() {
                0
            } else {
                let desc = msg_send_id(screen, sel_device);
                if desc.is_null() {
                    0
                } else {
                    let num = msg_send_id_obj(desc, sel_key, num_key);
                    if num.is_null() {
                        0
                    } else {
                        msg_send_usize(num, sel_uint) as u32
                    }
                }
            };
            frames.push((
                msg_send_rect(screen, sel_frame),
                msg_send_rect(screen, sel_visible),
                id,
            ));
        }
        if frames.is_empty() {
            return Vec::new();
        }
        let primary_h = frames
            .iter()
            .find(|(f, _, _)| f.x == 0.0 && f.y == 0.0)
            .map(|(f, _, _)| f.h)
            .unwrap_or(frames[0].0.h);
        frames
            .iter()
            .map(|(f, v, id)| ScreenArea {
                id: *id,
                full: (
                    f.x as f32,
                    (primary_h - (f.y + f.h)) as f32,
                    f.w as f32,
                    f.h as f32,
                ),
                usable: (
                    v.x as f32,
                    (primary_h - (v.y + v.h)) as f32,
                    v.w as f32,
                    v.h as f32,
                ),
            })
            .collect()
    }
}

// ───── NSWindow 子窗挂接（拖动跟随）─────

/// 主屏高（NSScreen 中原点 (0,0) 屏的 frame 高，单位点）：NS 全局坐标 ↦ Quartz
/// top-left 换算锚（screen_areas 同源式；未命中回退首屏高 · 与 screen_areas
/// unwrap_or(frames[0].0.h) 同口径）。
unsafe fn ns_primary_h() -> f64 {
    let cls = objc_getClass(c"NSScreen".as_ptr());
    if cls.is_null() {
        return 0.0;
    }
    let arr = msg_send_id(cls, sel_registerName(c"screens".as_ptr()));
    if arr.is_null() {
        return 0.0;
    }
    let count = msg_send_usize(arr, sel_registerName(c"count".as_ptr()));
    let sel_at = sel_registerName(c"objectAtIndex:".as_ptr());
    let sel_frame = sel_registerName(c"frame".as_ptr());
    let mut first_h = 0.0;
    for i in 0..count {
        let screen = msg_send_id_idx(arr, sel_at, i);
        if screen.is_null() {
            continue;
        }
        let f = msg_send_rect(screen, sel_frame);
        if first_h == 0.0 {
            first_h = f.h;
        }
        if f.x == 0.0 && f.y == 0.0 {
            return f.h;
        }
    }
    first_h
}

/// 面板↔Peek AppKit 子窗挂接（拖动主面板时内容详情 GUI 同步跟随）：
/// 枚举 `[NSApp windows]`（本进程窗口，含 NSPanel——无需 CGWindowList
/// 权限面），NS 底左原点 frame 换算 Quartz top-left（cg_y = primary_h − (ns_y+ns_h)
/// · screen_areas 同源式），按 bounds 四分量近似匹配（容差 2pt——取整/服务端微调
/// 噪声容忍）定位父窗（主面板实界）与子窗（Peek 实界），命中后
/// `[parent addChildWindow:child ordered:1]`（NSWindowAbove）——AppKit 原生保持
/// 子窗相对偏移与 z 序：拖动主面板全程零采样抖动跟随；Peek 关窗自动解除关联。
/// 任一窗定位失败 / 父子定位到同一窗 → false（调用方保持独立窗语义降级，不阻断）
/// 。仅本进程窗口参与匹配（NSApp windows 即自有窗口，外来窗不可污染）；测试面
/// （TestApp 无真窗语义）定位必失败，调用方 cfg 隔离。macOS 专用（Win/Linux 不
/// 编译本平台桥）；生产调用点均在 gpui 主线程（AppKit 主线程惯例）。
pub fn attach_child_window(
    parent_rect: (f32, f32, f32, f32),
    child_rect: (f32, f32, f32, f32),
) -> bool {
    unsafe {
        let app_cls = objc_getClass(c"NSApplication".as_ptr());
        if app_cls.is_null() {
            return false;
        }
        let app = msg_send_id(app_cls, sel_registerName(c"sharedApplication".as_ptr()));
        if app.is_null() {
            return false;
        }
        let windows = msg_send_id(app, sel_registerName(c"windows".as_ptr()));
        if windows.is_null() {
            return false;
        }
        let count = msg_send_usize(windows, sel_registerName(c"count".as_ptr()));
        let sel_at = sel_registerName(c"objectAtIndex:".as_ptr());
        let sel_frame = sel_registerName(c"frame".as_ptr());
        let sel_add_child = sel_registerName(c"addChildWindow:ordered:".as_ptr());
        let primary_h = ns_primary_h();
        let hit = |f: &NsRect, r: (f32, f32, f32, f32)| -> bool {
            let cg_y = primary_h - (f.y + f.h);
            (f.x - r.0 as f64).abs() <= 2.0
                && (cg_y - r.1 as f64).abs() <= 2.0
                && (f.w - r.2 as f64).abs() <= 2.0
                && (f.h - r.3 as f64).abs() <= 2.0
        };
        let mut parent: *mut c_void = std::ptr::null_mut();
        let mut child: *mut c_void = std::ptr::null_mut();
        for i in 0..count {
            let win = msg_send_id_idx(windows, sel_at, i);
            if win.is_null() {
                continue;
            }
            let frame = msg_send_rect(win, sel_frame);
            if parent.is_null() && hit(&frame, parent_rect) {
                parent = win;
                continue;
            }
            if child.is_null() && hit(&frame, child_rect) {
                child = win;
            }
        }
        if parent.is_null() || child.is_null() || parent == child {
            return false;
        }
        msg_send_void_id_isize(parent, sel_add_child, child, 1); // 1 = NSWindowAbove
        true
    }
}

// ───── IME 候选窗让位桥（组合期自有窗逐窗降级 · DEC-224/DEC-241/DEC-242）─────

// 窗 level 读写消息变体（同符号多签名声明 · 与上方各节同语义放行；仅本节消
// 费，不对外暴露）。读侧（level → NSInteger；NSWindow level getter 实名为
// `level`——`windowLevel` 非 NSWindow 方法，误发即 unrecognized selector
// 闪退 · DEC-241）；写侧（setLevel:）；另新增 (isize)→id 承载 NSNumber
// numberWithLongLong:（逐窗存档载体）。count/objectAtIndex: 复用既有
// msg_send_isize 与 msg_send_id_idx。
#[allow(clashing_extern_declarations)]
#[link(name = "objc")]
extern "C" {
    #[link_name = "objc_msgSend"]
    fn msg_send_void_isize_arg(receiver: *mut c_void, sel: *mut c_void, arg1: isize);
    #[link_name = "objc_msgSend"]
    fn msg_send_id_isize(receiver: *mut c_void, sel: *mut c_void, arg1: isize) -> *mut c_void;
}

// 让位档案载体（libobjc 原生 associated object · 新名字无撞名）：档案挂窗
// 自身，随窗口 dealloc 自动清除——组合中关窗零残留、无悬空指针；键取静态
// 常量地址。object_getClassName 用于窗类名过滤（仅 GPUIWindow/GPUIPanel，
// 排除系统 NSStatusBarWindow / macOS 26+ 注入窗 TUINSWindow）。
#[link(name = "objc")]
extern "C" {
    fn objc_getAssociatedObject(object: *mut c_void, key: *const c_void) -> *mut c_void;
    fn objc_setAssociatedObject(
        object: *mut c_void,
        key: *const c_void,
        value: *mut c_void,
        policy: usize,
    );
    fn object_getClassName(obj: *mut c_void) -> *const std::ffi::c_char;
}

/// IME 候选窗让位（DEC-224 用户指令：组合期候选浮层恒在上；DEC-241 读侧
/// selector 修正；DEC-242 稳定性改代）：`composing=true`——枚举
/// `[NSApp windows]` 中类名为 GPUIPanel/GPUIWindow 的本应用自有窗，level 高
/// 于 NSFloatingWindowLevel(3) 者逐窗存档（NSNumber 经 associated object
/// RETAIN 挂窗自身）后降至 3；`composing=false`——逐窗恢复存档值并清档案。
/// 与 keyWindow 完全解耦（不复读不写 keyWindow）：组合中 key 窗切换、组合中
/// 关窗均安全；重复 demote/多余 restore 幂等（以「该窗已存档与否」为门）。
/// DEC-224 期「单 keyWindow＋进程级静态哨兵」三缺口（其余在场窗仍遮挡／二次
/// 组合降级被哨兵跳过／错窗恢复与关窗哨兵卡死）一并收口。系统窗与他进程候选
/// 浮层不触碰。已知取舍：组合期主面板等自有窗同降至 3——仍浮于普通应用窗(0)
/// 之上，其余 app 浮窗可能短暂越过（登记）。macOS 专用；生产调用点均在 gpui
/// 主线程（AppKit 主线程惯例）。
pub fn ime_key_window_level(composing: bool) {
    use std::ffi::CStr;
    const DEMOTE_LEVEL: isize = 3; // NSFloatingWindowLevel
    const ASSOC_RETAIN: usize = 1; // OBJC_ASSOCIATION_RETAIN
    static DEMOTE_KEY: u8 = 0; // associated-object 键（取地址为键）

    unsafe {
        let app_cls = objc_getClass(c"NSApplication".as_ptr());
        if app_cls.is_null() {
            return;
        }
        let app = msg_send_id(app_cls, sel_registerName(c"sharedApplication".as_ptr()));
        if app.is_null() {
            return;
        }
        let windows = msg_send_id(app, sel_registerName(c"windows".as_ptr()));
        if windows.is_null() {
            return;
        }
        let count = msg_send_isize(windows, sel_registerName(c"count".as_ptr()));
        let number_cls = objc_getClass(c"NSNumber".as_ptr());
        if number_cls.is_null() {
            return;
        }
        let sel_nth = sel_registerName(c"objectAtIndex:".as_ptr());
        let sel_num = sel_registerName(c"numberWithLongLong:".as_ptr());
        let sel_ll_value = sel_registerName(c"longLongValue".as_ptr());
        let sel_level = sel_registerName(c"level".as_ptr());
        let sel_set_level = sel_registerName(c"setLevel:".as_ptr());
        let key = (&DEMOTE_KEY as *const u8).cast::<c_void>();
        for i in 0..count as usize {
            let win = msg_send_id_idx(windows, sel_nth, i);
            if win.is_null() {
                continue;
            }
            let cname = object_getClassName(win);
            if cname.is_null() {
                continue;
            }
            let name = CStr::from_ptr(cname).to_bytes();
            if name != b"GPUIPanel" && name != b"GPUIWindow" {
                continue;
            }
            if composing {
                if !objc_getAssociatedObject(win, key).is_null() {
                    continue; // 已降级（幂等）
                }
                let level = msg_send_isize(win, sel_level);
                if level <= DEMOTE_LEVEL {
                    continue; // 不构成遮挡，不存档不降级
                }
                msg_send_void_isize_arg(win, sel_set_level, DEMOTE_LEVEL);
                let num = msg_send_id_isize(number_cls, sel_num, level);
                objc_setAssociatedObject(win, key, num, ASSOC_RETAIN);
            } else {
                let saved = objc_getAssociatedObject(win, key);
                if saved.is_null() {
                    continue; // 未降级（幂等）
                }
                let level = msg_send_isize(saved, sel_ll_value);
                msg_send_void_isize_arg(win, sel_set_level, level);
                objc_setAssociatedObject(win, key, std::ptr::null_mut(), ASSOC_RETAIN);
            }
        }
    }
}

// ───── NSApp 应用图标桥接（设置应用图标=品牌 logo）─────

// 应用图标桥接消息变体（同符号多签名声明 · 与上方各节同语义放行；仅本节消费，
// 不对外暴露）：(bytes,len)→id 承载 NSData dataWithBytes:length:；id→id 承载
// NSImage initWithData:（alloc 后单参初始化）。
#[allow(clashing_extern_declarations)]
#[link(name = "objc")]
extern "C" {
    #[link_name = "objc_msgSend"]
    fn msg_send_id_ptr_len(
        receiver: *mut c_void,
        sel: *mut c_void,
        bytes: *const c_void,
        len: usize,
    ) -> *mut c_void;
    #[link_name = "objc_msgSend"]
    fn msg_send_id_id(receiver: *mut c_void, sel: *mut c_void, arg: *mut c_void) -> *mut c_void;
}

/// 应用图标装配。**主线程
/// 专用**（AppKit 惯例 · gpui App::run 闭包内调用，同 create_status_item）：内嵌
/// PNG → NSData dataWithBytes:length:（拷贝字节，入参指针仅生存于调用期）→
/// NSImage alloc/initWithData: → NSApp setApplicationIconImage:。任一环节失败
/// stderr 落档返回（图标缺失不阻断启动）；成功落一行判读（含字节数，供真机/e2e
/// 日志核验）。纯 C FFI（libobjc · 零新增依赖 · 规范 9）。
pub fn create_app_icon(png: &[u8]) {
    if png.is_empty() {
        eprintln!("[parchora] 应用图标装配失败：内嵌 PNG 为空");
        return;
    }
    unsafe {
        let data_cls = objc_getClass(c"NSData".as_ptr());
        if data_cls.is_null() {
            eprintln!("[parchora] 应用图标装配失败：NSData 类不可用");
            return;
        }
        let data = msg_send_id_ptr_len(
            data_cls,
            sel_registerName(c"dataWithBytes:length:".as_ptr()),
            png.as_ptr() as *const c_void,
            png.len(),
        );
        if data.is_null() {
            eprintln!("[parchora] 应用图标装配失败：NSData dataWithBytes 失败");
            return;
        }
        let img_cls = objc_getClass(c"NSImage".as_ptr());
        if img_cls.is_null() {
            eprintln!("[parchora] 应用图标装配失败：NSImage 类不可用");
            return;
        }
        let img = msg_send_id(img_cls, sel_registerName(c"alloc".as_ptr()));
        if img.is_null() {
            eprintln!("[parchora] 应用图标装配失败：NSImage alloc 失败");
            return;
        }
        let img = msg_send_id_id(img, sel_registerName(c"initWithData:".as_ptr()), data);
        if img.is_null() {
            eprintln!("[parchora] 应用图标装配失败：NSImage initWithData（内嵌 PNG 非法）");
            return;
        }
        let app_cls = objc_getClass(c"NSApplication".as_ptr());
        if app_cls.is_null() {
            eprintln!("[parchora] 应用图标装配失败：NSApplication 类不可用");
            return;
        }
        let app = msg_send_id(app_cls, sel_registerName(c"sharedApplication".as_ptr()));
        if app.is_null() {
            eprintln!("[parchora] 应用图标装配失败：NSApp 不可用");
            return;
        }
        msg_send_void_id(
            app,
            sel_registerName(c"setApplicationIconImage:".as_ptr()),
            img,
        );
        eprintln!(
            "[parchora] 应用图标装配完成（品牌 logo · {} 字节内嵌 PNG）",
            png.len()
        );
    }
}

// ───── NSStatusItem 状态栏桥接 ─────

/// 状态栏符号（fallback 第二级 · SF Symbols 名；第一级=品牌 logo ·
/// 第二级=SF Symbol note.text · 第三级=title 字形）。
const STATUS_SYMBOL: &str = "note.text";
/// 状态栏符号 accessibilityDescription（nil 亦可，给名便于读屏）。
const STATUS_SYMBOL_DESC: &str = "Parchora";

/// 状态栏点击旗标（菜单项 action IMP 写 · app 层守护 0.05s 排水 toggle 面板）。
static STATUS_CLICK: AtomicBool = AtomicBool::new(false);

/// 状态栏「退出」意図旗标（写源=菜单「退出」项 action IMP；左右键统一开菜单、
/// 退出归菜单项；app 层排水后走清理链 quit_app）。
static STATUS_QUIT: AtomicBool = AtomicBool::new(false);

/// 状态栏菜单开合旗标（menuWillOpen/menuDidClose delegate IMP 写 · 真值窗口
/// = 菜单模态期）：app 层 reveal 守护以此暂停边缘检测（SCN-SETTINGS-MENU-PAUSES-EDGE）。
static STATUS_MENU_OPEN: AtomicBool = AtomicBool::new(false);

/// 左键 up + 右键 down 双触发掩码（sendActionOn 4|8；掩码无消费点——保留
/// 供回归口径对账与 fallback 侧手动开菜单语义在案；仅测试域消费，生产
/// dead_code 显式放行）。
#[cfg_attr(not(test), allow(dead_code))]
const STATUS_SEND_ACTION_MASK: isize = (1 << 2) | (1 << 3);

/// click 目标对象裸指针（setTarget: 弱引用不 retain——AppKit 文档语义，对象释放
/// 即悬垂；alloc/init 的 +1 引用刻意永不释放保活，usize 指针仅经静态 Mutex 存取
/// · 演示宿主单次装配）。
static STATUS_TARGET: Mutex<usize> = Mutex::new(0);

/// 状态栏 item 本体强引用（statusItemWithLength: 以 autorelease 返回——Apple
/// 文档明示调用方须持有强引用，否则首个 autorelease 池排空即释放、菜单栏窗随之
/// 消失）。
/// retain 后 usize 指针静态保活（进程生命周期 · 同 STATUS_TARGET 先例）。
static STATUS_ITEM: Mutex<usize> = Mutex::new(0);

/// 状态栏点击 IMP（types "v@:@" =(void)(self, _cmd, sender) → 仅置位旗标；
/// 排水在 app 层守护统一消费，无锁竞争面）。读 NSApp.currentEvent
/// .type 分流——rightMouseDown(3)=退出意図（STATUS_QUIT），其余=点击显隐
///（STATUS_CLICK · DEC-90）。
unsafe extern "C" fn status_click_imp(_this: *mut c_void, _cmd: *mut c_void, _sender: *mut c_void) {
    unsafe {
        let app_cls = objc_getClass(c"NSApplication".as_ptr());
        if !app_cls.is_null() {
            let app = msg_send_id(app_cls, sel_registerName(c"sharedApplication".as_ptr()));
            if !app.is_null() {
                let ev = msg_send_id(app, sel_registerName(c"currentEvent".as_ptr()));
                if !ev.is_null() {
                    // NSEvent.type → NSInteger（NSEventType rightMouseDown=3）。
                    let ty = msg_send_isize(ev, sel_registerName(c"type".as_ptr()));
                    if ty == 3 {
                        STATUS_QUIT.store(true, Ordering::Release);
                        return;
                    }
                }
            }
        }
    }
    STATUS_CLICK.store(true, Ordering::Release);
}

/// 菜单「呼出面板」项 action IMP（types "v@:@"）：仅置位
/// STATUS_CLICK——app 层守护排水后走 status_click_toggle。
unsafe extern "C" fn status_menu_toggle_imp(
    _this: *mut c_void,
    _cmd: *mut c_void,
    _sender: *mut c_void,
) {
    STATUS_CLICK.store(true, Ordering::Release);
}

/// 菜单「退出」项 action IMP：仅置位 STATUS_QUIT——app
/// 层守护排水后走清理链 quit_app（先清资源再 cx.quit()）。
unsafe extern "C" fn status_menu_quit_imp(
    _this: *mut c_void,
    _cmd: *mut c_void,
    _sender: *mut c_void,
) {
    STATUS_QUIT.store(true, Ordering::Release);
}

/// NSMenu delegate「菜单将开」IMP（types "v@:@"）：置位
/// STATUS_MENU_OPEN=true——reveal 守护据此暂停边缘检测（
/// SCN-SETTINGS-MENU-PAUSES-EDGE）。
unsafe extern "C" fn status_menu_will_open_imp(
    _this: *mut c_void,
    _cmd: *mut c_void,
    _menu: *mut c_void,
) {
    STATUS_MENU_OPEN.store(true, Ordering::Release);
    eprintln!("[parchora] 状态栏菜单 willOpen（模态期开 · 边缘检测暂停中）");
}

/// NSMenu delegate「菜单已闭」IMP（types "v@:@"）：清零
/// STATUS_MENU_OPEN——边缘检测复臂。
unsafe extern "C" fn status_menu_did_close_imp(
    _this: *mut c_void,
    _cmd: *mut c_void,
    _menu: *mut c_void,
) {
    STATUS_MENU_OPEN.store(false, Ordering::Release);
    eprintln!("[parchora] 状态栏菜单 didClose（模态期收 · 边缘检测复臂）");
}

// 状态栏桥接消息变体与 class-pair 三件套（同符号多签名声明 · 与上方
// objc block 同语义放行；仅本节消费，不对外暴露）。
#[allow(clashing_extern_declarations)]
#[link(name = "objc")]
extern "C" {
    #[link_name = "objc_allocateClassPair"]
    fn objc_allocate_class_pair(
        super_cls: *mut c_void,
        name: *const std::ffi::c_char,
        extra: usize,
    ) -> *mut c_void;
    #[link_name = "objc_registerClassPair"]
    fn objc_register_class_pair(cls: *mut c_void);
    #[link_name = "class_addMethod"]
    fn class_add_method(
        cls: *mut c_void,
        sel: *mut c_void,
        imp: *mut c_void,
        types: *const std::ffi::c_char,
    ) -> bool;
    #[link_name = "objc_msgSend"]
    fn msg_send_void_isize(receiver: *mut c_void, sel: *mut c_void, arg: isize);
    #[link_name = "objc_msgSend"]
    fn msg_send_void_id(receiver: *mut c_void, sel: *mut c_void, arg: *mut c_void);
    #[link_name = "objc_msgSend"]
    fn msg_send_id_f64(receiver: *mut c_void, sel: *mut c_void, arg: f64) -> *mut c_void;
    #[link_name = "objc_msgSend"]
    fn msg_send_id_ptr(
        receiver: *mut c_void,
        sel: *mut c_void,
        arg: *const std::ffi::c_char,
    ) -> *mut c_void;
    #[link_name = "objc_msgSend"]
    fn msg_send_id_id_id(
        receiver: *mut c_void,
        sel: *mut c_void,
        a: *mut c_void,
        b: *mut c_void,
    ) -> *mut c_void;
    #[link_name = "objc_msgSend"]
    fn msg_send_bool_id(receiver: *mut c_void, sel: *mut c_void, arg: *mut c_void) -> bool;
    #[link_name = "objc_msgSend"]
    fn msg_send_isize(receiver: *mut c_void, sel: *mut c_void) -> isize;
    #[link_name = "objc_msgSend"]
    fn msg_send_void(receiver: *mut c_void, sel: *mut c_void);
    #[link_name = "objc_msgSend"]
    fn msg_send_void_size(receiver: *mut c_void, sel: *mut c_void, arg: NsSize);
}

/// NSSize（CGSize 同构 · 平铺双 f64 · setSize: 入参传值；ARM64 寄存器传参 ·
/// 同 msg_send_struct 返回同语义；x86_64 不在本仓目标域 · 登记局限）。
#[repr(C)]
struct NsSize {
    w: f64,
    h: f64,
}

/// NSString 构造（stringWithUTF8String: 类方法）。返回交消费 selector 立即使用
/// （AppKit 自行持有/拷贝），Rust 侧不留引用。构造失败 → null（调用方 fallback）。
unsafe fn ns_string(s: &str) -> *mut c_void {
    let Ok(cs) = std::ffi::CString::new(s) else {
        return std::ptr::null_mut();
    };
    let cls = objc_getClass(c"NSString".as_ptr());
    if cls.is_null() {
        return std::ptr::null_mut();
    }
    msg_send_id_ptr(
        cls,
        sel_registerName(c"stringWithUTF8String:".as_ptr()),
        cs.as_ptr(),
    )
}

/// 状态栏品牌 logo 挂图：内嵌 PNG bytes → NSData
/// dataWithBytes:length:（拷贝字节）→ NSImage alloc/initWithData: → setSize
///（18×18 pt · 菜单栏标准高度域）→ button.setImage:。非模板图（NSImage 默认
/// isTemplate=NO；图标稿=DEC-160 用户交付新稿，明暗菜单栏可视性待真机眼验）。
/// 返回是否已挂图（失败由调用方走 SF Symbol / title 字形 fallback 链）。
unsafe fn set_logo_image(target: *mut c_void, png: &[u8]) -> bool {
    if target.is_null() || png.is_empty() {
        return false;
    }
    let data_cls = objc_getClass(c"NSData".as_ptr());
    if data_cls.is_null() {
        return false;
    }
    let data = msg_send_id_ptr_len(
        data_cls,
        sel_registerName(c"dataWithBytes:length:".as_ptr()),
        png.as_ptr() as *const c_void,
        png.len(),
    );
    if data.is_null() {
        return false;
    }
    let img_cls = objc_getClass(c"NSImage".as_ptr());
    if img_cls.is_null() {
        return false;
    }
    let img = msg_send_id(img_cls, sel_registerName(c"alloc".as_ptr()));
    let img = msg_send_id_id(img, sel_registerName(c"initWithData:".as_ptr()), data);
    if img.is_null() {
        return false;
    }
    msg_send_void_size(
        img,
        sel_registerName(c"setSize:".as_ptr()),
        NsSize { w: 18.0, h: 18.0 },
    );
    msg_send_void_id(target, sel_registerName(c"setImage:".as_ptr()), img);
    true
}

/// 状态栏符号挂图（fallback 第二级 · SF Symbol · macOS 11+；respondsToSelector
/// 门槛——低版本直接 false 走 title 字形 fallback，防未识别 selector 崩溃）。
unsafe fn set_status_image(target_view: *mut c_void) -> bool {
    if target_view.is_null() {
        return false;
    }
    let img_cls = objc_getClass(c"NSImage".as_ptr());
    if img_cls.is_null() {
        return false;
    }
    let sel_sym = sel_registerName(c"imageWithSystemSymbolName:accessibilityDescription:".as_ptr());
    let responds = msg_send_bool_id(
        img_cls,
        sel_registerName(c"respondsToSelector:".as_ptr()),
        sel_sym,
    );
    if !responds {
        return false;
    }
    let sym = ns_string(STATUS_SYMBOL);
    if sym.is_null() {
        return false;
    }
    let desc = ns_string(STATUS_SYMBOL_DESC);
    let img = msg_send_id_id_id(img_cls, sel_sym, sym, desc);
    if img.is_null() {
        return false;
    }
    msg_send_void_id(target_view, sel_registerName(c"setImage:".as_ptr()), img);
    true
}

/// 状态栏常驻品牌图标装配（REQ-SETTINGS-STATUSBAR「菜单栏常驻」演示宿主承载；
/// png 入源优先，SF Symbol→title 字形退为 fallback 链）。
/// **主线程专用**（AppKit 惯例 · gpui App::run 闭包内调用）：
/// 1. NSApp activation policy = Accessory(1)：无 Dock 图标；
/// 2. NSStatusBar systemStatusBar → statusItemWithLength:NSVariableStatusItemLength
///    （-1.0 · 随内容自适应宽）——返回 autorelease 对象，retain 后经 STATUS_ITEM
///    进程保活；
/// 3. 点击目标类动态构造（allocateClassPair + addMethod "v @:@" + register）·
///    target 裸指针静态保活（setTarget 弱引用）；类注册四法
///    （statusMenuToggle:/statusMenuQuit: 菜单项 action 与 menuWillOpen:/
///    menuDidClose: 菜单 delegate）；按钮不设 action（setAction: nil）——
///    左右键统一经菜单打开。
/// 4. 菜单装配：NSMenu「Parchora」·
///    项①呼出主面板（statusMenuToggle: → STATUS_CLICK 排水 toggle）· 项②退出
///    （statusMenuQuit: → STATUS_QUIT 排水清理链）；setDelegate=target
///    （弱引用语义同 setTarget · 实例静态保活）；setMenu: 打在 NSStatusItem
///    本体（button.menu＝上下文菜单槽，不接线点击）——挂后无需手动弹菜单
///    （AppKit 内建：左右键按下自动打开 item.menu），回读自证落档判读。
/// 5. 图标三级 fallback：品牌 logo PNG → imageWithSystemSymbolName（macOS 11+）→
///    title 字形「❖」。任一失败链式回退不阻断装配（stderr 落档判读）。
pub fn create_status_item(png: &[u8]) {
    unsafe {
        let app_cls = objc_getClass(c"NSApplication".as_ptr());
        if !app_cls.is_null() {
            let app = msg_send_id(app_cls, sel_registerName(c"sharedApplication".as_ptr()));
            if !app.is_null() {
                msg_send_void_isize(app, sel_registerName(c"setActivationPolicy:".as_ptr()), 1);
            }
        }
        let bar_cls = objc_getClass(c"NSStatusBar".as_ptr());
        if bar_cls.is_null() {
            eprintln!("[parchora] 状态栏图标创建失败：NSStatusBar 类不可用");
            return;
        }
        let bar = msg_send_id(bar_cls, sel_registerName(c"systemStatusBar".as_ptr()));
        if bar.is_null() {
            eprintln!("[parchora] 状态栏图标创建失败：systemStatusBar 不可用");
            return;
        }
        let item = msg_send_id_f64(
            bar,
            sel_registerName(c"statusItemWithLength:".as_ptr()),
            -1.0,
        );
        if item.is_null() {
            eprintln!("[parchora] 状态栏图标创建失败：statusItemWithLength 失败");
            return;
        }
        // item 本体 retain 静态保活（见 STATUS_ITEM 文档）——
        // autorelease 对象未持有时池排空即释放，图标从未绘制；槽锁中毒不阻断装配。
        let item_retained = msg_send_id(item, sel_registerName(c"retain".as_ptr()));
        if let Ok(mut si) = STATUS_ITEM.lock() {
            *si = item_retained as usize;
        }
        // click 目标类/实例（一次构造；重复调用幂等——静态指针已登记则复用）。
        // 槽锁中毒即装配中断（理论路径 · 临界区无 panic 点），与槽锁他处 .ok() 风格统一。
        let Ok(mut slot) = STATUS_TARGET.lock() else {
            return;
        };
        if *slot == 0 {
            let base = objc_getClass(c"NSObject".as_ptr());
            if base.is_null() {
                eprintln!("[parchora] 状态栏目标类构造失败：NSObject 不可用");
                return;
            }
            let cls = objc_allocate_class_pair(base, c"ParchoraStatusClickTarget".as_ptr(), 0);
            if cls.is_null() {
                eprintln!("[parchora] 状态栏目标类构造失败：allocateClassPair");
                return;
            }
            type StatusClickImp = unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void);
            if !class_add_method(
                cls,
                sel_registerName(c"statusClick:".as_ptr()),
                status_click_imp as StatusClickImp as *mut c_void,
                c"v@:@".as_ptr(),
            ) {
                eprintln!("[parchora] 状态栏目标类构造失败：addMethod");
                return;
            }
            // 菜单项 action 两法 + 菜单 delegate 开/闭两法
            // （「v@:@」＝(void)(self,_cmd,arg)；同幂等构造块内一次注册）。
            {
                if !class_add_method(
                    cls,
                    sel_registerName(c"statusMenuToggle:".as_ptr()),
                    status_menu_toggle_imp as StatusClickImp as *mut c_void,
                    c"v@:@".as_ptr(),
                ) {
                    eprintln!("[parchora] 状态栏类构造失败：addMethod statusMenuToggle:");
                    return;
                }
                if !class_add_method(
                    cls,
                    sel_registerName(c"statusMenuQuit:".as_ptr()),
                    status_menu_quit_imp as StatusClickImp as *mut c_void,
                    c"v@:@".as_ptr(),
                ) {
                    eprintln!("[parchora] 状态栏类构造失败：addMethod statusMenuQuit:");
                    return;
                }
                if !class_add_method(
                    cls,
                    sel_registerName(c"menuWillOpen:".as_ptr()),
                    status_menu_will_open_imp as StatusClickImp as *mut c_void,
                    c"v@:@".as_ptr(),
                ) {
                    eprintln!("[parchora] 状态栏类构造失败：addMethod menuWillOpen:");
                    return;
                }
                if !class_add_method(
                    cls,
                    sel_registerName(c"menuDidClose:".as_ptr()),
                    status_menu_did_close_imp as StatusClickImp as *mut c_void,
                    c"v@:@".as_ptr(),
                ) {
                    eprintln!("[parchora] 状态栏类构造失败：addMethod menuDidClose:");
                    return;
                }
            }
            objc_register_class_pair(cls);
            let target = msg_send_id(cls, sel_registerName(c"alloc".as_ptr()));
            let target = msg_send_id(target, sel_registerName(c"init".as_ptr()));
            if target.is_null() {
                eprintln!("[parchora] 状态栏目标实例化失败");
                return;
            }
            *slot = target as usize;
        }
        let target = *slot as *mut c_void;
        drop(slot);
        if target.is_null() {
            return;
        }
        // 挂接点：button（10.14+ 属性）优先，缺 button 走旧式 item 直挂（同语义）。
        let button = msg_send_id(item, sel_registerName(c"button".as_ptr()));
        let mount = if !button.is_null() { button } else { item };
        msg_send_void_id(mount, sel_registerName(c"setTarget:".as_ptr()), target);
        // 菜单统一左右键——按钮 action 清除；若不清，sendActionOn 右键 down
        // 掩码仍命中 statusClick: → 右键即时退出而非开菜单（确定性硬伤），
        // 此处显式置 nil 阻断。
        msg_send_void_id(
            mount,
            sel_registerName(c"setAction:".as_ptr()),
            std::ptr::null_mut(),
        );
        // ---- NSMenu 装配：呼出主面板 + 退出 ----
        let menu_cls = objc_getClass(c"NSMenu".as_ptr());
        if menu_cls.is_null() {
            eprintln!("[parchora] 状态栏菜单装配失败：NSMenu 类不可用");
        } else {
            let menu = msg_send_id(menu_cls, sel_registerName(c"alloc".as_ptr()));
            let title = ns_string("Parchora");
            let menu = if title.is_null() {
                menu
            } else {
                msg_send_id_id(menu, sel_registerName(c"initWithTitle:".as_ptr()), title)
            };
            if !menu.is_null() {
                let item_cls = objc_getClass(c"NSMenuItem".as_ptr());
                if !item_cls.is_null() {
                    // 项①呼出主面板：title/target/action 逐槽分离式设置（alloc→init
                    // 后逐 setter，复用既有 void helper 零新 extern）。
                    let mi = msg_send_id(item_cls, sel_registerName(c"alloc".as_ptr()));
                    let mi = msg_send_id(mi, sel_registerName(c"init".as_ptr()));
                    let t1 = ns_string("呼出主面板");
                    if !mi.is_null() && !t1.is_null() {
                        msg_send_void_id(mi, sel_registerName(c"setTitle:".as_ptr()), t1);
                        msg_send_void_id(mi, sel_registerName(c"setTarget:".as_ptr()), target);
                        msg_send_void_id(
                            mi,
                            sel_registerName(c"setAction:".as_ptr()),
                            sel_registerName(c"statusMenuToggle:".as_ptr()),
                        );
                        let empty_key = ns_string("");
                        if !empty_key.is_null() {
                            msg_send_void_id(
                                mi,
                                sel_registerName(c"setKeyEquivalent:".as_ptr()),
                                empty_key,
                            );
                        }
                        msg_send_void_id(menu, sel_registerName(c"addItem:".as_ptr()), mi);
                        msg_send_void(mi, sel_registerName(c"release".as_ptr()));
                    } else if !mi.is_null() {
                        msg_send_void(mi, sel_registerName(c"release".as_ptr()));
                    }
                }
                // 分隔线：separatorItem 类方法在本机（macOS 27 消息链）单独调用即崩
                //——v1 菜单两款直排不加线（纯视觉位），产品壳轮 5 项菜单装配时再议复挂。
                if !item_cls.is_null() {
                    // 项②退出（statusMenuQuit: → STATUS_QUIT → 清理链）。
                    let mi = msg_send_id(item_cls, sel_registerName(c"alloc".as_ptr()));
                    let mi = msg_send_id(mi, sel_registerName(c"init".as_ptr()));
                    let t2 = ns_string("退出");
                    if !mi.is_null() && !t2.is_null() {
                        msg_send_void_id(mi, sel_registerName(c"setTitle:".as_ptr()), t2);
                        msg_send_void_id(mi, sel_registerName(c"setTarget:".as_ptr()), target);
                        msg_send_void_id(
                            mi,
                            sel_registerName(c"setAction:".as_ptr()),
                            sel_registerName(c"statusMenuQuit:".as_ptr()),
                        );
                        let empty_key = ns_string("");
                        if !empty_key.is_null() {
                            msg_send_void_id(
                                mi,
                                sel_registerName(c"setKeyEquivalent:".as_ptr()),
                                empty_key,
                            );
                        }
                        msg_send_void_id(menu, sel_registerName(c"addItem:".as_ptr()), mi);
                        msg_send_void(mi, sel_registerName(c"release".as_ptr()));
                    } else if !mi.is_null() {
                        msg_send_void(mi, sel_registerName(c"release".as_ptr()));
                    }
                }
                // delegate=target：菜单开/闭旗标承载（SCN-SETTINGS-MENU-PAUSES-EDGE）；
                // target 实例静态保活，delegate 弱引用安全（同 setTarget）。
                msg_send_void_id(menu, sel_registerName(c"setDelegate:".as_ptr()), target);
                // setMenu: 必须打在 NSStatusItem 本体：
                // item.menu 才是「点击开菜单」接线（AppKit 内建·左右键按下自动
                // 打开）；button 的 menu 属性＝NSResponder 上下文菜单槽，状态栏
                // 点击路径不消费。
                msg_send_void_id(item, sel_registerName(c"setMenu:".as_ptr()), menu);
                // 回读自证：item.menu 非空＝点击接线在册；项数落档供真机判读
                //（失败行即刻暴露装配未生效）。
                let menu_back = msg_send_id(item, sel_registerName(c"menu".as_ptr()));
                if menu_back.is_null() {
                    eprintln!("[parchora] 状态栏 item.menu 回读失败（未挂上 item · 点击不响应）");
                } else {
                    let arr = msg_send_id(menu_back, sel_registerName(c"itemArray".as_ptr()));
                    let n = msg_send_isize(arr, sel_registerName(c"count".as_ptr()));
                    eprintln!("[parchora] 状态栏 item.menu 回读确认（{n} 项 · 左右键点击打开）");
                }
                // alloc/init +1 已由 setMenu: 的 retain 消费，Rust 侧释放收口。
                msg_send_void(menu, sel_registerName(c"release".as_ptr()));
                eprintln!("[parchora] 状态栏菜单已装配（呼出主面板 / 退出 · 左右键同菜单）");
            } else {
                eprintln!("[parchora] 状态栏菜单装配失败：initWithTitle 失败");
            }
        }
        // 图标三级 fallback（品牌 logo 优先）：
        // ① 内嵌 PNG 直绘（非模板 · 灰版）；② SF Symbol note.text；③ title 字形。
        if !set_logo_image(mount, png) && !set_status_image(mount) {
            // title 字形 fallback（系统字库通用字符；10.15 级主机可达）。
            let title = ns_string("❖");
            if !title.is_null() {
                msg_send_void_id(mount, sel_registerName(c"setTitle:".as_ptr()), title);
            }
        }
    }
}

/// 排水状态栏点击旗标（app 层 0.05s 守护调用）：true = 用户点击待处理；
/// 读后清零（幂等排水，重复 take 不重复响应）。
pub fn status_click_take() -> bool {
    STATUS_CLICK.swap(false, Ordering::AcqRel)
}

/// 排水状态栏「退出」意図旗标（写源=菜单「退出」项 action IMP · app 层
/// quit_app 清理链入口）。
pub fn status_quit_take() -> bool {
    STATUS_QUIT.swap(false, Ordering::AcqRel)
}

/// 状态栏菜单开合态读数（只读不清零）：true=菜单模态期——
/// app 层 reveal 守护据此暂停边缘检测（SCN-SETTINGS-MENU-PAUSES-EDGE）。
pub fn status_menu_open() -> bool {
    STATUS_MENU_OPEN.load(Ordering::Acquire)
}

/// 移除状态栏 item 并释放保活引用（退出清理链终点；幂等：
/// 未装配=直返；**主线程专用** AppKit 惯例。removeStatusItem: 后 release 对
/// create_status_item retain 的引用，槽位清零）。
pub fn remove_status_item() {
    let ptr = match STATUS_ITEM.lock() {
        Ok(si) => *si,
        Err(_) => return,
    };
    if ptr == 0 {
        return;
    }
    unsafe {
        let bar_cls = objc_getClass(c"NSStatusBar".as_ptr());
        if !bar_cls.is_null() {
            let bar = msg_send_id(bar_cls, sel_registerName(c"systemStatusBar".as_ptr()));
            if !bar.is_null() {
                msg_send_void_id(
                    bar,
                    sel_registerName(c"removeStatusItem:".as_ptr()),
                    ptr as *mut c_void,
                );
            }
        }
        msg_send_void(ptr as *mut c_void, sel_registerName(c"release".as_ptr()));
    }
    if let Ok(mut si) = STATUS_ITEM.lock() {
        *si = 0;
    }
}

/// 状态栏 item 窗口自证读数：Some((w,h)) = item 指针
/// 在册且 button/窗已 materialize（frame w/h>0）；None = 指针空或窗未挂。
/// **主线程专用**（AppKit 惯例 · app 层前台续体调用）。macOS 27 上
/// CGWindowList 对所有应用的状态栏窗不可枚举，进程内
/// 直读 item→button→window frame 是唯一可靠常驻口径。
pub fn status_item_window_size() -> Option<(f32, f32)> {
    let ptr = match STATUS_ITEM.lock() {
        Ok(si) => *si,
        Err(_) => return None,
    };
    if ptr == 0 {
        return None;
    }
    unsafe {
        let item = ptr as *mut c_void;
        let button = msg_send_id(item, sel_registerName(c"button".as_ptr()));
        let mount = if !button.is_null() { button } else { item };
        let win = msg_send_id(mount, sel_registerName(c"window".as_ptr()));
        if win.is_null() {
            return None;
        }
        let f = msg_send_rect(win, sel_registerName(c"frame".as_ptr()));
        if f.w > 0.0 && f.h > 0.0 {
            Some((f.w as f32, f.h as f32))
        } else {
            None
        }
    }
}

// ───── NSPasteboard 剪贴板桥接（DEC-163 · 剪贴板捕获守护数据源）─────

// 本节消息变体（同符号多签名声明 · 同语义放行；仅本节消费）。
#[allow(clashing_extern_declarations)]
#[link(name = "objc")]
extern "C" {
    #[link_name = "objc_msgSend"]
    fn msg_send_utf8(receiver: *mut c_void, sel: *mut c_void) -> *const std::ffi::c_char;
}

/// 隐私标记 UTI（REQ-CAPTURE-PRIVACY · nspasteboard.org 约定）：命中任一即拒收
/// ——Transient/Concealed/AutoGenerated 100% 不入捕获链（密码管理器/临时数据等）。
const PASTEBOARD_PRIVACY_MARKERS: [&str; 3] = [
    "org.nspasteboard.TransientType",
    "org.nspasteboard.ConcealedType",
    "org.nspasteboard.AutoGeneratedType",
];

/// NSPasteboard generalPasteboard 共享探针（各桥共用；类缺失/调用失败 → null）。
unsafe fn general_pasteboard() -> *mut c_void {
    let cls = objc_getClass(c"NSPasteboard".as_ptr());
    if cls.is_null() {
        return std::ptr::null_mut();
    }
    msg_send_id(cls, sel_registerName(c"generalPasteboard".as_ptr()))
}

/// 剪贴板 changeCount（NSUInteger；None = 桥失败/类缺失）。守护轮询的“有变化
/// 才读内容”门槛源（REQ-CAPTURE-POLL）：未命中变化不得读板面。**主线程专用**
///（AppKit 惯例 · gpui 守护拍 cx.update 内调用）。
pub fn pasteboard_change_count() -> Option<u64> {
    unsafe {
        let pb = general_pasteboard();
        if pb.is_null() {
            return None;
        }
        Some(msg_send_usize(pb, sel_registerName(c"changeCount".as_ptr())) as u64)
    }
}

/// 板面类型快照桥（TASK-300 · REQ-CAPTURE-8TYPES 全类型面）：板面 UTI 全集
/// → core::classify 类型快照（隐私拒收三态直出）。判定链权威在
/// `parchora_core::classify::classify`，本桥只负责「OS 读板 → 快照」的诚实转写；
/// None = 桥失败/类缺失。**主线程专用**（同 changeCount/stringForType）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PasteboardBoard {
    /// 板上无任何类型（空复制 → 不产生条目）。
    Empty,
    /// 隐私标记命中（Transient/Concealed/AutoGenerated 任一 · 100% 拒收）。
    Rejected,
    /// 类型快照全集（逐类载荷可备 · 顺序即板面 types 序）。
    Snapshots(Vec<PasteboardTypeSnapshot>),
}

// 本节消息变体补充（同符号多签名声明 · 同语义放行；仅本节消费）：
// - msg_send_f64：f64 返回（NSColor 颜色分量）；
// - msg_send_data_len / msg_send_data_bytes：NSData 读数 / 字节首址。
#[allow(clashing_extern_declarations)]
#[link(name = "objc")]
extern "C" {
    #[link_name = "objc_msgSend"]
    fn msg_send_f64(receiver: *mut c_void, sel: *mut c_void) -> f64;
}

unsafe fn ns_data_bytes(data: *mut c_void) -> Option<Vec<u8>> {
    if data.is_null() {
        return None;
    }
    let len = msg_send_usize(data, sel_registerName(c"length".as_ptr()));
    if len == 0 {
        return None;
    }
    let bytes = msg_send_id(data, sel_registerName(c"bytes".as_ptr())) as *const u8;
    if bytes.is_null() {
        return None;
    }
    Some(std::slice::from_raw_parts(bytes, len).to_vec())
}

/// stringForType 取文本（UTF-8 可损转；nil/空串 → None）。
unsafe fn pb_string_for_type(pb: *mut c_void, uti: &str) -> Option<String> {
    let ty = ns_string(uti);
    if ty.is_null() {
        return None;
    }
    let obj = msg_send_id_obj(pb, sel_registerName(c"stringForType:".as_ptr()), ty);
    if obj.is_null() {
        return None;
    }
    let raw = msg_send_utf8(obj, sel_registerName(c"UTF8String".as_ptr()));
    if raw.is_null() {
        return None;
    }
    let s = std::ffi::CStr::from_ptr(raw).to_string_lossy().into_owned();
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

/// dataForType 取字节（TASK-300 image 载荷）。nil/空 → None。
unsafe fn pb_data_for_type(pb: *mut c_void, uti: &str) -> Option<Vec<u8>> {
    let ty = ns_string(uti);
    if ty.is_null() {
        return None;
    }
    let data = msg_send_id_obj(pb, sel_registerName(c"dataForType:".as_ptr()), ty);
    ns_data_bytes(data)
}

/// fileURL 列表读取：propertyListForType(public.file-url) → NSArray<NSURL>
/// → absoluteString 逐个直出（Finder 多选文件等）。
unsafe fn pb_file_urls(pb: *mut c_void) -> Vec<String> {
    let ty = ns_string("public.file-url");
    if ty.is_null() {
        return Vec::new();
    }
    let list = msg_send_id_obj(pb, sel_registerName(c"propertyListForType:".as_ptr()), ty);
    if list.is_null() {
        return Vec::new();
    }
    let sel_count = sel_registerName(c"count".as_ptr());
    let sel_at = sel_registerName(c"objectAtIndex:".as_ptr());
    let sel_abs = sel_registerName(c"absoluteString".as_ptr());
    let sel_utf8 = sel_registerName(c"UTF8String".as_ptr());
    let count = msg_send_usize(list, sel_count);
    let mut out = Vec::new();
    for i in 0..count {
        let url = msg_send_id_idx(list, sel_at, i);
        if url.is_null() {
            continue;
        }
        let s = msg_send_id(url, sel_abs);
        if s.is_null() {
            continue;
        }
        let raw = msg_send_utf8(s, sel_utf8);
        if !raw.is_null() {
            out.push(std::ffi::CStr::from_ptr(raw).to_string_lossy().into_owned());
        }
    }
    out
}

/// 色票读取：NSColor colorWithPasteboard → NSCalibratedRGBColorSpace 归一 →
/// RGBA 分量 hex 文本（`#RRGGBB`，alpha<1 时 `#RRGGBBAA`）。色票读失败 → None
///（防御性弃类型，不虚构造色）。
unsafe fn pb_color_hex(pb: *mut c_void) -> Option<String> {
    let color_cls = objc_getClass(c"NSColor".as_ptr());
    if color_cls.is_null() {
        return None;
    }
    let color = msg_send_id_obj(
        color_cls,
        sel_registerName(c"colorWithPasteboard:".as_ptr()),
        pb,
    );
    if color.is_null() {
        return None;
    }
    let calibrated = msg_send_id_obj(
        color,
        sel_registerName(c"colorUsingColorSpaceName:".as_ptr()),
        ns_string("NSCalibratedRGBColorSpace"),
    );
    if calibrated.is_null() {
        return None; // 分量选择器仅在 RGB 色域可调——归一失败即弃（防崩溃）
    }
    let sel_red = sel_registerName(c"redComponent".as_ptr());
    let sel_green = sel_registerName(c"greenComponent".as_ptr());
    let sel_blue = sel_registerName(c"blueComponent".as_ptr());
    let sel_alpha = sel_registerName(c"alphaComponent".as_ptr());
    let chan = |v: f64| -> u32 { (v.clamp(0.0, 1.0) * 255.0).round() as u32 };
    let (r, g, b) = (
        chan(msg_send_f64(calibrated, sel_red)),
        chan(msg_send_f64(calibrated, sel_green)),
        chan(msg_send_f64(calibrated, sel_blue)),
    );
    let a = msg_send_f64(calibrated, sel_alpha);
    if (a - 1.0).abs() < f64::EPSILON {
        Some(format!("#{r:02X}{g:02X}{b:02X}"))
    } else {
        Some(format!("#{r:02X}{g:02X}{b:02X}{:02X}", chan(a)))
    }
}

/// UTI → 快照逐类转写（读失败/载荷为空的类型不产快照——不虚构造载荷）。
unsafe fn pb_snapshot_for_uti(pb: *mut c_void, uti: &str) -> Vec<PasteboardTypeSnapshot> {
    match uti {
        "public.utf8-plain-text" => pb_string_for_type(pb, uti)
            .map(|t| {
                vec![PasteboardTypeSnapshot::with_text(
                    PasteboardKind::PlainText,
                    t,
                )]
            })
            .unwrap_or_default(),
        "public.html" => pb_string_for_type(pb, uti)
            .map(|t| vec![PasteboardTypeSnapshot::with_text(PasteboardKind::Html, t)])
            .unwrap_or_default(),
        // public.url：URL 字符串以「纯文本载荷」入快照——classify 经 URL 改判
        //（CB :75-80 纯文本 scheme+host 判定）产出 url 类。
        "public.url" | "public.url-name" => pb_string_for_type(pb, uti)
            .map(|t| {
                vec![PasteboardTypeSnapshot::with_text(
                    PasteboardKind::PlainText,
                    t,
                )]
            })
            .unwrap_or_default(),
        "public.rtf" | "NSRTFPboardType" => {
            // RTF 源文先行 NSAttributedString 转纯文本（DEC-188 · 行标题可读承载）；
            // 转换失败回退 lossy 源文（不虚造载荷）。
            pb_data_for_type(pb, uti)
                .map(|bytes| {
                    let text = rtf_to_plain_text(&bytes)
                        .unwrap_or_else(|| String::from_utf8_lossy(&bytes).into_owned());
                    vec![PasteboardTypeSnapshot::with_text(
                        PasteboardKind::RichText,
                        text,
                    )]
                })
                .unwrap_or_default()
        }
        "public.tiff" | "public.png" => pb_data_for_type(pb, uti)
            .map(|bytes| {
                vec![PasteboardTypeSnapshot::with_bytes(
                    PasteboardKind::Image,
                    bytes,
                )]
            })
            .unwrap_or_default(),
        "public.file-url" => pb_file_urls(pb)
            .into_iter()
            .map(|u| PasteboardTypeSnapshot::with_text(PasteboardKind::FileUrl, u))
            .collect(),
        "Apple NScolor pasteboard type" => pb_color_hex(pb)
            .map(|hex| {
                vec![PasteboardTypeSnapshot::with_text(
                    PasteboardKind::Color,
                    hex,
                )]
            })
            .unwrap_or_default(),
        _ => Vec::new(), // 未映射 UTI：不产快照（classify 空判定 → 不捕获；
                         // 与 CB 行为同构——未知专有类型不硬造 unknown 条目）
    }
}

/// 板面类型快照总读（TASK-300 全类型桥）：三态直出。**主线程专用**。
pub fn pasteboard_board() -> Option<PasteboardBoard> {
    unsafe {
        let pb = general_pasteboard();
        if pb.is_null() {
            return None;
        }
        let types = msg_send_id(pb, sel_registerName(c"types".as_ptr()));
        if types.is_null() {
            return None;
        }
        let count = msg_send_usize(types, sel_registerName(c"count".as_ptr()));
        let sel_at = sel_registerName(c"objectAtIndex:".as_ptr());
        let sel_utf8 = sel_registerName(c"UTF8String".as_ptr());
        let mut raw_utis: Vec<String> = Vec::new();
        for i in 0..count {
            let ty = msg_send_id_idx(types, sel_at, i);
            if ty.is_null() {
                continue;
            }
            let raw = msg_send_utf8(ty, sel_utf8);
            if raw.is_null() {
                continue;
            }
            raw_utis.push(std::ffi::CStr::from_ptr(raw).to_string_lossy().into_owned());
        }
        if raw_utis
            .iter()
            .any(|s| PASTEBOARD_PRIVACY_MARKERS.iter().any(|m| s == m))
        {
            return Some(PasteboardBoard::Rejected);
        }
        if raw_utis.is_empty() {
            return Some(PasteboardBoard::Empty);
        }
        let mut snapshots = Vec::new();
        for uti in &raw_utis {
            snapshots.extend(pb_snapshot_for_uti(pb, uti));
        }
        if snapshots.is_empty() {
            Some(PasteboardBoard::Empty)
        } else {
            Some(PasteboardBoard::Snapshots(snapshots))
        }
    }
}

/// RTF→纯文本的 autorelease pool 包裹执行（类便利构造器产物落池；无池线程
///（cargo test 等）调用 AppKit 必须自带池，否则 objc 异常 abort）。配对 alloc/
/// drain（NSAutoreleasePool 非 alloc/init 语义——`new` 返回已持有对象，drain
/// 即释放全部持有）。
unsafe fn autorelease_drain<T>(f: impl FnOnce() -> T) -> T {
    #[allow(clashing_extern_declarations)]
    #[link(name = "objc")]
    extern "C" {
        #[link_name = "objc_msgSend"]
        fn msg_send_new(receiver: *mut c_void, sel: *mut c_void) -> *mut c_void;
    }
    let pool_cls = objc_getClass(c"NSAutoreleasePool".as_ptr());
    if pool_cls.is_null() {
        // 类不可用（理论不可达）：直接执行——上层失败路径各自兜底。
        return f();
    }
    let pool = msg_send_new(pool_cls, sel_registerName(c"new".as_ptr()));
    let out = f();
    if !pool.is_null() {
        msg_send_void(pool, sel_registerName(c"drain".as_ptr()));
    }
    out
}

/// RTF 源文 → 纯文本（DEC-188）：NSAttributedString
/// `initWithData:options:documentAttributes:error:` 以**空 options 字典**由富文
/// 本系统按数据嗅探文档类型（RTF 头 `{\rtf` 识别）→ `.string` UTF-8 直出。
/// nil/失败/空文 → None（调用方回退 lossy 源文，不虚造载荷）。函数体自带
/// autorelease pool（主线程 gpui 事件循环已有池，嵌套池合法；无池线程
///（cargo test）同可安全承载）。**主线程专用**（AppKit 惯例 · 板面快照读点
/// 同链）。纯 C FFI（libobjc · 零新增依赖 · 规范 9）。
pub fn rtf_to_plain_text(rtf: &[u8]) -> Option<String> {
    #[allow(clashing_extern_declarations)]
    #[link(name = "objc")]
    extern "C" {
        #[link_name = "objc_msgSend"]
        fn msg_send_id_o4(
            receiver: *mut c_void,
            sel: *mut c_void,
            a1: *mut c_void,
            a2: *mut c_void,
            a3: *mut c_void,
            a4: *mut c_void,
        ) -> *mut c_void;
    }
    if rtf.is_empty() {
        return None;
    }
    unsafe {
        ensure_appkit_loaded();
        autorelease_drain(|| {
            let data_cls = objc_getClass(c"NSData".as_ptr());
            if data_cls.is_null() {
                return None;
            }
            let data = msg_send_id_ptr_len(
                data_cls,
                sel_registerName(c"dataWithBytes:length:".as_ptr()),
                rtf.as_ptr() as *const c_void,
                rtf.len(),
            );
            if data.is_null() {
                return None;
            }
            let dict_cls = objc_getClass(c"NSDictionary".as_ptr());
            if dict_cls.is_null() {
                return None;
            }
            let empty = msg_send_id(dict_cls, sel_registerName(c"dictionary".as_ptr()));
            if empty.is_null() {
                return None;
            }
            let attr_cls = objc_getClass(c"NSAttributedString".as_ptr());
            if attr_cls.is_null() {
                return None;
            }
            let attr = msg_send_id(attr_cls, sel_registerName(c"alloc".as_ptr()));
            let attr = msg_send_id_o4(
                attr,
                sel_registerName(c"initWithData:options:documentAttributes:error:".as_ptr()),
                data,
                empty,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            );
            if attr.is_null() {
                return None;
            }
            let s = msg_send_id(attr, sel_registerName(c"string".as_ptr()));
            let mut out: Option<String> = None;
            if !s.is_null() {
                let raw = msg_send_utf8(s, sel_registerName(c"UTF8String".as_ptr()));
                if !raw.is_null() {
                    let text = std::ffi::CStr::from_ptr(raw).to_string_lossy().into_owned();
                    if !text.is_empty() {
                        out = Some(text);
                    }
                }
            }
            msg_send_void(attr, sel_registerName(c"release".as_ptr()));
            out
        })
    }
}

// ───── NSPasteboard 写板与承诺上架桥（TASK-301 · REQ-QUEUE-DETECT）─────
//
// 队列承诺式投递（CB 原语义直译）：NSPasteboardItem + setDataProvider 惰性
// 承诺（外部索取＝粘贴信号）+ TransientType 三隐私标记（阻止 Spotlight/
// Chromium/自家守护读取——capture 守护读板恒 Rejected，即 §11.4 S4 的天然
// 抑制承载）+ prepareForNewContents(.currentHostOnly) 本机化（SCN-QUEUE-
// CURRENT-HOST-ONLY）。索取信号经回调置共享槽（provider 可能在任意线程），
// 宿主前沿引擎拍消费——回调内不做业务（design §2.3 注）。

/// 承诺共享槽（进程级单例）：上架时更新真值 payload，provider 回调据其
/// setData 并写索取信号；前台引擎拍消费 demand。声明期 demand 清空＝CB
/// 「一次 placement 只认第一次索取」的承载面（旧 placement 残留信号作废）。
pub struct PromiseShared {
    /// 承诺真值（public.utf8-plain-text 承载文本入队快照）。
    pub payload_text: Mutex<String>,
    /// 索取信号（provider 回调线程写时刻；前台引擎拍 drain 消费）。
    pub demand_at: Mutex<Option<Instant>>,
    /// 最近一次应用激活时刻（NSWorkspace didActivateApplication 事件槽 ·
    /// SCN-QUEUE-EXPLORE-BLOCKED 判定源）。
    pub last_activity_activation: Mutex<Option<Instant>>,
}

static PROMISE_SLOT: PromiseShared = PromiseShared {
    payload_text: Mutex::new(String::new()),
    demand_at: Mutex::new(None),
    last_activity_activation: Mutex::new(None),
};

/// 承诺共享槽读点（引擎拍与声明路径共用）。
pub fn pasteboard_promise_slot() -> &'static PromiseShared {
    &PROMISE_SLOT
}

/// 承诺上架（writeObjects 后）的隐私标记集（§11.4 S4：Transient/
/// Concealed/AutoGenerated——capture 守护同名单拒收）。
const PROMISE_MARKERS: [&str; 3] = PASTEBOARD_PRIVACY_MARKERS;

/// NSPasteboardContentsOptions.currentHostOnly raw 值【查证】（Swift
/// .currentHostOnly = 1＜＜0；10.15 可用面随真机腿验证，respondsToSelector
/// 防御缺失时回退 clearContents 全宿主域——SCN-QUEUE-CURRENT-HOST-ONLY 偏差
/// 登记）。
const PB_CONTENTS_CURRENT_HOST_ONLY: usize = 1;

// 承诺版 provider 哑类（NSPasteboardItemDataProvider 协议）：无状态纯转发，
// 类全局注册一次（OnceLock），实例每次上架新建（NSPasteboardItem 强引用、
// clearContents 随 item 释放——不泄漏）。回调 = 供真值 + 记索取时刻（信号
// 槽单写者语义——回调线程任意但只写共享 Mutex）。
static PROVIDER_CLASS: std::sync::OnceLock<usize> = std::sync::OnceLock::new();

// libobjc 类注册 API（零第三方依赖 · 规范 9）：类对分配/方法/协议注册。
#[allow(clashing_extern_declarations)]
#[link(name = "objc")]
extern "C" {
    fn objc_allocateClassPair(
        superclass: *mut c_void,
        name: *const std::ffi::c_char,
        extra_bytes: usize,
    ) -> *mut c_void;
    fn objc_registerClassPair(cls: *mut c_void);
    fn class_addMethod(
        cls: *mut c_void,
        sel: *mut c_void,
        imp: usize,
        types: *const std::ffi::c_char,
    ) -> std::ffi::c_int;
    fn class_addProtocol(cls: *mut c_void, proto: *mut c_void) -> std::ffi::c_int;
    fn objc_getProtocol(name: *const std::ffi::c_char) -> *mut c_void;
}

/// provider 协议方法 IMP（NSPasteboardItemDataProvider · 任意线程）：供真值
///（payload UTF8 → NSData → item.setData:forType:）+ 索取信号置槽。不做
/// 结算/消耗/探查判定（前台业务——回调零业务）。
unsafe extern "C" fn promise_provide_imp(
    _self: *mut c_void,
    _sel: *mut c_void,
    _pb: *mut c_void,
    item: *mut c_void,
    ty: *mut c_void,
) {
    let slot = pasteboard_promise_slot();
    if let Ok(mut at) = slot.demand_at.lock() {
        *at = Some(Instant::now());
    }
    let text = slot
        .payload_text
        .lock()
        .map(|t| t.clone())
        .unwrap_or_default();
    if text.is_empty() {
        return;
    }
    let data_cls = objc_getClass(c"NSData".as_ptr());
    if data_cls.is_null() {
        return;
    }
    #[allow(clashing_extern_declarations)]
    #[link(name = "objc")]
    extern "C" {
        #[link_name = "objc_msgSend"]
        fn msg_send_id_ptr_len(
            receiver: *mut c_void,
            sel: *mut c_void,
            ptr: *const c_void,
            len: usize,
        ) -> *mut c_void;
    }
    let data = msg_send_id_ptr_len(
        data_cls,
        sel_registerName(c"dataWithBytes:length:".as_ptr()),
        text.as_ptr() as *const c_void,
        text.len(),
    );
    if data.is_null() {
        return;
    }
    msg_send_void_id_isize(
        item,
        sel_registerName(c"setData:forType:".as_ptr()),
        data,
        ty as isize,
    );
}

/// provider 哑类注册（幂等 OnceLock）。
unsafe fn promise_provider_class() -> *mut c_void {
    let cls = *PROVIDER_CLASS.get_or_init(|| unsafe {
        ensure_appkit_loaded();
        let super_cls = objc_getClass(c"NSObject".as_ptr());
        if super_cls.is_null() {
            return 0usize;
        }
        let cls = objc_allocateClassPair(super_cls, c"PRCPasteboardProvider".as_ptr(), 0);
        if cls.is_null() {
            return 0usize;
        }
        let sel = sel_registerName(c"pasteboard:item:provideDataForType:".as_ptr());
        class_addMethod(
            cls,
            sel,
            promise_provide_imp as *const () as usize,
            c"v@:@@@@".as_ptr(),
        );
        let proto = objc_getProtocol(c"NSPasteboardItemDataProvider".as_ptr());
        if !proto.is_null() {
            class_addProtocol(cls, proto);
        }
        objc_registerClassPair(cls);
        cls as usize
    });
    if cls == 0 {
        std::ptr::null_mut()
    } else {
        cls as *mut c_void
    }
}

/// 声明期类型数组（NSMutableArray 便利构造；autoreleased 池语义）。
unsafe fn promise_types_array(types: &[&str]) -> *mut c_void {
    let arr_cls = objc_getClass(c"NSMutableArray".as_ptr());
    if arr_cls.is_null() {
        return std::ptr::null_mut();
    }
    let arr = msg_send_id(arr_cls, sel_registerName(c"array".as_ptr()));
    if arr.is_null() {
        return std::ptr::null_mut();
    }
    for uti in types {
        let s = ns_string(uti);
        if s.is_null() {
            continue;
        }
        msg_send_void_id_isize(arr, sel_registerName(c"addObject:".as_ptr()), s, 0);
    }
    arr
}

/// NSPasteboardItem 承诺上架：声明 UTF8 公开类型 provider 承诺 + 三隐私标记
///（空数据瞬时写定 · CB 原语义）+ prepareForNewContents(currentHostOnly)
/// 本机化 + writeObjects。返回上架后 changeCount 基线（None = 桥失败）。
/// **主线程专用**（AppKit 板面写惯例 · 宿主引擎拍承载）。声明即清 demand
/// 残留信号（新 placement 作废旧 placement 索取——见 PromiseShared 注）。
pub fn pasteboard_declare_promise(text: &str) -> Option<u64> {
    {
        // 声明前更新真值 + 清旧 demand（失败路径同清——声明意图已发生）。
        if let Ok(mut s) = PROMISE_SLOT.payload_text.lock() {
            *s = text.to_string();
        }
        if let Ok(mut d) = PROMISE_SLOT.demand_at.lock() {
            *d = None;
        }
    }
    unsafe {
        ensure_appkit_loaded();
        autorelease_drain(|| {
            let pb = general_pasteboard();
            if pb.is_null() {
                return None;
            }
            let provider_cls = promise_provider_class();
            if provider_cls.is_null() {
                return None;
            }
            let provider = msg_send_id(provider_cls, sel_registerName(c"alloc".as_ptr()));
            let provider = msg_send_id(provider, sel_registerName(c"init".as_ptr()));
            if provider.is_null() {
                return None;
            }
            let item_cls = objc_getClass(c"NSPasteboardItem".as_ptr());
            if item_cls.is_null() {
                msg_send_void(provider, sel_registerName(c"release".as_ptr()));
                return None;
            }
            let item = msg_send_id(item_cls, sel_registerName(c"alloc".as_ptr()));
            let item = msg_send_id(item, sel_registerName(c"init".as_ptr()));
            if item.is_null() {
                msg_send_void(provider, sel_registerName(c"release".as_ptr()));
                return None;
            }
            #[allow(clashing_extern_declarations)]
            #[link(name = "objc")]
            extern "C" {
                #[link_name = "objc_msgSend"]
                fn msg_send_id_id(
                    receiver: *mut c_void,
                    sel: *mut c_void,
                    a1: *mut c_void,
                    a2: *mut c_void,
                ) -> *mut c_void;
            }
            let types = promise_types_array(&[
                // 承诺公开类型（utf8 纯文本——队列条目文本快照载面）。
                "public.utf8-plain-text",
            ]);
            msg_send_id_id(
                item,
                sel_registerName(c"setDataProvider:forTypes:".as_ptr()),
                provider,
                types,
            );
            // 三隐私标记空数据瞬时写定（阻止 Spotlight/Chromium/自家守护）。
            let empty = msg_send_id_ptr_len(
                objc_getClass(c"NSData".as_ptr()),
                sel_registerName(c"dataWithBytes:length:".as_ptr()),
                std::ptr::null(),
                0,
            );
            for marker in PROMISE_MARKERS {
                let m = ns_string(marker);
                if !m.is_null() && !empty.is_null() {
                    msg_send_void_id_isize(
                        item,
                        sel_registerName(c"setData:forType:".as_ptr()),
                        empty,
                        m as isize,
                    );
                }
            }
            // 本机化新建（返回新 changeCount）+ 承诺 item 落板。
            let host_sel = sel_registerName(c"prepareForNewContents:".as_ptr());
            let accepts = msg_send_bool_arg(
                pb,
                sel_registerName(c"respondsToSelector:".as_ptr()),
                host_sel,
            );
            let count: usize = if accepts {
                msg_send_usize_arg(pb, host_sel, PB_CONTENTS_CURRENT_HOST_ONLY)
            } else {
                // 老 API 兜底（全宿主域——SCN-QUEUE-CURRENT-HOST-ONLY 偏差登记）。
                msg_send_void(pb, sel_registerName(c"clearContents".as_ptr()));
                msg_send_usize(pb, sel_registerName(c"changeCount".as_ptr()))
            };
            // 单对象经 NSMutableArray 包裹后写板（R184 F-183-01）：裸
            // NSPasteboardItem* 直传 writeObjects: 无数组语义（历史崩溃面）
            // ——pasteboard_write_text 同型承载，item 参照语义不变。
            let arr = promise_types_array(&[]);
            msg_send_void_id_isize(arr, sel_registerName(c"addObject:".as_ptr()), item, 0);
            msg_send_void_id_isize(pb, sel_registerName(c"writeObjects:".as_ptr()), arr, 0);
            // 写入板面后：item 持强引用 provider——我们的 alloc 参照转交。
            msg_send_void(provider, sel_registerName(c"release".as_ptr()));
            msg_send_void(item, sel_registerName(c"release".as_ptr()));
            Some(count as u64)
        })
    }
}

/// 真实数据写板（stop 补写 / 回贴同型）：clearContents → writeObjects:
/// [NSString]（公开纯文本落板）。**主线程专用**。false = 桥失败（调用方
/// stderr 披露）。
pub fn pasteboard_write_text(text: &str) -> bool {
    unsafe {
        ensure_appkit_loaded();
        autorelease_drain(|| {
            let pb = general_pasteboard();
            if pb.is_null() {
                return false;
            }
            let s = ns_string(text);
            if s.is_null() {
                return false;
            }
            msg_send_void(pb, sel_registerName(c"clearContents".as_ptr()));
            let arr = promise_types_array(&[]);
            msg_send_void_id_isize(arr, sel_registerName(c"addObject:".as_ptr()), s, 0);
            msg_send_void_id_isize(pb, sel_registerName(c"writeObjects:".as_ptr()), arr, 0);
            true
        })
    }
}

/// 最近一次应用激活时刻槽（NSWorkspace didActivateApplication 桥写入面；
/// 前台引擎拍判定读点）。None = 桥未注册/进程刚启无激活事件。
pub fn pasteboard_activation_slot() -> &'static Mutex<Option<Instant>> {
    &PROMISE_SLOT.last_activity_activation
}

/// activation watch 哑类（NSNotificationCenter target-action 残式）：无状态
/// 纯转发（通知到槽只写时刻）——target 由通知中心 retain，进程级单例。
static WATCH_CLASS: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
static WATCH_REGISTERED: AtomicBool = AtomicBool::new(false);

/// didActivate 回调（NSWorkspace 通知中心主线程 post）：只写时刻，不做
/// 业务（同 provider 回调纪律）。
unsafe extern "C" fn activation_note_imp(
    _self: *mut c_void,
    _sel: *mut c_void,
    _note: *mut c_void,
) {
    if let Ok(mut at) = pasteboard_activation_slot().lock() {
        *at = Some(Instant::now());
    }
}

/// workspace activation watch 哑类注册（幂等）。
unsafe fn activation_watch_class() -> *mut c_void {
    let cls = *WATCH_CLASS.get_or_init(|| unsafe {
        ensure_appkit_loaded();
        let super_cls = objc_getClass(c"NSObject".as_ptr());
        if super_cls.is_null() {
            return 0usize;
        }
        let cls = objc_allocateClassPair(super_cls, c"PRCWorkspaceActivationWatcher".as_ptr(), 0);
        if cls.is_null() {
            return 0usize;
        }
        class_addMethod(
            cls,
            sel_registerName(c"activatedNote:".as_ptr()),
            activation_note_imp as *const () as usize,
            c"v@:@".as_ptr(),
        );
        objc_registerClassPair(cls);
        cls as usize
    });
    if cls == 0 {
        std::ptr::null_mut()
    } else {
        cls as *mut c_void
    }
}

/// 应用激活事件侦听注册（SCN-QUEUE-EXPLORE-BLOCKED 事件级精度承载源 ·
/// NSWorkspace notificationCenter didActivateApplicationNotification——
/// 事件级精度满足 0.3s 探查窗；0.5s 守护拍太粗已裁定）。幂等：重复调用
/// 直出首次结果。注册失败 false（探查窗判定无源——note_demand 保守判非
/// 探查，队列主流程不受影响）。
pub fn pasteboard_watch_activations() -> bool {
    if WATCH_REGISTERED.load(Ordering::Acquire) {
        return true;
    }
    let ok = unsafe {
        ensure_appkit_loaded();
        autorelease_drain(|| {
            let cls = activation_watch_class();
            if cls.is_null() {
                return false;
            }
            let obs = msg_send_id(cls, sel_registerName(c"alloc".as_ptr()));
            let obs = msg_send_id(obs, sel_registerName(c"init".as_ptr()));
            if obs.is_null() {
                return false;
            }
            let ws_cls = objc_getClass(c"NSWorkspace".as_ptr());
            if ws_cls.is_null() {
                return false;
            }
            let ws = msg_send_id(ws_cls, sel_registerName(c"sharedWorkspace".as_ptr()));
            if ws.is_null() {
                return false;
            }
            let center = msg_send_id(ws, sel_registerName(c"notificationCenter".as_ptr()));
            if center.is_null() {
                return false;
            }
            let name = ns_string("NSWorkspaceDidActivateApplicationNotification");
            if name.is_null() {
                return false;
            }
            #[allow(clashing_extern_declarations)]
            #[link(name = "objc")]
            extern "C" {
                #[link_name = "objc_msgSend"]
                fn msg_send_void_id4(
                    receiver: *mut c_void,
                    sel: *mut c_void,
                    a1: *mut c_void,
                    a2: *mut c_void,
                    a3: *mut c_void,
                    a4: *mut c_void,
                );
            }
            msg_send_void_id4(
                center,
                sel_registerName(c"addObserver:selector:name:object:".as_ptr()),
                obs,
                sel_registerName(c"activatedNote:".as_ptr()),
                name,
                ws,
            );
            // 观察者由通知中心 retain（本进程全生命周期在册）——alloc 参照
            // 不需释放（引用转交 center）。
            true
        })
    };
    WATCH_REGISTERED.store(ok, Ordering::Release);
    ok
}

/// 图片载荷 → 最长边 `max_edge` px 的 PNG 缩略字节（SCN-CAPTURE-THUMB ·
/// AppKit 框架显式装载（幂等 · Once 护栏）：NSImage/NSBitmapImageRep 属
/// AppKit（Foundation 不含），生产进程由 gpui 平台层先行装载；无 gpui 装配
/// 的进程态（cargo test 二进制）须 dlopen 后 objc_getClass 方可命中。生产侧
/// 重复 dlopen 为引用计数增量，无害；RTLD_NOW 在护栏内一次绑定全部符号
///（并发惰性绑定有竞态面）。
unsafe fn ensure_appkit_loaded() {
    use std::sync::Once;
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        extern "C" {
            fn dlopen(path: *const std::ffi::c_char, mode: i32) -> *mut c_void;
        }
        const RTLD_NOW: i32 = 2;
        dlopen(
            c"/System/Library/Frameworks/AppKit.framework/AppKit".as_ptr(),
            RTLD_NOW,
        );
    });
}

/// 图片载荷 → 最长边 `max_edge` px 的 PNG 缩略字节（SCN-CAPTURE-THUMB ·
/// DEC-188）：NSBitmapImageRep 直读原始数据取像素尺寸（PNG/TIFF
/// 全格式面）；超限经 NSBitmapImageRep + NSGraphicsContext 精确像素重采样
///（点＝像素 1:1，无 lockFocus 的主屏 Retina 背衬倍增），不超限直接 PNG 重
/// 编码（无损保留）。任一环节失败 → None（调用方回退棋盘占位行，不虚造）。
/// 函数体自带 autorelease pool（同 rtf_to_plain_text 口径）＋ AppKit 显式装载
///（ensure_appkit_loaded）。**主线程专用**（AppKit 惯例 · 捕获链主拍承载——
/// 缩略生成即拍即转）。纯 C FFI（libobjc · 零新增依赖 · 规范 9）。
pub fn image_thumbnail_png(bytes: &[u8], max_edge: u32) -> Option<Vec<u8>> {
    if bytes.is_empty() {
        return None;
    }
    unsafe {
        autorelease_drain(|| {
            ensure_appkit_loaded();
            // 原始尺寸直读：NSBitmapImageRep initWithData（PNG/TIFF 等全位图格式面）。
            let rep_cls = objc_getClass(c"NSBitmapImageRep".as_ptr());
            if rep_cls.is_null() {
                return None;
            }
            let data = thumb_data_of(bytes)?;
            let rep0 = msg_send_id(rep_cls, sel_registerName(c"alloc".as_ptr()));
            let rep0 = msg_send_id_id(rep0, sel_registerName(c"initWithData:".as_ptr()), data);
            if rep0.is_null() {
                return None;
            }
            let wide = msg_send_usize(rep0, sel_registerName(c"pixelsWide".as_ptr()));
            let high = msg_send_usize(rep0, sel_registerName(c"pixelsHigh".as_ptr()));
            let max = wide.max(high);
            let out = if max == 0 || (max as u64) > (u32::MAX as u64) {
                None
            } else if max as u32 <= max_edge {
                // 不超限：原尺寸 PNG 重编码（无损保留）。
                png_data_of_rep(rep0)
            } else {
                // 超限：NSImage 等比重采样到最长边 max_edge（经典缩略技法）。
                resize_thumb_png(bytes, max_edge, wide, high)
            };
            msg_send_void(rep0, sel_registerName(c"release".as_ptr()));
            out
        })
    }
}

/// 超限重采样（image_thumbnail_png 内部分支）：NSBitmapImageRep 预建目标像素
/// 尺寸的位图面（8bpc·RGBA）→ NSGraphicsContext graphicsContextWithBitmapImageRep
/// → save/setCurrent → drawInRect 绘制 → restore → representationUsingType PNG。
/// 精确像素控制路径（点＝像素 1:1，无 lockFocus 的主屏 Retina 背衬倍增）；
/// alloc/init 配对手动 release；NSCompositingOperationSourceOver=2；
/// NSCalibratedRGBColorSpace 经 NSString stringWithUTF8String 承载。
unsafe fn resize_thumb_png(
    bytes: &[u8],
    max_edge: u32,
    wide: usize,
    high: usize,
) -> Option<Vec<u8>> {
    #[allow(clashing_extern_declarations)]
    #[link(name = "objc")]
    extern "C" {
        #[link_name = "objc_msgSend"]
        fn msg_send_rep_init(
            receiver: *mut c_void,
            sel: *mut c_void,
            planes: *mut c_void,
            wide: usize,
            high: usize,
            bps: usize,
            spp: usize,
            alpha: bool,
            planar: bool,
            cs: *mut c_void,
            bpr: usize,
            bpp: usize,
        ) -> *mut c_void;
        #[link_name = "objc_msgSend"]
        fn msg_send_void_rifr(
            receiver: *mut c_void,
            sel: *mut c_void,
            dst: NsRect,
            src: NsRect,
            op: usize,
            fraction: f64,
        );
    }
    let img_cls = objc_getClass(c"NSImage".as_ptr());
    let rep_cls = objc_getClass(c"NSBitmapImageRep".as_ptr());
    let ctx_cls = objc_getClass(c"NSGraphicsContext".as_ptr());
    if img_cls.is_null() || rep_cls.is_null() || ctx_cls.is_null() {
        return None;
    }
    let data = thumb_data_of(bytes)?;
    let src = msg_send_id(img_cls, sel_registerName(c"alloc".as_ptr()));
    let src = msg_send_id_id(src, sel_registerName(c"initWithData:".as_ptr()), data);
    if src.is_null() {
        return None;
    }
    let factor = max_edge as f64 / wide.max(high) as f64;
    let (nw, nh) = (
        (wide as f64 * factor).round() as usize,
        (high as f64 * factor).round() as usize,
    );
    let cs = msg_send_id_ptr(
        objc_getClass(c"NSString".as_ptr()),
        sel_registerName(c"stringWithUTF8String:".as_ptr()),
        c"NSCalibratedRGBColorSpace".as_ptr(),
    );
    let rep = if cs.is_null() {
        std::ptr::null_mut()
    } else {
        msg_send_rep_init(
            msg_send_id(rep_cls, sel_registerName(c"alloc".as_ptr())),
            sel_registerName(c"initWithBitmapDataPlanes:pixelsWide:pixelsHigh:bitsPerSample:samplesPerPixel:hasAlpha:isPlanar:colorSpaceName:bytesPerRow:bitsPerPixel:".as_ptr()),
            std::ptr::null_mut(),
            nw,
            nh,
            8,
            4,
            true,
            false,
            cs,
            0,
            0,
        )
    };
    let ctx = if rep.is_null() {
        std::ptr::null_mut()
    } else {
        msg_send_id_id(
            ctx_cls,
            sel_registerName(c"graphicsContextWithBitmapImageRep:".as_ptr()),
            rep,
        )
    };
    let out = if ctx.is_null() {
        if !rep.is_null() {
            msg_send_void(rep, sel_registerName(c"release".as_ptr()));
        }
        None
    } else {
        msg_send_void(ctx_cls, sel_registerName(c"saveGraphicsState".as_ptr()));
        msg_send_void_id(
            ctx_cls,
            sel_registerName(c"setCurrentContext:".as_ptr()),
            ctx,
        );
        msg_send_void_rifr(
            src,
            sel_registerName(c"drawInRect:fromRect:operation:fraction:".as_ptr()),
            NsRect {
                x: 0.0,
                y: 0.0,
                w: nw as f64,
                h: nh as f64,
            },
            NsRect {
                x: 0.0,
                y: 0.0,
                w: 0.0,
                h: 0.0,
            }, // NSZeroRect（全图自适配）
            2, // NSCompositingOperationSourceOver
            1.0,
        );
        msg_send_void(ctx_cls, sel_registerName(c"restoreGraphicsState".as_ptr()));
        let png = png_data_of_rep(rep);
        msg_send_void(rep, sel_registerName(c"release".as_ptr()));
        png
    };
    msg_send_void(src, sel_registerName(c"release".as_ptr()));
    out
}

/// 字节 → NSData（类便利构造器 · autorelease 对象，由调用线程 pool 排水）。
unsafe fn thumb_data_of(bytes: &[u8]) -> Option<*mut c_void> {
    let data_cls = objc_getClass(c"NSData".as_ptr());
    if data_cls.is_null() {
        return None;
    }
    let data = msg_send_id_ptr_len(
        data_cls,
        sel_registerName(c"dataWithBytes:length:".as_ptr()),
        bytes.as_ptr() as *const c_void,
        bytes.len(),
    );
    if data.is_null() {
        None
    } else {
        Some(data)
    }
}

/// NSBitmapImageRep → PNG 字节（NSBitmapImageFileTypePNG=4 · 空 props 字典）。
unsafe fn png_data_of_rep(rep: *mut c_void) -> Option<Vec<u8>> {
    #[allow(clashing_extern_declarations)]
    #[link(name = "objc")]
    extern "C" {
        #[link_name = "objc_msgSend"]
        fn msg_send_id_uprop(
            receiver: *mut c_void,
            sel: *mut c_void,
            ty: usize,
            props: *mut c_void,
        ) -> *mut c_void;
    }
    let dict_cls = objc_getClass(c"NSDictionary".as_ptr());
    if dict_cls.is_null() {
        return None;
    }
    let empty = msg_send_id(dict_cls, sel_registerName(c"dictionary".as_ptr()));
    if empty.is_null() {
        return None;
    }
    let png = msg_send_id_uprop(
        rep,
        sel_registerName(c"representationUsingType:properties:".as_ptr()),
        4,
        empty,
    );
    ns_data_bytes(png)
}

/// 前台应用 bundleId（REQ-CAPTURE-EXCLUDE 排除判定源 · NSWorkspace 只读探针）。
/// None = 桥失败 / 无前台应用（守护进程态）。**主线程专用**（NSWorkspace 惯例）。
pub fn frontmost_bundle_id() -> Option<String> {
    unsafe {
        let ws_cls = objc_getClass(c"NSWorkspace".as_ptr());
        if ws_cls.is_null() {
            return None;
        }
        let ws = msg_send_id(ws_cls, sel_registerName(c"sharedWorkspace".as_ptr()));
        if ws.is_null() {
            return None;
        }
        let app = msg_send_id(ws, sel_registerName(c"frontmostApplication".as_ptr()));
        if app.is_null() {
            return None;
        }
        let bid = msg_send_id(app, sel_registerName(c"bundleIdentifier".as_ptr()));
        if bid.is_null() {
            return None;
        }
        let raw = msg_send_utf8(bid, sel_registerName(c"UTF8String".as_ptr()));
        if raw.is_null() {
            return None;
        }
        let s = std::ffi::CStr::from_ptr(raw).to_string_lossy().into_owned();
        if s.is_empty() {
            None
        } else {
            Some(s)
        }
    }
}

#[cfg(test)]
mod status_item_tests {
    use super::*;

    /// 点击旗标排水语义：置位 → 首 take 取走 → 再 take 幂等为空。
    #[test]
    fn status_click_take_drains_once() {
        STATUS_CLICK.store(true, Ordering::Release);
        assert!(status_click_take(), "置位后首次 take 应取走");
        assert!(!status_click_take(), "take 幂等排水（二次为空）");
        assert!(!STATUS_CLICK.load(Ordering::Acquire), "排水后回零");
    }

    /// 退出旗标排水语义：同上幂等排水。
    #[test]
    fn status_quit_take_drains_once() {
        STATUS_QUIT.store(true, Ordering::Release);
        assert!(status_quit_take(), "置位后首次 take 应取走");
        assert!(!status_quit_take(), "take 幂等排水（二次为空）");
        assert!(!STATUS_QUIT.load(Ordering::Acquire), "排水后回零");
    }

    /// 掩码常量语义：左键 up（1<<2）+右键 down（1<<3）双位在册（NSEventMask
    /// = 1<<NSEventType；LeftMouseUp=2 → 4 · RightMouseDown=3 → 8）。
    #[test]
    fn status_send_action_mask_is_left_up_or_right_down() {
        assert_eq!(STATUS_SEND_ACTION_MASK, 12);
    }

    /// 菜单开合旗标读数语义（只读不清零）：置位可读真 →
    /// 读后旗标保持「开」态（多次读同真，区别于 take 系排水）→ 显式清零后读伪。
    #[test]
    fn status_menu_open_flag_read_does_not_drain() {
        STATUS_MENU_OPEN.store(true, Ordering::Release);
        assert!(status_menu_open(), "置位后读真");
        assert!(status_menu_open(), "只读：重复读仍真（不排水）");
        STATUS_MENU_OPEN.store(false, Ordering::Release);
        assert!(!status_menu_open(), "清零后读伪");
    }
}

#[cfg(test)]
mod media_bridges_tests {
    use super::*;

    /// RTF → 纯文本（DEC-188）：NSAttributedString 空字典
    /// 嗅探路径把控制词/字体表全部剥离，仅留可读正文（非空 · 无 RTF 控制串）。
    #[test]
    fn rtf_to_plain_text_strips_control_words() {
        let rtf = b"{\\rtf1\\ansi\\deff0 {\\fonttbl{\\f0 Helvetica;}}\\f0\\fs24 \\b bold\\b0 text}";
        let plain = rtf_to_plain_text(rtf).expect("NSAttributedString 转换应成功");
        assert!(plain.contains("bold"), "正文应保留：{plain}");
        assert!(plain.contains("text"), "正文应保留：{plain}");
        assert!(!plain.contains('\\'), "控制词应剥除：{plain}");
    }

    /// RTF 转换失败面：空字节 → None（不虚造）。
    #[test]
    fn rtf_to_plain_text_empty_is_none() {
        assert!(rtf_to_plain_text(b"").is_none(), "空字节应 None");
    }

    /// 品牌稿缩略（DEC-188）：1024px 品牌稿缩到最长边
    /// 320px —— 产物为合法 PNG 且 IHDR 尺寸等比收到 320 以内（重采样发生）。
    #[test]
    fn image_thumbnail_png_scales_large_png_down() {
        let src = include_bytes!("../../parchora-app/assets/parchora-logo-512.png");
        let thumb = image_thumbnail_png(src, 320).expect("缩略应成功");
        assert_eq!(&thumb[0..4], b"\x89PNG", "产物应为 PNG（magic头）");
        // PNG IHDR：字节 16..20 与 20..24 大端 u32 宽/高（固定偏移，确定性断言）。
        let w = u32::from_be_bytes([thumb[16], thumb[17], thumb[18], thumb[19]]);
        let h = u32::from_be_bytes([thumb[20], thumb[21], thumb[22], thumb[23]]);
        assert!(w.max(h) <= 320, "最长边应收到 320 内，实际 {w}x{h}");
        // 不超限（max_edge ≥ 原尺寸）→ 原尺寸无损 PNG 重编码（IHDR 保持原尺寸）。
        let pass_through = image_thumbnail_png(src, u32::MAX).expect("不超限重编码应成功");
        assert_eq!(&pass_through[0..4], b"\x89PNG");
        let pw = u32::from_be_bytes([
            pass_through[16],
            pass_through[17],
            pass_through[18],
            pass_through[19],
        ]);
        let ph = u32::from_be_bytes([
            pass_through[20],
            pass_through[21],
            pass_through[22],
            pass_through[23],
        ]);
        assert_eq!(pw.max(ph), 1024, "不超限应保持原尺寸 {pw}x{ph}");
    }

    /// 非法字节面：空字节 → None（不虚造、不 panic）。
    #[test]
    fn image_thumbnail_png_empty_is_none() {
        assert!(image_thumbnail_png(b"", 320).is_none());
    }
}

/// 队列提示桥槽语义测试（TASK-405 · 同 status 槽口径）。
#[cfg(test)]
mod notify_bridge_tests {
    use super::*;

    /// 点击旗标排水语义：系统置位 → 首 take 取走 → 再 take 幂等为空。
    #[test]
    fn notification_click_take_drains_once() {
        NOTIF_CLICK.store(true, Ordering::Release);
        assert!(notification_click_take(), "置位后首次 take 应取走");
        assert!(!notification_click_take(), "take 幂等排水（二次为空）");
    }

    /// 授权三态读点：未请求/未决读伪；置位后读真；拒绝态读伪。
    #[test]
    fn notification_authorized_reads_shared_state() {
        NOTIF_AUTH.store(0, Ordering::Release);
        assert!(!notification_authorized(), "未决读伪");
        NOTIF_AUTH.store(2, Ordering::Release);
        assert!(!notification_authorized(), "拒绝读伪");
        NOTIF_AUTH.store(1, Ordering::Release);
        assert!(notification_authorized(), "已授权读真");
    }
}

// ═════════════════════════ 队列提示桥（TASK-405 · DEC-44/§11.2）═════════════════════════

/* ───── 用户注意请求（弹一弹 · Dock 图标）───── */

/// NSApplication.requestUserAttention: InformationalRequest（§11.2 ①）：Dock 图
/// 标弹一弹，返回请求 id（NSRequestID——撤回用）。NSInformationalRequest=10
///（NSRequestUserAttentionType · AppKit 头定值）；CriticalRequest 不用（§11.2
/// 「避免持续弹跳骚扰」）。应用激活时系统自动取消请求（Apple 语义）。
/// NSApplication 缺失/共享实例不可得 → None（防御不崩溃）。纯 C FFI
///（libobjc · 零第三方依赖 · 规范 9）。主线程专用（AppKit 惯例）。
pub fn request_user_attention_informational() -> Option<u64> {
    unsafe {
        ensure_appkit_loaded();
        let app_cls = objc_getClass(c"NSApplication".as_ptr());
        if app_cls.is_null() {
            return None;
        }
        let app = msg_send_id(app_cls, sel_registerName(c"sharedApplication".as_ptr()));
        if app.is_null() {
            return None;
        }
        // NSInformationalRequest = 10（NSRequestUserAttentionType 枚举值 ·
        // AppKit 头定数）；返回 NSRequestID（NSUInteger）。
        let id = msg_send_usize_arg(app, sel_registerName(c"requestUserAttention:".as_ptr()), 10);
        if id == 0 {
            return None;
        }
        Some(id as u64)
    }
}

/// 撤回未决用户注意请求（NSApplication.cancelUserAttentionRequest: · design
/// §2.3 表行同款）；id 无效/请求已兑现 = 无害 no-op（Apple 语义）。
pub fn cancel_user_attention_request(id: u64) {
    #[allow(clashing_extern_declarations)]
    #[link(name = "objc")]
    extern "C" {
        #[link_name = "objc_msgSend"]
        fn msg_send_void_usize_arg(receiver: *mut c_void, sel: *mut c_void, arg1: usize);
    }
    unsafe {
        ensure_appkit_loaded();
        let app_cls = objc_getClass(c"NSApplication".as_ptr());
        if app_cls.is_null() {
            return;
        }
        let app = msg_send_id(app_cls, sel_registerName(c"sharedApplication".as_ptr()));
        if app.is_null() {
            return;
        }
        msg_send_void_usize_arg(
            app,
            sel_registerName(c"cancelUserAttentionRequest:".as_ptr()),
            id as usize,
        );
    }
}

/* ───── App Nap 对策（DEC-210 · debug/release 定时器精度一致性）───── */

/// NSActivityOptions 位常量（Foundation NSProcessInfo.h 原式照抄 · 本机 SDK
/// 头文件查证）：IdleSystemSleepDisabled = 1ULL << 20。
const NS_ACTIVITY_IDLE_SYSTEM_SLEEP_DISABLED: u64 = 1 << 20;
/// NSActivityUserInitiated（头文件原式：0x00FFFFFF | IdleSystemSleepDisabled）。
const NS_ACTIVITY_USER_INITIATED: u64 = 0x00FF_FFFF | NS_ACTIVITY_IDLE_SYSTEM_SLEEP_DISABLED;
/// NSActivityUserInitiatedAllowingIdleSystemSleep（头文件原式取模）：App Nap /
/// 计时器合并压制档，同时允许系统空闲休眠（后台常驻工具不拦截系统休眠 ·
/// DEC-210 取此档，不用含 IdleSystemSleepDisabled 的 UserInitiated 全档）。
const NS_ACTIVITY_USER_INITIATED_ALLOWING_IDLE_SYSTEM_SLEEP: u64 =
    NS_ACTIVITY_USER_INITIATED & !NS_ACTIVITY_IDLE_SYSTEM_SLEEP_DISABLED;

// App Nap 桥接消息变体（同符号多签名声明 · 与上方各节同语义放行；仅本节消费，
// 不对外暴露）：(usize,id)→id 承载 NSProcessInfo beginActivityWithOptions:reason:。
#[allow(clashing_extern_declarations)]
#[link(name = "objc")]
extern "C" {
    #[link_name = "objc_msgSend"]
    fn msg_send_id_usize_obj(
        receiver: *mut c_void,
        sel: *mut c_void,
        arg1: usize,
        arg2: *mut c_void,
    ) -> *mut c_void;
}

/// App Nap 对策挂接（DEC-210）：进程级常驻 NSActivityUserInitiatedAllowingIdle
/// SystemSleep activity——主面板关闭（无可见窗口）后，系统对后台进程的 App Nap
/// / 计时器合并不再放宽本进程定时器节拍；唤出守护 0.05s 轮询与 tap 事件锁存
/// 排水在 release 载体（LSUIElement 打包应用）与 cargo run（Terminal 托管进程）
/// 之间行为一致（用户面「debug 比 release 灵敏」根因收口）。允许系统空闲休
/// 眠；不调用 endActivity——activity 对象显式 retain 后随进程生命周期常驻。
/// 挂接失败（类缺失/消息失败）静默降级为系统默认节拍（真机腿登记：release
/// 双载体复测唤出延迟）。仅 main() 启动单调用，无幂等护栏需求。
pub fn disable_app_nap() {
    unsafe {
        ensure_appkit_loaded();
        // NSProcessInfo 属 Foundation（AppKit 传递装载）；类查找失败 → 降级退出。
        let pi_cls = objc_getClass(c"NSProcessInfo".as_ptr());
        if pi_cls.is_null() {
            return;
        }
        let pi = msg_send_id(pi_cls, sel_registerName(c"processInfo".as_ptr()));
        if pi.is_null() {
            return;
        }
        let reason = ns_string("Parchora 全局唤出守护需低延迟定时节拍（App Nap 对策 · DEC-210）");
        let token = msg_send_id_usize_obj(
            pi,
            sel_registerName(c"beginActivityWithOptions:reason:".as_ptr()),
            NS_ACTIVITY_USER_INITIATED_ALLOWING_IDLE_SYSTEM_SLEEP as usize,
            reason,
        );
        if token.is_null() {
            return;
        }
        // beginActivity 返回对象按 Cocoa 惯例 autoreleased（MRC 口径），池排空即
        // 失效——显式 retain 后永不释放 = 活动进程级常驻（Apple endActivity 语义
        // 的常驻变体；仅此一对象，量级可忽略）。
        let _kept = msg_send_id(token, sel_registerName(c"retain".as_ptr()));
    }
}

/* ───── 系统通知（UserNotifications · UNUserNotificationCenter）───── */

/// 授权三态（Atomic · 回调线程写任意线程读）：0=未请求/未决 · 1=已授权 ·
/// 2=拒绝。未决/拒绝 → 通知静默禁用（§11.2）；门在 send 侧统一承载。
static NOTIF_AUTH: AtomicU8 = AtomicU8::new(0);
/// 通知点击槽（didReceiveNotificationResponse 置位 · 宿主拍排水消费）。
static NOTIF_CLICK: AtomicBool = AtomicBool::new(false);
/// 已投递通知计数（identifier 唯一性承载 · 静态递增）。
static NOTIF_SEQ: AtomicU64 = AtomicU64::new(0);
/// delegate 注册护栏（幂等——setDelegate 重复调用同实例无害，不需标志位）。
/// delegate 类对（OnceLock 幂等注册 · 同 PROVIDER_CLASS 口径）。
static NOTIF_CLASS: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
/// delegate 实例（OnceLock 单例——setDelegate 强引用后进程级在期，不再释放）。
static NOTIF_DELEGATE: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
/// UserNotifications 框架显式装载护栏（幂等 · 同 ensure_appkit_loaded 口径）。
static UN_LOADED: std::sync::Once = std::sync::Once::new();

// ObjC block ABI（零依赖手写 · 规范 9）：无捕获块 = [isa, flags:i32,
// reserved:i32, invoke, descriptor{reserved,size}]（无 copy/dispose 助手——
// HAS_COPY_DISPOSE 未置位的编译器原生形）。系统 async API 持块时自行
// _Block_copy（无捕获栈块 = 全量可 memcopy · 生态最常见块型，运行时原生处理）。
#[repr(C)]
struct BlockDescriptor {
    reserved: usize,
    size: usize,
}
#[repr(C)]
struct BlockLiteral<F> {
    isa: *const c_void,
    flags: u32,
    reserved: u32,
    invoke: F,
    descriptor: &'static BlockDescriptor,
}

// 系统给侧 block 调用器（读第 4 机器字 invoke 指针直呼）——willPresent/didReceive
// 的 completionHandler 回传专用（不建块只调块）。
unsafe fn block_invoke_1(block: *mut c_void, a1: usize) {
    if block.is_null() {
        return;
    }
    #[repr(C)]
    struct Block1 {
        _isa: *const c_void,
        _flags: u32,
        _reserved: u32,
        invoke: unsafe extern "C" fn(*mut c_void, usize),
    }
    let b = block as *const Block1;
    ((*b).invoke)(block, a1);
}
unsafe fn block_invoke_0(block: *mut c_void) {
    if block.is_null() {
        return;
    }
    #[repr(C)]
    struct Block0 {
        _isa: *const c_void,
        _flags: u32,
        _reserved: u32,
        invoke: unsafe extern "C" fn(*mut c_void),
    }
    let b = block as *const Block0;
    ((*b).invoke)(block);
}

/// UserNotifications.framework 显式装载（dlopen 幂等 · 同 ensure_appkit_loaded
/// 语义；UserNotifications 依赖 Foundation——已由 AppKit 链装）。
unsafe fn ensure_usernotifications_loaded() {
    UN_LOADED.call_once(|| {
        extern "C" {
            fn dlopen(path: *const std::ffi::c_char, mode: i32) -> *mut c_void;
        }
        const RTLD_NOW: i32 = 2;
        dlopen(
            c"/System/Library/Frameworks/UserNotifications.framework/UserNotifications".as_ptr(),
            RTLD_NOW,
        );
    });
}

// delegate 系统回调头两 IMP（回调线程任意——只写共享原子，零业务零 gpui）：
// ① willPresent：前台呈现门——面板可能 key（队列运行时用户常在前台），缺省
//   不呈现会被吞（§11.2 要求 banner+list）。呈现位用 badge|alert 集合（位基序
//   【查证不确定】——badge/alert 双位并集对两种基序都覆盖 alert，badge 无值
//   no-op、无声音；真机腿验证呈现效果 · T-Q18 同批）。
unsafe extern "C" fn notif_will_present_imp(
    _self: *mut c_void,
    _sel: *mut c_void,
    _center: *mut c_void,
    _notification: *mut c_void,
    handler: *mut c_void,
) {
    // UNNotificationPresentationOptions：badge(1<<0)|alert(1<<2) = 5——基序
    // 双覆盖（alert 单bit亦可达）。真机腿验证记录位随 stderr。
    block_invoke_1(handler, 5);
}
// ② didReceiveNotificationResponse：默认动作（点击横幅）→ 置点击槽。响应
//   判别（actionIdentifier default vs dismiss）不做——回调即点击
//  （dismissal 不上本 delegate · 系统 API 语义）。
unsafe extern "C" fn notif_did_receive_imp(
    _self: *mut c_void,
    _sel: *mut c_void,
    _center: *mut c_void,
    _response: *mut c_void,
    handler: *mut c_void,
) {
    NOTIF_CLICK.store(true, Ordering::Release);
    block_invoke_0(handler);
}

// 授权回调（异步完成块 invoke——回调线程任意）：granted → 1 / 拒绝 → 2；
// error 非空（granted=false 伴生）→ stderr 防御披露一次。
unsafe extern "C" fn notif_auth_cb_invoke(_block: *mut c_void, granted: bool, err: *mut c_void) {
    NOTIF_AUTH.store(if granted { 1 } else { 2 }, Ordering::Release);
    if !err.is_null() {
        eprintln!("[parchora] 通知授权失败（拒绝/错误——通知通道静默，动效不受影响）");
    }
}

// add 完成回调（无操作占位——错误防御披露）。
unsafe extern "C" fn notif_add_cb_invoke(_block: *mut c_void, err: *mut c_void) {
    if !err.is_null() {
        eprintln!("[parchora] 通知投递失败（system 拒收——静默丢弃不重试）");
    }
}

// NO-CAPTURE 静态 descriptor（两字段版——无 HAS_COPY_DISPOSE；
// arm64 块字面量布局：isa8 + flags4 + reserved4 + invoke8 + desc8 = 32 字节）。
static NOTIF_BLOCK_DESC: BlockDescriptor = BlockDescriptor {
    reserved: 0,
    size: 32,
};

/// delegate 哑类注册（幂等 OnceLock · NSObject 子类两协议方法）。
unsafe fn notif_delegate_class() -> *mut c_void {
    let cls = *NOTIF_CLASS.get_or_init(|| unsafe {
        ensure_usernotifications_loaded();
        let super_cls = objc_getClass(c"NSObject".as_ptr());
        if super_cls.is_null() {
            return 0usize;
        }
        let cls = objc_allocateClassPair(super_cls, c"PRCUserNotifDelegate".as_ptr(), 0);
        if cls.is_null() {
            return 0usize;
        }
        class_addMethod(
            cls,
            sel_registerName(
                c"userNotificationCenter:willPresentNotification:withCompletionHandler:".as_ptr(),
            ),
            notif_will_present_imp as *const () as usize,
            c"v@:@@@@".as_ptr(),
        );
        class_addMethod(
            cls,
            sel_registerName(
                c"userNotificationCenter:didReceiveNotificationResponse:withCompletionHandler:"
                    .as_ptr(),
            ),
            notif_did_receive_imp as *const () as usize,
            c"v@:@@@@".as_ptr(),
        );
        objc_registerClassPair(cls);
        cls as usize
    });
    if cls == 0 {
        std::ptr::null_mut()
    } else {
        cls as *mut c_void
    }
}

/// UNUserNotificationCenter 现取＋delegate 挂装（幂等——delegate set 一次）。
/// 裸二进制（无 bundle）/类不可得 → null（调用方短路：不崩溃不请求）。函数体
/// 自带 autorelease pool（通知对象创建链 AppKit 同池语义）。
unsafe fn notification_center() -> *mut c_void {
    ensure_usernotifications_loaded();
    autorelease_drain(|| {
        let cls = objc_getClass(c"UNUserNotificationCenter".as_ptr());
        if cls.is_null() {
            return std::ptr::null_mut();
        }
        let center = msg_send_id(cls, sel_registerName(c"currentNotificationCenter".as_ptr()));
        if center.is_null() {
            return std::ptr::null_mut();
        }
        // delegate 挂装（幂等——同实例重复 setDelegate 无害；OnceLock 单例）。
        let dcls = notif_delegate_class();
        if !dcls.is_null() {
            let inst = NOTIF_DELEGATE.get_or_init(|| unsafe {
                let obj = msg_send_id(dcls, sel_registerName(c"alloc".as_ptr()));
                let obj = msg_send_id(obj, sel_registerName(c"init".as_ptr()));
                obj as usize
            });
            if *inst != 0 {
                msg_send_void_id(
                    center,
                    sel_registerName(c"setDelegate:".as_ptr()),
                    *inst as *mut c_void,
                );
            }
        }
        center
    })
}

/// 发起通知授权请求（REQ-QUEUE-NOTIFY 授权语义 · 时机由调用方控制——队列首
/// 投递；不在应用启动时请求）。异步完成块（手写无捕获栈块——回调写共享三态）。
/// 中心不可得（裸二进制）→ 静默短路（不崩溃不请求）。重复调用 = 系统直答
/// 已决态（幂等无害）。
pub fn notification_request_authorization() {
    #[allow(clashing_extern_declarations)]
    #[link(name = "objc")]
    extern "C" {
        #[link_name = "objc_msgSend"]
        fn msg_send_void_usize_block(
            receiver: *mut c_void,
            sel: *mut c_void,
            a1: usize,
            a2: *mut c_void,
        );
    }
    unsafe {
        let center = notification_center();
        if center.is_null() {
            eprintln!("[parchora] 通知中心不可得（无 bundle——授权请求短路）");
            return;
        }
        // blocks isa 具体值不可得（理论不可达：libSystem 必带 blocks 运行时）
        // → 块无效，不再传给系统（防 _Block_copy 读空 isa 崩溃）。
        let isa = dlsym_or_null(c"_NSConcreteStackBlock".as_ptr());
        if isa.is_null() {
            eprintln!("[parchora] blocks 运行时探测失败——授权请求短路");
            return;
        }
        // UNAuthorizationOptions 全基位集 7（badge|sound|alert——基序不确定性下
        // 集合并集必达 alert+sound；badge 无值 no-op。真机腿验证）。
        let options: usize = 7;
        let invoke: unsafe extern "C" fn(*mut c_void, bool, *mut c_void) = notif_auth_cb_invoke;
        let block = BlockLiteral {
            isa,
            flags: 0,
            reserved: 0,
            invoke,
            descriptor: &NOTIF_BLOCK_DESC,
        };
        let block_ptr = &block
            as *const BlockLiteral<unsafe extern "C" fn(*mut c_void, bool, *mut c_void)>
            as *mut c_void;
        msg_send_void_usize_block(
            center,
            sel_registerName(c"requestAuthorizationWithOptions:completionHandler:".as_ptr()),
            options,
            block_ptr,
        );
    }
}

/// dlsym 缺省句柄查找（RTLD_DEFAULT）：blocks 运行时 isa 具体值取用；缺败 →
/// null（调用方短路）。
unsafe fn dlsym_or_null(symbol: *const std::ffi::c_char) -> *mut c_void {
    extern "C" {
        fn dlsym(handle: *mut c_void, symbol: *const std::ffi::c_char) -> *mut c_void;
    }
    let rtld_default = (-2isize) as *mut c_void; // RTLD_DEFAULT（macOS 语义值 -2）
    dlsym(rtld_default, symbol)
}

/// 立即投递系统通知（标题+正文 · C5 隐私口径由调用方承载——本层不裁内容）。
/// 未授权/未决（NOTIF_AUTH != 1）→ 静默 no-op（§11.2 通知通道静默禁用）。
/// 裸二进制 → 静默短路。add 完成块走手写无捕获栈块（错误 stderr 防御）。
pub fn notification_send_now(title: &str, body: &str) {
    #[allow(clashing_extern_declarations)]
    #[link(name = "objc")]
    extern "C" {
        #[link_name = "objc_msgSend"]
        fn msg_send_id_id_id_id(
            receiver: *mut c_void,
            sel: *mut c_void,
            a1: *mut c_void,
            a2: *mut c_void,
            a3: *mut c_void,
        ) -> *mut c_void;
    }
    unsafe {
        if NOTIF_AUTH.load(Ordering::Acquire) != 1 {
            return; // 未获授权 → 静默禁用
        }
        let center = notification_center();
        if center.is_null() {
            return;
        }
        let content_cls = objc_getClass(c"UNMutableNotificationContent".as_ptr());
        if content_cls.is_null() {
            return;
        }
        let content = msg_send_id(content_cls, sel_registerName(c"alloc".as_ptr()));
        let content = msg_send_id(content, sel_registerName(c"init".as_ptr()));
        if content.is_null() {
            return;
        }
        let t = ns_string(title);
        let b = ns_string(body);
        if t.is_null() || b.is_null() {
            // content 持 +1（alloc/init · R184 F-183-06 泄漏收口）：失败路径先
            // 释放再返回（t/b 为 stringWithUTF8String 自动释放面，不在此列）。
            msg_send_void(content, sel_registerName(c"release".as_ptr()));
            return;
        }
        msg_send_void_id(content, sel_registerName(c"setTitle:".as_ptr()), t);
        msg_send_void_id(content, sel_registerName(c"setBody:".as_ptr()), b);
        let req_cls = objc_getClass(c"UNNotificationRequest".as_ptr());
        if req_cls.is_null() {
            msg_send_void(content, sel_registerName(c"release".as_ptr()));
            return;
        }
        let seq = NOTIF_SEQ.fetch_add(1, Ordering::AcqRel);
        let ident = ns_string(&format!("parchora-queue-{seq}"));
        if ident.is_null() {
            msg_send_void(content, sel_registerName(c"release".as_ptr()));
            return;
        }
        let req = msg_send_id_id_id_id(
            req_cls,
            sel_registerName(c"requestWithIdentifier:content:trigger:".as_ptr()),
            ident,
            content,
            std::ptr::null_mut(),
        );
        if req.is_null() {
            msg_send_void(content, sel_registerName(c"release".as_ptr()));
            return;
        }
        // blocks isa 防御（同授权请求侧——不可得不传块）。
        let isa = dlsym_or_null(c"_NSConcreteStackBlock".as_ptr());
        if isa.is_null() {
            msg_send_void(content, sel_registerName(c"release".as_ptr()));
            return;
        }
        let invoke: unsafe extern "C" fn(*mut c_void, *mut c_void) = notif_add_cb_invoke;
        let block = BlockLiteral {
            isa,
            flags: 0,
            reserved: 0,
            invoke,
            descriptor: &NOTIF_BLOCK_DESC,
        };
        let block_ptr = &block
            as *const BlockLiteral<unsafe extern "C" fn(*mut c_void, *mut c_void)>
            as *mut c_void;
        #[allow(clashing_extern_declarations)]
        #[link(name = "objc")]
        extern "C" {
            #[link_name = "objc_msgSend"]
            fn msg_send_void_id_block(
                receiver: *mut c_void,
                sel: *mut c_void,
                a1: *mut c_void,
                a2: *mut c_void,
            );
        }
        msg_send_void_id_block(
            center,
            sel_registerName(c"addNotificationRequest:completionHandler:".as_ptr()),
            req,
            block_ptr,
        );
        // 成功路径收口（R184 F-183-06）：request 已持必要引用，本方 +1 释放。
        msg_send_void(content, sel_registerName(c"release".as_ptr()));
    }
}

/// 通知点击槽排水（Some=true=有未消费点击）：消费式（取走即清）——宿主拍
/// 读取路由。线程安全（回调任意线程写）。
pub fn notification_click_take() -> bool {
    NOTIF_CLICK.swap(false, Ordering::AcqRel)
}

/// 通知授权态读点（宿主编排门传入 · 生产读共享三态）。
pub fn notification_authorized() -> bool {
    NOTIF_AUTH.load(Ordering::Acquire) == 1
}
