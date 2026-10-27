//! 面板触发位置与停靠（DEC-67/76/78：收缩态 = 主面板整窗关闭全隐、进程保持；
//! 拖动中不限位，释放后由 app 层 snap_watch 一次性吸附重泊）。
//!
//! 7 向触发位置（底部/左侧/右侧/左上角/右上角/左下角/右下角 · DEC-84：「顶部」
//! 无对应触发位（DEC-84/86/87）；仅 macOS；Win/Linux 后续版本）：主面板
//! 位置贴缘/角停泊于**可用区域**（系统排除菜单栏/Dock 后的可用区，platform
//! NSScreen visibleFrame 为真几何源）；可用性：底部（Dock 在左/右时）、左侧/右侧
//! （Dock 在底/另一侧时），四角恒可用；Dock 占用同侧冲突自动切换（左→右、右→
//! 左、底→右 · `resolve_conflict`）。移出隐藏（REQ-PANEL-HIDE）与 Pin 屏蔽
//! （REQ-PANEL-PIN-BLOCKS-HIDE）经 PanelView→app 承载，收缩态全隐；唤出 =
//! 可用区域边缘 12pt 触发热区（边族整缘 · 角族 4px 方区 · DEC-222）轮询判定
//! （platform::cursor_position 读探针 +
//! trigger_zone_bounds 几何 · app 层 reveal_watch · DEC-79；窗口级 hover 载体真
//! 机实证不可行：非 key 非激活 NSPanel 收不到 mouseMoved（events.rs 不映射
//! NSMouseEntered + PopUp 分支无 setAcceptsMouseMovedEvents），不可感知的窗仅
//! 余吞点击与额外窗体副作用）。
//! platform 接线轮真语义（DEC-81）：CGEventTap 只听监听光标源（授权
//! preflight 显式探测 · 未授权保留读探针 poll 降级）、50pt 排除带
//! （`in_exclusion`）；内缘不禁触（DEC-83）；0.5s 二次校验 /
//! 激活延迟可配置（SCN-PANEL-ACTIVATION-DELAY）/ row-peek 失活期仍 defer
//! （DEC-67 清单）。底部触发将主程序与内容详情面板全部宽高互换
//! （320×H → H×320 · 480×H2 → H2×480，最小尺寸同步互换）。
//!
//! Peek 内容详情窗同侧邻接（DEC-76）：左族 → 主面板右邻、右族 → 左邻、底 → 上邻
//! （间距 ADJACENT_GAP=16，沿停靠轴；中轴与主面板对齐）。多显示器以
//! 「应用所在屏幕」为参照（DockPrefs.active_display 探针：主程序被拖曳至的屏），
//! 停靠/触发/越界钳制均以该屏**可用区域**为参照（DEC-84）。
//!
//! 缝侧邻接宽限带（DEC-95 · 副屏邻接
//! 缝侧走位）：光标越出触发缘共享缝深入邻接屏 ≤ 20pt（SEAM_GRACE_BAND）→ app 层
//! 挂起收起判定 0.6s（SEAM_GRACE_TICKS=12 × 0.05s，长驻超限后恢复常规两拍），
//! 带外/回域宽限计数清零；缝边由热区 rect 推导（热区 strip 外缘即共享缝，几何
//! 单一来源，不引入 screen 参数）；点击外隐藏路径不含宽限（点击 = 显式动作）。
//! 判定为纯函数（in_seam_grace_band），可独立单测。
//!
//! 偏差登记（DEC-67 · DEC-79）：REQ-PANEL-EDGE-TRIGGER 的
//! 全局鼠标监听（CGEventTap）/ 上下 50pt 排除带 / 0.5s 二次校验为
//! platform 接线轮真语义；触发阈值与唤出几何由演示宿主以「光标位置轮询」承载
//! （platform::cursor_position 免权限只读探针）；
//! 编程式
//! 移窗 API 不存在（vendor 无 set_frame/set_position）→ 位置切换 = 关窗重开。
//! 屏坐标系：应用所在屏左上原点 · y 向下（vendor PlatformDisplay::bounds origin 恒
//! (0,0)，display.rs:117-126——副屏真原点 defer platform 轮）。几何全部为纯函数
//! （不依赖 gpui），可独立单测。

/// 主面板与内容详情（Peek）窗邻接间隙 16pt（DEC-76：内容详情窗口须紧贴主程序
/// 窗口，两者间距固定为 16px）。
pub const ADJACENT_GAP: f32 = 16.0;

/// 边缘触发热区厚度 12pt（REQ-PANEL-EDGE-TRIGGER 阈值 · DEC-100：扩大有效触
/// 发范围——轮询拍距 0.05s 下窄带高速穿行会两拍间漏检，事件锁存（app 层
/// ZONE_HIT）补漏后带宽仍取宽值双保险；轮询判定域宽/高即取此值）。
/// 边族（左/右/底）整缘取此值；四角不取此值——角族独立按 `CORNER_SZ`
/// 4px 方形角区承载（DEC-222 用户指令）。
pub const TRIGGER_T: f32 = 12.0;

/// 四角触发角区边长 4px（DEC-222 · 用户指令「触发角尺寸限制在 4 像素范围内」）：
/// 角族热区 = 贴可用区域对应角的 4×4 方形（替换原 12pt 厚×半缘条带 · DEC-100
/// 口径）。
pub const CORNER_SZ: f32 = 4.0;

