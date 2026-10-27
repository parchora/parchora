//! parchora-app —— 进程编排（M5 Shell 装配 · TASK-501/711 演示宿主）。
//!
//! 窗口装配（对齐 TASK-501 口径 + DEC-66/67；演示宿主形态）：
//! - 主面板 320 宽 × 屏高 80% clamp 480~840（PG-SHELL 三视图；v3 固定宽 + 高度钳制）
//! - Peek 480 宽 × 屏高 70% clamp 480~780（PG-PEEK）
//! - 设置 430 × 560（PG-SET）· 菜单 300（PG-MENU 演示）· DG-ROOT（430 基准）
//! - 五窗统一 titlebar=None（隐藏标题栏，内容顶对齐窗口边缘；macOS styleMask=
//!   titled+fullSizeContentView，无红绿灯/无标题文本，is_movable 保持拖动）
//! - 置顶：五窗统一 kind=PopUp——vendor
//!   macOS GPUIPanel(NSPanel) + NonactivatingPanel + NSPopUpWindowLevel(101) +
//!   CanJoinAllSpaces + FullScreenAuxiliary（window.rs:1022-1257）：启动/后台/
//!   失焦/全屏覆盖/多显示器切换下均不被其它应用窗口遮挡，同层后序开窗仍压顶；
//!   系统模态（NSModalPanel level 8+）低于 PopUp 层级——空间上系统弹窗居中、
//!   面板贴缘，罕有交叠（登记局限）；非激活面板不夺焦但可收键
//!   （canBecomeKeyWindow=YES · Esc/编辑键入不受影响）
//! - window_min_size：面板 (360,480)（PANEL_W 承载 · DEC-129）· Peek (480,480)；底部触发位置互换后同步互换
//! - 触发位置（DEC-67/76 · DEC-84 · DEC-87）：默认=右侧；
//!   7 向停靠（底部/左侧/右侧/四角；四角恒可用）——主面板按位置贴缘/角停泊
//!   （左右缘垂直居中 · 底部宽高互换居中 · 四角双缘角锚），几何一律以应用所在屏
//!   的**可用区域**为参照（系统排除菜单栏/Dock 后的可用区 · platform
//!   NSScreen visibleFrame 真几何源 · screen_areas() 逐屏全帧+可用区双帧）；Dock
//!   占用同侧冲突自动切换（左→右、右→左、底→右 · resolve_conflict，启动/设置切位/
//!   屏帧变化三路均过）；可用性经 DockAvailability 全局落值 → 设置 pill 置灰；
//!   应用所在屏参照多显示器（active_display 探针 · DEC-76，全帧尺寸匹配屏帧）；收缩态 =
//!   主面板整窗关闭全隐（进程保持）；唤出 =
//!   可用区域边缘 12pt 触发热区（边族整缘 · 角族 4px 方区 CORNER_SZ ·
//!   DEC-100/222）轮询（边族扩带扩大有效触发范围；
//!   parchora_platform::cursor_position 只读探针 +
//!   dock::trigger_zone_bounds 判定域 · reveal_watch 0.05s 轮询 + 滑过立即唤出
//!   （REVEAL_CONFIRM_TICKS=1）+
//!   reveal_suppressed 门控：进带唤出、离带重臂，Esc 后滞留带内不回弹；收起域
//!   两拍 0.1s 防抖保持（CLOSE_CONFIRM_TICKS · DEC-83）；
//!   tap 快速滑过锁存（DEC-100：任意速度滑过经 CGEventTap 事件判入有效热区
//!   即置 ZONE_HIT，消轮询采样拍间漏检）；离界缓冲带（DEC-100：面板实界外
//!   扩 20pt 挂起收起，0.6s 长驻超限恢复常规——不瞬隐不闪烁）；
//!   守护仅停靠形态启动（is_docked_mode · DEC-80））；
//!   面板移出瞬间若光标仍在热区或离界缓冲带则不收（「光标仍在触发位置/界面
//!   边缘附近 → 显示」防抖眼不瞬隐 · DEC-100）；
//!   窗口级 hover 不承载（非 key 非激活 NSPanel 收不到 mouseMoved，
//!   vendor events.rs 不映射 NSMouseEntered + PopUp 分支无 setAcceptsMouseMovedEvents
//!   ——DEC-79）；位置变更 = 关窗重开（vendor 无编程式移窗 API）；
//!   （DEC-81）：CGEventTap 只听监听光标源（授权
//!   preflight 显式探测落 ListenAuth → 引导卡条件渲染；未授权保留读探针
//!   poll 降级 · REQ-PANEL-LISTEN-GRACEFUL 演示宿主偏差注记登记；设置引导卡
//!   点击 → 后台 CGRequestListenEventAccess，守护拍复探免重启恢复）、50pt
//!   排除带（dock::in_exclusion · 三处守卫同口径）、内缘照常唤出（DEC-83）、
//!   失活期面板移出仿真（panel_probe_armed · D-01）、轮询收起两拍
//!   确认防抖（close_streak · DEC-83）、缝侧邻接宽限挂起收起（seam_grace
//!   · DEC-95：越缝深入邻屏 ≤20pt 挂起两拍，带内长驻 0.6s 恢复常规）；
//!   单击直开 + 唤出轮询/失活期探测/MouseExitHide 三路收板承载免 hover 依赖
//!   （DEC-102）· DEC-67
//! - 拖动释放后吸附（DEC-78）：拖曳中不限位、
//!   不预览吸附（守护门 2 拍静止判定，SNAP_STABLE_TICKS）；释放后一次性贴边重泊
//! - 状态栏常驻图标：NSStatusItem（variable length · SF
//!   Symbol（note.text）→ title 字形 fallback · Accessory 政策无 Dock 图标——
//!   REQ-SETTINGS-STATUSBAR「菜单栏常驻」演示宿主承载，5 项内容菜单 defer
//!   产品壳轮）；点击 → 停靠形态 toggle 主面板（旗标 0.05s 排水；展示语义
//!   revealed=false——光标入板前不收）；演示窗形态仅 stderr 登记；目标对象
//!   裸指针进程生命周期持有（setTarget: 弱引用不 retain）
//! - 应用图标：品牌 logo PNG（include_bytes 内嵌）→ platform::create_app_icon 主线程
//!   NSData/NSImage 桥 → NSApp.applicationIconImage；Accessory 政策下 Dock 无图标
//!   （既定），可见面 = 通知横幅/NSAlert 等程序化图标场景
//! - 设置持久化：触发位置
//!   变更即刻落 $HOME/.parchora/data/parchora-settings.env（行式
//!   trigger= · tmp+rename 原子写 · cfg(test) noop）；启动优先级 =
//!   显式 CLI token > 档位值 > 默认右侧；设置窗 hover 于自身面板之外连续 4 拍
//!   （0.2s）→ 自动保存（幂等）+ 关闭（settings_watch 全形态；例外：
//!   仅移出设置窗即关——即便光标在其它在册 GUI 上，不牵动其它窗），
//! - 单实例锁（DEC-70）：temp 目录 flock，重复启动提示后立即退出；锁机制故障放行
//!   （防御不阻断启动）
//! - 仅主面板可拖 + 拖动跟随：window_opts 分档 is_movable
//!   （主面板 true；Peek/设置/菜单/根/页演示窗 false——vendor setMovable_ 直映射）；
//!   Peek 开窗后经 parchora_platform::attach_child_window 挂为面板 AppKit 子窗
//!   （addChildWindow:ordered:NSWindowAbove）：拖动主面板时内容详情窗原生零抖
//!   跟随并保持 16px 邻接与对齐（AppKit 原生保持子窗相对偏移）；挂接失败降级
//!   独立窗（stderr 登记）；重泊/换位路径先关 Peek 再关面板（子窗先卸）
//! - 单击 Peek（DEC-72）：三子视图行
//!   单击 → ShellEvent::PeekShow → Peek 窗主面板同侧邻接（dock::peek_bounds · 间距
//!   16px）开窗/原位更新（单实例：handle.update set_item，陈旧句柄清理后重开）；
//!   关窗经 PeekEvent::HoverChanged 对称 0.12s（peek_close_gen 防争）；窗内悬停面板
//!   MouseExitHide 抑制（SCN-PREVIEW-SCROLL-IN-PREVIEW 阅读态）；⚙/Peek 均
//!   幂等开窗（需求 3：最多 1 主面板 + 1 内容详情 + 1 设置独立窗）
//! - 编辑转出（REQ-PREVIEW-IMAGE / SCN-PREVIEW-QUICKLOOK-EDIT-
//!   HANDOFF · DEC-24）：Peek「编辑」钮/⌘E → PeekEvent::EditHandoff 载条目 →
//!   open_editor_handoff：读 Peek 实界 → 关 Peek（转出语义：Quick Look 不承载
//!   编辑 · 双窗同域叠置防噪声）→ 统一编辑器窗（EditorShell::with_content）
//!   Peek 原址接手（实界不可读 fallback 居中 · --page editor 同口径）；
//!   DockHandles.editor 单实例登记（再次 handoff set_content 原位热换 · DEC-94）；
//!   Esc 关编辑窗（EditorEvent::Escape · 独立任务窗不入面板级联）；编辑窗入
//!   keep_domain_contains（窗内点击不误收面板 · GUI 域口径）；编辑
//!   语义（REQ-EDIT-MARKDOWN 输入引擎）随 TASK-504 接线，现壳为只读源码承接
//! - app 动作出借域 defer（DEC-77）：gpui-fast 实体租借
//!   语义（entity_map.rs lease 独占 · 嵌套同实体 update = double_lease_panic abort，
//!   真机 5 份崩溃报告同一签名）下，listener/sink 转发链（on_hover listener、
//!   行单击 → forward.update(panel)）执行期间 view 实体处于租借态，app
//!   sink 同步触同实体窗柄（close_panel/open_peek_item→panel_actual_bounds 等）
//!   即崩——sink 动作（PeekShow/HidePanel/Activate/Escape）统一经
//!   defer_app 延迟至租借释放后的下一调度拍（arm_peek_close 同款 spawn 先例）；
//!   OpenSettings/HoverChanged 仅触其它实体与全局，保持同步
//! - 主窗吸附守护（DEC-77）：0.25s 轮询主面板实界 vs
//!   dock::panel_bounds 停泊位，偏差 > 2px = 被拖离贴缘 → 关 Peek + 贴边重泊
//!   （vendor 无编程式移窗 API，关窗重开为既定模式 · DEC-67）；gen bump 使未决
//!   peek-close timer 失效（防误关重泊后新面板）；多显示器副屏真原点 defer
//!   （DEC-76 口径不变，单屏机完全正确）
//!
//! CLI：
//!   parchora-app                  主面板（默认 · 按触发位置贴边停泊，无参=右侧）
//!   parchora-app demo 底部        主面板 · 指定触发位置（7 选项简体中文标签 · DEC-84）
//!   parchora-app --peek 底部      Peek 演示窗（底部位置 = 宽高互换演示）
//!   parchora-app --page editor|conflict|ext|picker
//!                                 无语境页面演示窗（PG-B1 编辑区 / DG-CONFLICT /
//!                                 DG-EXT / DG-PICKER · 19 页全承载 DEC-73）
//!   parchora-app --settings       设置窗（面板分区可选触发位置，即时生效）
//!   parchora-app --menu           菜单栏菜单演示窗
//!   parchora-app --root           DG-ROOT 启动根选择器演示窗
//!
//! NSPanel park_offscreen / 双材质语义属 parchora-platform（DEC-59/60），本演示
//! 宿主窗口为 PopUp 置顶家族（DEC-79）。

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use gpui::{
    point, px, size, App, AppContext as _, Bounds, DisplayId, Entity, Image, ImageFormat, Pixels,
    Window, WindowBounds, WindowHandle, WindowKind, WindowOptions,
};
use parchora_capture::notify::{
    QueueNotifier, QueuePromptAction, QUEUE_NOTIFY_INTERVAL_DEFAULT, QUEUE_NOTIFY_INTERVAL_MAX,
    QUEUE_NOTIFY_INTERVAL_MIN, QUEUE_NOTIFY_TITLE,
};
use parchora_capture::promise::{PromiseBoard, QueueEngine, QueueEventKind, StopKind, StopOutcome};
use parchora_capture::queue::{ClipQueue, QueueOrder};
use parchora_capture::{BoardContent, Captured, DedupeLru, Engine, FrontmostApp, PasteboardSource};
#[cfg(not(test))]
use parchora_platform::screen_areas;
use parchora_platform::{
    cursor_position, display_bounds_list, image_thumbnail_png, listen_access_granted,
    pasteboard_board, pasteboard_change_count, request_listen_access, spawn_mouse_tap,
    MouseTapHandle, ScreenArea,
};
#[cfg(not(test))]
use parchora_platform::{
    frontmost_bundle_id, pasteboard_activation_slot, pasteboard_declare_promise,
    pasteboard_promise_slot, pasteboard_watch_activations, pasteboard_write_text,
};
use parchora_ui::dock::{self, DockPrefs, TriggerPosition};
use parchora_ui::pages::collect::{ClipRow, FilterKind, QueueAction, QueueRow};
use parchora_ui::pages::collection::{DiskNote, TrashAction};
use parchora_ui::pages::dialogs::{ConflictSheet, ExtConflictView, PickerSheet, RootPicker};
use parchora_ui::pages::editor::{self, EditorShell};
use parchora_ui::pages::menu::MenuView;
use parchora_ui::pages::peek::{self, PeekHover, PeekItem, PeekView};
use parchora_ui::pages::settings::{ListenAuth, SettingsEvent, SettingsView};
use parchora_ui::pages::shell::{PanelView, ShellEvent, View};
use parchora_ui::pages::tasks::{TaskCreateEvent, TaskCreateView, TaskDbEvent};
use parchora_ui::scale as s;
use parchora_ui::tokens as tk;
use parchora_ui::walltime;

use parchora_core::content::{new_uuid, Clip, ContentType, Uuid as CoreUuid};
use parchora_core::task::{Priority, ScheduleBucket, TaskStatus, Todo as DbTodo};
use parchora_core::timefmt::{epoch_from_iso, iso_utc};
use parchora_db::{
    clear_queue_entries, delete_clip, delete_queue_entry, delete_todo_by_identifier,
    delete_unpinned_before, delete_unpinned_over_limit, insert_clip, insert_queue_entry,
    insert_todo, list_blob_rel_paths, list_clips, list_queue_entries, list_todos,
    max_identifier_seq, recent_clip_hashes, set_todo_bucket, set_todo_status, Db, RootStore,
};
// 帧遥测（性能轮 · DEC-215）：仅 frame-profiler feature 构建编译入。
#[cfg(feature = "frame-profiler")]
mod frame_telemetry;

/// 单实例锁文件名（temp 目录下；flock 随进程退出自动释放，残留文件无害）。
const LOCK_FILE: &str = "parchora-app.lock";

/// 应用图标内嵌 PNG（DEC-92/160 · 用户交付 parchora_logo_color.png 经 pad_logo
/// 垫方 1024×1024 基底 · DEC-193 随用户新稿重派生（源 1032×1024 非正方形 ·
/// 内容 1024×1016 居中）；运行时图标与关于分区同源；include_bytes 路径相对
/// src/ 解析 → crate 根 assets/；文件名「512」为沿用句柄，字节面即现行彩色稿）。
const APP_ICON_PNG: &[u8] = include_bytes!("../assets/parchora-logo-512.png");

/// 状态栏专属灰版 logo（DEC-160 · 用户交付 parchora_logo_gray.png 垫方 → sips 36
/// 基底 · DEC-193 随用户新稿重派生（源 1024×1024）· DEC-195 随灰稿二换重派生
/// （源仍 1024×1024·根稿直缩同字节核验）；状态栏用灰版，应用其余位置
/// 用彩色版）：36×36
/// 像素基底 = 18pt@2x 菜单栏标准高度域（set_logo_image 内 setSize 18×18 pt
/// 承载），srcset 无需更高分。
const APP_ICON_STATUS_PNG: &[u8] = include_bytes!("../assets/parchora-logo-status.png");

/// Embedded 资产名（ui 层 img(path) 引用；同名常量值硬对齐）。
const ASSET_LOGO_COLOR: &str = "parchora-logo-color.png";

/// 内嵌资产源（Application::with_assets）：路径→字节静态表，
/// ui 层关于分区彩色 logo 经 Resource::Embedded 解析（gpui 内部 image 解码）。
/// 未登记路径返回 None（gpui 兜底空载荷）。
struct ParchoraAssets;

impl gpui::AssetSource for ParchoraAssets {
    fn load(&self, path: &str) -> gpui::Result<Option<std::borrow::Cow<'static, [u8]>>> {
        let bytes = match path {
            ASSET_LOGO_COLOR => Some(std::borrow::Cow::Borrowed(APP_ICON_PNG)),
            _ => None,
        };
        Ok(bytes)
    }

    fn list(&self, _path: &str) -> gpui::Result<Vec<gpui::SharedString>> {
        Ok(Vec::new())
    }
}

// ───── 时延面分离总注（DEC-210）：显示侧（唤出）零延时常量——REVEAL_CONFIRM_
// TICKS=1 单拍直唤，唯一等待 = 守护轮询拍距（REVEAL_POLL_SECS）；隐藏侧（用
// 户面自动隐藏）唯一延时 = PANEL_CLOSE_DELAY_SECS（1s 复核 · DEC-184，定义
// 见下方 CLOSE_CONFIRM_TICKS 邻位）。两侧唯一交会点 = open_panel_docked 首行
// cancel_panel_close（开面板撤隐藏挂起）；隐藏链不反触显示链（cancel 仅丢
// 本侧 timer 持活槽）。
/// 主窗吸附守护轮询间隔（秒）：被拖离贴缘后最迟 ~0.25s 内重泊。
const SNAP_POLL_SECS: f32 = 0.25;
/// 吸附判定容差（逻辑 px）：实界与停泊位四分量差 ≤ 此值视为仍在贴缘（取整容忍）。
const SNAP_EPS: f32 = 2.0;
/// 释放即吸附守护轮询间隔（秒 · DEC-210）：tap 左键释放锁存排水拍距——拖动释
/// 放后最迟 ~0.05s 内贴边重泊（0.25s 吸附守护的即席前哨；释放信号为 tap 真
/// 键态，非两拍静止近似）。与唤出守护拍距各自独立命名，时延面互不耦合。
const DRAG_RELEASE_WATCH_POLL_SECS: f32 = 0.05;
/// 唤出守护轮询间隔（秒）：进带唤出最迟 ~0.05s 内响应（tap 事件源下轮询仅
/// 为降级兜底，成本可忽略）。
const REVEAL_POLL_SECS: f32 = 0.05;
/// 唤出确认拍数：连续 N 拍读得光标在热区内才唤出（=1：滑过即唤，唤出侧无停留
/// 确认；防抖眼与误呼由 reveal_suppressed 门 + 收起两拍确认反向承载）。
const REVEAL_CONFIRM_TICKS: u32 = 1;
/// 收起确认拍数（DEC-83 防抖保持）：轮询收起域与失活期出板两路连续 N 拍同离
/// 面板与有效热区才收起（0.05s×2=0.1s 停留确认；快速穿越不收板）。
const CLOSE_CONFIRM_TICKS: u32 = 2;
/// 延时收起挂起时长（秒 · DEC-184「隐藏操作延时 1 秒执行，提升界面
/// 切换流畅度」）：用户面自动隐藏链命中后不即收——挂 1s 复核 timer，回域/回
/// 板/重开即取消。显式路径（Esc 级联/退出链/状态栏 toggle/换泊重泊）不经此
/// ——立即收语义不变（close_panel 唯一收口）。（隐藏侧唯一延时常量 · DEC-210
/// 时延面分离总注：显示侧无延时常量。）
const PANEL_CLOSE_DELAY_SECS: f32 = 1.0;
/// 缝侧邻接宽限拍数（DEC-95）：光标
/// 越出触发缘共享缝深入邻屏 ≤20pt 带内挂起收起判定的长驻上限——拍距
/// REVEAL_POLL_SECS=0.05s × 12 = 0.6s，超限后恢复常规 CLOSE_CONFIRM_TICKS 两拍
/// 收起（总时长 ≤0.7s）；带外/回域宽限计数清零（复位链与 close_streak 成对）。
const SEAM_GRACE_TICKS: u32 = 12;
/// 面板离界缓冲带宽（DEC-100）：鼠标自 GUI 边缘向应用外部移动时，面板实界外扩此带宽
/// 内的落点视为「离界过渡带」——带内挂起收起判定（不瞬隐、不闪烁），长驻
/// 超限后恢复常规两拍收起。沿触发缘向屏外方向以屏缘裁剪——屏外深域归缝侧
/// 邻接宽限带专责（DEC-95 口径不动，两带域不相交、计数互不冻结）。
const EXIT_GRACE_BAND: f32 = 20.0;
/// 面板离界缓冲长驻上限拍数（DEC-100 · 与 SEAM_GRACE_TICKS 同构同值）：拍距
/// REVEAL_POLL_SECS=0.05s × 12 = 0.6s，超限后恢复常规 CLOSE_CONFIRM_TICKS 两拍
/// 收起；带外/回面板/回热区计数清零（复位点与 seam_grace_ticks 成对）。
const EXIT_GRACE_TICKS: u32 = 12;
/// 设置窗悬停外自动关闭确认拍数：光标在设置窗自身实界之外连续 N 拍 → 即刻
/// 持久化（幂等）+ 关闭设置窗（0.05s×4=0.2s 持续 hover-out；仅移出设置窗即关
/// ——即便位于其它在册 GUI 上）。
const SETTINGS_CLOSE_CONFIRM_TICKS: u32 = 4;
/// tap 采样槽新鲜度阈值（秒）：槽时刻超龄视为 tap 假死/失活，光标源回退
/// 读探针 poll；tap 复活后槽回新鲜自动自愈（DEC-82）。
const TAP_FRESH_SECS: f32 = 0.5;

/// 剪贴板守护轮询间隔（秒 · REQ-CAPTURE-POLL 0.5s：changeCount 未命中变化不得
/// 读板面）与同内容判重窗（秒 · REQ-CAPTURE-DEDUPE 10s）。
const CLIP_POLL_SECS: f32 = 0.5;
const CLIP_DEDUPE_SECS: u64 = 10;

/// 队列引擎守护轮询间隔（秒 · TASK-301）：承诺式引擎独立拍——索取信号
/// 消费与 350ms 结算窗兑现的拍制承载（capture::promise 模块注：索取→消耗
/// 总时限 ≤0.45s，结算基点＝桥承载的回调时刻，拍延迟不侵蚀语义）。
const QUEUE_POLL_SECS: f32 = 0.1;

/// 应用数据根目录名（HOME 下基准：~/.parchora——默认根/设置档/files 根三口径共用单源 · FIX-20261023-12）。
const APP_DIR: &str = ".parchora";
/// 面板关闭期捕获缓冲上限（DEC-163 · 溢出丢最旧；DB 持久化随 TASK-300 实装）。
const CLIP_PENDING_CAP: usize = 200;
/// 拾记条目双阈值默认值（DEC-163/178）：剪贴板
/// 历史保留时长（天）＝除 Pin 外的硬上限（REQ-CAPTURE-LIMIT 精神）· 配置档
/// clip_retention_days（非法/缺省回默认 1）；有效时长（分钟）＝未处理窗口（超
/// 窗未转化为任务/集锦知识即清）· 配置档 clip_valid_min（非法/缺省回默认）。
/// 保留时长运行阈＝天数 × 1440 分钟（CLIP_MINUTES_PER_DAY · clip_sweep 换算）；
/// 旧档键 clip_survive_min（7 天分钟制存活档）解析一律忽略。
const CLIP_RETENTION_DAYS_DEFAULT: u64 = 1;
const CLIP_VALID_MIN_DEFAULT: u64 = 24 * 60;
/// 天 → 分钟换算因子（保留时长档唯一换算点 · clip_sweep）。
const CLIP_MINUTES_PER_DAY: u64 = 1440;
/// 历史上限档默认值（REQ-CAPTURE-LIMIT 档位 [100,500,1000,5000,0] · 0=不限；
/// DEC-188 实装——默认 500 ＝设置 spec 存储分区展示值锚定）。
const CLIP_LIMIT_DEFAULT: u64 = 500;
/// 上限档位合法集（设置档解析门槛：非集内值视为缺省回默认）。
const CLIP_LIMIT_STEPS: [u64; 5] = [100, 500, 1000, 5000, 0];
/// 捕获监控开关默认值（SCN-CAPTURE-SWITCH-OFF · DEC-188 · 默认开——既有行为
/// 不回退）。
const CLIP_CAPTURE_ENABLED_DEFAULT: bool = true;
/// 缩略图最长边（SCN-CAPTURE-THUMB · DEC-188 · 320px PNG）。
const CLIP_THUMB_EDGE_PX: u32 = 320;
/// DB 写失败重试队列上限（TASK-300 · 超限丢最旧——与 pending 缓冲同防滥用面；
/// 队列仅在存储根打开但单拍写失败时承载，进程内存态）。
const CLIP_DB_RETRY_CAP: usize = 200;
/// 剪贴板排除应用设置档键（REQ-CAPTURE-EXCLUDE · DEC-188：逗号
/// 分隔 bundleId 列表，命中前台即永久丢弃不捕获；设置存储页文本输入承载）。
const CLIP_EXCLUDE_APPS_KEY: &str = "capture_exclude_apps=";

/// 队列通知节流间隔设置档键（REQ-SETTINGS-QUEUE-NOTIFY-INTERVAL · DEC-48）。
const QUEUE_NOTIFY_INTERVAL_KEY: &str = "queue_notify_interval=";

/// 队列消费序模式设置档键与值字面量（DEC-212）：fifo/lifo；非法值解析拒绝
/// ＝键缺省回 FIFO 默认。
const QUEUE_ORDER_KEY: &str = "queue_order=";
const QUEUE_ORDER_VAL_FIFO: &str = "fifo";
const QUEUE_ORDER_VAL_LIFO: &str = "lifo";

/// 队列消费序模式 → 档值（DEC-212 序列化唯一点）。
fn queue_order_str(o: QueueOrder) -> &'static str {
    match o {
        QueueOrder::Fifo => QUEUE_ORDER_VAL_FIFO,
        QueueOrder::Lifo => QUEUE_ORDER_VAL_LIFO,
    }
}

/// 档值 → 队列消费序模式（DEC-212 解析唯一点；非法/未知值 → None＝键缺省）。
fn queue_order_parse(v: &str) -> Option<QueueOrder> {
    match v.trim() {
        QUEUE_ORDER_VAL_FIFO => Some(QueueOrder::Fifo),
        QUEUE_ORDER_VAL_LIFO => Some(QueueOrder::Lifo),
        _ => None,
    }
}

/// CGEventTap 只听 tap 的光标采样槽（tap 线程写 · reveal 守护读；空闲期存最后
/// 位置——鼠标静止无新事件时语义等价 · DEC-81）。
static TAP_POS: Mutex<Option<(f32, f32)>> = Mutex::new(None);
/// tap 活性标志（装配成功置位；守护读光标源的开关 · DEC-81）。
static TAP_ACTIVE: AtomicBool = AtomicBool::new(false);
/// tap 句柄单例仓（进程生命周期持有；不存则句柄即 Drop = tap 即停 · DEC-81）。
static TAP_HANDLE: Mutex<Option<MouseTapHandle>> = Mutex::new(None);
/// tap 创建失败闩（防守护拍 0.05s 高频重试空转：preflight 与 tap 创建间授权可被
/// 撤回/策略拒绝）；授权态翻转或引导卡显式点击时清除重试 · DEC-81。
static TAP_CREATE_LATCHED: AtomicBool = AtomicBool::new(false);
/// tap 采样槽写入时刻（与 TAP_POS 同步写；新鲜度校验依据 · DEC-82）。静态
/// 初始化 None，种子装配与 tap 回调运行时写值。
static TAP_AT: Mutex<Option<Instant>> = Mutex::new(None);
/// tap 槽过期回退 poll 的一次性日志闩（含静止期槽超龄正常回落，仅首次记录
/// 供真机判读 · DEC-82）。
static TAP_STALE_LOGGED: AtomicBool = AtomicBool::new(false);

/// 触发热区镜像条目：（热区 rect, 触发位）——DEC-100 tap 回调逐事件比对基准。
type ZoneMirror = ((f32, f32, f32, f32), TriggerPosition);

/// 触发热区镜像（DEC-100 · tap 回调比对基准）：生产守护拍写入最新
/// (zone, trigger)——tap 回调线程逐事件判「入排除带感知有效热区」→ 置
/// ZONE_HIT 锁存。None（tap 未起首拍前/测试态）回调不判不置。
static ZONE_MIRROR: Mutex<Option<ZoneMirror>> = Mutex::new(None);
/// 快速滑过锁存（DEC-100）：tap 回调线程检出穿越有效热区即置位；
/// reveal_watch_tick 每拍单次消费注入 reveal_watch_tick_with_hit——面板在场
/// 分支（唤出态轮询/失活期探测）消费即丢弃（防陈旧 hit 延迟触发），仅面板
/// 不在场分支生效唤出（与「在带」同径）。
static ZONE_HIT: AtomicBool = AtomicBool::new(false);

/// 停靠形态登记（状态栏点击 toggle 判定面）：true=停靠族（面板
/// toggle 可用）；false=演示窗形态（点击仅 stderr 登记）。main() 启动落值。
static APP_MODE_DOCKED: AtomicBool = AtomicBool::new(false);

/// 单实例守卫三态：持有（唯一实例）/ 已有实例在跑（退出）/ 锁机制故障（放行）。
enum InstanceGuard {
    /// 持有锁：句柄仅承载 flock 生存期（不读内容，drop 即解锁）。
    Held(#[allow(dead_code)] std::fs::File),
    /// 已有实例在运行 → 本次启动退出。
    Busy,
    /// 锁机制自身故障 → 放行继续启动（防御不阻断应用）。
    Pass,
}

/// 抢单实例锁：temp 目录文件 flock（进程级互斥；std::fs::try_lock，零外部依赖）。
fn single_instance_lock() -> InstanceGuard {
    let path = std::env::temp_dir().join(LOCK_FILE);
    let file = match std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(false)
        .append(false)
        .open(&path)
    {
        Ok(f) => f,
        Err(_) => return InstanceGuard::Pass,
    };
    match file.try_lock() {
        Ok(()) => InstanceGuard::Held(file),
        Err(std::fs::TryLockError::WouldBlock) => InstanceGuard::Busy,
        Err(std::fs::TryLockError::Error(_)) => InstanceGuard::Pass,
    }
}

/// 双屏帧快照（DEC-84 · Dock 运行时检测）：活动屏 (全帧, 可用区) 四元组对。
type ScreenFrames = ((f32, f32, f32, f32), (f32, f32, f32, f32));

/// 停靠窗句柄登记（会话态全局）：主面板单例窗（收缩态 = 不存在，全隐）；
/// Peek/设置/编辑转出为独立单例窗；
/// peek_close_gen 为 Peek 关闭防争代次（任何后续 Show/重开都会 bump，使
/// 未决 close timer 失效——不依赖 Task drop 语义）；docked_bounds 为主窗
/// 吸附守护的停泊基准（首轮 tick 播种——待 macOS 菜单栏钳制等窗口服务端
/// 调整落定后读实界为基，重泊/重开时重置重播种）；drag_last 为释放门的
/// 上一拍实界记录（与基准同拍重置）；reveal_streak 为唤出守护的连续在带
/// 拍数（离带清零）；last_frames 为屏帧缓存（DEC-84：Dock/菜单栏布局变化
/// 检测基准）。
#[derive(Default)]
struct DockHandles {
    panel: Option<WindowHandle<PanelView>>,
    /// 显隐保持的活选项卡记账（DEC-181）：
    /// close_panel 整窗移除前读记（显隐唯一收口）· open_panel_docked 重开恢复
    ///（restore_view 首帧前落值＋notify 选中态随帧同步）；None = 无记录（首开
    /// 走默认集锦 · DEC-78）。镜像沿 pinned 全局口径先例（DEC-94）· View 枚举
    /// 不动＝规范 7 无新枚举值。
    panel_view: Option<View>,
    /// 面板实体跨关板存续（DEC-186 根修）：close_panel 整窗移除前提取强句柄
    /// 落全局（窗亡实体存）→ open_panel_docked 复用整实体——拾记行/待办行/
    /// 面板内态跨显隐周期持久（缺陷二「隐藏后列表变空」与缺陷一「面板在场
    /// 期捕获随关窗湮灭」的根修点）；重开时 take 出（下次 close 再提取）。
    panel_entity: Option<Entity<PanelView>>,
    peek: Option<WindowHandle<PeekView>>,
    settings: Option<WindowHandle<SettingsView>>,
    /// 编辑转出窗单例（SCN-PREVIEW-QUICKLOOK-EDIT-HANDOFF）：Peek
    /// 「编辑」/⌘E → 统一编辑器窗（peek 原址接手 · 独立任务窗不入面板级联，Esc
    /// 自关 · 窗内点击入 GUI 保持域）。
    editor: Option<WindowHandle<EditorShell>>,
    /// 待办创建面板窗单例（DEC-165）：待办页 FAB「新建任务」→ 独立
    /// 面板窗（与「创建集锦」编辑窗同构 chrome · 单实例幂等 settings 先例；独立
    /// 任务窗不入面板级联，Esc 自关）。
    task_create: Option<WindowHandle<TaskCreateView>>,
    /// 唤出抑制门（防弹出-隐藏反复循环 · DEC-79）：执行唤出时置 true（面板关闭
    /// 后若光标仍滞留热区——如 Esc 收起——不立即复呼）；光标离带的守护拍复位
    /// （false = armed 可唤出；Default 即 armed）。
    reveal_suppressed: bool,
    /// 唤出确认拍计数（=1 滑过即唤；离带清零）。
    reveal_streak: u32,
    /// 当前面板是否为唤出态所开（true → 入轮询收起域：光标同离面板+热区且无
    /// Peek → 收；false = 启动/换位/重泊开的展示语义面板，无条件呈现不入此域
    /// ——防启动闪隐与重泊后即收违背 DEC-78「不脱离边缘」）。
    revealed: bool,
    peek_close_gen: u64,
    /// 未决 peek-leave 对称宽限 close timer（持活避免任务被回收；正确性由 gen 保证）
    peek_close_task: Option<gpui::Task<()>>,
    /// 未决延时收起代次（DEC-184）：用户面自动
    /// 隐藏链（MouseExitHide / 守护两拍确认 / 点击外）不即收——arm 挂 1s 复核
    /// timer，兑时复核（Pin/Peek/面板在场/回域光标）仍成立才 close_panel；
    /// 任一取消源（回域复位块/回板 arm 探针/open_panel_docked）bump 代次使
    /// timer 失效。镜像 peek_close_gen 先例（gen 防 · 持活 task 槽）。
    close_pending_gen: u64,
    /// 未决延时收起 timer 持活槽（丢槽 = Task drop 撤销 timer · cancel 共用）
    close_pending_task: Option<gpui::Task<()>>,
    /// 吸附守护停泊基准（None = 待播种；窗口实界四元组）
    docked_bounds: Option<(f32, f32, f32, f32)>,
    /// 释放门上一拍实界（None = 拖动基线待建；与基准一并在重泊/重开时重置）
    drag_last: Option<(f32, f32, f32, f32)>,
    /// 失活期面板移出仿真探测臂（DEC-81 · D-01）：展示语义面板（启动/换位/
    /// 重泊 · revealed=false）光标入板即 arm；armed 后出板（排除带感知热区外 +
    /// 无 Peek）→ 收起（与 MouseExitHide 语义同构）。开/关面板时复位。
    panel_probe_armed: bool,
    /// 轮询收起两拍确认计数（DEC-83）：与唤出对应的收起防抖——唤出态轮询
    /// 收起域与失活期探测出板两路共用；任一保持样本（入板/在带）清零，连续
    /// CLOSE_CONFIRM_TICKS 拍同离面板与热区才收起（快速穿越不收板）。
    close_streak: u32,
    /// 缝侧邻接宽限计数（DEC-95）：光标越出触发
    /// 缘共享缝深入邻屏 ≤20pt 带内的连续拍数——带内计数上行并挂起收起判定
    /// （close_streak 同拍清零），连续 SEAM_GRACE_TICKS（0.6s）后恢复常规两拍；
    /// 带外/回域清零（复位点与 close_streak 成对：close/open/in_zone/in_panel
    /// 六处）；点击外隐藏路径不含宽限（点击 = 显式动作 · REQ-PANEL-HIDE）。
    seam_grace_ticks: u32,
    /// 面板离界缓冲计数（DEC-100）：光标落「面板实界外扩
    /// EXIT_GRACE_BAND ∩ 当前屏」过渡带内的连续拍数——带内计数上行并挂起收起
    /// 判定（close_streak 同拍清零），连续 EXIT_GRACE_TICKS（0.6s）后恢复常规
    /// 两拍；带外/回面板/回热区清零（复位点与 seam_grace_ticks 成对六处）；
    /// 点击外隐藏路径不含缓冲（点击 = 显式动作 · REQ-PANEL-HIDE 不动）。
    exit_grace_ticks: u32,
    /// 设置窗悬停外自动关闭计数：光标持续在设置
    /// 窗自身实界之外 SETTINGS_CLOSE_CONFIRM_TICKS 拍 → 即刻持久化 + 关闭设置窗；
    /// 设置于窗内（含与其它 GUI 视觉重叠）或光标读失败清零（开/关窗时复位）。
    settings_close_streak: u32,
    /// 设置窗悬停外自动关闭臂：false = 未臂
    /// ——光标首次进入设置窗实界前不累计外闭计数。⚙/状态栏菜单/CLI 开窗时光标
    /// 恒在居中设置窗实界之外（主面板停靠边缘），无臂则开窗 SETTINGS_WATCH_POLL_SECS
    /// ×SETTINGS_CLOSE_CONFIRM_TICKS（0.2s）即被判「移出」关闭——「移出」语义
    /// 预设先前在窗内；与 panel_probe_armed「入板即 arm」同构同源。开/关窗复位 false。
    settings_close_armed: bool,
    /// Pin 全局口径（DEC-94 · SCN-PANEL-PIN-BLOCKS-HIDE）：PanelView Pin 翻转经
    /// ShellEvent::PinChanged 同步（单一事实源在 app 层）——轮询收起/失活期探测/
    /// 点击外收起/arm_peek_close 面板段四隐藏路径统一读此值；面板关闭复位
    /// （新面板 PanelView::pinned=false 初始同源）。
    pinned: bool,
    /// 状态栏点击防竞态门（DEC-94）：status_click_toggle 两分支置位 → 当拍点击
    /// 旗标由 click_outside_tick 消费不落动作（防「点击开面板 → 残余点击旗标 →
    /// 立即点击外误收」闪烁）；消费即清。
    skip_click: bool,
    /// 屏帧缓存（DEC-84 · Dock 运行时检测）：上次守护拍读到的活动屏
    /// (全帧, 可用区)。None = 待播种；帧变化 → 重解析触发位冲突（变更才重泊）。
    last_frames: Option<ScreenFrames>,
}

impl gpui::Global for DockHandles {}

/// 剪贴板捕捉守护态（TASK-300 全量 · REQ-CAPTURE-*）：拍级状态机（changeCount
/// 基线 + 判定链 + 前台排除 + 隐私/空板三闸，承载于 capture::Engine）、SHA256
/// 10s 判重（capture::DedupeLru · 跨重启墙钟回拨播种）、面板关闭期捕获缓冲
///（重开按序冲洗 · DEC-186）、写失败重试队列与双阈值配置（保留时长天 / 有效
/// 时长分钟 · 配置档为准，DEC-178）。隐私拒收在 platform 板面桥三态、自写板
/// 跳过经 ui clipstamp，本态只承载「已过闸」的载荷。
struct ClipCapture {
    /// 拍级状态机（changeCount 基线归其承载 · SCN-BOOT-BASELINE）。
    engine: Engine,
    /// 判重窗（SHA256 指纹 + 10s · REQ-CAPTURE-DEDUPE；跨重启种子回拨注册）。
    recent: DedupeLru,
    /// 面板关闭期捕获缓冲（行完整形态；面板打开时序向冲入拾记列表顶 · 行时随
    /// 行显示 · DEC-183/186）。
    pending: Vec<ClipRow>,
    /// DB 写失败重试队列（载荷原封；下拍先行补写——REQ-CAPTURE-PERSIST 可靠
    /// 承载；仅存储根在场时消费）。
    db_retry: Vec<CapturedPayload>,
    /// 剪贴板历史保留时长（天 · 配置档 clip_retention_days · 非 Pin 硬上限 ·
    /// DEC-178 · sweep 换算 × 1440 分钟）。
    retention_days: u64,
    /// 有效时长（分钟 · 配置档 clip_valid_min · 未处理窗口）。
    valid_min: u64,
    /// 前台排除列表（REQ-CAPTURE-EXCLUDE · 配置档 capture_exclude_apps；bundleId
    /// 命中即永久丢弃。DEC-188 设置存储页文本输入承载）。
    exclude_apps: Vec<String>,
    /// 捕获监控开关（SCN-CAPTURE-SWITCH-OFF · DEC-188 · 配置档 capture_enabled）：
    /// 关闭期捕获全链停走（重开时 engine 基线重播种——关窗期 changeCount 积累
    /// 一次性吞掉，不回灌）。
    capture_enabled: bool,
    /// 历史上限档（REQ-CAPTURE-LIMIT · DEC-188 · 配置档 capture_limit · 非 Pin
    /// 计数上限；0=不限；调小立即生效 sweep 两面对账）。
    limit: u64,
}

impl Default for ClipCapture {
    fn default() -> Self {
        Self {
            engine: Engine::new(),
            recent: DedupeLru::new(),
            pending: Vec::new(),
            db_retry: Vec::new(),
            retention_days: CLIP_RETENTION_DAYS_DEFAULT,
            valid_min: CLIP_VALID_MIN_DEFAULT,
            exclude_apps: Vec::new(),
            capture_enabled: CLIP_CAPTURE_ENABLED_DEFAULT,
            limit: CLIP_LIMIT_DEFAULT,
        }
    }
}

impl gpui::Global for ClipCapture {}

/// 队列宿主态（DEC-207 · A1 队列接线）：队列单一事实源＝capture::ClipQueue
///（F/CAP-4 数据面 · design 骨架图 ClipQueue 归 capture crate）；UI 只持
/// 显示镜像（set_queue_rows 全量替换·set_disk_notes 同构）。DB 持久化随
/// DEC-212 实装：条目即写 queue_entries，恢复随启动对账（reseed_queue_from_db）。
struct QueueHost {
    queue: ClipQueue,
    /// 承诺式投递状态机（TASK-301）：进行中条目快照 + 结算窗状态；
    /// 队列数据本体仍在本 queue（placed 项=视图首项），单拍编排归 queue_tick。
    engine: QueueEngine,
    /// 队列提示编排（TASK-716 · C-14）：「弹一弹＋系统通知」双通道状态机
    ///（会话内存态 · N 值设置项即时生效）。
    notifier: QueueNotifier,
    /// 持久承接映射（DEC-212 · 会话内存态）：内存条目 id → DB uuid——
    /// 消耗/停止删行查 uuid 唯一通道；入队成功登记，消耗/清空摘除（防泄漏）。
    uuid_by_id: HashMap<u64, String>,
    /// 入队序计数器（DEC-212 · 单调递增；恢复链 = max(ord)+1 播种）。
    next_ord: u64,
}

impl Default for QueueHost {
    fn default() -> Self {
        Self {
            queue: ClipQueue::new(),
            engine: QueueEngine::new(),
            notifier: QueueNotifier::default(),
            uuid_by_id: HashMap::new(),
            next_ord: 1,
        }
    }
}

impl gpui::Global for QueueHost {}

/// 捕获载荷（到达态统一形态 · 文本与二进制同源）：engine 产出 or 测试注入。
#[derive(Debug, Clone)]
struct CapturedPayload {
    content_type: ContentType,
    /// 文本载荷（plainText 原文 / html 源文 / color hex / fileURL 列表；image None）。
    text: Option<String>,
    /// image 原始字节（blob 落库承载）。
    bytes: Option<Vec<u8>>,
    /// SHA256 内容指纹（判重键 + DB content_hash 同源）。
    hash: String,
    /// 前台应用 bundleId（REQ-CAPTURE-EXCLUDE 读点 · DB source_app 承载）。
    source_app: Option<String>,
    /// 缩略图 PNG 字节（SCN-CAPTURE-THUMB · DEC-188 · 即拍即转；None = 桥失败
    /// 或非 image 类——行面回退棋盘占位）。
    thumb: Option<std::sync::Arc<Vec<u8>>>,
    /// 捕获点墙钟 epoch 秒（行显示与 DB created_at 同源 · DEC-183）。
    wall: i64,
    /// 捕获点单调时刻（sweep/判重基准）。
    at: Instant,
}

impl CapturedPayload {
    fn from_captured(cap: &Captured, source_app: Option<String>, wall: i64, at: Instant) -> Self {
        Self {
            content_type: cap.content_type,
            text: cap.text.clone(),
            bytes: cap.image_bytes.clone(),
            hash: cap.content_hash(),
            source_app,
            thumb: None,
            wall,
            at,
        }
    }
}

// ───── 持久化存储接线（TASK-300 · REQ-CAPTURE-PERSIST + REQ-TASK-PERSIST 唯一
// 接入点：db crate 经 AppStore 全局承载；打不开 → stderr 披露 + 内存降级）─────

/// 应用存储根（db 承载 · `root/data/parchora.db` + `root/data/blobs/<uuid>/raw`）：
/// 与 files 根同轴（=~HOME/.parchora；HOME 缺失回退相对路径，同 files_root 先例）。
/// DEC-189：唯一消费点＝生产开根（init_app_store）；blob 运行域（落盘/读档/
/// 清理）恒随活跃 RootStore 根（store_root）不散用本口；测试构建下本口不存在
///（测试一律 temp 显式根，真机档不沾）。
#[cfg(not(test))]
fn data_root() -> std::path::PathBuf {
    match std::env::var("HOME") {
        Ok(home) => std::path::PathBuf::from(home).join(APP_DIR),
        Err(_) => std::path::PathBuf::from(".parchora"),
    }
}

/// 活跃单根连接仓（per-root · 切根 = 先关旧后关新，RootStore 同源不变量）。
#[derive(Default)]
struct AppStore {
    store: RootStore,
}

impl gpui::Global for AppStore {}

/// 打开（或重开）存储根（生产入口 · main() 启动链显式调用；cfg(test) 不自动
/// 调用——测试一律经 init_app_store_at(temp) 显式指定，防污染真机档）。
/// 打开失败 → stderr 披露 + 无活跃根（内存降级，capture/ui 链路不中断）。
#[cfg(not(test))]
fn init_app_store(cx: &mut App) {
    init_app_store_at(cx, &data_root());
}

/// 指定根打开（测试/诊断入口）：重入幂等（switch_to 先关旧后开新 · §2.10.3）。
fn init_app_store_at(cx: &mut App, root: &std::path::Path) {
    let open = RootStore::new();
    let outcome = open.switch_to(root);
    match outcome {
        Ok(()) => {
            cx.set_global(AppStore { store: open });
            prune_db_retention(cx);
            seed_dedupe_from_db(cx);
            // blob 差集清理随启动对账（DEC-189 · design L287/288「崩溃孤儿由
            // 启动清理」字面承载）：磁盘孤儿与删行无必然关联，不以 trim>0 为前提。
            prune_blob_dirs(cx);
            // 队列持久化恢复（DEC-212 · 启动对账第四项）：上会话残余条目重建。
            reseed_queue_from_db(cx);
        }
        Err(e) => {
            // fail-closed 披露（含 SchemaTooNew 应用过旧）：不静默，不虚构降级。
            eprintln!("[parchora] 存储根打开失败：{e}（本轮运行无持久化，仅内存态）");
            cx.set_global(AppStore { store: open });
        }
    }
}

/// 存储根打开后的两项启动对账（TASK-300）。
/// ① 保留时长清理（REQ-CAPTURE-LIMIT 精神 · 非 Pin 存量超墙钟阈值即清）——
/// 与 UI 侧 Instant 存活阈值同轴；失败仅披露不中断。
/// ② 判重播种（跨重启 10s 窗）：kill-9 快重启场景窗内同内容不重入库。
fn prune_db_retention(cx: &mut App) {
    let now_wall = walltime::now_epoch();
    let cutoff = now_wall
        - (cx
            .global::<ClipCapture>()
            .retention_days
            .saturating_mul(86_400) as i64)
            .saturating_sub(1); // 当日边界内不误清（含今日全部）
    let Some(db) = app_db(cx) else { return };
    match delete_unpinned_before(&db, &iso_utc(cutoff)) {
        Ok(0) => {}
        Ok(n) => eprintln!("[parchora] 存储对账：清理未 Pin 过期拾记 {n} 条"),
        Err(e) => eprintln!("[parchora] 存储对账清理失败：{e}"),
    }
}

fn seed_dedupe_from_db(cx: &mut App) {
    let Some(db) = app_db(cx) else { return };
    let now_wall = walltime::now_epoch();
    let now = Instant::now();
    match recent_clip_hashes(&db, &iso_utc(now_wall - CLIP_DEDUPE_SECS as i64)) {
        Ok(rows) => {
            let cap = cx.global_mut::<ClipCapture>();
            for (hash, created_at) in rows {
                let Some(captured_wall) = epoch_from_iso(&created_at) else {
                    continue;
                };
                cap.recent.seed(&hash, captured_wall, now_wall, now);
            }
        }
        Err(e) => eprintln!("[parchora] 判重播种读失败：{e}"),
    }
}

/// 队列持久化恢复（DEC-212 · 启动对账）：queue_entries 全表按 ord 升序读 →
/// 内存队列重建（fresh 实例内存 id 1..n 重发），uuid 映射与 next_ord（=max+1）
/// 随行播种；瞬时承诺态不持久（恢复恒 Idle → queue_tick reconcile 自愈）。
/// FIFO/LIFO 模式已由 init_dock_globals 随设置档播种，此处不重复。
/// 读失败／行空 → 静默空队列起步（启动时队列本就空——语义等价）；开根失败
/// 面由 init_app_store_at 上游承载（AppStore 无活跃根 → app_db None 早退）。
fn reseed_queue_from_db(cx: &mut App) {
    let Some(db) = app_db(cx) else {
        return;
    };
    let rows = match list_queue_entries(&db) {
        Ok(rows) => rows,
        Err(e) => {
            eprintln!("[parchora] 队列恢复读失败：{e}（空队列起步）");
            return;
        }
    };
    if rows.is_empty() {
        return;
    }
    let host = cx.global_mut::<QueueHost>();
    host.queue.clear();
    host.uuid_by_id.clear();
    let mut max_ord = 0u64;
    for row in rows {
        max_ord = max_ord.max(row.ord.max(0) as u64);
        let id = host.queue.enqueue(&row.text);
        host.uuid_by_id.insert(id, row.uuid);
    }
    host.next_ord = max_ord + 1;
}

/// 活跃根连接句柄（未开根/开根失败 → None；调用方内存降级承载）。
fn app_db(cx: &App) -> Option<std::sync::Arc<Db>> {
    cx.try_global::<AppStore>()
        .and_then(|store| store.store.active_db().ok())
}

/// 活跃存储根快照（DEC-189 blob 域锚定 · 唯一事实源随 RootStore）：blob 落盘/
/// 读档/差集清理统一随活跃根——引用集读自活跃库，磁盘域必须与其同根（若走
/// HOME 基 data_root() 会与 temp/他根库分叉，差集空引用误判全库孤儿）。无活
/// 跃根 → None，调用方同「无存储」降级语义承载。
fn store_root(cx: &App) -> Option<std::path::PathBuf> {
    cx.try_global::<AppStore>()
        .and_then(|store| store.store.active_root().ok())
        .flatten()
}

/// image 载荷落盘（TASK-300 · `root/data/blobs/<uuid>/<name>`）：raw 原件与
/// thumb 缩略同名目录双件（DEC-188 缩略随链）；root 恒为活跃存储根（DEC-189
/// blob 域锚定 · 与 DB 同根）。返回相对 data/ 的 blob 路径（DB blob_rel_path/
/// thumb_rel_path 承载）。
fn write_blob(
    root: &std::path::Path,
    id: &CoreUuid,
    name: &str,
    bytes: &[u8],
) -> std::io::Result<String> {
    let rel = format!("blobs/{id}/{name}");
    let dir = root.join("data").join("blobs").join(id.to_string());
    std::fs::create_dir_all(&dir)?;
    std::fs::write(dir.join(name), bytes)?;
    Ok(rel)
}

// ───── WAL 检点接线（DEC-89 双时机 · checkpoint_truncate/checkpoint_passive app 层唯一消费）─────

/// 空闲检点周期（DEC-89「空闲 5 分钟 PASSIVE」字面锚定 · design §2.6/L274）：
/// 节流宿主＝专职守护 loop（六守护同型 · cx.spawn 异步 parked timer 近零成本），
/// 每周期至多一次 PASSIVE——WAL 空时近零成本空转，有帧主动落库收回。
const DB_CHECKPOINT_IDLE_SECS: f32 = 300.0;

/// 空闲检点臂（守护 loop 每周期调用 · 退出臂同源披露风格）：busy>0（读者在场）
/// 本周期放弃、下周期自然重试（无重试风暴——检点可无限推迟不损数据）；有帧
///（log>0）或失败 stderr 披露，空转静默。
fn idle_checkpoint_tick(cx: &mut App) {
    let Some(db) = app_db(cx) else { return };
    match db.checkpoint_passive() {
        Ok((busy, log, ck)) => {
            if log > 0 {
                eprintln!("[parchora] WAL 空闲检点：log {log} 帧 / 已检 {ck} 帧 / busy {busy}");
            }
        }
        Err(e) => eprintln!("[parchora] WAL 空闲检点失败：{e}"),
    }
}

/// WAL 空闲检点守护（DEC-89 双时机 · 空闲臂）：启动满一周期首拍——检点无
/// 启动急迫性，不抢首帧。
fn start_db_idle_checkpoint(cx: &App) {
    cx.spawn(async move |cx| loop {
        cx.background_executor()
            .timer(Duration::from_secs_f32(DB_CHECKPOINT_IDLE_SECS))
            .await;
        cx.update(idle_checkpoint_tick);
    })
    .detach();
}

/// 退出检点（DEC-89 双时机 · 退出臂 · quit 链收口步）：TRUNCATE 把 -wal 复
/// 零交 OS（下次开根免恢复扫描）；单写者主线程语义 busy 恒 0，>0 仅披露不
/// 阻塞退出（数据侧不丢，下次打开自动承接）。
fn quit_checkpoint(cx: &mut App) {
    let Some(db) = app_db(cx) else { return };
    match db.checkpoint_truncate() {
        Ok((busy, log, ck)) => {
            eprintln!("[parchora] 退出检点（TRUNCATE）：log {log} 帧 / 已检 {ck} 帧 / busy {busy}")
        }
        Err(e) => eprintln!("[parchora] 退出检点失败：{e}"),
    }
}

// ───── 捕捉流水线生产桥（capture trait × platform 桥装配 · TASK-300）─────

/// 生产板面源（changeCount + 类型快照桥直转 · 主线程专用面，gpui 守护拍更新）。
struct PlatformPasteboard;

impl PasteboardSource for PlatformPasteboard {
    fn change_count(&self) -> Option<u64> {
        pasteboard_change_count()
    }
    fn board(&self) -> Option<BoardContent> {
        pasteboard_board().map(|b| match b {
            parchora_platform::PasteboardBoard::Empty => BoardContent::Empty,
            parchora_platform::PasteboardBoard::Rejected => BoardContent::Rejected,
            parchora_platform::PasteboardBoard::Snapshots(s) => BoardContent::Snapshots(s),
        })
    }
}

/// 生产前台源（NSWorkspace frontmostApplication 只读探针）。测试态返回 None
///（真机物理前台不可判读——同 cfg(test) 读探针先例；排除判定链在注入测试覆盖）。
struct PlatformFrontmost;

impl parchora_capture::FrontmostApp for PlatformFrontmost {
    fn bundle_id(&self) -> Option<String> {
        #[cfg(test)]
        {
            None
        }
        #[cfg(not(test))]
        {
            frontmost_bundle_id()
        }
    }
}

/// 生产承诺板桥（TASK-301 · capture::PromiseBoard × platform 桥装配）。
/// 真实写回前置 ui clipstamp 登记自写板抑制（S-fixup 抑制点——与回贴 S3
/// 同链：守护读回同文 1.5s 窗内跳过）。测试构建短路（NoopBoard——真板面
/// FFI 不入测试 · 同 PlatformFrontmost cfg(test) 先例；状态机时序由
/// capture 假板全族承载）。
struct PlatformPromiseBoard;

impl PromiseBoard for PlatformPromiseBoard {
    fn declare_promise(&mut self, text: &str) -> bool {
        #[cfg(test)]
        {
            let _ = text;
            false
        }
        #[cfg(not(test))]
        {
            pasteboard_declare_promise(text).is_some()
        }
    }
    fn write_real_text(&mut self, text: &str) -> bool {
        // 抑制登记前置（写板可能触发 1~2 次 changeCount——clipstamp 文本回读
        // 匹配承载，两变化同值同文均被吞）。
        #[cfg(not(test))]
        parchora_ui::clipstamp::mark(text);
        #[cfg(test)]
        {
            let _ = text;
            false
        }
        #[cfg(not(test))]
        pasteboard_write_text(text)
    }
    fn drain_demand(&mut self) -> Option<std::time::Instant> {
        #[cfg(test)]
        {
            None
        }
        #[cfg(not(test))]
        {
            pasteboard_promise_slot()
                .demand_at
                .lock()
                .ok()
                .and_then(|mut d| d.take())
        }
    }
    fn last_activation(&self) -> Option<std::time::Instant> {
        #[cfg(test)]
        {
            None
        }
        #[cfg(not(test))]
        {
            *pasteboard_activation_slot().lock().ok()?
        }
    }
}

/// 主面板高：屏高 80% 钳制到 [min,max] × 档位倍率（DEC-198：窄档收紧、高
/// 分档舒展；Hd 档倍率 1.0 = 设计基线原值）。
fn panel_height(screen_h: f32, tier: s::DispTier) -> f32 {
    (screen_h * tk::PANEL_VH).clamp(
        s::sp_at(tier, tk::PANEL_H_MIN),
        s::sp_at(tier, tk::PANEL_H_MAX),
    )
}

/// Peek 高：屏高 70% 钳制到 [min,max] × 档位倍率（DEC-198 同上）。
fn detail_height(screen_h: f32, tier: s::DispTier) -> f32 {
    (screen_h * tk::DETAIL_VH).clamp(
        s::sp_at(tier, tk::DETAIL_H_MIN),
        s::sp_at(tier, tk::DETAIL_H_MAX),
    )
}

/// 活动屏档位（DEC-198）：应用所在屏逻辑宽 → 分辨率档（macOS 逻辑点已内含
/// 系统缩放语义）；无屏/测试 = Hd 基线（与 800/620 兜底同源）。
fn active_tier(cx: &App) -> s::DispTier {
    active_screen_size(cx)
        .map(|(w, _)| s::DispTier::for_width(w))
        .unwrap_or(s::DispTier::Hd)
}

/// 主面板尺寸：PANEL_W × panel_height（按档位倍率缩放 · DEC-198；应用所在屏
/// 参照；无显示器 fallback 800 → 640 · Hd 档 = 设计基线原值）。
fn main_panel_size(cx: &App) -> (f32, f32) {
    let sz = active_screen_size(cx);
    let tier = sz
        .map(|(w, _)| s::DispTier::for_width(w))
        .unwrap_or(s::DispTier::Hd);
    let h = sz
        .map(|(_, sh)| panel_height(sh, tier))
        .unwrap_or_else(|| panel_height(800.0, s::DispTier::Hd));
    (s::sp_at(tier, tk::PANEL_W), h)
}

/// Peek 尺寸：DETAIL_W × detail_height（按档位倍率缩放 · DEC-198；应用所在
/// 屏参照；无显示器 fallback 620 → 480 · Hd 档 = 设计基线原值）。
fn detail_size(cx: &App) -> (f32, f32) {
    let sz = active_screen_size(cx);
    let tier = sz
        .map(|(w, _)| s::DispTier::for_width(w))
        .unwrap_or(s::DispTier::Hd);
    let h = sz
        .map(|(_, sh)| detail_height(sh, tier))
        .unwrap_or_else(|| detail_height(620.0, s::DispTier::Hd));
    (s::sp_at(tier, tk::DETAIL_W), h)
}

/// 应用所在屏逻辑像素尺寸 (w, h)（多屏参照 · DEC-76）：DockPrefs.active_display
/// 探针登记的屏 → 主屏（displays().first()）。None 仅无显示器时返回。
fn active_screen_size(cx: &App) -> Option<(f32, f32)> {
    let target = cx.try_global::<DockPrefs>().and_then(|p| p.active_display);
    let d = match target {
        Some(id) => cx.find_display(id),
        None => cx.displays().into_iter().next(),
    };
    d.map(|d| {
        let s = d.bounds().size;
        (f32::from(s.width), f32::from(s.height))
    })
}

/// 应用所在屏双屏帧（DEC-84 · REQ-PANEL-EDGE-TRIGGER 可用区域边缘语义载体）：
/// 按 active_display 探针屏的**全帧尺寸**匹配 platform::screen_areas()（NSScreen
/// frame/visibleFrame 真几何 · Quartz top-left 原点 y 向下，与停靠几何坐标系同
/// 源）；命中 → (全帧, 可用区) 双帧。失配/空表（FFI 失败/无显示器）→ 全帧当可用
/// 兑底。同尺寸多屏辨屏限制登记 DEC-84（按尺寸匹配取首个命中）。
/// 测试面隔离（cfg test）：AppKit NSScreen FFI 不入测试面（TestApp 无真屏语义，
/// 恒走兜底，测试不受测试机分辨率扰动；真几何验证由真机腿承担）。
#[cfg(not(test))]
fn screen_areas_seam() -> Vec<ScreenArea> {
    screen_areas()
}

#[cfg(test)]
fn screen_areas_seam() -> Vec<ScreenArea> {
    Vec::new()
}

fn screen_frames(cx: &App) -> ScreenFrames {
    let (w, h) = active_screen_size(cx).unwrap_or((1280.0, 800.0));
    let areas = screen_areas_seam();
    areas
        .into_iter()
        .find(|a| {
            let (_, _, fw, fh) = a.full;
            fw == w && fh == h
        })
        .map(|a| (a.full, a.usable))
        .unwrap_or(((0.0, 0.0, w, h), (0.0, 0.0, w, h)))
}

/// 可用区域 rect（DEC-84）：停靠/触发/邻接几何的屏参照从全帧收紧到
/// 系统可用区域（排除菜单栏/Dock）。fallback = 全帧当可用。
fn screen_rect(cx: &App) -> dock::ScreenRect {
    screen_frames(cx).1
}

/// Dock 占用态同步（DEC-84 · REQ-PANEL-TRIGGER-POSITION 可用性）：实测三缘占用
/// 落 DockAvailability 全局（值变才写：设置窗在开则即时刷渲染 pill 置灰态）。
fn refresh_dock_availability_with(cx: &mut App, full: dock::ScreenRect, usable: dock::ScreenRect) {
    let next = dock::DockAvailability {
        occupied_left: dock::is_edge_occupied(dock::TriggerPosition::Left, full, usable),
        occupied_right: dock::is_edge_occupied(dock::TriggerPosition::Right, full, usable),
        occupied_bottom: dock::is_edge_occupied(dock::TriggerPosition::Bottom, full, usable),
    };
    let changed = cx
        .try_global::<dock::DockAvailability>()
        .map(|cur| *cur != next)
        .unwrap_or(true);
    if !changed {
        return;
    }
    cx.set_global(next);
    if let Some(handle) = cx.global::<DockHandles>().settings {
        let _ = handle.update(cx, |_, _, cx| cx.notify());
    }
}

/// 面板所在屏探针读取（DEC-76 · DEC-184 五窗同屏锚定源）：DockPrefs
/// .active_display 由 note_active_display 在面板开/收时记录——独立窗口几何
/// 统一锚此屏。None = 探针未立（演示形态/首开前）→ vendor 回退 primary。
fn panel_display(cx: &App) -> Option<DisplayId> {
    cx.try_global::<DockPrefs>().and_then(|p| p.active_display)
}

/// 全局坐标 → 探针屏相对坐标（DEC-184 同屏锚定换算）：dock 几何
///（panel_bounds/peek_bounds/window_rect）产「Quartz top-left 全局」坐标，而
/// vendor open_window 把 bounds.origin 当「目标屏 frame 原点相对坐标」消费
///（screen_frame.origin + bounds.origin 双加，window.rs:1080）——探针屏原点
/// 已知时必须先行扣除，否则副屏开窗双重偏移（DEC-76）。
/// 探针 None 或屏枚举未命中 → 原样返回（primary origin=(0,0) 两轴恒等 ·
/// single-screen 零差异）。
fn to_panel_screen_local(cx: &App, bounds: Bounds<Pixels>) -> Bounds<Pixels> {
    match panel_display(cx).and_then(|d| screen_origin_for_display(u64::from(d))) {
        Some((ox, oy)) => Bounds {
            origin: point(
                px(bounds.origin.x.as_f32() - ox),
                px(bounds.origin.y.as_f32() - oy),
            ),
            size: bounds.size,
        },
        None => bounds,
    }
}

/// 五窗统一装配（DEC-184）：隐藏标题栏（titlebar=None）+ 指定 bounds +
/// 可选最小尺寸 + 探针屏 display_id 同屏锚定 + 整点像素吸附。kind=PopUp
///（置顶 · DEC-79）：NSPanel 非激活面板 + 层级 NSPopUpWindowLevel
///(101) + CanJoinAllSpaces + FullScreenAuxiliary（vendor window.rs PopUp 分支），
/// 应用窗口恒在最顶层（启动/后台/失焦/全屏覆盖/多屏）。movable：主面板
/// true；编辑窗（创建集锦/Peek 编辑接手/handoff）true（同源标题条拖动，
/// DEC-128）；其余独立
/// 窗（Peek/设置/菜单/根选择器演示）false——vendor WindowParams.is_movable →
/// NSWindow.setMovable_ 直映射（L1180）。
fn window_opts_at(
    cx: &App,
    bounds: Bounds<Pixels>,
    min: Option<(f32, f32)>,
    movable: bool,
) -> WindowOptions {
    // 整点像素吸附（DEC-184 黑线根因面）：dock 几何 `(sh-bh)/2` 等运算可产 .5
    // 分数坐标——非整数窗口原点在 Retina 合成下层错位，缝隙漏出 vendor Opaque
    // 纯黑窗底（set_background_appearance 对 Opaque 硬编码 0,0,0,1）呈现为面板
    // 外边缘偶发黑线。round 原点与尺寸（±0.5px 位移无感知语义差异）。
    let bounds = Bounds {
        origin: point(bounds.origin.x.round(), bounds.origin.y.round()),
        size: size(bounds.size.width.round(), bounds.size.height.round()),
    };
    WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        titlebar: None,
        kind: WindowKind::PopUp,
        // 仅主程序 GUI 支持拖曳：主面板 true，其余 false。
        is_movable: movable,
        window_min_size: min.map(|(mw, mh)| size(px(mw), px(mh))),
        // 同屏锚定（DEC-184 · 主程序 GUI 所在屏）：探针 display_id 直通 vendor
        // open_window 的 NSPanel target_screen 精确选屏（window.rs:1076）——
        // 副屏主面板场景五窗不再落主屏。None = 探针未立（演示形态）→ vendor
        // 回退 primary。
        display_id: panel_display(cx),
        ..Default::default()
    }
}

/// 居中装配（Peek --page 演示/设置/菜单/根选择器演示窗 · 均不可拖；
/// DEC-184：居中锚定探针屏——Bounds::centered 按该屏 visible bounds 求
/// 心，坐标天然屏相对）。
fn window_opts(cx: &App, w: f32, h: f32, min: Option<(f32, f32)>) -> WindowOptions {
    window_opts_at(
        cx,
        Bounds::centered(panel_display(cx), size(px(w), px(h)), cx),
        min,
        false,
    )
}

/// 停靠形态判定（DEC-80）：默认主面板与位置演示宿主（token2=位置）参与边缘
/// 唤出守护；五种无语境演示窗（--peek/--settings/--menu/--root/--page）与边缘
/// 唤出域无关，不启守护。
fn is_docked_mode(mode: &str) -> bool {
    !matches!(
        mode,
        "--peek" | "--settings" | "--menu" | "--root" | "--page"
    )
}

fn main() {
    // 单实例保护（DEC-70）：重复启动立即退出，防无关闭钮面板隐匿态下多实例堆叠。
    let instance_guard = single_instance_lock();
    if matches!(instance_guard, InstanceGuard::Busy) {
        eprintln!("Parchora 已在运行 · 本次启动退出");
        return;
    }
    // Held/Pass 守卫活至进程退出（Held drop 即解锁）。
    let _instance_guard = instance_guard;
    // App Nap 对策（DEC-210）：进程级常驻 UserInitiatedAllowingIdleSystemSleep
    // activity——面板关闭（无可见窗口）后系统不放宽本进程定时器节拍，release
    // 载体（LSUIElement 打包）与 cargo run 唤出/排水灵敏度一致。
    parchora_platform::disable_app_nap();

    let mut args = std::env::args().skip(1);
    let mode = args.next().unwrap_or_default();
    // --page 模式下 token2 即页面名；其余模式 token2 按触发位置解析。
    // 触发位置来源：显式 CLI token > 配置档值（init_dock_globals
    // 回填）> 默认右侧（DEC-85）。
    let second = args.next().unwrap_or_default();
    let cli_pos = if mode == "--page" {
        None
    } else {
        TriggerPosition::parse(second.as_str())
    };
    // --page <editor|conflict|ext|picker>：无语境页面演示窗（19 页全承载 · DEC-73）。
    let page = if mode == "--page" {
        second
    } else {
        String::new()
    };

    // 启动惯例：gpui-fast-platform 官方 application() → Application::run（editor-spike 同源）。
    // with_assets 注册内嵌资产表——关于分区彩色 logo 走 Embedded 解析
    //（零新依赖：字节源自 APP_ICON_PNG · gpui 内部 image 解码链）。资产源以
    // with_assets 前置于 run（Application 配置面 · gpui app.rs L204）。
    let app = gpui_platform::application();
    let app = app.with_assets(ParchoraAssets);
    app.run(move |cx: &mut App| {
        // 集合组件库初始化：gpui_component::init +
        // Theme::update 全局色对齐羊皮墨（lib.rs——wrapper 每帧自 Theme 取色，
        // per-state set_editor_style 会被覆盖，须全局）；挂于 init_dock_globals 前
        //（spike 同源挂点）。
        parchora_ui::init(cx);
        init_dock_globals(cx);
        // 持久化存储根（TASK-300 · REQ-CAPTURE-PERSIST）：启动即开（打开失败内存
        // 降级不阻断）；开根后判重播种/保留对账在函数内承载。挂于捕获守护启动前。
        // 仅生产构建（cfg(test) 恒不自动开真机根——init_app_store_at 显式指定）。
        #[cfg(not(test))]
        init_app_store(cx);
        // 输入监控授权态显式探测落值（REQ-PANEL-LISTEN-GRACEFUL「启动时探测」·
        // preflight 纯只读不弹系统提示 · 引导卡按此渲染 · DEC-81）。
        let listen_granted = listen_access_granted();
        cx.set_global(ListenAuth { listen_granted });
        // 可用屏帧与 Dock 占用态（DEC-84）：启动即落 DockAvailability（设置 pill
        // 可用性）+ 播种屏帧缓存（守护拍变化检测基准）。
        let (full, usable) = screen_frames(cx);
        cx.global_mut::<DockHandles>().last_frames = Some((full, usable));
        refresh_dock_availability_with(cx, full, usable);
        // 触发位置：CLI 覆盖（显式 token）→ 冲突解析落值 → 即刻持久化 resolved
        // （档值已由 init_dock_globals 回填；CLI 未给 = 用档值，无档值 = 默认右侧）。
        if let Some(cli) = cli_pos {
            cx.global_mut::<DockPrefs>().trigger = cli;
        }
        let previous = trigger_pos(cx);
        let resolved = dock::resolve_conflict(previous, full, usable);
        if resolved != previous {
            eprintln!(
                "[parchora] 启动触发位 {} 与 Dock 占用冲突 → 自动切换 {}（DEC-84）",
                previous.label(),
                resolved.label()
            );
        }
        cx.global_mut::<DockPrefs>().trigger = resolved;
        save_settings_to_disk(cx, resolved);
        // 启动诊断（DEC-81/84 · 真机判读：授权态 / 活动屏数 / 全帧与可用区 / 触发位）。
        eprintln!(
            "[parchora] 启动诊断（DEC-81/84）：输入监控授权={listen_granted} 活动屏={} 全帧=({:.0},{:.0},{:.0},{:.0}) 可用区=({:.0},{:.0},{:.0},{:.0}) 触发位={}",
            display_bounds_list().len(),
            full.0,
            full.1,
            full.2,
            full.3,
            usable.0,
            usable.1,
            usable.2,
            usable.3,
            resolved.label(),
        );
        eprintln!(
            "[parchora] 配置档：触发位={}（已持久化 · DEC-85；Peek 延迟档已随 DEC-104 裁撤）",
            resolved.label(),
        );
        // 应用图标：品牌 logo → NSApp.applicationIconImage
        // （Accessory 策略可见面 = 通知/NSAlert 程序化场景 · 无 Dock 图标既定）。
        parchora_platform::create_app_icon(APP_ICON_PNG);
        // 状态栏常驻品牌图标（灰版 logo 36px@2x +
        // 左右键统一下拉菜单「呼出主面板/退出」DEC-121——旗标排水守护同款）。
        APP_MODE_DOCKED.store(is_docked_mode(mode.as_str()), Ordering::Relaxed);
        parchora_platform::create_status_item(APP_ICON_STATUS_PNG);
        // 分辨率/缩放档位（DEC-198）：启动全显示器建档（apply_all_displays 自
        // 播 ScreenProfiles 仓）+ 扫描守护（跨屏迁移与档变 → 受控窗整窗刷新）。
        parchora_ui::scale::apply_all_displays(cx);
        start_scale_sweep(cx);
        start_status_click_watch(cx);
        // 主窗吸附守护（拖离贴边自动重泊 · DEC-77）+ 设置窗观测守护（hover-out
        // 自动保存并关窗 · 全形态）+ 光标唤出守护（DEC-79，仅停靠形态 ·
        // DEC-80）：后台轮询，守护间互不依赖。
        start_snap_watch(cx);
        // 释放即吸附守护（DEC-210）：拖动释放后 ≤0.05s 贴边重泊（tap 左键释放
        // 锁存驱动；未授权降级由上方吸附守护兖底）；与设置窗观测守护同型泵频。
        start_drag_release_watch(cx);
        start_settings_watch(cx);
        // 剪贴板捕获守护（DEC-163 · 拾记实时捕获）：changeCount 轮询 + 双阈值清理。
        // 队列引擎拍（TASK-301）同源挂接；激活侦听注册（探查窗事件级精度
        // 源——失败披露后保守判非探查兜底，队列主流程不受影响）。
        #[cfg(not(test))]
        {
            if !pasteboard_watch_activations() {
                eprintln!("[parchora] 队列激活侦听注册失败（探查窗判定无源）");
            }
        }
        start_clipboard_watch(cx);
        // WAL 空闲检点守护（DEC-89 双时机 · 空闲臂）：5 分钟节流 PASSIVE；
        // 退出臂随 quit 链（quit_checkpoint）。守护间互不依赖，同型七号。
        start_db_idle_checkpoint(cx);
        if is_docked_mode(mode.as_str()) {
            start_reveal_watch(cx);
        }
        match mode.as_str() {
            "--peek" => open_peek(cx),
            "--settings" => open_settings(cx),
            "--menu" => open_menu(cx),
            "--root" => open_root_picker(cx),
            "--page" => open_page(cx, &page),
            _ => open_panel_docked(cx),
        }
        // G1 帧遥测挂点（性能轮 DEC-215）：全部模式窗口开完后启动（若 env 未置则空操作）。
        #[cfg(feature = "frame-profiler")]
        frame_telemetry::maybe_start(cx);
    });
}

/// 停靠与预览全局初始化（触发位置默认右 · 句柄空表 · Peek 窗悬停态复位）。
/// 持久化：读配置档回填（档值缺失/解析失败回默认；cfg(test) 读档
/// noop——测试不受真机档位扰动，恒默认起算）。
fn init_dock_globals(cx: &mut App) {
    init_dock_globals_with(cx, load_settings_from_disk());
}

/// 队列宿主态初始化（注入态心 · saved 显式入参）：触发位/双阈值/排除/开关
/// 上限/节流间隔/消费序模式六面档回填（缺失回默认）；队列空起算——DB 行
/// 恢复归 init_app_store_at→reseed_queue_from_db（DEC-212）。
fn init_dock_globals_with(cx: &mut App, saved: AppSettings) {
    cx.set_global(DockPrefs {
        trigger: saved.trigger.unwrap_or_default(),
        active_display: None,
    });
    cx.set_global(DockHandles::default());
    // 档位记账（DEC-198 sweep 专用）：受控窗上次档位，空表起算。
    cx.set_global(ScaleWatch::default());
    // 拾记捕获双阈值（DEC-163/178）：档值缺失/非法 → 常量默认。
    // 前台排除列表（REQ-CAPTURE-EXCLUDE · TASK-300）：档值缺失/空 → 空列表。
    cx.set_global(ClipCapture {
        retention_days: saved
            .clip_retention_days
            .unwrap_or(CLIP_RETENTION_DAYS_DEFAULT),
        valid_min: saved.clip_valid_min.unwrap_or(CLIP_VALID_MIN_DEFAULT),
        exclude_apps: saved
            .capture_exclude_apps
            .map(|raw| {
                raw.split(',')
                    .map(|p| p.trim().to_string())
                    .filter(|p| !p.is_empty())
                    .collect()
            })
            .unwrap_or_default(),
        capture_enabled: saved
            .capture_enabled
            .unwrap_or(CLIP_CAPTURE_ENABLED_DEFAULT),
        limit: saved.capture_limit.unwrap_or(CLIP_LIMIT_DEFAULT),
        ..ClipCapture::default()
    });
    cx.set_global(PeekHover { inside: false });
    // 队列宿主态（DEC-207）：空队列起算（FIFO 默认；DEC-212 档值 LIFO 播种——
    // toggle 单向翻转承载，不入 capture）＋队列序计数器 1 起算；节流间隔档回填
    //（REQ-SETTINGS-QUEUE-NOTIFY-INTERVAL · 档值缺失/非法回默认 5）。
    let mut queue = ClipQueue::new();
    if saved.queue_order == Some(QueueOrder::Lifo) {
        queue.toggle_order();
    }
    cx.set_global(QueueHost {
        queue,
        notifier: QueueNotifier::new(
            saved
                .queue_notify_interval
                .unwrap_or(QUEUE_NOTIFY_INTERVAL_DEFAULT),
        ),
        ..QueueHost::default()
    });
}

// ───── 设置持久化（任一设置项变更即刻持久化）─────

/// 配置档路径（$HOME 下 · 行式 env 档；HOME 缺失 → None——非常规环境防御）。
#[cfg(not(test))]
fn settings_path() -> Option<std::path::PathBuf> {
    let home = std::env::var("HOME").ok()?;
    Some(
        std::path::PathBuf::from(home)
            .join(APP_DIR)
            .join("data/parchora-settings.env"),
    )
}

/// 设置项集合（DEC-163/188 + DEC-209 + DEC-212）：
/// 触发位置 + 剪贴板历史保留时长（天）+ 捕获有效时长（分钟）+ 前台排除应用
///（REQ-CAPTURE-EXCLUDE）+ 捕获监控开关（SCN-CAPTURE-SWITCH-OFF）+ 历史上限
/// 档（REQ-CAPTURE-LIMIT）+ 队列通知节流间隔（REQ-SETTINGS-QUEUE-NOTIFY-
/// INTERVAL · 1~20）+ 队列消费序模式（DEC-212 · queue_order=）。档缺省键以
/// Option 承载（None=未落档，调用方回默认）——避免触发位缺失连坐丢弃其余键。
#[derive(Default)]
struct AppSettings {
    trigger: Option<TriggerPosition>,
    clip_retention_days: Option<u64>,
    clip_valid_min: Option<u64>,
    capture_exclude_apps: Option<String>,
    capture_enabled: Option<bool>,
    capture_limit: Option<u64>,
    queue_notify_interval: Option<u64>,
    queue_order: Option<QueueOrder>,
}

/// 配置档纯函数：设置项序列化（trigger=中文标签；clip_retention_days 天值；
/// clip_valid_min 分钟值；capture_enabled= true/false；capture_limit= 档值；
/// capture_exclude_apps=逗号分隔 bundleId（空列表写出空值承载）；queue_notify_interval=
/// 节流间隔（1~20 缺省即写默认）；queue_order= fifo/lifo（DEC-212 缺省即写默认）；
/// 历史遗留键 peek_delay=（DEC-104）与 clip_survive_min=（DEC-178）不再写出，
/// 解析侧一律忽略）。
fn build_settings_body(s: &AppSettings) -> String {
    let exclude = s
        .capture_exclude_apps
        .as_deref()
        .unwrap_or("")
        .split(',')
        .map(|p| p.trim())
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "trigger={}\nclip_valid_min={}\nclip_retention_days={}\ncapture_enabled={}\ncapture_limit={}\nqueue_notify_interval={}\n{QUEUE_ORDER_KEY}{}\n{CLIP_EXCLUDE_APPS_KEY}{exclude}\n",
        s.trigger.unwrap_or_default().label(),
        s.clip_valid_min.unwrap_or(CLIP_VALID_MIN_DEFAULT),
        s.clip_retention_days.unwrap_or(CLIP_RETENTION_DAYS_DEFAULT),
        s.capture_enabled
            .unwrap_or(CLIP_CAPTURE_ENABLED_DEFAULT),
        s.capture_limit.unwrap_or(CLIP_LIMIT_DEFAULT),
        s.queue_notify_interval
            .unwrap_or(QUEUE_NOTIFY_INTERVAL_DEFAULT),
        queue_order_str(s.queue_order.unwrap_or_default()),
    )
}

/// 配置档纯函数：设置项解析（未知名/非法值/历史遗留键——含 peek_delay=
///（DEC-104）与 clip_survive_min=（DEC-178）——一律忽略；单键独立容错：trigger
/// 非法不影响双阈值读回，
/// 反之亦然；0/负界值同为非法（DEC-184：0 天档会使存活清理即刻清空拾记——
/// 档值钳 ≥1，设置侧 commit_retention 同口径）。
fn parse_settings_body(body: &str) -> AppSettings {
    let mut s = AppSettings::default();
    for line in body.lines() {
        let line = line.trim();
        if let Some(v) = line.strip_prefix("trigger=") {
            s.trigger = TriggerPosition::parse(v.trim());
        }
        if let Some(v) = line.strip_prefix("clip_retention_days=") {
            s.clip_retention_days = v.trim().parse::<u64>().ok().filter(|v| *v >= 1);
        }
        if let Some(v) = line.strip_prefix("clip_valid_min=") {
            s.clip_valid_min = v.trim().parse::<u64>().ok().filter(|v| *v >= 1);
        }
        if let Some(v) = line.strip_prefix(CLIP_EXCLUDE_APPS_KEY) {
            // 逗号分隔 bundleId（trim + 去空项）；非法字符不作校验（同设置行
            // 归一化口径 · DEC-188 实值行承载）。
            let list: Vec<String> = v
                .split(',')
                .map(|p| p.trim().to_string())
                .filter(|p| !p.is_empty())
                .collect();
            s.capture_exclude_apps = Some(list.join(","));
        }
        if let Some(v) = line.strip_prefix("capture_enabled=") {
            match v.trim() {
                "true" => s.capture_enabled = Some(true),
                "false" => s.capture_enabled = Some(false),
                _ => {} // 非法值 = 键缺省（回默认开）
            }
        }
        if let Some(v) = line.strip_prefix("capture_limit=") {
            // 档位合法集门槛（REQ-CAPTURE-LIMIT · 非 set 内值视为缺省回默认）。
            s.capture_limit = v
                .trim()
                .parse::<u64>()
                .ok()
                .filter(|n| CLIP_LIMIT_STEPS.contains(n));
        }
        if let Some(v) = line.strip_prefix(QUEUE_NOTIFY_INTERVAL_KEY) {
            // 1~20 整数合法域门槛（REQ-SETTINGS-QUEUE-NOTIFY-INTERVAL · 越界拒绝=键缺省回默认）。
            s.queue_notify_interval =
                v.trim().parse::<u64>().ok().filter(|n| {
                    (QUEUE_NOTIFY_INTERVAL_MIN..=QUEUE_NOTIFY_INTERVAL_MAX).contains(n)
                });
        }
        if let Some(v) = line.strip_prefix(QUEUE_ORDER_KEY) {
            // fifo/lifo 值域门槛（DEC-212 · 非法值拒绝=键缺省回 FIFO）。
            s.queue_order = queue_order_parse(v);
        }
    }
    s
}

/// 读配置档（cfg(test) noop：TestApp 恒默认起算，真机档位不扰动测试）。
#[cfg(test)]
fn load_settings_from_disk() -> AppSettings {
    AppSettings::default()
}

#[cfg(not(test))]
fn load_settings_from_disk() -> AppSettings {
    let path = match settings_path() {
        Some(p) => p,
        None => return AppSettings::default(),
    };
    match std::fs::read_to_string(path) {
        Ok(body) => parse_settings_body(&body),
        // NotFound＝档未建（首启常态·静默回默认）；其余错误披露后回默认
        //（R184 建议 3——权限/IO 异常不再无声吞没）。旧版 Err(_) 同吞 NotFound。
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => AppSettings::default(),
        Err(e) => {
            eprintln!("[parchora] 配置档读取失败（回默认档）：{e}");
            AppSettings::default()
        }
    }
}

/// 写配置档（cfg(test) noop：测试不落真机档；生产 = create_dir_all + tmp+rename
/// 原子写，幂等——同值重写无害。无 fsync：演示宿主档丢失可容忍，产品轮入 files 层）。
/// 双阈值自 ClipCapture 全局读取（单一事实源 · 守护态即运行态 · 保留时长
/// 天制 DEC-178）。
#[cfg(test)]
fn save_settings_to_disk(_cx: &App, _trigger: TriggerPosition) {}

#[cfg(not(test))]
fn save_settings_to_disk(cx: &App, trigger: TriggerPosition) {
    let cap = cx.global::<ClipCapture>();
    let notify = cx
        .try_global::<QueueHost>()
        .map(|h| h.notifier.interval())
        .unwrap_or(QUEUE_NOTIFY_INTERVAL_DEFAULT);
    let order = cx
        .try_global::<QueueHost>()
        .map(|h| h.queue.order())
        .unwrap_or_default();
    let body = build_settings_body(&AppSettings {
        trigger: Some(trigger),
        clip_retention_days: Some(cap.retention_days),
        clip_valid_min: Some(cap.valid_min),
        capture_exclude_apps: Some(cap.exclude_apps.join(",")),
        capture_enabled: Some(cap.capture_enabled),
        capture_limit: Some(cap.limit),
        queue_notify_interval: Some(notify),
        queue_order: Some(order),
    });
    let Some(path) = settings_path() else {
        return;
    };
    if let Some(dir) = path.parent() {
        if let Err(e) = std::fs::create_dir_all(dir) {
            eprintln!("[parchora] 配置档目录创建失败：{e}");
            return;
        }
    }
    let tmp = path.with_extension("env.tmp");
    if let Err(e) = std::fs::write(&tmp, body) {
        eprintln!("[parchora] 配置档写入失败：{e}");
        return;
    }
    if let Err(e) = std::fs::rename(&tmp, &path) {
        eprintln!("[parchora] 配置档落定失败：{e}");
        let _ = std::fs::remove_file(&tmp);
    }
}

/// 当前触发位置（DockPrefs 全局单写 · 与 ui 侧触发位置 pill 同源；冲突解析后落值）。
fn trigger_pos(cx: &App) -> TriggerPosition {
    cx.global::<DockPrefs>().trigger
}

/// 探针登记应用所在屏（多屏参照 · DEC-76）：主程序（面板）关收时经
/// window.display 取实时所在屏 → DockPrefs.active_display（下次开窗按该屏取几何）。
/// vendor 拖屏拖动后 on_moved → display_changed 更新 display_id（window.rs:2827-2829）。
fn note_active_display(window: &mut Window, cx: &mut App) {
    if let Some(d) = window.display(cx) {
        cx.global_mut::<DockPrefs>().active_display = Some(d.id());
    }
}

/// 把 app 层动作延迟出当前租借上下文执行（DEC-77）：gpui-fast 实体租借独占语义（entity_map.rs lease 把实体移出 map，
/// 嵌套同实体 update 即 double_lease_panic abort）下，view 的 listener/sink 转发
/// 链执行期间对应实体处于租借态——sink 内同步触同实体窗口句柄必崩（真机崩溃栈：
/// open_peek_item → panel_actual_bounds → WindowHandle::update → lease_erased）。
/// 动作统一 spawn 到租借释放后的下一调度拍（arm_peek_close 同款先例）；进程退出
/// 时 executor 随 quit 停止，未决 defer 不再被轮询。仅触其它实体/全局的分支不走
/// 此通道（最小改动）。
fn defer_app(cx: &App, f: impl FnOnce(&mut App) + 'static) {
    cx.spawn(async move |cx| {
        cx.update(f);
    })
    .detach();
}

fn close_panel(cx: &mut App) {
    // 级联关详情窗（SCN-PANEL-HIDE-RETREAT「内容详情窗一并收起，不残留孤立
    // 窗口」· DEC-94）：bump gen 使未决 peek-close timer 失效（对先关路径幂等）。
    bump_peek_close_gen(cx);
    close_peek(cx, true); // 级联强关（DEC-159）：面板随行，错误面无处承载（stderr 留档）
    let handle = cx.global_mut::<DockHandles>().panel.take();
    // DEC-181 显隐保持：整窗移除前记录活选项卡（宿主唯一收口——Esc 级联末级/
    // 移出收回/守护拍/点击外/换泊重泊/退出链全漏斗此）；重开由
    // open_panel_docked 恢复（restore_view）。DEC-186：同一闭包内先行提取面板
    // 强句柄（remove_window 之后 Context 已亡、取不到）＋面板侧 Pin 复位
    //（复用实体重开不得带出陈旧 Pin · 与全局 pinned=false 同源）。
    let mut saved_entity = None;
    let saved = if let Some(handle) = handle {
        handle
            .update(cx, |panel, window, cx| {
                let view = panel.current_view();
                saved_entity = Some(cx.entity());
                panel.reset_pin(cx);
                note_active_display(window, cx);
                window.remove_window();
                view
            })
            .ok()
    } else {
        None
    };
    let handles = cx.global_mut::<DockHandles>();
    handles.panel_view = saved;
    handles.panel_entity = saved_entity;
    handles.revealed = false;
    handles.panel_probe_armed = false;
    handles.close_streak = 0;
    handles.seam_grace_ticks = 0;
    handles.exit_grace_ticks = 0;
    handles.pinned = false;
}

/// 用户面自动隐藏挂起（DEC-184）：四自动链（MouseExitHide / Peek 悬停离场
/// 连带 / 守护两拍确认 / 点击外）统一经此——挂 1s 复核 timer；兑时
/// 逐项复核（代次现役 / 非 Pin / 面板在场 / cfg(test) 外光标仍离板离带）才
/// 执行 close_panel（级联语义不变）。Peek 在悬不挂由各链上游自己的保持域
/// 把关（MouseExitHide/guard/probe 均已 peek 门；点击外路径不带门——级联收
/// 详情窗）；回板回域取消由守护拍复位块承接。Pin：不挂（压倒一切
/// · DEC-94）。重复 arm 自 bump 代次取旧挂新。
fn arm_panel_close(cx: &mut App) {
    if cx.global::<DockHandles>().pinned {
        return;
    }
    if cx.global::<DockHandles>().panel.is_none() {
        return; // 面板不在场：无隐藏对象
    }
    let (generation, held) = {
        let handles = cx.global_mut::<DockHandles>();
        handles.close_pending_gen += 1;
        (handles.close_pending_gen, true)
    };
    let _ = held;
    let task = cx.spawn(async move |cx| {
        cx.background_executor()
            .timer(Duration::from_secs_f32(PANEL_CLOSE_DELAY_SECS))
            .await;
        cx.update(move |cx| panel_close_confirm(cx, generation));
    });
    cx.global_mut::<DockHandles>().close_pending_task = Some(task);
}

/// 挂起 timer 兑时复核（arm 唯一消费口）：代次现役且复核域全过 → close_panel
///（级联关 Peek/记账/复位全沿 close_panel 唯一收口）。
fn panel_close_confirm(cx: &mut App, generation: u64) {
    // 先清持活槽（本 task 已完结，防下一次 cancel 误判「有挂起」）。
    cx.global_mut::<DockHandles>().close_pending_task = None;
    if cx.global::<DockHandles>().close_pending_gen != generation {
        return; // 已被取消/重挂：代次失效
    }
    if cx.global::<DockHandles>().pinned {
        return; // 挂起期间被 Pin（SCN-PANEL-PIN-BLOCKS-HIDE · DEC-94）：不收
    }
    let Some(handle) = cx.global::<DockHandles>().panel else {
        return; // 面板已他途关闭：无隐藏对象
    };
    // 回域终审（DEC-95 测试先例：cfg(test) 光标不可读，按离场口径放行——测试
    // 确定性；生产单次光标读「热区∪面板」复核，仍在场 → 取消不收）。handle
    // 仅 cfg(not(test)) 光标复核消费（panel_contains）。
    #[cfg(test)]
    let _ = handle;
    #[cfg(test)]
    let gone = true;
    #[cfg(not(test))]
    let gone = match cursor_position() {
        Some((x, y)) => {
            let trigger = trigger_pos(cx);
            let zone = dock::trigger_zone_bounds(trigger, screen_rect(cx));
            !(dock::point_in_zone(x, y, zone) || panel_contains(cx, &handle, x, y))
        }
        None => true,
    };
    if !gone {
        return; // 回域取消（不重挂：守护拍后续离场自行再确认）
    }
    close_panel(cx);
}

/// 挂起取消（任一取消源共用）：丢 timer 持活槽（Task drop 撤销）+ bump 代次
/// 双保险（timer 已入队未兑时也必失效）；幂等——无挂起时零扰动。
fn cancel_panel_close(cx: &mut App) {
    let handles = cx.global_mut::<DockHandles>();
    if handles.close_pending_task.take().is_some() {
        handles.close_pending_gen += 1;
    }
}

/// 编辑窗统一退出 → 主界面恢复出口（DEC-178「面板隐藏时点返回 → 先关当前
/// 面板，再显示主界面」）：editor_exit 落 true（窗确已关 · reveal 路窗保持
/// 开启不触发）→ ensure_panel_visible。Esc 与返回钮同源统一事件（DEC-129/151）。
fn editor_escape_exit(cx: &mut App) {
    if editor_exit(cx) {
        ensure_panel_visible(cx);
    }
}

/// 主面板显式恢复（DEC-178）：句柄活 = 在场 → 直返（幂等不重复显示）；None 或
/// 陈旧句柄 = 隐藏态 → 清登记后重开停靠面板（open_panel_docked 单例守卫不冲突；
/// close_panel 整窗移除语义 = 显主界面即重开）。仅自双面板回工作台出口挂用
///（编辑窗 Escape 漏斗 / 待办创建三漏斗）；quit 链内 close_panel/close_editor/
/// close_task_create 直呼不走此路（退出窗重开无效，勿闪板）。
fn ensure_panel_visible(cx: &mut App) {
    let alive = match cx.global::<DockHandles>().panel {
        Some(handle) => handle.update(cx, |_, _, _| ()).is_ok(),
        None => false,
    };
    if alive {
        return;
    }
    cx.global_mut::<DockHandles>().panel = None;
    open_panel_docked(cx);
}

/// 关闭 Peek 窗（无窗幂等；复位窗内悬停态，防跨窗陈旧标志）。软/强两档
///（DEC-159）：软（独立关窗——Esc/悬停宽限/单例关 Peek）冲刷保存失败 → 窗
/// 保留 + bar 错误条幅 + 失败条目寄存（下次关窗重试，引擎/在编缓冲原封）——
/// 编辑不静默丢失；强（级联/quit/换泊/编辑器转出——宿主消亡随行、错误面无可
/// 承载）失败仅 stderr 留档照常关窗。面板内编辑冲刷兑底（DEC-156 关窗即冲刷
/// 点 · editor_exit 同构）：root 显式入参（temp root 测试；生产 files_root()）。
fn close_peek_at(cx: &mut App, root: &std::path::Path, force: bool) {
    let handle = cx.global_mut::<DockHandles>().peek.take();
    cx.set_global(PeekHover { inside: false });
    let Some(handle) = handle else { return };
    // 冲刷探测（DEC-159 · 只读不摘引擎）：编辑在编脏位（引擎活值 × 基线）+
    // 失败寄存重试项。探测零副作用：失败驻留路径引擎/基线/缓冲原封不动。
    let probe = handle
        .update(cx, |view, _window, cx| view.flush_probe(cx))
        .ok();
    let mut failures: Vec<(PeekItem, String)> = Vec::new();
    if let Some(probe) = probe {
        // 保存次序：编辑在编脏位优先（最新意图），失败寄存项次之（异题遗留）；
        // 同题在编覆盖时跳过寄存重试（在编保存即同稿覆写，免双写双同步）。
        let edit = probe.edit;
        // 寄存条目集逐项消费（R184 F-183-08 Vec 化 · 多失败并存不再互相覆写）：
        // 同题在编覆盖的寄存项逐项跳过（在编保存即同稿覆写，免双写双同步）。
        let pending: Vec<PeekItem> = probe
            .pending
            .into_iter()
            .filter(|p| {
                !(edit
                    .as_ref()
                    .is_some_and(|(e, changed)| *changed && e.title == p.title))
            })
            .collect();
        if let Some((item, true)) = edit {
            if let PeekSaveOutcome::Failed(err) = save_peek_edit_at(root, &item, cx) {
                failures.push((item, err));
            }
        }
        for item in pending {
            if let PeekSaveOutcome::Failed(err) = save_peek_edit_at(root, &item, cx) {
                failures.push((item, err));
            }
        }
    }
    if !failures.is_empty() && !force {
        // 软关失败驻留（DEC-159）：bar 错误条幅 + 失败寄存（下次关窗重试），
        // 句柄放回——冲刷与窗亡留待重试成功或用户显式处理。
        let _ = handle.update(cx, |view, _window, cx| {
            for (item, err) in &failures {
                view.mark_save_failed(item.clone(), err, cx);
            }
        });
        cx.global_mut::<DockHandles>().peek = Some(handle);
        return;
    }
    // 收尾（成功保存 / 无保存需要 / 强制）：摘引擎回落只读 + 窗移除。
    let _ = handle.update(cx, |view, _window, cx| {
        let _ = view.tear_down_edit(cx);
    });
    let _ = handle.update(cx, |_, window, _| window.remove_window());
}

/// 生产入口（files 根 · DEC-159 软/强两档）。
fn close_peek(cx: &mut App, force: bool) {
    close_peek_at(cx, &files_root(), force);
}

/// 主面板按当前触发位置贴边停泊（可用区域参照 · DEC-84：左右缘垂直居中 · 底部
/// 宽高互换水平居中 · 四角角锚）。
fn open_panel_docked(cx: &mut App) {
    if cx.global::<DockHandles>().panel.is_some() {
        return;
    }
    cancel_panel_close(cx); // 重开即撤延时收起挂起（DEC-184 · 唤出/toggle/换泊共用）
    let pos = trigger_pos(cx);
    let tier = active_tier(cx); // DEC-198：min 尺寸随档位倍率
    let (x, y, w, h) = dock::panel_bounds(pos, screen_rect(cx), main_panel_size(cx));
    let opts = window_opts_at(
        cx,
        // dock 几何为全局坐标（DEC-184）：先扣探针屏原点换屏相对坐标再装配。
        to_panel_screen_local(cx, Bounds::new(point(px(x), px(y)), size(px(w), px(h)))),
        Some(dock::swap_dims_if_horizontal(
            pos,
            (s::sp_at(tier, tk::PANEL_W), s::sp_at(tier, tk::PANEL_H_MIN)),
        )),
        true,
    );
    // DEC-181 显隐保持：恢复宿主记账的活选项卡（close_panel 唯一收口记录·
    // None = 无记录首开走默认集锦 · DEC-78 口径不变）。
    let saved = cx.global::<DockHandles>().panel_view;
    // 实体复用（DEC-186 根修）：close_panel 整窗移除前提取的强句柄在场 →
    // 复用整实体（拾记行/待办行/面板内态跨显隐周期持久；事件 sink 纯 app
    // 回调随实体保留不重装，弱引用转发链同实体 ID 依旧活）；None = 首开或
    // 无提取（句柄它途失效兜底）→ 全新构造（原链）。
    let reused = cx.global_mut::<DockHandles>().panel_entity.take();
    // 全新实体判定先于 move（open_window 闭包消费 reused——回填/冲刷分路依赖）。
    let fresh = reused.is_none();
    let handle = match cx.open_window(opts, |window, cx| {
        // 探针首立（DEC-184 · 同屏锚定链第一环）：面板开窗即记所在屏——
        // close_panel 已在收侧记账，这里补开侧；自此首会话即可正确锚定
        // 五窗几何（首开前探针 None 只影响无面板语境的演示窗）。
        note_active_display(window, cx);
        let panel = match reused {
            Some(entity) => {
                entity.update(cx, |panel, cx| {
                    // Pin 已在 close_panel 复位（与全局同源）；选项卡恢复
                    // 仍走 DEC-181 记账口径（复用实体 view 与镜像同源 ·
                    // 幂等落值＋notify，选中态随首帧同步）。
                    if let Some(v) = saved {
                        panel.restore_view(v, cx);
                    }
                });
                entity
            }
            None => cx.new(|cx| {
                let mut panel = PanelView::new(cx);
                if let Some(v) = saved {
                    panel.restore_view(v, cx);
                }
                panel.set_event_sink(Box::new(|ev: &ShellEvent, cx: &mut App| match ev {
                    ShellEvent::OpenSettings => open_settings(cx),
                    ShellEvent::MouseExitHide => {
                        // 移出收回（Pin 态 PanelView 不上抛）：收缩态 = 主面板整窗
                        // 关闭全隐（进程保持）。移出瞬间光标仍滞留触发热区（顺缘穿
                        // 越触发热区离板——如沿触发面向缘侧/角侧滑出）或面板离界缓冲
                        // 带外扩矩形（DEC-100
                        // 「不得刚离边界即瞬隐」）→ 不收：面板可见 = 光标在「热区
                        // ∪ 面板 ∪ 离界缓冲带 ∪ 缝带」域内的规格语义
                        // （REQ-PANEL-EDGE-TRIGGER 展示态）；出带后的持续离场走
                        // 守护缓冲/两拍常规收起。Peek 阅读/行程态抑制：
                        // 预览窗在悬 → 面板保持（SCN-PREVIEW-SCROLL-IN-PREVIEW
                        // 关闭路径交给 Peek 窗）。动作延迟出 PanelView 租借
                        //（on_hover listener 转发链持租）——同步 close_panel 触
                        // 同实体句柄 = double lease abort（DEC-77）。
                        defer_app(cx, |cx| {
                            if cx.global::<DockHandles>().peek.is_some() {
                                return;
                            }
                            // 缝侧邻接宽限带旁路（DEC-95 · 沿 arm_peek_close
                            // cfg(test) 先例）：测试态 cursor_position 读真机物理
                            // 光标不可判读——测试按「离板即收」口径直收，宽限/缓
                            // 冲挂起的确定性覆盖交守护注入式测试；生产单次光标读
                            // 做「热区 ∪ 缝带 ∪ 离界缓冲带」三段保持（None →
                            // fallthrough 收）。
                            #[cfg(test)]
                            let keep_open = false;
                            #[cfg(not(test))]
                            let keep_open = match cursor_position() {
                                Some((cx_, cy)) => {
                                    let trigger = trigger_pos(cx);
                                    let zone = dock::trigger_zone_bounds(trigger, screen_rect(cx));
                                    dock::point_in_zone(cx_, cy, zone)
                                        || dock::in_seam_grace_band(trigger, cx_, cy, zone)
                                        || panel_exit_band_contains(cx, cx_, cy)
                                }
                                None => false,
                            };
                            if keep_open {
                                return; // 光标仍在触发位置/缝侧宽限带/离界缓冲带：面板保持
                            }
                            // 延时收起（DEC-184）：用户面自动隐藏不即收，1s 复核。
                            arm_panel_close(cx);
                        });
                    }
                    // Esc 级联末级（无次级浮层）→ 收面板（§〇-4 · DEC-73；收缩态
                    // 全隐同上）；on_key_down listener 链持租，动作
                    // 延迟出借域（DEC-77）。
                    // 队列持板先补写（SCN-PASTE-ESCAPE-QUEUE-OWNED）：Esc 是
                    // 中断手势（区别于点击外/自动隐藏的隐匿手势——队列继
                    // 续后台运行）；placed 项真实数据写回＋停止全链后再
                    // 关面板。
                    ShellEvent::HidePanel => defer_app(cx, |cx| {
                        if cx.global::<QueueHost>().engine.holding().is_some() {
                            // Esc 中断停止也进提示编排（§11.2 停止行：补写完成
                            // 通知——TASK-716 事件流单点 queue_notify_events）。
                            let evs = queue_stop_full(cx, &QueueAction::Stop)
                                .into_iter()
                                .collect::<Vec<_>>();
                            queue_notify_events(cx, evs);
                        }
                        close_panel(cx);
                    }),
                    // 行单击 → Peek 开窗/原位更新：open_peek_item → panel_actual_bounds
                    // 触主面板同实体句柄（行单击 → forward.update 转发链持租期
                    // 必崩），延迟出借域（DEC-77）。
                    ShellEvent::PeekShow(item) => {
                        let item = item.clone();
                        defer_app(cx, move |cx| open_or_update_peek(cx, item));
                    }
                    // Pin 翻转 → 全局口径同步（DEC-94）：四隐藏路径读单一事实源。
                    // 仅触全局，同步直呼（租借铁律豁免 · DEC-77）。
                    ShellEvent::PinChanged(pinned) => {
                        cx.global_mut::<DockHandles>().pinned = *pinned;
                    }
                    // QuickNote → 集锦页 FAB「新建笔记」上抛（DEC-129）：开/热换编辑窗
                    // 空稿（Untitled · 落盘随 editor_exit 漏斗 files 原子写，正文空
                    // 不产生文件）；OpenQueue：面板内视图内部态承载（演示宿主不跨窗）。
                    ShellEvent::QuickNote => defer_app(cx, open_editor_new),
                    // QuickTask → 待办页 FAB「新建任务」上抛（DEC-165）：宿主开/
                    // 激活「创建待办事项」独立面板窗。FAB on_click listener 链持
                    // BoardView 租借——延迟出借域（DEC-77 同构）。
                    ShellEvent::QuickTask => defer_app(cx, open_task_create),
                    // ClipToNote → 拾记右键「转为笔记」（DEC-194）：宿主开/热换
                    // 编辑窗预填条目原文。on_click listener 链持 CaptureView 租借
                    // ——延迟出借域（DEC-77 同构）。
                    ShellEvent::ClipToNote(content) => {
                        let content = content.clone();
                        defer_app(cx, move |cx| open_editor_from_clip(cx, content));
                    }
                    // ClipToTask → 拾记右键「转为任务」（DEC-194）：宿主开/激活
                    // 创建面板并预填「任务」域。延迟出借域同构。
                    ShellEvent::ClipToTask(content) => {
                        let content = content.clone();
                        defer_app(cx, move |cx| open_task_create_from_clip(cx, content));
                    }
                    // TrashNote → 集锦首页笔记行「删除→回收站」（DEC-164）：宿主
                    // 真移 `.trash/` 并回灌列表；菜单 on_click listener 链持
                    // CollectionView 租借——延迟出借域（DEC-77 同构）。
                    ShellEvent::TrashNote(title) => {
                        let title = title.clone();
                        defer_app(cx, move |cx| trash_note(cx, &title));
                    }
                    _ => {}
                }));
                panel
            }),
        };
        // TASK-300 持久化链装配（复用/全新两路统一：sink 恒槽覆写幂等；
        // 捕拾删除/待办事件 → 宿主事务写；同步 DB 写不触租借域窗口操作）。
        panel.update(cx, |panel, cx| {
            panel.set_clip_delete_sink(
                std::rc::Rc::new(|db_id: &str, cx: &mut App| {
                    clip_delete_from_db(cx, db_id);
                }),
                cx,
            );
            panel.set_task_db_sink(
                std::rc::Rc::new(|ev: &TaskDbEvent, cx: &mut App| {
                    task_db_event(cx, ev);
                }),
                cx,
            );
            panel.set_trash_action_sink(
                std::rc::Rc::new(|act: &TrashAction, cx: &mut App| {
                    // 回收站动作（DEC-195）：物理删除后 sync_trash_notes 会
                    // update 同一 CollectionView 租借域 → 必须 defer 出借域。
                    let act = act.clone();
                    defer_app(cx, move |cx| match act {
                        TrashAction::Purge(title) => purge_trash_note(cx, &title),
                        TrashAction::Empty => empty_trash(cx),
                    });
                }),
                cx,
            );
            // 队列动作（DEC-207 · A1 队列接线）：上拋宿主 mutator 后回推
            // 同一 CaptureView 租借域 → defer 出借域（TrashAction 同构）。
            panel.set_queue_action_sink(
                std::rc::Rc::new(|act: &QueueAction, cx: &mut App| {
                    let act = act.clone();
                    defer_app(cx, move |cx| queue_event(cx, &act));
                }),
                cx,
            );
        });
        // 键焦定向面板根（Esc 级联 on_key_down 经 focus 路径分派 · DEC-73）。
        window.focus(&panel.read(cx).focus_handle(), cx);
        panel
    }) {
        Ok(handle) => handle,
        Err(e) => {
            eprintln!("[parchora] 主面板开窗失败（降级：本拍不装配主面板）：{e}");
            return;
        }
    };
    let handles = cx.global_mut::<DockHandles>();
    handles.panel = Some(handle);
    // 守护基准与拖动基线重置：新窗待首拍播种（重泊/重开/新触发位均重吸 macOS 钳制）。
    handles.docked_bounds = None;
    handles.drag_last = None;
    handles.panel_probe_armed = false;
    handles.close_streak = 0;
    handles.seam_grace_ticks = 0;
    handles.exit_grace_ticks = 0;
    // 面板唤出 → 集锦列表磁盘真实面同步（DEC-144 · 唤出即取最新落盘面；
    // 无面板/句柄失效在 sync 内 no-op，编辑窗落盘期间面板未挂的补同步收口）。
    sync_disk_notes(cx);
    // 回收站真实面同步（DEC-195 · 镜像口径：唤出即取最新 .trash 面）。
    sync_trash_notes(cx);
    // 持久化种子与关窗期捕获缓冲（TASK-300 + DEC-163/186）：
    // - 全新实体 + 存储在场 → DB 回填整表落板（拾记行全集 + 待办行集/序号续接
    //   + 存活清理），pending 仅剩孤儿（无 DB 身份）补写后冲入——DB 已含全部
    //   已落库行，全量冲入会重复；
    // - 复用实体（显隐周期）→ 会话内存行即最新面，pending 全量按序冲入（DEC-186
    // 口径不变；孤儿行同拍补写）。
    // 冲刷自 app 全局逐条转发（复用实体同窗新挂，无租借竞争面 · DEC-186），
    // 失败（窗已闭）静默弃置——缓冲生命周期只到下一次冲刷机会。
    let store_present = app_db(cx).is_some();
    if fresh && store_present {
        backfill_panel_from_db(cx, &handle);
    }
    let pending: Vec<ClipRow> = std::mem::take(&mut cx.global_mut::<ClipCapture>().pending);
    for mut row in pending {
        // 入列态先行判定：已在 DB 的行（捕获期已落库）在 fresh 种子中已含，冲入
        // 会重复；孤儿行（无 DB 身份）不在种子中，补写成功后照常入列。
        let already_seeded = fresh && store_present && row.db_id.is_some();
        if row.db_id.is_none() && store_present {
            // 孤儿行补写（写失败期捕获 · 保序重试一次）：先查重试队列同刻载荷
            //（DEC-188——image/富类型行不在文本重建面，但 payloads
            // 随队列在册可补写；命中即摘队防双写），无队列载荷时回退文本行
            // 重建（DEC-187 文本孤儿面）。
            let from_retry = {
                let cap = cx.global_mut::<ClipCapture>();
                let idx = cap.db_retry.iter().position(|p| p.at == row.added_at);
                idx.map(|i| cap.db_retry.remove(i))
            };
            let payload = from_retry.or_else(|| orphan_payload_for_row(&row));
            if let Some(payload) = payload {
                if let Persisted::Done(id) = persist_capture(cx, &payload) {
                    row.db_id = Some(id);
                }
            }
        }
        if already_seeded {
            continue;
        }
        let _ = handle.update(cx, |view, _w, cx| view.ingest_row(row, cx));
    }
    if fresh && store_present {
        clip_sweep(cx); // 回填行按原捕获时刻回拨——超阈存量首拍即清
    }
    // 队列镜像回灌（DEC-207）：app 层 QueueHost＝队列单一事实源，面板重开
    // 统一幂等回推（消费序已排齐）——队列条目跨显隐周期随实体复用存续
    //（DEC-186 同向·镜像面无丢失）。
    push_queue_to_panel(cx, &handle);
}

/// 待办序号续接受助纯函数（R184 F-183-03 收敛点）：DB 数值聚合（max_identifier_seq
/// 的 ok 承载）与会话行解析极大值取大后 +1——DB 读败/空表/脏 identifier 全滤
///（聚合缺值）时行内极大值兑底，双源皆空 → 起算 1。
fn next_seq_after(max_res: Option<Option<u64>>, rows_max: usize) -> usize {
    max_res.flatten().unwrap_or(0).max(rows_max as u64) as usize + 1
}

/// DB 回填落板（TASK-300 启动恢复 · 仅全新实体 + 存储在场）：拾记行全集
///（created_at 倒序 · UI 行序 = 最新在顶；Instant 按捕获墙钟回拨——清存活
/// 阈值跨重启仍然真实）+ 待办行集与 next_seq 续接（max(identifier)+1）。
fn backfill_panel_from_db(cx: &mut App, handle: &gpui::WindowHandle<PanelView>) {
    let Some(db) = app_db(cx) else { return };
    let now_wall = walltime::now_epoch();
    // 拾记行：DB 行 → 捕拾行（kind 对账 · 标题诚实承载 · image 无正文占位；
    // thumb 文件在档 → 行真图，缺档/读败 → 棋盘占位回退）。
    let clip_rows: Vec<ClipRow> = match list_clips(&db) {
        Ok(clips) => {
            // blob 读档随活跃根（DEC-189 · 与落盘同域）：根快照一次逐行取——
            // 行内反复探根是热循环浪费面。
            let blob_root = store_root(cx);
            // 播种过滤（R184 F-183-04 · DEC-163 谓词原文）：未处理窗口（valid_min
            // 分钟）已过的行不播种——UI 存活清理首拍即清的复活面收口（跨重启
            // 不再闪现即逝）；schema 无 handled 列（短期全行视作未处理，v3 候选
            // 规范 7 在案）；谓词未处理轴 Pin 不豁免（DEC-163 谓词原文）。过滤
            // 置于 thumb 读前——被跳过行不付 blob IO。
            let valid_min_secs =
                i64::try_from(cx.global::<ClipCapture>().valid_min.saturating_mul(60))
                    .unwrap_or(i64::MAX);
            clips
                .into_iter()
                .filter_map(move |c| {
                    let wall = epoch_from_iso(&c.created_at).unwrap_or(0);
                    if now_wall.saturating_sub(wall) >= valid_min_secs {
                        return None;
                    }
                    let content_type = c.content_type;
                    let at = backdated_instant(now_wall, wall);
                    let thumb: Option<std::sync::Arc<gpui::Image>> = c
                        .thumb_rel_path
                        .as_ref()
                        .and_then(|rel| {
                            std::fs::read(blob_root.as_ref()?.join("data").join(rel)).ok()
                        })
                        .map(|bytes| {
                            std::sync::Arc::new(Image::from_bytes(ImageFormat::Png, bytes))
                        });
                    let payload = CapturedPayload {
                        content_type,
                        text: c.text_content,
                        bytes: None,
                        hash: c.content_hash.clone(),
                        source_app: c.source_app.clone(),
                        thumb: None,
                        wall,
                        at,
                    };
                    let mut row = capture_row(&payload, Some(c.id.to_string()));
                    if let Some(thumb) = thumb {
                        row.thumb_img = Some(thumb);
                    }
                    Some(row)
                })
                .collect()
        }
        Err(e) => {
            eprintln!("[parchora] 拾记回填读失败：{e}");
            Vec::new()
        }
    };
    // 待办行：DB 行 → 序号续接（identifier 解析失败行跳过——不虚构序号）。
    let (board_rows, next_seq) = match list_todos(&db) {
        Ok(todos) => {
            let rows = todos
                .into_iter()
                .filter_map(|t| {
                    // identifier 解析失败的行跳过（不虚构序号链）；行序已由 list
                    // 的数值序承载，行内不重带 seq。
                    if t.identifier
                        .strip_prefix("TASK-")?
                        .parse::<usize>()
                        .is_err()
                    {
                        return None;
                    }
                    Some((
                        t.identifier,
                        t.title,
                        status_index(t.status),
                        bucket_index(t.schedule_bucket),
                        priority_index(t.priority),
                        t.mark,
                    ))
                })
                .collect::<Vec<_>>();
            let rows_max = rows
                .iter()
                .filter_map(|(id, _, _, _, _, _)| id.strip_prefix("TASK-")?.parse::<usize>().ok())
                .max()
                .unwrap_or(0);
            // 序号续接受助（R184 F-183-03）：DB 聚合主导、行内极大值兑底。
            (rows, next_seq_after(max_identifier_seq(&db).ok(), rows_max))
        }
        Err(e) => {
            eprintln!("[parchora] 待办回填读失败：{e}");
            (Vec::new(), 1)
        }
    };
    let rows_len = clip_rows.len();
    let _ = handle.update(cx, |view, _w, cx| {
        view.seed_data(clip_rows, board_rows, next_seq, cx)
    });
    if rows_len > 0 || next_seq > 1 {
        eprintln!(
            "[parchora] 启动回填：拾记 {rows_len} 条 / 待办序号续接 TASK-{}",
            next_seq.saturating_sub(1)
        );
    }
}

/// 墙钟回拨单调时刻（Instant 年龄语义修复 · TASK-300）：DB 行按捕获时刻回拨，
/// 存活/有效清理与真实年龄同轴；超进程运行时长的年龄回到 now（sweep 首拍自
/// 然清理，行为等价）。
fn backdated_instant(now_wall: i64, wall: i64) -> Instant {
    let age = now_wall.saturating_sub(wall).max(0) as u64;
    Instant::now()
        .checked_sub(Duration::from_secs(age.min(u64::MAX / 2)))
        .unwrap_or_else(Instant::now)
}

/// 待办右键事件 → DB 事务写（TASK-300）：Created 插行 / Status/Bucket 按.identifier
/// UPDATE（LWW 戳随改）；写失败 stderr 披露不重试（提交路径低频，队列面归
/// 捕获链优先级；登记边界 DEC-187）。
fn task_db_event(cx: &mut App, ev: &TaskDbEvent) {
    let Some(db) = app_db(cx) else { return };
    let now = iso_utc(walltime::now_epoch());
    match ev {
        TaskDbEvent::Created {
            identifier,
            title,
            bucket,
            priority,
            mark,
        } => {
            let seq: usize = identifier
                .strip_prefix("TASK-")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            let todo = DbTodo {
                id: new_uuid(),
                title: title.clone(),
                description: String::new(),
                status: TaskStatus::Todo,
                priority: priority_from_index(*priority),
                schedule_bucket: bucket_from_index(*bucket),
                sort_order: seq as i64,
                identifier: identifier.clone(),
                project_id: None,
                source: None,
                mark: (*mark).min(6),
                created_at: now.clone(),
                modified_at: now,
            };
            if let Err(e) = insert_todo(&db, &todo) {
                eprintln!("[parchora] 待办落库失败（{identifier}）：{e}（本会话仅内存态）");
            }
        }
        TaskDbEvent::Status { identifier, status } => {
            let status = task_status_from_index(*status);
            if let Err(e) = set_todo_status(&db, identifier, status, &now) {
                eprintln!("[parchora] 待办状态落库失败（{identifier}）：{e}");
            }
        }
        TaskDbEvent::Bucket { identifier, bucket } => {
            let bucket = bucket_from_index(*bucket);
            if let Err(e) = set_todo_bucket(&db, identifier, bucket, &now) {
                eprintln!("[parchora] 待办排期落库失败（{identifier}）：{e}");
            }
        }
        TaskDbEvent::Deleted { identifier } => {
            if let Err(e) = delete_todo_by_identifier(&db, identifier) {
                eprintln!("[parchora] 待办删除落库失败（{identifier}）：{e}");
            }
        }
    }
}

/// 拾记行删除 → DB 事务删（TASK-300 · 右键「删除」逆向动作；无 DB 身份行
/// 不落此路）。失败 stderr 披露（存量对账下拍兑底）；删行生效
/// → blob 差集清理随拍（DEC-189 · blob 与条目同生命周期）。
fn clip_delete_from_db(cx: &mut App, db_id: &str) {
    let Some(db) = app_db(cx) else { return };
    match CoreUuid::parse_str(db_id) {
        Ok(id) => match delete_clip(&db, &id) {
            Ok(1) => prune_blob_dirs(cx),
            Ok(_) => {} // 行已不在（幂等重删）：blob 清理交 sweep 对账兜底
            Err(e) => eprintln!("[parchora] 拾记删除失败：{e}"),
        },
        Err(e) => eprintln!("[parchora] 拾记删除 id 非法：{e}"),
    }
}

/// 队列动作宿主 mutator（DEC-207 装配 · TASK-301 引擎面扩展）：单一事实源
/// QueueHost 变更后镜像全量回推面板（set_queue_rows）。调用域＝defer 出借
/// 域外（set_queue_action_sink 装配处 defer_app 延后），面板不在场时仅变更
/// 全局——镜像在下次重开链由 push_queue_to_panel 回灌（幂等）。
/// Enqueue 只改本体——上架/重申/结算归 queue_tick（拍制 reconcile 统一承
/// 载，下拍≤QUEUE_POLL_SECS 自动投递）；Stop/Clear＝停止全链（fixup 补写
/// → clear 本体 → 事件披露）。事件镜像面见 disclose_queue_event。
fn queue_event(cx: &mut App, action: &QueueAction) {
    let mut events: Vec<QueueEventKind> = Vec::new();
    let mirror: Vec<QueueRow> = match action {
        QueueAction::Enqueue(text) => {
            // 入队 + 持久承接（DEC-212 · 每动作事务即写）：ord 会话单调分配，
            // uuid/时间戳在 app 层映射；写失败仅披露（内存队列继续跑）。
            let (id, ord) = {
                let host = cx.global_mut::<QueueHost>();
                let id = host.queue.enqueue(text);
                let ord = host.next_ord;
                host.next_ord += 1;
                (id, ord)
            };
            queue_persist_enqueue(cx, id, text, ord);
            queue_mirror(cx)
        }
        QueueAction::ToggleOrder => {
            cx.global_mut::<QueueHost>().queue.toggle_order();
            // 持板态重申收敛归 queue_tick（placed.id≠view 首项 → 拍内重申）
            //——本面只改本体与镜像。模式持久化随设置档（DEC-212 · save 链读
            // 全局单一事实源，同 apply_queue_notify_interval 先例）。
            save_settings_to_disk(cx, trigger_pos(cx));
            queue_mirror(cx)
        }
        QueueAction::Stop | QueueAction::Clear => {
            if let Some(ev) = queue_stop_full(cx, action) {
                events.push(ev);
            }
            queue_mirror(cx)
        }
    };
    push_queue_mirror(cx, mirror);
    queue_notify_events(cx, events);
}

/// 队列停止全链（SCN-QUEUE-STOP-FIXUP）：持板→placed 项真实数据写回（终
/// 态——板面解除承诺，后续 ⌘V 不再触发队列）；本体 clear（SCN「停止/清空
/// 后队列不持任何条目」）；事件产出（Stopped/Cleared 分型——宿主 gate：
/// 「队列曾运行过」（本体非空或持板）才上 Cleared 口径，空态清空静默）。
/// 事件披露与提示编排归 queue_notify_events（调用方收集承载）。
fn queue_stop_full(cx: &mut App, action: &QueueAction) -> Option<QueueEventKind> {
    let kind = match action {
        QueueAction::Stop => StopKind::Stop,
        _ => StopKind::Clear,
    };
    let was_holding;
    let released;
    {
        let host = cx.global_mut::<QueueHost>();
        was_holding = host.engine.holding().is_some();
        released = host.queue.clear();
        // 持久承接终态同步（DEC-212）：映射随本体清空（uuid 对随行摘除防泄漏）。
        host.uuid_by_id.clear();
        let mut board = PlatformPromiseBoard;
        match host.engine.stop(&mut board, kind) {
            StopOutcome::Released { written, .. } => {
                if !written {
                    eprintln!("[parchora] 队列停止补写失败（真实数据未落板）");
                }
            }
            // 分叉披露收口（R184 建议 1）：调用序保证理论不可达（was_holding 取
            // 自同拍 holding()），防御性披露并按未持板收口（不 panic 中断停止链）。
            StopOutcome::NotHeld if was_holding => {
                eprintln!("[parchora] 队列停止：持板判定与引擎态分叉（按未持板收口）");
            }
            StopOutcome::NotHeld => {}
        }
    }
    // 持久承接全表清（DEC-212 · Stop/Clear 唯一收口点＝queue_stop_full——
    // queue_event 双臂不重复挂写）。失败仅披露：内存终态为准，残余行待重启
    // 披露面在案（遗留风险）。
    if let Some(db) = app_db(cx) {
        if let Err(e) = clear_queue_entries(&db) {
            eprintln!("[parchora] 队列落库清空失败：{e}");
        }
    }
    let mut ev = QueueEventKind::Stopped { released };
    if matches!(action, QueueAction::Clear) {
        ev = QueueEventKind::Cleared;
    }
    // 宿主 gate（§11.2 清空通知「仅当队列曾运行过」）：空态清空且从未持板
    // → 静默（None=无事件进通知编排面）。
    if matches!(ev, QueueEventKind::Cleared) && released == 0 && !was_holding {
        return None;
    }
    Some(ev)
}

/// 队列条目落库（DEC-212 · 入队动作即写）：uuid（core 助手）/ ord（会话单调
/// 序）/ created_at（墙钟 ISO8601）app 层映射；uuid 映射仅在写成功后登记
///（失败面后消耗查 miss 幂等跳过）。无存储接线（测试/降级）＝纯内存态静默。
fn queue_persist_enqueue(cx: &mut App, id: u64, text: &str, ord: u64) {
    let Some(db) = app_db(cx) else {
        return;
    };
    let uid = new_uuid().to_string();
    if let Err(e) = insert_queue_entry(&db, &uid, text, ord as i64, &iso_utc(walltime::now_epoch()))
    {
        eprintln!("[parchora] 队列条目落库失败：{e}（本会话仅内存态）");
        return;
    }
    cx.global_mut::<QueueHost>().uuid_by_id.insert(id, uid);
}

/// 队列条目删行（DEC-212 · 消耗动作即写）：宿主租借块外执行（global_mut 借
/// 期间不触 cx）；失败仅披露——内存态为真相，残余行重启复活面随遗留风险在案。
fn queue_delete_rows(cx: &mut App, uuids: &[String]) {
    if uuids.is_empty() {
        return;
    }
    let Some(db) = app_db(cx) else {
        return;
    };
    for uuid in uuids {
        if let Err(e) = delete_queue_entry(&db, uuid) {
            eprintln!("[parchora] 队列条目删行失败：{e}");
        }
    }
}

/// 队列事件披露（TASK-301/716 · §11.2 提示层 stderr 面——数量口径，不含剪
/// 贴板内容（规范 8/C5））。
fn disclose_queue_event(ev: QueueEventKind) {
    match ev {
        QueueEventKind::Placed { remaining } => {
            eprintln!("[parchora] 队列投递上架（待耗 {remaining} 条）");
        }
        QueueEventKind::Consumed { remaining } => {
            eprintln!("[parchora] 队列粘贴消耗一条（剩余 {remaining} 条）");
        }
        QueueEventKind::Stopped { released } => {
            eprintln!("[parchora] 队列停止（补写完成 · 释放 {released} 条）");
        }
        QueueEventKind::Cleared => {
            eprintln!("[parchora] 队列已清空");
        }
    }
}

/// 队列提示桥执行面（TASK-405/716 · trait 化使全链可 mock）：生产 = 各面直走
/// platform 桥 / 面板注入链；测试 = cfg(test) 恒短路（不碰真 TCC/通知面）or
/// MockQueueBridge 测试自驱。cfg(test) 语义同 PlatformPromiseBoard 口径
///（短路态——测试零系统副作用）。
trait QueueBridge {
    /// 向系统发起通知授权请求（首次需要送通知时 · 异步回调写共享三态）。
    fn authorize(&self);
    /// 授权三态读点（未决/拒绝均为伪——send 侧静默禁用）。
    fn authorized(&self) -> bool;
    /// Dock 图标弹一弹（Informational · 返回请求 id 供点击撤回）。
    fn dock_attention(&self) -> Option<u64>;
    /// 撤回未决注意请求（点击路由前排水——面板已可见动效位）。
    fn cancel_attention(&self, id: u64);
    /// 面板队列按钮 bounce 一轮（面板注入链 · 仅主面板在场时落地）。
    fn panel_bounce(&self, cx: &mut App);
    /// 系统通知立即投递（标题+条数 · 未授权静默）。
    fn send_now(&self, title: &str, body: &str);
    /// 通知点击槽排水（消费式）。
    fn click_take(&self) -> bool;
    /// 点击路由（打开面板聚焦拾记）。
    fn route_click(&self, cx: &mut App);
}

struct PlatformQueueBridge;

impl QueueBridge for PlatformQueueBridge {
    fn authorize(&self) {
        parchora_platform::notification_request_authorization();
    }
    fn authorized(&self) -> bool {
        parchora_platform::notification_authorized()
    }
    fn dock_attention(&self) -> Option<u64> {
        parchora_platform::request_user_attention_informational()
    }
    fn cancel_attention(&self, id: u64) {
        parchora_platform::cancel_user_attention_request(id);
    }
    fn panel_bounce(&self, cx: &mut App) {
        push_queue_bounce(cx);
    }
    fn send_now(&self, title: &str, body: &str) {
        parchora_platform::notification_send_now(title, body);
    }
    fn click_take(&self) -> bool {
        parchora_platform::notification_click_take()
    }
    fn route_click(&self, cx: &mut App) {
        notification_click_route(cx);
    }
}

/// 队列事件提示编排单点（TASK-716 · §11.2 表逐行）：通知点击排水路由（每拍
/// 必查——独立于事件有无）→ 逐事件：授权读门→notifier 状态机→桥执行→stderr
/// 披露。注意请求 id 登记归 host.notifier（点击撤回承载）。
fn queue_notify_events(cx: &mut App, events: Vec<QueueEventKind>) {
    queue_notify_events_with(cx, events, &mut PlatformQueueBridge);
}

/// 桥注入变体（测试驱入口 · 同 clipboard_tick_with 先例）。
fn queue_notify_events_with(
    cx: &mut App,
    events: Vec<QueueEventKind>,
    bridge: &mut dyn QueueBridge,
) {
    // ① 点击排水路由（§11.2「点击行为 = 打开面板并聚焦拾记视图」；未决注意
    //    请求撤回——面板已可见，弹跳目的已达）。
    if bridge.click_take() {
        let id = cx.global_mut::<QueueHost>().notifier.take_attention();
        if let Some(id) = id {
            bridge.cancel_attention(id);
        }
        bridge.route_click(cx);
    }
    // ② 事件编排（空集直出——纯点击拍不触全局）。
    for ev in events {
        let authorized = bridge.authorized();
        let actions = cx
            .global_mut::<QueueHost>()
            .notifier
            .process(ev, authorized);
        for action in actions {
            match action {
                QueuePromptAction::DockAttention => {
                    if let Some(id) = bridge.dock_attention() {
                        cx.global_mut::<QueueHost>().notifier.note_attention(id);
                    }
                }
                QueuePromptAction::PanelBounce => bridge.panel_bounce(cx),
                QueuePromptAction::AuthorizeRequest => bridge.authorize(),
                QueuePromptAction::SystemNotify { body } => {
                    bridge.send_now(QUEUE_NOTIFY_TITLE, &body);
                }
            }
        }
        disclose_queue_event(ev);
    }
}

/// 队列按钮动效注入面板（set_queue_rows 同构链：宿主 → PanelView →
/// CaptureView；面板不在场 no-op——动效位不存在）。
fn push_queue_bounce(cx: &mut App) {
    if let Some(handle) = cx.global::<DockHandles>().panel {
        let _ = handle.update(cx, |panel, _w, cx| panel.set_queue_bounce(cx));
    }
}

/// 通知点击路由（§11.2）：打开面板聚焦拾记视图（restore_view 覆盖记账——
/// 通知点击语义恒聚焦拾记）。open_panel_docked 幂等（在场早返）。
fn notification_click_route(cx: &mut App) {
    open_panel_docked(cx);
    if let Some(handle) = cx.global::<DockHandles>().panel {
        let _ = handle.update(cx, |panel, _w, cx| panel.restore_view(View::Capture, cx));
    }
}

/// 队列通知节流间隔应用（REQ-SETTINGS-QUEUE-NOTIFY-INTERVAL · DEC-48）：单一
/// 事实源落值 + 即刻持久化；改值立即生效（notifier 内部阈值直改——后续消耗
/// 立按新阈值判，不重置节流计数）。
fn apply_queue_notify_interval(cx: &mut App, interval: u64) {
    cx.global_mut::<QueueHost>().notifier.set_interval(interval);
    save_settings_to_disk(cx, trigger_pos(cx));
}

/// 队列消费序镜像组行（DEC-207）：capture::ClipQueue.view()（首项＝下一消耗
/// 项，LIFO 整列倒置）→ UI QueueRow 全量替换载荷。
fn queue_mirror(cx: &App) -> Vec<QueueRow> {
    cx.global::<QueueHost>()
        .queue
        .view()
        .into_iter()
        .map(|e| QueueRow {
            id: e.id,
            text: e.text,
        })
        .collect()
}

/// 队列镜像回推面板（DEC-207 · sync_disk_notes 同构：无面板/句柄失效 no-op），
/// 队列详情层内嵌于拾记视图（queue_open 态）时随回推逐帧刷新。
fn push_queue_mirror(cx: &mut App, rows: Vec<QueueRow>) {
    if let Some(handle) = cx.global::<DockHandles>().panel {
        let _ = handle.update(cx, |panel, _w, cx| panel.set_queue_rows(rows, cx));
    }
}

/// 队列镜像回灌（面板重开链收口 · 幂等全量替换；与 backfill/pending 冲刷同
/// 链尾位）：复用/全新实体统一回推（复用实体覆盖刷新，防动作窗口期失同步）。
fn push_queue_to_panel(cx: &mut App, handle: &gpui::WindowHandle<PanelView>) {
    let rows = queue_mirror(cx);
    let _ = handle.update(cx, |panel, _w, cx| panel.set_queue_rows(rows, cx));
}

/// blob 差集清理（DEC-189 · design §2.6「blob 与条目同生命周期」· TASK-202
/// 技术口径）：磁盘 blobs/<uuid>/ 目录 vs DB 引用集差集——行已删而目录在册
/// 即孤儿，整目录移除（raw/thumb 同目录双件同生共死）；引用目录不动。清理
/// 域恒随活跃存储根（store_root · 与引用集读同库同根，防跨根误判全删）；仅
/// 计数披露（stderr），失败披露不中断（下拍 sweep 再战）。
fn prune_blob_dirs(cx: &mut App) {
    let Some(db) = app_db(cx) else { return };
    let refs: std::collections::HashSet<String> = match list_blob_rel_paths(&db) {
        Ok(paths) => paths
            .into_iter()
            .filter_map(|rel| rel.split('/').nth(1).map(String::from))
            .collect(),
        Err(e) => {
            eprintln!("[parchora] blob 清理引用集读失败：{e}");
            return;
        }
    };
    let Some(root) = store_root(cx) else { return };
    let Ok(entries) = std::fs::read_dir(root.join("data").join("blobs")) else {
        return; // 目录不在场＝无孤儿可清
    };
    let mut removed = 0usize;
    for entry in entries.flatten() {
        // 仅目录域（blobs/<uuid>/ 承载 · 散落文件不在清理面，remove 对文件面无意义）。
        if entry.file_type().map(|t| !t.is_dir()).unwrap_or(true) {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        if refs.contains(&name) {
            continue;
        }
        if std::fs::remove_dir_all(entry.path()).is_ok() {
            removed += 1;
        } else {
            eprintln!("[parchora] blob 清理移除失败：{name}");
        }
    }
    if removed > 0 {
        eprintln!("[parchora] blob 清理：移除孤儿目录 {removed} 个");
    }
}

/// 孤儿行回溯载荷（pending 缓冲补写用 · 文本行直重建）：文本类重建载荷（hash
/// 同源重算）；image/富类型行字节面已随重试队列在册（DEC-188：冲刷
/// 前先查队列同刻载荷，本口仅文本无队列时的兑底）。
fn orphan_payload_for_row(row: &ClipRow) -> Option<CapturedPayload> {
    // 非文本行不在重建面（字节载荷随重试队列在册 · DEC-188）；文本行直用
    // 行题名（R184 建议 6：text==title 语义已核——Some 包装的
    // as_ref().unwrap() 中转面删除，哈希输入等价不变）。
    if row.kind != FilterKind::PlainText {
        return None;
    }
    let text = row.title.clone();
    let content_type = ContentType::PlainText;
    let hash = parchora_core::hash::sha256_hex(text.as_bytes());
    Some(CapturedPayload {
        content_type,
        text: Some(text),
        bytes: None,
        hash,
        source_app: None,
        thumb: None,
        wall: row.wall.epoch,
        at: row.added_at,
    })
}

/// core 枚举 ↔ UI 下标对账（STATUSES/BUCKETS 序即 enum 序 · 双向纯映射）。
fn status_index(s: TaskStatus) -> usize {
    match s {
        TaskStatus::Todo => 0,
        TaskStatus::InProgress => 1,
        TaskStatus::Backlog => 2,
        TaskStatus::Done => 3,
        TaskStatus::Cancelled => 4,
    }
}

fn priority_index(p: Priority) -> usize {
    match p {
        Priority::None => 0,
        Priority::Low => 1,
        Priority::Medium => 2,
        Priority::High => 3,
        Priority::Urgent => 4,
    }
}

fn priority_from_index(i: usize) -> Priority {
    match i {
        1 => Priority::Low,
        2 => Priority::Medium,
        3 => Priority::High,
        4 => Priority::Urgent,
        _ => Priority::None,
    }
}

fn bucket_index(b: ScheduleBucket) -> usize {
    match b {
        ScheduleBucket::Today => 0,
        ScheduleBucket::Tomorrow => 1,
        ScheduleBucket::ThisWeek => 2,
        ScheduleBucket::Later => 3,
        ScheduleBucket::Unscheduled => 4,
    }
}

fn task_status_from_index(i: usize) -> TaskStatus {
    match i {
        1 => TaskStatus::InProgress,
        2 => TaskStatus::Backlog,
        3 => TaskStatus::Done,
        4 => TaskStatus::Cancelled,
        _ => TaskStatus::Todo,
    }
}

fn bucket_from_index(i: usize) -> ScheduleBucket {
    match i {
        1 => ScheduleBucket::Tomorrow,
        2 => ScheduleBucket::ThisWeek,
        3 => ScheduleBucket::Later,
        4 => ScheduleBucket::Unscheduled,
        _ => ScheduleBucket::Today,
    }
}

/// 设置面板切触发位置 → 全局落值 + 主面板按新位重开（关窗重开 · DEC-67 D-03/D-08）。
/// 生产入口：读当前活动屏双屏帧 → 交注入式变体；冲突自动切换（DEC-84）在内。
fn apply_trigger_pos(cx: &mut App, pos: TriggerPosition) {
    let (full, usable) = screen_frames(cx);
    apply_trigger_pos_with(cx, pos, full, usable);
}

/// 设置面板切触发位置：实界注入式变体（测试入口 · 生产走 apply_trigger_pos）。
/// 先经 resolve_conflict 解析 Dock 同侧冲突（DEC-84：被占用侧缘自动切换——左→右、
/// 右→左、底→右；切换时 stderr 提示落档）；唤出门/确认拍复位（新位按 armed +
/// 0 拍起算 · DEC-79）；占用态同步落 DockAvailability（值变才 notify 设置窗）。
/// Peek 锥定旧几何 → 一并关闭（下轮单击按新位重开）。重开前 bump peek_close_gen：
/// 未决 peek-close timer（宽限落关）若仍生效会误关重开后的新面板
/// （DEC-77 登记隐患）。
fn apply_trigger_pos_with(
    cx: &mut App,
    pos: TriggerPosition,
    full: dock::ScreenRect,
    usable: dock::ScreenRect,
) {
    let resolved = dock::resolve_conflict(pos, full, usable);
    if resolved != pos {
        // 冲突切换登记（无服务端 toast 通道；UI-TOAST 属设置窗内域，宿主级切换
        // 以 stderr 落档 + 04 人工验证项承载 · DEC-84）。
        eprintln!(
            "[parchora] 触发位 {} 与 Dock 占用冲突 → 自动切换 {}（DEC-84）",
            pos.label(),
            resolved.label()
        );
    }
    cx.global_mut::<DockPrefs>().trigger = resolved; // 仅落 trigger，保留 active_display 探针值
                                                     // 即刻持久化（DEC-85：启动/设置换位/屏帧变化三路同口；cfg(test) noop）。
    save_settings_to_disk(cx, resolved);
    refresh_dock_availability_with(cx, full, usable);
    bump_peek_close_gen(cx);
    {
        // 仅复位唤出门与确认拍（close_panel 不触 · DEC-79 新位 armed 起算）；
        // revealed/close_streak 交紧随的 close_panel 统一复位（显隐链收敛去重）。
        let handles = cx.global_mut::<DockHandles>();
        handles.reveal_suppressed = false;
        handles.reveal_streak = 0;
    }
    // 子窗先卸（拖动跟随：Peek 已挂面板 AppKit 子窗 → 先关 Peek 再关
    // 面板，与 snap 重泊路径同序）。
    close_peek(cx, true);
    close_panel(cx);
    open_panel_docked(cx);
}

/// 「剪贴板历史保留时长」（天）应用（DEC-178 · 设置存储页数值输入上抛）：
/// 先落 ClipCapture 单一事实源（守护态即运行态 · 天制）再即刻持久化
///（apply_trigger_pos 同链 · env 键 clip_retention_days；非法值不触本口——
/// 设置侧 commit_retention 校验后仅携合法正整数上抛）。
fn apply_clip_retention_days(cx: &mut App, days: u64) {
    // 兖底钳制（DEC-184）：设置侧已校验 ≥1，宿主入口再钉下界防未知调用源——
    // 0 天档会使存活清理即刻清空拾记（survive=0 → 任何时刻 age≥0 即过期）。
    cx.global_mut::<ClipCapture>().retention_days = days.max(1);
    save_settings_to_disk(cx, trigger_pos(cx));
    // 调小当拍即清（DEC-189 · 与上限档 SHRINK-IMMEDIATE 同款立即生效：存活阈值
    // 改小后 UI+DB 两轴对账随行，不等下一次复制 · REQ-CAPTURE-LIMIT 精神，
    // DEC-163/178 存活阈值轴）。
    clip_sweep(cx);
}

/// 剪贴板监控开关应用（SCN-CAPTURE-SWITCH-OFF · DEC-188 · 设置存储页 switch
/// 即时上抛）：单一事实源落值 + 即刻持久化（apply_clip_retention_days 同链）。
/// 开启路径 engine 基线重播种——关闭期 changeCount 积累一次性吞掉（重开首拍只
/// 记新基线不捕旧内容，SCN-CAPTURE-SWITCH-OFF 恢复语义）。
fn apply_capture_enabled(cx: &mut App, on: bool) {
    cx.global_mut::<ClipCapture>().capture_enabled = on;
    if on {
        cx.global_mut::<ClipCapture>().engine = Engine::new();
    }
    save_settings_to_disk(cx, trigger_pos(cx));
}

/// 历史上限档应用（REQ-CAPTURE-LIMIT · DEC-188 · 设置存储页五档 chips）：单一
/// 事实源落值 + 即刻持久化；调小裁剪当拍 sweep 两面对账（SCN-CAPTURE-LIMIT-
/// SHRINK-IMMEDIATE「不等待下一次复制」· DEC-189）。
fn apply_capture_limit(cx: &mut App, limit: u64) {
    cx.global_mut::<ClipCapture>().limit = limit;
    save_settings_to_disk(cx, trigger_pos(cx));
    // 调小当拍即清（UI 面板行 + DB 两轴 · 含 blob 差集随行），不等下一次复制；
    // 0=不限档由 sweep 内「limit>0 才触口」既有语义承载。
    clip_sweep(cx);
}

/// 前台排除列表应用（REQ-CAPTURE-EXCLUDE · DEC-188 · 设置存储页文本输入归一
/// 化串上抛）：逗号分隔 → 列表去空项；单一事实源落值 + 即刻持久化。
fn apply_capture_exclude_apps(cx: &mut App, joined: String) {
    cx.global_mut::<ClipCapture>().exclude_apps = joined
        .split(',')
        .map(|p| p.trim().to_string())
        .filter(|p| !p.is_empty())
        .collect();
    save_settings_to_disk(cx, trigger_pos(cx));
}

/// 设置独立窗（430；单例：幂等开窗 + ⌘W 陈旧句柄校验重开）。
/// 面板分区触发位置经 Sink 即时生效（DEC-67）。
/// 开窗即复位悬停外关闭臂（未臂不累计，首次入设置窗才臂）。
fn open_settings(cx: &mut App) {
    if let Some(handle) = cx.global::<DockHandles>().settings {
        if handle.update(cx, |_, _, _| ()).is_ok() {
            return; // 仍在 → 幂等跳过（需求 3：无额外重复界面）
        }
        cx.global_mut::<DockHandles>().settings = None;
    }
    let handle = match cx.open_window(
        window_opts(cx, tk::SETTINGS_W, 560.0, None),
        |_window, cx| {
            cx.new(|cx| {
                let mut settings = SettingsView::new(cx);
                // 捕获域四件播种（DEC-178/DEC-188）：单一事实源 ClipCapture
                // 开窗镜像 → 输入初值；try_global 兼容未初始化语境（回默认）。
                let capref = cx.try_global::<ClipCapture>();
                settings.set_retention_days(
                    capref
                        .as_ref()
                        .map(|c| c.retention_days)
                        .unwrap_or(CLIP_RETENTION_DAYS_DEFAULT),
                );
                settings.set_capture_enabled(
                    capref
                        .as_ref()
                        .map(|c| c.capture_enabled)
                        .unwrap_or(CLIP_CAPTURE_ENABLED_DEFAULT),
                );
                settings.set_capture_limit(
                    capref
                        .as_ref()
                        .map(|c| c.limit)
                        .unwrap_or(CLIP_LIMIT_DEFAULT),
                );
                settings
                    .set_exclude_apps(capref.map(|c| c.exclude_apps.join(",")).unwrap_or_default());
                // 队列通知节流间隔播种（DEC-209 · REQ-SETTINGS-QUEUE-NOTIFY-
                // INTERVAL）：QueueHost 单一事实源读点（try_global 兼容未初
                // 始化语境——回默认）。
                settings.set_queue_notify_interval(
                    cx.try_global::<QueueHost>()
                        .map(|h| h.notifier.interval())
                        .unwrap_or(parchora_capture::notify::QUEUE_NOTIFY_INTERVAL_DEFAULT),
                );
                settings.set_event_sink(Box::new(|ev: &SettingsEvent, cx: &mut App| match ev {
                    // 切位联动 refresh_dock_availability_with 会在租借态回触
                    // 设置窗实体（值变才写路径 · cx.notify）——sink 于 listener
                    // 租借态内同步直调，同实体句柄嵌套租借即 double lease abort
                    //（DEC-77）：延迟出借域执行（PeekShow/HidePanel 同款）。
                    SettingsEvent::TriggerPosChanged(pos) => {
                        let pos = *pos;
                        defer_app(cx, move |cx| apply_trigger_pos(cx, pos));
                    }
                    // 保留时长（DEC-178）：仅触全局落值 + 持久化（无实体回触
                    // · 租借铁律豁免同 PinChanged）——同步直调 apply 即刻生效。
                    SettingsEvent::ClipRetentionChanged(days) => {
                        apply_clip_retention_days(cx, *days);
                    }
                    // 捕获域三件（DEC-188）：开关/上限档/排除应用——无实体
                    // 回触（同 retention 豁免口径），同步直调 apply 即刻生效。
                    SettingsEvent::CaptureEnabledChanged(on) => {
                        apply_capture_enabled(cx, *on);
                    }
                    SettingsEvent::CaptureLimitChanged(limit) => {
                        apply_capture_limit(cx, *limit);
                    }
                    SettingsEvent::CaptureExcludeChanged(joined) => {
                        apply_capture_exclude_apps(cx, joined.clone());
                    }
                    // 队列通知节流间隔（DEC-209）：无实体回触（同 retention
                    // 豁免口径），同步直调 apply 即刻生效。
                    SettingsEvent::QueueNotifyIntervalChanged(n) => {
                        apply_queue_notify_interval(cx, *n);
                    }
                    // 引导卡点击（DEC-81）：后台请求授权（TCC 弹窗引导入口）
                    // + 回写 ListenAuth + tap 惰性装配。
                    SettingsEvent::ListenAuthRequest => handle_listen_auth_request(cx),
                }));
                settings
            })
        },
    ) {
        Ok(handle) => handle,
        Err(e) => {
            eprintln!("[parchora] 设置窗开窗失败（降级：本拍不装配设置窗）：{e}");
            return;
        }
    };
    let handles = cx.global_mut::<DockHandles>();
    handles.settings = Some(handle);
    handles.settings_close_streak = 0;
    handles.settings_close_armed = false;
}

/// 关闭设置窗（hover-out 自动关路径；无窗幂等；复位外闭计数与臂）。
/// 关窗前漏斗提交（DEC-178）：未 blur/enter 的保留时长键入值先经 commit_retention
///（合法变更 → 生效 + 持久化；auto-close/quit 路不丢值）再移除窗口。
fn close_settings(cx: &mut App) {
    let handle = cx.global_mut::<DockHandles>().settings.take();
    if let Some(handle) = handle {
        let _ = handle.update(cx, |view, window, cx| {
            view.commit_retention(window, cx);
            view.commit_exclude(window, cx);
        });
        let _ = handle.update(cx, |_, window, _| window.remove_window());
    }
    let handles = cx.global_mut::<DockHandles>();
    handles.settings_close_streak = 0;
    handles.settings_close_armed = false;
}

/// 设置窗观测守护轮询间隔（秒）：hover-out 自动关最迟 ~0.05s 响应。
const SETTINGS_WATCH_POLL_SECS: f32 = 0.05;

/// 设置窗悬停外自动关闭守护（全形态）：0.05s 轮询
/// ——设置窗在开时鼠标 hover 在设置窗自身面板之外 → 自动保存（幂等）并关闭设
/// 置窗；光标回到设置窗上 → 保持。与 reveal/snap/status 守护互不依赖；设置窗不
/// 在场时单拍直返（近乎零成本）。
fn start_settings_watch(cx: &App) {
    cx.spawn(async move |cx| loop {
        cx.background_executor()
            .timer(Duration::from_secs_f32(SETTINGS_WATCH_POLL_SECS))
            .await;
        cx.update(settings_watch_tick);
    })
    .detach();
}

/// 设置窗观测守护生产单拍：真光标源（tap 槽新鲜度校验 + poll 降级 · 与 reveal
/// 守护同源）交注入式闭窗单拍。
fn settings_watch_tick(cx: &mut App) {
    auto_close_settings_tick(cx, current_cursor());
}

/// 设置窗悬停外自动关闭单拍（关窗判定域 = 设置窗自身实界——鼠标仅移出设置
/// 窗口面板时，即便鼠标位于应用其它 GUI 界面上，也需保存设置并关闭该设置窗口
/// 面板；注入式变体测试可驱动）：
/// - 设置窗不在场 → 直返（计数在开/关窗时复位）；
/// - 光标读失败 → 计数清零（无证据不累计，防误关）；
/// - 设置窗实界读失败 = 陈旧句柄 → 防御性不关（open_settings 幂等路径下拍清理）；
/// - 光标在设置窗实界内(含与其它 GUI 视觉重叠区）→ 计数清零（保持）+ 置臂；
/// - 未臂（光标未首次进入设置窗）→ 不计数（⚙/菜单/CLI 开窗时光标
///   在窗外属正常开窗态，不构成「移出」——无臂则开窗 0.2s 即闪关）；
/// - 臂后连续 SETTINGS_CLOSE_CONFIRM_TICKS 拍设置窗实界之外（含主面板/
///   Peek 等其它 GUI 界面——唯一例外语义落点）→ 即刻持久化（幂等）+ 关闭设置窗。
///   主面板/Peek 的显示与隐藏正交：交互约束「鼠标在任一应用 GUI 内不触发隐藏」
///   由 reveal/probe 守护域（panel/peek/热区判定）独立承载，本拍不回传隐藏动作
///   （设置窗关闭不牵动其它窗）。
fn auto_close_settings_tick(cx: &mut App, cursor: Option<(f32, f32)>) {
    let Some(settings_handle) = cx.global::<DockHandles>().settings else {
        return;
    };
    let Some((x, y)) = cursor else {
        cx.global_mut::<DockHandles>().settings_close_streak = 0;
        return;
    };
    // 关窗判定域 = 设置窗自身实界（例外口径；面板/Peek 不在保持域
    // ——「仅移出设置窗」即累计关闭）。全局坐标对比全局光标（DEC-99 缝）。
    let settings_rect = window_rect(cx, &settings_handle);
    // settings 实界读失败 = 陈旧句柄 → 防御性不关（open_settings 幂等路径下拍清理）。
    let Some((sx, sy, sw, sh)) = settings_rect else {
        return;
    };
    if x >= sx && x <= sx + sw && y >= sy && y <= sy + sh {
        let handles = cx.global_mut::<DockHandles>();
        handles.settings_close_streak = 0;
        // 入板即臂：首次进入后「移出」语义才能成立。
        handles.settings_close_armed = true;
        return; // 光标仍在设置窗面板内（GUI 内交互约束）
    }
    if !cx.global::<DockHandles>().settings_close_armed {
        // 未臂不累计（防开窗即判移出闪关）。
        cx.global_mut::<DockHandles>().settings_close_streak = 0;
        return;
    }
    let confirmed = {
        let handles = cx.global_mut::<DockHandles>();
        handles.settings_close_streak += 1;
        let confirmed = handles.settings_close_streak >= SETTINGS_CLOSE_CONFIRM_TICKS;
        if confirmed {
            handles.settings_close_streak = 0;
        }
        confirmed
    };
    if confirmed {
        save_settings_to_disk(cx, trigger_pos(cx));
        close_settings(cx);
    }
}

// ───── 剪贴板捕获守护（DEC-163 · REQ-CAPTURE-* 演示宿主最小闭环）─────

/// 剪贴板捕获守护（DEC-163）：0.5s 轮询 changeCount（REQ-CAPTURE-POLL）——未命中
/// 变化不读板面；首拍只播种基线不捕获（SCN-CAPTURE-BOOT-BASELINE）。变化 → 读文本
///（platform 桥面内置隐私标记拒收 · REQ-CAPTURE-PRIVACY）→ 自写板跳过
///（clipstamp · DEC-46）→ 同内容 10s 判重（REQ-CAPTURE-DEDUPE）→ 面板在场实时
/// 注入拾记列表顶 / 不在场入 pending 缓冲（重开冲洗）；每拍兼任存活清理（两阈值
/// 自配置档全局）。与 reveal/snap/settings/status 守护互不依赖。
fn start_clipboard_watch(cx: &App) {
    cx.spawn(async move |cx| loop {
        cx.background_executor()
            .timer(Duration::from_secs_f32(CLIP_POLL_SECS))
            .await;
        cx.update(clipboard_tick);
    })
    .detach();
    cx.spawn(async move |cx| loop {
        cx.background_executor()
            .timer(Duration::from_secs_f32(QUEUE_POLL_SECS))
            .await;
        cx.update(queue_tick);
    })
    .detach();
}

/// 队列引擎单拍（TASK-301 · REQ-QUEUE-DETECT 拍制编排）：四段序——
/// ① 结算兑现（Settling 到期：非探查=消耗流出→本体摘除→Consumed 事件；
///    探查=引擎内重申承诺无事件）；
/// ② 索取吸收（Placed 态 drain demand → Settling——无事件）；
/// ③ 重申收敛（Placed 态 placed.id≠view 首项〔toggle 后〕→ replace_promise）；
/// ④ 空板 reconcile（Idle 态且本体非空 → place view 首项——首次入队/失败
///    重试/消耗后继上架统一承载，Placed 事件）。
/// 事件收集进 Vec（宿主 borrow 块内不触 cx）——块外交给 queue_notify_events
///（TASK-716 提示编排：点击排水→路由、事件→notifier→桥执行→stderr 披露）；
/// TASK-716 提示编排：点击排水→路由、事件→notifier→桥执行→stderr 披露）；
/// 本体变化后镜像回推（事件驱动不每拍推——无事件无重渲）。
fn queue_tick(cx: &mut App) {
    let mut events: Vec<QueueEventKind> = Vec::new();
    let mut any_change = false;
    let mut consumed_uuids: Vec<String> = Vec::new();
    {
        let host = cx.global_mut::<QueueHost>();
        let mut board = PlatformPromiseBoard;

        // ① 结算兑现：Settling 到期首拍（非探查=消耗流出）。
        let now = Instant::now();
        if host.engine.settle_due(now) {
            if let Some(consumed) = host.engine.fulfill_settle(&mut board) {
                host.queue.remove_by_id(consumed.id);
                // 持久承接同步（DEC-212）：映射摘 uuid 随行带出，删行在块外执行。
                if let Some(uuid) = host.uuid_by_id.remove(&consumed.id) {
                    consumed_uuids.push(uuid);
                }
                any_change = true;
                events.push(QueueEventKind::Consumed {
                    remaining: host.queue.view().len(),
                });
            }
            // 探查分支（None）：重申已在引擎内完成——无事件无推送。
        }

        // ② 索取吸收：Placed 态 demand → Settling（桥内消费式 drain）。
        host.engine.note_demand(&mut board);

        // ③ 重申收敛：toggle 后 view 首项变化（Settling 态由 replace_promise
        //    自拒——物理板窗口内不得被换）。
        if let Some(entry) = host.engine.holding().cloned() {
            let view = host.queue.view();
            if let Some(first) = view.first() {
                if first.id != entry.id && host.engine.replace_promise(first, &mut board) {
                    // 板面换序承诺——镜像反映新序即可（本体无变化，行
                    // 面由 set_queue_rows 全量替换承载——重推一次）。
                    any_change = true;
                }
            }
        }

        // ④ 空板 reconcile：Idle 且本体非空 → 上架首项（首次入队/失败重试/
        //    消耗后继统一入口；桥失败保持待重试——stderr 披露一次性的失败
        //    面随 disclose）。
        if host.engine.holding().is_none() {
            let view = host.queue.view();
            if let Some(first) = view.first() {
                if host.engine.place_promise(first, &mut board) {
                    any_change = true;
                    events.push(QueueEventKind::Placed {
                        remaining: view.len(),
                    });
                }
            }
        }
    }
    if !consumed_uuids.is_empty() {
        queue_delete_rows(cx, &consumed_uuids);
    }
    if any_change {
        push_queue_mirror(cx, queue_mirror(cx));
    }
    queue_notify_events(cx, events);
}

/// 生产单拍（TASK-300 · capture::Engine 拍级状态机）：真板面源 → 基线门槛 →
/// 隐私/空板/前台排除 → 8 类判定 → 载荷装配 → 统一到达通道（读桥在唯一生产
/// 点 · 注入式变体测试驱动）。
fn clipboard_tick(cx: &mut App) {
    // 监控开关守门（SCN-CAPTURE-SWITCH-OFF · DEC-188）：关闭期不读板不判变；
    // 重开基线重播种由 apply_capture_enabled 承载。
    if !cx.global::<ClipCapture>().capture_enabled {
        return;
    }
    let excludes = cx.global::<ClipCapture>().exclude_apps.clone();
    let captured = {
        let cap = cx.global_mut::<ClipCapture>();
        cap.engine
            .tick(&PlatformPasteboard, &PlatformFrontmost, &excludes)
    };
    let Some(captured) = captured else {
        return; // 基线播种/未变化/隐私拒收/空板/前台排除/判重前置判定承载
    };
    let now = Instant::now();
    let wall = walltime::now_epoch();
    let source_app = PlatformFrontmost.bundle_id();
    clipboard_tick_with_payload(
        cx,
        CapturedPayload::from_captured(&captured, source_app, wall, now),
    );
}

/// 注入式捕获单拍（DEC-163 测试锚点 · 纯文本路径，仅测试构建）：自写板跳过 →
/// 判重 → 载荷落库 → 面板注入 / pending 缓冲 → 存活清理。生产链已由
/// clipboard_tick（engine 全类型面）承载同一到达通道。
#[cfg(test)]
fn clipboard_tick_with(cx: &mut App, text: Option<String>) {
    let Some(text) = text else {
        return; // 无文本/隐私拒收（None 语义在桥面）
    };
    let now = Instant::now();
    let wall = walltime::now_epoch();
    clipboard_tick_with_payload(
        cx,
        CapturedPayload {
            content_type: ContentType::PlainText,
            text: Some(text),
            bytes: None,
            hash: String::new(),
            source_app: None,
            thumb: None,
            wall,
            at: now,
        },
    );
}

/// 到达态统一链（TASK-300 · 文本与二进制同源）：重试队列先行补写 → 监控开关
/// 守门 → 指纹计算 → 缩略图生成 → 自写板跳过（文本类）→ 10s 判重 → DB 落库
///（失败入重试队列）→ 行组 → 面板注入 / pending 缓冲 → 存活清理。**唯一捕获
/// 入口**（生产 tick 与测试注入同径）。
fn clipboard_tick_with_payload(cx: &mut App, mut payload: CapturedPayload) {
    // 重试队列先行补写（REQ-CAPTURE-PERSIST 可靠承载：与监控开关解耦——关闭期
    // 存储恢复仍补写在册载荷，持久化轴不因开关停摆）。
    drain_db_retry(cx);
    // 监控开关守门（SCN-CAPTURE-SWITCH-OFF · DEC-188）：关闭期到达链全停——不
    // 判重不入缓冲不入库；重开的基线重播种由 apply_capture_enabled 承载。
    if !cx.global::<ClipCapture>().capture_enabled {
        return;
    }
    // 指纹惰性计算（测试注入态 hash 空位补齐；engine 路径已由 Captured 承载）。
    if payload.hash.is_empty() {
        payload.hash = match payload.bytes.as_ref() {
            Some(bytes) => parchora_core::hash::sha256_hex(bytes),
            None => match payload.text.as_deref() {
                Some(text) => parchora_core::hash::sha256_hex(text.as_bytes()),
                None => {
                    parchora_core::hash::sha256_hex(payload.content_type.as_db_value().as_bytes())
                }
            },
        };
    }
    // 缩略图即拍即转（SCN-CAPTURE-THUMB · DEC-188 偏差④）：image 类最长边
    // 320px PNG；桥失败 → None（行面棋盘回退 · 不虚造）。
    if payload.content_type == ContentType::Image {
        payload.thumb = payload
            .bytes
            .as_ref()
            .and_then(|b| image_thumbnail_png(b, CLIP_THUMB_EDGE_PX).map(std::sync::Arc::new));
    }
    // 自写板跳过（REQ-CAPTURE-SELF-SKIP · DEC-46）：本进程刚写的同文本不入列
    //（文本类载荷承载比对基准；二进制类无写板登记面，不涉）。
    if let Some(text) = &payload.text {
        if parchora_ui::clipstamp::fresh_match(text) {
            return;
        }
    }
    // 同内容 10s 窗判重（REQ-CAPTURE-DEDUPE · SHA256 指纹）。
    {
        let cap = cx.global_mut::<ClipCapture>();
        if !cap.recent.admit(&payload.hash, payload.at) {
            return;
        }
    }
    let row = match persist_capture(cx, &payload) {
        Persisted::NoStore => capture_row(&payload, None),
        Persisted::Failed => {
            push_db_retry(cx, payload.clone());
            capture_row(&payload, None)
        }
        Persisted::Done(id) => capture_row(&payload, Some(id)),
    };
    let panel = cx.global::<DockHandles>().panel;
    let delivered = panel
        .map(|handle| {
            handle
                .update(cx, |view, _w, cx| view.ingest_row(row.clone(), cx))
                .is_ok()
        })
        .unwrap_or(false);
    if delivered {
        // 真机判读锚（DEC-186 · 与「未取到文本」诊断行对称）：交付成功即披
        // 露，只报长度不报内容（规范 8 敏感信息零暴露）。
        eprintln!(
            "[parchora] 剪贴板捕获已入列（{} 字符）",
            payload_title_len(&payload)
        );
    } else {
        // 面板不在场（收缩态全隐/未开）或句柄坏死（DEC-186 D-2 兜底）：捕获
        // 改入缓冲，重开面板时按序冲入，不丢字。
        let cap = cx.global_mut::<ClipCapture>();
        cap.pending.push(row);
        while cap.pending.len() > CLIP_PENDING_CAP {
            cap.pending.remove(0);
        }
    }
    clip_sweep(cx);
}

/// 载荷落库结果（三态显式优于隐式）：存储不在场（内存降级）/ 写失败（调用方
/// 入重试队列）/ 成功（DB id）。
enum Persisted {
    NoStore,
    Failed,
    Done(String),
}

/// 载荷落库（REQ-CAPTURE-PERSIST 每命中事务写）：文本类直入；image 类先 blobs
/// 落盘再入行。不静默吞错（stderr 披露）；不入重试队列（职责归到达链）。
fn persist_capture(cx: &mut App, payload: &CapturedPayload) -> Persisted {
    let Some(store) = cx.try_global::<AppStore>() else {
        return Persisted::NoStore; // 无存储接线（纯内存态测试/降级）：不排队
    };
    let Ok(db) = store.store.active_db() else {
        // 接线在册但暂无活跃根（开根失败/连接态暂失）→ 写失败同构，入重试面。
        return Persisted::Failed;
    };
    let id = new_uuid();
    let (blob_rel, thumb_rel, text_content, size) = if payload.content_type == ContentType::Image {
        match payload.bytes.as_ref() {
            Some(bytes) => {
                // blob 落盘随活跃根（DEC-189 · 与 DB 同域；根缺失同写失败重试面）。
                let Some(root) = store_root(cx) else {
                    return Persisted::Failed;
                };
                match write_blob(&root, &id, "raw", bytes) {
                    Ok(rel) => {
                        // 缩略随链落盘（DEC-188）：thumb 字节由到达链即拍生成。
                        let thumb_rel = payload
                            .thumb
                            .as_ref()
                            .and_then(|t| write_blob(&root, &id, "thumb", t).ok());
                        (Some(rel), thumb_rel, None, bytes.len() as i64)
                    }
                    Err(e) => {
                        eprintln!("[parchora] blobs 落盘失败：{e}（本轮未入行）");
                        return Persisted::Failed;
                    }
                }
            }
            None => {
                // image 无载荷：不虚构造行（桥面空读——本轮丢弃，下拍再战）。
                return Persisted::Failed;
            }
        }
    } else {
        let text_len = payload.text.as_ref().map(|t| t.len() as i64).unwrap_or(0);
        (None, None, payload.text.clone(), text_len)
    };
    let clip = Clip {
        id,
        content_type: payload.content_type,
        text_content,
        content_hash: payload.hash.clone(),
        source_app: payload.source_app.clone(),
        size,
        is_pinned: false,
        blob_rel_path: blob_rel,
        thumb_rel_path: thumb_rel,
        edited_at: None,
        created_at: iso_utc(payload.wall),
    };
    match insert_clip(&db, &clip) {
        Ok(()) => Persisted::Done(id.to_string()),
        Err(e) => {
            eprintln!("[parchora] 拾记落库失败：{e}");
            Persisted::Failed
        }
    }
}

/// 重试队列先行补写（REQ-CAPTURE-PERSIST 可靠承载）：保序逐条；首败即停——
/// 存储仍不可用时剩余项原序保留，下拍再战（禁止静默丢项）。成功项回收 (at, id)
/// 对——pending 行与面板实体行按加入时刻回填 DB 身份（DEC-188 · D-06：
/// 删除仅清 UI 面 → 删了又回来的缺口）。
fn drain_db_retry(cx: &mut App) {
    if app_db(cx).is_none() {
        return;
    }
    let queue = std::mem::take(&mut cx.global_mut::<ClipCapture>().db_retry);
    if queue.is_empty() {
        return;
    }
    let mut failed_at: Option<usize> = None;
    let mut hits: Vec<(Instant, String)> = Vec::new();
    for (i, payload) in queue.iter().enumerate() {
        match persist_capture(cx, payload) {
            Persisted::Done(id) => hits.push((payload.at, id)),
            _ => {
                failed_at = Some(i);
                break;
            }
        }
    }
    if let Some(i) = failed_at {
        // 首败项及其后全部原序返还（队列头 = 最旧）。
        let remaining = queue.into_iter().skip(i).collect::<Vec<_>>();
        cx.global_mut::<ClipCapture>().db_retry = remaining;
    }
    if !hits.is_empty() {
        // pending 缓冲行回填（写失败期行仍在缓冲）。
        {
            let cap = cx.global_mut::<ClipCapture>();
            for row in cap.pending.iter_mut() {
                if row.db_id.is_none() && hits.iter().any(|(at, _)| row.added_at == *at) {
                    let id = hits
                        .iter()
                        .find(|(at, _)| row.added_at == *at)
                        .map(|(_, id)| id.clone());
                    if let Some(id) = id {
                        row.db_id = Some(id);
                    }
                }
            }
        }
        // 面板实体行回填（DEC-186 实体提权：关窗期存续实体同域可更新）。
        let panel = cx.global::<DockHandles>().panel;
        if let Some(entity) = match panel {
            Some(handle) => handle.entity(cx).ok(),
            None => cx.global::<DockHandles>().panel_entity.clone(),
        } {
            entity.update(cx, |view, cx| {
                view.update_clip_db_ids(&hits, cx);
            });
        }
    }
}

fn push_db_retry(cx: &mut App, payload: CapturedPayload) {
    let cap = cx.global_mut::<ClipCapture>();
    cap.db_retry.push(payload);
    while cap.db_retry.len() > CLIP_DB_RETRY_CAP {
        cap.db_retry.remove(0);
    }
}

/// 载荷 → 拾记行（kind 映射 + 标题选择 + db 身份；icon 统一 Type 恒值——行面
/// 语义由标题/缩略块承载，零视觉漂移 · 规范 4）。image 缩略块由 thumb_img
/// 驱动（DEC-188 真图承载；桥失败/无 thumb 文件 → 棋盘占位回退），标题取类名
/// 标注。
fn capture_row(payload: &CapturedPayload, db_id: Option<String>) -> ClipRow {
    let kind = match payload.content_type {
        ContentType::PlainText => FilterKind::PlainText,
        ContentType::RichText => FilterKind::RichText,
        ContentType::Html => FilterKind::Html,
        ContentType::Image => FilterKind::Image,
        ContentType::Url => FilterKind::Url,
        ContentType::FileUrl => FilterKind::FileUrl,
        ContentType::Color => FilterKind::Color,
        ContentType::Unknown => FilterKind::Unknown,
    };
    let title = match payload.content_type {
        ContentType::Image => "(图片)".to_string(),
        ContentType::Unknown => "(未知内容)".to_string(),
        _ => payload
            .text
            .clone()
            .unwrap_or_else(|| "(无内容)".to_string()),
    };
    let thumb_img = payload.thumb.as_ref().map(|bytes| {
        std::sync::Arc::new(gpui::Image::from_bytes(
            gpui::ImageFormat::Png,
            bytes.to_vec(),
        ))
    });
    ClipRow {
        icon: parchora_ui::components::Icon::Type,
        title,
        meta: payload.source_app.clone(),
        thumb: payload.content_type == ContentType::Image,
        kind,
        added_at: payload.at,
        wall: walltime::local_civil(payload.wall),
        db_id,
        thumb_img,
        ..Default::default()
    }
}

/// 行标题字长披露（诊断行 · 不报内容只报量 · 规范 8）。
fn payload_title_len(payload: &CapturedPayload) -> usize {
    payload
        .text
        .as_deref()
        .map(|t| t.chars().count())
        .unwrap_or(payload.bytes.as_ref().map(|b| b.len()).unwrap_or(0))
}

/// 拾记条目存活清理单拍（DEC-163/188）：双阈值（分钟）＋非 Pin 计数上限
///（0=无限）自 ClipCapture 全局——
/// 存量硬上限＝保留天数 × 1440 分钟（saturating_mul 防溢出）；面板不在场时若
/// 实体仍活着（DEC-186 实体提权后关窗期存续）同样清——跨显隐周期清理阈值不因
/// 隐藏中断（实体窗无关更新承载）。DB 侧同拍对账（REQ-CAPTURE-LIMIT：时间截止
/// + 计数上限两轴 · Pin 恒豁免 · 任务零负担随捕触发）。
fn clip_sweep(cx: &mut App) {
    let (survive, valid, limit) = {
        let cap = cx.global::<ClipCapture>();
        (
            cap.retention_days.saturating_mul(CLIP_MINUTES_PER_DAY),
            cap.valid_min,
            cap.limit,
        )
    };
    let panel = cx.global::<DockHandles>().panel;
    if let Some(entity) = match panel {
        Some(handle) => handle.entity(cx).ok(),
        None => cx.global::<DockHandles>().panel_entity.clone(),
    } {
        entity.update(cx, |view, cx| {
            view.sweep_clip(survive, valid, limit, cx);
        });
    }
    // DB 对账（同拍承载：UI 清理面与存储清理面同轴；开根失败静默跳过）。
    let mut trimmed = 0usize;
    if let Some(db) = app_db(cx) {
        let now_wall = walltime::now_epoch();
        let cutoff = now_wall
            - cx.global::<ClipCapture>()
                .retention_days
                .saturating_mul(86_400) as i64;
        match delete_unpinned_before(&db, &iso_utc(cutoff)) {
            Ok(n) => trimmed += n,
            Err(e) => eprintln!("[parchora] 拾记存量对账失败：{e}"),
        }
        // 计数上限对账（REQ-CAPTURE-LIMIT · design §3.2 立即 DELETE 保留最新 N
        // 条非 Pin；0=不限不触口）。
        if limit > 0 {
            match delete_unpinned_over_limit(&db, limit as usize) {
                Ok(n) => trimmed += n,
                Err(e) => eprintln!("[parchora] 拾记上限对账失败：{e}"),
            }
        }
    }
    // blob 差集清理随行（DEC-189 · blob 与条目同生命周期）：仅本拍确有删行才
    // 扫盘（零删除不付 IO——tick 链每捕触发本口，恒扫是热路径浪费面）。
    if trimmed > 0 {
        prune_blob_dirs(cx);
    }
}

/// 悬停详情类演示窗共用几何（--peek / --page editor 同构两处共用）：
/// 内容面板尺寸按触发位宽高互换 + 最小尺寸兜底（DEC-76 口径）。movable 由调用
/// 方定（DEC-128）：真实编辑窗 true（同主面板原生标题条拖动），
/// Peek/--page editor 演示窗仍 false。
fn detail_window_opts(cx: &App, movable: bool) -> WindowOptions {
    let tier = active_tier(cx); // DEC-198：min 尺寸随档位倍率
    let (w, h) = dock::swap_dims_if_horizontal(trigger_pos(cx), detail_size(cx));
    // 居中锚定探针屏（DEC-184）：Bounds::centered 按该屏 visible bounds 求心，
    // display_id 同源 —— 无探针（演示形态）回退 primary。
    window_opts_at(
        cx,
        Bounds::centered(panel_display(cx), size(px(w), px(h)), cx),
        Some(dock::swap_dims_if_horizontal(
            trigger_pos(cx),
            (
                s::sp_at(tier, tk::DETAIL_W),
                s::sp_at(tier, tk::DETAIL_H_MIN),
            ),
        )),
        movable,
    )
}

/// 单击驱动的 Peek 窗开立与关闭（单实例）见上方 open_or_update_peek / arm_peek_close；
/// 演示窗形态：--peek CLI · 居中 · 静态演示内容 1:1 · 底部触发位置=宽高互换。
fn open_peek(cx: &mut App) {
    let opts = detail_window_opts(cx, false);
    let _ = cx.open_window(opts, |window, cx| {
        let view = cx.new(PeekView::new);
        window.focus(&view.read(cx).focus_handle(), cx);
        view
    });
}

/// 悬停 Peek 窗开/原位更新（单实例：已有窗 handle → set_item；陈旧句柄清理重开）。
fn open_or_update_peek(cx: &mut App, item: PeekItem) {
    if let Some(handle) = cx.global::<DockHandles>().peek {
        if handle
            .update(cx, |view, _window, cx| view.set_item(item.clone(), cx))
            .is_ok()
        {
            bump_peek_close_gen(cx); // 任何 Show 都作废未决 close timer
            return;
        }
        cx.global_mut::<DockHandles>().peek = None;
    }
    open_peek_item(cx, item);
}

/// Peek「编辑」转出接手（SCN-PREVIEW-QUICKLOOK-EDIT-HANDOFF ·
/// REQ-PREVIEW-IMAGE · DEC-24）：编辑动作移交统一编辑器（Quick Look 自身不承载
/// 编辑）——读 Peek 实界 → 关 Peek（转出语义 · 防双窗同域叠置）→ EditorShell
/// （with_content 注入条目标题/正文）Peek 原址开立；实界不可读 fallback 居中
/// detail_window_opts（--page editor 同口径 · DEC-73）。单实例门（open_or_update
/// _peek 同构）：已有 handle → set_content 原位热换（同内容 no-op · DEC-94），
/// 陈旧句柄清理重开；DockHandles.editor 登记（Esc 关闭 / GUI 保持域消费）。
/// 编辑窗不入面板级联（独立任务窗）、不入子窗挂接（拖动不跟随）、编辑语义
/// （REQ-EDIT-MARKDOWN）随 TASK-504 接线。
fn open_editor_handoff(cx: &mut App, item: PeekItem) {
    let at = cx
        .global::<DockHandles>()
        .peek
        .and_then(|h| window_rect(cx, &h));
    close_peek(cx, true); // 编辑器转出强关（DEC-159）：编辑窗接管，Peek 同址消亡
                          // 单实例门（open_or_update_peek 同构）：已有 handle → set_content 原位热换
                          //（同内容 no-op · DEC-94），陈旧句柄清理重开。
    if let Some(handle) = cx.global::<DockHandles>().editor {
        // 授权稿热换前先冲刷（DEC-130 save 模型）——换入前读
        // save_state/export_note 快照；未存稿顺延首存、已存稿同名覆写；随后交回
        // 「内容不入盘」口径。同内容 no-op（DEC-94）不冲不动；非授权稿直接热换；
        // 陈旧句柄清理重开。
        let state = handle.update(cx, |view, _window, cx| {
            let (enabled, saved) = view.save_state();
            let (title, body) = view.export_note(cx);
            let swapped = view.set_content(item.title.clone(), item.body.clone(), cx);
            (enabled, saved, title, body, swapped)
        });
        match state {
            Err(_) => cx.global_mut::<DockHandles>().editor = None,
            // 同内容 no-op：窗口与保存态原样保持（不冲不动 · DEC-94）。
            Ok((_, _, _, _, false)) => return,
            // 非授权稿（handoff/演示）：原位热换即收口（内容不入盘）。
            Ok((false, ..)) => return,
            // 授权稿（新建稿路）：冲刷快照 → 记账 + 交回 handoff 口径。
            Ok((true, saved, title, body, true)) => {
                let path = save_draft_at(
                    &files_root(),
                    title.as_deref(),
                    body.as_deref(),
                    saved.as_deref(),
                );
                let stem = path
                    .as_ref()
                    .and_then(|p| p.file_stem())
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_string());
                let _ = handle.update(cx, |view, _window, cx| {
                    view.cancel_title_edit(cx);
                    if let Some(stem) = stem {
                        view.remember_saved(stem);
                    }
                    view.set_disk_save(false);
                });
                // 冲刷落盘后集锦列表磁盘面同步（DEC-144）：换出稿落盘（首存/覆
                // 写/另落）→ 重扫即收；同内容 no-op（不冲不动）与非授权稿（不落
                // 盘）两臂提前 return 不触发。
                sync_disk_notes(cx);
                return;
            }
        }
    }
    open_editor_window(cx, item.title, item.body, at, false);
}

/// 新建稿标题常量（DEC-129）：落盘判定与 FAB 复按守卫共用。守卫判定
/// save_state 双态（授权旗标 + 已落盘题名 · DEC-130 save 模型）——标题不参与
/// 授权判定；命名＝净化题名（sanitize_note_title）。
const UNTITLED: &str = "Untitled";

/// 编辑窗共用开立体：handoff（Peek 原址）与新建
///（Untitled 空稿）共用装配；DockHandles.editor 登记（Esc 关闭 / GUI 保持域消
/// 费）。编辑窗不入面板级联（独立任务窗）、不入子窗挂接（拖不跟随主面板）；
/// 可拖性＝is_movable=true（DEC-128 · 同主面板原生标题条拖动，
/// 不参与重泊守护——浮动详情窗非贴缘实体）。编辑语义（REQ-EDIT-MARKDOWN）随
/// TASK-504 接线。new_draft（DEC-130 save 模型）：true＝新建稿落盘
/// 授权（editor_exit 漏斗冲刷），false＝handoff/演示「内容不入盘」。
fn open_editor_window(
    cx: &mut App,
    title: String,
    body: String,
    at: Option<(f32, f32, f32, f32)>,
    new_draft: bool,
) {
    let tier = active_tier(cx); // DEC-198：min 尺寸随档位倍率
    let opts = match at {
        Some((x, y, w, h)) => window_opts_at(
            cx,
            // 编辑窗接管实界为全局坐标（DEC-184）：同 panel/peek 换算路径。
            to_panel_screen_local(cx, Bounds::new(point(px(x), px(y)), size(px(w), px(h)))),
            Some(dock::swap_dims_if_horizontal(
                trigger_pos(cx),
                (
                    s::sp_at(tier, tk::DETAIL_W),
                    s::sp_at(tier, tk::DETAIL_H_MIN),
                ),
            )),
            // 可拖（DEC-128）：原生标题条拖动同主面板源。
            true,
        ),
        None => detail_window_opts(cx, true),
    };
    let handle = match cx.open_window(opts, |window, cx| {
        let view = cx.new(|cx| {
            let mut view = EditorShell::with_content(title, body, cx);
            // 落盘授权装配（DEC-130）：新建稿 true / handoff false
            //（editor_exit 漏斗以 save_state 判定冲刷）。
            view.set_disk_save(new_draft);
            view.set_event_sink(Box::new(
                |ev: &editor::EditorEvent, cx: &mut App| match ev {
                    // Esc / 返回钮 / 「定位到磁盘」三源统一事件 → editor_exit
                    // 单漏斗（保存判定 + reveal 旗标分派 · DEC-129）；
                    // 窗确已关闭 → 主界面恢复（DEC-178「返回 → 先关
                    // 当前面板，再显示主界面」）。
                    // on_key_down listener 链持 EditorShell 租借——延迟出借域
                    //（DEC-77）。
                    editor::EditorEvent::Escape => defer_app(cx, editor_escape_exit),
                    // IME 组合期让位（DEC-224 · DEC-242）：组合期自有窗逐窗
                    // 降级、结束逐窗恢复（associated 档案）——候选窗浮层恒
                    // 可见。平台 FFI 直调（无 gpui 态变更，无需 defer）。
                    editor::EditorEvent::Composing { active } => {
                        parchora_platform::ime_key_window_level(*active);
                    }
                },
            ));
            view
        });
        // 聚焦定向根（Esc 经 key_context("Editor") 分派 · DEC-73 Peek 同构）。
        window.focus(&view.read(cx).focus_handle(), cx);
        view
    }) {
        Ok(handle) => handle,
        Err(e) => {
            eprintln!("[parchora] 编辑窗开窗失败（降级：本拍不开编辑窗）：{e}");
            return;
        }
    };
    cx.global_mut::<DockHandles>().editor = Some(handle);
}

/// 集锦层 FAB「新建笔记」→ 编辑窗空稿：Untitled 空稿进编辑区 480×70%
///（PG-B1 编辑态承接 · 落盘随 editor_exit 漏斗 files 原子写；单实例热换语义同
/// handoff）。守卫判定（DEC-130 save 模型）：未存新稿（授权且未落盘·可能已键
/// 入）→ 保持现稿不冲；已存稿 → 冲刷覆写（同题）后热换 blank 新稿；handoff/
/// 演示稿 → 直接热换（内容不入盘·原稿随弃）；陈旧句柄 → 清理重开。
fn open_editor_new(cx: &mut App) {
    open_editor_from_clip(cx, String::new());
}

/// 拾记右键「转为笔记」→ 编辑窗预填（DEC-194 真实承载）：条目原文进正文、
/// 题名留 Untitled 由用户定夺。窗位/守卫/冲刷语义与上方 open_editor_new 同链
///（DEC-129/130/144）：未存授权稿保护不冲；已存稿同名覆写冲刷后热换；
/// handoff/演示稿热换即弃；热换目标＝预填新稿（落盘授权 · 随 editor_exit 漏
/// 斗落盘）；陈旧句柄清理重开。
fn open_editor_from_clip(cx: &mut App, content: String) {
    if let Some(handle) = cx.global::<DockHandles>().editor {
        let guard = handle.update(cx, |view, _window, cx| {
            let (enabled, saved) = view.save_state();
            let (title, body) = view.export_note(cx);
            (enabled, saved, title, body)
        });
        // 冲刷落盘捕获（DEC-144）：Some=已存稿冲刷落盘 → 触发集锦面同步。
        let mut flushed = None;
        match guard {
            Ok((true, None, _, _)) => {
                return; // 未保存新稿会话（可能已键入）：保持现稿不冲内容。
            }
            Ok((true, saved, title, body)) => {
                // 已存稿冲刷（owned=已落盘题名 → 同名覆写续存），随后热换预填新稿。
                flushed = save_draft_at(
                    &files_root(),
                    title.as_deref(),
                    body.as_deref(),
                    saved.as_deref(),
                );
            }
            _ => {} // handoff/演示稿：热换即弃置；陈旧句柄走重开。
        }
        // 冲刷落盘 → 集锦列表磁盘面同步（DEC-144；未存新稿/handoff 臂无落盘不
        // 触发，flushed.is_some() 门控）。
        if flushed.is_some() {
            sync_disk_notes(cx);
        }
        if handle
            .update(cx, |view, _window, cx| {
                view.reset_new_draft_with(UNTITLED.into(), content.clone(), cx);
            })
            .is_ok()
        {
            return;
        }
        cx.global_mut::<DockHandles>().editor = None;
    }
    open_editor_window(cx, UNTITLED.to_string(), content, None, true);
}

/// 状态栏菜单「退出」清理链（触发源 = 统一菜单退出项 · DEC-121）：关四窗（幂
/// 等）→ 移除状态栏 item（removeStatusItem+release
/// 保活引用）→ 摘除鼠标 tap 事件源（句柄 Drop = CFRunLoopStop → 线程 join →
/// 槽摘除/Source/MachPort 失效回收 · 先解事件源再退进程）→ 官方优雅退出
///（App::quit · NSApp terminate 常规路径；余下后台守护/临时文件随进程终止由
/// OS 回收）。
fn quit_app(cx: &mut App) {
    eprintln!("[parchora] 状态栏菜单退出 · 清理链（关窗+移除状态栏 item+摘鼠标 tap+quit）");
    close_panel(cx);
    close_peek(cx, true); // 退出强关（DEC-159）：进程终结，stderr 留档
    close_settings(cx);
    close_editor(cx);
    close_task_create(cx);
    parchora_platform::remove_status_item();
    // 鼠标 tap 句柄收口：Drop 内 CFRunLoopStop → join；槽/事件源
    // 生命周期纪律在 platform 收尾链已闭环（tap 事件源先于进程终止解挂）。
    if let Ok(mut g) = TAP_HANDLE.lock() {
        let _ = g.take();
    }
    // 退出检点（DEC-89 双时机 · 退出臂）：WAL TRUNCATE 复零后交 OS。
    quit_checkpoint(cx);
    cx.quit();
}

/// 关闭编辑转出窗（无窗幂等 · 独立任务窗——不入面板级联）：统一
/// 收口 editor_exit（保存判定内嵌）。返回值（bool）不消费——quit 链不走
/// ensure 恢复口（DEC-178 挂点在事件臂 editor_escape_exit）。
fn close_editor(cx: &mut App) {
    let _ = editor_exit(cx);
}

/// 待办创建面板窗打开（DEC-165）：单实例幂等（settings 先例——alive 即跳过，
/// 防重复覆盖已键入内容；陈旧句柄清理重开）；几何与「创建集锦」编辑窗同函数
///（detail_window_opts 可拖 · 480×70% 居中）；开窗后根聚焦（Esc 经根
/// key_context「TaskCreate」分派）。提交/收口经 TaskCreateEvent 漏斗（listener
/// 链持 TaskCreateView 租借——延迟出借域 · DEC-77 同 EditorShell Esc 漏斗）。
fn open_task_create(cx: &mut App) {
    open_task_create_inner(cx, None);
}

/// 拾记右键「转为任务」→ 创建面板预填（DEC-194 真实承载）：条目原文进「任务」
/// 多行域、题名不动由用户定夺。窗幂等/激活语义同 open_task_create；已开窗
/// 激活后仅预填「任务」域（显式转换动作，旧任务域正文让位），未开窗建窗即预填
///（任务引擎首帧建时落值）。
fn open_task_create_from_clip(cx: &mut App, content: String) {
    open_task_create_inner(cx, Some(content));
}

/// 创建面板开窗统一体（DEC-165 幂等/激活 + DEC-194 可选预填）：prefill=None
/// ＝ FAB 语义（DEC-166 不重置已键入内容），Some＝转换预填（仅任务域）。
fn open_task_create_inner(cx: &mut App, prefill: Option<String>) {
    if let Some(handle) = cx.global::<DockHandles>().task_create {
        if handle
            .update(cx, |view, window, cx| {
                if let Some(text) = &prefill {
                    view.prefill_task(text.clone(), window, cx);
                }
                window.activate_window()
            })
            .is_ok()
        {
            return; // 仍在 → 幂等跳过 + 抬升既有窗（DEC-166 · 预填仅触任务域）。
        }
        cx.global_mut::<DockHandles>().task_create = None;
    }
    let handle = match cx.open_window(detail_window_opts(cx, true), |window, cx| {
        let view = cx.new(|cx| {
            let mut view = TaskCreateView::new(cx);
            view.set_event_sink(Box::new(|ev: &TaskCreateEvent, cx: &mut App| match ev {
                TaskCreateEvent::Submit {
                    title,
                    bucket,
                    priority,
                    mark,
                } => {
                    let title = title.clone();
                    let bucket = *bucket;
                    let priority = *priority;
                    let mark = *mark;
                    defer_app(cx, move |cx| {
                        task_create_submit(cx, title, bucket, priority, mark)
                    });
                }
                TaskCreateEvent::BackEmpty => defer_app(cx, task_create_back_empty),
                // 取消 → 窗尽即回工作台（DEC-178 漏斗级超集 · 与返回钮同义：
                // 独立面板会话结束统一恢复主界面，若隐）。listener 链持租——
                // 延迟出借域（DEC-77）。
                TaskCreateEvent::Cancel => defer_app(cx, |cx| {
                    close_task_create(cx);
                    ensure_panel_visible(cx);
                }),
            }));
            view
        });
        // 转换预填（DEC-194）：引擎未建态由 prefill_task 暂存，任务引擎首帧建
        //（ensure_task）时落值；题名不动。
        if let Some(text) = prefill {
            view.update(cx, |v, cx| v.prefill_task(text, window, cx));
        }
        window.focus(&view.read(cx).focus_handle(), cx);
        view
    }) {
        Ok(handle) => handle,
        Err(e) => {
            eprintln!("[parchora] 任务创建窗开窗失败（降级：本拍不开窗）：{e}");
            return;
        }
    };
    cx.global_mut::<DockHandles>().task_create = Some(handle);
}

/// 关闭待办创建面板窗（无窗幂等 · close_settings 同式）。
fn close_task_create(cx: &mut App) {
    let handle = cx.global_mut::<DockHandles>().task_create.take();
    if let Some(handle) = handle {
        let _ = handle.update(cx, |_, window, _| window.remove_window());
    }
}

/// 待办创建面板提交漏斗（DEC-165/166 · DEC-178 恢复口）：主面板不在场（隐藏/
/// 未开）→ 先 restore 主界面再投递——返回钮（保存并关闭口径）与创建钮同漏斗
///（DEC-178「返回 → 先关当前面板，再显示主界面」的超集承载：提交会话统一回
/// 工作台）。入行权威在 BoardView（created/序号/成功 Toast 均经
/// panel.ingest_created_task 转发）；主面板陈旧句柄（Some 而实界已亡）→ 投递
/// 失败驻留（DEC-166）：不关窗 + stderr 披露 + 创建面板就地 Toast 可重试——
/// 编辑不静默丢（trash_note 失败面同口径；BackEmpty 空稿无可保内容不走此路）。
fn task_create_submit(cx: &mut App, title: String, bucket: usize, priority: usize, mark: usize) {
    if cx.global::<DockHandles>().panel.is_none() {
        ensure_panel_visible(cx);
    }
    let delivered = match cx.global::<DockHandles>().panel {
        Some(handle) => handle
            .update(cx, |panel, _w, cx| {
                panel.ingest_created_task(title.clone(), bucket, priority, mark, cx)
            })
            .is_ok(),
        None => false,
    };
    if delivered {
        close_task_create(cx);
        return;
    }
    eprintln!(
        "[parchora] 待办提交「{title}」未入列：主面板已关闭（面板驻留 · 重开主面板后可重试）"
    );
    if let Some(handle) = cx.global::<DockHandles>().task_create {
        let _ = handle.update(cx, |view, _w, cx| view.notify_submit_failed(cx));
    }
}

/// 待办创建面板「返回」空标题漏斗（DEC-165 保存并关闭口径 · DEC-178 恢复口）：
/// 关窗 → 主界面恢复（若隐 · 幂等）→ 列表披露 Toast（恢复后具可达承载面；
/// 恢复异常（open 失败）→ stderr 自证兜底）。
fn task_create_back_empty(cx: &mut App) {
    close_task_create(cx);
    ensure_panel_visible(cx);
    match cx.global::<DockHandles>().panel {
        Some(handle) => {
            let _ = handle.update(cx, |panel, _w, cx| panel.task_create_closed_empty(cx));
        }
        None => {
            eprintln!("[parchora] 待办创建面板空稿收口披露不可达：主面板已关闭（Toast 无承载面）")
        }
    }
}

/// 编辑窗统一收口漏斗（DEC-130 save 模型）：Esc / 返回钮 /
/// 「定位到磁盘」三源统一事件落点。先读 reveal 旗标（ed-finder 置位 · 单一消费
/// 点）与 save 授权态（app 层装配：新建稿 true / handoff·演示 false），授权稿
/// 正文非空 → files 原子写落盘（未存稿顺延首存；已存稿同题覆写；行内改名另落
/// 新名·旧稿保留）；正文空不产生文件；非授权
/// 稿不落盘。落盘成功回写 saved_name（reveal 路窗持开启 → 下次退出同题覆写续
/// 存）。分派： reveal=true → 窗保持开启，有落盘即 reveal 文件，否则 reveal
/// files 根目录；reveal=false → 关窗。返回 bool：true = 窗已闭合（返回钮/Esc
/// 常规收口）——editor_escape_exit 的主界面恢复信号（DEC-178）；false = reveal
/// 窗保持开启 / 陈旧句柄（无恢复口）。
fn editor_exit(cx: &mut App) -> bool {
    let Some(handle) = cx.global_mut::<DockHandles>().editor.take() else {
        return false;
    };
    let state = handle.update(cx, |view, _window, cx| {
        let reveal = view.take_reveal();
        let (enabled, saved) = view.save_state();
        (reveal, enabled, saved, view.export_note(cx))
    });
    let Ok((reveal, enabled, saved, (title, body))) = state else {
        return false; // 陈旧句柄：保持 take（窗已随实体失效）。
    };
    let path = if enabled {
        save_draft_at(
            &files_root(),
            title.as_deref(),
            body.as_deref(),
            saved.as_deref(),
        )
    } else {
        None
    };
    // 落盘成功回写（续存面）：下次退出 owned=题名 → 同名覆写不增殖。
    if let Some(path) = &path {
        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
            let stem = stem.to_string();
            let _ = handle.update(cx, |view, _window, _cx| view.remember_saved(stem));
        }
    }
    // 落盘后集锦列表磁盘面同步（DEC-144）：有落盘（首存/覆写/另落新名）→ 重扫
    // 即收新条目；无落盘（空稿/handoff）不触发（无新增面，省一次目录读）。
    if path.is_some() {
        sync_disk_notes(cx);
    }
    if reveal {
        // 窗保持开启（旗标已消费，后续 Esc 常规关窗）；无可定位文件 → 先建根目录
        // 再 reveal（打开目标目录）。
        cx.global_mut::<DockHandles>().editor = Some(handle);
        match path {
            Some(path) => cx.reveal_path(&path),
            None => {
                let root = files_root();
                let _ = std::fs::create_dir_all(&root);
                cx.reveal_path(&root);
            }
        }
        // 窗开启中（会话未结束）：不给面板恢复口（DEC-178 仅「确已关窗」出口触发）。
        false
    } else {
        let _ = handle.update(cx, |_, window, _| window.remove_window());
        // 窗已闭合（返回钮/Esc）：真值即 editor_escape_exit 的恢复信号（DEC-178）。
        true
    }
}

/// 编辑稿落盘执行（DEC-130）：正文空（trim 后）不产生文
/// 件；未存稿（owned 与净化题名失配或 None）走
/// write_note 同名顺延（不覆盖既有笔记），已存同题走 rewrite_note 原地覆写；
/// 行内改名另落新名（旧稿保留——题名即命名源）。root 显式入参（temp root 测
/// 试）；生产调用点传 files_root()。授权判定不在本函数（save_state 双态由调用
/// 方（editor_exit 漏斗 / hot-swap 冲刷路）自持）。
fn save_draft_at(
    root: &std::path::Path,
    title: Option<&str>,
    body: Option<&str>,
    owned: Option<&str>,
) -> Option<std::path::PathBuf> {
    let body = body?;
    if body.trim().is_empty() {
        return None;
    }
    let name = parchora_files::sanitize_note_title(title.unwrap_or(UNTITLED));
    let result = if owned == Some(name.as_str()) {
        parchora_files::rewrite_note(root, &name, body)
    } else {
        parchora_files::write_note(root, &name, body)
    };
    match result {
        Ok(path) => {
            eprintln!("[parchora] 编辑稿自动保存 → {}", path.display());
            Some(path)
        }
        Err(e) => {
            eprintln!("[parchora] 编辑稿落盘失败: {e}");
            None
        }
    }
}

/// Peek 编辑保存结果（DEC-159）：Noop ＝ 非磁盘条目/空正文（不入盘口径 ·
/// 非失败）；Saved ＝ 落盘成功；Failed ＝ 存储失败（io 错误明文上抛，调用方
/// 承载错误面：bar 条幅 + 失败驻留/寄存重试——不静默丢失）。
#[derive(Debug)]
enum PeekSaveOutcome {
    Noop,
    Saved,
    Failed(String),
}

/// Peek 面板内编辑保存漏斗（DEC-156 · DEC-159 失败面化）：磁盘真实条目
///（disk_real）原地覆写——rewrite_note（题名词干即盘名）+ 集锦列表磁盘面同步
///（DEC-144 同源漏斗）；演示/剪贴板/待办条目不入盘（与编辑窗 handoff「内容不
/// 入盘」同口径）；空正文不落盘（save_draft_at 同口径，Noop 承载非失败）。
/// 保存执行直连 rewrite_note（owned=题名词干语义同 DEC-156，但失败
/// io::Result 逐支上抛——save_draft_at 的 None 折叠无法区分失败/空稿）。root
/// 显式入参（temp root 测试）；生产调用点传 files_root()。
fn save_peek_edit_at(root: &std::path::Path, item: &PeekItem, cx: &mut App) -> PeekSaveOutcome {
    if !item.disk_real || item.body.trim().is_empty() {
        return PeekSaveOutcome::Noop;
    }
    let name = parchora_files::sanitize_note_title(&item.title);
    match parchora_files::rewrite_note(root, &name, &item.body) {
        Ok(path) => {
            eprintln!("[parchora] Peek 面板内编辑保存 → {}", path.display());
            sync_disk_notes(cx);
            PeekSaveOutcome::Saved
        }
        Err(e) => {
            eprintln!("[parchora] Peek 面板内编辑落盘失败: {e}");
            PeekSaveOutcome::Failed(format!("{e}"))
        }
    }
}

/// 生产入口（files 根 · DEC-156/159）。
fn save_peek_edit(cx: &mut App, item: PeekItem) -> PeekSaveOutcome {
    save_peek_edit_at(&files_root(), &item, cx)
}

/// files 存储根（DEC-41 双目录 data/+files/ 的 files 面；演示宿主固定
/// ~/.parchora/files；HOME 缺失时退相对路（测试/沙箱容错））。
fn files_root() -> std::path::PathBuf {
    match std::env::var("HOME") {
        Ok(home) => std::path::PathBuf::from(home).join(APP_DIR).join("files"),
        Err(_) => std::path::PathBuf::from(".parchora/files"),
    }
}

/// files 同级 `.trash` 目录推导（DEC-195 · 回收域动作三处共享：trash_note /
/// purge / empty，REQ-STORE-TRASH 指定位）。root 显式入参（temp root 测试；生产
/// files_root()）。
fn trash_dir_for(root: &std::path::Path) -> std::path::PathBuf {
    root.with_file_name(".trash")
}

/// 集锦首页笔记行「删除→回收站」（DEC-164 · REQ-STORE-TRASH 文件侧切片）：题名
/// 词干原值定位根面 `.md`（勿再净化——盘名即命名源，外部落盘文件以盘名为准），
/// rename 移入 files 同级 `.trash/`（REQ-STORE-TRASH 指定位；同名顺延不覆盖；
/// mtime 保留承载 60 天期基座）并回灌列表（行消失＝用户可见成功面）与回收站
/// 列表（DEC-195·行随拍入站）。DB trash 记录/uuid 前缀命名/60 天启动清理随
/// trash crate 接线（TASK-50x · 偏差台账 DEC-164）。
fn trash_note_at(
    root: &std::path::Path,
    title: &str,
    cx: &mut App,
) -> std::io::Result<std::path::PathBuf> {
    let dst = parchora_files::trash_note(root, &trash_dir_for(root), title)?;
    eprintln!("[parchora] 笔记移入回收站 → {}", dst.display());
    sync_disk_notes(cx);
    sync_trash_notes(cx);
    Ok(dst)
}

/// 生产入口（files 根 · 同级 .trash 推导内聚于 trash_note_at）。
fn trash_note(cx: &mut App, title: &str) {
    if let Err(e) = trash_note_at(&files_root(), title, cx) {
        // 失败不吞（7.7.1）：stderr 自证；列表不动＝用户可见失败面，可重试。
        // Toast 失败反馈面随 PeekSaveOutcome 三态同型收口（遗留 · DEC-164）。
        eprintln!("[parchora] 笔记移入回收站失败「{title}」: {e}");
    }
}

/// 回收站磁盘笔记扫描（DEC-195 · 镜像 scan_disk_notes_at 口径）：仅收普通
/// `.md`——点前缀临时残留/子目录/非 md 一律跳过；mtime 降序（新删置顶）+
/// 题名升序定稳态 tie；正文读失败降级空串。title＝文件名词干；目录不存在
///（首次删除前）＝空集。
fn scan_trash_notes_at(trash_dir: &std::path::Path) -> Vec<DiskNote> {
    let Ok(rd) = std::fs::read_dir(trash_dir) else {
        return Vec::new();
    };
    let mut notes: Vec<(u64, DiskNote)> = Vec::new();
    for entry in rd.flatten() {
        let p = entry.path();
        if !p.is_file() {
            continue;
        }
        let Some(name) = p.file_name().and_then(|s| s.to_str()) else {
            continue;
        };
        if name.starts_with('.') || !name.to_ascii_lowercase().ends_with(".md") {
            continue;
        }
        let title = p
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(name)
            .to_string();
        let body = std::fs::read_to_string(&p).unwrap_or_else(|e| {
            eprintln!("[parchora] 回收站笔记读取失败 {}: {e}", p.display());
            String::new()
        });
        let meta = entry.metadata().ok();
        let mtime_ms = meta
            .as_ref()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        let age_secs = meta
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.elapsed().ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);
        notes.push((
            mtime_ms,
            DiskNote {
                title,
                body,
                created: rel_time_label(age_secs),
            },
        ));
    }
    notes.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.title.cmp(&b.1.title)));
    notes.into_iter().map(|(_, n)| n).collect()
}

/// 回收站页真实面同步漏斗（DEC-195 · 唯一刷新入口）：扫描 files 同级 `.trash/`
/// → 注入回收站页列表（全量替换幂等 · 镜像 sync_disk_notes 口径）。触发点四处：
/// 面板唤出（open_panel_docked 尾）、首页删除真移后（trash_note_at）、单件永久
/// 删除后、清空后。无面板在挂 = 安全 no-op。
fn sync_trash_notes(cx: &mut App) {
    let notes = scan_trash_notes_at(&trash_dir_for(&files_root()));
    if let Some(handle) = cx.global::<DockHandles>().panel {
        let _ = handle.update(cx, |panel, _window, cx| panel.set_trash_notes(notes, cx));
    }
}

/// 回收站单件永久删除（DEC-195 · 仅回收站域删除链用）：物理 remove_file 并回灌
/// 列表；失败 stderr 自证＋行不消（可重试，同 trash_note 惯例）。root 显式入参
///（temp root 测试；生产 files_root()）。
fn purge_trash_note_at(root: &std::path::Path, title: &str, cx: &mut App) {
    match parchora_files::purge_trash_note(&trash_dir_for(root), title) {
        Ok(()) => {
            eprintln!("[parchora] 回收条目永久删除 → {title}");
        }
        Err(e) => eprintln!("[parchora] 回收条目永久删除失败「{title}」: {e}"),
    }
    sync_trash_notes(cx);
}

/// 生产入口（files 根 · purge）。
fn purge_trash_note(cx: &mut App, title: &str) {
    purge_trash_note_at(&files_root(), title, cx);
}

/// 清空回收站（DEC-195 · DG-CONFIRM 确认后唯一入口）：物理删除全部 .md 并回灌
/// 列表；失败 stderr 自证（已删项保留，重试幂等）。root 显式入参（temp root 测试；
/// 生产 files_root()）。
fn empty_trash_at(root: &std::path::Path, cx: &mut App) {
    match parchora_files::empty_trash(&trash_dir_for(root)) {
        Ok(n) => eprintln!("[parchora] 回收站已清空（{n} 项）"),
        Err(e) => eprintln!("[parchora] 清空回收站失败: {e}"),
    }
    sync_trash_notes(cx);
}

/// 生产入口（files 根 · empty）。
fn empty_trash(cx: &mut App) {
    empty_trash_at(&files_root(), cx);
}

/// 相对时距标注（DEC-144 · 纯函数可测）：磁盘笔记条目「创建时距」文案桶。
fn rel_time_label(secs: u64) -> String {
    if secs < 60 {
        "刚刚".into()
    } else if secs < 3600 {
        format!("{} 分钟前", secs / 60)
    } else if secs < 86_400 {
        format!("{} 小时前", secs / 3600)
    } else if secs < 2_592_000 {
        format!("{} 天前", secs / 86_400)
    } else {
        "更早".into()
    }
}

/// files 根磁盘笔记扫描（DEC-144）：仅收根面普通 `.md` 笔记——目录 / 点前缀名
///（写稿临时文件 `.{name}.tmp` 与共置图片目录 `.<笔记名>/`）/ 非 md 一律跳过；
/// mtime 降序（新建置顶，真实面=列表文件组最前）+ 题名升序定稳态 tie；正文读
/// 失败降级空串（stderr 落档不中断扫描）。title=文件名词干（与 saved_name 口
/// 径一致）。
fn scan_disk_notes_at(root: &std::path::Path) -> Vec<DiskNote> {
    let Ok(rd) = std::fs::read_dir(root) else {
        return Vec::new(); // 根不存在（首次落盘前）= 空注入，幂等安全。
    };
    let mut notes: Vec<(u64, DiskNote)> = Vec::new();
    for entry in rd.flatten() {
        let p = entry.path();
        if !p.is_file() {
            continue;
        }
        let Some(name) = p.file_name().and_then(|s| s.to_str()) else {
            continue;
        };
        if name.starts_with('.') || !name.to_ascii_lowercase().ends_with(".md") {
            continue;
        }
        let title = p
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(name)
            .to_string();
        let body = std::fs::read_to_string(&p).unwrap_or_else(|e| {
            eprintln!("[parchora] 磁盘笔记读取失败 {}: {e}", p.display());
            String::new()
        });
        let meta = entry.metadata().ok();
        let mtime_ms = meta
            .as_ref()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        let age_secs = meta
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.elapsed().ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);
        notes.push((
            mtime_ms,
            DiskNote {
                title,
                body,
                created: rel_time_label(age_secs),
            },
        ));
    }
    notes.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.title.cmp(&b.1.title)));
    notes.into_iter().map(|(_, n)| n).collect()
}

/// 生产入口：扫描 files 根（DEC-144）。
fn scan_disk_notes() -> Vec<DiskNote> {
    scan_disk_notes_at(&files_root())
}

/// 集锦页磁盘真实面同步漏斗（DEC-144 · 唯一刷新入口）：扫描 files 根 → 注入
/// 集锦浏览列表（全量替换幂等）。无主面板 / 句柄失效 = 安全 no-op（编辑窗独立
/// 任务窗不依赖面板在挂，唤出时统一取最新面）。触发点四处：面板唤出
///（open_panel_docked 尾）、editor_exit 落盘回写后、handoff 热换冲刷臂、
/// open_editor_new 冲刷臂。
fn sync_disk_notes(cx: &mut App) {
    let notes = scan_disk_notes();
    if let Some(handle) = cx.global::<DockHandles>().panel {
        let _ = handle.update(cx, |panel, _window, cx| panel.set_disk_notes(notes, cx));
    }
}

/// 主面板实际窗界（全局坐标 · window_rect 缝 DEC-99）——Peek 邻接锚（DEC-76：
/// 拖曳后仍紧贴）；无面板/句柄失效 → 按 panel_bounds 纯函数推定（与开窗几何同源
/// 兜底）。
fn panel_actual_bounds(cx: &mut App) -> (f32, f32, f32, f32) {
    if let Some(handle) = cx.global::<DockHandles>().panel {
        if let Some(b) = window_rect(cx, &handle) {
            return b;
        }
    }
    dock::panel_bounds(trigger_pos(cx), screen_rect(cx), main_panel_size(cx))
}

/// 主面板实界域内判定（显隐守护两路共用 · reveal 轮询收起域与失活期探测同口径）：
/// 窗界含边闭合区间（全局坐标对比全局光标 · window_rect 缝 DEC-99）；句柄失效
/// （窗已关）→ false（视为离板）。
fn panel_contains(cx: &mut App, handle: &WindowHandle<PanelView>, x: f32, y: f32) -> bool {
    window_rect(cx, handle)
        .is_some_and(|(bx, by, bw, bh)| x >= bx && x <= bx + bw && y >= by && y <= by + bh)
}

/// Peek 窗开立：主面板同侧邻接（dock::peek_bounds · 主面板实界锚定 · DEC-76）·
/// 底部触发宽高互换尺寸 · 单实例句柄登记（需求 3：同时最多 1 内容详情展示界面）。窗内悬停事件注入
/// （app 层对称 0.12s 宽限关闭 + 回窗取消，peek_close_gen 防争）。
fn open_peek_item(cx: &mut App, item: PeekItem) {
    let pos = trigger_pos(cx);
    let tier = active_tier(cx); // DEC-198：min 尺寸随档位倍率
    let (w, h) = dock::swap_dims_if_horizontal(pos, detail_size(cx));
    let (x, y, w, h) = dock::peek_bounds(pos, screen_rect(cx), panel_actual_bounds(cx), (w, h));
    let opts = window_opts_at(
        cx,
        // Peek 实界为全局坐标（DEC-184）：同 panel 换算路径——锚定面板所在屏。
        to_panel_screen_local(cx, Bounds::new(point(px(x), px(y)), size(px(w), px(h)))),
        Some(dock::swap_dims_if_horizontal(
            pos,
            (
                s::sp_at(tier, tk::DETAIL_W),
                s::sp_at(tier, tk::DETAIL_H_MIN),
            ),
        )),
        false,
    );
    let handle = match cx.open_window(opts, |window, cx| {
        let view = cx.new(|cx| {
            let mut view = PeekView::with_item(item, cx);
            view.set_event_sink(Box::new(|ev: &peek::PeekEvent, cx: &mut App| match ev {
                peek::PeekEvent::HoverChanged(true) => {
                    // 回到窗内 → 作废未决关闭（gen 防争）。
                    bump_peek_close_gen(cx);
                }
                peek::PeekEvent::HoverChanged(false) => {
                    // 指针离开 Peek 窗 → 对称 0.12s 宽限后关（gen 不变才落关）。
                    // arm_peek_close 仅触全局+自持 timer 任务，无同实体回触（同步保）。
                    arm_peek_close(cx);
                }
                // Esc 级联第 2 级：只关 Peek，面板不动（§〇-4 / §十 · DEC-73）；
                // on_key_down listener 链持 PeekView 租借，close_peek 触同实体——
                // 延迟出借域（DEC-77）。
                peek::PeekEvent::Escape => defer_app(cx, |cx| close_peek(cx, false)),
                // 编辑转出（REQ-PREVIEW-IMAGE · DEC-24）：关 Peek
                // → 统一编辑器窗原址接手（含单实例门 · open_editor_handoff 内聚）；
                // fire 链持 PeekView 租借，延迟出借域同构。
                peek::PeekEvent::EditHandoff(item) => {
                    let item = item.clone();
                    defer_app(cx, move |cx| open_editor_handoff(cx, item));
                }
                // IME 组合期让位（DEC-224 · R184 F-183-11 Peek 镜像 editor 窗
                // 同口径 · DEC-242）：组合期自有窗逐窗降级＋组合结束/引擎拆卸
                // 逐窗恢复（associated 档案）——候选窗浮层恒在上。平台 FFI 直
                // 调（无 gpui 态变更，无需 defer）。
                peek::PeekEvent::Composing { active } => {
                    parchora_platform::ime_key_window_level(*active);
                }
                // 面板内编辑保存（DEC-156）：退出编辑态 / 编辑中热换异题条目
                // 冲刷上抛——磁盘真实条目原地覆写 + 集锦列表同步，非磁盘条目
                // 不入盘（handoff 同口径）。fire 链持 PeekView 租借，延迟出借域
                // 同构。
                peek::PeekEvent::SaveEdit(item) => {
                    let item = item.clone();
                    // 保存结果三态（DEC-159）：Saved → 同题失败面清除；Failed
                    // → bar 错误条幅 + 失败条目寄存（下次关窗冲刷先重试——
                    // 编辑不静默丢失）；Noop（非磁盘/空正文口径）无面。
                    defer_app(cx, move |cx| match save_peek_edit(cx, item.clone()) {
                        PeekSaveOutcome::Saved => {
                            if let Some(handle) = cx.global::<DockHandles>().peek {
                                let _ = handle.update(cx, |view, _window, cx| {
                                    view.clear_save_failed(&item.title, cx)
                                });
                            }
                        }
                        PeekSaveOutcome::Failed(err) => {
                            if let Some(handle) = cx.global::<DockHandles>().peek {
                                let _ = handle.update(cx, |view, _window, cx| {
                                    view.mark_save_failed(item, &err, cx)
                                });
                            }
                        }
                        PeekSaveOutcome::Noop => {}
                    });
                }
            }));
            view
        });
        // 键焦定向 Peek 根（Esc 事件经 focus 路径分派 · DEC-73）。
        window.focus(&view.read(cx).focus_handle(), cx);
        view
    }) {
        Ok(handle) => handle,
        Err(e) => {
            eprintln!("[parchora] Peek 窗开窗失败（降级：本拍不开预览）：{e}");
            return;
        }
    };
    let handles = cx.global_mut::<DockHandles>();
    handles.peek = Some(handle);
    handles.peek_close_gen += 1;
    // 拖动跟随挂接：面板实界为父窗、Peek 实界为子窗 → AppKit
    // addChildWindow 原生父子关联（此后拖动仅主面板可拖，Peek 同步跟随）。
    attach_peek_child(cx);
}

/// Peek 子窗挂接（拖动跟随）：读取 Peek/主面板实界 → platform
/// attach_child_window（AppKit addChildWindow 父子关联，拖动零抖跟随）。测试面
/// no-op：TestApp 无真 NSWindow 语义，FFI 定位必失败（与 screen_areas_seam
/// 同口径隔离）。
#[cfg(test)]
fn attach_peek_child(_cx: &mut App) {}

#[cfg(not(test))]
fn attach_peek_child(cx: &mut App) {
    let peek_rect = cx
        .global::<DockHandles>()
        .peek
        .and_then(|h| window_rect(cx, &h));
    let Some(child) = peek_rect else {
        return; // 陈旧句柄（open_or_update_peek 会重开）；无 Peek 实界不挂
    };
    let parent = panel_actual_bounds(cx);
    if parchora_platform::attach_child_window(parent, child) {
        eprintln!("[parchora] Peek 已挂接为面板子窗（拖动跟随）");
    } else {
        eprintln!("[parchora] Peek 子窗挂接失败 → 独立窗降级（拖动暂不跟随）");
    }
}

/// Peek 关闭防争代次 bump（作废未决 close timer——gen 检查兑，Task drop 语义无关）。
fn bump_peek_close_gen(cx: &mut App) {
    cx.global_mut::<DockHandles>().peek_close_gen += 1;
}

/// 状态栏点击排水守护轮询间隔（秒）：状态栏点击旗标最迟 ~0.05s 内响应。
const STATUS_POLL_SECS: f32 = 0.05;

/// 分辨率/缩放档位扫描守护轮询间隔（秒 · DEC-198）：跨屏迁移与档位变化最迟
/// ~0.5s 内生效（status watch 同型泵频）。
const SCALE_SWEEP_SECS: f32 = 0.5;

/// 状态栏点击 toggle（停靠形态）：面板在 → 关；无 → 开（展示语义
/// revealed=false——光标入板前不收，防点击位（菜单栏）与触发热区远隔导致的
/// 呼出即收闪烁；隐藏路径 = 再次点击/失活期移出）。演示窗形态：仅 stderr 登记。
fn status_click_toggle(cx: &mut App) {
    if !APP_MODE_DOCKED.load(Ordering::Relaxed) {
        eprintln!("[parchora] 状态栏点击：演示窗形态，无停靠面板可 toggle");
        return;
    }
    // 状态栏点击即全局左键按下（DEC-94）：本点击的 tap 旗标当拍由
    // click_outside_tick 消费——开分支防「开面板即被残余点击旗标误收」，关分支
    // 防双闭竞态；消费即清。
    cx.global_mut::<DockHandles>().skip_click = true;
    if cx.global::<DockHandles>().panel.is_some() {
        close_panel(cx);
    } else {
        open_panel_docked(cx);
    }
}

/// 状态栏点击排水守护（全形态）：0.05s 轮询排水位旗标 → toggle;
/// 与 reveal/snap/settings 守护互不依赖；排水在 gpui 前台续体执行（reveal watch
/// 同模式），无租借上下文可直呼 toggle。
fn start_status_click_watch(cx: &App) {
    // 自证旗标（仅各落一行 · 真机判读）：#1 装配即刻读数 + #2 穿越首个
    // autorelease 池排空（60 拍 ≈ 3s）持续读数；retain 保活成立则两拍均在册。
    static SELFTEST_TICK: AtomicUsize = AtomicUsize::new(0);
    cx.spawn(async move |cx| loop {
        cx.background_executor()
            .timer(Duration::from_secs_f32(STATUS_POLL_SECS))
            .await;
        let tick = SELFTEST_TICK.fetch_add(1, Ordering::Relaxed) + 1;
        if tick == 1 {
            match parchora_platform::status_item_window_size() {
                Some((w, h)) => eprintln!(
                    "[parchora] 状态栏 item 窗自证#1 alive w={w:.1} h={h:.1}（retain 保活）"),
                None => eprintln!(
                    "[parchora] 状态栏 item 窗自证#1 未检出（item 指针空或窗未 materialize）"),
            }
        }
        if tick == 61 {
            match parchora_platform::status_item_window_size() {
                Some((w, h)) => eprintln!(
                    "[parchora] 状态栏 item 窗自证#2 alive w={w:.1} h={h:.1}（跨 autorelease 排空 · 常驻确认）"),
                None => eprintln!(
                    "[parchora] 状态栏 item 窗自证#2 未检出（item 已被释放或窗未挂）"),
            }
        }
        // 点击外隐藏排水（DEC-94 · REQ-PANEL-HIDE「点击面板外部隐藏」路径）：
        // tap 左键按下旗标与状态栏旗标由同一守护序次排水——toggle 先行置
        // skip_click 门，当拍点击旗标消费，杜绝双观察者竞态（开面板即收/误重开）。
        // 右键退出旗标同守护排水：命中即走清理链并短路。
        let status_quit = parchora_platform::status_quit_take();
        let status_toggled = parchora_platform::status_click_take();
        let clicked = parchora_platform::tap_click_take();
        cx.update(move |cx| {
            if status_quit {
                quit_app(cx);
                return;
            }
            if status_toggled {
                status_click_toggle(cx);
            }
            click_outside_tick(cx, clicked, current_cursor());
        });
    })
    .detach();
}

/// 受控五窗档位记账（DEC-198 sweep 专用）：各窗上次所在屏档位；None = 窗关
/// /未记（重开首轮即视为变化，重刷一拍无害）。
#[derive(Default)]
struct ScaleWatch {
    panel: Option<s::DispTier>,
    peek: Option<s::DispTier>,
    settings: Option<s::DispTier>,
    editor: Option<s::DispTier>,
    task_create: Option<s::DispTier>,
}

impl gpui::Global for ScaleWatch {}

/// 窗所在屏档位（DEC-198）：所在屏逻辑宽 → DispTier；无屏（首帧未上屏）→ None。
fn window_tier(window: &gpui::Window, cx: &App) -> Option<s::DispTier> {
    window
        .display(cx)
        .map(|d| s::DispTier::for_width(f32::from(d.bounds().size.width)))
}

/// 受控窗单槽扫描（DEC-198）：读窗所在屏档位，与上次记账不同（跨屏迁移/
/// 分辨率·缩放变化）→ 整窗刷新（下一帧渲染经 TLS 取新档表值）；返回本拍
/// 档位（窗已关 → update Err → None，记账清位）。
fn sweep_tier_slot<T: 'static>(
    cx: &mut App,
    h: Option<gpui::WindowHandle<T>>,
    last: Option<s::DispTier>,
) -> Option<s::DispTier> {
    let h = h?; // 窗未在场（None）→ 无档位；clippy question_mark 建议（无 else 桩）
    h.update(cx, |_, window, cx| {
        let tier = window_tier(window, cx);
        if tier != last {
            window.refresh();
        }
        tier
    })
    .ok()
    .flatten()
}

/// 档位扫描守护（DEC-198 · 启动全量建档后泵运行 · status watch 同型）：
/// ① 逐拍全显示器档位剖面建档/更新（ScreenProfiles · 分辨率与系统缩放变化侦测）；
/// ② 受控五窗逐窗所在屏档位比对记账，档变 → window.refresh() 整窗重建。
/// （Peek/设置/编辑/任务窗窗内 TLS enter 以 profile_for_display 取档，无需
/// 单独接线；面板内嵌子视图经根视图作用域继承同档。）
fn start_scale_sweep(cx: &App) {
    cx.spawn(async move |cx| loop {
        cx.background_executor()
            .timer(Duration::from_secs_f32(SCALE_SWEEP_SECS))
            .await;
        cx.update(|cx| {
            parchora_ui::scale::apply_all_displays(cx);
            let dock = cx.global::<DockHandles>();
            let (panel, peek, settings, editor, task_create) = (
                dock.panel,
                dock.peek,
                dock.settings,
                dock.editor,
                dock.task_create,
            );
            let (last_panel, last_peek, last_settings, last_editor, last_task) = {
                let w = cx.global::<ScaleWatch>();
                (w.panel, w.peek, w.settings, w.editor, w.task_create)
            };
            let new_panel = sweep_tier_slot(cx, panel, last_panel);
            let new_peek = sweep_tier_slot(cx, peek, last_peek);
            let new_settings = sweep_tier_slot(cx, settings, last_settings);
            let new_editor = sweep_tier_slot(cx, editor, last_editor);
            let new_task = sweep_tier_slot(cx, task_create, last_task);
            // 同值守卫（DEC-217 · P-20261023-17）：五槽全无档位变化时跳过
            // global_mut 写回——global_mut 调用即无条件 push effect＋stamp 依赖
            // 记账（DEC-203 F1 同因），2Hz 稳态同值 stamp 空转。
            if (new_panel, new_peek, new_settings, new_editor, new_task)
                != (last_panel, last_peek, last_settings, last_editor, last_task)
            {
                let w = cx.global_mut::<ScaleWatch>();
                w.panel = new_panel;
                w.peek = new_peek;
                w.settings = new_settings;
                w.editor = new_editor;
                w.task_create = new_task;
            }
        });
    })
    .detach();
}

/// 指针离开 Peek 窗 → 0.12s 对称宽限后关 Peek（gen 未变才落关）。单击范式：
/// Peek 独立关闭后，仅当指针不在「热区 ∪ 面板」域内才收面板——光标回
/// 板续点条目/浏览时面板保持（REQ-PANEL-EDGE-TRIGGER 展示态同源 · 与
/// MouseExitHide 同原语同语义）。
/// 测试构建 bypass 保护分支：cursor_position 为真机 CG 探针，测试中读取物理光标
/// 会污染窗口虚拟坐标判定——测试宽限届满即收面板（不做真光标复核），回归确定
/// 性不降。Task 持在窗柄表（替换旧值；旧 timer 由 gen 检查失效，不依赖 drop 取消）。
fn arm_peek_close(cx: &mut App) {
    let generation = cx.global::<DockHandles>().peek_close_gen;
    let timer = cx.spawn(async move |cx| {
        cx.background_executor()
            .timer(Duration::from_secs_f32(tk::PEEK_GRACE))
            .await;
        cx.update(|cx| {
            if cx.global::<DockHandles>().peek_close_gen == generation {
                // 滚轮冻结伪退出守门（DEC-109 · D-01）：vendor hover_freeze 于滚
                // 轮事件后 100ms 内（指针未动）合成「无命中」hit-test → Peek 根
                // div 合成 on_hover(false) → 本路径被误触发（快滚连续冻结跨
                // 0.12s 宽限 → 误级联收窗收面板 · DEC-109）。显隐只认「鼠标真离开」
                // （DEC-107）：届满时真光标仍居 Peek 实界内 = 阅读/滚轮会话在续 →
                // 重臂续等（下届以真光标再验；真移出照常重 arm、回窗 gen bump 均可
                // 作废未决 close）。测试态读真机物理光标不可判读——直收
                // （MouseExitHide cfg 旁路同款先例）。
                #[cfg(test)]
                let synthetic_exit = false;
                #[cfg(not(test))]
                let synthetic_exit = cx
                    .global::<DockHandles>()
                    .peek
                    .map(|h| peek_close_gate(cursor_position(), window_rect(cx, &h)))
                    .unwrap_or(false);
                if synthetic_exit {
                    arm_peek_close(cx);
                    return;
                }
                close_peek(cx, false); // 悬停宽限软关：保存失败驻留重试（DEC-159）
                #[cfg(test)]
                let in_dock_domain = false;
                #[cfg(not(test))]
                let in_dock_domain = {
                    let (mx, my) =
                        cursor_position().unwrap_or((f32::NEG_INFINITY, f32::NEG_INFINITY));
                    let (x, y, w, h) = panel_actual_bounds(cx);
                    let in_rect = mx >= x && mx < x + w && my >= y && my < y + h;
                    // 缝侧邻接宽限带同样属保持域（DEC-95 · 纯判定不计数：事件路径
                    // 不占宽限时间轴，长驻超限由守护轮询路承载）。
                    let trigger = trigger_pos(cx);
                    let zone = dock::trigger_zone_bounds(trigger, screen_rect(cx));
                    in_rect
                        || dock::point_in_zone(mx, my, zone)
                        || dock::in_seam_grace_band(trigger, mx, my, zone)
                };
                // Pin 压倒一切自动隐藏路径（SCN-PANEL-PIN-BLOCKS-HIDE · DEC-94）。
                if !cx.global::<DockHandles>().pinned && !in_dock_domain {
                    // 延时收起（DEC-184）：Peek 悬停离场连带面板隐藏同属用户面
                    // 自动链——Peek 本体软关照旧立即，面板 1s 复核。
                    arm_panel_close(cx);
                }
            }
        });
    });
    cx.global_mut::<DockHandles>().peek_close_task = Some(timer);
}

/// 滚轮冻结伪退出守门纯函数（DEC-109 · D-01）：光标可读且居 Peek 实界内（含边
/// 闭合区间，panel_contains 同式）→ 判滚轮会话在续（不落关）。光标不可读 / 窗
/// 界不可读 → false 照旧落关（保守口径与面板保持域 None 语义同向）。
fn peek_close_gate(cursor: Option<(f32, f32)>, rect: Option<(f32, f32, f32, f32)>) -> bool {
    match (cursor, rect) {
        (Some((x, y)), Some((rx, ry, rw, rh))) => {
            x >= rx && x <= rx + rw && y >= ry && y <= ry + rh
        }
        _ => false,
    }
}

/// 唤出守护（DEC-79）：0.05s 轮询光标位置（panel 在场时读探针域外
/// 跳过——CGEvent 只读查询 ~μs 级，常驻轮询成本可忽略）；面板不在场时按
/// dock::trigger_zone_bounds（当前触发位置热区）判定。真机选型：非 key 非激活
/// NSPanel 收不到 mouseMoved（vendor events.rs 不映射 NSMouseEntered / PopUp 分支
/// 无 setAcceptsMouseMovedEvents），窗口级 hover 无法承载收缩态唤出；CGEventTap
/// 真语义光标源（REQ-PANEL-EDGE-TRIGGER · DEC-81/82）：tap 主源 + 槽新鲜度
/// poll 降级；CGEvent 位置读探针免权限免注入，为未授权/未就绪期的 poll 承载
/// （DEC-79 · 真机实证）。
fn start_reveal_watch(cx: &App) {
    // tap 初装配（授权已给 → 真语义光标源；未授权 → poll 承载 + 引导卡 · DEC-81）。
    maybe_start_mouse_tap();
    cx.spawn(async move |cx| loop {
        cx.background_executor()
            .timer(Duration::from_secs_f32(REVEAL_POLL_SECS))
            .await;
        // 菜单模态期边缘检测暂停（SCN-SETTINGS-MENU-PAUSES-EDGE · DEC-121）：模
        // 态 latch 单次损耗可接受（带内入带判定挂起一拍）。
        if parchora_platform::status_menu_open() {
            continue;
        }
        cx.update(reveal_watch_tick);
    })
    .detach();
}

/// 唤出守护单拍（生产包装）：tap 装配惰性重探 + 授权态复探回写 → 光标源选择
/// （tap 采样槽优先）→ 交注入式单拍。光标源统一 `current_cursor()`（DEC-83：
/// 内缘照常唤出；防跨屏误触由唤出确认节流与抑制门承载）。
/// 无显示器时 cursor 仍可读取（屏几何 fallback 与 panel 开窗同源），
/// 光标读失败按 None（出带）处置。
fn reveal_watch_tick(cx: &mut App) {
    // tap 装配惰性重探（SCN-PANEL-LISTEN-UNAUTHORIZED 恢复场景：系统设置授权后
    // 无需重启即起真语义光标源；preflight 只读 ~us 级可忽略）。
    maybe_start_mouse_tap();
    // 授权态复探回写（全局不一致时才写：引导卡消隐 + 在开设置窗即时刷渲染）。
    sync_listen_auth(cx);
    // Dock/菜单栏布局运行时检测（DEC-84 · REQ-PANEL-TRIGGER-POSITION 冲突切换
    // 「系统修改 Dock 位置」场景）：屏帧缓存比对——帧变化 → 重解析触发位冲突
    // （变更才重泊重开）+ 刷新占用态全局。无冲突但几何变（如菜单栏伸缩）不主动
    // 重泊：吸附守护以 panel_bounds(screen_rect) 新基准自动重泊自愈。
    let frames = screen_frames(cx);
    if cx.global::<DockHandles>().last_frames != Some(frames) {
        cx.global_mut::<DockHandles>().last_frames = Some(frames);
        let (full, usable) = frames;
        refresh_dock_availability_with(cx, full, usable);
        let previous = trigger_pos(cx);
        let resolved = dock::resolve_conflict(previous, full, usable);
        if resolved != previous {
            apply_trigger_pos_with(cx, resolved, full, usable);
            return; // 重泊重开已按新帧落定，本拍不再做唤出判定
        }
    }
    let cursor = current_cursor();
    let trigger = trigger_pos(cx);
    let zone = dock::trigger_zone_bounds(trigger, screen_rect(cx));
    // 热区镜像播种 + 快速滑过锁存消费（DEC-100）：tap 回调线程逐事件比对镜像
    // 判「入排除带感知有效热区」→ 置 ZONE_HIT；本拍单次消费注入守护（面板在场
    // 分支消费即丢弃，仅面板不在场分支生效唤出）。镜像 None 起始拍（tap 未起/
    // 测试态）回调不判不置。
    if let Ok(mut mirror) = ZONE_MIRROR.lock() {
        *mirror = Some((zone, trigger));
    }
    let zone_hit = ZONE_HIT.swap(false, Ordering::Relaxed);
    reveal_watch_tick_with_hit(cx, cursor, zone, zone_hit);
}

/// 守护拍注入入口（测试别名）：zone_hit=false 直通——tap 快速滑过锁存注入走
/// reveal_watch_tick_with_hit（DEC-100 注入口；生产独占 ZONE_HIT 真锁存，测试
/// 态静态恒 false 无跨测竞态）。生产无调用方（test-only 存在防死码告警）。
#[cfg(test)]
fn reveal_watch_tick_with(cx: &mut App, cursor: Option<(f32, f32)>, zone: (f32, f32, f32, f32)) {
    reveal_watch_tick_with_hit(cx, cursor, zone, false);
}

/// 唤出守护单拍（zone_hit 注入变体 · DEC-100）：光标/热区注入（生产读探针/tap
/// 锁存；测试注入模拟值）。状态机
/// （reveal_suppressed 门 + reveal_streak 节流 + revealed 轮询收域 · DEC-79；
/// close_streak 轮询收起两拍确认 · DEC-83；「在带」均为排除带感知有效热区口径
/// in_effective_zone · DEC-81）：
/// - 面板在场（启动/换位/重泊面板 · revealed=false）→ 失活期移出仿真（DEC-81
///   · D-01）：光标入板 arm → 出板（带感知热区外 + 无 Peek）连续两拍确认
///   离场才收起（快速穿越不收板 · DEC-83）；出板落离界缓冲带挂起（DEC-100 与
///   缝带同径）；光标从未入板则保持呈现（防启动闪隐与重泊即收 · DEC-78 不
///   违背）；
/// - 面板在场（唤出态 · revealed=true）→ 轮询收起域（规格「光标离开触发位置
///   或主程序界面时隐藏」状态式承载 · CLOSE_CONFIRM_TICKS 两拍 0.1s 停留防抖
///   · DEC-83）：光标同离
///   面板与有效热区且无 Peek 连续两拍 → 收；离界缓冲带内挂起（外扩 20pt ·
///   0.6s 长驻 · DEC-100「不得刚离边界即瞬隐」）；任一保持样本清零；覆盖
///   exit 迁移不达的场景（sliver 防抖眼保住的面板后续离场）。Peek
///   行程/阅读态抑制同 MouseExitHide 口径；
/// - 面板不在场（显示侧零延时 · DEC-210：单拍直唤，唯一等待 = 守护拍距，
///   无延时门、无隐藏链挂起耦合）：光标在有效热区（排除带感知）&& armed →
///   滑过即唤
///   （REVEAL_CONFIRM_TICKS=1 单拍）+
///   抑制 + 标记唤出态；zone_hit 锁存（DEC-100）→ 轮询拍间已穿越带（高速滑过
///   采样漏检补口）与在带同径单拍唤出；光标在带 && 抑制中 → no-op（Esc 收起
///   后带内滞留不反弹；抑制期锁存同口径不反弹，落离带重臂）；光标离带 &&
///   抑制中 → 重臂；离带 && armed → 清零维持。内缘照常唤出（DEC-83）。
fn reveal_watch_tick_with_hit(
    cx: &mut App,
    cursor: Option<(f32, f32)>,
    zone: (f32, f32, f32, f32),
    zone_hit: bool,
) {
    let trigger = trigger_pos(cx);
    if let Some(handle) = cx.global::<DockHandles>().panel {
        if !cx.global::<DockHandles>().revealed {
            // 失活期面板移出仿真（DEC-81 · D-01）：展示语义面板（启动/换位/
            // 重泊）光标入板 arm → 出板（排除带感知 + 无 Peek）收起——与
            // MouseExitHide 语义同构（active 期双发幂等：close_panel 对 None
            // no-op）；无 vendor 元素事件以守护拍轮询仿真。
            probe_dormant_panel(cx, &handle, cursor, zone, trigger);
            return;
        }
        // 轮询收起域（仅唤出态面板）。Pin 屏蔽整条轮询隐藏路径（SCN-PANEL-PIN-
        // BLOCKS-HIDE「Pin 压倒一切」· DEC-94）：组件 MouseExitHide 已在 shell.rs
        // 挡，轮询守护路同口径补齐。
        if cx.global::<DockHandles>().pinned {
            return;
        }
        if cx.global::<DockHandles>().peek.is_some() {
            return; // Peek 阅读/行程态：面板保持（SCN-PREVIEW-SCROLL-IN-PREVIEW）
        }
        let Some((x, y)) = cursor else {
            return;
        };
        if in_effective_zone(x, y, zone, trigger) {
            // 同值守卫（DEC-217 · R184 建议 4）：稳态（三计数恒零）跳过
            // global_mut 写——守护拍调用即 push effect＋stamp 记账空转收口。
            let steady = {
                let h = cx.global::<DockHandles>();
                h.close_streak == 0 && h.seam_grace_ticks == 0 && h.exit_grace_ticks == 0
            };
            if !steady {
                let handles = cx.global_mut::<DockHandles>();
                handles.close_streak = 0;
                handles.seam_grace_ticks = 0; // 回域复位（DEC-95）
                handles.exit_grace_ticks = 0; // 回域复位（DEC-100）
            }
            cancel_panel_close(cx); // 延时收起取消（DEC-184 · 回域即撤；幂等自守卫）
            return; // 光标仍在触发位置（排除带感知）：保持显示
        }
        // 已确认离带观察（面板在场）：重臂（唤出后门不残留——离带观察是唯一
        // 重臂来源，Esc 后带内滞留仍不重臂因无离带观察）。
        cx.global_mut::<DockHandles>().reveal_suppressed = false;
        let in_panel = panel_contains(cx, &handle, x, y);
        if in_panel {
            // 同值守卫（DEC-217 · R184 建议 4）：稳态（三计数恒零）跳过写
            //（与上方回域守卫同口径）。
            let steady = {
                let h = cx.global::<DockHandles>();
                h.close_streak == 0 && h.seam_grace_ticks == 0 && h.exit_grace_ticks == 0
            };
            if !steady {
                let handles = cx.global_mut::<DockHandles>();
                handles.close_streak = 0;
                handles.seam_grace_ticks = 0; // 回域复位（DEC-95）
                handles.exit_grace_ticks = 0; // 回域复位（DEC-100）
            }
            cancel_panel_close(cx); // 延时收起取消（DEC-184 · 回板即撤；幂等自守卫）
            return; // 光标仍在面板内：保持显示
        }
        // 缝侧邻接宽限带（DEC-95）：越缝深入邻屏
        // ≤20pt → 挂起收起（宽限计数上行 + 收起确认清零），带内长驻 0.6s 超限
        // 后恢复常规两拍。
        if seam_grace_holds(cx, trigger, x, y, zone) {
            return;
        }
        // 面板离界缓冲带（DEC-100）：
        // 刚离面板边界落「实界外扩 20pt ∩ 当前屏」过渡带 → 挂起收起（不瞬隐、
        // 不闪烁），长驻 0.6s 超限后恢复常规两拍；触发缘向屏外方向已按屏缘裁
        // 剪——缝侧深域由上方缝带专责。
        if exit_grace_holds(cx, x, y) {
            return;
        }
        // 光标已同离热区与面板（规格隐藏条款；含 sliver 守住后的后续离场）→
        // 两拍确认防抖（DEC-83）：连续
        // CLOSE_CONFIRM_TICKS 拍同离面板与有效热区才收起（快速穿越不收板；
        // 收起域 0.05s 拍距 = 0.1s 停留确认）。
        let confirmed = {
            let handles = cx.global_mut::<DockHandles>();
            handles.close_streak += 1;
            if handles.close_streak >= CLOSE_CONFIRM_TICKS {
                handles.close_streak = 0;
                handles.revealed = false;
                true
            } else {
                false
            }
        };
        if confirmed {
            // 延时收起（DEC-184）：两拍确认成立后不再即收，1s 复核 timer 接手
            //（返回域内守护拍复位链仍可取消）。
            arm_panel_close(cx);
        }
        return;
    }
    let inside = cursor.is_some_and(|(x, y)| in_effective_zone(x, y, zone, trigger));
    if inside {
        if cx.global::<DockHandles>().reveal_suppressed {
            return; // 抑制中：带内滞留不反弹（Esc 收起场景）
        }
        cx.global_mut::<DockHandles>().reveal_streak += 1;
        if cx.global::<DockHandles>().reveal_streak >= REVEAL_CONFIRM_TICKS {
            {
                let handles = cx.global_mut::<DockHandles>();
                handles.reveal_suppressed = true;
                handles.reveal_streak = 0;
            }
            // 无租借上下文（守护拍非 listener 链）→ 开面板直呼。
            open_panel_docked(cx);
            cx.global_mut::<DockHandles>().revealed = true;
        }
        return;
    }
    // 快速滑过锁存唤出（DEC-100）：轮询采样两拍之间光标已穿越热区
    // （高速滑过 in 采样漏检）而 tap 事件已置 ZONE_HIT → 与「在带」同径单拍
    // 唤出（REVEAL_CONFIRM_TICKS=1 等价）；抑制门同口径——抑制期锁存不反弹，
    // 落此自然走下方离带重臂（锁存拍光标本就在带外 = 离带观察）。
    if zone_hit && !cx.global::<DockHandles>().reveal_suppressed {
        {
            let handles = cx.global_mut::<DockHandles>();
            handles.reveal_suppressed = true;
            handles.reveal_streak = 0;
        }
        // 无租借上下文（守护拍非 listener 链）→ 开面板直呼。
        open_panel_docked(cx);
        cx.global_mut::<DockHandles>().revealed = true;
        return;
    }
    // 离带：清零确认拍；抑制中则重臂；armed 维持。
    // 同值守卫（DEC-217 · P-20261023-17）：稳态（streak=0 && !suppressed）跳过
    // global_mut 写——调用即无条件 push effect＋stamp 依赖记账（DEC-203 F1 同
    // 因），20Hz 稳态同值 stamp 空转；抑制后首个离带拍 suppressed true→false
    // 属真值变化（重臂语义），守卫不拦。
    let off_band_steady = {
        let h = cx.global::<DockHandles>();
        h.reveal_streak == 0 && !h.reveal_suppressed
    };
    if !off_band_steady {
        let handles = cx.global_mut::<DockHandles>();
        handles.reveal_streak = 0;
        handles.reveal_suppressed = false;
    }
}

/// 排除带感知的有效热区判定（DEC-81 · 三处守卫同口径复用）：点在热区且不在
/// 排除带段。`dock::in_exclusion` 蕴含 point_in_zone，组合即「有效触发热区」。
fn in_effective_zone(x: f32, y: f32, zone: (f32, f32, f32, f32), trigger: TriggerPosition) -> bool {
    dock::point_in_zone(x, y, zone) && !dock::in_exclusion(trigger, x, y, zone)
}

/// 缝侧邻接宽限挂起判定（DEC-95 ·
/// 仅守护两条轮询路：唤出态轮询收起域 + 失活期出板探测）：光标越出触发缘共享
/// 缝深入邻接屏 ≤20pt（dock::in_seam_grace_band）→ 挂起常规收起——宽限计数
/// 上行、收起确认当拍清零；连续 SEAM_GRACE_TICKS（0.05s×12=0.6s）后返回 false
/// 恢复常规两拍（带内长驻再收）。带外 → 宽限计数清零 + false（远离即复位）。
/// 事件路径（MouseExitHide/arm_peek_close）不计数，仅 dock::in_seam_grace_band
/// 纯判定；点击外隐藏路径不含宽限（点击 = 显式动作 · REQ-PANEL-HIDE 不动）。
fn seam_grace_holds(
    cx: &mut App,
    trigger: TriggerPosition,
    x: f32,
    y: f32,
    zone: (f32, f32, f32, f32),
) -> bool {
    let handles = cx.global_mut::<DockHandles>();
    if !dock::in_seam_grace_band(trigger, x, y, zone) {
        handles.seam_grace_ticks = 0;
        return false;
    }
    if handles.seam_grace_ticks < SEAM_GRACE_TICKS {
        handles.seam_grace_ticks += 1;
        handles.close_streak = 0; // 挂起拍：收起确认不累计（DEC-95）
        return true;
    }
    false // 带内长驻超限：恢复常规两拍收起（宽限计数封顶不动，回域再清零）
}

/// 面板离界缓冲带判定（DEC-100 · 纯几何）：点是否落在「面板实界外扩
/// EXIT_GRACE_BAND ∩ 当前屏」过渡带矩形内（外扩含四方向；触发缘向屏外方向以
/// 屏缘裁剪——屏外深域归缝侧宽限带专责 · DEC-95 两带域不相交、计数互不冻结）。
/// 实界不可读按 panel_bounds 推定（panel_actual_bounds 同源兑底）；面板与屏不
/// 相交（异常态）时判域退化为空 → 永不挂起（回退常规两拍，防御式）。
fn panel_exit_band_contains(cx: &mut App, x: f32, y: f32) -> bool {
    let (bx, by, bw, bh) = panel_actual_bounds(cx);
    let (sx, sy, sw, sh) = screen_rect(cx);
    let b = EXIT_GRACE_BAND;
    let x0 = (bx - b).max(sx);
    let y0 = (by - b).max(sy);
    let x1 = (bx + bw + b).min(sx + sw);
    let y1 = (by + bh + b).min(sy + sh);
    x >= x0 && x <= x1 && y >= y0 && y <= y1
}

/// 离界缓冲挂起判定（DEC-100 · 与 seam_grace_holds 同构）：光标落
/// 过渡带内 → 缓冲计数上行 + 收起确认当拍清零；连续 EXIT_GRACE_TICKS
/// （0.05s×12=0.6s）后返回 false 恢复常规两拍收起；带外计数清零（远离即复位，
/// 回面板/回热区复位在保持样本分支成对处理）。事件路径不计数
/// （MouseExitHide keep_open 侧仅 panel_exit_band_contains 纯保持判定）。
fn exit_grace_holds(cx: &mut App, x: f32, y: f32) -> bool {
    let in_band = panel_exit_band_contains(cx, x, y);
    let handles = cx.global_mut::<DockHandles>();
    if !in_band {
        handles.exit_grace_ticks = 0;
        return false;
    }
    if handles.exit_grace_ticks < EXIT_GRACE_TICKS {
        handles.exit_grace_ticks += 1;
        handles.close_streak = 0; // 挂起拍：收起确认不累计（DEC-100）
        return true;
    }
    false // 带内长驻超限：恢复常规两拍收起（回域再清零）
}

/// 守护光标源（DEC-81）：tap 活性 → 读 tap 采样槽（空闲期存最后位置，鼠标静止
/// 无新事件时语义等价）；未授权/poll 承载 → cursor_position 只读探针（DEC-79
/// 降级保留 · 偏差注记登记）。
/// tap 采样槽新鲜度判定（纯函数 · DEC-82 单测锚点）：槽时刻缺失或超龄
/// TAP_FRESH_SECS 即视为过期。
fn tap_slot_fresh(at: Option<Instant>, now: Instant) -> bool {
    at.is_some_and(|t| now.duration_since(t).as_secs_f32() <= TAP_FRESH_SECS)
}

/// 从 tap 槽取光标（新鲜度校验通过才返回 · DEC-82）：鼠标移动期槽被 tap 事件
/// 持续刷新恒新鲜；静止期槽超龄返回 None（此时 poll 读数与槽语义一致，零风险）。
fn cursor_from_tap_slot() -> Option<(f32, f32)> {
    let at = TAP_AT.lock().ok().and_then(|guard| *guard);
    if tap_slot_fresh(at, Instant::now()) {
        TAP_POS.lock().ok().and_then(|slot| *slot)
    } else {
        None
    }
}

/// 当前光标读数（DEC-81/82 选型：tap 已激活时新鲜槽值优先，槽缺失/超龄回退
/// cursor_position poll 读探针——偶发静默与静止期槽超龄均属正常回落）。
fn current_cursor() -> Option<(f32, f32)> {
    if TAP_ACTIVE.load(Ordering::Relaxed) {
        if let Some(pos) = cursor_from_tap_slot() {
            return Some(pos);
        }
        // 槽缺失/过期 → poll 源回退（tap 偶发静默兜底 · DEC-82）；静止期槽超龄
        // 亦走此路（与 poll 读数一致非异常），日志闩仅首次记录供真机判读。
        if !TAP_STALE_LOGGED.swap(true, Ordering::Relaxed) {
            eprintln!("[parchora] tap 槽过期 → poll 源回退（偶发静默兜底生效 · 含静止期正常回落）");
        }
    }
    cursor_position()
}

/// 授权态复探回写（DEC-81 · SCN-PANEL-LISTEN-UNAUTHORIZED 恢复口径）：preflight
/// 与全局不一致（用户在系统设置授权）→ 回写 ListenAuth + 在开设置窗即时刷渲染
/// （引导卡消隐）。变化才写，守护拍 0.05s 频次下成本可忽略。
fn sync_listen_auth(cx: &mut App) {
    let granted = listen_access_granted();
    let changed = cx
        .try_global::<ListenAuth>()
        .map(|state| state.listen_granted != granted)
        .unwrap_or(true);
    if !changed {
        return;
    }
    cx.set_global(ListenAuth {
        listen_granted: granted,
    });
    if granted {
        // 授权态翻转恢复（未授权→已授权）：清创建失败闩，下拍重试 tap 装配。
        TAP_CREATE_LATCHED.store(false, Ordering::Relaxed);
    }
    if let Some(handle) = cx.global::<DockHandles>().settings {
        let _ = handle.update(cx, |_, _, cx| cx.notify());
    }
}

/// tap 装配（幂等 · DEC-81）：授权 preflight 通过 → spawn 只听 tap（写 TAP_POS
/// 槽；种子取读探针首拍防首个事件前空槽）→ 置活性标志 + 句柄入单例仓（不存
/// 则句柄即 Drop = tap 即停）。未授权静默保持 poll 承载（
/// REQ-PANEL-LISTEN-GRACEFUL 降级语义 + 设置引导卡路径 · 偏差注记登记）。启动
/// 与守护拍均可调（SCN 授权恢复场景）。
fn maybe_start_mouse_tap() {
    if TAP_ACTIVE.load(Ordering::Relaxed)
        || TAP_CREATE_LATCHED.load(Ordering::Relaxed)
        || !listen_access_granted()
    {
        return;
    }
    // 种子取读探针首拍：读失败写 None 槽（不盖伪读数 (0,0)——伪新鲜值可致左缘
    // 热区误唤出；None 槽经 cursor_from_tap_slot 返 None，自然走 poll 降级）。
    let seed = cursor_position();
    if let Ok(mut slot) = TAP_POS.lock() {
        *slot = seed;
    }
    if let Ok(mut at) = TAP_AT.lock() {
        *at = Some(Instant::now());
    }
    match spawn_mouse_tap(Box::new(|x, y| {
        if let Ok(mut slot) = TAP_POS.lock() {
            *slot = Some((x, y));
        }
        if let Ok(mut at) = TAP_AT.lock() {
            *at = Some(Instant::now());
        }
        // 快速滑过锁存（DEC-100）：回调线程逐事件比对热区镜像（排除带
        // 感知有效域口径，与守护轮询 in_effective_zone 同源）——入带任一事件即
        // 置位，守护拍单次消费；镜像 None（tap 未起首拍前/测试态）不判不置。
        // 轮询 0.05s 拍距下高速滑过会在两拍之间穿带（in 采样漏检），事件驱动
        // 判定零漏检。
        if let Ok(mirror) = ZONE_MIRROR.lock() {
            if let Some((zone, trigger)) = *mirror {
                if dock::point_in_zone(x, y, zone) && !dock::in_exclusion(trigger, x, y, zone) {
                    ZONE_HIT.store(true, Ordering::Relaxed);
                }
            }
        }
    })) {
        Some(handle) => {
            TAP_ACTIVE.store(true, Ordering::Relaxed);
            if let Ok(mut registry) = TAP_HANDLE.lock() {
                *registry = Some(handle);
            }
            eprintln!("[parchora] CGEventTap 只听监听已启动（真语义光标源 · DEC-81）");
        }
        None => {
            // 局限登记：授权恒 granted 而 spawn 恒失败时，闩仅由授权态翻转 /
            // 引导卡点击两路清除——tap 不再重试、恒 poll 承载（poll 承载语义完整，
            // 仅少一层真语义源）。
            TAP_CREATE_LATCHED.store(true, Ordering::Relaxed);
            eprintln!("[parchora] CGEventTap 创建失败 → 沿用读探针 poll 承载（DEC-79；授权翻转/引导卡点击后重试）")
        }
    }
}

/// 输入监控授权请求（引导卡点击路径 · DEC-81 · REQ-PANEL-LISTEN-GRACEFUL）：
/// 后台线程执行 CGRequestListenEventAccess（可能弹 TCC 弹窗并等待响应，不占主
/// 线程；启动禁止自动调用）；返回后回写 ListenAuth + tap 惰性装配（授权到位
/// 即起真语义光标源）。仅触全局与 FFI，无同实体嵌套（无 listener 链租借风险
/// · DEC-77）。
fn handle_listen_auth_request(cx: &mut App) {
    cx.spawn(async move |cx| {
        let granted = cx
            .background_executor()
            .spawn(async { request_listen_access() })
            .await;
        cx.update(move |cx| {
            cx.set_global(ListenAuth {
                listen_granted: granted,
            });
            // 显式点击路径：清失败闩后重试装配（成功则起真语义光标源）。
            TAP_CREATE_LATCHED.store(false, Ordering::Relaxed);
            maybe_start_mouse_tap();
            eprintln!("[parchora] 输入监控授权请求返回：granted={granted}");
        });
    })
    .detach();
}

/// 失活期面板探测（DEC-81 · D-01；收起确认 DEC-83）：None 光标不动；Peek
/// 在悬不动（行程/阅读态同 MouseExitHide 口径）；光标入板 → arm（清收起确认
/// 计数）；armed 且出板且不在有效热区（排除带感知 · 防收-唤抖眼）→ 连续两拍
/// 确认离场才收起（快速穿越不收板，与唤出确认对称）；启动/换位/重泊面板在
/// 光标从未入板时保持呈现（防闪隐）。
fn probe_dormant_panel(
    cx: &mut App,
    handle: &WindowHandle<PanelView>,
    cursor: Option<(f32, f32)>,
    zone: (f32, f32, f32, f32),
    trigger: TriggerPosition,
) {
    if cx.global::<DockHandles>().pinned {
        return; // Pin 态永不自动收起（SCN-PANEL-PIN-BLOCKS-HIDE · DEC-94）
    }
    let Some((x, y)) = cursor else {
        return;
    };
    if cx.global::<DockHandles>().peek.is_some() {
        return;
    }
    let in_panel = panel_contains(cx, handle, x, y);
    if in_panel {
        // 同值守卫（DEC-217 · R184 建议 4）：稳态（已臂且三计数恒零）跳过写。
        let steady = {
            let h = cx.global::<DockHandles>();
            h.panel_probe_armed
                && h.close_streak == 0
                && h.seam_grace_ticks == 0
                && h.exit_grace_ticks == 0
        };
        if !steady {
            let handles = cx.global_mut::<DockHandles>();
            handles.panel_probe_armed = true;
            handles.close_streak = 0;
            handles.seam_grace_ticks = 0; // 回域复位（DEC-95）
            handles.exit_grace_ticks = 0; // 回域复位（DEC-100）
        }
        cancel_panel_close(cx); // 延时收起取消（DEC-184 · 失活期回板即撤；幂等自守卫）
        return;
    }
    if !cx.global::<DockHandles>().panel_probe_armed {
        return;
    }
    if in_effective_zone(x, y, zone, trigger) {
        // 同值守卫（DEC-217 · R184 建议 4）：稳态（三计数恒零）跳过写。
        let steady = {
            let h = cx.global::<DockHandles>();
            h.close_streak == 0 && h.seam_grace_ticks == 0 && h.exit_grace_ticks == 0
        };
        if !steady {
            let handles = cx.global_mut::<DockHandles>();
            handles.close_streak = 0;
            handles.seam_grace_ticks = 0; // 回域复位（DEC-95）
            handles.exit_grace_ticks = 0; // 回域复位（DEC-100）
        }
        cancel_panel_close(cx); // 延时收起取消（DEC-184 · 失活期回域即撤；幂等自守卫）
        return;
    }
    // 缝侧邻接宽限带（DEC-95）：失活期出板落缝带（越缝走位）同样挂起长驻 0.6s
    // 超限后恢复常规两拍——与唤出态轮询路径同口径。
    if seam_grace_holds(cx, trigger, x, y, zone) {
        return;
    }
    // 面板离界缓冲带（DEC-100）：失活期出板落过渡带同样挂起（不瞬隐）
    // ——与唤出态轮询路径同口径。
    if exit_grace_holds(cx, x, y) {
        return;
    }
    // 出板+出带观察 → 两拍确认防抖（DEC-83）：连续 CLOSE_CONFIRM_TICKS 拍同
    // 离面板与有效热区才收起（快速穿越
    // 不收板，与唤出域收口对称）；任一保持样本清零。
    let confirmed = {
        let handles = cx.global_mut::<DockHandles>();
        handles.close_streak += 1;
        if handles.close_streak >= CLOSE_CONFIRM_TICKS {
            handles.close_streak = 0;
            handles.panel_probe_armed = false;
            true
        } else {
            false
        }
    };
    if confirmed {
        // 延时收起（DEC-184）：失活期两拍确认后 1s 复核 timer 接手。
        arm_panel_close(cx);
    }
}

/// 显示 id → 该屏全帧全局原点 (gx, gy)（ScreenArea.id 精确身份匹配 · Quartz
/// top-left）；seam 表（test 面恒空/无显示器）或未命中 → None。仅供 window_rect
/// 坐标缝消费（DEC-99）。
fn screen_origin_for_display(display_id: u64) -> Option<(f32, f32)> {
    screen_areas_seam()
        .into_iter()
        .find(|a| u64::from(a.id) == display_id)
        .map(|a| (a.full.0, a.full.1))
}

/// 窗全局实界读取（DEC-99 坐标缝 · top-left 原点 (x, y, w, h)）：
/// vendored gpui `window.bounds()` 在非主屏返回「所在屏局部坐标」（x 已减本屏
/// NS 原点、y 以本屏高翻转后再加本屏 NS 原点——代数恒等 global − 本屏全局原点；
/// 主屏 origin=(0,0) 两轴巧合等于全局）。故经 `window.display` 的 DisplayId
/// （=CGDirectDisplayID · vendor `From<DisplayId> for u64`）到 platform 屏枚举
/// （NSScreenNumber 身份匹配）查回本屏全局原点平移还原——与 cursor_position/
/// screen_areas/触发热区全局口径（Quartz top-left）统一，跨屏（DEC-76 多屏参照）
/// 域判定/邻接几何/子窗挂接/保持域全部消费此缝。屏身份未命中（test 面 seam 空表
/// /无显示器/探针哨兵 0）→ 原样返回局部读数（主屏无损）。
/// 句柄失效 → None（陈旧窗不入域）。
fn window_rect<V: 'static>(cx: &mut App, handle: &WindowHandle<V>) -> Option<(f32, f32, f32, f32)> {
    let mut local = None;
    handle
        .update(cx, |_, window, cx| {
            let b = window.bounds();
            local = Some((
                (
                    f32::from(b.origin.x),
                    f32::from(b.origin.y),
                    f32::from(b.size.width),
                    f32::from(b.size.height),
                ),
                window.display(cx).map(|d| u64::from(d.id())),
            ));
        })
        .ok()?;
    let ((lx, ly, w, h), did) = local?;
    let Some((ox, oy)) = did.and_then(screen_origin_for_display) else {
        return Some((lx, ly, w, h));
    };
    Some((lx + ox, ly + oy, w, h))
}

/// 点击外隐藏保持域判定（DEC-94 · REQ-PANEL-HIDE「点击面板外部隐藏」保持侧）：
/// 点在「面板 ∪ 详情窗 ∪ 两者邻接间隙条 ∪ 触发热区 ∪ 设置窗」内 = 应用 GUI 域
/// （交互约束「鼠标在任一 GUI 界面内的操作不得触发隐藏」同源口径）；
/// 域外点击 = 隐藏对象。面板实界不可读时按 panel_bounds 推定兑底
/// （与 panel_actual_bounds 同源）；热区取排除带感知有效口径。
fn keep_domain_contains(cx: &mut App, x: f32, y: f32) -> bool {
    let in_rect = |r: (f32, f32, f32, f32)| {
        let (rx, ry, rw, rh) = r;
        x >= rx && x <= rx + rw && y >= ry && y <= ry + rh
    };
    // 主面板（实界推定兑底）。
    if in_rect(panel_actual_bounds(cx)) {
        return true;
    }
    // 详情窗 + 面板↔详情窗邻接间隙条（防跨界点击误收 · DEC-94）。
    let peek_handle = cx.global::<DockHandles>().peek;
    let peek_rect = peek_handle.and_then(|h| window_rect(cx, &h));
    if let Some(pr) = peek_rect {
        if in_rect(pr) {
            return true;
        }
        if let Some(strip) = dock::adjacent_gap_strip(trigger_pos(cx), panel_actual_bounds(cx), pr)
        {
            if in_rect(strip) {
                return true;
            }
        }
    }
    // 触发热区（排除带感知有效域）。
    let trigger = trigger_pos(cx);
    if in_effective_zone(
        x,
        y,
        dock::trigger_zone_bounds(trigger, screen_rect(cx)),
        trigger,
    ) {
        return true;
    }
    // 设置窗（在开时）。
    let settings_handle = cx.global::<DockHandles>().settings;
    if let Some(sr) = settings_handle.and_then(|h| window_rect(cx, &h)) {
        if in_rect(sr) {
            return true;
        }
    }
    // 编辑转出窗（在开时）：handoff 期间窗内点击不误收面板
    //（「任一 GUI 界面内的操作不得触发隐藏」同域口径）。
    let editor_handle = cx.global::<DockHandles>().editor;
    if let Some(er) = editor_handle.and_then(|h| window_rect(cx, &h)) {
        if in_rect(er) {
            return true;
        }
    }
    false
}

/// 点击外隐藏单拍（DEC-94 · REQ-PANEL-HIDE「点击面板外部隐藏」· EdgeMark
/// hideOnClickOutside 同源）：仅处理有证据点击（tap 左键按下旗标排水）；面板不
/// 在场/光标不可读（保守不动）/点击在保持域内（GUI 内交互约束）/Pin 态
/// （SCN-PANEL-PIN-BLOCKS-HIDE 压倒一切）均不动；命中即 close_panel（级联关
/// 详情窗）。状态栏 toggle 的本点击旗标由 skip_click 门当拍消费。
fn click_outside_tick(cx: &mut App, clicked: bool, cursor: Option<(f32, f32)>) {
    if !clicked {
        return;
    }
    if cx.global::<DockHandles>().skip_click {
        cx.global_mut::<DockHandles>().skip_click = false;
        return; // 状态栏 toggle 本点击旗标消费（防同拍点击外误动作 · DEC-94）
    }
    if cx.global::<DockHandles>().panel.is_none() {
        return; // 面板不在场：无隐藏对象
    }
    let Some((x, y)) = cursor else {
        return; // 光标不可读：保守不动作（误收面板代价高于漏收一拍）
    };
    if keep_domain_contains(cx, x, y) {
        return; // 应用 GUI 域内点击：保持（GUI 内操作不触发隐藏）
    }
    if cx.global::<DockHandles>().pinned {
        return; // Pin 压倒一切隐藏路径（SCN-PANEL-PIN-BLOCKS-HIDE · DEC-94）
    }
    // 延时收起（DEC-184）：点击面板外属用户面隐藏操作——1s 复核 timer 接手。
    arm_panel_close(cx);
}

/// 主窗吸附守护（DEC-77「拖动到其它窗口区域时…自动吸附到对应
/// 屏幕边缘；任何情况下均不允许主窗口脱离屏幕边缘显示」）+ 释放门
/// （DEC-78：拖曳中不限位、不预览吸附，释放后一次性吸附）：0.25s 轮询——实界与上拍一致（连续两拍读数相同 ≈ 拖动已
/// 结束的释放近似）且偏离停泊基准超过 SNAP_EPS（2px）= 释放后未贴缘，则
/// 关 Peek 并贴边重泊（vendor 无编程式移窗 API，关窗重开为既定模式 · DEC-67）；
/// 拖动进行中（与上拍不一致）仅记录不动。gen bump 使未决 peek-close timer 失效
/// （防误关重泊后新面板）。基准 = 首轮 tick 播种的后钳制实界（macOS 菜单栏
/// 下缘钳 y=0→34、四角伴 1px 回缩——E2E 实测在档，若以 panel_bounds 纯函数推
/// 定位为期望会致顶族永久重泊循环）；重泊/重开经 open_panel_docked 重置待
/// 释放近似为窗口位置采样推断，非鼠标键状态：慢拖长停顿存在单拍误判
/// 可能（固有局限——键态收口已落地为 tap 左键释放锁存前哨
/// · start_drag_release_watch DEC-210；本守护保留作未授权降级与兖底双保
/// 险）；多显示器副屏真原点
/// defer（DEC-76 口径不变，单屏机完全正确）。
fn start_snap_watch(cx: &App) {
    cx.spawn(async move |cx| loop {
        cx.background_executor()
            .timer(Duration::from_secs_f32(SNAP_POLL_SECS))
            .await;
        cx.update(snap_watch_tick);
    })
    .detach();
}

/// 单拍吸附检查（生产包装）：读主面板实界（全局坐标 · DEC-99 缝——跨屏拖放后
/// 实界与停泊基准同全局口径判定漂移，避免同尺寸/同位局部坐标巧合漏判重泊）后交
/// `snap_watch_tick_with`；面板未开（演示模式/面板外）为 no-op（无窗无拖动可言）。
/// 实界读取失败（句柄失效）同样 no-op。
fn snap_watch_tick(cx: &mut App) {
    let actual = match cx.global::<DockHandles>().panel {
        None => None,
        Some(handle) => window_rect(cx, &handle),
    };
    snap_watch_tick_with(cx, actual);
}

/// 单拍吸附检查：实界注入（生产传 Some(实界)；测试注入模拟位）。释放门语义
///（DEC-78）：
/// - 上拍基线未建（None）→ 视为静止：仅记录上拍；若停泊基准也未播种则本拍播种
///   （开窗/重泊后的常规首拍路径）。
/// - 实界与上拍不一致 = 拖动进行中 → 仅记录，不吸附、不限位、不预览吸附。
/// - 实界与上拍一致（连续两拍读数相同）= 释放近似 → 与停泊基准比对：超容差
///   SNAP_EPS 则贴边重泊（关 Peek + 关窗重开；基准/基线均在 open_panel_docked
///   重置待播种）。
fn snap_watch_tick_with(cx: &mut App, actual: Option<(f32, f32, f32, f32)>) {
    if cx.global::<DockHandles>().panel.is_none() {
        return;
    }
    let Some(actual) = actual else {
        return; // 句柄失效（窗口已关）：本轮无动作
    };
    let stable = match cx.global::<DockHandles>().drag_last {
        // 基线未建：视为静止（不阻塞常规首拍播种）。
        None => true,
        // 与上拍一致 = 拖动已停（释放近似）；不一致 = 拖动中。
        Some(last) => !drifted(actual, last, 0.0),
    };
    cx.global_mut::<DockHandles>().drag_last = Some(actual);
    if !stable {
        return; // 拖动进行中：不吸附、不限位、不预览吸附
    }
    let Some(seed) = cx.global::<DockHandles>().docked_bounds else {
        // 首拍播种：读到的实界（含 macOS 钳制调整）即为贴缘基准（R184 建议 6：
        // let-else 消除 unwrap 面——播种语义原样内移）。
        cx.global_mut::<DockHandles>().docked_bounds = Some(actual);
        return;
    };
    if drifted(actual, seed, SNAP_EPS) {
        redock_panel(cx);
    }
}

/// 拖动释放即吸附 / 吸附守护重泊共用收口（DEC-78 一次性重泊操作原样提取 ·
/// DEC-210）：关 Peek（未决 peek-close timer 失效 + 强关）→ 关面板 → 重开；
/// 重泊语义唯一事实源——0.25s 吸附守护与释放即吸附守护双触发面行为恒同，
/// 重泊后基准/拖动基线均在 open_panel_docked 重置待播种（双面幂等，无二次重泊）。
fn redock_panel(cx: &mut App) {
    bump_peek_close_gen(cx); // 未决 peek-close timer 失效，防误关重泊后新面板
    close_peek(cx, true); // 换泊强关（DEC-159）：面板重建随行，stderr 留档
    close_panel(cx);
    open_panel_docked(cx); // 内部重置基准与拖动基线 → 下拍重播种
}

/// 停泊漂移判定：(x, y, w, h) 四元组任一分量差 > eps 即漂移（拖离贴缘/被改尺寸）。
fn drifted(a: (f32, f32, f32, f32), b: (f32, f32, f32, f32), eps: f32) -> bool {
    (a.0 - b.0).abs() > eps
        || (a.1 - b.1).abs() > eps
        || (a.2 - b.2).abs() > eps
        || (a.3 - b.3).abs() > eps
}

/// 释放即吸附守护（DEC-210）：tap 左键释放锁存驱动——释放后首个守护拍（≤0.05s）
/// 读主面板实界判漂移，脱离停泊基准超 SNAP_EPS → 立即贴边重泊（收口沿
/// redock_panel 唯一实现，与 0.25s 吸附守护同语义双保险：释放信号为键态，
/// 慢拖长停顿无误判面）。未授权降级（无 tap → 无锁存）恒 no-op，释放吸附由
/// 0.25s 吸附守护两拍静止判定兖底（DEC-78 既有口径不变）。鼠标在其它应用
/// 按下/释放属常态噪声：漂移判定天然过滤（未拖动 = 未漂移 = 无动作）。
fn start_drag_release_watch(cx: &App) {
    cx.spawn(async move |cx| loop {
        cx.background_executor()
            .timer(Duration::from_secs_f32(DRAG_RELEASE_WATCH_POLL_SECS))
            .await;
        if parchora_platform::tap_mouse_up_take() {
            cx.update(drag_release_tick);
        }
    })
    .detach();
}

/// 释放即吸附单拍（生产包装）：读主面板实界（全局坐标 · DEC-99 缝同口径）交
/// 注入式单拍；面板不在场 = 无拖动对象（no-op）。
fn drag_release_tick(cx: &mut App) {
    let actual = match cx.global::<DockHandles>().panel {
        None => None,
        Some(handle) => window_rect(cx, &handle),
    };
    drag_release_tick_with(cx, actual);
}

/// 释放即吸附单拍（注入 · 测试别名）：释放键态语义由调用链上游承载（生产
/// watch 仅在 tap 左键释放锁存命中时调 tick；测试直呼注入实界）。基准未播种
/// 时保守不动作（播种职责守旧归 0.25s 吸附守护首拍——重泊/开窗后 ≤0.25s 落
/// 定；防把释放落点误播为贴缘基准）；漂移超容差 → redock_panel 即时重泊。
fn drag_release_tick_with(cx: &mut App, actual: Option<(f32, f32, f32, f32)>) {
    if cx.global::<DockHandles>().panel.is_none() {
        return;
    }
    let Some(actual) = actual else {
        return; // 句柄失效（窗口已关）：本轮无动作
    };
    let Some(seed) = cx.global::<DockHandles>().docked_bounds else {
        return; // 基准未播种：本拍保守不动作（守护首拍播种职责）
    };
    if drifted(actual, seed, SNAP_EPS) {
        redock_panel(cx);
    }
}

/// 菜单栏菜单演示窗（300 紧凑宽）。
fn open_menu(cx: &mut App) {
    let opts = window_opts(cx, tk::MENU_W, 380.0, None);
    let _ = cx.open_window(opts, |_window, cx| cx.new(MenuView::new));
}

/// DG-ROOT 启动根选择器演示窗（430 基准；askOnLaunch 判定在 app/任务层）。
fn open_root_picker(cx: &mut App) {
    let opts = window_opts(cx, tk::SETTINGS_W, 420.0, None);
    let _ = cx.open_window(opts, |_window, cx| cx.new(|_cx| RootPicker::default()));
}

/// 无语境页面演示窗（--page <name> · DEC-73 19 页全承载）：
/// - editor：PG-B1 编辑区 480×70%（底部触发位置宽高互换，同 Peek）；
/// - conflict：DG-CONFLICT 命名冲突四选一（430 基准 sheet 形态）；
/// - ext：DG-EXT 外部变更冲突挂起（430 基准）；
/// - picker：DG-PICKER 选择手记浮层（430 基准）。
///
/// 触发链（批量移动/恢复被占/F6/⌘⌥K）随数据层实装后接宿主页，本模式仅承载页面形态。
fn open_page(cx: &mut App, name: &str) {
    match name {
        "editor" => {
            let opts = detail_window_opts(cx, false);
            let _ = cx.open_window(opts, |_window, cx| cx.new(EditorShell::new));
        }
        "conflict" => {
            let opts = window_opts(cx, tk::SETTINGS_W, 360.0, None);
            let _ = cx.open_window(opts, |_window, cx| cx.new(ConflictSheet::new));
        }
        "ext" => {
            let opts = window_opts(cx, tk::SETTINGS_W, 400.0, None);
            let _ = cx.open_window(opts, |_window, cx| cx.new(ExtConflictView::new));
        }
        "picker" => {
            let opts = window_opts(cx, tk::SETTINGS_W, 460.0, None);
            let _ = cx.open_window(opts, |_window, cx| cx.new(PickerSheet::new));
        }
        other => eprintln!("未知 --page 页面：{other}（可用：editor / conflict / ext / picker）"),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn next_seq_after_merges_db_max_and_rows_max() {
        // 序号续接受助（R184 F-183-03）：DB 聚合主导、行内极大值兑底、
        // 双源皆空起算 1（三例场景全覆盖）。
        assert_eq!(next_seq_after(Some(Some(9)), 5), 10, "DB 聚合主导");
        assert_eq!(
            next_seq_after(Some(Some(3)), 5),
            6,
            "行极大值更大时取大兑底"
        );
        assert_eq!(
            next_seq_after(Some(None), 4),
            5,
            "DB 空表/脏 id 全滤 → 行兑底"
        );
        assert_eq!(next_seq_after(None, 0), 1, "双源皆空 → 起算 1");
    }

    #[test]
    fn peek_close_gate_holds_while_cursor_inside_peek() {
        // 滚轮冻结伪退出守门（DEC-109 · D-01）：真光标居 Peek 实界内 → 合成伪
        // 退出不落关（快滚 hover_freeze 场景）；光标出界 / 不可读、窗界不可读 →
        // false 照旧落关（保守口径）。
        let rect = Some((100.0, 200.0, 300.0, 400.0));
        assert!(
            peek_close_gate(Some((100.0, 200.0)), rect),
            "左上角含边保持"
        );
        assert!(
            peek_close_gate(Some((400.0, 600.0)), rect),
            "右下角含边保持"
        );
        assert!(
            !peek_close_gate(Some((400.0, 601.0)), rect),
            "光标出界照旧落关"
        );
        assert!(!peek_close_gate(None, rect), "光标不可读保守落关");
        assert!(
            !peek_close_gate(Some((200.0, 400.0)), None),
            "窗界不可读保守落关"
        );
        assert!(!peek_close_gate(None, None), "双不可读保守落关");
    }
    use super::*;

    #[test]
    fn tap_slot_fresh_cases() {
        // None 槽时刻 → 过期（缺失即不可信）。
        assert!(!tap_slot_fresh(None, Instant::now()));
        let now = Instant::now();
        // 刚写入 → 新鲜。
        assert!(tap_slot_fresh(Some(now), now));
        // 1s 前 → 超龄（大于 TAP_FRESH_SECS=0.5s）。
        assert!(!tap_slot_fresh(
            Some(now - Duration::from_secs_f32(1.0)),
            now
        ));
    }

    #[test]
    fn panel_height_adapts_and_clamps() {
        // 比例命中区间：800×0.80=640。（DEC-198：Hd 档 = 设计基线原值）
        let hd = s::DispTier::Hd;
        assert_eq!(panel_height(800.0, hd), 640.0);
        // 下限钳制：小屏（500×0.80=400 < 480 → 480）。
        assert_eq!(panel_height(500.0, hd), tk::PANEL_H_MIN);
        // 上限钳制：大屏（2000×0.80=1600 > 840 → 840）。
        assert_eq!(panel_height(2000.0, hd), tk::PANEL_H_MAX);
    }

    #[test]
    fn detail_height_adapts_and_clamps() {
        let hd = s::DispTier::Hd;
        assert_eq!(detail_height(1000.0, hd), 700.0);
        assert_eq!(detail_height(600.0, hd), tk::DETAIL_H_MIN); // 420 → 480
        assert_eq!(detail_height(1200.0, hd), tk::DETAIL_H_MAX); // 840 → 780
    }

    #[test]
    fn single_instance_lock_is_exclusive() {
        // 同进程双 fd 对同一文件：第二个 try_lock 应 WouldBlock（flock 语义按 OFD 互斥）。
        let path =
            std::env::temp_dir().join(format!("parchora-lock-test-{}.lock", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let first = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(false)
            .append(false)
            .open(&path)
            .expect("open lock file first");
        first.try_lock().expect("first try_lock succeeds");
        let second = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(false)
            .append(false)
            .open(&path)
            .expect("open lock file second");
        assert!(matches!(
            second.try_lock(),
            Err(std::fs::TryLockError::WouldBlock)
        ));
        let _ = std::fs::remove_file(&path);
    }

    // ───── app 层 Peek/设置窗单实例与防争（test-support 确定性仿真）─────

    fn test_item(title: &str) -> PeekItem {
        PeekItem {
            kind: parchora_ui::pages::peek::PeekKind::Clip,
            title: title.to_string(),
            meta: String::new(),
            body: "正文".to_string(),
            disk_real: false,
        }
    }

    /// 磁盘真实条目（DEC-156 保存漏斗测试承载：title = 题名词干）。
    fn test_disk_item(title: &str, body: &str) -> PeekItem {
        PeekItem {
            kind: parchora_ui::pages::peek::PeekKind::Note,
            title: title.to_string(),
            meta: "磁盘真实 · 刚刚".to_string(),
            body: body.to_string(),
            disk_real: true,
        }
    }

    fn peek_open_count(cx: &App) -> usize {
        cx.windows().len()
    }

    /// 右位热区（演示宿主右位标准 rect：面板 (880,40,320,720) + 屏 1200×800 语义）。
    fn right_zone() -> (f32, f32, f32, f32) {
        dock::trigger_zone_bounds(TriggerPosition::Right, (0.0, 0.0, 1200.0, 800.0))
    }

    /// 实际面板窗界（TestApp 显示器非假想 1200×800，失活期探测的板内点须按实
    /// 界计算 · DEC-81 测试口径）。
    fn actual_panel_rect(cx: &mut gpui::App) -> (f32, f32, f32, f32) {
        let handle = cx.global::<DockHandles>().panel.expect("面板应在开");
        handle
            .update(cx, |_, window, _| {
                let b = window.bounds();
                (
                    f32::from(b.origin.x),
                    f32::from(b.origin.y),
                    f32::from(b.size.width),
                    f32::from(b.size.height),
                )
            })
            .expect("面板实界应可读")
    }

    #[gpui::test]
    async fn peek_is_single_window_updated_in_place(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            init_dock_globals(cx);
            open_or_update_peek(cx, test_item("条目一"));
            open_or_update_peek(cx, test_item("条目二"));
            // 需求 3：同时最多 1 内容详情展示界面（第二次 Show 走原位更新，不重开）。
            assert_eq!(peek_open_count(cx), 1);
            assert!(cx.global::<DockHandles>().peek.is_some());
        });
    }

    #[gpui::test]
    async fn settings_window_is_idempotent(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            init_dock_globals(cx);
            open_settings(cx);
            open_settings(cx);
            open_settings(cx);
            // 需求 3：⚙ 重复点击不堆叠设置窗（幂等单例）。
            assert_eq!(peek_open_count(cx), 1);
        });
    }

    // ───── 待办创建面板提交漏斗（DEC-166 失败驻留 + 幂等抬升）─────

    #[gpui::test]
    async fn task_create_submit_delivers_and_closes_panel(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            init_dock_globals(cx);
            // 面板渲染真 Input（DEC-165 同注）：组件库 Theme 全局测试域同需注册。
            parchora_ui::init(cx);
            // 显式右位：避 TestWindow 初始指针 (0,0) 工件语义（DEC-77 回归同注）。
            cx.set_global(DockPrefs {
                trigger: TriggerPosition::Right,
                active_display: None,
            });
            open_panel_docked(cx);
            open_task_create(cx);
            assert!(cx.global::<DockHandles>().task_create.is_some());
            // 成功路：主面板 update 成功（入行送达）→ 创建面板窗随关（槽位清空）。
            task_create_submit(cx, "回灌行".to_string(), 4, 0, 0);
            assert!(
                cx.global::<DockHandles>().task_create.is_none(),
                "成功提交应关创建面板窗"
            );
        });
    }

    #[gpui::test]
    async fn task_create_submit_failure_retains_panel(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            init_dock_globals(cx);
            // 面板渲染真 Input（DEC-165 同注）：组件库 Theme 全局测试域同需注册。
            parchora_ui::init(cx);
            open_task_create(cx);
            // 失败驻留（DEC-166 · DEC-178 后仅陈旧句柄可达）：主面板登记为
            // Some 僵尸（不摘登记直接移除窗，模拟面板被外部关闭的真实竞态）→
            // 恢复口不触发（None 面才走 ensure）；投递失败 → 不关窗 + stderr
            // 披露 + 创建面板就地 Toast——编辑不静默丢可重试。None 态投递已改
            // 走「先恢复主界面再投递」成功路（restores_hidden 用例承载）。
            open_panel_docked(cx);
            let stale = cx.global::<DockHandles>().panel.unwrap();
            let _ = stale.update(cx, |_, window, _| window.remove_window());
            assert!(cx.global::<DockHandles>().panel.is_some(), "僵尸登记保留");
            task_create_submit(cx, "无主提交".to_string(), 4, 0, 0);
            let handle = cx
                .global::<DockHandles>()
                .task_create
                .expect("失败驻留：创建面板窗不应关闭");
            let toast = handle
                .update(cx, |view, _, _| view.toast())
                .expect("创建面板窗应仍活（update 成功）");
            assert!(toast.is_some(), "失败 Toast 应就地披露");
        });
    }

    // ───── DEC-178：双面板回工作台出口 → 主界面恢复（若隐 · 幂等）─────

    #[gpui::test]
    async fn task_create_submit_restores_hidden_main_panel(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            init_dock_globals(cx);
            parchora_ui::init(cx);
            // 主面板不在场（隐藏基线）：提交先恢复停靠面板再投递（漏斗级超集）
            // → 送达且创建窗随关。
            open_task_create(cx);
            task_create_submit(cx, "隐藏态返回稿".to_string(), 4, 0, 0);
            let h = cx.global::<DockHandles>();
            assert!(h.task_create.is_none(), "提交会话结束关创建窗");
            assert!(h.panel.is_some(), "隐藏态提交应恢复主界面");
        });
    }

    #[gpui::test]
    async fn task_create_back_empty_restores_hidden_main_panel(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            init_dock_globals(cx);
            parchora_ui::init(cx);
            // 空稿返回（返回钮 · BackEmpty 漏斗）：关创建窗 → 恢复主界面 →
            // Toast 披露至重开的面板。
            open_task_create(cx);
            assert!(cx.global::<DockHandles>().panel.is_none());
            task_create_back_empty(cx);
            let h = cx.global::<DockHandles>();
            assert!(h.task_create.is_none(), "空稿返回应关创建窗");
            assert!(h.panel.is_some(), "隐藏态空稿返回应恢复主界面");
        });
    }

    #[gpui::test]
    async fn task_create_cancel_restores_hidden_main_panel(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            init_dock_globals(cx);
            parchora_ui::init(cx);
            // 取消（漏斗级超集 · 独立窗会话结束统一回工作台）：窗内 fire Cancel
            // → defer 出借域关窗 → 主界面恢复。
            open_task_create(cx);
            assert!(cx.global::<DockHandles>().panel.is_none());
            let handle = cx.global::<DockHandles>().task_create.unwrap();
            let _ = handle.update(cx, |view, _, cx| view.fire(&TaskCreateEvent::Cancel, cx));
        });
        cx.run_until_parked();
        cx.read(|cx| {
            let h = cx.global::<DockHandles>();
            assert!(h.task_create.is_none(), "取消应关创建窗");
            assert!(h.panel.is_some(), "隐藏态取消应恢复主界面");
        });
    }

    #[gpui::test]
    async fn editor_escape_restores_hidden_main_panel(cx: &mut gpui::TestAppContext) {
        // 窗内 fire Escape（生产同构 · defer 出借域，editor_escape_closes_handoff
        // _window 同装配）：主面板隐藏基线 → editor_exit 落 true → 恢复口。
        cx.update(|cx| {
            init_dock_globals(cx);
            parchora_ui::init(cx);
            cx.set_global(DockPrefs {
                trigger: TriggerPosition::Right,
                active_display: None,
            });
            open_editor_handoff(cx, test_item("返回恢复"));
            assert!(cx.global::<DockHandles>().panel.is_none());
            let handle = cx.global::<DockHandles>().editor.unwrap();
            let _ = handle.update(cx, |view, _, cx| {
                view.fire_editor_event(&editor::EditorEvent::Escape, cx)
            });
        });
        cx.run_until_parked();
        cx.read(|cx| {
            let h = cx.global::<DockHandles>();
            assert!(h.editor.is_none(), "Esc 应已关编辑窗");
            assert!(h.panel.is_some(), "隐藏态返回应恢复主界面（DEC-178）");
        });
    }

    #[gpui::test]
    async fn ensure_panel_visible_is_idempotent_and_reopens_hidden(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            init_dock_globals(cx);
            parchora_ui::init(cx);
            // 隐藏态 → 重开；在场 → 幂等直返（不重复显示 · 单例守卫）。
            assert!(cx.global::<DockHandles>().panel.is_none());
            ensure_panel_visible(cx);
            assert!(cx.global::<DockHandles>().panel.is_some(), "隐藏态恢复");
            ensure_panel_visible(cx);
            assert!(cx.global::<DockHandles>().panel.is_some(), "在场幂等不重开");
        });
    }

    /// DEC-181 显隐保持：任意显隐周期后仍停留原工作视图——首开默认集锦（无记
    /// 录）· 记账于 close_panel 唯一收口 · 恢复于 open_panel_docked 重开；用户
    /// 切换写经 restore_view 注入（on_tab 同源字段），两周期（待办/拾记）逐轮
    /// 断言宿主记账与 UI 活值。
    #[gpui::test]
    async fn close_reopen_restores_active_tab(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            init_dock_globals(cx);
            parchora_ui::init(cx);
            ensure_panel_visible(cx);
            // 首开无记录 = 默认集锦（View::Collection · DEC-78 口径不变）。
            let panel = cx.global::<DockHandles>().panel.unwrap();
            let _ = panel.update(cx, |p, _, _| assert_eq!(p.current_view(), View::Collection));
            // 用户切至待办（on_tab 同源字段·注入写）→ 隐（记账）→ 显（恢复）。
            let _ = panel.update(cx, |p, _, cx| p.restore_view(View::Todo, cx));
            close_panel(cx);
            assert!(cx.global::<DockHandles>().panel.is_none());
            assert_eq!(
                cx.global::<DockHandles>().panel_view,
                Some(View::Todo),
                "隐藏前记录激活选项卡"
            );
            ensure_panel_visible(cx);
            let panel = cx.global::<DockHandles>().panel.unwrap();
            let _ = panel.update(cx, |p, _, _| {
                assert_eq!(p.current_view(), View::Todo, "重显恢复激活选项卡");
            });
            // 二次周期·拾记：所有选项卡类型通用（枚举承载 · 路径同构）。
            let _ = panel.update(cx, |p, _, cx| p.restore_view(View::Capture, cx));
            close_panel(cx);
            assert_eq!(cx.global::<DockHandles>().panel_view, Some(View::Capture));
            ensure_panel_visible(cx);
            let panel = cx.global::<DockHandles>().panel.unwrap();
            let _ = panel.update(cx, |p, _, _| {
                assert_eq!(p.current_view(), View::Capture, "二次周期同恢复");
            });
        });
    }

    #[gpui::test]
    async fn peek_close_race_gen_invalidates_stale_timer(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            init_dock_globals(cx);
            // TestWindow 初始指针 = 窗内本地 (0,0)：root div 全窗承接 → 任意窗首渲染
            // 触发 on_hover(true)（div.rs paint 期 hover 设定 · DEC-77 实证；左位与
            // 右位均触发，本地坐标口径；真实 mac 无此自动悬停——DEC-67 已验）；
            // 显式设右位且防未来默认变动回归。
            cx.set_global(DockPrefs {
                trigger: TriggerPosition::Right,
                active_display: None,
            });
            open_or_update_peek(cx, test_item("条目一"));
            assert_eq!(peek_open_count(cx), 1);
            // arm 关闭（gen）→ 后续 Show（gen bump）→ 到期后不作废新窗。
            arm_peek_close(cx);
            open_or_update_peek(cx, test_item("条目二"));
        });
        cx.background_executor
            .advance_clock(Duration::from_secs_f32(tk::PEEK_GRACE));
        cx.run_until_parked();
        cx.read(|cx| assert_eq!(peek_open_count(cx), 1, "未决 close 应被 Show 作废"));
        // 再 arm 且无后续 Show → 到期落关：Peek 关 + 面板收起（收缩态 = 完全隐藏）。
        cx.update(arm_peek_close);
        cx.background_executor
            .advance_clock(Duration::from_secs_f32(tk::PEEK_GRACE));
        cx.run_until_parked();
        cx.read(|cx| {
            let h = &cx.global::<DockHandles>();
            // 关键不变量：Peek 已落关 + 面板收起（宽限复位 = 悬停路径已退 → 收缩态
            // 全隐；TestWindow 无自动悬停工件残留）。
            assert!(h.peek.is_none(), "代次未变应落关");
            assert!(h.panel.is_none(), "悬停路径已退：面板应收起");
        });
    }

    // ───── 租借回归（DEC-77）：租借态内直达 app sink 不得同实体嵌套 update ─────
    // 生产链同构触发：shell.rs forward.update(panel) / cx.listener 转发链在
    // PanelView/PeekView 实体租借持有中回调 app sink；sink 同步触同实体
    // 窗柄（panel_actual_bounds/close_panel/close_peek）= double_lease_panic
    // abort，统一 defer 出借域。实例均显式
    // 右位：避 TestWindow 初始指针 (0,0) 工件语义（窗内本地坐标首渲染悬停，
    // DEC-77 已实证），防未来默认变动回归。

    #[gpui::test]
    async fn panel_sink_peek_show_survives_panel_lease(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            init_dock_globals(cx);
            cx.set_global(DockPrefs {
                trigger: TriggerPosition::Right,
                active_display: None,
            });
            open_panel_docked(cx);
            let panel = cx.global::<DockHandles>().panel.unwrap();
            // 租借持有中上抛 PeekShow（同构 forward.update 转发链）：
            // open_peek_item → panel_actual_bounds → 同实体句柄 update = panic。
            let _ = panel.update(cx, |panel, _, cx| {
                panel.emit(ShellEvent::PeekShow(test_item("租借内悬停")), cx)
            });
        });
        cx.run_until_parked();
        cx.read(|cx| {
            // defer 落空：Peek 窗已开（单实例 + 同侧邻接）。
            assert!(
                cx.global::<DockHandles>().peek.is_some(),
                "PeekShow defer 应已生效"
            );
        });
    }

    #[gpui::test]
    async fn panel_sink_mouse_exit_hide_suppressed_while_peek(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            init_dock_globals(cx);
            cx.set_global(DockPrefs {
                trigger: TriggerPosition::Right,
                active_display: None,
            });
            open_panel_docked(cx);
            open_or_update_peek(cx, test_item("阅读态锚定"));
            let panel = cx.global::<DockHandles>().panel.unwrap();
            // 租借持有中上抛 MouseExitHide（close_panel 同实体 update = panic）：
            // Peek 阅读态抑制（SCN-PREVIEW-SCROLL-IN-PREVIEW）：预览窗在悬 → 面板保持。
            // 守卫在 defer 执行内判读（出借域时序：Show 先于 exit 落空执行）。
            let _ = panel.update(cx, |panel, _, cx| panel.emit(ShellEvent::MouseExitHide, cx));
        });
        cx.run_until_parked();
        cx.read(|cx| {
            let h = cx.global::<DockHandles>();
            assert!(h.panel.is_some(), "Peek 在悬时面板应保持");
            assert!(h.peek.is_some(), "Peek 应不受移出影响");
        });
    }

    #[gpui::test]
    async fn panel_sink_mouse_exit_hide_survives_panel_lease(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            init_dock_globals(cx);
            cx.set_global(DockPrefs {
                trigger: TriggerPosition::Right,
                active_display: None,
            });
            open_panel_docked(cx);
            let panel = cx.global::<DockHandles>().panel.unwrap();
            // 租借持有中上抛 MouseExitHide：close_panel 同实体 update = panic
            //（DEC-184：arm 延时挂起，租借安全语义不变）。
            let _ = panel.update(cx, |panel, _, cx| panel.emit(ShellEvent::MouseExitHide, cx));
        });
        cx.run_until_parked();
        // 推进计时器至挂起兑付，再断言收起终态（DEC-184 延时语义）。
        cx.background_executor
            .advance_clock(Duration::from_secs_f32(PANEL_CLOSE_DELAY_SECS + 0.5));
        cx.run_until_parked();
        cx.read(|cx| {
            let h = cx.global::<DockHandles>();
            // 移出隐藏挂起已兑付（1s 复核 close_panel）：收缩态 = 主面板整窗关
            // 闭全隐，进程保持——唤出 = 热
            // 区轮询（reveal_watch · DEC-79 宿主承载），platform 真
            // 语义仍 defer（DEC-67）。断确定性终态 + 无 panic（该路径同
            // 实体嵌套 update 必 abort）。
            assert!(h.panel.is_none(), "移出收回后面板收起全隐");
            assert!(h.peek.is_none());
        });
    }

    #[gpui::test]
    async fn peek_escape_survives_peek_lease(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            init_dock_globals(cx);
            cx.set_global(DockPrefs {
                trigger: TriggerPosition::Right,
                active_display: None,
            });
            open_or_update_peek(cx, test_item("Esc 租借内"));
            let peek_h = cx.global::<DockHandles>().peek.unwrap();
            // 租借持有中上抛 Escape：close_peek 同实体 update = panic。
            let _ = peek_h.update(cx, |view, _, cx| {
                view.fire_peek_event(&peek::PeekEvent::Escape, cx)
            });
        });
        cx.run_until_parked();
        cx.read(|cx| {
            assert!(cx.global::<DockHandles>().peek.is_none(), "Esc 应已关 Peek");
        });
    }

    // ───── 面板内编辑保存漏斗（DEC-156）：磁盘真实条目原地覆写 + 列表同步；
    // 非磁盘/空正文不入盘（temp_files_root 复用同模块既有 helper）─────

    #[gpui::test]
    async fn peek_edit_save_disk_real_rewrites_in_place(cx: &mut gpui::TestAppContext) {
        let root = temp_files_root("rewrite");
        // 预置磁盘真实笔记（模拟盘上已存在同名词干）。
        std::fs::write(root.join("设计笔记.md"), "旧正文").expect("seed note");
        let item = test_disk_item("设计笔记", "新正文（面板内编辑）");
        cx.update(|cx| {
            init_dock_globals(cx);
            // sync 目标无面板在挂 → 安全 no-op（漏斗不依赖面板在挂 · DEC-144）。
            let outcome = save_peek_edit_at(&root, &item, cx);
            assert!(
                matches!(outcome, PeekSaveOutcome::Saved),
                "覆写成功应 Saved"
            );
        });
        let saved = std::fs::read_to_string(root.join("设计笔记.md")).expect("note kept");
        assert_eq!(saved, "新正文（面板内编辑）", "原地覆写不增殖");
        assert_eq!(
            std::fs::read_dir(&root).unwrap().count(),
            1,
            "无顺延新档（owned=词干 ⇒ rewrite）"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[gpui::test]
    async fn peek_edit_save_skips_non_disk_and_empty_body(cx: &mut gpui::TestAppContext) {
        let root = temp_files_root("skip");
        let non_disk = test_item("演示条目改写");
        cx.update(|cx| {
            init_dock_globals(cx);
            let outcome = save_peek_edit_at(&root, &non_disk, cx);
            assert!(matches!(outcome, PeekSaveOutcome::Noop), "非磁盘应 Noop");
        });
        assert!(
            std::fs::read_dir(&root).unwrap().next().is_none(),
            "非磁盘条目不入盘（handoff 同口径）"
        );
        let empty = test_disk_item("空稿", "   \n  ");
        cx.update(|cx| {
            let outcome = save_peek_edit_at(&root, &empty, cx);
            assert!(
                matches!(outcome, PeekSaveOutcome::Noop),
                "空正文非失败应 Noop"
            );
        });
        assert!(
            std::fs::read_dir(&root).unwrap().next().is_none(),
            "空正文不落盘（save_draft_at 同口径）"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[gpui::test]
    async fn peek_edit_save_reports_storage_failure(cx: &mut gpui::TestAppContext) {
        // 存储失败面（DEC-159）：root 为普通文件路径 → create_dir_all 失败 →
        // Failed(io 错误明文) —— 不再折叠 None 静默吞错（R4 错误面信号源）。
        let bad = std::env::temp_dir().join(format!(
            "parchora-peek-fail-root-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        std::fs::write(&bad, "not a dir").expect("seed root file");
        let item = test_disk_item("失守稿", "在编内容");
        cx.update(|cx| {
            init_dock_globals(cx);
            let outcome = save_peek_edit_at(&bad, &item, cx);
            match outcome {
                PeekSaveOutcome::Failed(err) => {
                    assert!(!err.is_empty(), "失败应携带 io 错误明文");
                }
                other => panic!("存储失败应 Failed，实际 {other:?}"),
            }
        });
        let _ = std::fs::remove_file(&bad);
    }

    #[gpui::test]
    async fn trash_note_moves_note_into_sibling_trash(cx: &mut gpui::TestAppContext) {
        // 集锦首页笔记行「删除→回收站」（DEC-164）：真移 files 同级 .trash/
        //（生产布局容器式同构：容器/files + 容器/.trash）→ 根面消失（扫描面同步
        // 收缩）+ 内容原样保留；缺源 = NotFound 上抛不吞（stderr 自证在生产链）。
        let container = std::env::temp_dir().join(format!(
            "parchora-trash-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let root = container.join("files");
        std::fs::create_dir_all(&root).unwrap();
        let src =
            parchora_files::write_note(&root, "会议记录", "# 会议记录\n决议甲").expect("seed note");
        cx.update(|cx| {
            init_dock_globals(cx);
            // sync 目标无面板在挂 → 安全 no-op（DEC-144 口径）。
            let dst = trash_note_at(&root, "会议记录", cx).unwrap();
            assert_eq!(dst, container.join(".trash").join("会议记录.md"));
        });
        assert!(!src.exists(), "根面原位应消失（列表行随同步消失）");
        let dst = container.join(".trash").join("会议记录.md");
        assert_eq!(
            std::fs::read_to_string(&dst).unwrap(),
            "# 会议记录\n决议甲",
            "回收条目内容原样"
        );
        cx.update(|cx| {
            let err = trash_note_at(&root, "幽灵", cx).unwrap_err();
            assert_eq!(err.kind(), std::io::ErrorKind::NotFound, "缺源不吞");
        });
        std::fs::remove_dir_all(&container).unwrap();
    }

    #[gpui::test]
    async fn peek_close_soft_retains_on_save_failure_then_retries(cx: &mut gpui::TestAppContext) {
        // 软关失败驻留重试链（DEC-159 · R3/R4 全链）：保存失败 → 软关窗保留
        //（句柄仍在挂）+ 寄存条目；下次软关好根重试成功 → 落盘内容为寄存
        // 正文 + 窗关（面板内显示与文件内容一致）。
        let root = temp_files_root("retake");
        std::fs::write(root.join("设计笔记.md"), "旧正文").expect("seed note");
        // 坏根：普通文件路径（create_dir_all 忌）。
        let bad = std::env::temp_dir().join(format!(
            "parchora-peek-fail-root-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        std::fs::write(&bad, "not a dir").expect("seed root file");
        cx.update(|cx| {
            init_dock_globals(cx);
            cx.set_global(DockPrefs {
                trigger: TriggerPosition::Right,
                active_display: None,
            });
            open_or_update_peek(cx, test_disk_item("设计笔记", "旧正文"));
            // 模拟前次冲刷上抛失败（保存漏斗失败后 mark 寄存）。
            let handle = cx.global::<DockHandles>().peek.unwrap();
            let _ = handle.update(cx, |view, _w, cx| {
                view.mark_save_failed(
                    test_disk_item("设计笔记", "未落盘新正文"),
                    "模拟存储失败",
                    cx,
                );
            });
        });
        cx.run_until_parked();
        cx.update(|cx| close_peek_at(cx, &bad, false));
        cx.update(|cx| {
            assert!(
                cx.global::<DockHandles>().peek.is_some(),
                "软关保存失败应驻留不关窗（不静默丢失）"
            );
        });
        cx.update(|cx| close_peek_at(cx, &root, false));
        cx.update(|cx| {
            assert!(
                cx.global::<DockHandles>().peek.is_none(),
                "重试成功后应关窗"
            );
        });
        let saved = std::fs::read_to_string(root.join("设计笔记.md")).expect("retry saved");
        assert_eq!(saved, "未落盘新正文", "寄存条目关窗重试落盘");
        let _ = std::fs::remove_dir_all(&root);
        let _ = std::fs::remove_file(&bad);
    }

    #[gpui::test]
    async fn peek_close_force_ignores_save_failure(cx: &mut gpui::TestAppContext) {
        // 强制关窗（DEC-159）：级联/quit/换泊/转出路径失败不驻留（宿主消亡随
        // 行，错误面无可承载）——照常关窗，寄存/在编随窗亡（stderr 留档在册）。
        let bad = std::env::temp_dir().join(format!(
            "parchora-peek-fail-root-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        std::fs::write(&bad, "not a dir").expect("seed root file");
        cx.update(|cx| {
            init_dock_globals(cx);
            cx.set_global(DockPrefs {
                trigger: TriggerPosition::Right,
                active_display: None,
            });
            open_or_update_peek(cx, test_disk_item("设计笔记", "旧正文"));
            let handle = cx.global::<DockHandles>().peek.unwrap();
            let _ = handle.update(cx, |view, _w, cx| {
                view.mark_save_failed(
                    test_disk_item("设计笔记", "未落盘新正文"),
                    "模拟存储失败",
                    cx,
                );
            });
        });
        cx.run_until_parked();
        cx.update(|cx| close_peek_at(cx, &bad, true));
        cx.update(|cx| {
            assert!(
                cx.global::<DockHandles>().peek.is_none(),
                "强关失败不驻留（宿主消亡随行）"
            );
        });
        let _ = std::fs::remove_file(&bad);
    }

    // ───── 详情窗共用回归（DEC-77）：三类条目共用同一内容详情窗（原位更新，禁多开）─────

    #[gpui::test]
    async fn peek_single_window_across_all_kinds(cx: &mut gpui::TestAppContext) {
        use parchora_ui::pages::peek::PeekKind;
        cx.update(|cx| {
            init_dock_globals(cx);
            cx.set_global(DockPrefs {
                trigger: TriggerPosition::Right,
                active_display: None,
            });
            for (kind, title) in [
                (PeekKind::Clip, "剪贴板条目"),
                (PeekKind::Note, "笔记条目"),
                (PeekKind::Task, "待办条目"),
            ] {
                open_or_update_peek(
                    cx,
                    PeekItem {
                        kind,
                        title: title.to_string(),
                        meta: String::new(),
                        body: "正文".to_string(),
                        disk_real: false,
                    },
                );
            }
            // 需求 2：图片/文本/声音/视频与笔记/剪贴板/待办统一详情窗（演示宿主
            // 数据源三类先行：Clip/Note/Task 交替悬停只换内容，窗口数恒为 1）。
            assert_eq!(peek_open_count(cx), 1, "三类条目应原位更新同一详情窗");
            assert!(cx.global::<DockHandles>().peek.is_some());
        });
    }

    // ───── 编辑转出（SCN-PREVIEW-QUICKLOOK-EDIT-HANDOFF ·
    // DEC-24）：关 Peek → 编辑窗原址接手 · 单实例热换 · Esc 自关 · GUI 保持域 ─────

    #[gpui::test]
    async fn editor_handoff_closes_peek_and_updates_editor_in_place(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            init_dock_globals(cx);
            // 编辑窗注入态渲染 Textarea：组件库 Theme 全局在测试域
            // 同需注册（wrapper 每帧读 cx.theme() · 无 init 则 panic）。
            parchora_ui::init(cx);
            cx.set_global(DockPrefs {
                trigger: TriggerPosition::Right,
                active_display: None,
            });
            open_or_update_peek(cx, test_item("编辑转出条目"));
            assert_eq!(peek_open_count(cx), 1, "仅 Peek 窗在场");
            let peek_rect = window_rect(cx, &cx.global::<DockHandles>().peek.unwrap())
                .expect("Peek 实界可读（原址接手锚）");
            // 转出：关 Peek + 编辑窗 peek 原址开立（单实例登记）。
            open_editor_handoff(cx, test_item("编辑转出条目"));
            assert!(cx.global::<DockHandles>().peek.is_none(), "转出应已关 Peek");
            assert!(
                cx.global::<DockHandles>().editor.is_some(),
                "编辑窗应已登记"
            );
            let editor_rect = window_rect(cx, &cx.global::<DockHandles>().editor.unwrap())
                .expect("编辑窗实界可读");
            assert_eq!(peek_rect, editor_rect, "编辑窗应原址接手（Peek 同址）");
            // 再次 handoff：编辑窗单例原位热换，不多开（DEC-94 同源）；Peek 无窗
            // 实界不可读 → 继续保位不重锚（原位更新分支提前 return）。
            open_editor_handoff(cx, test_item("第二条"));
            assert_eq!(peek_open_count(cx), 1, "再次转出应原位更新编辑窗（单实例）");
        });
    }

    #[gpui::test]
    async fn editor_escape_closes_handoff_window(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            init_dock_globals(cx);
            // 同上：Textarea 渲染需组件库 Theme 全局。
            parchora_ui::init(cx);
            cx.set_global(DockPrefs {
                trigger: TriggerPosition::Right,
                active_display: None,
            });
            open_editor_handoff(cx, test_item("Esc 关闭"));
            let handle = cx.global::<DockHandles>().editor.unwrap();
            // 租借持有中上抛 Escape（on_key_down 生产同构）：close_editor
            // 同实体 update = panic；sink 统一 defer 出借域（DEC-77）。
            let _ = handle.update(cx, |view, _, cx| {
                view.fire_editor_event(&editor::EditorEvent::Escape, cx)
            });
        });
        cx.run_until_parked();
        cx.read(|cx| {
            assert!(
                cx.global::<DockHandles>().editor.is_none(),
                "Esc 应已关编辑窗"
            );
        });
    }

    #[gpui::test]
    async fn keep_domain_contains_editor_window(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            init_dock_globals(cx);
            // 同上：Textarea 渲染需组件库 Theme 全局。
            parchora_ui::init(cx);
            cx.set_global(DockPrefs {
                trigger: TriggerPosition::Right,
                active_display: None,
            });
            open_editor_handoff(cx, test_item("域覆盖"));
            let (x, y, w, h) = window_rect(cx, &cx.global::<DockHandles>().editor.unwrap())
                .expect("编辑窗实界可读");
            // 窗内点击（中心点）＝应用 GUI 保持域（同域口径）。
            assert!(
                keep_domain_contains(cx, x + w / 2.0, y + h / 2.0),
                "编辑窗内点击不得触发面板隐藏"
            );
            // 域外对照（屏左上角 · 热区/窗域外）。
            assert!(!keep_domain_contains(cx, 0.0, 0.0), "域外点击仍应隐藏对象");
        });
    }

    // ───── 停泊漂移单元（DEC-77）：停泊漂移判定与守护拍 ─────

    #[test]
    fn drifted_detects_origin_and_size_drift() {
        // 同位/容差内取整差：不漂移（含 = eps 边界：差值恰等于容差不算漂移）。
        assert!(!drifted(
            (0.0, 0.0, 320.0, 640.0),
            (0.0, 0.0, 320.0, 640.0),
            SNAP_EPS
        ));
        assert!(!drifted(
            (98.0, 0.0, 320.0, 640.0),
            (100.0, 0.0, 320.0, 640.0),
            SNAP_EPS
        ));
        // 原点超差（拖离贴缘）→ 漂移；零容差同元组 → 非漂移；超界 2.1 → 漂移。
        assert!(drifted(
            (120.0, 300.0, 320.0, 640.0),
            (0.0, 0.0, 320.0, 640.0),
            SNAP_EPS
        ));
        assert!(!drifted(
            (0.0, 0.0, 320.0, 640.0),
            (0.0, 0.0, 320.0, 640.0),
            0.0
        ));
        assert!(drifted(
            (97.9, 0.0, 320.0, 640.0),
            (100.0, 0.0, 320.0, 640.0),
            SNAP_EPS
        ));
        // 宽高被改（超差）→ 漂移（重泊恢复规格尺寸）。
        assert!(drifted(
            (0.0, 0.0, 330.0, 640.0),
            (0.0, 0.0, 320.0, 640.0),
            SNAP_EPS
        ));
    }

    #[gpui::test]
    async fn snap_watch_tick_noop_when_panel_closed(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            init_dock_globals(cx);
            // 面板未开（其他演示模式/收缩态全隐）：守护拍 no-op，不开窗。
            snap_watch_tick(cx);
            // 注入式单拍同理：无面板不入守护流。
            snap_watch_tick_with(cx, Some((100.0, 100.0, 320.0, 640.0)));
            let h = cx.global::<DockHandles>();
            assert!(h.panel.is_none() && h.peek.is_none());
            assert!(h.docked_bounds.is_none() && h.drag_last.is_none());
        });
    }

    #[gpui::test]
    async fn snap_watch_tick_keeps_docked_panel_untouched(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            init_dock_globals(cx);
            cx.set_global(DockPrefs {
                trigger: TriggerPosition::Right,
                active_display: None,
            });
            open_panel_docked(cx);
            let before = cx.global::<DockHandles>().panel;
            // 首拍（基线未建 = 视为静止）：播种停泊基准（实界=请求停泊位，
            // TestWindow 无窗口服务端调整）。
            snap_watch_tick(cx);
            // 二拍（与上拍一致 = 静止）：基准一致 → 无动作（防无谓关闪，重泊循环
            // 不可能发生）。
            snap_watch_tick(cx);
            assert_eq!(cx.global::<DockHandles>().panel, before, "未漂移不复泊");
            assert!(
                cx.global::<DockHandles>().docked_bounds.is_some(),
                "基准已播种"
            );
        });
    }

    #[gpui::test]
    async fn snap_watch_tick_redocks_after_drift_beyond_eps(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            init_dock_globals(cx);
            cx.set_global(DockPrefs {
                trigger: TriggerPosition::Right,
                active_display: None,
            });
            open_panel_docked(cx);
            let before = cx.global::<DockHandles>().panel.unwrap();
            // 模拟已拖离：预置偏移基准（实界≠基准 → 超容差）。
            cx.global_mut::<DockHandles>().docked_bounds = Some((0.0, 0.0, 320.0, 640.0));
            snap_watch_tick(cx);
            {
                let h = cx.global::<DockHandles>();
                // 重泊 = 关窗重开（vendor 无编程式移窗）：新句柄、Peek 不在场、
                // 基准已重置（下拍按 macOS 钳制后实界重播种——防顶族重泊循环）。
                assert_ne!(h.panel, Some(before), "重泊应为新窗句柄");
                assert!(h.panel.is_some(), "重泊后面板仍在开");
                assert!(h.docked_bounds.is_none(), "基准应重置待播种");
                assert!(h.peek.is_none());
            }
            // 二拍重播种（吸收钳制后实界），三拍与基准一致 → 无动作（无循环）。
            snap_watch_tick(cx);
            let reseeded = cx.global::<DockHandles>().panel;
            snap_watch_tick(cx);
            assert_eq!(
                cx.global::<DockHandles>().panel,
                reseeded,
                "重播种后无再重泊"
            );
        });
    }

    // ───── DEC-78：拖动释放门（拖曳中不限位，释放后一次性吸附）───

    #[gpui::test]
    async fn snap_watch_gate_holds_during_drag_and_redocks_on_release(
        cx: &mut gpui::TestAppContext,
    ) {
        cx.update(|cx| {
            init_dock_globals(cx);
            cx.set_global(DockPrefs {
                trigger: TriggerPosition::Right,
                active_display: None,
            });
            open_panel_docked(cx);
            let before = cx.global::<DockHandles>().panel.unwrap();
            // 播种：实界 = 停泊位（TestWindow 无服务端钳制）。
            snap_watch_tick_with(cx, Some((880.0, 40.0, 320.0, 720.0)));
            assert!(cx.global::<DockHandles>().docked_bounds.is_some());
            // 拖动进行中（相邻拍逐拍位移）：不吸附、不限位，句柄不变。
            snap_watch_tick_with(cx, Some((500.0, 200.0, 320.0, 720.0)));
            snap_watch_tick_with(cx, Some((400.0, 260.0, 320.0, 720.0)));
            assert_eq!(
                cx.global::<DockHandles>().panel,
                Some(before),
                "拖动中不重泊"
            );
            // 释放近似（与上拍读数一致且偏离停泊基准超容差）→ 一次性重泊。
            snap_watch_tick_with(cx, Some((400.0, 260.0, 320.0, 720.0)));
            let h = cx.global::<DockHandles>();
            assert_ne!(h.panel, Some(before), "释放后一次性重泊");
            assert!(h.panel.is_some(), "重泊后面板仍在开");
            assert!(h.docked_bounds.is_none(), "重泊后基准重置待播种");
            assert!(h.drag_last.is_none(), "重泊后拖动基线重置");
            assert!(h.peek.is_none());
        });
        // 重泊后重播种 + 恒位 → 无再泊（无循环）。
        cx.update(|cx| {
            snap_watch_tick_with(cx, Some((880.0, 40.0, 320.0, 720.0)));
            let reseeded = cx.global::<DockHandles>().panel;
            snap_watch_tick_with(cx, Some((880.0, 40.0, 320.0, 720.0)));
            assert_eq!(
                cx.global::<DockHandles>().panel,
                reseeded,
                "重播种后无再重泊"
            );
        });
    }

    #[gpui::test]
    async fn snap_watch_gate_no_redock_when_released_back_on_dock(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            init_dock_globals(cx);
            cx.set_global(DockPrefs {
                trigger: TriggerPosition::Right,
                active_display: None,
            });
            open_panel_docked(cx);
            let before = cx.global::<DockHandles>().panel.unwrap();
            snap_watch_tick_with(cx, Some((880.0, 40.0, 320.0, 720.0))); // 播种
            snap_watch_tick_with(cx, Some((500.0, 200.0, 320.0, 720.0))); // 拖动中
                                                                          // 拖回停泊位释放：连续两拍读数一致且与基准一致 → 不重泊（无谓关闪）。
            snap_watch_tick_with(cx, Some((880.0, 40.0, 320.0, 720.0)));
            snap_watch_tick_with(cx, Some((880.0, 40.0, 320.0, 720.0)));
            assert_eq!(
                cx.global::<DockHandles>().panel,
                Some(before),
                "拖回贴缘释放不重泊"
            );
        });
    }

    // ───── DEC-210：释放即吸附守护（tap 左键释放锁存驱动）───

    #[gpui::test]
    async fn drag_release_tick_noop_without_panel(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            init_dock_globals(cx);
            cx.set_global(DockPrefs {
                trigger: TriggerPosition::Right,
                active_display: None,
            });
            // 面板不在场（演示模式 / 已关闭）：注入「实界」也无动作，状态零扰动
            // （无拖动对象；无基准播种、无重泊）。
            drag_release_tick_with(cx, Some((100.0, 100.0, 320.0, 640.0)));
            let h = cx.global::<DockHandles>();
            assert!(h.panel.is_none() && h.peek.is_none());
            assert!(h.docked_bounds.is_none());
        });
        cx.run_until_parked();
    }

    #[gpui::test]
    async fn drag_release_tick_redocks_immediately_on_release(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            init_dock_globals(cx);
            cx.set_global(DockPrefs {
                trigger: TriggerPosition::Right,
                active_display: None,
            });
            open_panel_docked(cx);
            let before = cx.global::<DockHandles>().panel.unwrap();
            // 首拍播种（实界 = 停泊位 · TestWindow 无服务端驯制）：播种职责守旧归
            // 0.25s 吸附守护首拍（开窗后 ≤0.25s 落定），释放拍只消费基准。
            snap_watch_tick_with(cx, Some((880.0, 40.0, 320.0, 720.0)));
            assert!(cx.global::<DockHandles>().docked_bounds.is_some());
            // 拖离停泊位后释放：释放锁存驱动的单一拍即重泊，不等 0.25s 守护
            // 两拍静止确认（tap 左键释放真键态信号 · DEC-210 释放即吸附）。
            drag_release_tick_with(cx, Some((400.0, 260.0, 320.0, 720.0)));
            let h = cx.global::<DockHandles>();
            assert_ne!(h.panel, Some(before), "释放即吸附：立即重泊");
            assert!(h.panel.is_some(), "重泊后面板仍在开");
            assert!(h.docked_bounds.is_none(), "重泊后基准重置待播种");
            assert!(h.peek.is_none());
        });
        cx.run_until_parked();
        // 重泊后（基准未播种区间）再次释放拍保守不动作：无二次重泊（释放即
        // 吸附与守护双面幂等，播种职责归 0.25s 吸附守护首拍）。
        cx.update(|cx| {
            let re = cx.global::<DockHandles>().panel;
            drag_release_tick_with(cx, Some((880.0, 40.0, 320.0, 720.0)));
            assert_eq!(
                cx.global::<DockHandles>().panel,
                re,
                "重泊后再次释放无二次重泊"
            );
        });
    }

    #[gpui::test]
    async fn drag_release_tick_holds_within_eps(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            init_dock_globals(cx);
            cx.set_global(DockPrefs {
                trigger: TriggerPosition::Right,
                active_display: None,
            });
            open_panel_docked(cx);
            let before = cx.global::<DockHandles>().panel.unwrap();
            drag_release_tick_with(cx, Some((880.0, 40.0, 320.0, 720.0))); // 播种
                                                                           // 释放落点仍在容差带（SNAP_EPS=2px 内差分不漂移）：无动作（无谓关闪
                                                                           // 压制，与吸附守护同口径）。
            drag_release_tick_with(cx, Some((881.0, 42.0, 320.0, 720.0)));
            assert_eq!(
                cx.global::<DockHandles>().panel,
                Some(before),
                "容差带内释放不重泊"
            );
        });
    }

    #[gpui::test]
    async fn reveal_open_leaves_no_pending_close_arm(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            init_dock_globals(cx);
            cx.set_global(DockPrefs {
                trigger: TriggerPosition::Right,
                active_display: None,
            });
            // 显示路径（面板不在场 → 在带单拍直唤）不携带隐藏挂起：显示/隐藏
            // 延时面解耦保证（DEC-210）——无 arm 持活槽，隐藏延时仅在隐藏链
            // 自身命中后由 arm_panel_close 独立挂起。
            let zone = right_zone();
            reveal_watch_tick_with(cx, Some((1199.0, 400.0)), zone);
            let h = cx.global::<DockHandles>();
            assert!(h.panel.is_some(), "滑过首拍立即唤出");
            assert!(h.close_pending_task.is_none(), "显示路径零隐藏挂起");
        });
    }

    // ───── DEC-79：光标轮询唤出（确认节流 + 门控 + 重臂 + 顺缘不收）───

    #[gpui::test]
    async fn reveal_watch_sweeps_reveal_immediately(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            init_dock_globals(cx);
            cx.set_global(DockPrefs {
                trigger: TriggerPosition::Right,
                active_display: None,
            });
            let zone = right_zone();
            let in_zone = (1199.0, 400.0); // Right 热区（12pt 带内 · DEC-100）
                                           // 滑过首拍立即唤出 + 抑制落锁（REVEAL_CONFIRM_TICKS=1；防误呼由抑制门与收起两拍确认反向承载）。
            reveal_watch_tick_with(cx, Some(in_zone), zone);
            let h = cx.global::<DockHandles>();
            assert!(h.panel.is_some(), "滑过首拍立即唤出");
            assert!(h.reveal_suppressed, "唤出后抑制落锁");
            assert_eq!(h.reveal_streak, 0, "唤出后计数复位");
            // 抑制中重复在带拍：不重开（单例不变）。
            let before = cx.global::<DockHandles>().panel;
            reveal_watch_tick_with(cx, Some(in_zone), zone);
            assert_eq!(cx.global::<DockHandles>().panel, before, "抑制期不重开");
            // Esc 收起（光标仍在带内 = 抑制保持）→ 带内滞留不反弹。
            close_panel(cx);
            reveal_watch_tick_with(cx, Some(in_zone), zone);
            let h = cx.global::<DockHandles>();
            assert!(h.panel.is_none(), "带内滞留不反弹");
            assert!(h.reveal_suppressed, "抑制保持");
            // 离带 → 重臂；复入带 → 单拍再唤出（新句柄，无门残留）。
            reveal_watch_tick_with(cx, Some((1000.0, 400.0)), zone);
            assert!(!cx.global::<DockHandles>().reveal_suppressed, "离带重臂");
            reveal_watch_tick_with(cx, Some(in_zone), zone);
            assert!(cx.global::<DockHandles>().panel.is_some(), "重臂后再唤出");
        });
        cx.run_until_parked();
        cx.read(|cx| {
            let h = cx.global::<DockHandles>();
            assert!(h.panel.is_some(), "唤出已落定");
        });
    }

    #[gpui::test]
    async fn reveal_watch_disarms_after_cursor_leaves_zone(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            init_dock_globals(cx);
            cx.set_global(DockPrefs {
                trigger: TriggerPosition::Right,
                active_display: None,
            });
            let zone = right_zone();
            let in_zone = (1199.0, 400.0);
            reveal_watch_tick_with(cx, Some(in_zone), zone);
            assert!(
                cx.global::<DockHandles>().panel.is_some(),
                "滑过首拍唤出成立"
            );
            // 收缩（模拟移出隐藏路径）+ 光标已离带 → 离带拍重臂（armed）。
            close_panel(cx);
            reveal_watch_tick_with(cx, Some((1000.0, 400.0)), zone);
            let h = cx.global::<DockHandles>();
            assert!(h.panel.is_none() && !h.reveal_suppressed, "离带重臂");
            // 重臂后再入带单拍 → 再次唤出（滑过即唤，新句柄，无门残留）。
            reveal_watch_tick_with(cx, Some(in_zone), zone);
            let h = cx.global::<DockHandles>();
            assert!(h.panel.is_some(), "重臂后可再唤出");
        });
    }

    #[gpui::test]
    async fn reveal_watch_holds_suppressed_while_cursor_dwells_in_zone(
        cx: &mut gpui::TestAppContext,
    ) {
        cx.update(|cx| {
            init_dock_globals(cx);
            cx.set_global(DockPrefs {
                trigger: TriggerPosition::Right,
                active_display: None,
            });
            let zone = right_zone();
            // 唤出成立（滑过即唤）后收缩（光标仍在带内 = Esc 收起场景）。
            reveal_watch_tick_with(cx, Some((1199.0, 400.0)), zone);
            close_panel(cx);
            // 带内滞留拍（Esc 后光标未离带）：抑制保持，不反弹唤起（防循环门主体）。
            for _ in 0..4 {
                reveal_watch_tick_with(cx, Some((1199.0, 400.0)), zone);
            }
            let h = cx.global::<DockHandles>();
            assert!(h.panel.is_none(), "带内滞留不反弹");
            assert!(
                h.reveal_suppressed && h.reveal_streak == 0,
                "抑制保持且不计数"
            );
            // 光标离带 → 重臂；再入带 → 滑过即唤（单拍）。
            reveal_watch_tick_with(cx, Some((1000.0, 400.0)), zone);
            assert!(!cx.global::<DockHandles>().reveal_suppressed, "离带重臂");
            reveal_watch_tick_with(cx, Some((1199.0, 400.0)), zone);
            assert!(cx.global::<DockHandles>().panel.is_some(), "重臂后唤出");
        });
    }

    #[gpui::test]
    async fn dormant_probe_arms_inside_and_keeps_without_leave(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            init_dock_globals(cx);
            cx.set_global(DockPrefs {
                trigger: TriggerPosition::Right,
                active_display: None,
            });
            open_panel_docked(cx);
            let zone = right_zone();
            let before = cx.global::<DockHandles>().panel;
            // 板内点/板外点均按实界计算（TestApp 显示器 1920×1080，TestWindow
            // 取请求的贴缘停泊位：右位 = 屏宽减板宽）。
            let (bx, by, bw, bh) = actual_panel_rect(cx);
            let inside = (bx + bw / 2.0, by + bh / 2.0);
            // 光标从未入板（带内外任意拍）：失活期探测不动作（展示语义 · 防启动
            // 闪隐）；门/计数不污染（轮询只管收缩态与失活期仿真）。
            reveal_watch_tick_with(cx, Some((100.0, 100.0)), zone);
            reveal_watch_tick_with(cx, Some((500.0, 400.0)), zone);
            let h = cx.global::<DockHandles>();
            assert_eq!(h.panel, before, "未入板不收（防闪隐）");
            assert!(!h.panel_probe_armed, "带外拍不 arm");
            assert!(
                !h.reveal_suppressed && h.reveal_streak == 0,
                "门与计数不污染"
            );
            // 光标入板（实界中心）：arm、不开不收。
            reveal_watch_tick_with(cx, Some(inside), zone);
            reveal_watch_tick_with(cx, Some(inside), zone);
            let h = cx.global::<DockHandles>();
            assert_eq!(h.panel, before, "入板 arm 期间守护不动");
            assert!(h.panel_probe_armed, "入板即 arm");
        });
    }

    #[gpui::test]
    async fn dormant_probe_closes_after_leave_with_band_and_peek_guards(
        cx: &mut gpui::TestAppContext,
    ) {
        cx.update(|cx| {
            init_dock_globals(cx);
            cx.set_global(DockPrefs {
                trigger: TriggerPosition::Right,
                active_display: None,
            });
            let zone = right_zone();
            // 出板收起（带外 · 无 Peek）：入板 arm → 出板两拍确认 → close（
            // MouseExitHide 同构 · 收起确认 DEC-83）。
            open_panel_docked(cx);
            let (bx, by, bw, bh) = actual_panel_rect(cx);
            let inside = (bx + bw / 2.0, by + bh / 2.0);
            reveal_watch_tick_with(cx, Some(inside), zone);
            assert!(cx.global::<DockHandles>().panel_probe_armed, "入板 arm");
            reveal_watch_tick_with(cx, Some((100.0, 100.0)), zone);
            let h = cx.global::<DockHandles>();
            assert!(h.panel.is_some(), "首拍出板仅计数不收（两拍确认 · DEC-83）");
            reveal_watch_tick_with(cx, Some((100.0, 100.0)), zone);
        });
        // 推进计时器至挂起兑付，再断言收起终态（DEC-184 延时语义）。
        cx.background_executor
            .advance_clock(Duration::from_secs_f32(PANEL_CLOSE_DELAY_SECS + 0.5));
        cx.run_until_parked();
        cx.update(|cx| {
            let h = cx.global::<DockHandles>();
            assert!(h.panel.is_none(), "第二拍确认收起");
            assert!(h.close_streak == 0, "收起后确认计数复位");
            assert!(
                !h.panel_probe_armed && !h.revealed,
                "收起复位：armed 与唤出态清零"
            );
        });
        cx.update(|cx| {
            let zone = right_zone();
            // 排除带感知离场（三处守卫同口径 · DEC-81）：arm 后出板落在排除带段
            // （热区 y<50，板外）→ 不属有效热区 → 两拍确认后收起。
            open_panel_docked(cx);
            let (bx2, by2, bw2, bh2) = actual_panel_rect(cx);
            let inside2 = (bx2 + bw2 / 2.0, by2 + bh2 / 2.0);
            reveal_watch_tick_with(cx, Some(inside2), zone);
            reveal_watch_tick_with(cx, Some((1199.0, 20.0)), zone);
            reveal_watch_tick_with(cx, Some((1199.0, 20.0)), zone);
            let h = cx.global::<DockHandles>();
            assert!(h.panel.is_some(), "带段首拍仅计数不收");
            reveal_watch_tick_with(cx, Some((1199.0, 20.0)), zone);
        });
        // 推进计时器至挂起兑付：带段两拍确认 → 挂起收起兑现（DEC-184 延时
        // 语义；后续段 open_panel_docked 自然重开）。
        cx.background_executor
            .advance_clock(Duration::from_secs_f32(PANEL_CLOSE_DELAY_SECS + 0.5));
        cx.run_until_parked();
        cx.update(|cx| {
            let h = cx.global::<DockHandles>();
            assert!(h.panel.is_none(), "带段出板视同离场（口径一致）");
            assert!(!h.panel_probe_armed, "收起复位");
        });
        cx.update(|cx| {
            let zone = right_zone();
            // 保持样本清零（DEC-83）：出板一拍后回到板内 → 计数清零，再次出板
            // 需重新两拍才收。
            open_panel_docked(cx);
            let (bx4, by4, bw4, bh4) = actual_panel_rect(cx);
            let inside4 = (bx4 + bw4 / 2.0, by4 + bh4 / 2.0);
            reveal_watch_tick_with(cx, Some(inside4), zone);
            reveal_watch_tick_with(cx, Some((100.0, 100.0)), zone);
            reveal_watch_tick_with(cx, Some(inside4), zone);
            reveal_watch_tick_with(cx, Some((100.0, 100.0)), zone);
            let h = cx.global::<DockHandles>();
            assert!(h.panel.is_some(), "保持样本清零后单拍出板不收");
            // Peek 行程抑制：arm 后 Peek 在悬 → 出板不收（MouseExitHide 同口径）。
            open_panel_docked(cx);
            let (bx3, by3, bw3, bh3) = actual_panel_rect(cx);
            let inside3 = (bx3 + bw3 / 2.0, by3 + bh3 / 2.0);
            reveal_watch_tick_with(cx, Some(inside3), zone);
            open_or_update_peek(cx, test_item("失活期行程抑制"));
            reveal_watch_tick_with(cx, Some((100.0, 100.0)), zone);
            let h = cx.global::<DockHandles>();
            assert!(
                h.panel.is_some() && h.peek.is_some(),
                "Peek 在悬失活期面板保持"
            );
        });
    }

    #[gpui::test]
    async fn dormant_probe_zone_keep_resets_close_streak(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            init_dock_globals(cx);
            cx.set_global(DockPrefs {
                trigger: TriggerPosition::Right,
                active_display: None,
            });
            let zone = right_zone();
            // 出板后在有效热区停留（顺缘滑出场景）：在带拍清零计数，不累计收起。
            open_panel_docked(cx);
            let (bx, by, bw, bh) = actual_panel_rect(cx);
            let inside = (bx + bw / 2.0, by + bh / 2.0);
            reveal_watch_tick_with(cx, Some(inside), zone);
            assert!(cx.global::<DockHandles>().panel_probe_armed, "入板 arm");
            // 顺缘滑出到热区段（板外 · 带内）：排除带感知口径 = 有效热区 → 保持。
            reveal_watch_tick_with(cx, Some((1199.0, 400.0)), zone);
            let h = cx.global::<DockHandles>();
            assert!(h.panel.is_some() && h.close_streak == 0, "在带拍保持且清零");
            // 离带出板：重计两拍才收。
            reveal_watch_tick_with(cx, Some((100.0, 100.0)), zone);
            reveal_watch_tick_with(cx, Some((100.0, 100.0)), zone);
        });
        // 推进计时器至挂起兑付，再断言收起（DEC-184 延时语义）。
        cx.background_executor
            .advance_clock(Duration::from_secs_f32(PANEL_CLOSE_DELAY_SECS + 0.5));
        cx.run_until_parked();
        cx.update(|cx| {
            let h = cx.global::<DockHandles>();
            assert!(h.panel.is_none(), "独立离场两拍确认后收起");
        });
    }

    #[gpui::test]
    async fn revealed_poll_close_requires_two_out_ticks(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            init_dock_globals(cx);
            cx.set_global(DockPrefs {
                trigger: TriggerPosition::Right,
                active_display: None,
            });
            let zone = right_zone();
            // 唤出成立（滑过即唤）→ 唵出态面板。
            reveal_watch_tick_with(cx, Some((1199.0, 400.0)), zone);
            assert!(
                cx.global::<DockHandles>().revealed && cx.global::<DockHandles>().panel.is_some(),
                "唤出成立"
            );
            // 首拍同离面板与热区：仅计数，不收（sliver 防抖眼保住的面板后续离场
            // 轮询域 · DEC-83 两拍确认）。
            reveal_watch_tick_with(cx, Some((100.0, 100.0)), zone);
            let h = cx.global::<DockHandles>();
            assert!(h.panel.is_some(), "首拍离场不收");
            assert_eq!(h.close_streak, 1, "离场拍计数");
            assert!(!h.reveal_suppressed, "离带重臂保持离带清零语义");
            // 第二拍：确认收起（延时语义 DEC-184：确认挂 1s timer 接手）。
            reveal_watch_tick_with(cx, Some((100.0, 100.0)), zone);
        });
        // 推进计时器至挂起兑付，再断言收起终态。
        cx.background_executor
            .advance_clock(Duration::from_secs_f32(PANEL_CLOSE_DELAY_SECS + 0.5));
        cx.run_until_parked();
        cx.update(|cx| {
            let h = cx.global::<DockHandles>();
            assert!(h.panel.is_none(), "第二拍确认收起");
            assert_eq!(h.close_streak, 0, "确认后计数复位");
            assert!(!h.revealed, "唤出态清除");
        });
    }

    // ───── 缝侧邻接宽限带（DEC-95 · 副屏缝侧走位挂起收起 · 三态注入式测试）
    // 装配与 revealed_poll_close_requires_two_out_ticks 同构：右位语义 zone
    // （1200 屏宽 12pt 带 · 缝 x=1200 · 带 x∈(1200,1220]）；panel 点按实界口径
    // （TestApp 虚拟屏 1920×1080 · DEC-81 测试口径分叉）。─────

    #[gpui::test]
    async fn seam_grace_band_holds_poll_close_then_times_out(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            init_dock_globals(cx);
            cx.set_global(DockPrefs {
                trigger: TriggerPosition::Right,
                active_display: None,
            });
            let zone = right_zone();
            // 唵出成立（滑过即唤）→ 唵出态面板。
            reveal_watch_tick_with(cx, Some((1199.0, 400.0)), zone);
            assert!(
                cx.global::<DockHandles>().revealed && cx.global::<DockHandles>().panel.is_some(),
                "唤出成立"
            );
            // 常规语义下两拍即收（对照 revealed_poll_close_requires_two_out_ticks）
            // ——缝带内两拍不收（宽限挂起差异证明）。
            reveal_watch_tick_with(cx, Some((1205.0, 400.0)), zone);
            reveal_watch_tick_with(cx, Some((1205.0, 400.0)), zone);
            let h = cx.global::<DockHandles>();
            assert!(h.panel.is_some(), "缝带内两拍不收（常规语义应已收）");
            assert_eq!(h.close_streak, 0, "挂起拍收起确认清零");
            assert_eq!(h.seam_grace_ticks, 2, "宽限计数上行");
            // 带内长驻：补足至 12 拍；满格当拍仍不收（超限后恢复常规两拍）。
            for _ in 0..(SEAM_GRACE_TICKS - 2) {
                reveal_watch_tick_with(cx, Some((1205.0, 400.0)), zone);
            }
            let h = cx.global::<DockHandles>();
            assert_eq!(h.seam_grace_ticks, SEAM_GRACE_TICKS, "宽限计数满格");
            assert!(h.panel.is_some(), "满格当拍不收");
            // 超限后恢复常规两拍：再两拍确认收起（总时长 0.6s+0.1s 语义）。
            reveal_watch_tick_with(cx, Some((1205.0, 400.0)), zone);
            assert_eq!(
                cx.global::<DockHandles>().close_streak,
                1,
                "超限拍走常规计数"
            );
            reveal_watch_tick_with(cx, Some((1205.0, 400.0)), zone);
        });
        // 推进计时器至挂起兑付，再断言收起终态（DEC-184 延时语义）。
        cx.background_executor
            .advance_clock(Duration::from_secs_f32(PANEL_CLOSE_DELAY_SECS + 0.5));
        cx.run_until_parked();
        cx.update(|cx| {
            let h = cx.global::<DockHandles>();
            assert!(h.panel.is_none(), "带内长驻超限后恢复常规收起");
            assert_eq!(h.seam_grace_ticks, 0, "收起复位链：宽限计数随面板关闭清零");
        });
    }

    #[gpui::test]
    async fn seam_grace_band_out_clears_ticks_and_closes_two_ticks(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            init_dock_globals(cx);
            cx.set_global(DockPrefs {
                trigger: TriggerPosition::Right,
                active_display: None,
            });
            let zone = right_zone();
            reveal_watch_tick_with(cx, Some((1199.0, 400.0)), zone);
            assert!(cx.global::<DockHandles>().panel.is_some(), "唤出成立");
            // 带内积 2 拍宽限计数 → 带外首拍清零并走常规计数 → 恰两拍收（无宽限）。
            reveal_watch_tick_with(cx, Some((1205.0, 400.0)), zone);
            reveal_watch_tick_with(cx, Some((1205.0, 400.0)), zone);
            assert_eq!(
                cx.global::<DockHandles>().seam_grace_ticks,
                2,
                "带内计数上行"
            );
            reveal_watch_tick_with(cx, Some((100.0, 100.0)), zone);
            let h = cx.global::<DockHandles>();
            assert_eq!(h.seam_grace_ticks, 0, "带外宽限计数清零");
            assert_eq!(h.close_streak, 1, "带外首拍走常规计数");
            assert!(h.panel.is_some(), "首拍不收");
            reveal_watch_tick_with(cx, Some((100.0, 100.0)), zone);
        });
        // 推进计时器至挂起兑付，再断言收起（DEC-184 延时语义）。
        cx.background_executor
            .advance_clock(Duration::from_secs_f32(PANEL_CLOSE_DELAY_SECS + 0.5));
        cx.run_until_parked();
        cx.update(|cx| {
            assert!(
                cx.global::<DockHandles>().panel.is_none(),
                "带外无宽限恰两拍收"
            );
        });
    }

    #[gpui::test]
    async fn seam_grace_panel_reentry_resets_grace_ticks(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            init_dock_globals(cx);
            cx.set_global(DockPrefs {
                trigger: TriggerPosition::Right,
                active_display: None,
            });
            let zone = right_zone();
            reveal_watch_tick_with(cx, Some((1199.0, 400.0)), zone);
            assert!(cx.global::<DockHandles>().panel.is_some(), "唤出成立");
            // 带内 12 拍满格 → 光标回板内（保持样本同拍复位宽限计数）。
            for _ in 0..SEAM_GRACE_TICKS {
                reveal_watch_tick_with(cx, Some((1205.0, 400.0)), zone);
            }
            assert_eq!(
                cx.global::<DockHandles>().seam_grace_ticks,
                SEAM_GRACE_TICKS,
                "满格在案"
            );
            let (bx, by, bw, bh) = actual_panel_rect(cx);
            reveal_watch_tick_with(cx, Some((bx + bw / 2.0, by + bh / 2.0)), zone);
            let h = cx.global::<DockHandles>();
            assert!(h.panel.is_some(), "板内保持");
            assert_eq!(h.seam_grace_ticks, 0, "回域复位：板内拍清零宽限计数");
            assert_eq!(h.close_streak, 0, "板内保持样本同拍清零（既定语义）");
            // 回域后带内 2 拍仍挂起不收（宽限计数重起算证明）。
            reveal_watch_tick_with(cx, Some((1205.0, 400.0)), zone);
            reveal_watch_tick_with(cx, Some((1205.0, 400.0)), zone);
            let h = cx.global::<DockHandles>();
            assert!(h.panel.is_some(), "复位后再带内两拍仍挂起");
            assert_eq!(h.seam_grace_ticks, 2, "宽限计数自零重起算");
        });
    }

    // ───── DEC-100：快速滑过锁存（zone_hit 注入）与离界缓冲带 ─────

    #[gpui::test]
    async fn reveal_watch_hit_latch_opens_after_swipe(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            init_dock_globals(cx);
            cx.set_global(DockPrefs {
                trigger: TriggerPosition::Right,
                active_display: None,
            });
            let zone = right_zone();
            // 快速滑过：轮询采样拍间光标已穿带（out 采样），tap 已锁存 → 单拍喚出
            // （与在带同径 · 任意速度滑过均喚 · DEC-100）。
            reveal_watch_tick_with_hit(cx, Some((1000.0, 400.0)), zone, true);
            let h = cx.global::<DockHandles>();
            assert!(h.panel.is_some(), "锁存喚出成立（采样漏检补口）");
            assert!(h.revealed, "锁存喚出落喚出态");
            assert!(h.reveal_suppressed, "喚出后抑制落锁（与在带同径）");
            // 面板在场拍重复锁存：消费即丢弃（喚出态轮询分支吸收）。
            let before = cx.global::<DockHandles>().panel;
            reveal_watch_tick_with_hit(cx, Some((1000.0, 400.0)), zone, true);
            assert_eq!(
                cx.global::<DockHandles>().panel,
                before,
                "面板在场锁存即丢弃"
            );
            // 面板收起后无锁存（zone_hit=false）离带拍：不喚出（锁存不跨拍存活，
            // 仅当拍生效）。
            close_panel(cx);
            reveal_watch_tick_with_hit(cx, Some((1000.0, 400.0)), zone, false);
            assert!(cx.global::<DockHandles>().panel.is_none(), "无锁存不解喚");
        });
    }

    #[gpui::test]
    async fn reveal_watch_hit_latch_suppressed_falls_through_rearm(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            init_dock_globals(cx);
            cx.set_global(DockPrefs {
                trigger: TriggerPosition::Right,
                active_display: None,
            });
            let zone = right_zone();
            // Esc 收起后带内滞留（抑制保持）→ 锁存拍光标在带外：抑制期锁存不反弹
            // ——落离带重臂（面板保持收起态），复入带可再喚。
            reveal_watch_tick_with(cx, Some((1199.0, 400.0)), zone);
            close_panel(cx);
            assert!(cx.global::<DockHandles>().reveal_suppressed, "抑制在案");
            reveal_watch_tick_with_hit(cx, Some((1000.0, 400.0)), zone, true);
            let h = cx.global::<DockHandles>();
            assert!(h.panel.is_none(), "抑制期锁存不反弹");
            assert!(!h.reveal_suppressed, "锁存拍带外 = 离带观察重臂");
            reveal_watch_tick_with(cx, Some((1199.0, 400.0)), zone);
            assert!(
                cx.global::<DockHandles>().panel.is_some(),
                "重臂后入带可再喚"
            );
        });
    }

    #[gpui::test]
    async fn exit_grace_band_holds_poll_close_then_times_out(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            init_dock_globals(cx);
            cx.set_global(DockPrefs {
                trigger: TriggerPosition::Right,
                active_display: None,
            });
            let zone = right_zone();
            reveal_watch_tick_with(cx, Some((1199.0, 400.0)), zone);
            assert!(cx.global::<DockHandles>().panel.is_some(), "喚出成立");
            // 离界过渡带点（面板实界内缘外 10pt · y 居中）：常规语义两拍应已
            // 收，缓冲带内两拍不收（DEC-100 不瞬隐）。
            let (bx, by, _bw, bh) = actual_panel_rect(cx);
            let near = (bx - 10.0, by + bh / 2.0);
            reveal_watch_tick_with(cx, Some(near), zone);
            reveal_watch_tick_with(cx, Some(near), zone);
            let h = cx.global::<DockHandles>();
            assert!(h.panel.is_some(), "缓冲带内两拍不收（常规语义应已收）");
            assert_eq!(h.close_streak, 0, "挂起拍收起确认清零");
            assert_eq!(h.exit_grace_ticks, 2, "缓冲计数上行");
            // 带内长驻：补足至 12 拍；满格当拍仍不收。
            for _ in 0..(EXIT_GRACE_TICKS - 2) {
                reveal_watch_tick_with(cx, Some(near), zone);
            }
            let h = cx.global::<DockHandles>();
            assert_eq!(h.exit_grace_ticks, EXIT_GRACE_TICKS, "缓冲计数满格");
            assert!(h.panel.is_some(), "满格当拍不收");
            // 超限后恢复常规两拍（总时长 0.6s+0.1s 语义 · 与缝带同构）。
            reveal_watch_tick_with(cx, Some(near), zone);
            assert_eq!(
                cx.global::<DockHandles>().close_streak,
                1,
                "超限拍走常规计数"
            );
            reveal_watch_tick_with(cx, Some(near), zone);
        });
        // 推进计时器至挂起兑付，再断言收起（DEC-184 延时语义）。
        cx.background_executor
            .advance_clock(Duration::from_secs_f32(PANEL_CLOSE_DELAY_SECS + 0.5));
        cx.run_until_parked();
        cx.update(|cx| {
            let h = cx.global::<DockHandles>();
            assert!(h.panel.is_none(), "长驻超限后恢复常规收起");
            assert_eq!(h.exit_grace_ticks, 0, "收起复位链：缓冲计数随面板关闭清零");
        });
    }

    #[gpui::test]
    async fn exit_grace_band_out_clears_ticks_and_closes_two_ticks(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            init_dock_globals(cx);
            cx.set_global(DockPrefs {
                trigger: TriggerPosition::Right,
                active_display: None,
            });
            let zone = right_zone();
            reveal_watch_tick_with(cx, Some((1199.0, 400.0)), zone);
            assert!(cx.global::<DockHandles>().panel.is_some(), "喚出成立");
            // 带内积 2 拍 → 带外首拍清零并走常规计数 → 恰两拍收。
            let (bx, by, _bw, bh) = actual_panel_rect(cx);
            let near = (bx - 10.0, by + bh / 2.0);
            reveal_watch_tick_with(cx, Some(near), zone);
            reveal_watch_tick_with(cx, Some(near), zone);
            assert_eq!(
                cx.global::<DockHandles>().exit_grace_ticks,
                2,
                "带内计数上行"
            );
            reveal_watch_tick_with(cx, Some((100.0, 100.0)), zone);
            let h = cx.global::<DockHandles>();
            assert_eq!(h.exit_grace_ticks, 0, "带外缓冲计数清零");
            assert_eq!(h.close_streak, 1, "带外首拍走常规计数");
            assert!(h.panel.is_some(), "首拍不收");
            reveal_watch_tick_with(cx, Some((100.0, 100.0)), zone);
        });
        // 推进计时器至挂起兑付，再断言收起（DEC-184 延时语义）。
        cx.background_executor
            .advance_clock(Duration::from_secs_f32(PANEL_CLOSE_DELAY_SECS + 0.5));
        cx.run_until_parked();
        cx.update(|cx| {
            assert!(
                cx.global::<DockHandles>().panel.is_none(),
                "带外无缓冲恰两拍收"
            );
        });
    }

    #[gpui::test]
    async fn exit_grace_reentry_resets_ticks(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            init_dock_globals(cx);
            cx.set_global(DockPrefs {
                trigger: TriggerPosition::Right,
                active_display: None,
            });
            let zone = right_zone();
            reveal_watch_tick_with(cx, Some((1199.0, 400.0)), zone);
            assert!(cx.global::<DockHandles>().panel.is_some(), "喚出成立");
            let (bx, by, bw, bh) = actual_panel_rect(cx);
            let near = (bx - 10.0, by + bh / 2.0);
            let inside = (bx + bw / 2.0, by + bh / 2.0);
            // 带内 2 拍 → 回板内（保持样本同拍清零）→ 再带内 2 拍重起算仍不收。
            reveal_watch_tick_with(cx, Some(near), zone);
            reveal_watch_tick_with(cx, Some(near), zone);
            reveal_watch_tick_with(cx, Some(inside), zone);
            let h = cx.global::<DockHandles>();
            assert!(h.panel.is_some(), "板内保持");
            assert_eq!(h.exit_grace_ticks, 0, "回域复位：板内拍清零缓冲计数");
            reveal_watch_tick_with(cx, Some(near), zone);
            reveal_watch_tick_with(cx, Some(near), zone);
            let h = cx.global::<DockHandles>();
            assert!(h.panel.is_some(), "复位后再带内两拍仍挂起");
            assert_eq!(h.exit_grace_ticks, 2, "缓冲计数自零重起算");
        });
    }

    #[gpui::test]
    async fn exit_grace_band_holds_dormant_probe(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            init_dock_globals(cx);
            cx.set_global(DockPrefs {
                trigger: TriggerPosition::Right,
                active_display: None,
            });
            let zone = right_zone();
            // 失活期探测路同径（DEC-100）：arm → 出板落过渡带两拍不收 → 远离两拍收。
            open_panel_docked(cx);
            let (bx, by, bw, bh) = actual_panel_rect(cx);
            let inside = (bx + bw / 2.0, by + bh / 2.0);
            let near = (bx - 10.0, by + bh / 2.0);
            reveal_watch_tick_with(cx, Some(inside), zone);
            assert!(cx.global::<DockHandles>().panel_probe_armed, "入板 arm");
            reveal_watch_tick_with(cx, Some(near), zone);
            reveal_watch_tick_with(cx, Some(near), zone);
            let h = cx.global::<DockHandles>();
            assert!(h.panel.is_some(), "失活期出板缓冲带内两拍不收");
            assert_eq!(h.exit_grace_ticks, 2, "缓冲计数上行");
            reveal_watch_tick_with(cx, Some((100.0, 100.0)), zone);
            reveal_watch_tick_with(cx, Some((100.0, 100.0)), zone);
        });
        // 推进计时器至挂起兑付，再断言收起（DEC-184 延时语义）。
        cx.background_executor
            .advance_clock(Duration::from_secs_f32(PANEL_CLOSE_DELAY_SECS + 0.5));
        cx.run_until_parked();
        cx.update(|cx| {
            assert!(
                cx.global::<DockHandles>().panel.is_none(),
                "远离两拍确认收起"
            );
        });
    }

    #[gpui::test]
    async fn revealed_poll_keep_sample_resets_close_streak(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            init_dock_globals(cx);
            cx.set_global(DockPrefs {
                trigger: TriggerPosition::Right,
                active_display: None,
            });
            let zone = right_zone();
            // 唤出成立（滑过即唤）→ 离场一拍 → 回带保持（清零）→ 离场一拍 → 仍不收。
            reveal_watch_tick_with(cx, Some((1199.0, 400.0)), zone);
            assert!(cx.global::<DockHandles>().panel.is_some(), "唤出成立");
            reveal_watch_tick_with(cx, Some((100.0, 100.0)), zone);
            assert_eq!(cx.global::<DockHandles>().close_streak, 1, "离场拍计数");
            reveal_watch_tick_with(cx, Some((1199.0, 400.0)), zone);
            assert_eq!(
                cx.global::<DockHandles>().close_streak,
                0,
                "在带保持样本清零"
            );
            reveal_watch_tick_with(cx, Some((100.0, 100.0)), zone);
            assert_eq!(cx.global::<DockHandles>().close_streak, 1, "重新累计");
            assert!(cx.global::<DockHandles>().panel.is_some(), "单拍离场不收");
            // 回面板内保持（热区外 · 板内）同样清零：再离场需两拍。
            let (bx, by, bw, bh) = actual_panel_rect(cx);
            reveal_watch_tick_with(cx, Some((bx + bw / 2.0, by + bh / 2.0)), zone);
            assert_eq!(cx.global::<DockHandles>().close_streak, 0, "板内保持清零");
            reveal_watch_tick_with(cx, Some((100.0, 100.0)), zone);
            reveal_watch_tick_with(cx, Some((100.0, 100.0)), zone);
        });
        // 推进计时器至挂起兑付，再断言收起（DEC-184 延时语义）。
        cx.background_executor
            .advance_clock(Duration::from_secs_f32(PANEL_CLOSE_DELAY_SECS + 0.5));
        cx.run_until_parked();
        cx.update(|cx| {
            assert!(cx.global::<DockHandles>().panel.is_none(), "独立两拍收起");
        });
    }

    #[gpui::test]
    async fn reveal_exclusion_band_blocks_reveal_in_band_segments(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            init_dock_globals(cx);
            cx.set_global(DockPrefs {
                trigger: TriggerPosition::Right,
                active_display: None,
            });
            let zone = right_zone();
            // 排除带段（顶 50pt）两连拍：不唤出、确认拍不累计（离带口径 · DEC-81）。
            reveal_watch_tick_with(cx, Some((1199.0, 20.0)), zone);
            reveal_watch_tick_with(cx, Some((1199.0, 20.0)), zone);
            let h = cx.global::<DockHandles>();
            assert!(h.panel.is_none(), "排除带段不唤出");
            assert_eq!(h.reveal_streak, 0, "带段拍按离带清零");
            // 带外中段 → 滑过即唤。
            reveal_watch_tick_with(cx, Some((1199.0, 400.0)), zone);
            assert!(cx.global::<DockHandles>().panel.is_some(), "带外正常唤出");
            // 唤出态面板在场，光标移入底排除带段（y>750）→ 不属有效热区 → 两拍确认
            // 后离场收起（收起防抖与唤出对称 · DEC-83；带段拍视同离场口径不变）。
            reveal_watch_tick_with(cx, Some((1199.0, 780.0)), zone);
            let h = cx.global::<DockHandles>();
            assert!(
                h.panel.is_some() && h.close_streak == 1,
                "首拍带段仅计数不收"
            );
            reveal_watch_tick_with(cx, Some((1199.0, 780.0)), zone);
        });
        // 推进计时器至挂起兑付，再断言收起（DEC-184 延时语义）。
        cx.background_executor
            .advance_clock(Duration::from_secs_f32(PANEL_CLOSE_DELAY_SECS + 0.5));
        cx.run_until_parked();
        cx.update(|cx| {
            assert!(
                cx.global::<DockHandles>().panel.is_none(),
                "带段视同离场两拍确认收起"
            );
        });
    }

    #[gpui::test]
    async fn apply_trigger_pos_resets_reveal_gate_and_redocks(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            init_dock_globals(cx);
            cx.set_global(DockPrefs {
                trigger: TriggerPosition::Left,
                active_display: None,
            });
            // 左位唤出（门已落锁）→ 切右位：门/拍复位 + 面板按新位重开（关窗重开）。
            // 注入双帧（全帧=可用区，无占用）避开平台 FFI（TestApp 无真屏语义）。
            let frames = (0.0, 0.0, 1200.0, 800.0);
            let zone = dock::trigger_zone_bounds(TriggerPosition::Left, frames);
            reveal_watch_tick_with(cx, Some((1.0, 400.0)), zone);
            assert!(cx.global::<DockHandles>().panel.is_some(), "左位唤出成立");
            assert!(cx.global::<DockHandles>().reveal_suppressed, "唤出已落锁");
            apply_trigger_pos_with(cx, TriggerPosition::Right, frames, frames);
            let h = cx.global::<DockHandles>();
            assert!(h.panel.is_some(), "面板按新位重开");
            assert!(!h.reveal_suppressed && h.reveal_streak == 0, "门/拍复位");
            assert!(!h.revealed, "换位重开 = 展示语义，不入轮询收起域");
        });
    }

    #[gpui::test]
    async fn apply_trigger_pos_resolves_dock_conflict_and_syncs_availability(
        cx: &mut gpui::TestAppContext,
    ) {
        cx.update(|cx| {
            init_dock_globals(cx);
            // 左侧被 Dock 占用（可用区右移 120）：切左位应自动右切（DEC-84）。
            let full = (0.0, 0.0, 1200.0, 800.0);
            let usable = (120.0, 0.0, 1080.0, 800.0);
            assert!(dock::is_edge_occupied(TriggerPosition::Left, full, usable));
            apply_trigger_pos_with(cx, TriggerPosition::Left, full, usable);
            assert_eq!(
                cx.global::<DockPrefs>().trigger,
                TriggerPosition::Right,
                "左缘被占切换右侧"
            );
            // 占用态全局同步落值（设置 pill 置灰数据源）。
            let avail = cx.global::<dock::DockAvailability>();
            assert!(avail.occupied_left && !avail.occupied_right && !avail.occupied_bottom);
            // 面板按解析后新位（右）重开。
            assert!(
                cx.global::<DockHandles>().panel.is_some(),
                "冲突切换后面板重开"
            );
            // 未占用位原样落值+占用态刷新（底被占→占用位同步，不切位）。
            let full_b = (0.0, 0.0, 1200.0, 800.0);
            let usable_b = (0.0, 0.0, 1200.0, 780.0);
            apply_trigger_pos_with(cx, TriggerPosition::Right, full_b, usable_b);
            assert_eq!(
                cx.global::<DockPrefs>().trigger,
                TriggerPosition::Right,
                "右缘未占原样"
            );
            let avail_b = cx.global::<dock::DockAvailability>();
            assert!(avail_b.occupied_bottom && !avail_b.occupied_left && !avail_b.occupied_right);
        });
    }

    #[gpui::test]
    async fn poll_close_reveals_only_and_keeps_launch_panel(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            init_dock_globals(cx);
            cx.set_global(DockPrefs {
                trigger: TriggerPosition::Right,
                active_display: None,
            });
            let zone = right_zone();
            // 启动面板（revealed=false）：光标远离 + 无 Peek → 不收（展示语义豁免）。
            open_panel_docked(cx);
            let launched = cx.global::<DockHandles>().panel;
            reveal_watch_tick_with(cx, Some((100.0, 100.0)), zone);
            assert_eq!(cx.global::<DockHandles>().panel, launched, "启动面板不闪隐");
            // 关闭启动面板（模拟移出隐藏路径）→ 收缩态；再入带滑过即唤。
            close_panel(cx);
            assert!(cx.global::<DockHandles>().panel.is_none());
            reveal_watch_tick_with(cx, Some((1199.0, 400.0)), zone);
            assert!(cx.global::<DockHandles>().revealed, "唤出面板入收起域");
            // 同离面板+热区：两拍确认离场才收（收起防抖与唤出对称 · DEC-83）。
            reveal_watch_tick_with(cx, Some((100.0, 100.0)), zone);
            let h = cx.global::<DockHandles>();
            assert!(
                h.panel.is_some() && h.close_streak == 1,
                "首拍离场仅计数不收"
            );
            assert!(!h.reveal_suppressed, "离带重臂在首拍落定");
            reveal_watch_tick_with(cx, Some((100.0, 100.0)), zone);
        });
        // 推进计时器至挂起兑付，再断言收起（DEC-184 延时语义）。
        cx.background_executor
            .advance_clock(Duration::from_secs_f32(PANEL_CLOSE_DELAY_SECS + 0.5));
        cx.run_until_parked();
        cx.update(|cx| {
            let zone = dock::trigger_zone_bounds(TriggerPosition::Right, (0.0, 0.0, 1200.0, 800.0));
            assert!(
                cx.global::<DockHandles>().panel.is_none(),
                "第二拍同离面板+热区确认收起"
            );
            assert!(!cx.global::<DockHandles>().revealed, "收起后标记复位");
            // Peek 行程抑制：唤出面板 + Peek 在悬 → 轮询收起不触发。
            reveal_watch_tick_with(cx, Some((1199.0, 400.0)), zone);
            reveal_watch_tick_with(cx, Some((1199.0, 400.0)), zone);
            open_or_update_peek(cx, test_item("行程抑制"));
            reveal_watch_tick_with(cx, Some((300.0, 400.0)), zone);
            let h = cx.global::<DockHandles>();
            assert!(h.panel.is_some() && h.peek.is_some(), "Peek 在悬面板保持");
        });
    }

    // ───── 设置窗悬停外自动关窗 + 持久化 + 状态栏点击 toggle ─────

    #[gpui::test]
    async fn settings_hover_out_saves_and_auto_closes(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            init_dock_globals(cx);
            open_settings(cx);
            assert!(cx.global::<DockHandles>().settings.is_some(), "设置窗在开");
            // 设置窗内 hover：在册 GUI → 保持且计数清零。
            let center = cx
                .global::<DockHandles>()
                .settings
                .and_then(|h| {
                    h.update(cx, |_, window, _| {
                        let b = window.bounds();
                        Some((
                            f32::from(b.origin.x) + f32::from(b.size.width) / 2.0,
                            f32::from(b.origin.y) + f32::from(b.size.height) / 2.0,
                        ))
                    })
                    .ok()
                })
                .expect("设置窗实界应可读");
            auto_close_settings_tick(cx, center);
            let h = cx.global::<DockHandles>();
            assert!(
                h.settings.is_some() && h.settings_close_streak == 0,
                "窗内 hover 保持且清零"
            );
            // 全 GUI 之外 hover（panel/peek 均不在场）×3：未满 4 拍窗口保持。
            for _ in 0..3 {
                auto_close_settings_tick(cx, Some((10.0, 10.0)));
            }
            let h = cx.global::<DockHandles>();
            assert!(
                h.settings.is_some() && h.settings_close_streak == 3,
                "未满确认拍窗口保持"
            );
            // 光标读失败（None）→ 无证据清零（防误关）。
            auto_close_settings_tick(cx, None);
            assert_eq!(
                cx.global::<DockHandles>().settings_close_streak,
                0,
                "None 无证据清零"
            );
            assert!(cx.global::<DockHandles>().settings.is_some());
            // 连续 4 拍全 GUI 之外 → 自动保存（cfg(test) 写档 noop）+ 关窗。
            for _ in 0..4 {
                auto_close_settings_tick(cx, Some((10.0, 10.0)));
            }
            let h = cx.global::<DockHandles>();
            assert!(h.settings.is_none(), "4 拍窗外自动关闭");
            assert_eq!(h.settings_close_streak, 0, "关闭后计数复位");
        });
    }

    // ───── 设置例外关窗（仅移出设置窗即关）+ 首入臂控 ─────

    #[gpui::test]
    async fn settings_hover_out_needs_first_entry_then_armed_closes_on_other_gui(
        cx: &mut gpui::TestAppContext,
    ) {
        cx.update(|cx| {
            init_dock_globals(cx);
            cx.set_global(DockPrefs {
                trigger: TriggerPosition::Right,
                active_display: None,
            });
            open_panel_docked(cx);
            open_settings(cx);
            // 开窗即闪关回归防线：开窗时光标在主面板
            //（居中设置窗实界之外）——未臂不累计，SETTINGS_CLOSE_CONFIRM_TICKS+
            // 3 拍后设置窗仍在且计数为 0。
            let (bx, by, bw, bh) = actual_panel_rect(cx);
            let on_panel = (bx + bw / 2.0, by + bh / 2.0);
            for _ in 0..(SETTINGS_CLOSE_CONFIRM_TICKS + 3) {
                auto_close_settings_tick(cx, Some(on_panel));
            }
            let h = cx.global::<DockHandles>();
            assert!(
                h.settings.is_some() && h.settings_close_streak == 0,
                "未臂（未首入设置窗）不累计不闪关"
            );
            // 臂后（例外口径）：光标首次进入设置窗 → 臂；随后光标在主面板
            //（其它在册 GUI）上 hover 同样累计；未满 4 拍保持。
            let center = cx
                .global::<DockHandles>()
                .settings
                .and_then(|h| {
                    h.update(cx, |_, window, _| {
                        let b = window.bounds();
                        Some((
                            f32::from(b.origin.x) + f32::from(b.size.width) / 2.0,
                            f32::from(b.origin.y) + f32::from(b.size.height) / 2.0,
                        ))
                    })
                    .ok()
                })
                .expect("设置窗实界应可读");
            auto_close_settings_tick(cx, center);
            assert!(
                cx.global::<DockHandles>().settings_close_armed,
                "首入设置窗即臂"
            );
            for _ in 0..3 {
                auto_close_settings_tick(cx, Some(on_panel));
            }
            let h = cx.global::<DockHandles>();
            assert!(
                h.settings.is_some() && h.settings_close_streak == 3,
                "臂后面板上 hover 也累计；未满确认拍保持"
            );
            for _ in 0..(SETTINGS_CLOSE_CONFIRM_TICKS - 3) {
                auto_close_settings_tick(cx, Some(on_panel));
            }
            let h = cx.global::<DockHandles>();
            assert!(
                h.settings.is_none(),
                "臂后移出设置窗：面板上 hover 也关设置窗"
            );
            assert!(
                h.panel.is_some(),
                "设置关窗不牵动主面板（GUI 内不触发隐藏）"
            );
            assert_eq!(h.settings_close_streak, 0, "关闭后计数复位");
            assert!(!h.settings_close_armed, "关闭后臂复位");
        });
    }

    #[gpui::test]
    async fn gui_open_close_is_idempotent_no_reinstance(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            init_dock_globals(cx);
            cx.set_global(DockPrefs {
                trigger: TriggerPosition::Right,
                active_display: None,
            });
            // 需求 1：已显示 → 重复触发忽略（单例守卫不重开不重复渲染）。
            open_panel_docked(cx);
            let before = cx.global::<DockHandles>().panel;
            open_panel_docked(cx);
            assert_eq!(
                cx.global::<DockHandles>().panel,
                before,
                "重复触发不重开面板"
            );
            // 已隐藏 → 重复隐藏 no-op（无窗不 panic，状态收敛复位完整）。
            close_panel(cx);
            close_panel(cx);
            cx.global_mut::<DockHandles>().revealed = true; // 注入残留：对 None 仍收敛
            close_panel(cx);
            let h = cx.global::<DockHandles>();
            assert!(h.panel.is_none(), "重复关闭幂等");
            assert!(!h.revealed && h.close_streak == 0, "状态复位完整");
        });
    }

    // ───── DEC-94：级联关详情窗 / Pin 全路径屏蔽 / 点击面板外隐藏 / skip 门 ─────

    #[gpui::test]
    async fn close_panel_cascades_peek_no_orphan(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            init_dock_globals(cx);
            cx.set_global(DockPrefs {
                trigger: TriggerPosition::Right,
                active_display: None,
            });
            open_or_update_peek(cx, test_item("详情一"));
            assert_eq!(peek_open_count(cx), 1, "详情窗单开");
            // 面板收起级联：详情窗一并清除（SCN-PANEL-HIDE-RETREAT 不残留孤窗）。
            close_panel(cx);
            let h = cx.global::<DockHandles>();
            assert!(h.peek.is_none(), "详情窗级联清除");
            assert!(h.panel.is_none());
            assert_eq!(peek_open_count(cx), 0, "不残留孤立窗口");
        });
    }

    #[gpui::test]
    async fn pin_blocks_polling_hide_and_resumes_after_unpin(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            init_dock_globals(cx);
            cx.set_global(DockPrefs {
                trigger: TriggerPosition::Right,
                active_display: None,
            });
            // 唤出态建立（无面板在带滑过即唤 · 既有 revealed_poll 测试同构）。
            let zone = right_zone();
            reveal_watch_tick_with(cx, Some((1199.0, 400.0)), zone);
            assert!(cx.global::<DockHandles>().revealed, "唤出成立");
            assert!(cx.global::<DockHandles>().panel.is_some(), "面板由唤出开立");
            // Pin → 域外连拍不收（压倒一切 · SCN-PANEL-PIN-BLOCKS-HIDE）。
            cx.global_mut::<DockHandles>().pinned = true;
            for _ in 0..4 {
                reveal_watch_tick_with(cx, Some((100.0, 100.0)), zone);
            }
            assert!(
                cx.global::<DockHandles>().panel.is_some(),
                "Pin 态轮询收起屏蔽"
            );
            // unpin → 两拍确认恢复收起（无状态残留）。
            cx.global_mut::<DockHandles>().pinned = false;
            reveal_watch_tick_with(cx, Some((100.0, 100.0)), zone);
            reveal_watch_tick_with(cx, Some((100.0, 100.0)), zone);
        });
        // 推进计时器至挂起兑付，再断言收起（DEC-184 延时语义）。
        cx.background_executor
            .advance_clock(Duration::from_secs_f32(PANEL_CLOSE_DELAY_SECS + 0.5));
        cx.run_until_parked();
        cx.update(|cx| {
            let h = cx.global::<DockHandles>();
            assert!(h.panel.is_none(), "unpin 后两拍确认收起");
            assert!(!h.pinned, "面板关闭复位 Pin 口径");
        });
    }

    #[gpui::test]
    async fn pin_blocks_dormant_probe_leave(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            init_dock_globals(cx);
            cx.set_global(DockPrefs {
                trigger: TriggerPosition::Right,
                active_display: None,
            });
            open_panel_docked(cx); // 展示语义 revealed=false
            let (px, py, pw, ph) = actual_panel_rect(cx);
            // 入板 arm（失活期探测）。
            reveal_watch_tick_with(cx, Some((px + pw / 2.0, py + ph / 2.0)), right_zone());
            assert!(cx.global::<DockHandles>().panel_probe_armed, "入板探测臂");
            // Pin → 出板两拍不收。
            cx.global_mut::<DockHandles>().pinned = true;
            reveal_watch_tick_with(cx, Some((100.0, 100.0)), right_zone());
            reveal_watch_tick_with(cx, Some((100.0, 100.0)), right_zone());
            assert!(
                cx.global::<DockHandles>().panel.is_some(),
                "Pin 态失活期探测不收起"
            );
            // unpin → 两拍确认收起。
            cx.global_mut::<DockHandles>().pinned = false;
            reveal_watch_tick_with(cx, Some((100.0, 100.0)), right_zone());
            reveal_watch_tick_with(cx, Some((100.0, 100.0)), right_zone());
        });
        // 推进计时器至挂起兑付，再断言收起（DEC-184 延时语义）。
        cx.background_executor
            .advance_clock(Duration::from_secs_f32(PANEL_CLOSE_DELAY_SECS + 0.5));
        cx.run_until_parked();
        cx.update(|cx| {
            assert!(
                cx.global::<DockHandles>().panel.is_none(),
                "unpin 后恢复探测收起"
            );
        });
    }

    #[gpui::test]
    async fn click_outside_close_respects_domain_pin_and_skip(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            init_dock_globals(cx);
            cx.set_global(DockPrefs {
                trigger: TriggerPosition::Right,
                active_display: None,
            });
            open_panel_docked(cx);
            let (px, py, pw, ph) = actual_panel_rect(cx);
            let inside = (px + pw / 2.0, py + ph / 2.0);
            let outside = (100.0, 100.0); // 与既有收起测试同源域外探针点
                                          // 域内点击：保持（GUI 内操作不触发隐藏）。
            click_outside_tick(cx, true, Some(inside));
            assert!(
                cx.global::<DockHandles>().panel.is_some(),
                "GUI 域内点击不隐藏"
            );
            // Pin + 域外点击：保持（压倒一切 · SCN-PANEL-PIN-BLOCKS-HIDE）。
            cx.global_mut::<DockHandles>().pinned = true;
            click_outside_tick(cx, true, Some(outside));
            assert!(
                cx.global::<DockHandles>().panel.is_some(),
                "Pin 压倒点击外隐藏"
            );
            // 开详情窗 + unpin + 域外点击：挂起收起 + 级联清详情窗（DEC-184：
            // 点击外由 1s 复核 timer 接手，兑付后终态同 close_panel 唯一收口）。
            cx.global_mut::<DockHandles>().pinned = false;
            open_or_update_peek(cx, test_item("点击外详情"));
            click_outside_tick(cx, true, Some(outside));
        });
        // 推进计时器至挂起兑付，再断言收起终态（DEC-184 延时语义）。
        cx.background_executor
            .advance_clock(Duration::from_secs_f32(PANEL_CLOSE_DELAY_SECS + 0.5));
        cx.run_until_parked();
        cx.update(|cx| {
            let h = cx.global::<DockHandles>();
            assert!(h.panel.is_none(), "域外点击收起主面板");
            assert!(h.peek.is_none(), "详情窗级联清除（不残留孤窗）");
            assert_eq!(peek_open_count(cx), 0);
        });
        cx.update(|cx| {
            // 无点击旗标：no-op（面板已收态下也不动作）。
            click_outside_tick(cx, false, Some((0.0, 0.0)));
            // skip 门：状态栏 toggle 本点击旗标当拍消费不落动作。
            open_panel_docked(cx);
            let outside = (100.0, 100.0); // 与既有收起测试同源域外探针点
            cx.global_mut::<DockHandles>().skip_click = true;
            click_outside_tick(cx, true, Some(outside));
            let h = cx.global::<DockHandles>();
            assert!(h.panel.is_some(), "skip 门点击消费：不动作");
            assert!(!h.skip_click, "skip 门消费即清");
            // 光标不可读（None）：保守不动作（误收面板代价高于漏收一拍）。
            click_outside_tick(cx, true, None);
            assert!(
                cx.global::<DockHandles>().panel.is_some(),
                "光标不可读保守不动"
            );
        });
    }

    // ───── DEC-184：延时收起机器（用户面自动隐藏链 1s 复核 × 取消防抖）─────

    #[gpui::test]
    async fn delayed_close_arms_then_confirms_after_grace(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            init_dock_globals(cx);
            cx.set_global(DockPrefs {
                trigger: TriggerPosition::Right,
                active_display: None,
            });
            open_panel_docked(cx);
            assert!(cx.global::<DockHandles>().panel.is_some());
            arm_panel_close(cx);
            // 挂起未兑：面板保持（1s 内）。
            assert!(cx.global::<DockHandles>().panel.is_some(), "挂起期面板保持");
            assert!(
                cx.global::<DockHandles>().close_pending_gen >= 1,
                "挂代次已立"
            );
        });
        cx.background_executor
            .advance_clock(Duration::from_secs_f32(PANEL_CLOSE_DELAY_SECS + 0.5));
        cx.run_until_parked();
        cx.update(|cx| {
            let h = cx.global::<DockHandles>();
            assert!(h.panel.is_none(), "兑付后收起");
            assert!(h.close_pending_task.is_none(), "兑付清持活槽");
            assert!(h.peek.is_none(), "级联清详情窗（close_panel 唯一收口）");
        });
    }

    #[gpui::test]
    async fn delayed_close_cancel_on_reopen_and_pin(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            init_dock_globals(cx);
            cx.set_global(DockPrefs {
                trigger: TriggerPosition::Right,
                active_display: None,
            });
            open_panel_docked(cx);
            arm_panel_close(cx);
            cancel_panel_close(cx); // 显式取消源：代次失效
            assert!(cx.global::<DockHandles>().panel.is_some(), "取消后面板保持");
        });
        cx.background_executor
            .advance_clock(Duration::from_secs_f32(PANEL_CLOSE_DELAY_SECS + 0.5));
        cx.run_until_parked();
        cx.update(|cx| {
            assert!(
                cx.global::<DockHandles>().panel.is_some(),
                "已取消挂起不得兑付收起"
            );
        });
        // 重开取消路径（open_panel_docked 头 cancel）：面板不在场态先 arm 再开。
        cx.update(|cx| {
            close_panel(cx);
            let gen_before = cx.global::<DockHandles>().close_pending_gen;
            arm_panel_close(cx); // 面板不在场：不挂（守门）
            assert!(
                cx.global::<DockHandles>().close_pending_gen == gen_before,
                "不在场不重复挂起"
            );
            open_panel_docked(cx);
            arm_panel_close(cx); // 重开态正常挂
            assert!(
                cx.global::<DockHandles>().close_pending_gen == gen_before + 1,
                "重开态正常挂起"
            );
        });
        cx.update(|cx| {
            let gen_pending = cx.global::<DockHandles>().close_pending_gen;
            cx.global_mut::<DockHandles>().pinned = true;
            arm_panel_close(cx); // Pin：不挂（压倒一切 · DEC-94）
            assert!(
                cx.global::<DockHandles>().close_pending_gen == gen_pending,
                "Pin 忌挂起重挂"
            );
            cx.global_mut::<DockHandles>().pinned = false;
            cancel_panel_close(cx); // 清尾（后段测试隔离）
        });
    }

    #[test]
    fn settings_persist_body_round_trip() {
        // 序列化 → 解析 round-trip（DEC-85 锚点 + DEC-163 扩展；DEC-178 保留时长
        // 天制 clip_retention_days；TASK-300 前台排除键 capture_exclude_apps= 逗
        // 号分隔 bundleId；DEC-188 捕获域三件键——capture_enabled=/capture_limit=；
        // DEC-209 队列节流键 queue_notify_interval=；DEC-212 队列消费序键
        // queue_order=）。
        let s = AppSettings {
            trigger: Some(TriggerPosition::Bottom),
            clip_retention_days: Some(3),
            clip_valid_min: Some(50),
            capture_exclude_apps: Some("com.a.x, com.b.y".into()),
            capture_enabled: Some(false),
            capture_limit: Some(1000),
            queue_notify_interval: Some(10),
            queue_order: Some(QueueOrder::Lifo),
        };
        let body = build_settings_body(&s);
        assert_eq!(
            body,
            "trigger=底部\nclip_valid_min=50\nclip_retention_days=3\ncapture_enabled=false\ncapture_limit=1000\nqueue_notify_interval=10\nqueue_order=lifo\ncapture_exclude_apps=com.a.x,com.b.y\n"
        );
        let parsed = parse_settings_body(&body);
        assert_eq!(parsed.trigger, Some(TriggerPosition::Bottom));
        assert_eq!(parsed.clip_retention_days, Some(3));
        assert_eq!(parsed.clip_valid_min, Some(50));
        assert_eq!(
            parsed.capture_exclude_apps.as_deref(),
            Some("com.a.x,com.b.y")
        );
        assert_eq!(parsed.capture_enabled, Some(false));
        assert_eq!(parsed.capture_limit, Some(1000));
        assert_eq!(parsed.queue_notify_interval, Some(10));
        assert_eq!(parsed.queue_order, Some(QueueOrder::Lifo));
        // 缺档键独立容错：trigger 缺不失双阈值，双阈值缺不失 trigger；
        // 未知键/历史遗留键（peek_delay= · DEC-104；clip_survive_min= · DEC-178）
        // 一律忽略；非法值丢弃回默认。
        let partial = parse_settings_body("clip_retention_days=3\n");
        assert_eq!(partial.trigger, None);
        assert_eq!(partial.clip_retention_days, Some(3));
        let legacy = parse_settings_body(
            "foo=bar\ntrigger=上\npeek_delay=9\nclip_survive_min=30\nclip_valid_min=abc\n",
        );
        assert_eq!(legacy.trigger, None);
        assert_eq!(legacy.clip_valid_min, None);
        assert_eq!(
            legacy.clip_retention_days, None,
            "历史键 clip_survive_min 忽略"
        );
        let empty = parse_settings_body("");
        assert_eq!(empty.trigger, None);
        assert_eq!(empty.clip_retention_days, None);
        // 零值档同非法（DEC-184）：0 天/0 分钟档即刻清空拾记——解析铂 ≥1 丢弃。
        let zero = parse_settings_body("clip_retention_days=0\nclip_valid_min=0\n");
        assert_eq!(zero.clip_retention_days, None, "0 天档非法丢弃");
        assert_eq!(zero.clip_valid_min, None, "0 分钟档非法丢弃");
        // DEC-188 新键独立容错：非法开关/非法档值（非五档集）＝缺档回默认。
        let dirty =
            parse_settings_body("capture_enabled=yes\ncapture_limit=777\ncapture_limit=500\n");
        assert_eq!(dirty.capture_enabled, None, "非法开关值丢弃");
        assert_eq!(
            dirty.capture_limit,
            Some(500),
            "777 非档丢弃 · 同名键末值生效"
        );
        // DEC-212 队列消费序键独立容错：非法值丢弃＝键缺省（回 FIFO）；同名末值生效。
        let dirty_order = parse_settings_body("queue_order=weird\nqueue_order=lifo\n");
        assert_eq!(
            dirty_order.queue_order,
            Some(QueueOrder::Lifo),
            "非法值丢弃 · 末值生效"
        );
        assert_eq!(
            parse_settings_body("").queue_order,
            None,
            "缺档键＝None（调用方回 FIFO 默认）"
        );
        // 序列化缺省面：queue_order None → fifo（缺省即写默认 · 同 notify 间隔先例）。
        let body_default = build_settings_body(&AppSettings::default());
        assert!(
            body_default.contains("\nqueue_order=fifo\n"),
            "缺省序列化面 fifo：{body_default}"
        );
    }

    #[test]
    fn clip_capture_defaults_match_config_constants() {
        // DEC-163 默认档（DEC-178 · DEC-188 捕获域三件）：历史保留 1 天 / 有效
        // 24h / 开关开 / 上限 500；守态默认即配置默认。
        let cap = ClipCapture::default();
        assert_eq!(cap.retention_days, CLIP_RETENTION_DAYS_DEFAULT);
        assert_eq!(cap.retention_days, 1);
        assert_eq!(cap.valid_min, 24 * 60);
        assert_eq!(
            cap.retention_days.saturating_mul(CLIP_MINUTES_PER_DAY),
            1440,
            "换算点：默认 1 天"
        );
        assert_eq!(cap.capture_enabled, CLIP_CAPTURE_ENABLED_DEFAULT);
        assert!(cap.capture_enabled, "默认开（既有行为不回退）");
        assert_eq!(cap.limit, CLIP_LIMIT_DEFAULT);
        assert_eq!(cap.limit, 500, "上限档默认＝设置 spec 存储分区展示值");
        assert!(cap.pending.is_empty() && cap.db_retry.is_empty() && cap.exclude_apps.is_empty());
    }

    /// 监控开关（SCN-CAPTURE-SWITCH-OFF · DEC-188）：关闭期到达链全停（不判重
    /// 不入缓冲不入库）；重开后恢复捕获（engine 重播种无行为面差——注入链不
    /// 经 engine，此断言落 apply 落值面）。
    #[gpui::test]
    async fn apply_capture_enabled_gates_capture_chain(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            init_dock_globals(cx);
            clipboard_tick_with(cx, Some("关闭前捕获".into()));
            assert_eq!(cx.global::<ClipCapture>().pending.len(), 1);
            // 关闭：到达链全停——后续载荷不判重不入缓冲。
            apply_capture_enabled(cx, false);
            assert!(!cx.global::<ClipCapture>().capture_enabled);
            clipboard_tick_with(cx, Some("关闭期复制".into()));
            clipboard_tick_with(cx, Some("关闭期复制".into()));
            {
                let cap = cx.global::<ClipCapture>();
                assert_eq!(cap.pending.len(), 1, "关闭期复制不入缓冲");
                assert_eq!(cap.pending[0].title, "关闭前捕获");
            }
            // 重新开启：新内容恢复入库（SCN 恢复语义）；engine 已重播种。
            apply_capture_enabled(cx, true);
            clipboard_tick_with(cx, Some("开启后新捕获".into()));
            {
                let cap = cx.global::<ClipCapture>();
                // pending 为追加序（旧→新）· 冲入列表时逐条顶插还原显示序。
                let texts: Vec<&str> = cap.pending.iter().map(|r| r.title.as_str()).collect();
                assert_eq!(texts, vec!["关闭前捕获", "开启后新捕获"], "追加序");
            }
        });
    }

    /// 历史上限档立即生效（REQ-CAPTURE-LIMIT · DEC-188）：UI 面板非 Pin 裁剪
    /// ＋ DB 对账两轴同拍；Pin 恃免；limit=0 不裁。
    #[gpui::test]
    async fn apply_capture_limit_trims_ui_and_db_immediately(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            parchora_ui::init(cx);
            let root = temp_store_root("limit");
            init_dock_globals(cx);
            init_app_store_at(cx, &root);
            open_panel_docked(cx);
            for t in ["条目一", "条目二", "条目三"] {
                clipboard_tick_with(cx, Some(t.to_string()));
            }
            // Pin 最新条（恒豁免）：面板行右键域外——直接改宿主行面（UI Pin 面
            // 独立域，此处以未 Pin 裁剪为主轴）；apply 内部 sweep 当拍两面对账
            //（SCN-CAPTURE-LIMIT-SHRINK-IMMEDIATE · DEC-189），无手动 sweep。
            apply_capture_limit(cx, 2);
            let panel = cx.global::<DockHandles>().panel.unwrap();
            let titles = panel
                .update(cx, |p, _w, cx| p.clip_titles(cx))
                .expect("面板可读");
            assert_eq!(
                titles,
                vec!["条目三".to_string(), "条目二".to_string()],
                "非 Pin 超限裁最旧（立即生效）"
            );
            let db = app_db(cx).unwrap();
            assert_eq!(list_clips(&db).unwrap().len(), 2, "DB 对账同拍裁剪");
            // 0=不限（五档末位）：不裁（apply 内部 sweep 照跑，limit=0 不触口）。
            apply_capture_limit(cx, 0);
            let db = app_db(cx).unwrap();
            assert_eq!(list_clips(&db).unwrap().len(), 2, "limit=0 不触口");
        });
    }

    /// 排除应用 apply 链（REQ-CAPTURE-EXCLUDE · DEC-188）：归一化串 → 列表去空。
    #[gpui::test]
    async fn apply_capture_exclude_apps_normalizes(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            init_dock_globals(cx);
            apply_capture_exclude_apps(cx, " com.a.x , com.b.y ,  ".into());
            assert_eq!(
                cx.global::<ClipCapture>().exclude_apps,
                vec!["com.a.x".to_string(), "com.b.y".to_string()],
            );
        });
    }

    /// 重试补写成功回填 DB 身份（DEC-188 · D-06）：写失败期行先入缓冲（内存），
    /// 存储恢复后队列先行补写成功——面板/pending 行 db_id 即刻回填（否则右键
    /// 删除只清 UI 面，重启后复现）。
    #[gpui::test]
    async fn retry_flush_backfills_row_db_ids(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            parchora_ui::init(cx);
            let root = temp_store_root("retry-backfill");
            init_dock_globals(cx);
            // AppStore 在场但无活跃根 → 写全败（与 capture_retry_queue 同构）。
            cx.set_global(AppStore::default());
            open_panel_docked(cx);
            clipboard_tick_with(cx, Some("失败期甲".into()));
            // 存储恢复：下一拍捕获先 drain 队列——成功对回填面板行 db_id。
            cx.global::<AppStore>().store.switch_to(&root).unwrap();
            clipboard_tick_with(cx, Some("恢复期乙".into()));
            let panel = cx.global::<DockHandles>().panel.unwrap();
            let (titles, ids) = panel
                .update(cx, |p, _w, cx| (p.clip_titles(cx), p.clip_db_ids(cx)))
                .expect("面板可读");
            assert_eq!(
                titles,
                vec!["恢复期乙".to_string(), "失败期甲".to_string()],
                "队列先行补写后新捕获交付（最新在顶）"
            );
            assert!(
                ids.iter().all(|id| id.is_some()),
                "两行 DB 身份均已回填（D-06 收口）· 实际 {ids:?}"
            );
        });
    }

    /// 冲刷孤儿补写（DEC-188）：写失败期行（无 DB 身份）经重试队列同刻载荷
    /// 补写——image 等非文本行不被文本重建面遗漏。
    #[gpui::test]
    async fn orphan_flush_backfills_via_db_retry_payload(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            let root = temp_store_root("orphan-image");
            init_dock_globals(cx);
            cx.set_global(AppStore::default());
            // 注入 image 类载荷直入到达链（面板不在场 → 缓冲）；写败入重试队。
            clipboard_tick_with_payload(
                cx,
                CapturedPayload {
                    content_type: ContentType::Image,
                    text: None,
                    bytes: Some(b"png-fixture".to_vec()),
                    hash: String::new(),
                    source_app: None,
                    thumb: None,
                    wall: walltime::now_epoch(),
                    at: Instant::now(),
                },
            );
            {
                let cap = cx.global::<ClipCapture>();
                assert_eq!(cap.db_retry.len(), 1, "写失败 image 载荷入队列");
                assert_eq!(cap.pending.len(), 1, "面板不在场行入缓冲");
            }
            // 重开面板（全新实体 + 存储在场）→ 冲刷：孤儿行经队列载荷补写。
            cx.global::<AppStore>().store.switch_to(&root).unwrap();
            init_app_store_at(cx, &root);
            open_panel_docked(cx);
            let db = app_db(cx).unwrap();
            let clips = list_clips(&db).unwrap();
            assert_eq!(clips.len(), 1, "孤儿 image 行补写落库");
            assert_eq!(clips[0].content_type, ContentType::Image);
        });
    }

    /// blob 差集清理（DEC-189 · design L285/287/288 blob 与条目同生命周期＋
    /// 崩溃孤儿启动清理）：启动对账扫盘／左键删除路径（Ok(1)）／sweep 存量对账
    /// 三触发面；单删同生命周期移除对应目录，孤儿目录移除，引用目录恒保留；
    /// blob 落盘随活跃根（store_root · temp 根承载即真机 HOME 不沾）。
    #[gpui::test]
    async fn blob_dirs_pruned_after_delete_and_sweep(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            let root = temp_store_root("blob-gc");
            let blobs = root.join("data").join("blobs");
            // 预置孤儿目录于开根前（崩溃遗留形态）：启动对账扫盘即清（design
            // L287/288 启动清理面）。
            let startup_orphan = blobs.join("00000000-0000-0000-0000-000000000099");
            std::fs::create_dir_all(&startup_orphan).unwrap();
            std::fs::write(startup_orphan.join("raw"), b"stale").unwrap();
            init_dock_globals(cx);
            init_app_store_at(cx, &root);
            assert!(!startup_orphan.exists(), "启动对账扫盘即清孤儿");
            // 三个 image 载荷直落库（raw+thumb 双件 · blob 现落 temp 活跃根）。
            let mut ids = Vec::new();
            for i in 0..3 {
                let payload = CapturedPayload {
                    content_type: ContentType::Image,
                    text: None,
                    bytes: Some(format!("png-{i}").into_bytes()),
                    hash: format!("gc-hash-{i}"),
                    source_app: None,
                    thumb: Some(std::sync::Arc::new(b"thumb-fixture".to_vec())),
                    wall: walltime::now_epoch(),
                    at: Instant::now(),
                };
                let id = match persist_capture(cx, &payload) {
                    Persisted::Done(id) => id,
                    _ => panic!("image 载荷 {i} 应落库"),
                };
                ids.push(id);
            }
            let dir_of = |id: &str| blobs.join(id);
            for (i, id) in ids.iter().enumerate() {
                assert!(dir_of(id).join("raw").is_file(), "raw 落活跃根 #{i}");
                assert!(dir_of(id).join("thumb").is_file(), "thumb 落活跃根 #{i}");
            }
            // 预置第二孤儿目录（行落库后预置 · 单删拍随清面）。
            let orphan = blobs.join("00000000-0000-0000-0000-0000000000aa");
            std::fs::create_dir_all(&orphan).unwrap();
            std::fs::write(orphan.join("raw"), b"stale").unwrap();
            // ① 单删路径：被删行目录与孤儿目录同拍清理，引用目录恒保留。
            clip_delete_from_db(cx, &ids[0]);
            assert!(!dir_of(&ids[0]).exists(), "单删同生命周期清目录");
            assert!(!orphan.exists(), "孤儿目录随拍清理");
            assert!(
                dir_of(&ids[1]).exists() && dir_of(&ids[2]).exists(),
                "引用目录恒保留"
            );
            // ② sweep 存量对账路径：超阈旧行（3 天前 · 保留 1 天档）删行同拍扫盘。
            let stale_payload = CapturedPayload {
                content_type: ContentType::Image,
                text: None,
                bytes: Some(b"png-stale".to_vec()),
                hash: "gc-stale-hash".into(),
                source_app: None,
                thumb: None,
                wall: walltime::now_epoch() - 3 * 86_400,
                at: Instant::now(),
            };
            let stale_id = match persist_capture(cx, &stale_payload) {
                Persisted::Done(id) => id,
                _ => panic!("旧 image 载荷应落库"),
            };
            assert!(dir_of(&stale_id).exists());
            clip_sweep(cx);
            assert!(!dir_of(&stale_id).exists(), "sweep 删行同拍清目录");
            assert!(
                dir_of(&ids[1]).exists() && dir_of(&ids[2]).exists(),
                "存活引用目录跨拍恒保留"
            );
            let db = app_db(cx).unwrap();
            assert_eq!(list_clips(&db).unwrap().len(), 2, "DB 剩两行存活");
        });
    }

    /// 存活时长档调小立即生效（DEC-189）：apply 落值后尾部 sweep 随行两轴
    /// 对账——旧行当拍即清（不等下一次复制）。
    #[gpui::test]
    async fn apply_clip_retention_days_trims_immediately(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            parchora_ui::init(cx);
            let root = temp_store_root("retention-shrink");
            init_dock_globals(cx);
            init_app_store_at(cx, &root);
            open_panel_docked(cx);
            clipboard_tick_with(cx, Some("新鲜条目".to_string()));
            // 三天前旧行直入 DB（不经到达链）：默认保留 1 天档下 apply 前在场。
            let stale = Clip {
                id: new_uuid(),
                content_type: ContentType::PlainText,
                text_content: Some("三天旧闻".into()),
                content_hash: "stale-retained".into(),
                source_app: None,
                size: 15,
                is_pinned: false,
                blob_rel_path: None,
                thumb_rel_path: None,
                edited_at: None,
                created_at: iso_utc(walltime::now_epoch() - 3 * 86_400),
            };
            insert_clip(&app_db(cx).unwrap(), &stale).unwrap();
            let db = app_db(cx).unwrap();
            assert_eq!(list_clips(&db).unwrap().len(), 2, "apply 前新旧两行在场");
            // 调档（1 天档重申）：apply 内部 sweep 当拍对账——旧行即刻消失。
            apply_clip_retention_days(cx, 1);
            let db = app_db(cx).unwrap();
            let rows = list_clips(&db).unwrap();
            assert_eq!(rows.len(), 1, "存活阈值重申当拍即清（DEC-189）");
            assert_eq!(rows[0].text_content.as_deref(), Some("新鲜条目"));
        });
    }

    #[gpui::test]
    async fn apply_clip_retention_days_updates_capture_source(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            init_dock_globals(cx);
            // 设置存储页上抛链落点（DEC-178）：全局单一事实源落值（cfg(test)
            // 写档 noop——真机档不扰动测试；序列化面由 round-trip 纯函数覆盖）。
            apply_clip_retention_days(cx, 3);
            assert_eq!(cx.global::<ClipCapture>().retention_days, 3);
            assert_eq!(
                cx.global::<ClipCapture>()
                    .retention_days
                    .saturating_mul(CLIP_MINUTES_PER_DAY),
                3 * 1440,
                "运行阈随天数换算"
            );
            assert_eq!(cx.global::<ClipCapture>().valid_min, 24 * 60, "有效档不动");
        });
    }

    #[gpui::test]
    async fn clipboard_tick_pend_dedupe_when_panel_closed(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            init_dock_globals(cx);
            // 面板不在场：首条入 pending；10s 窗内同文本判重不入。
            clipboard_tick_with(cx, Some("第一条捕获".to_string()));
            clipboard_tick_with(cx, Some("第一条捕获".to_string()));
            clipboard_tick_with(cx, Some("第二条捕获".to_string()));
            let cap = cx.global::<ClipCapture>();
            let texts: Vec<&str> = cap.pending.iter().map(|r| r.title.as_str()).collect();
            assert_eq!(texts, vec!["第一条捕获", "第二条捕获"], "同内容 10s 判重");
            // 捕获点墙钟随行记录（DEC-183 · 行时显示源）：非零且接近当前 epoch。
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);
            for row in cap.pending.iter() {
                assert!(
                    row.wall.epoch > 0 && now - row.wall.epoch < 60,
                    "墙钟快照应新鲜非零"
                );
            }
        });
    }

    #[gpui::test]
    async fn clipboard_tick_skips_self_write(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            init_dock_globals(cx);
            // 自写板登记后窗内同文本拒收（REQ-CAPTURE-SELF-SKIP · DEC-46）。
            parchora_ui::clipstamp::mark("刚刚从本应用复制的文本");
            clipboard_tick_with(cx, Some("刚刚从本应用复制的文本".to_string()));
            clipboard_tick_with(cx, Some("外部复制的文本".to_string()));
            let cap = cx.global::<ClipCapture>();
            let texts: Vec<&str> = cap.pending.iter().map(|r| r.title.as_str()).collect();
            assert_eq!(texts, vec!["外部复制的文本"], "自写板不入列");
        });
    }

    #[gpui::test]
    async fn clipboard_tick_pending_cap_drops_oldest(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            init_dock_globals(cx);
            // 溢出丢最旧（DEC-163 · cap 200）。
            for i in 0..(CLIP_PENDING_CAP + 10) {
                clipboard_tick_with(cx, Some(format!("条目 {i}")));
            }
            let cap = cx.global::<ClipCapture>();
            assert_eq!(cap.pending.len(), CLIP_PENDING_CAP);
            assert_eq!(
                cap.pending[0].title,
                "条目 10".to_string(),
                "最旧 10 条被丢"
            );
        });
    }

    // ───── 面板实体跨关板存续（DEC-186 两缺陷根修测试）─────

    /// 缺陷二根修（DEC-186）：「隐藏后列表变空」——close_panel 整窗移除前提
    /// 取实体强句柄落全局，重开复用整实体：拾记行/待办行跨显隐周期持久；关
    /// 窗期捕获经 pending 按序冲入（新捕获在顶）。并验面板实体 ID 跨周期
    /// 不变（非全新构造）。
    #[gpui::test]
    async fn close_reopen_preserves_rows_and_flush_order(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            init_dock_globals(cx);
            parchora_ui::init(cx);
            ensure_panel_visible(cx);
            let wall = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);
            let panel = cx.global::<DockHandles>().panel.unwrap();
            panel
                .update(cx, |p, _w, cx| {
                    p.ingest_clip("甲".into(), Instant::now(), wall, cx);
                    p.ingest_clip("乙".into(), Instant::now(), wall, cx);
                    p.ingest_created_task("任务甲".into(), 0, 0, 0, cx);
                })
                .expect("面板在场注入应成功");
            // 整窗移除（显隐唯一收口）：实体提取落全局——窗亡实体存；面板侧
            // Pin 同点复位（与全局 pinned=false 同源 · DEC-186）。
            close_panel(cx);
            assert!(cx.global::<DockHandles>().panel.is_none(), "面板窗已移除");
            let entity = cx
                .global::<DockHandles>()
                .panel_entity
                .clone()
                .expect("close_panel 应提取面板实体强句柄");
            assert_eq!(
                entity.read(cx).clip_titles(cx),
                vec!["乙".to_string(), "甲".to_string()],
                "窗亡实体行存续（最新在顶）"
            );
            assert_eq!(
                entity.read(cx).board_titles(cx),
                vec!["任务甲".to_string()],
                "待办行随实体存续（缺陷二根修）"
            );
            // 关窗期捕获：窗已移除（登记 None）→ 入 pending 缓冲（不丢字）。
            clipboard_tick_with(cx, Some("丙".into()));
            assert_eq!(
                cx.global::<ClipCapture>().pending.len(),
                1,
                "关窗期捕获入缓冲"
            );
            // 重开：复用实体＋缓冲按序冲入——新捕获在顶、存续行随实体保留。
            ensure_panel_visible(cx);
            let panel2 = cx.global::<DockHandles>().panel.unwrap();
            let (clip2, board2) = panel2
                .update(cx, |p, _w, cx| (p.clip_titles(cx), p.board_titles(cx)))
                .expect("重开面板应可读");
            assert_eq!(
                clip2,
                vec!["丙".to_string(), "乙".to_string(), "甲".to_string()],
                "缓冲依捕获序冲入拾记顶（新捕获在顶 · 存续行不丢）"
            );
            assert_eq!(board2, vec!["任务甲".to_string()], "重开后待办行保留");
            // 重开窗根＝同一实体（非全新构造）。
            let entity2 = panel2.entity(cx).expect("重开窗根实体应可取");
            assert_eq!(
                entity2.entity_id(),
                entity.entity_id(),
                "重开复用同一面板实体"
            );
        });
    }

    /// 缺陷一 D-2 兑底（DEC-186）：面板登记在场但窗已被外部移除（僵尸句柄竞
    /// 态）——更新失败入 pending 缓冲兑底，不丢字。
    #[gpui::test]
    async fn clipboard_tick_with_dead_handle_buffers_to_pending(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            init_dock_globals(cx);
            parchora_ui::init(cx);
            open_panel_docked(cx);
            // 僵尸登记（显隐句柄竞态既有先例同式）：窗已 remove 而登记仍在。
            let stale = cx.global::<DockHandles>().panel.unwrap();
            let _ = stale.update(cx, |_, window, _| window.remove_window());
            // 句柄坏死 → 更新失败 → 兑底入缓冲。
            clipboard_tick_with(cx, Some("僵尸期捕获".into()));
            let cap = cx.global::<ClipCapture>();
            let texts: Vec<&str> = cap.pending.iter().map(|r| r.title.as_str()).collect();
            assert_eq!(
                texts,
                vec!["僵尸期捕获"],
                "死句柄竞态不丢字（更新失败改入缓冲）"
            );
        });
    }

    #[gpui::test]
    async fn status_click_toggles_docked_panel(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            APP_MODE_DOCKED.store(true, Ordering::Relaxed);
            init_dock_globals(cx);
            // 无面板 → 开（展示语义：revealed=false，光标入板前不收——
            // 防点击位（菜单栏）与热区远隔的呼出即收闪烁）。
            status_click_toggle(cx);
            let h = cx.global::<DockHandles>();
            assert!(h.panel.is_some() && !h.revealed, "点击展开（展示语义）");
            // 面板在场 → 关（toggle 往返）。
            status_click_toggle(cx);
            assert!(cx.global::<DockHandles>().panel.is_none(), "再点收起");
            APP_MODE_DOCKED.store(false, Ordering::Relaxed);
        });
    }

    #[test]
    fn is_docked_mode_default_and_position_tokens_true() {
        assert!(is_docked_mode(""), "无参 = 主面板停靠形态");
        assert!(is_docked_mode("demo"), "位置演示宿主 = 停靠形态");
        for p in TriggerPosition::ALL {
            assert!(
                is_docked_mode(p.label()),
                "位置 token {} = 停靠形态",
                p.label()
            );
        }
    }

    #[test]
    fn is_docked_mode_demo_windows_false() {
        for m in ["--peek", "--settings", "--menu", "--root", "--page"] {
            assert!(!is_docked_mode(m), "演示窗 {m} 不启唤出守护");
        }
    }

    /// 测试专用临时根（进程号+纳秒唯一目录，用后即清；save_draft_at 显式 root
    /// 入参面）。
    fn temp_files_root(tag: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "parchora-save-draft-test-{}-{}-{}",
            std::process::id(),
            tag,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn save_draft_first_save_writes_titled_file() {
        let root = temp_files_root("first");
        let p = save_draft_at(&root, Some(UNTITLED), Some("首存正文"), None).unwrap();
        assert_eq!(p, root.join("Untitled.md"), "首存=净化题名顺延落盘");
        assert_eq!(std::fs::read_to_string(&p).unwrap(), "首存正文");
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn save_draft_same_title_overwrites_in_place() {
        let root = temp_files_root("overwrite");
        let first = save_draft_at(&root, Some("笔记"), Some("旧稿"), None).unwrap();
        assert_eq!(first, root.join("笔记.md"));
        // 续存同题（owned=净化题名）→ 同路径覆写，不增殖「笔记 2.md」。
        let p = save_draft_at(&root, Some("笔记"), Some("新稿"), Some("笔记")).unwrap();
        assert_eq!(p, root.join("笔记.md"), "同题续存=原地覆写");
        assert_eq!(std::fs::read_to_string(&p).unwrap(), "新稿");
        assert!(!root.join("笔记 2.md").exists(), "同题续存不得增殖文件");
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn save_draft_rename_lands_new_file_keeps_old() {
        let root = temp_files_root("rename");
        save_draft_at(&root, Some(UNTITLED), Some("改名前稿"), None).unwrap();
        // 行内改名后：owned=Untitled 失配新净化题名 → 另落新名（旧稿保留）。
        let p = save_draft_at(&root, Some("新名"), Some("改名后稿"), Some(UNTITLED)).unwrap();
        assert_eq!(p, root.join("新名.md"));
        assert_eq!(std::fs::read_to_string(&p).unwrap(), "改名后稿");
        assert_eq!(
            std::fs::read_to_string(root.join("Untitled.md")).unwrap(),
            "改名前稿",
            "旧稿保留（另落语义）"
        );
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn save_draft_empty_body_produces_no_file() {
        let root = temp_files_root("empty");
        let p = save_draft_at(&root, Some(UNTITLED), Some("   \n"), None);
        assert!(p.is_none(), "纯空白正文不落盘");
        assert!(!root.join("Untitled.md").exists(), "空正文不产生文件");
        std::fs::remove_dir_all(&root).unwrap();
    }

    // 磁盘笔记扫描（DEC-144）：mtime 降序新建置顶 + 噪声面全跳过（点前缀临时
    // 文件/非 md/子目录/共置点前缀目录）。60ms 间隔保证 mtime 严格递增。
    #[test]
    fn disk_scan_orders_by_mtime_desc_and_skips_noise() {
        let root = temp_files_root("disk-scan");
        std::fs::write(root.join("A.md"), "alpha 正文").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(60));
        std::fs::write(root.join("B.md"), "bravo 正文").unwrap();
        std::fs::write(root.join(".A.md.tmp"), "噪声").unwrap();
        std::fs::write(root.join("other.txt"), "噪声").unwrap();
        std::fs::create_dir_all(root.join("subdir")).unwrap();
        std::fs::write(root.join("subdir/deep.md"), "噪声").unwrap();
        std::fs::create_dir_all(root.join(".笔记名")).unwrap();
        std::fs::write(root.join(".笔记名/IMG-x.md"), "噪声").unwrap();
        let notes = scan_disk_notes_at(&root);
        assert_eq!(notes.len(), 2, "仅收根面普通 .md，噪声全跳过");
        assert_eq!(notes[0].title, "B", "mtime 降序=新稿置顶");
        assert_eq!(notes[0].body, "bravo 正文");
        assert_eq!(notes[1].title, "A");
        assert_eq!(notes[1].body, "alpha 正文");
        std::fs::remove_dir_all(&root).unwrap();
    }

    // 相对时距标注桶（DEC-144 · 纯函数）：五桶边界值。
    #[test]
    fn rel_time_label_buckets() {
        assert_eq!(rel_time_label(0), "刚刚");
        assert_eq!(rel_time_label(59), "刚刚");
        assert_eq!(rel_time_label(60), "1 分钟前");
        assert_eq!(rel_time_label(3599), "59 分钟前");
        assert_eq!(rel_time_label(3600), "1 小时前");
        assert_eq!(rel_time_label(86_399), "23 小时前");
        assert_eq!(rel_time_label(86_400), "1 天前");
        assert_eq!(rel_time_label(2_591_999), "29 天前");
        assert_eq!(rel_time_label(2_592_000), "更早");
    }

    // ───── TASK-300 持久化接线（A 档核心场景 · init_app_store_at temp 承载）─────

    /// 隔离 DB 存储根（每个测试独占 temp；进程内多测试并发不互扰）。
    fn temp_store_root(tag: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "parchora-app-store-{tag}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// 重启语义仿真：RootStore 全新实例重开同根（连接先关后开）+ 守护态复位
    ///（init_dock_globals）+ 面板实体/句柄全清（跨进程边界的一切内存态不存续）。
    fn simulate_restart(cx: &mut App, root: &std::path::Path) {
        cx.global_mut::<DockHandles>().panel = None;
        cx.global_mut::<DockHandles>().panel_entity = None;
        cx.global_mut::<DockHandles>().panel_view = None;
        init_dock_globals(cx);
        init_app_store_at(cx, root);
    }

    /// 跨会话拾记持久化（A 档缺陷一根修端到端）：捕获 → DB 事务写 → 重启仿真
    ///（连接重开 + 实体全清）→ 开面板回填 → 行重现且时序保持（最新在顶）。
    #[gpui::test]
    async fn clip_persistence_across_restart(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            parchora_ui::init(cx);
            let root = temp_store_root("clip-restart");
            init_dock_globals(cx);
            init_app_store_at(cx, &root);
            open_panel_docked(cx);
            clipboard_tick_with(cx, Some("甲甲".into()));
            clipboard_tick_with(cx, Some("乙乙".into()));
            // DB 已即时落盘（REQ-CAPTURE-PERSIST）：UI 行都携带 DB 身份。
            {
                let cap = cx.global::<ClipCapture>();
                assert!(cap.pending.is_empty(), "面板在场直接交付");
            }
            let db = app_db(cx).expect("存储根在场");
            assert_eq!(list_clips(&db).unwrap().len(), 2);

            // 重启仿真：连接重开 + 内存态全清（面板/实体/pending/engine/recent）。
            simulate_restart(cx, &root);
            assert!(app_db(cx).is_some(), "重开连接成功");
            open_panel_docked(cx);
            let panel = cx.global::<DockHandles>().panel.unwrap();
            let titles = panel
                .update(cx, |p, _w, cx| p.clip_titles(cx))
                .expect("面板可读");
            assert_eq!(
                titles,
                vec!["乙乙".to_string(), "甲甲".to_string()],
                "重启后行重现（最新在顶 · DB 读序）"
            );
        });
    }

    /// WAL 空闲检点 PASSIVE（DEC-89 双时机 · 空闲臂端到端）：写落 → WAL 有帧
    ///（log>0）→ 单次 PASSIVE 全量检点（busy 0 · ck=帧数，单写者无读者阻塞）。
    #[gpui::test]
    async fn db_idle_checkpoint_passive_moves_wal_frames(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            parchora_ui::init(cx);
            init_dock_globals(cx);
            let root = temp_store_root("checkpoint-idle");
            init_app_store_at(cx, &root);
            let db = app_db(cx).expect("存储根在场");
            insert_clip(
                &db,
                &Clip {
                    id: new_uuid(),
                    content_type: ContentType::PlainText,
                    text_content: Some("检点前落帧".into()),
                    content_hash: "checkpoint-idle-1".into(),
                    source_app: None,
                    size: 18,
                    is_pinned: false,
                    blob_rel_path: None,
                    thumb_rel_path: None,
                    edited_at: None,
                    created_at: iso_utc(walltime::now_epoch()),
                },
            )
            .unwrap();
            let (busy, log, ck) = db.checkpoint_passive().unwrap();
            assert_eq!(busy, 0, "单写者无读者争用");
            assert!(log > 0, "写落后 WAL 有帧");
            assert_eq!(ck, log, "PASSIVE 全量检点（无读者半位截停）");
        });
    }

    /// WAL 退出检点 TRUNCATE（DEC-89 双时机 · 退出臂端到端）：真链 quit_checkpoint
    /// 收口 → 帧全迁＋-wal 文件复零字节 → 重开同根数据仍在——「复零交接不丢态」
    /// 全链（复检点空转 0/0/0 对仗）。
    #[gpui::test]
    async fn db_quit_checkpoint_truncates_wal_and_survives_reopen(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            parchora_ui::init(cx);
            init_dock_globals(cx);
            let root = temp_store_root("checkpoint-exit");
            init_app_store_at(cx, &root);
            let db = app_db(cx).expect("存储根在场");
            insert_clip(
                &db,
                &Clip {
                    id: new_uuid(),
                    content_type: ContentType::PlainText,
                    text_content: Some("退出前落帧".into()),
                    content_hash: "checkpoint-exit-1".into(),
                    source_app: None,
                    size: 18,
                    is_pinned: false,
                    blob_rel_path: None,
                    thumb_rel_path: None,
                    edited_at: None,
                    created_at: iso_utc(walltime::now_epoch()),
                },
            )
            .unwrap();
            quit_checkpoint(cx);
            let wal = root.join("data").join("parchora.db-wal");
            assert_eq!(
                std::fs::metadata(&wal).expect("-wal 在场").len(),
                0,
                "TRUNCATE 复零字节"
            );
            let (busy, log, ck) = db.checkpoint_truncate().unwrap();
            assert_eq!((busy, log, ck), (0, 0, 0), "复检点空转（已归零）");
            simulate_restart(cx, &root);
            let db = app_db(cx).expect("重开成功");
            assert_eq!(list_clips(&db).unwrap().len(), 1, "交接重开数据仍在");
        });
    }

    /// 跨会话待办持久化 + identifier 续接（A 档缺陷二根修端到端）：创建 → DB →
    /// 重启仿真 → 回填重现 → 续建 TASK-2（序号链不重复不断裂）。
    #[gpui::test]
    async fn task_persistence_and_identifier_continuity_across_restart(
        cx: &mut gpui::TestAppContext,
    ) {
        cx.update(|cx| {
            parchora_ui::init(cx);
            let root = temp_store_root("task-restart");
            init_dock_globals(cx);
            init_app_store_at(cx, &root);
            open_panel_docked(cx);
            let panel = cx.global::<DockHandles>().panel.unwrap();
            panel
                .update(cx, |p, _w, cx| {
                    p.ingest_created_task("首任务".into(), 0, 0, 0, cx)
                })
                .expect("面板在场创建");
            let db = app_db(cx).expect("存储根在场");
            assert_eq!(list_todos(&db).unwrap().len(), 1, "创建即落库");

            simulate_restart(cx, &root);
            open_panel_docked(cx);
            let panel = cx.global::<DockHandles>().panel.unwrap();
            let (titles, next_seq) = panel
                .update(cx, |p, _w, cx| p.board_rows_snapshot(cx))
                .expect("面板可读");
            assert_eq!(titles, vec!["首任务".to_string()], "重启后待办重现");
            assert_eq!(next_seq, 2, "序号续接 max(identifier)+1");
            // 续建 → TASK-2（无重复）；DB 行数 2。
            panel
                .update(cx, |p, _w, cx| {
                    p.ingest_created_task("续任务".into(), 1, 0, 0, cx)
                })
                .expect("续建");
            let db = app_db(cx).unwrap();
            let todos = list_todos(&db).unwrap();
            assert_eq!(todos.len(), 2);
            assert_eq!(todos[1].identifier, "TASK-2", "续接不重号");
            assert_eq!(todos[1].schedule_bucket, ScheduleBucket::Tomorrow);
        });
    }

    /// 右键状态/排期改值落库 + 拾记右键删除落库（TASK-300 逆向动作链）。
    #[gpui::test]
    async fn right_click_mutations_persist_to_db(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            parchora_ui::init(cx);
            let root = temp_store_root("right-click");
            init_dock_globals(cx);
            init_app_store_at(cx, &root);
            open_panel_docked(cx);
            let panel = cx.global::<DockHandles>().panel.unwrap();
            panel
                .update(cx, |p, _w, cx| {
                    p.ingest_created_task("任务甲".into(), 0, 0, 0, cx)
                })
                .expect("创建");
            panel
                .update(cx, |p, _w, cx| {
                    p.board().update(cx, |v, cx| {
                        v.status_set(0, 2, "右键", cx); // 已搁置
                        v.bucket_set(0, 3, "右键", cx); // 以后
                    })
                })
                .expect("右键改值");

            // 右键改值已落库（事件经 sink 同步事务写）。
            let db = app_db(cx).unwrap();
            let todos = list_todos(&db).unwrap();
            assert_eq!(todos[0].status, TaskStatus::Backlog);
            assert_eq!(todos[0].schedule_bucket, ScheduleBucket::Later);

            // 拾记行：携带 db 身份的行删除 → DB 删行。
            clipboard_tick_with(cx, Some("待右键删除".into()));
            panel
                .update(cx, |p, _w, cx| {
                    p.capture().update(cx, |v, cx| v.ctx_delete(0, cx))
                })
                .expect("右键删除");
            let db = app_db(cx).unwrap();
            assert_eq!(
                list_clips(&db).unwrap().len(),
                0,
                "UI 删除同拍 DB 删除（持久化身份行）"
            );
        });
    }

    /// 待办右键「删除」（DEC-195 · TASK-300 逆向动作）：UI 行移除＋DB 行删除
    /// 同拍落库；重启回填不再复现。
    #[gpui::test]
    async fn task_delete_event_deletes_db_row(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            parchora_ui::init(cx);
            let root = temp_store_root("task-delete");
            init_dock_globals(cx);
            init_app_store_at(cx, &root);
            open_panel_docked(cx);
            let panel = cx.global::<DockHandles>().panel.unwrap();
            panel
                .update(cx, |p, _w, cx| {
                    p.ingest_created_task("任务甲".into(), 0, 0, 0, cx)
                })
                .expect("创建");
            panel
                .update(cx, |p, _w, cx| {
                    p.board().update(cx, |v, cx| v.ctx_delete(0, cx))
                })
                .expect("右键删除");
            let db = app_db(cx).unwrap();
            assert!(list_todos(&db).unwrap().is_empty(), "待办删除同拍 DB 删行");
            // 重启后不回填（行已删）。
            simulate_restart(cx, &root);
            open_panel_docked(cx);
            assert_eq!(list_todos(&db).unwrap().len(), 0, "删除后重启不回填已删行");
        });
    }

    /// 回收站真删/清空（DEC-195 · 用户语义收口）：首页删除→.trash 在册→页里
    /// 扫描注入；单件永久删除/清空仅物理删回收站内文件；取消失误路径不吞。
    #[gpui::test]
    async fn trash_scan_purge_and_empty_physical_delete(cx: &mut gpui::TestAppContext) {
        let container = std::env::temp_dir().join(format!(
            "parchora-trash-purge-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let root = container.join("files");
        std::fs::create_dir_all(&root).unwrap();
        let _ = parchora_files::write_note(&root, "甲", "一篇").unwrap();
        let _ = parchora_files::write_note(&root, "乙", "二篇").unwrap();
        cx.update(|cx| {
            init_dock_globals(cx);
            // 首页删除（真移 .trash）→ 扫描面在册；根面消失。
            trash_note_at(&root, "甲", cx).unwrap();
            let trash = container.join(".trash");
            let rows = scan_trash_notes_at(&trash);
            assert_eq!(rows.len(), 1, "回收站扫描在册（首页删除后）");
            assert_eq!(rows[0].title, "甲");
            assert!(!root.join("甲.md").exists(), "根面消失");
            // 宿主动作：单件永久删除。
            purge_trash_note_at(&root, "甲", cx);
            assert!(!trash.join("甲.md").exists(), "永久删除物理删行");
            assert!(scan_trash_notes_at(&trash).is_empty());
            // 缺源重删：不吞（stderr 自证，生产链；此处断言不 panic 即可）。
            purge_trash_note_at(&root, "甲", cx);
            // 宿主动作：清空（乙再入回收站 → 清空后全部物理删除）。
            trash_note_at(&root, "乙", cx).unwrap();
            empty_trash_at(&root, cx);
            assert!(!trash.join("乙.md").exists(), "清空物理删除全部");
            assert!(scan_trash_notes_at(&trash).is_empty());
        });
        std::fs::remove_dir_all(&container).unwrap();
    }

    /// 写失败重试（A 档「写失败重试」）：无活跃根时捕获不丢（内存 + 队列），
    /// 存储恢复后队列先行补写、DB 全量对齐。
    #[gpui::test]
    async fn capture_retry_queue_recovers_after_store_return(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            let root = temp_store_root("retry");
            init_dock_globals(cx);
            // AppStore 在场但无活跃根（开根失败同构）→ 捕获期写全败（写失败注入）。
            cx.set_global(AppStore::default());
            clipboard_tick_with(cx, Some("失败期甲".into()));
            clipboard_tick_with(cx, Some("失败期乙".into()));
            {
                let cap = cx.global::<ClipCapture>();
                assert_eq!(cap.db_retry.len(), 2, "写失败两载荷入重试队列");
            }
            // 存储恢复（switch_to 开真根）。
            cx.global::<AppStore>().store.switch_to(&root).unwrap();
            // 下一拍捕获：先 drains 重试队列，再落新条目。
            clipboard_tick_with(cx, Some("恢复期丙".into()));
            let db = app_db(cx).unwrap();
            let clips = list_clips(&db).unwrap();
            let titles: Vec<String> = clips
                .iter()
                .map(|c| c.text_content.clone().unwrap())
                .collect();
            assert!(
                titles.contains(&"失败期甲".to_string()),
                "失败期条目补写成功"
            );
            assert!(titles.contains(&"失败期乙".to_string()));
            assert!(titles.contains(&"恢复期丙".to_string()));
            assert_eq!(clips.len(), 3, "队列无重复补写");
            assert!(cx.global::<ClipCapture>().db_retry.is_empty(), "队列排空");
        });
    }

    /// 跨重启判重播种（A 档）：重启后窗内（10s）同内容再捕获不重入库；窗外
    /// 历史不拦截。
    #[gpui::test]
    async fn dedupe_seeded_from_db_across_restart(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            let root = temp_store_root("dedupe-seed");
            init_dock_globals(cx);
            init_app_store_at(cx, &root);
            open_panel_docked(cx);
            clipboard_tick_with(cx, Some("同内容甲".into()));
            // DB 行已落（create_at = now）。
            // 重启仿真：recent LRU 复位 → 种子回填（init_app_store_at 内承载）。
            simulate_restart(cx, &root);
            open_panel_docked(cx);
            // 窗内同内容再捕获：判重拒收（DB 行数不增长）。
            clipboard_tick_with(cx, Some("同内容甲".into()));
            let db = app_db(cx).unwrap();
            assert_eq!(list_clips(&db).unwrap().len(), 1, "重启后窗内不重入库");
        });
    }

    /// 回填行时回拨（A 档「Instant 年龄语义修复」）：1 天保留档下，昨日捕获
    /// 行（墙钟锚定）重启后立即被存活清理，不再以进程内新建时刻起算。
    /// 注：面板回填在 open_panel_docked 内 seed + clip_sweep 承载。
    #[gpui::test]
    async fn backfilled_rows_sweep_by_true_age(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            let root = temp_store_root("backfill-age");
            // 直接经 DB 写一条“昨日”行（绕过今日捕获），验证回填后年龄被清。
            let db = Db::open(&root).unwrap();
            let yesterday_wall = walltime::now_epoch() - 90_000; // > 1 天
            let clip = Clip {
                id: new_uuid(),
                content_type: ContentType::PlainText,
                text_content: Some("昨日旧闻".into()),
                content_hash: "seed-old".into(),
                source_app: None,
                size: 12,
                is_pinned: false,
                blob_rel_path: None,
                thumb_rel_path: None,
                edited_at: None,
                created_at: iso_utc(yesterday_wall),
            };
            insert_clip(&db, &clip).unwrap();
            drop(db);

            init_dock_globals(cx);
            init_app_store_at(cx, &root);
            open_panel_docked(cx);
            // 启动对账（init_app_store_at → prune_db_retention）已删未 Pin 超阈行。
            let db = app_db(cx).unwrap();
            assert!(
                list_clips(&db).unwrap().is_empty(),
                "超阈未 Pin 行被启动对账清理（不回填）"
            );
        });
    }

    /// 队列动作宿主 mutator（DEC-207 · A1 队列接线）：QueueHost 单一事实源
    /// 变更后镜像全量回推面板（FIFO/LIFO 消费序视图 · 停止/清空终态清空），
    /// 入队不自动开层（入口标记呈现即可见面）。
    #[gpui::test]
    async fn queue_actions_reach_host_and_mirror_panel(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            init_dock_globals(cx);
            open_panel_docked(cx);
            queue_event(cx, &QueueAction::Enqueue("甲".into()));
            queue_event(cx, &QueueAction::Enqueue("乙".into()));
            let panel = cx.global::<DockHandles>().panel.unwrap();
            let mirror = panel
                .update(cx, |p, _w, cx| p.queue_texts(cx))
                .expect("镜像回推");
            assert_eq!(mirror, vec!["甲", "乙"], "FIFO 镜像＝入队序");
            // 切换 LIFO：整列倒置首耗最新（SCN-QUEUE-ORDER-LIFO）。
            queue_event(cx, &QueueAction::ToggleOrder);
            let mirror = panel
                .update(cx, |p, _w, cx| p.queue_texts(cx))
                .expect("镜像回推");
            assert_eq!(mirror, vec!["乙", "甲"], "LIFO 镜像＝倒序首耗最新");
            // Stop 终态：全局与镜像同空（消费序模式随 QueueHost 保留不再断言——
            // 清位面）。
            queue_event(cx, &QueueAction::Stop);
            let mirror = panel
                .update(cx, |p, _w, cx| p.queue_texts(cx))
                .expect("镜像回推");
            assert!(mirror.is_empty(), "停止后镜像无残留");
            assert!(cx.global::<QueueHost>().queue.is_empty(), "队列不持条目");
        });
    }

    /// 入队快照隔离（SCN-QUEUE-SNAPSHOT-ISOLATED · 宿主面）：入队后删除拾记
    /// 源行（DB 同拍删）不波及队列条目（队列持独立快照）。
    #[gpui::test]
    async fn queue_entry_survives_clip_delete(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            let root = temp_store_root("queue-snapshot");
            init_dock_globals(cx);
            init_app_store_at(cx, &root);
            open_panel_docked(cx);
            // 真实捕获＋入队（快照文本入 QueueHost）。
            clipboard_tick_with(cx, Some("隔离验证文本".into()));
            queue_event(cx, &QueueAction::Enqueue("隔离验证文本".into()));
            // 删源行（DB 删行同拍）。
            let db = app_db(cx).unwrap();
            let clip = &list_clips(&db).unwrap()[0];
            let id = clip.id.to_string();
            drop(db);
            clip_delete_from_db(cx, &id);
            // 队列快照不受源删除影响。
            assert_eq!(
                cx.global::<QueueHost>().queue.view()[0].text,
                "隔离验证文本",
                "源行删除不波及队列快照"
            );
        });
    }

    /// 队列引擎接线面（TASK-301 · cfg test 桥短路态）：queue_tick 拍——桥失
    /// 败不自伤（Idle 保持 · 队列本体不丢 · 镜像不清）；stop 全链本体终态。
    #[gpui::test]
    async fn queue_tick_noop_bridge_self_heals(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            init_dock_globals(cx);
            queue_event(cx, &QueueAction::Enqueue("甲".into()));
            // 桥失败态：queue_tick reconcile 不持假板（保持 Idle · 队列保留）。
            queue_tick(cx);
            let host = cx.global::<QueueHost>();
            assert!(host.engine.holding().is_none(), "桥失败不假持板");
            assert_eq!(host.queue.view().len(), 1, "本体不丢（下拍重试承载）");
            // 停止全链：无持板状态 stop → 本体仍终态清空（fixup 静默不写板）。
            queue_stop_full(cx, &QueueAction::Stop);
            assert!(cx.global::<QueueHost>().queue.is_empty(), "停止全链终态");
        });
    }

    /// 队列持久化全链（DEC-212）：入队即落库（ord 单调递增）→ 消耗删行
    ///（uuid 映射查查）→ 重启仿真恢复（内容/序/next_ord/映射重建）→
    /// 恢复后续入队接序 → Clear 终态全表清。
    #[gpui::test]
    async fn queue_persistence_enqueue_consume_reseed_clear(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            parchora_ui::init(cx);
            let root = temp_store_root("queue-persist");
            init_dock_globals(cx);
            init_app_store_at(cx, &root);
            queue_event(cx, &QueueAction::Enqueue("甲".into()));
            queue_event(cx, &QueueAction::Enqueue("乙".into()));
            queue_event(cx, &QueueAction::Enqueue("丙".into()));
            // 入队即落库：三行就位、ord 单调（DEC-212 · 每动作事务即写）。
            let db = app_db(cx).expect("存储根在场");
            let rows = list_queue_entries(&db).unwrap();
            assert_eq!(
                rows.iter().map(|r| r.text.as_str()).collect::<Vec<_>>(),
                vec!["甲", "乙", "丙"],
                "入队即落库，读回＝ord 升序（入队序）"
            );
            assert_eq!(
                rows.iter().map(|r| r.ord).collect::<Vec<_>>(),
                vec![1, 2, 3],
                "ord 会话单调递增"
            );
            drop(db);
            // 消耗流（DEC-212）：宿主态摘除 + 删行（queue_tick 借还块外同径）。
            let (first_id, first_uuid) = {
                let host = cx.global_mut::<QueueHost>();
                let id = host.queue.view()[0].id;
                host.queue.remove_by_id(id); // 内存面（引擎兑现消耗同拍已摘）
                let uuid = host.uuid_by_id.remove(&id).expect("uuid 映射在册");
                (id, uuid)
            };
            queue_delete_rows(cx, std::slice::from_ref(&first_uuid));
            assert_eq!(first_id, 1);
            let db = app_db(cx).unwrap();
            assert!(
                list_queue_entries(&db)
                    .unwrap()
                    .iter()
                    .all(|r| r.uuid != first_uuid),
                "消耗删行生效"
            );
            drop(db);

            // 重启仿真（连接重开 + 实体全清）：恢复链重建队列与承接映射。
            simulate_restart(cx, &root);
            {
                let host = cx.global::<QueueHost>();
                assert_eq!(
                    host.queue
                        .view()
                        .iter()
                        .map(|e| e.text.as_str())
                        .collect::<Vec<_>>(),
                    vec!["乙", "丙"],
                    "FIFO 恢复＝ord 升序（入队序权威）"
                );
                assert_eq!(host.next_ord, 4, "next_ord = max(ord)+1 播种");
                assert_eq!(host.uuid_by_id.len(), 2, "uuid 映射随行重建");
            }
            // 恢复后续入队接序：ord 接 max+1（不复位不重号）。
            queue_event(cx, &QueueAction::Enqueue("丁".into()));
            let db = app_db(cx).unwrap();
            let rows = list_queue_entries(&db).unwrap();
            assert_eq!(rows.len(), 3);
            assert_eq!(rows[2].ord, 4, "恢复后 next_ord 接续");
            drop(db);

            // Clear 终态（DEC-212 全表清唯一收口）：DB 与映射同空。
            queue_event(cx, &QueueAction::Clear);
            let db = app_db(cx).unwrap();
            assert!(list_queue_entries(&db).unwrap().is_empty(), "清空落库面");
            drop(db);
            assert!(cx.global::<QueueHost>().uuid_by_id.is_empty(), "映射终态空");
        });
    }

    /// 删行 miss 幂等（DEC-212 遗留边界）：uuid 未在表（写失败未登记面）不报错。
    #[gpui::test]
    async fn queue_delete_rows_miss_is_noop(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            let root = temp_store_root("queue-persist-miss");
            init_dock_globals(cx);
            init_app_store_at(cx, &root);
            queue_delete_rows(cx, &["无此行".into()]);
            let db = app_db(cx).unwrap();
            assert!(list_queue_entries(&db).unwrap().is_empty(), "miss 无副作用");
        });
    }

    /// LIFO 模式持久化链（DEC-212 · 设置档）：toggle 后模式在档（queue_order=
    /// round-trip 面已测）；重启后按 saved 播种 LIFO（view 倒置）——
    /// init_dock_globals_with 注入面直填承载（cfg(test) load noop）。
    #[gpui::test]
    async fn queue_lifo_mode_seeds_from_saved_settings(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            parchora_ui::init(cx);
            let root = temp_store_root("queue-lifo");
            init_dock_globals(cx);
            init_app_store_at(cx, &root);
            queue_event(cx, &QueueAction::Enqueue("甲".into()));
            queue_event(cx, &QueueAction::Enqueue("乙".into()));
            queue_event(cx, &QueueAction::ToggleOrder);
            assert_eq!(
                cx.global::<QueueHost>().queue.order(),
                QueueOrder::Lifo,
                "切换后 LIFO"
            );
            // 档值面 round-trip：lifo 字面量往返（save 链 prod 直写全局 · 测试 noop）。
            let saved_mode = parse_settings_body(&format!(
                "{QUEUE_ORDER_KEY}{}\n",
                queue_order_str(QueueOrder::Lifo)
            ))
            .queue_order;
            assert_eq!(saved_mode, Some(QueueOrder::Lifo));
            // 重启仿真（档面注入 LIFO）：模式随档播种 + 队列恢复同链。
            cx.global_mut::<DockHandles>().panel = None;
            cx.global_mut::<DockHandles>().panel_entity = None;
            cx.global_mut::<DockHandles>().panel_view = None;
            init_dock_globals_with(
                cx,
                AppSettings {
                    queue_order: saved_mode,
                    ..AppSettings::default()
                },
            );
            init_app_store_at(cx, &root);
            let host = cx.global::<QueueHost>();
            assert_eq!(host.queue.order(), QueueOrder::Lifo, "重启后模式随档播种");
            assert_eq!(
                host.queue
                    .view()
                    .iter()
                    .map(|e| e.text.as_str())
                    .collect::<Vec<_>>(),
                vec!["乙", "甲"],
                "LIFO 恢复视图＝入队倒序（首耗最新）"
            );
            assert_eq!(host.uuid_by_id.len(), 2, "uuid 映射随行重建");
        });
    }

    // ═══ TASK-405/716 提示桥全链 mock（trait mock 授权三态/点击路由/节流）═══

    /// 测试桥（trait mock · TASK-405 契约「trait 化使通知可 mock」）：&self+
    /// 内部可变记录面（RefCell · 同 ui queued sink 先例）。
    struct MockQueueBridge {
        authorized: bool,
        click_pending: bool,
        auth_calls: std::cell::Cell<usize>,
        dock_calls: std::cell::Cell<usize>,
        cancel_calls: std::cell::Cell<usize>,
        panel_bounces: std::cell::Cell<usize>,
        routes: std::cell::Cell<usize>,
        sends: std::cell::RefCell<Vec<String>>,
    }

    impl MockQueueBridge {
        fn new(authorized: bool) -> Self {
            Self {
                authorized,
                click_pending: false,
                auth_calls: Default::default(),
                dock_calls: Default::default(),
                cancel_calls: Default::default(),
                panel_bounces: Default::default(),
                routes: Default::default(),
                sends: Default::default(),
            }
        }
    }

    impl QueueBridge for MockQueueBridge {
        fn authorize(&self) {
            self.auth_calls.set(self.auth_calls.get() + 1);

            // mock：授权请求即视为已决（回写测试字段不通用——直接置 authorized 是
            // &self 不可变字段；此处不回写，授权态由测试构造时定）。真回调异
            // 步语义由 platform 层原子槽承载（生产）。
        }
        fn authorized(&self) -> bool {
            self.authorized
        }
        fn dock_attention(&self) -> Option<u64> {
            self.dock_calls.set(self.dock_calls.get() + 1);
            Some(100 + self.dock_calls.get() as u64)
        }
        fn cancel_attention(&self, _id: u64) {
            self.cancel_calls.set(self.cancel_calls.get() + 1);
        }
        fn panel_bounce(&self, cx: &mut App) {
            self.panel_bounces.set(self.panel_bounces.get() + 1);
            push_queue_bounce(cx);
        }
        fn send_now(&self, _title: &str, body: &str) {
            self.sends.borrow_mut().push(body.to_string());
        }
        fn click_take(&self) -> bool {
            self.click_pending
        }
        fn route_click(&self, _cx: &mut App) {
            self.routes.set(self.routes.get() + 1);
        }
    }

    /// 首投全链（TASK-405 mock 三态 · §11.2 首行）：置授权 → Placed 事件 → Dock
    /// 弹一弹（id 登记 notifier）＋通知（标题+条数）＋授权请求仅首次。
    #[gpui::test]
    async fn queue_notifier_chain_placed_full_mock(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            init_dock_globals(cx);
            let mut bridge = MockQueueBridge::new(true);
            queue_notify_events_with(
                cx,
                vec![
                    QueueEventKind::Placed { remaining: 2 },
                    QueueEventKind::Placed { remaining: 2 },
                ],
                &mut bridge,
            );
            assert_eq!(bridge.auth_calls.get(), 1, "授权请求仅首次");
            assert_eq!(bridge.dock_calls.get(), 2, "每事件一弹");
            assert_eq!(bridge.sends.borrow().len(), 2, "每投递一通知");
            assert_eq!(bridge.sends.borrow()[0], "队列开始投递，待粘贴 2 条");
            assert_eq!(
                cx.global::<QueueHost>().notifier.last_attention_id(),
                Some(102),
                "末次弹跳 id 登记（点击撤回承载）"
            );
        });
    }

    /// 未决/拒绝授权静默（§11.2 授权语义）：send 零调用；动效照常（弹一弹不
    /// 受授权影响）。
    #[gpui::test]
    async fn queue_notifier_chain_unauthorized_silent_mock(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            init_dock_globals(cx);
            let mut bridge = MockQueueBridge::new(false);
            queue_notify_events_with(
                cx,
                vec![
                    QueueEventKind::Placed { remaining: 1 },
                    QueueEventKind::Consumed { remaining: 0 },
                    QueueEventKind::Stopped { released: 1 },
                ],
                &mut bridge,
            );
            assert_eq!(bridge.sends.borrow().len(), 0, "未授权全静默");
            assert!(bridge.dock_calls.get() >= 2, "弹一弹不受影响");
            assert_eq!(bridge.panel_bounces.get(), 1, "消耗动效不受影响");
        });
    }

    /// 点击路由排水（§11.2 点击行为）：点击槽 → 路由＋未决注意请求撤回；幂等。
    #[gpui::test]
    async fn queue_notifier_chain_click_route_mock(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            init_dock_globals(cx);
            cx.global_mut::<QueueHost>().notifier.note_attention(7);
            let mut bridge = MockQueueBridge::new(true);
            bridge.click_pending = true;
            queue_notify_events_with(cx, Vec::new(), &mut bridge);
            assert_eq!(bridge.routes.get(), 1, "点击路由一次");
            assert_eq!(bridge.cancel_calls.get(), 1, "未决弹跳撤回");
            assert_eq!(
                cx.global::<QueueHost>().notifier.last_attention_id(),
                None,
                "id 取走"
            );
            // 幂等：点击已排水，再拍无路由。
            bridge.click_pending = false;
            queue_notify_events_with(cx, Vec::new(), &mut bridge);
            assert_eq!(bridge.routes.get(), 1, "排水后无二次路由");
        });
    }
}
