// parchora-files —— 知识内容纯 .md 文件存储（原子写/单写者/扫描对账，design §2.6/§3.3）。
// 边界不变量：files 接口不含 DB 类型（编译期强制，TASK-200）。
//
// 最小实装（DEC-129 · DEC-130）：
// - sanitize_note_title：题名净化为文件名主干——标题为唯一命名源：trim + 控制字符剥除，
//   '/'（macOS 唯一非法路径符）置换 '-'，
//   取前 20 字符（CJK 按字符计；不足取全部；空时兜底「未命名」）。
// - write_note：单文件原子写（tmp+rename+fsync 文件 · design §2.6 口径）；
//   同名冲突按 EdgeMark 惯例「标题 2.md / 标题 3.md」顺延，不覆盖既有笔记。
// - rewrite_note：已存稿同名覆写（tmp+rename+fsync · 无顺延）——续存语义。
// - trash_note：笔记移入回收站（DEC-164 · REQ-STORE-TRASH 文件侧最小实装）——
//   原样 rename 移入调用方定位的 .trash 目录（files 同级口径 · REQ-STORE-IMAGES
//   保留名域），同名冲突顺延不覆盖；DB trash 记录/uuid 前缀命名/60 天启动清理
//   随 trash crate 接线（TASK-50x · 偏差台账 DEC-164）。
// - purge_trash_note / empty_trash：回收站物理删除（DEC-195 · 仅回收站域删除
//   链用）：单件 remove_file（缺源 NotFound 上抛）/ 清空全部 .md（点前缀与子
//   目录不动 · 返回删除数）。
// 扫描对账/单写者串行/共置图片目录等余下面维持 defer（TASK-200 在册不动）。

use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

/// 标题兑底（题名净化后为空时；与 editor 渲染态 fallback 同词）。
const FALLBACK_TITLE: &str = "未命名";
/// 净化题名截断长度（前 20 个字符）。
const TITLE_MAX_CHARS: usize = 20;

/// 题名净化为文件名主干（DEC-130 · 标题为唯一命名
/// 源）：trim + 全文控制字符剥除（不可见字符面），'/' 置换为
/// '-'（macOS 文件名合法化），取前 20 个字符（按字符计，CJK 安全；不足取全部）；
/// 结果为空时兑底「未命名」。
pub fn sanitize_note_title(title: &str) -> String {
    let cleaned: String = title
        .chars()
        .filter(|c| !c.is_control())
        .map(|c| if c == '/' { '-' } else { c })
        .collect();
    let name: String = cleaned.trim().chars().take(TITLE_MAX_CHARS).collect();
    let name = name.trim().to_string();
    if name.is_empty() {
        FALLBACK_TITLE.to_string()
    } else {
        name
    }
}

/// 单文件原子写：内容写 `.{name}.tmp` 兄弟临时文件 → fsync → rename 到终名。
/// 同名冲突不覆盖：按「标题 2.md / 标题 3.md …」顺延取首个空位（EdgeMark
/// Untitled 2 同例）。返回终名绝对路径。临时文件残留由 rename 原子性保证无痕。
pub fn write_note(root: &Path, title: &str, body: &str) -> io::Result<PathBuf> {
    fs::create_dir_all(root)?;
    let final_name = first_free_name(root, title);
    let tmp = root.join(format!(".{final_name}.tmp"));
    // 崩溃残留临时文件自愈（rewrite_note 同款 · DEC-159 · R184 F-183-07）：
    // create_new 对已存在 tmp 永久失败（同题重试路无自愈面）；正常原子写链
    // 路 tmp 无痕（rename 保证），存留即崩溃残留判据，安全移除后重试。
    if tmp.exists() {
        let _ = fs::remove_file(&tmp);
    }
    {
        let mut f = OpenOptions::new().write(true).create_new(true).open(&tmp)?;
        f.write_all(body.as_bytes())?;
        f.sync_all()?;
    }
    let final_path = root.join(&final_name);
    fs::rename(&tmp, &final_path)?;
    Ok(final_path)
}

/// 取根目录下首个空闲的「标题.md / 标题 2.md …」文件名（确定性顺延）。
fn first_free_name(root: &Path, title: &str) -> String {
    let base = format!("{title}.md");
    if !root.join(&base).exists() {
        return base;
    }
    for n in 2.. {
        let name = format!("{title} {n}.md");
        if !root.join(&name).exists() {
            return name;
        }
    }
    unreachable!("first_free_name loop is infinite by construction");
}

