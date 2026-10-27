//! 设计令牌层（UI-0）——Parchora 视觉令牌唯一来源。
//!
//! 依据（不得偏离）：
//! - `docs/DESIGN.md`（Parchora Parchment Ink · alpha）：17 色 + 字型 + 圆角/间距阶 + 组件规范；
//! - 2026-10-23 用户四色指令重构（辅助 #142346 · 背景 #F5EBD2 · 文字 #191923 ·
//!   点缀 #D4AF37）：17 色逐项派生 + 新增 ACCENT 点缀金，派生依据见各常量行注释；
//! - `docs/产品交付/统一产品方案/需求文档/design tokens.json`（17 色 + spacing/rounded/typography）；
//! - `页面设计/HTML/parchora-shared.css`：组件原语逐项度量；
//! - 全量页面设计.md 修订 v3：主面板 360×屏高 80%（DEC-129）·
//!   内容面板 480×70% · 菜单 300 · 独立窗 430。
//! - 字体栈（DEC-66/68）：内容区=Kaiti SC（楷体-简）·
//!   UI 控件=Songti SC（宋体-简）· 代码=Menlo（系统默认等宽）；未安装时 CoreText 落默认回退
//!   （偏差 D-02 承接；DEC-68 实测「宋体/楷体」裸标签不可解析、必须用捆绑标准变体族名）。
//!   代码段 Menlo：macOS 10.6+ 捆绑、本机解析成功（Menlo-Regular），即「系统默认等宽」
//!   的实体。
//!
//! 值一律以 `u32` 十六进制（0xRRGGBB）常量保存，绘制处经 `gpui::rgb()` 转换；
//! 本层不依赖 gpui，保证令牌可独立测试（确定性 / 完整性 / 对比度）。

// 归档 v1.0 碑墨青体系不得引用——本文件之外禁止出现碑墨青色值（DESIGN.md Overview 定裁）。

// ─────────────── 色板（DESIGN.md colors 17 色 + ACCENT · 2026-10-23 用户四色重构） ───────────────
//
// 2026-10-23 用户指令四色基（辅助 #142346 · 背景 #F5EBD2 · 文字 #191923 · 点缀 #D4AF37），
// 17 色逐项派生（WCAG 实测值见各行注释）：文字阶梯与灰阶走蓝墨/蓝灰族（对应用户藏青系），
// 纸面族保持米黄提亮阶。金色在米黄底仅 1.77:1——禁止作浅底文字，限填充/描边/藏青底高亮。

pub const PRIMARY: u32 = 0x191923; // 文字=用户指定：正文/图标/主文字/落墨钮（bg 14.69:1）
pub const ON_PRIMARY: u32 = 0xF5EBD2; // 纸光文字=用户背景色：墨/藏青实心面上的字（14.69/13.03:1）
pub const PRESS: u32 = 0x0C0C12; // 落墨实心按钮 hover 底（原 press 阶梯语义）
pub const INK_DEEP: u32 = 0x07070C; // 深墨：落墨笔触 / Toast 底 / 夜读反白的深面
pub const SECONDARY: u32 = 0x50515C; // 次级描边（仅 ≥18px 场景；bg 6.62:1）
pub const MUTED: u32 = 0x5A5A64; // 18px 以下次级文字/时间戳/占位（bg 5.74:1 · surface 6.22:1）
pub const FAINT: u32 = 0xDFD2B0; // 装饰痕迹专用：斑点/飞白/极淡墨痕（只画不说话）
pub const TERTIARY: u32 = 0x142346; // 强调=用户辅助色：选中/链接/重要状态（bg 13.03:1）
pub const NEUTRAL: u32 = 0xF5EBD2; // 主背景=用户指定（羊皮米黄，永不用纯白）
pub const SURFACE: u32 = 0xFBF4E4; // 卡片面（比底更浅一档）
pub const WARM: u32 = 0xEBDFC2; // 次级表面与选中底（极浅暖底）
pub const BORDER: u32 = 0xD8C9A8; // 细边框
pub const BORDER_SOFT: u32 = 0xEFE5CD; // 发丝线（笔记本横线）
pub const GROWTH: u32 = 0x6B7F5C; // 生长绿：markdown 标签/tip 文字（内容语义保留；UI 完成点改 ACCENT）
pub const NIGHT_SAGE: u32 = 0xD4AF37; // 深面提亮点缀=点缀金（on 藏青 7.35:1）
pub const NIGHT_BG: u32 = 0x142346; // 夜读反白底=用户辅助色藏青
pub const NIGHT_INK: u32 = 0xF5EBD2; // 夜读反白文字=纸光
pub const ACCENT: u32 = 0xD4AF37; // 点缀金=用户指定：完成点/mark 高亮/选中描边（浅底只填充不作文字；on-navy 7.35:1）

