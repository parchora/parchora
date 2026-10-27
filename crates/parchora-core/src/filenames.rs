//! 拒绝式文件名校验（TASK-100 / TASK-003 域）。
//!
//! 锚点：REQ-STORE-FILENAME / SCN-STORE-NAME-REJECT-CLASSES（C6）；
//! design §3.3（`core::filenames::validate`，「正则与字符串判定重写且单测覆盖全部拒绝类目」）。
//!
//! 命名口径：以 tasks TASK-100 与 design §3.3 的 `validate` 为准；§3.4 草案的
//! `validate_filename` 名称不再另立（避免双口径，偏差登记见 DEC-63）。

/// 拒绝类目（逐类可观察原因——SCN-STORE-NAME-REJECT-CLASSES「明确提示对应原因」）
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FilenameError {
    /// 空名
    Empty,
    /// `.`（当前目录）
    CurrentDir,
    /// `..`（上级目录）
    ParentDir,
    /// 含 `/`
    ContainsSlash,
    /// 含 `:`
    ContainsColon,
    /// 含控制字符（Unicode Cc，含 `\0`）
    ContainsControl,
    /// 超过 255 字节（UTF-8 字节数）
    TooLong { byte_len: usize },
}

impl core::fmt::Display for FilenameError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            FilenameError::Empty => write!(f, "文件名不能为空"),
            FilenameError::CurrentDir => write!(f, "文件名不能为 ."),
            FilenameError::ParentDir => write!(f, "文件名不能为 .."),
            FilenameError::ContainsSlash => write!(f, "文件名不能包含 /"),
            FilenameError::ContainsColon => write!(f, "文件名不能包含 :"),
            FilenameError::ContainsControl => write!(f, "文件名不能包含控制字符"),
            FilenameError::TooLong { byte_len } => {
                write!(f, "文件名超过 255 字节（当前 {byte_len} 字节）")
            }
        }
    }
}

/// C6 拒绝式校验：拒绝空名 / `.` / `..` / 含 `/` / 含 `:` / 含控制字符 / 超过 255 字节。
///
/// 拒绝时给出逐类原因，不做任何静默改名（改名是调用方职责——如 Untitled 序列）。
/// 检查顺序即拒绝原因优先级（规格未规定多错并存时取哪类，按「先特例后字符类再长度」
/// 自然序，登记见 DEC-63）。
pub fn validate(name: &str) -> Result<(), FilenameError> {
    if name.is_empty() {
        return Err(FilenameError::Empty);
    }
    if name == "." {
        return Err(FilenameError::CurrentDir);
    }
    if name == ".." {
        return Err(FilenameError::ParentDir);
    }
    if name.contains('/') {
        return Err(FilenameError::ContainsSlash);
    }
    if name.contains(':') {
        return Err(FilenameError::ContainsColon);
    }
    if name.chars().any(|c| c.is_control()) {
        return Err(FilenameError::ContainsControl);
    }
    let byte_len = name.len(); // UTF-8 字节数
    if byte_len > 255 {
        return Err(FilenameError::TooLong { byte_len });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_each_class() {
        // SCN-STORE-NAME-REJECT-CLASSES 七类逐一
        assert_eq!(validate(""), Err(FilenameError::Empty));
        assert_eq!(validate("."), Err(FilenameError::CurrentDir));
        assert_eq!(validate(".."), Err(FilenameError::ParentDir));
        assert_eq!(validate("a/b"), Err(FilenameError::ContainsSlash));
        assert_eq!(validate("a:b"), Err(FilenameError::ContainsColon));
        assert_eq!(validate("a\0b"), Err(FilenameError::ContainsControl));
        assert_eq!(validate("a\u{7f}b"), Err(FilenameError::ContainsControl));
        assert_eq!(validate("a\nb"), Err(FilenameError::ContainsControl));
    }

    #[test]
    fn dot_inside_longer_name_is_valid() {
        // 仅整名 "." / ".." 拒绝；中间含点的普通名合法
        assert_eq!(validate("a.b"), Ok(()));
        assert_eq!(validate("2026.10.17.md"), Ok(()));
        assert_eq!(validate("..."), Ok(())); // "..." 非 "." / ".."
    }

    #[test]
    fn byte_boundary_255() {
        // 255 字节（含 .md 全名按调用方传入口径）恰可、256 拒绝
        assert_eq!(validate("a".repeat(255).as_str()), Ok(()));
        let err = validate("a".repeat(256).as_str()).unwrap_err();
        assert_eq!(err, FilenameError::TooLong { byte_len: 256 });
    }

    #[test]
    fn unicode_titles_count_utf8_bytes() {
        // Unicode 标题：3 字节汉字 ×85 = 255 恰可；×86 = 258 拒绝
        assert_eq!(validate("漢".repeat(85).as_str()), Ok(()));
        let err = validate("漢".repeat(86).as_str()).unwrap_err();
        assert_eq!(err, FilenameError::TooLong { byte_len: 258 });
        // Emoji 4 字节
        assert_eq!(validate("🎉".repeat(63).as_str()), Ok(())); // 252
        assert!(validate("🎉".repeat(64).as_str()).is_err()); // 256
    }

    #[test]
    fn valid_names_pass() {
        for name in ["Untitled", "Untitled 2", "会议纪要", "a-b_c.md", "速记/"] {
            // "速记/" 含斜杠 → ContainsSlash 拒绝（与错误类型口径一致）
            if name.contains('/') {
                assert_eq!(validate(name), Err(FilenameError::ContainsSlash));
            } else {
                assert_eq!(validate(name), Ok(()));
            }
        }
    }
}