/// 触发位置（DEC-84：7 向，「顶部」无对应触发位（DEC-84/86/87）；仅 macOS，
/// Win/Linux 后续版本；呈现序 = 设置面板 pill 序 = 可用性描述序）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TriggerPosition {
    /// 底部（吸附底缘 · 全部面板宽高互换 · Dock 在左/右时可用）
    Bottom,
    /// 左侧（吸附左缘 · Dock 在底/右时可用）
    Left,
    /// 右侧（吸附右缘 · 默认；Dock 在底/左时可用 · DEC-76）
    #[default]
    Right,
    /// 左上角（吸附左缘上半 + 双缘角锚 · 恒可用）
    TopLeft,
    /// 右上角（吸附右缘上半 + 双缘角锚 · 恒可用）
    TopRight,
    /// 左下角（吸附左缘下半 + 双缘角锚 · 恒可用）
    BottomLeft,
    /// 右下角（吸附右缘下半 + 双缘角锚 · 恒可用）
    BottomRight,
}

impl TriggerPosition {
    /// 设置面板呈现序（底部/左侧/右侧/四角 · 呈现序即 pill 序）。
    pub const ALL: [Self; 7] = [
        Self::Bottom,
        Self::Left,
        Self::Right,
        Self::TopLeft,
        Self::TopRight,
        Self::BottomLeft,
        Self::BottomRight,
    ];

    /// 设置面板/CLI 共用中文标签。
    pub const fn label(self) -> &'static str {
        match self {
            Self::Bottom => "底部",
            Self::Left => "左侧",
            Self::Right => "右侧",
            Self::TopLeft => "左上角",
            Self::TopRight => "右上角",
            Self::BottomLeft => "左下角",
            Self::BottomRight => "右下角",
        }
    }

    /// 中文标签解析（CLI 位置演示位）。
    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|p| p.label() == s)
    }

    /// 底部触发：全部面板宽高互换（位置名「顶部」无对应触发位 · DEC-84/87）。
    pub const fn swaps_dims(self) -> bool {
        matches!(self, Self::Bottom)
    }

    /// 左缘族（左侧与含「左」的角）。
    pub const fn docks_left(self) -> bool {
        matches!(self, Self::Left | Self::TopLeft | Self::BottomLeft)
    }

    /// 右缘族（右侧与含「右」的角）。
    pub const fn docks_right(self) -> bool {
        matches!(self, Self::Right | Self::TopRight | Self::BottomRight)
    }
}

/// 触发位置会话态全局（设置面板单写 · app 装配读取）；trigger 经 settings env
/// 即刻持久化、宿主启动时恢复（DEC-85），Pin 等其余会话态重启回默认（DEC-67）。
pub struct DockPrefs {
    /// 当前触发位置（7 向 · 与 Dock 占用冲突已解析后落值 · DEC-84）
    pub trigger: TriggerPosition,
    /// 应用所在屏幕（多屏参照 · DEC-76：主程序被拖曳至的屏；宿主关收面板时
    /// 经 window.display 探针登记，下次开窗按该屏取几何）。None = 未测定（按主屏
    /// 处理）；副屏绝对定位（真原点）属 platform 轮。
    pub active_display: Option<gpui::DisplayId>,
}

impl gpui::Global for DockPrefs {}

/// Dock 占用态全局（DEC-84 · REQ-PANEL-TRIGGER-POSITION 可用性）：app 层在启动与
/// 屏帧变化时落值（is_edge_occupied 实测）；设置面板 pill 按此置灰（被占缘不可选）。
/// 未初始化（纯 ui 语境/无宿主演示）Default 全 false = 全可用兜底。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DockAvailability {
    pub occupied_left: bool,
    pub occupied_right: bool,
    pub occupied_bottom: bool,
}

impl gpui::Global for DockAvailability {}

/// 演示宿主屏 rect (x, y, w, h)——top-left 原点 · y 向下（DEC-67 坐标口径）。
/// DEC-84 起，传入停靠/触发/邻接几何的 screen 参数一律为**可用区域**
/// rect（系统排除菜单栏/Dock 后的可用区 · platform NSScreen visibleFrame 为真几
/// 何源；全帧 rect 另行参与占用判定，见 is_edge_occupied）。
pub type ScreenRect = (f32, f32, f32, f32);

/// 面板停泊 rect：(停靠锚点 + 宽高互换后尺寸)。screen 参数 = **可用区域** rect
/// （DEC-84）。左右缘 = x 贴缘 · y 垂直居中；底部 = 宽高互换 · y 贴底缘 · x 水
/// 平居中（D-05）；四角 = 可用区域双缘角锚（不互换，互换条目仅限底部全缘）。
pub fn panel_bounds(
    pos: TriggerPosition,
    screen: ScreenRect,
    base: (f32, f32),
) -> (f32, f32, f32, f32) {
    let (sx, sy, sw, sh) = screen;
    let (bw, bh) = base;
    match pos {
        TriggerPosition::Left => (sx, sy + (sh - bh) / 2.0, bw, bh),
        TriggerPosition::Right => (sx + sw - bw, sy + (sh - bh) / 2.0, bw, bh),
        TriggerPosition::Bottom => {
            let (w, h) = swap_dims_if_horizontal(pos, base);
            (sx + (sw - w) / 2.0, sy + sh - h, w, h)
        }
        TriggerPosition::TopLeft => (sx, sy, bw, bh),
        TriggerPosition::TopRight => (sx + sw - bw, sy, bw, bh),
        TriggerPosition::BottomLeft => (sx, sy + sh - bh, bw, bh),
        TriggerPosition::BottomRight => (sx + sw - bw, sy + sh - bh, bw, bh),
    }
}

/// 底部触发尺寸互换（窗口尺寸与最小尺寸同口径；其余位置原样返回）。
pub fn swap_dims_if_horizontal(pos: TriggerPosition, dims: (f32, f32)) -> (f32, f32) {
    if pos.swaps_dims() {
        (dims.1, dims.0)
    } else {
        dims
    }
}

