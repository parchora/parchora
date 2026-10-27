//! 屏幕分辨率与系统缩放档位 → 全局字号/间距/窗体几何档位表（DEC-198）。
//!
//! 侦测与映射闭环：
//! 1. **档位判定**：`DispTier::for_width(逻辑宽)` 按「所在屏逻辑点尺寸」分六级档（1366
//!    宽以下 / ≤2048 / ≤2560 / ≤3620 超宽 / ≤4480 / 其余超宽 5120 级）——macOS
//!    的「系统缩放」表达即逻辑空间本身（4K@2x=1920 逻辑、4K@1.5x≈2560 逻辑），
//!    逻辑尺寸已内含缩放语义，无需另判；backingScale 作为物理密度佐证随剖面
//!    记录（`ScaleProfile.backing`，诊断用；trait 未暴露逐屏 scale，恒记基线
//!    2.0）。
//! 2. **字号表**：每档 × 11 个文本层级（正文/辅助/Meta/切换条/激活/小标题二支
//!    —h2·h3/标题/展示/字标/等宽）显式取值，值域严格落在《桌面应用字体大小
//!    推荐表》对应梯度列内；主档 Hd = tokens.rs 现行 consts（表 FHD 行基线）。
//! 3. **传播机制**：档位剖面按 DisplayId 存 app 级 Global（同屏窗共享）；渲染
//!    期经 per-window 线程局部栈（`enter`→RAII `Scope`）向全部样式 builder 零
//!    签名变更地提供 `font(lvl)/sp(v)` 查询——gpui 各窗 paint 串行（整树在
//!    `enter` 作用域内完成），TLS 生命周期与 paint 周期同构。
//! 4. **触发**：启动 `apply_all_displays` 全量建档；0.5s 扫描守护（main.rs
//!    `start_scale_sweep`，status-click 泵同模式）按「窗所在 DisplayId → 档位
//!    与上次记录异同」侦测跨屏迁移与分辨率/缩放变化，档变 → `window.refresh()`
//!    整窗重建（TLS 新剖面随帧生效）。
//!
//! 回退语义（测试/无显示器）：TLS 空栈 → 直接取 tokens.rs consts（设计基线，
//! 与历史渲染完全一致）——测试面确定性不受测试机分辨率扰动（screen_areas_seam
//! 同口径隔离先例）。

use std::cell::RefCell;
use std::collections::HashMap;

use gpui::{px, App, Pixels, Window};

use crate::tokens as t;

// ───────────────────────── 档位（分辨率梯度 · DEC-198） ─────────────────────────

/// 分辨率档位；判定载体 = 所在屏**逻辑点尺寸**（系统缩放语义内含 · 模块头 §1）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DispTier {
    /// 1366×768 类（≤1440 逻辑宽；含 1280×800/1440×900 笔记本）。
    Low,
    /// FHD 基准（≤2048；1512×982/1920×1080/1200、1080p@150% 逻辑 1280 等）。
    Hd,
    /// QHD（≤2560；2560×1440/2560×1080）。
    Qhd,
    /// 超宽 QHD（≤3620；3440×1440——表：字体保持与 QHD 接近，标题上限略抬）。
    Uqhd,
    /// 4K 逻辑原生（≤4480；3840×2160 —— macOS「更多空间」档）。
    FourK,
    /// 超宽 5120 级（>4480；5120×1440/5120×2880 1x 档）。
    U4K,
}

/// 全部分档（低→高）；映射/测试共用。
pub const ALL_TIERS: [DispTier; 6] = [
    DispTier::Low,
    DispTier::Hd,
    DispTier::Qhd,
    DispTier::Uqhd,
    DispTier::FourK,
    DispTier::U4K,
];

impl DispTier {
    /// 逻辑宽 → 档位（宽度单轴分档；垂直空间由面板 vh 比例自适应——主面板
    /// 80%/Peek 70%·DEC-129 口径，768~2880 全覆盖无溢出）。
    pub fn for_width(logical_w: f32) -> DispTier {
        let w = logical_w.max(0.0);
        if w <= 1440.0 {
            DispTier::Low
        } else if w <= 2048.0 {
            DispTier::Hd
        } else if w <= 2560.0 {
            DispTier::Qhd
        } else if w <= 3620.0 {
            DispTier::Uqhd
        } else if w <= 4480.0 {
            DispTier::FourK
        } else {
            DispTier::U4K
        }
    }
}

