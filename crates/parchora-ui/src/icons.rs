//! 图标层（UI-1 扩展）——lucide 线性图标内联集（lucide-static 1.59.0 原文）。
//!
//! 来源锁定：`https://unpkg.com/lucide-static@1.59.0/icons/<name>.svg`，构建期
//! `include_bytes!` 原文内联（零新增依赖——规范 9；数据为 unpkg 逐字节原文，禁止凭记忆改写）。
//! 着色：gpui-fast `paint_svg` 为单色 alpha-mask 渲染，图标颜色由元素 `.text_color(...)`
//! 承载，SVG 内 `currentColor` 语义天然成立。
//! 语义选用与覆盖范围见 docs（DEC-66）：图标仅替换设计稿原 glyph 位，不新增装饰位。

use gpui::{px, rgb, svg, Styled as _, Svg};

/// 应用内图标集（lucide 语义名；变体名 = 文件名 PascalCase）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Icon {
    ArrowLeft,
    ArrowUpRight,
    ChevronDown,
    ChevronRight,
    ChevronUp,
    Circle,
    CircleDot,
    CircleDashed,
    Code,
    Copy,
    ExternalLink,
    Feather,
    File,
    FileText,
    Folder,
    Monitor,
    Pin,
    Plus,
    Search,
    Settings,
    SquarePen,
    Star,
    Trash2,
    Type,
}

/// 内联字节（lucide-static 1.59.0 原文；`match` 穷尽保证新增变体必挂文件）。
fn data(i: Icon) -> &'static [u8] {
    match i {
        Icon::ArrowLeft => include_bytes!("icons/arrow-left.svg"),
        Icon::ArrowUpRight => include_bytes!("icons/arrow-up-right.svg"),
        Icon::ChevronDown => include_bytes!("icons/chevron-down.svg"),
        Icon::ChevronRight => include_bytes!("icons/chevron-right.svg"),
        Icon::ChevronUp => include_bytes!("icons/chevron-up.svg"),
        Icon::Circle => include_bytes!("icons/circle.svg"),
        Icon::CircleDot => include_bytes!("icons/circle-dot.svg"),
        Icon::CircleDashed => include_bytes!("icons/circle-dashed.svg"),
        Icon::Code => include_bytes!("icons/code.svg"),
        Icon::Copy => include_bytes!("icons/copy.svg"),
        Icon::ExternalLink => include_bytes!("icons/external-link.svg"),
        Icon::Feather => include_bytes!("icons/feather.svg"),
        Icon::File => include_bytes!("icons/file.svg"),
        Icon::FileText => include_bytes!("icons/file-text.svg"),
        Icon::Folder => include_bytes!("icons/folder.svg"),
        Icon::Monitor => include_bytes!("icons/monitor.svg"),
        Icon::Pin => include_bytes!("icons/pin.svg"),
        Icon::Plus => include_bytes!("icons/plus.svg"),
        Icon::Search => include_bytes!("icons/search.svg"),
        Icon::Settings => include_bytes!("icons/settings.svg"),
        Icon::SquarePen => include_bytes!("icons/square-pen.svg"),
        Icon::Star => include_bytes!("icons/star.svg"),
        Icon::Trash2 => include_bytes!("icons/trash-2.svg"),
        Icon::Type => include_bytes!("icons/type.svg"),
    }
}

/// 构造一枚线性图标（`size` 单位 px；颜色由文本色通道承载）。
pub fn icon(i: Icon, size: f32, color: u32) -> Svg {
    svg()
        .data(data(i))
        .text_color(rgb(color))
        .size(px(size))
        .flex_none()
}

/// 构造一枚不显式着色的线性图标（颜色经文本样式通道继承）。
/// 供组级悬停变色场景（`crate::components::tgo`：祖先 refinement 改 text_color
/// 即级联到本图标——`svg` 涂装取 `style.text.color`，显式色会压住继承，故此变体不设色）。
pub fn icon_inherited(i: Icon, size: f32) -> Svg {
    svg().data(data(i)).size(px(size)).flex_none()
}

// ───────────────────────── 测试 ─────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// 全量图标登记（语义名 ↔ 文件名）；新增图标须同步此表。
    const SET: [(Icon, &str); 24] = [
        (Icon::ArrowLeft, "arrow-left"),
        (Icon::ArrowUpRight, "arrow-up-right"),
        (Icon::ChevronDown, "chevron-down"),
        (Icon::ChevronRight, "chevron-right"),
        (Icon::ChevronUp, "chevron-up"),
        (Icon::Circle, "circle"),
        (Icon::CircleDot, "circle-dot"),
        (Icon::CircleDashed, "circle-dashed"),
        (Icon::Code, "code"),
        (Icon::Copy, "copy"),
        (Icon::ExternalLink, "external-link"),
        (Icon::Feather, "feather"),
        (Icon::File, "file"),
        (Icon::FileText, "file-text"),
        (Icon::Folder, "folder"),
        (Icon::Monitor, "monitor"),
        (Icon::Pin, "pin"),
        (Icon::Plus, "plus"),
        (Icon::Search, "search"),
        (Icon::Settings, "settings"),
        (Icon::SquarePen, "square-pen"),
        (Icon::Star, "star"),
        (Icon::Trash2, "trash-2"),
        (Icon::Type, "type"),
    ];

    #[test]
    fn icon_set_is_complete_and_valid() {
        assert_eq!(SET.len(), 24, "图标总数与登记表一致");
        for (i, name) in SET {
            let b = data(i);
            // lucide-static 原文带 ISC 许可注释头，<svg> 在其后（usvg 对 XML 注释天然兼容）。
            assert!(b.windows(4).any(|w| w == b"<svg"), "{name} 缺 <svg");
            assert!(b.windows(6).any(|w| w == b"</svg>"), "{name} 缺收尾");
            assert!(b.windows(5).any(|w| w == b"xmlns"), "{name} 缺 xmlns");
            assert!(
                b.windows(21).any(|w| w == b"lucide-static v1.59.0"),
                "{name} 非 1.59.0 原文"
            );
        }
    }
}
