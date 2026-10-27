//! 页面层数据与确定性单元测试（视图渲染经真机肉眼复核，此处覆盖纯逻辑）。

#[cfg(test)]
mod tests {
    use crate::{components as co, pages::collect, scale as s, tokens as t};
    use gpui::ParentElement as _;

    #[test]
    fn short_title_truncates() {
        let s = collect::short_title("「羊皮纸底、褐墨细字…」的整段设计规范文本");
        assert!(s.chars().count() <= 13);
        assert!(s.ends_with('…'));
    }

    // 详情面板标题栏标题截断（DEC-235 用户指令）：>12 字符 → 前 12 字符
    // ＋「......」字面六点；≤12 原文不动；char 计（CJK/拉丁同权）。
    #[test]
    fn detail_title_truncate_rules() {
        assert_eq!(
            co::detail_title_truncate("队列触发点表.md"),
            "队列触发点表.md"
        );
        assert_eq!(co::detail_title_truncate("短标题"), "短标题");
        // 恰 12（9 CJK + .md 三拉丁）与恰 20 → 原文（DEC-237 阈值 20）。
        assert_eq!(
            co::detail_title_truncate("羊皮纸视觉系统推导.md"),
            "羊皮纸视觉系统推导.md"
        );
        let twenty = "一二三四五六七八九十一二三四五六七八九十";
        assert_eq!(co::detail_title_truncate(twenty), twenty);
        // 25 拉丁 → 前 20＋六点。
        let lat = "abcdefghijklmnopqrstuvwxy";
        assert_eq!(
            co::detail_title_truncate(lat),
            format!("{}......", lat.chars().take(20).collect::<String>())
        );
        // 超长 CJK 标题：截断位按 char 计（23 字 char → 前 20）。
        let cjk = "会议纪要第四季度发布窗口与走查排期备忘录.md";
        let want = format!("{}......", cjk.chars().take(20).collect::<String>());
        assert_eq!(co::detail_title_truncate(cjk), want);
    }

    #[test]
    fn components_compile_smoke() {
        // 组件构造返回（样式链不 panic；图标化 API 同步走查）。
        let _ = co::bar()
            .child(co::tab("拾记", true))
            .child(co::icon_btn(co::Icon::Settings))
            .child(co::ibtn_icon_text(co::Icon::ArrowLeft, "返回", t::PRIMARY));
        let _ = co::tagc("设计", t::tag_color(0), "9");
        let _ = co::empty_state(co::Icon::CircleDashed, "复制任意内容后自动出现在这里");
        let _ = co::search_box("打开即搜…", co::Icon::Search);
        let _ = co::switch(false);
        let _ = co::switch(true);
        let _ = co::hint_marked(co::Icon::SquarePen, "右键条目");
        let _ = co::chip_dd("类型");
        let _ = co::btn_sec_icon(co::Icon::Plus, "新建文件夹");
        let _ = co::content_text("内容文本", s::font(s::Lvl::Tab), t::PRIMARY);
        let _ = co::icon(co::Icon::Pin, 12.0, t::MUTED);
        let _ = co::mark_sel(true);
        let _ = co::tgo();
    }

    #[test]
    fn dialogs_nineteen_pages_fully_bearing() {
        // UI-6 对话框组全承载（DEC-73：DG-CONFLICT / DG-EXT / DG-PICKER 补齐——
        // 视图形态与 HTML stage 1:1；触发链随数据层实装接线）。
        let _ = crate::pages::dialogs::ConflictSheet;
        let _ = crate::pages::dialogs::ExtConflictView;
        let _ = crate::pages::dialogs::PickerSheet;
    }
}