// ───────────────────────── 字号表（严格表值 · DEC-198） ─────────────────────────

/// 文本层级（tokens.rs FONT_SIZE_* 的档位化视图；fallback = 现行 consts）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Lvl {
    /// 正文 body-md（表·正文 Body）。
    Body,
    /// 说明文字 small（表·辅助/标签 Caption）。
    Small,
    /// 行 meta/时间戳（辅助系次级派生档＝Caption−1 且 ≥10 · 模块模块注记）。
    Meta,
    /// 视图切换条/行内标签（辅助/标签系派生档）。
    Tab,
    /// 切换条激活态（= Tab + 2）。
    TabActive,
    /// 小标题 h3（表·小标题 Subtitle）。
    H3,
    /// 块级标题二 h2（表·小标题 Subtitle 上支——DEC-134 三级映射层级关系保持）。
    H2,
    /// 大标题 h1（表·标题 Title）。
    H1,
    /// 展示字号（表·大标题/展示 Display）。
    Display,
    /// 字标 PARCHORA（Display 系派生档）。
    Wordmark,
    /// 等宽/代码（表·等宽/代码 Mono；源码态/预览围栏/PG-B1）。
    Mono,
}

/// 每档字号表（值域逐列对照《桌面应用字体大小推荐表》；推导注记入 DEC-198）。
#[derive(Clone, Copy, Debug)]
pub struct FontTable {
    pub body: f32,
    pub small: f32,
    pub meta: f32,
    pub tab: f32,
    pub tab_active: f32,
    pub h3: f32,
    pub h2: f32,
    pub h1: f32,
    pub display: f32,
    pub wordmark: f32,
    pub mono: f32,
}

/// 设计基线 = tokens.rs 现行 consts（Hd 档 · 表 FHD 行；回退语义载体）。
fn base_table() -> FontTable {
    FontTable {
        body: t::FONT_SIZE_BODY,
        small: t::FONT_SIZE_SMALL,
        meta: t::FONT_SIZE_META,
        tab: t::FONT_SIZE_TAB,
        tab_active: t::FONT_SIZE_TAB_ACTIVE,
        h3: t::FONT_SIZE_H3,
        h2: t::FONT_SIZE_H2,
        h1: t::FONT_SIZE_H1,
        display: t::FONT_SIZE_DISPLAY,
        wordmark: t::FONT_SIZE_WORDMARK,
        mono: t::FONT_SIZE_MONO,
    }
}