// ───────────────────── 功能特批色（用户指令 · DEC-228） ─────────────────────

/// 围栏（```）码块卡底藏青：用户指令指定 #142346 @60% 半透明（R181），铺于
/// 纸上承载深卡码区；码文与复制钮取 ON_PRIMARY 纸光反白（复合 slate 底对
/// 比 ≈4.0:1）。行内码底/表格斑马纹/嵌入 chip 仍用 WARM 半透底（code_bg），
/// 不共用此色。
pub const CODE_NAVY: u32 = 0x142346;

/// 标签 7 色语义色（PG-B.html 实测 + REQ-TAG-7COLORS；顺序即 UI 呈现序）。
/// （名称, hex）：归档=SECONDARY 派生，灵感=GROWTH 派生。
pub const TAG_NAMES: [&str; 7] = ["设计", "调研", "速记", "灵感", "项目", "资料", "归档"];
pub const TAG_COLORS: [u32; 7] = [
    0xA2543E, // 设计
    0x97742F, // 调研
    0x8F7F35, // 速记
    0x6B7F5C, // 灵感 = growth
    0x5E6E80, // 项目
    0x7C5E77, // 资料
    0x8B7A68, // 归档（内容语义色独立取值 · 不再随 SECONDARY 派生）
];

/// 标签下标 → 色值 helper。
pub fn tag_color(index: usize) -> u32 {
    TAG_COLORS[index % 7]
}

// ─────────────────────── 字体栈（DEC-66） ───────────────────────

/// UI 显示族：字标 PARCHORA / h1 / 详情标题（宋体 → Songti SC；DEC-68 实测裸标签不可解析）。
pub const FONT_DISPLAY: &str = "Songti SC";
/// UI 控件族：导航/按钮/菜单/chip/meta 等界面文字（宋体 → Songti SC）。
pub const FONT_BODY: &str = "Songti SC";
/// 内容族：笔记/剪贴板/待办等具体内容展示区（楷体 → Kaiti SC）。
pub const FONT_CONTENT: &str = "Kaiti SC";
/// 等宽（PG-B1 源码区 / 公式降级条 · 系统默认等宽 = Menlo，DEC-68 实测解析成功）。
pub const FONT_MONO: &str = "Menlo";

pub const FONT_SIZE_BODY: f32 = 16.0; // 正文 body-md（DEC-198：Hd 档严格落表 FHD 行下沿）
pub const FONT_SIZE_SMALL: f32 = 12.0; // small 说明文字
pub const FONT_SIZE_TAB: f32 = 13.0; // 视图切换条
pub const FONT_SIZE_TAB_ACTIVE: f32 = 15.0; // 视图切换条激活态（base 13 + 2）
pub const FONT_SIZE_META: f32 = 12.0; // 行 meta/时间戳（DEC-198：Caption 派生档落表）
pub const FONT_SIZE_H1: f32 = 26.0;
pub const FONT_SIZE_H2: f32 = 20.0; // 标题二（DEC-134 · 块级标题映射 1→26/2→20/3→18·DEC-198）
pub const FONT_SIZE_H3: f32 = 18.0; // 标题三（DEC-134 · DEC-198 表值域下沿校正）
pub const FONT_SIZE_WORDMARK: f32 = 33.0;
pub const FONT_SIZE_DISPLAY: f32 = 32.0; // 2rem
pub const FONT_SIZE_MONO: f32 = 14.0; // 等宽/代码（DEC-198：表 FHD 行等宽列下沿）