/// Peek 定位 rect：主面板同侧邻接（DEC-76 · 内容详情窗口紧贴主程序窗口，间距
/// 固定 16px · REQ-PREVIEW-HOVER-TARGET / SCN-PREVIEW-POSITION-FIXED）：
/// 左族 → 主面板右邻；右族 → 左邻；底 → 上邻（间距沿停靠轴 ADJACENT_GAP；
/// 垂直/水平中轴与主面板对齐，DEC-67 中轴居中语义）。锚 panel_rect =
/// 主面板实界（调用方经 window.bounds 取，跨屏拖曳后仍紧贴），无实界时调用方以
/// panel_bounds 推定位兜底。Peek 自身不做宽高互换——互换语义仅限面板族，调用方
/// 先经 swap_dims_if_horizontal 换好 base 再传入。_screen 参数 = 可用区域 rect
/// （DEC-84）。
/// 越界兜底：结果 x/y 钳制进可用区域（极端小屏下间隙塌缩让位完整呈现——默认尺寸
/// 在正常屏高下不会触发）。
pub fn peek_bounds(
    pos: TriggerPosition,
    screen: ScreenRect,
    panel_rect: (f32, f32, f32, f32),
    base: (f32, f32),
) -> (f32, f32, f32, f32) {
    let (sx, sy, sw, sh) = screen;
    let (ax, ay, aw, ah) = panel_rect;
    let (bw, bh) = base;
    let gap = ADJACENT_GAP;
    let (mut x, mut y, w, h) = match pos {
        TriggerPosition::Left | TriggerPosition::TopLeft | TriggerPosition::BottomLeft => {
            (ax + aw + gap, ay + (ah - bh) / 2.0, bw, bh)
        }
        TriggerPosition::Right | TriggerPosition::TopRight | TriggerPosition::BottomRight => {
            (ax - gap - bw, ay + (ah - bh) / 2.0, bw, bh)
        }
        TriggerPosition::Bottom => (ax + (aw - bw) / 2.0, ay - gap - bh, bw, bh),
    };
    if w <= sw {
        x = x.clamp(sx, sx + sw - w);
    } else {
        x = sx;
    }
    if h <= sh {
        y = y.clamp(sy, sy + sh - h);
    } else {
        y = sy;
    }
    (x, y, w, h)
}

/// 邻接间隙条（DEC-94 · 点击面板外隐藏保持域成员）：主面板与详情窗之间沿停靠
/// 轴的 ADJACENT_GAP 过渡条 —— 光标自面板跨入详情窗期间点击不误收（面板∪详情
/// ∪间隙∪热区保持域的缝隙补片）。两条窗沿停靠轴分离且间距 ≤ ADJACENT_GAP+
/// 1.0px（设计间距或轻微漂移容差；漂移过大不补条防吞域扩容）且垂直轴向交叠段
/// 非空才存在，否则 None。top-left 原点坐标。
pub fn adjacent_gap_strip(
    pos: TriggerPosition,
    panel_rect: (f32, f32, f32, f32),
    peek_rect: (f32, f32, f32, f32),
) -> Option<(f32, f32, f32, f32)> {
    let (px_, py, pw, ph) = panel_rect;
    let (qx, qy, qw, qh) = peek_rect;
    const MAX_DRIFT: f32 = 1.0;
    let (lo, hi, perp0, perp1) = match pos {
        TriggerPosition::Left | TriggerPosition::TopLeft | TriggerPosition::BottomLeft => {
            (px_ + pw, qx, py.max(qy), (py + ph).min(qy + qh))
        }
        TriggerPosition::Right | TriggerPosition::TopRight | TriggerPosition::BottomRight => {
            (qx + qw, px_, py.max(qy), (py + ph).min(qy + qh))
        }
        TriggerPosition::Bottom => (qy + qh, py, px_.max(qx), (px_ + pw).min(qx + qw)),
    };
    if hi <= lo || hi - lo > ADJACENT_GAP + MAX_DRIFT || perp1 <= perp0 {
        return None;
    }
    // 条带按停靠轴组装：水平族（轴 = x）→ (轴起, 垂起, 轴长, 垂长)；
    // 底缘（轴 = y）→ (垂起, 轴起, 垂长, 轴长)。
    Some(match pos {
        TriggerPosition::Bottom => (perp0, lo, perp1 - perp0, hi - lo),
        _ => (lo, perp0, hi - lo, perp1 - perp0),
    })
}

/// 边缘触发热区 rect（DEC-79）：触发位置的贴缘感知带（REQ-PANEL-EDGE-TRIGGER
/// 阈值 12pt · DEC-84 起以可用区域为参照）。边族全缘（整条边，高/宽 = 可用区
/// 幅，厚 `TRIGGER_T` = 12pt）；四角 4px 方形角区（`CORNER_SZ` = 4 · DEC-222
/// 用户指令「触发角尺寸限制在 4 像素范围内」，替换原 12pt 厚×半缘条带口径——
/// 贴可用区域对应角，与主面板角锚同参照）。屏坐标系同 panel_bounds（top-left
/// 原点 · y 向下 · screen = 可用区域 rect）。承载方式 = app 层光标轮询判定域
/// （platform::cursor_position 读探针 · 真机实证：非 key 非激活 NSPanel 收不到
/// mouseMoved，窗口 hover 死路）。无窗体化：不产生额外窗口（单窗口径）。
pub fn trigger_zone_bounds(pos: TriggerPosition, screen: ScreenRect) -> (f32, f32, f32, f32) {
    let (sx, sy, sw, sh) = screen;
    match pos {
        TriggerPosition::Left => (sx, sy, TRIGGER_T, sh),
        TriggerPosition::Right => (sx + sw - TRIGGER_T, sy, TRIGGER_T, sh),
        TriggerPosition::Bottom => (sx, sy + sh - TRIGGER_T, sw, TRIGGER_T),
        TriggerPosition::TopLeft => (sx, sy, CORNER_SZ, CORNER_SZ),
        TriggerPosition::TopRight => (sx + sw - CORNER_SZ, sy, CORNER_SZ, CORNER_SZ),
        TriggerPosition::BottomLeft => (sx, sy + sh - CORNER_SZ, CORNER_SZ, CORNER_SZ),
        TriggerPosition::BottomRight => (
            sx + sw - CORNER_SZ,
            sy + sh - CORNER_SZ,
            CORNER_SZ,
            CORNER_SZ,
        ),
    }
}