/// 笔记移入回收站（DEC-164）：`root/<name>.md` 原样 rename 移入 `trash_dir/`。
/// 内容零读写（rename 同卷原子移动，mtime 保留——60 天保留期计时基座）；同名
/// 冲突按「题名 2.md / 题名 3.md …」顺延（write_note 同口径），不覆盖既有回收
/// 站条目。`name` 传文件名词干原值（题名已是命名源，勿再净化——外部落盘的同
/// 名文件以盘名为准）。
pub fn trash_note(root: &Path, trash_dir: &Path, name: &str) -> io::Result<PathBuf> {
    let src = root.join(format!("{name}.md"));
    if !src.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("笔记不存在: {}", src.display()),
        ));
    }
    fs::create_dir_all(trash_dir)?;
    let mut final_name = format!("{name}.md");
    let mut dst = trash_dir.join(&final_name);
    let mut n = 2;
    while dst.exists() {
        final_name = format!("{name} {n}.md");
        dst = trash_dir.join(&final_name);
        n += 1;
    }
    fs::rename(&src, &dst)?;
    Ok(dst)
}

/// 回收站单件物理删除（DEC-195 · 仅回收站内删除链用；根面笔记永不经此路）：
/// `trash_dir/<name>.md` remove_file；源不存在 = NotFound 上抛不吞（调用方
/// stderr 自证 · 失败面用户可见可重试惯例同 trash_note）。
pub fn purge_trash_note(trash_dir: &Path, name: &str) -> io::Result<()> {
    let p = trash_dir.join(format!("{name}.md"));
    if !p.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("回收条目不存在: {}", p.display()),
        ));
    }
    fs::remove_file(&p)?;
    Ok(())
}

/// 清空回收站（DEC-195 · DG-CONFIRM 确认后唯一入口）：删 `trash_dir` 下全部
/// 普通 `.md`（点前缀临时残留与子目录不动）；目录不存在＝空集（Ok(0) 幂等）。
/// 返回删除数（面板确认 Toast「N 项」承载）。
pub fn empty_trash(trash_dir: &Path) -> io::Result<usize> {
    let rd = match fs::read_dir(trash_dir) {
        Ok(rd) => rd,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(0),
        Err(e) => return Err(e),
    };
    let mut n = 0usize;
    for entry in rd.flatten() {
        let p = entry.path();
        let Some(name) = p.file_name().and_then(|s| s.to_str()) else {
            continue;
        };
        if name.starts_with('.') || !name.to_ascii_lowercase().ends_with(".md") || !p.is_file() {
            continue;
        }
        fs::remove_file(&p)?;
        n += 1;
    }
    Ok(n)
}