/// 档位 → 字号表。每列取值：**在表值域内取最贴近设计基线的值**（档间连续性优先
/// · 取值推导逐条落 DEC-198 注记）。
pub fn font_at(tier: DispTier) -> FontTable {
    match tier {
        // 表 1366×768 行：正文 14–16 / 辅助 12 / 小标题 16–18 / 标题 20–24 /
        // 展示 28–32 / 等宽 13–15。
        DispTier::Low => FontTable {
            body: 14.0,
            small: 12.0,
            meta: 11.0,
            tab: 12.0,
            tab_active: 14.0,
            h3: 16.0,
            h2: 18.0,
            h1: 22.0,
            display: 28.0,
            wordmark: 30.0,
            mono: 13.0,
        },
        // 表 FHD 行：正文 16–18 / 辅助 12–14 / 小标题 18–22 / 标题 24–28 /
        // 展示 32–40 / 等宽 14–16。h2/h3 落「小标题」支：h2 取下沿 20（设计
        // 基线原值）、h3 取下沿 18（基线 17 表值域外 → 18）；正文/时间戳随表
        // 下沿各 +1（基线 15/11 → 16/12）。
        DispTier::Hd => FontTable {
            body: 16.0,
            small: 12.0,
            meta: 12.0,
            tab: 13.0,
            tab_active: 15.0,
            h3: 18.0,
            h2: 20.0,
            h1: 26.0,
            display: 32.0,
            wordmark: 33.0,
            mono: 14.0,
        },
        // 表 QHD 行（2K 事）：正文 15–17 / 辅助 13–15 / 小标题 18–22 / 标题
        // 24–30 / 展示 32–40 / 等宽 14–16 —— 与 FHD 同取，仅辅助列抬至表下沿
        // 13（像素密度更高，辅助字不再压下限）。
        DispTier::Qhd => FontTable {
            body: 16.0,
            small: 13.0,
            meta: 12.0,
            tab: 13.0,
            tab_active: 15.0,
            h3: 18.0,
            h2: 20.0,
            h1: 26.0,
            display: 32.0,
            wordmark: 33.0,
            mono: 14.0,
        },
        // 表 3440×1440 行：正文 15–17（同 QHD 保持）/ 辅助 13–15 / 标题 26–30
        // / 展示 34–40。字标/展示随列抬升；正文/等宽不盲放（超宽价值在横栏）。
        DispTier::Uqhd => FontTable {
            body: 16.0,
            small: 14.0,
            meta: 12.0,
            tab: 14.0,
            tab_active: 16.0,
            h3: 20.0,
            h2: 22.0,
            h1: 26.0,
            display: 36.0,
            wordmark: 36.0,
            mono: 15.0,
        },
        // 表 4K 行（27" 150% 档为基准）：正文 16–18 / 辅助 14–16 / 小标题
        // 20–24 / 标题 28–36 / 展示 40–48 / 等宽 16–18。h1 取 32（列中档）。
        DispTier::FourK => FontTable {
            body: 17.0,
            small: 15.0,
            meta: 13.0,
            tab: 15.0,
            tab_active: 17.0,
            h3: 21.0,
            h2: 24.0,
            h1: 32.0,
            display: 42.0,
            wordmark: 42.0,
            mono: 16.0,
        },
        // 表 5120×1440 行：正文 16–18 / 辅助 14–16 / 小标题 20–24 / 标题
        // 28–34 / 展示 38–46 / 等宽 16–18 —— 垂直仍 1440，与 QHD 超宽同策略
        //（利用横向分栏不放字体）；标题取 31 保持 h1<h2 阶差与列值域。
        DispTier::U4K => FontTable {
            body: 17.0,
            small: 15.0,
            meta: 13.0,
            tab: 15.0,
            tab_active: 17.0,
            h3: 22.0,
            h2: 24.0,
            h1: 31.0,
            display: 42.0,
            wordmark: 42.0,
            mono: 16.0,
        },
    }
}

// ───────────────────────── 剖面（字号 + 间距/窗体因子） ─────────────────────────

/// 档位剖面：字号表 + 间距/窗体几何倍率。
///
/// **间距/几何策略**（DEC-198 注记）：SP_*/FAB 等令牌与窗体宽/高钳制按档位倍率
/// 缩放（窄屏收紧、高分屏舒展）；行高倍率发丝线（border 1px）与 8px 网格内联
/// 微间距**不参与缩放**（DESIGN.md 8px 基线与发丝线恒 1px 是视觉身份不变量，
/// 逐点缩放需 500+ 直设位接入，收益低且破坏网格语义——登记 defer）。
#[derive(Clone, Copy, Debug)]
pub struct ScaleProfile {
    pub tier: DispTier,
    /// 档位字号表。
    pub font: FontTable,
    /// 间距/窗体几何倍率（×1.0 = 设计基线）。
    pub space: f32,
    /// 记录面：建剖面时屏逻辑宽（诊断/测试断言用）。
    pub logical_w: f32,
    /// 记录面：建剖面时屏 backingScale（物理密度佐证 · 模块头 §1）。
    pub backing: f32,
}

/// 档位 → 间距/窗体倍率（DEC-198）：低档收紧防溢出（768 高），高分档舒展；
/// 超宽 1440 高档位低于 4K 竖向（垂直空间参照 QHD 而非 2160）。
pub fn space_at(tier: DispTier) -> f32 {
    match tier {
        DispTier::Low => 0.92,
        DispTier::Hd => 1.0,
        DispTier::Qhd => 1.05,
        DispTier::Uqhd => 1.10,
        DispTier::FourK => 1.18,
        DispTier::U4K => 1.12,
    }
}

impl ScaleProfile {
    /// 设计基线剖面（Hd · 无档位信息时的回退值）。
    pub fn base() -> ScaleProfile {
        ScaleProfile {
            tier: DispTier::Hd,
            font: base_table(),
            space: space_at(DispTier::Hd),
            logical_w: 0.0,
            backing: 2.0,
        }
    }

    /// 档位剖面（按屏逻辑宽/backing 建档）。
    pub fn for_screen(logical_w: f32, backing: f32) -> ScaleProfile {
        let tier = DispTier::for_width(logical_w);
        ScaleProfile {
            tier,
            font: font_at(tier),
            space: space_at(tier),
            logical_w,
            backing,
        }
    }
}