/// Dock 占用判定点数容差：全帧与可用区沿对应轴的差值超过此值视为该缘被系统
/// 占用（菜单栏/Dock 保留区；1px 取整噪声不计）。
pub const OCCUPIED_EPS: f32 = 2.0;

/// Dock 占用判定（DEC-84 · REQ-PANEL-TRIGGER-POSITION 可用性 · 纯函数）：触发位
/// 对应缘是否被系统占用（可用区沿该轴内缩超过 OCCUPIED_EPS）。
/// - 左缘：usable.x − full.x > eps；右缘：(full.x+full.w) − (usable.x+usable.w) > eps；
///   底缘：(full.y+full.h) − (usable.y+usable.h) > eps（底部保留区 = Dock）。
/// - 四角恒 false（DEC-84：角位无可用性条件，角锚可用区域自然让位系统保留区）。
pub fn is_edge_occupied(pos: TriggerPosition, full: ScreenRect, usable: ScreenRect) -> bool {
    match pos {
        TriggerPosition::Left => usable.0 - full.0 > OCCUPIED_EPS,
        TriggerPosition::Right => (full.0 + full.2) - (usable.0 + usable.2) > OCCUPIED_EPS,
        TriggerPosition::Bottom => (full.1 + full.3) - (usable.1 + usable.3) > OCCUPIED_EPS,
        TriggerPosition::TopLeft
        | TriggerPosition::TopRight
        | TriggerPosition::BottomLeft
        | TriggerPosition::BottomRight => false,
    }
}

/// Dock 同侧冲突自动切换（DEC-84：左→右、右→左、底→右的非对称映射）：被占用
/// 侧缘映射到目标缘；其余（含四角、未占用缘）原样返回。单 Dock 仅占一侧，目标
/// 缘不可能同时被占，无递归切换。
pub fn resolve_conflict(
    pos: TriggerPosition,
    full: ScreenRect,
    usable: ScreenRect,
) -> TriggerPosition {
    if !is_edge_occupied(pos, full, usable) {
        return pos;
    }
    match pos {
        TriggerPosition::Left => TriggerPosition::Right,
        TriggerPosition::Right => TriggerPosition::Left,
        TriggerPosition::Bottom => TriggerPosition::Right,
        _ => pos,
    }
}

/// 点落在热区 rect 内（含边界 · 双闭区间）判定（轮询唤出/离带重臂的判定核）。
pub fn point_in_zone(px_: f32, py: f32, zone: (f32, f32, f32, f32)) -> bool {
    let (zx, zy, zw, zh) = zone;
    px_ >= zx && px_ <= zx + zw && py >= zy && py <= zy + zh
}

/// 边缘排除带高度（REQ-PANEL-EDGE-TRIGGER · EdgeMark 真语义）：屏幕顶部与底部
/// 各 50pt 必须为绝对点数的水平排除带（非比例、非角标）。对 12pt 宽热区 strip
/// 而言（DEC-100），判定域 = 水平带 ∩ 热区 strip，即 strip 顶/底各
/// 50pt 段——几何取 strip 段口径（复用调用方传入的已钳制 zone，单源）。
pub const EXCLUSION_BAND: f32 = 50.0;

/// 边缘排除带判定（platform 接线轮 · DEC-81 · REQ-PANEL-EDGE-TRIGGER
/// SCN-PANEL-CORNER-EXCLUDE）：点是否落在排除段（热区 strip 顶/底各 50pt）。
/// 仅左右全缘族生效；底与四角恒 false（排除带系全缘触发语义，四角为
/// DEC-76⑤ 扩展域豁免）。zone 高 ≤ 2×EXCLUSION_BAND（整带吞噬热区，禁触将退化为
/// 常禁）兜底返回 false。判定含域内前置（点不在热区 → false）。
pub fn in_exclusion(trigger: TriggerPosition, x: f32, y: f32, zone: (f32, f32, f32, f32)) -> bool {
    if !matches!(trigger, TriggerPosition::Left | TriggerPosition::Right) {
        return false;
    }
    if !point_in_zone(x, y, zone) {
        return false;
    }
    let (zy, zh) = (zone.1, zone.3);
    if zh <= EXCLUSION_BAND * 2.0 {
        return false;
    }
    y < zy + EXCLUSION_BAND || y > zy + zh - EXCLUSION_BAND
}

/// 缝侧邻接宽限带宽度（DEC-95）：触发缘共享缝外侧向邻接屏深入的过渡带宽——
/// 双屏内缝相邻（如副屏右邻）时越缝读数短暂落邻屏窄带属正常走位，不应挂入两拍
/// 收起判定。带内挂起收起判定（app 层宽限计数上行），长驻超限后恢复常规收起
/// 两拍。
pub const SEAM_GRACE_BAND: f32 = 20.0;