/// 已存稿同名覆写（DEC-130 · 续存语义）：内容写 `.{name}.tmp` 兄弟
/// 临时文件 → fsync → rename 覆写同 `{name}.md`（write_note 的顺延语义不适用——
/// 已存稿必须原地更新，防「Untitled N」增殖与静默丢编辑）。返回终名绝对路径。
pub fn rewrite_note(root: &Path, name: &str, body: &str) -> io::Result<PathBuf> {
    fs::create_dir_all(root)?;
    let tmp = root.join(format!(".{name}.tmp"));
    // 崩溃残留临时文件清理（DEC-159）：create_new 对已存在 tmp 永久失败（失败
    // 驻留重试路径会无限命中同错）；正常原子写链路 tmp 无痕（rename 保证），
    // 存留即崩溃残留判据，安全移除后重试可自愈。
    if tmp.exists() {
        let _ = fs::remove_file(&tmp);
    }
    {
        let mut f = OpenOptions::new().write(true).create_new(true).open(&tmp)?;
        f.write_all(body.as_bytes())?;
        f.sync_all()?;
    }
    let final_path = root.join(format!("{name}.md"));
    fs::rename(&tmp, &final_path)?;
    Ok(final_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试专用临时根（无三方依赖：进程号+纳秒唯一目录，用后即清）。
    fn temp_root(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "parchora-files-test-{}-{}-{}",
            std::process::id(),
            tag,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn sanitize_title_maps_slash_and_control_chars() {
        assert_eq!(sanitize_note_title("a/b\\c:d"), "a-b\\c:d");
        // 控制字符剥除（不可见字符面）。
        assert_eq!(sanitize_note_title("标\u{7}题"), "标题");
    }

    #[test]
    fn sanitize_title_trims_and_truncates_20_chars_cjk_safe() {
        assert_eq!(sanitize_note_title("  前后空白  "), "前后空白");
        let long = "一二三四五六七八九十一二三四五六七八九十一二三";
        assert_eq!(
            sanitize_note_title(long),
            "一二三四五六七八九十一二三四五六七八九十"
        );
    }

    #[test]
    fn sanitize_title_falls_back_on_empty() {
        // 空题名 / 纯空白 → 兑底「未命名」。
        assert_eq!(sanitize_note_title(""), FALLBACK_TITLE);
        assert_eq!(sanitize_note_title("   "), FALLBACK_TITLE);
    }

    /// 测试专用临时存储容器（trash 测试：容器内 files/ ＋ 同级 .trash/ 生产同
    /// 构布局，用后整容器即清）。
    fn temp_storage(tag: &str) -> (PathBuf, PathBuf, PathBuf) {
        let container = temp_root(tag);
        let root = container.join("files");
        let trash = container.join(".trash");
        fs::create_dir_all(&root).unwrap();
        (container, root, trash)
    }

    #[test]
    fn trash_note_moves_file_into_sibling_trash_dir() {
        let (container, root, trash) = temp_storage("trash-move");
        let src = write_note(&root, "笔记", "第一篇").unwrap();
        let dst = trash_note(&root, &trash, "笔记").unwrap();
        assert_eq!(dst, trash.join("笔记.md"));
        assert!(!src.exists(), "原位文件应消失");
        assert_eq!(fs::read_to_string(&dst).unwrap(), "第一篇");
        // 文件侧回收语义核验：根面已无该 md（扫描不会再列出）。
        let rest: Vec<_> = fs::read_dir(&root)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        assert!(!rest.iter().any(|n| n == "笔记.md"), "根面残留: {rest:?}");
        fs::remove_dir_all(&container).unwrap();
    }

    #[test]
    fn trash_note_collides_suffices_and_missing_src_fails() {
        let (container, root, trash) = temp_storage("trash-dedupe");
        let p1 = write_note(&root, "甲", "第一稿").unwrap();
        let _ = trash_note(&root, &trash, "甲").unwrap();
        // 同名再删（同名文件不会共存于 root——此处模拟外部重建后二删）：顺延不覆盖。
        fs::write(&p1, "重建稿").unwrap();
        let dst2 = trash_note(&root, &trash, "甲").unwrap();
        assert_eq!(dst2, trash.join("甲 2.md"));
        assert_eq!(fs::read_to_string(&dst2).unwrap(), "重建稿");
        assert_eq!(
            fs::read_to_string(trash.join("甲.md")).unwrap(),
            "第一稿",
            "既有回收条目不被覆盖"
        );
        // 源文件不存在 → NotFound（不吞错）。
        let err = trash_note(&root, &trash, "不存在").unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::NotFound);
        fs::remove_dir_all(&container).unwrap();
    }

    #[test]
    fn purge_trash_note_removes_single_and_reports_missing() {
        let (container, root, trash) = temp_storage("trash-purge");
        let _ = write_note(&root, "甲", "第一篇").unwrap();
        let _ = write_note(&root, "乙", "第二篇").unwrap();
        trash_note(&root, &trash, "甲").unwrap();
        trash_note(&root, &trash, "乙").unwrap();
        purge_trash_note(&trash, "甲").unwrap();
        assert!(!trash.join("甲.md").exists(), "回收内单件物理删除");
        assert!(trash.join("乙.md").exists(), "其余条目不受累");
        let err = purge_trash_note(&trash, "甲").unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::NotFound, "重删缺源不吞");
        fs::remove_dir_all(&container).unwrap();
    }

    #[test]
    fn empty_trash_removes_all_md_keeps_dot_and_missing_dir_ok() {
        let (container, root, trash) = temp_storage("trash-empty");
        for t in ["甲", "乙", "丙"] {
            let _ = write_note(&root, t, &format!("{t} 篇")).unwrap();
            trash_note(&root, &trash, t).unwrap();
        }
        fs::write(trash.join(".甲.md.tmp"), "残留").unwrap();
        assert_eq!(empty_trash(&trash).unwrap(), 3, "返回删除数");
        assert!(trash.join(".甲.md.tmp").exists(), "点前缀残留不动");
        assert_eq!(empty_trash(&trash).unwrap(), 0, "已空再清幂等");
        // 目录不存在 = 空集（Ok(0)）。
        let missing = container.join(".no-trash");
        assert_eq!(empty_trash(&missing).unwrap(), 0);
        fs::remove_dir_all(&container).unwrap();
    }

    #[test]
    fn rewrite_note_overwrites_in_place() {
        let root = temp_root("rewrite");
        let p = write_note(&root, "笔记", "旧内容").unwrap();
        let q = rewrite_note(&root, "笔记", "新内容").unwrap();
        assert_eq!(q, root.join("笔记.md"));
        assert_eq!(fs::read_to_string(&q).unwrap(), "新内容");
        assert_eq!(
            fs::read_to_string(&p).unwrap(),
            "新内容",
            "同一路径原地覆写"
        );
        // 无临时残留（rename 原子收口）。
        let stray: Vec<_> = fs::read_dir(&root)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|n| n.starts_with('.'))
            .collect();
        assert!(stray.is_empty(), "临时文件残留: {stray:?}");
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn rewrite_note_self_heals_stale_tmp() {
        // 崩溃残留 .tmp（DEC-159）：create_new 对已存在 tmp 永久失败；存留即
        // 残留判据 → 移除后自愈（失败驻留重试路径可收敛，不无限命中同错）。
        let root = temp_root("stale-tmp");
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join(".笔记.tmp"), "陈年残骸").unwrap();
        let q = rewrite_note(&root, "笔记", "新内容").expect("残留 tmp 应自愈");
        assert_eq!(fs::read_to_string(&q).unwrap(), "新内容");
        let stray: Vec<_> = fs::read_dir(&root)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|n| n.starts_with('.'))
            .collect();
        assert!(stray.is_empty(), "残留应被清扫: {stray:?}");
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn write_note_self_heals_stale_tmp() {
        // 崩溃残留 .tmp 自愈（R184 F-183-07 · rewrite_note 同款）：同题名重试
        // 不被「create_new 对已存在 tmp 永久失败」卡死——残留移除后走正常原子写。
        let root = temp_root("write-stale-tmp");
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join(".会议纪要.md.tmp"), "陈年残骸").unwrap();
        let p = write_note(&root, "会议纪要", "# 会议纪要\n正文").expect("残留 tmp 应自愈");
        assert_eq!(p, root.join("会议纪要.md"));
        assert_eq!(fs::read_to_string(&p).unwrap(), "# 会议纪要\n正文");
        let stray: Vec<_> = fs::read_dir(&root)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|n| n.starts_with('.'))
            .collect();
        assert!(stray.is_empty(), "残留应被清扫: {stray:?}");
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn write_note_creates_file_with_exact_content() {
        let root = temp_root("exact");
        let p = write_note(&root, "会议纪要", "# 会议纪要\n正文").unwrap();
        assert_eq!(p, root.join("会议纪要.md"));
        assert_eq!(fs::read_to_string(&p).unwrap(), "# 会议纪要\n正文");
        // 无临时残留（rename 原子收口）。
        let stray: Vec<_> = fs::read_dir(&root)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|n| n.starts_with('.'))
            .collect();
        assert!(stray.is_empty(), "临时文件残留: {stray:?}");
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn write_note_dedupes_conflicts_without_overwrite() {
        let root = temp_root("dedupe");
        let p1 = write_note(&root, "笔记", "第一篇").unwrap();
        let p2 = write_note(&root, "笔记", "第二篇").unwrap();
        assert_eq!(p1, root.join("笔记.md"));
        assert_eq!(p2, root.join("笔记 2.md"));
        assert_eq!(
            fs::read_to_string(&p1).unwrap(),
            "第一篇",
            "既有笔记不被覆盖"
        );
        assert_eq!(fs::read_to_string(&p2).unwrap(), "第二篇");
        let p3 = write_note(&root, "笔记", "第三篇").unwrap();
        assert_eq!(p3, root.join("笔记 3.md"));
        fs::remove_dir_all(&root).unwrap();
    }
}