// ───────────────────────── Global 档位仓（按 DisplayId） ─────────────────────────

/// 按 DisplayId（=CGDirectDisplayID · vendor From<DisplayId> for u64 同口径）存
/// 档位剖面；同屏窗共享同剖面。未建档 Display → `ScaleProfile::base()`（Hd 基线
/// 兜底，测试面恒此态——确定性不扰真机分辨率，screen_areas_seam 同口径）。
#[derive(Default)]
pub struct ScreenProfiles {
    by_display: HashMap<u64, ScaleProfile>,
}

impl gpui::Global for ScreenProfiles {}

impl ScreenProfiles {
    /// 建档/更新（档位或剖面有变才写并返回 true，值变即触发窗刷新的判据）。
    pub fn ensure(&mut self, display_id: u64, logical_w: f32, backing: f32) -> bool {
        let next = ScaleProfile::for_screen(logical_w, backing);
        match self.by_display.get(&display_id) {
            Some(cur) if cur.tier == next.tier && cur.logical_w == next.logical_w => false,
            _ => {
                self.by_display.insert(display_id, next);
                true
            }
        }
    }

    /// 读取（未建档 → None；调用方回退 base 剖面）。
    pub fn get(&self, display_id: u64) -> Option<ScaleProfile> {
        self.by_display.get(&display_id).copied()
    }
}

/// 读 app 全局仓（不存在/未建档 → None）。
pub fn profile_for_display(cx: &App, display_id: u64) -> Option<ScaleProfile> {
    cx.try_global::<ScreenProfiles>()
        .and_then(|p| p.get(display_id))
}

/// 全显示器建档（启动/守护首拍共用）：逐屏按 PlatformDisplay::size()（逻辑点）
/// 建档；返回本拍发生档位/剖面变化的 display id 集（守护据此刷窗）。
pub fn apply_all_displays(cx: &mut App) -> Vec<u64> {
    let mut changed = Vec::new();
    for d in cx.displays() {
        let id = u64::from(d.id());
        let bounds = d.bounds();
        let w = f32::from(bounds.size.width);
        let _logical_h = f32::from(bounds.size.height);
        // 同值拍零写零记账：vendor `global_mut` 调用即无条件 push effect＋stamp
        // 依赖记账（与值是否变化无关）——先读比、变才写，否则空闲拍每次调用都
        // 会置 render 期读本仓的窗全树 stale（DEC-203 F1 · FIX-20261023-01）。
        let next = ScaleProfile::for_screen(w, 2.0);
        let existing = cx.try_global::<ScreenProfiles>();
        if existing.is_some_and(|p| {
            p.get(id)
                .is_some_and(|cur| cur.tier == next.tier && cur.logical_w == next.logical_w)
        }) {
            continue;
        }
        if existing.is_none() {
            cx.set_global(ScreenProfiles::default());
        }
        if cx.global_mut::<ScreenProfiles>().ensure(id, w, 2.0) {
            changed.push(id);
        }
    }
    changed
}

// ───────────────────────── 渲染期 TLS 栈（builder 零签名接入） ─────────────────────────

thread_local! {
    /// 档位栈（paint 串行 · RAII 弹出）：栈顶 = 当前窗剖面；空栈 = 回退基线。
    static TIERS: RefCell<Vec<DispTier>> = const { RefCell::new(Vec::new()) };
}

/// 渲染作用域守卫：压入当前窗所在屏档位，drop 弹出——根视图 render 首行
/// `let _s = scale::enter(window, cx);` 一行接入（gpui 每窗整树串行 paint，
/// 作用域与 render 调用栈同构）。
pub struct Scope {
    _priv: (),
}

impl Drop for Scope {
    fn drop(&mut self) {
        TIERS.with_borrow_mut(|v| {
            v.pop();
        });
    }
}

/// 进入渲染作用域：窗所在屏已建档 → 该档位；否则**保持栈顶**（面板根视图内嵌
/// 子视图/浮层 render 复入时档位不变——同窗同档）或基线（根级首渲染未建档 =
/// 测试态）。
pub fn enter(window: &Window, cx: &App) -> Scope {
    let tier = window
        .display(cx)
        .and_then(|d| profile_for_display(cx, u64::from(d.id())))
        .map(|p| p.tier);
    TIERS.with_borrow_mut(|v| {
        // 命名避开模块级 `t`（tokens 别名）——局部遮蔽属危险先兆。
        let tier_v = tier.or(v.last().copied());
        if let Some(tier_v) = tier_v {
            v.push(tier_v);
        }
    });
    Scope { _priv: () }
}