/// 缝侧邻接宽限带判定（DEC-95 · 纯函数）：点是否落在触发缘共享缝外侧 ≤ 20pt
/// 过渡带内。缝边由热区 rect 推导（热区 strip 外缘即共享缝，几何单一来源，不
/// 引入 screen 参数）：右缝 = zx+zw、左缝 = zx、底缝 = zy+zh；左族（左侧/左上/
/// 左下）→ 左带 x ∈ [zx − band, zx)；右族（右侧/右上/右下）→ 右带
/// x ∈ (zx+zw, zx+zw+band]；底 → 底带 y ∈ (zy+zh, zy+zh+band]（枚举仅 7 值 ·
/// DEC-87）。缝上（恰在缝位）不属带内（外缘闭、缝侧开）；带内
/// 不作垂直轴向约束（全缘走行带宽承载，长驻超限兜底）。top-left 原点 · y 向下
/// （DEC-67 口径）。
pub fn in_seam_grace_band(
    pos: TriggerPosition,
    x: f32,
    y: f32,
    zone: (f32, f32, f32, f32),
) -> bool {
    let (zx, zy, zw, zh) = zone;
    let band = SEAM_GRACE_BAND;
    match pos {
        TriggerPosition::Left | TriggerPosition::TopLeft | TriggerPosition::BottomLeft => {
            x < zx && x >= zx - band
        }
        TriggerPosition::Right | TriggerPosition::TopRight | TriggerPosition::BottomRight => {
            x > zx + zw && x <= zx + zw + band
        }
        TriggerPosition::Bottom => y > zy + zh && y <= zy + zh + band,
    }
}

/// 内缘不禁触（DEC-83）：内缘禁触机械（axis_overlap/SeamEdge/`is_inner_seam`）
/// 不存在，双屏内缘照常唤出（偏差 SRC-BASE-EDGEMARK §2.1「内缘不得触发」·
/// DEC-83 裁定）；防跨屏误触由唤出两拍确认（0.2s 停留节流）+ 抑制门承载；收起
/// 侧缝位误伤由缝侧邻接宽限带承载（DEC-95，见下）。
#[cfg(test)]
mod tests {
    use super::*;

    const S: ScreenRect = (0.0, 0.0, 1200.0, 800.0);
    // 停靠/热区/邻接几何的 screen 参数均为可用区域 rect（DEC-84）：S 即按可用区
    // 语义传入，全帧 rect 仅参与 is_edge_occupied / resolve_conflict 判定。

    #[test]
    fn labels_are_seven_unique_and_parse_roundtrip() {
        let labels: Vec<&str> = TriggerPosition::ALL.iter().map(|p| p.label()).collect();
        assert_eq!(
            labels,
            [
                "底部",
                "左侧",
                "右侧",
                "左上角",
                "右上角",
                "左下角",
                "右下角"
            ]
        );
        // 标签与位置互异且可解析回环（设置 pill ↔ CLI 同源）。
        let mut seen: Vec<&str> = Vec::new();
        for p in TriggerPosition::ALL {
            assert!(!seen.contains(&p.label()), "标签重复 {}", p.label());
            seen.push(p.label());
            assert_eq!(TriggerPosition::parse(p.label()), Some(p));
        }
        assert_eq!(TriggerPosition::parse("不存在"), None);
        // 「顶部」无对应触发位，不可解析（DEC-84/87）。
        assert_eq!(TriggerPosition::parse("顶部"), None);
        // 「顶」字仍不作为前缀误命中任何位置。
        assert_eq!(TriggerPosition::parse("顶"), None);
        // 默认 = 右侧（DEC-76）。
        assert_eq!(TriggerPosition::default(), TriggerPosition::Right);
    }

    #[test]
    fn swap_semantics_only_for_bottom() {
        for p in TriggerPosition::ALL {
            assert_eq!(p.swaps_dims(), matches!(p, TriggerPosition::Bottom));
        }
        assert_eq!(
            swap_dims_if_horizontal(TriggerPosition::Left, (320.0, 720.0)),
            (320.0, 720.0)
        );
        assert_eq!(
            swap_dims_if_horizontal(TriggerPosition::Bottom, (480.0, 700.0)),
            (700.0, 480.0)
        );
    }

    #[test]
    fn panel_bounds_edge_family() {
        // 左：x=0 · y 垂直居中（800-720）/2=40。
        assert_eq!(
            panel_bounds(TriggerPosition::Left, S, (320.0, 720.0)),
            (0.0, 40.0, 320.0, 720.0)
        );
        // 右：x=1200-320=880。
        assert_eq!(
            panel_bounds(TriggerPosition::Right, S, (320.0, 720.0)),
            (880.0, 40.0, 320.0, 720.0)
        );
    }

    #[test]
    fn panel_bounds_bottom_family_swaps_and_centers() {
        // 底：宽高互换 720×320 · x 水平居中 240 · y=800-320=480（可用区底缘）。
        assert_eq!(
            panel_bounds(TriggerPosition::Bottom, S, (320.0, 720.0)),
            (240.0, 480.0, 720.0, 320.0)
        );
    }

    #[test]
    fn panel_bounds_corners_anchor_without_swap() {
        assert_eq!(
            panel_bounds(TriggerPosition::TopLeft, S, (320.0, 720.0)),
            (0.0, 0.0, 320.0, 720.0)
        );
        assert_eq!(
            panel_bounds(TriggerPosition::TopRight, S, (320.0, 720.0)),
            (880.0, 0.0, 320.0, 720.0)
        );
        assert_eq!(
            panel_bounds(TriggerPosition::BottomLeft, S, (320.0, 720.0)),
            (0.0, 80.0, 320.0, 720.0)
        );
        assert_eq!(
            panel_bounds(TriggerPosition::BottomRight, S, (320.0, 720.0)),
            (880.0, 80.0, 320.0, 720.0)
        );
    }

    #[test]
    fn dock_families() {
        assert!(TriggerPosition::docks_left(TriggerPosition::Left));
        assert!(TriggerPosition::docks_left(TriggerPosition::TopLeft));
        assert!(TriggerPosition::docks_left(TriggerPosition::BottomLeft));
        assert!(TriggerPosition::docks_right(TriggerPosition::Right));
        assert!(TriggerPosition::docks_right(TriggerPosition::BottomRight));
        assert!(!TriggerPosition::docks_right(TriggerPosition::Bottom));
    }