pub const FONT_WEIGHT_LIGHT: f32 = 300.0; // h1/正文字重（≤500 铁律）
pub const FONT_WEIGHT_REGULAR: f32 = 400.0;
pub const FONT_WEIGHT_MEDIUM: f32 = 500.0; // 字重阶梯上限（DESIGN.md：禁 600+）

// ─────────────────────── 尺寸（全量页面设计.md 修订 v3 + clamp，DEC-66） ───────────────────────

/// 主面板固定宽 360px（任何时刻 · DEC-129；不支持拖拽调宽）。
pub const PANEL_W: f32 = 360.0;
/// 主面板高 = 所在屏高比例 80%（随屏自适应）。
pub const PANEL_VH: f32 = 0.80;
/// 主面板高下限（窗口高度范围 480~840）。
pub const PANEL_H_MIN: f32 = 480.0;
/// 主面板高上限（同上；初始装配按屏钳制）。
pub const PANEL_H_MAX: f32 = 840.0;
/// 内容展示面板（编辑/预览）固定宽 480px。
pub const DETAIL_W: f32 = 480.0;
/// 内容展示面板高 = 所在屏高比例 70%（随屏自适应）。
pub const DETAIL_VH: f32 = 0.70;
/// 内容展示面板高下限（面板高度范围 480~780）。
pub const DETAIL_H_MIN: f32 = 480.0;
/// 内容展示面板高上限（同上）。
pub const DETAIL_H_MAX: f32 = 780.0;
/// 菜单栏菜单紧凑宽 300px。
pub const MENU_W: f32 = 300.0;
/// 独立设置窗口 430 基准。
pub const SETTINGS_W: f32 = 430.0;
/// FAB 直径 54px。
pub const FAB: f32 = 54.0;
/// FAB 右下停靠边距 16px。
pub const FAB_DOCK: f32 = 16.0;
/// Peek 消失宽限（秒 · REQ-PREVIEW-PEEK「预览消失保留 0.12s 消失宽限」；
/// Peek 窗移出的关闭路径采用同一宽限对称延伸）。
pub const PEEK_GRACE: f32 = 0.12;
pub const TOAST_MS: u64 = 1400; // Toast 自动消失（不常量驱动动画，仅语义标注）

// ───────────────────────── 动效令牌（parchora-shared.css :root --dur-* / --ease-ink） ─────────────────────────

/// 淡入/墨迹族动画时长基准 = `--dur-fast` 180ms（Toast 淡入等 `with_animation`
/// 动画族唯一口径）：hover/按下为即时态切换（gpui-fast 无 CSS transition ·
/// DESIGN.md L174），不属动画时长面；曲线 = `--ease-ink` cubic-bezier(.22,.61,.36,1)。
/// DESIGN.md 动效铁限（DESIGN.md L168）：仅「墨迹显现」与「淡入」两类，无弹跳、无缩放。
pub const MOTION_FAST_MS: u64 = 180;
/// 淡入族·浮层档 = `--dur-base` 320ms（sheet 等浮层淡入；档值随
/// parchora-shared.css :root，R184 建议 20 注释对齐）。
pub const MOTION_BASE_MS: u64 = 320;
/// 墨迹显现 = `--dur-slow` 550ms。
pub const MOTION_SLOW_MS: u64 = 550;

// ───────────────────────── 间距 / 圆角阶（DESIGN.md 8px 基线） ─────────────────────────

pub const SP_XS: f32 = 4.0;
pub const SP_SM: f32 = 8.0;
pub const SP_MD: f32 = 16.0;
pub const SP_LG: f32 = 24.0;
pub const SP_XL: f32 = 48.0;

/// 行文本截断（DEC-232 用户指令）：拾记行文本 ≤3 行 / 待办行标题 ≤2 行，
/// 超出省略号截断；截断面行高统一 1.5em。
pub const ROW_TEXT_LH: f32 = 1.5;
pub const ROW_CLAMP_COLLECT: usize = 3;
pub const ROW_CLAMP_TASK: usize = 2;

pub const RAD_SM: f32 = 6.0; // 按钮/输入/行控件/标签
pub const RAD_MD: f32 = 8.0; // 安静块
pub const RAD_LG: f32 = 16.0; // 底部浮层顶缘 / sheet