fn current_tier() -> Option<DispTier> {
    TIERS.with_borrow(|v| v.last().copied())
}

// ───────────────────────── 查询 API（builder/direct 位通吃） ─────────────────────────

/// 层级字号（f32，供 txt()/px(s::font(..)) 传参）：TLS 档位 → 该档表值；无档位
/// → tokens.rs 基线 consts（回退确定性 · 测试面恒基线）。
pub fn font(lvl: Lvl) -> f32 {
    let tbl = current_tier().map(font_at).unwrap_or_else(base_table);
    match lvl {
        Lvl::Body => tbl.body,
        Lvl::Small => tbl.small,
        Lvl::Meta => tbl.meta,
        Lvl::Tab => tbl.tab,
        Lvl::TabActive => tbl.tab_active,
        Lvl::H3 => tbl.h3,
        Lvl::H2 => tbl.h2,
        Lvl::H1 => tbl.h1,
        Lvl::Display => tbl.display,
        Lvl::Wordmark => tbl.wordmark,
        Lvl::Mono => tbl.mono,
    }
}

/// 层级字号 px（直接 text_size 位用）。
pub fn px_of(lvl: Lvl) -> Pixels {
    px(font(lvl))
}

/// 间距/尺寸令牌缩放（SP_*/FAB 等尺寸面）：TLS 档位倍率 × v；无档位 → v 原值
/// （回退与基线完全一致·DEC-198 策略注记）。
pub fn sp(v: f32) -> f32 {
    current_tier().map(space_at).unwrap_or(1.0) * v
}

/// 窗体几何/间距令牌缩放（显式定档位版——main.rs 开窗几何无 TLS 语境处用）。
pub fn sp_at(tier: DispTier, v: f32) -> f32 {
    space_at(tier) * v
}