    #[test]
    fn peek_bounds_adjacent_same_side() {
        // base 沿用调用方口径：左右/四角族 (480,700)、底族已换向 (700,480)。
        // 屏 1200×800 · y 向下 · GAP=16 · 面板 rect 取 panel_bounds 同源。
        let panel = |pos| panel_bounds(pos, S, (320.0, 720.0));
        // 左族：面板 (0,40,320,720) → peek 右邻 x=0+320+16=336 · y 中轴对齐；
        // 四角面板 y 锚各自角（TL=0 · BL=80），peek y 随面板 y 中轴。
        for (pos, expected) in [
            (TriggerPosition::Left, (336.0, 50.0, 480.0, 700.0)),
            (TriggerPosition::TopLeft, (336.0, 10.0, 480.0, 700.0)),
            (TriggerPosition::BottomLeft, (336.0, 90.0, 480.0, 700.0)),
            // 右族：面板 (880,40,320,720) → peek 左邻 x=880-16-480=384；四角同理。
            (TriggerPosition::Right, (384.0, 50.0, 480.0, 700.0)),
            (TriggerPosition::TopRight, (384.0, 10.0, 480.0, 700.0)),
            (TriggerPosition::BottomRight, (384.0, 90.0, 480.0, 700.0)),
        ] {
            assert_eq!(peek_bounds(pos, S, panel(pos), (480.0, 700.0)), expected);
        }
        // 底：面板 (240,480,720,320) → peek 上邻 y=480-16-480=-16 → 钳制进可用区 y=0。
        assert_eq!(
            peek_bounds(
                TriggerPosition::Bottom,
                S,
                panel(TriggerPosition::Bottom),
                (700.0, 480.0)
            ),
            (250.0, 0.0, 700.0, 480.0)
        );
    }

    #[test]
    fn peek_bounds_adjacent_on_secondary_display() {
        // 多屏参照（DEC-76）：屏 rect (1440,0,1170,800)；右族面板 x=1440+1170-320=2290，
        // peek 左邻 x=2290-16-480=1794（几何纯函数按传入屏 rect 逐屏计算）。
        let screen: ScreenRect = (1440.0, 0.0, 1170.0, 800.0);
        let panel = panel_bounds(TriggerPosition::Right, screen, (320.0, 720.0));
        assert_eq!(panel, (2290.0, 40.0, 320.0, 720.0));
        assert_eq!(
            peek_bounds(TriggerPosition::Right, screen, panel, (480.0, 700.0)),
            (1794.0, 50.0, 480.0, 700.0)
        );
    }

    #[test]
    fn trigger_zone_bounds_edge_family_full_edge() {
        // 边族全缘（12pt 厚 · 整条边）：厚值锚定 REQ-PANEL-EDGE-TRIGGER（DEC-100）。
        assert_eq!(TRIGGER_T, 12.0);
        assert_eq!(
            trigger_zone_bounds(TriggerPosition::Left, S),
            (0.0, 0.0, 12.0, 800.0)
        );
        assert_eq!(
            trigger_zone_bounds(TriggerPosition::Right, S),
            (1188.0, 0.0, 12.0, 800.0)
        );
        assert_eq!(
            trigger_zone_bounds(TriggerPosition::Bottom, S),
            (0.0, 788.0, 1200.0, 12.0)
        );
    }

    #[test]
    fn trigger_zone_bounds_corner_family_4px_square() {
        // 四角 4px 方形角区（贴可用区域对应角 · DEC-222 用户指令；替换原 12pt
        // 厚 × 半缘条带）。
        assert_eq!(CORNER_SZ, 4.0);
        assert_eq!(
            trigger_zone_bounds(TriggerPosition::TopLeft, S),
            (0.0, 0.0, 4.0, 4.0)
        );
        assert_eq!(
            trigger_zone_bounds(TriggerPosition::TopRight, S),
            (1196.0, 0.0, 4.0, 4.0)
        );
        assert_eq!(
            trigger_zone_bounds(TriggerPosition::BottomLeft, S),
            (0.0, 796.0, 4.0, 4.0)
        );
        assert_eq!(
            trigger_zone_bounds(TriggerPosition::BottomRight, S),
            (1196.0, 796.0, 4.0, 4.0)
        );
    }

    #[test]
    fn trigger_zone_bounds_secondary_display_offset() {
        // 多屏参照（DEC-76）：按传入屏 rect 逐屏计算（真原点 defer 不影响纯函数）。
        let screen: ScreenRect = (1440.0, 0.0, 1170.0, 800.0);
        assert_eq!(
            trigger_zone_bounds(TriggerPosition::Right, screen),
            (2598.0, 0.0, 12.0, 800.0)
        );
        assert_eq!(
            trigger_zone_bounds(TriggerPosition::BottomRight, screen),
            (2606.0, 796.0, 4.0, 4.0)
        );
    }

    // ───── platform 接线轮（DEC-81）：排除带几何 ─────