// ───────────────────────── 行高（CSS 原文，供显式复现） ─────────────────────────

pub const LH_BODY: f32 = 1.95; // 正文 15px × 1.95 ≈ 29px

// ───────────────────────── 对比度计算（AA 校验用） ─────────────────────────

/// sRGB 相对亮度（WCAG 定义）。
fn luminance(hex: u32) -> f64 {
    let lin = |c: f64| -> f64 {
        if c <= 0.04045 {
            c / 12.92
        } else {
            f64::powf((c + 0.055) / 1.055, 2.4)
        }
    };
    let r = lin(f64::from((hex >> 16) & 0xFF) / 255.0);
    let g = lin(f64::from((hex >> 8) & 0xFF) / 255.0);
    let b = lin(f64::from(hex & 0xFF) / 255.0);
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

/// WCAG 对比度比值（亮/暗自动取向）。
pub fn contrast(fg: u32, bg: u32) -> f64 {
    let l1 = luminance(fg);
    let l2 = luminance(bg);
    let (hi, lo) = if l1 >= l2 { (l1, l2) } else { (l2, l1) };
    (hi + 0.05) / (lo + 0.05)
}

// ───────────────────────── 测试 ─────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn palette_matches_design_md() {
        // DESIGN.md colors 17 色逐项锚定 + ACCENT 点缀金（2026-10-23 用户四色重构）。
        assert_eq!(PRIMARY, 0x191923); // 用户文字色
        assert_eq!(ON_PRIMARY, 0xF5EBD2); // 用户背景色（纸光）
        assert_eq!(PRESS, 0x0C0C12);
        assert_eq!(INK_DEEP, 0x07070C);
        assert_eq!(SECONDARY, 0x50515C);
        assert_eq!(MUTED, 0x5A5A64);
        assert_eq!(FAINT, 0xDFD2B0);
        assert_eq!(TERTIARY, 0x142346); // 用户辅助色（藏青）
        assert_eq!(NEUTRAL, 0xF5EBD2); // 用户背景色
        assert_eq!(SURFACE, 0xFBF4E4);
        assert_eq!(WARM, 0xEBDFC2);
        assert_eq!(BORDER, 0xD8C9A8);
        assert_eq!(BORDER_SOFT, 0xEFE5CD);
        assert_eq!(GROWTH, 0x6B7F5C);
        assert_eq!(NIGHT_SAGE, 0xD4AF37); // 点缀金
        assert_eq!(NIGHT_BG, 0x142346); // 用户辅助色
        assert_eq!(NIGHT_INK, 0xF5EBD2);
        assert_eq!(ACCENT, 0xD4AF37); // 用户点缀色
    }

    #[test]
    fn tag_palette_is_seven_unique_and_derived() {
        assert_eq!(TAG_NAMES.len(), 7);
        assert_eq!(TAG_COLORS.len(), 7);
        // 灵感 = growth（派生关系保持）；归档已独立取值
        //（2026-10-23 主色系重构：SECONDARY 换蓝灰族，标签 7 色为内容语义色不随换色）。
        assert_eq!(tag_color(3), GROWTH);
        assert_eq!(tag_color(6), 0x8B7A68);
        // 7 色互异。
        let mut seen: Vec<u32> = Vec::new();
        for color in TAG_COLORS {
            assert!(!seen.contains(&color), "tag 同色 {color:#06x}");
            seen.push(color);
        }
    }

    #[test]
    fn contrast_is_aa_grade() {
        // 文字/纸面：#191923 / #FBF4E4 WCAG 实测 15.91，AA+ 。
        let c = contrast(PRIMARY, SURFACE);
        assert!((15.5..=16.3).contains(&c), "primary/surface = {c}");
        // Muted 派生 AA 值（18px 以下次级文字）：surface 实测 6.22 / bg 实测 5.74。
        let m = contrast(MUTED, SURFACE);
        assert!((5.7..=6.5).contains(&m), "muted/surface = {m}");
        assert!(
            contrast(MUTED, NEUTRAL) >= 5.5,
            "muted/bg = {}",
            contrast(MUTED, NEUTRAL)
        );
        // 夜读反白：night-ink / 夜读藏青底 13.03。
        let n = contrast(NIGHT_INK, NIGHT_BG);
        assert!(n > 7.0, "night ink/bg = {n}");
        // 落墨实心按钮：纸光字 / 墨底 14.69。
        assert!((14.3..=15.0).contains(&contrast(ON_PRIMARY, PRIMARY)));
        // 点缀金只上深面：金 / 藏青底 7.35（浅底禁止金文字——金/米黄仅 1.77）。
        assert!(contrast(ACCENT, NIGHT_BG) > 4.5, "accent/night = {c}");
        assert!(
            contrast(ACCENT, NEUTRAL) < 3.0,
            "金在浅底不作文字 = {}",
            contrast(ACCENT, NEUTRAL)
        );
        // 夜读批量操作条：点缀金链接 / 藏青底。
        assert!(contrast(NIGHT_SAGE, NIGHT_BG) > 4.5);
    }

    #[test]
    fn size_system_v3() {
        assert_eq!(PANEL_W, 360.0);
        assert_eq!(PANEL_VH, 0.80);
        assert_eq!(DETAIL_W, 480.0);
        assert_eq!(DETAIL_VH, 0.70);
        assert_eq!(MENU_W, 300.0);
        assert_eq!(SETTINGS_W, 430.0);
        assert_eq!(FAB, 54.0);
        assert_eq!(FAB_DOCK, 16.0);
    }

    #[test]
    fn window_height_clamp_bounds() {
        // 主窗口 480~840 · 内容面板 480~780（DEC-66）。
        assert_eq!((PANEL_H_MIN, PANEL_H_MAX), (480.0, 840.0));
        assert_eq!((DETAIL_H_MIN, DETAIL_H_MAX), (480.0, 780.0));
        // 区间自洽：min < max（编译期恒真，const 块留痕）。
        const _: () = const { assert!(PANEL_H_MIN < PANEL_H_MAX && DETAIL_H_MIN < DETAIL_H_MAX) };
    }

    #[test]
    fn motion_tokens_match_shared_css() {
        // 动效令牌锚定（parchora-shared.css :root）：--dur-fast .18s / --dur-base .32s / --dur-slow .55s。
        assert_eq!(
            (MOTION_FAST_MS, MOTION_BASE_MS, MOTION_SLOW_MS),
            (180, 320, 550)
        );
    }

    #[test]
    fn spacing_radius_ladder() {
        assert_eq!(
            (SP_XS, SP_SM, SP_MD, SP_LG, SP_XL),
            (4.0, 8.0, 16.0, 24.0, 48.0)
        );
        assert_eq!((RAD_SM, RAD_MD, RAD_LG), (6.0, 8.0, 16.0));
        assert_eq!(PEEK_GRACE, 0.12);
        assert_eq!(TOAST_MS, 1400);
        // 行文本截断令牌（DEC-232）。
        assert_eq!(
            (ROW_TEXT_LH, ROW_CLAMP_COLLECT, ROW_CLAMP_TASK),
            (1.5, 3, 2)
        );
    }

    #[test]
    fn font_stacks_user_instructed() {
        // 字体栈（DEC-66/68）：
        // 内容=楷体→Kaiti SC · UI=宋体→Songti SC · 代码=系统默认等宽→Menlo。
        // DEC-68 实测（app/target/fontprobe）：裸标签「宋体/楷体」不可解析（落 Helvetica），
        // 故使用 macOS 10.13+ 捆绑标准变体族名；缺族时 CoreText 隐式回退系统默认
        // （偏差 D-02 承接：「不存在的字体回退系统默认」即由该机制实现）。
        assert_eq!(FONT_DISPLAY, "Songti SC");
        assert_eq!(FONT_BODY, "Songti SC");
        assert_eq!(FONT_CONTENT, "Kaiti SC");
        assert_eq!(FONT_MONO, "Menlo");
        // 字重阶梯 ≤500（DESIGN.md 禁 600+）——编译期已可判，测试仅作口径留痕。
        const _: () = const { assert!(FONT_WEIGHT_MEDIUM <= 500.0) };
    }
}