// ───────────────────────── 测试（表值域严格校验 · 纯函数面） ─────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// 《桌面应用字体大小推荐表》值域（列序 = Body/Caption/Subtitle/Title/
    /// Display/Mono × Low,Hd,Qhd,Uqhd,FourK,U4K 档序；DEC-198）。
    const RANGES: [(&str, [(f32, f32); 6]); 6] = [
        (
            "body",
            [
                (14., 16.),
                (16., 18.),
                (15., 17.),
                (15., 17.),
                (16., 18.),
                (16., 18.),
            ],
        ),
        (
            "caption",
            [
                (12., 12.),
                (12., 14.),
                (13., 15.),
                (13., 15.),
                (14., 16.),
                (14., 16.),
            ],
        ),
        (
            "subtitle",
            [
                (16., 18.),
                (18., 22.),
                (18., 22.),
                (18., 22.),
                (20., 24.),
                (20., 24.),
            ],
        ),
        (
            "title",
            [
                (20., 24.),
                (24., 28.),
                (24., 30.),
                (26., 30.),
                (28., 36.),
                (28., 34.),
            ],
        ),
        (
            "display",
            [
                (28., 32.),
                (32., 40.),
                (32., 40.),
                (34., 40.),
                (40., 48.),
                (38., 46.),
            ],
        ),
        (
            "mono",
            [
                (13., 15.),
                (14., 16.),
                (14., 16.),
                (14., 16.),
                (16., 18.),
                (16., 18.),
            ],
        ),
    ];

    fn pick(tbl: &FontTable, col: &str) -> f32 {
        match col {
            "body" => tbl.body,
            "caption" => tbl.small,
            "subtitle" => tbl.h3,
            "title" => tbl.h1,
            "display" => tbl.display,
            "mono" => tbl.mono,
            _ => unreachable!(),
        }
    }

    #[test]
    fn table_values_strictly_within_spec_ranges() {
        for (col, ranges) in RANGES {
            for (i, tier) in ALL_TIERS.iter().enumerate() {
                let (lo, hi) = ranges[i];
                let v = pick(&font_at(*tier), col);
                assert!(
                    v >= lo && v <= hi,
                    "{col}/{tier:?} = {v} 不在表值域 [{lo}, {hi}]"
                );
            }
        }
    }

    #[test]
    fn tier_band_boundaries() {
        assert_eq!(DispTier::for_width(1366.0), DispTier::Low);
        assert_eq!(DispTier::for_width(1440.0), DispTier::Low);
        assert_eq!(DispTier::for_width(1441.0), DispTier::Hd);
        assert_eq!(DispTier::for_width(1512.0), DispTier::Hd); // MBP 14 默认
        assert_eq!(DispTier::for_width(1920.0), DispTier::Hd);
        assert_eq!(DispTier::for_width(2048.0), DispTier::Hd);
        assert_eq!(DispTier::for_width(2560.0), DispTier::Qhd);
        assert_eq!(DispTier::for_width(3440.0), DispTier::Uqhd);
        assert_eq!(DispTier::for_width(3621.0), DispTier::FourK);
        assert_eq!(DispTier::for_width(3840.0), DispTier::FourK);
        assert_eq!(DispTier::for_width(5120.0), DispTier::U4K);
    }

    #[test]
    fn hierarchy_holds_in_every_tier() {
        for tier in ALL_TIERS {
            let tbl = font_at(tier);
            assert!(tbl.meta <= tbl.small, "{tier:?} meta≤small");
            assert!(tbl.small <= tbl.tab + 1.0, "{tier:?} small≈tab");
            assert!(tbl.tab <= tbl.tab_active, "{tier:?} tab≤tab_active");
            assert!(tbl.h3 <= tbl.h2, "{tier:?} h3≤h2");
            assert!(tbl.h2 <= tbl.h1, "{tier:?} h2≤h1");
            assert!(tbl.h1 <= tbl.display, "{tier:?} h1≤display");
        }
    }

    #[test]
    fn fallback_equals_tokens_consts() {
        // 无 TLS（测试基线态）→ 与 tokens consts 恒等（回退确定性）。
        let base = base_table();
        assert_eq!(font(Lvl::Body), base.body);
        assert_eq!(font(Lvl::Meta), base.meta);
        assert_eq!(font(Lvl::Mono), base.mono);
        assert_eq!(font(Lvl::H1), base.h1);
        assert_eq!(sp(t::SP_SM), t::SP_SM);
    }

    #[test]
    fn scoped_tier_switches_font_and_space() {
        // TLS 栈压 Low → Body=14/small=12/间距×0.92；弹出后回基线。
        let low = font_at(DispTier::Low);
        TIERS.with_borrow_mut(|v| v.push(DispTier::Low));
        assert_eq!(font(Lvl::Body), low.body);
        assert_eq!(font(Lvl::Meta), low.meta);
        assert_eq!(font(Lvl::Mono), low.mono);
        assert!(
            (sp(t::SP_MD) - t::SP_MD * 0.92).abs() < 1e-3,
            "Low 档 SP_MD 倍率"
        );
        TIERS.with_borrow_mut(|v| v.pop());
        assert_eq!(font(Lvl::Body), t::FONT_SIZE_BODY);
        assert_eq!(sp(t::SP_MD), t::SP_MD);
    }

    #[test]
    fn profile_store_change_detection() {
        let mut store = ScreenProfiles::default();
        assert!(store.ensure(1, 1920.0, 2.0)); // 首建 = 变
        assert!(!store.ensure(1, 1920.0, 2.0)); // 恒等 = 不变
        assert!(store.ensure(1, 2560.0, 2.0)); // 跨屏/换档 = 变
        assert_eq!(store.get(1).map(|p| p.tier), Some(DispTier::Qhd));
        assert!(store.get(2).is_none());
    }

    #[gpui::test]
    async fn apply_all_displays_idle_rerun_is_noop(cx: &mut gpui::TestAppContext) {
        // 模拟 1920×1080（TestDisplay 默认 bounds · Hd 档）：首拍建档报变，
        // 同值重跑必须零变化——守护每 0.5s 重跑时不再触 global_mut 记账
        // （render 期 try_global 读方的 stale 链源 · DEC-203 F1）。
        cx.simulate_display_added(1u64.into(), None);
        cx.update(|cx| {
            let first = apply_all_displays(cx);
            assert_eq!(first, vec![1], "首建档应报变");
            let again = apply_all_displays(cx);
            assert!(again.is_empty(), "同值重跑应零变化");
            assert_eq!(
                profile_for_display(cx, 1).map(|p| (p.tier, p.logical_w)),
                Some((DispTier::Hd, 1920.0)),
                "建档值不受幂等重跑扰动",
            );
        });
    }
}