    #[test]
    fn exclusion_band_suppresses_strip_ends_only_for_full_edges() {
        assert_eq!(EXCLUSION_BAND, 50.0);
        // 右缘全缘热区 (1188,0,12,800)：带宽 = 顶/底各 50pt 段（x=1199 在
        // 12pt 带内 · DEC-100）。
        let zone = trigger_zone_bounds(TriggerPosition::Right, S);
        assert!(in_exclusion(TriggerPosition::Right, 1199.0, 0.0, zone));
        assert!(in_exclusion(TriggerPosition::Right, 1199.0, 49.9, zone));
        assert!(in_exclusion(TriggerPosition::Right, 1199.0, 750.1, zone));
        assert!(in_exclusion(TriggerPosition::Right, 1199.0, 800.0, zone));
        // 中段（带外）不排除；热区外点恒 false。
        assert!(!in_exclusion(TriggerPosition::Right, 1199.0, 400.0, zone));
        assert!(!in_exclusion(TriggerPosition::Right, 1000.0, 20.0, zone));
        // 左缘同口径。
        let lzone = trigger_zone_bounds(TriggerPosition::Left, S);
        assert!(in_exclusion(TriggerPosition::Left, 1.0, 10.0, lzone));
        assert!(!in_exclusion(TriggerPosition::Left, 1.0, 400.0, lzone));
        // 底与四角：排除带豁免（DEC-76⑤ 扩展域豁免）。
        assert!(!in_exclusion(
            TriggerPosition::Bottom,
            600.0,
            799.0,
            trigger_zone_bounds(TriggerPosition::Bottom, S)
        ));
        let crzone = trigger_zone_bounds(TriggerPosition::TopRight, S);
        assert!(!in_exclusion(
            TriggerPosition::TopRight,
            1199.0,
            20.0,
            crzone
        ));
    }

    #[test]
    fn exclusion_band_short_zone_fallback_never_excludes() {
        // zone 高 ≤ 100（2×EXCLUSION_BAND）：整带吞热区 → 兑底豁免（禁触退化防护）。
        let tiny: ScreenRect = (0.0, 0.0, 1200.0, 60.0);
        let zone = trigger_zone_bounds(TriggerPosition::Right, tiny);
        assert!(!in_exclusion(TriggerPosition::Right, 1199.0, 30.0, zone));
    }

    // ───── DEC-84：可用区域占用判定与 Dock 同侧冲突自动切换 ─────

    const FULL: ScreenRect = (0.0, 0.0, 1200.0, 800.0);

    #[test]
    fn occupied_flags_across_edge_families() {
        assert_eq!(OCCUPIED_EPS, 2.0);
        // 无占用（全帧 = 可用区）：全部不可判为占用。
        for p in TriggerPosition::ALL {
            assert!(!is_edge_occupied(p, FULL, FULL));
        }
        // 左侧被占（Dock 在左）：仅 Left 命中。
        let dock_left: ScreenRect = (120.0, 0.0, 1080.0, 800.0);
        assert!(is_edge_occupied(TriggerPosition::Left, FULL, dock_left));
        assert!(!is_edge_occupied(TriggerPosition::Right, FULL, dock_left));
        assert!(!is_edge_occupied(TriggerPosition::Bottom, FULL, dock_left));
        // 右侧被占：仅 Right 命中。
        let dock_right: ScreenRect = (0.0, 0.0, 1120.0, 800.0);
        assert!(is_edge_occupied(TriggerPosition::Right, FULL, dock_right));
        // 底部被占（Dock 在底，菜单栏不占顶余项不计）：仅 Bottom 命中。
        let dock_bottom: ScreenRect = (0.0, 0.0, 1200.0, 780.0);
        assert!(is_edge_occupied(TriggerPosition::Bottom, FULL, dock_bottom));
        assert!(!is_edge_occupied(TriggerPosition::Left, FULL, dock_bottom));
        // 容差边界：差值恰等于 OCCUPIED_EPS 不判占用（1px 取整噪声容忍）。
        let edge: ScreenRect = (2.0, 0.0, 1198.0, 800.0);
        assert!(!is_edge_occupied(TriggerPosition::Left, FULL, edge));
        // 四角恒不占用（DEC-84 · 可用性无条件）。
        for p in [
            TriggerPosition::TopLeft,
            TriggerPosition::TopRight,
            TriggerPosition::BottomLeft,
            TriggerPosition::BottomRight,
        ] {
            assert!(!is_edge_occupied(p, FULL, dock_left));
            assert!(!is_edge_occupied(p, FULL, dock_bottom));
        }
    }

    #[test]
    fn resolve_conflict_maps_occupied_sides_only() {
        // 未占用：原样（含四角）。
        for p in TriggerPosition::ALL {
            assert_eq!(resolve_conflict(p, FULL, FULL), p);
        }
        // 左被占：左→右；右/底/四角不动。目标缘与四角锚可用区，不二次切换。
        let dock_left: ScreenRect = (120.0, 0.0, 1080.0, 800.0);
        assert_eq!(
            resolve_conflict(TriggerPosition::Left, FULL, dock_left),
            TriggerPosition::Right
        );
        assert_eq!(
            resolve_conflict(TriggerPosition::Right, FULL, dock_left),
            TriggerPosition::Right
        );
        assert_eq!(
            resolve_conflict(TriggerPosition::Bottom, FULL, dock_left),
            TriggerPosition::Bottom
        );
        assert_eq!(
            resolve_conflict(TriggerPosition::TopLeft, FULL, dock_left),
            TriggerPosition::TopLeft
        );
        // 右被占：右→左。
        let dock_right: ScreenRect = (0.0, 0.0, 1120.0, 800.0);
        assert_eq!(
            resolve_conflict(TriggerPosition::Right, FULL, dock_right),
            TriggerPosition::Left
        );
        // 底被占：底→右（非对称口径：底冲突切右不切左）。
        let dock_bottom: ScreenRect = (0.0, 0.0, 1200.0, 780.0);
        assert_eq!(
            resolve_conflict(TriggerPosition::Bottom, FULL, dock_bottom),
            TriggerPosition::Right
        );
    }

    // ───── DEC-94：邻接间隙条（点击外保持域缝隙补片）─────

    const PANEL_R: (f32, f32, f32, f32) = (880.0, 40.0, 320.0, 720.0);

    #[test]
    fn adjacent_gap_strip_spans_between_windows_on_dock_axis() {
        // 右族：详情窗左邻（面板 x-16）→ 条 = 缝隙 16 × 轴向交叠段。
        let peek = (544.0, 240.0, 320.0, 320.0);
        assert_eq!(
            adjacent_gap_strip(TriggerPosition::Right, PANEL_R, peek),
            Some((864.0, 240.0, 16.0, 320.0))
        );
        // 左族：详情窗右邻 → 条同构镜像。
        let panel_l = (0.0, 40.0, 320.0, 720.0);
        let peek_l = (336.0, 240.0, 320.0, 320.0);
        assert_eq!(
            adjacent_gap_strip(TriggerPosition::Left, panel_l, peek_l),
            Some((320.0, 240.0, 16.0, 320.0))
        );
        // 底缘：详情窗上邻（间距 16，交叠段水平）→ 条 = 垂直缝隙 × 水平交叠段。
        let panel_b = (380.0, 440.0, 440.0, 320.0);
        let peek_b = (400.0, 104.0, 400.0, 320.0); // 底缘 424 = 面板顶 440 − 16
        assert_eq!(
            adjacent_gap_strip(TriggerPosition::Bottom, panel_b, peek_b),
            Some((400.0, 424.0, 400.0, 16.0))
        );
    }

    #[test]
    fn adjacent_gap_strip_none_when_overlapped_drifted_or_perp_disjoint() {
        // 交叠（详情窗已覆面板边）：条不存在。
        let overlapped = (896.0, 240.0, 320.0, 320.0);
        assert_eq!(
            adjacent_gap_strip(TriggerPosition::Right, PANEL_R, overlapped),
            None
        );
        // 漂移超容差（间距 > ADJACENT_GAP + 1.0）：不补条防吞域扩容。
        let drifted = (500.0, 240.0, 320.0, 320.0); // 面板右缘 880 − 详情右缘 820 = 60
        assert_eq!(
            adjacent_gap_strip(TriggerPosition::Right, PANEL_R, drifted),
            None
        );
        // 垂直轴向不交叠：条不存在。
        let disjoint = (544.0, 900.0, 320.0, 320.0);
        assert_eq!(
            adjacent_gap_strip(TriggerPosition::Right, PANEL_R, disjoint),
            None
        );
    }

    // ───── 缝侧邻接宽限带（DEC-95 · 副屏缝侧走位挂起收起）─────

    #[test]
    fn seam_grace_band_right_family_depth_bounds() {
        // 右族（右侧/右上/右下）同映射右缝（zx+zw=1200）：带内 = (1200, 1220]。
        // 外缘闭（1220 恰含）、缝侧开（1200 不在带内）、超深不挂起。
        for p in [
            TriggerPosition::Right,
            TriggerPosition::TopRight,
            TriggerPosition::BottomRight,
        ] {
            let z = trigger_zone_bounds(p, S);
            assert_eq!(z.0 + z.2, 1200.0, "右缝 = 热区 strip 外缘");
            assert!(in_seam_grace_band(p, 1205.0, 400.0, z), "带内深度点");
            assert!(in_seam_grace_band(p, 1220.0, 400.0, z), "外缘 20pt 恰含");
            assert!(!in_seam_grace_band(p, 1200.0, 400.0, z), "缝上不属带内");
            assert!(
                !in_seam_grace_band(p, 1221.0, 400.0, z),
                "超出 20pt 深度不挂起"
            );
            assert!(!in_seam_grace_band(p, 1199.0, 400.0, z), "热区内非带域");
            assert!(!in_seam_grace_band(p, 1300.0, 400.0, z), "邻屏深处非带域");
        }
    }

    #[test]
    fn seam_grace_band_left_family_depth_bounds() {
        // 左族（左侧/左上/左下）同映射左缝（zx=0）：带内 = [−20, 0)。
        for p in [
            TriggerPosition::Left,
            TriggerPosition::TopLeft,
            TriggerPosition::BottomLeft,
        ] {
            let z = trigger_zone_bounds(p, S);
            assert_eq!(z.0, 0.0, "左缝 = 热区 strip 外缘");
            assert!(in_seam_grace_band(p, -1.0, 400.0, z), "带内深度点");
            assert!(in_seam_grace_band(p, -20.0, 400.0, z), "外缘 20pt 恰含");
            assert!(!in_seam_grace_band(p, 0.0, 400.0, z), "缝上不属带内");
            assert!(
                !in_seam_grace_band(p, -21.0, 400.0, z),
                "超出 20pt 深度不挂起"
            );
            assert!(!in_seam_grace_band(p, 5.0, 400.0, z), "屏内侧非带域");
        }
    }

    #[test]
    fn seam_grace_band_bottom_depth_bounds() {
        // 底缘（底缝 = zy+zh = 800）：带内 = (800, 820]，仅按 y 判定。
        let z = trigger_zone_bounds(TriggerPosition::Bottom, S);
        assert_eq!(z.1 + z.3, 800.0, "底缝 = 热区 strip 外缘");
        assert!(
            in_seam_grace_band(TriggerPosition::Bottom, 600.0, 805.0, z),
            "带内深度点"
        );
        assert!(
            in_seam_grace_band(TriggerPosition::Bottom, 600.0, 820.0, z),
            "外缘 20pt 恰含"
        );
        assert!(
            !in_seam_grace_band(TriggerPosition::Bottom, 600.0, 800.0, z),
            "缝上不属带内"
        );
        assert!(
            !in_seam_grace_band(TriggerPosition::Bottom, 600.0, 821.0, z),
            "超出 20pt 深度不挂起"
        );
        assert!(
            !in_seam_grace_band(TriggerPosition::Bottom, 600.0, 797.0, z),
            "热区内非带域"
        );
        assert!(
            !in_seam_grace_band(TriggerPosition::Bottom, 600.0, 900.0, z),
            "邻屏深处非带域"
        );
    }
}
