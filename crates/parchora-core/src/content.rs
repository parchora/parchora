//! 三态内容与实体的强类型定义（TASK-108）。
//!
//! 锚点：REQ-CONTENT-3STATE（Clip/Note/Task 三态）；design §3.1 不变量 4（core 类型不含持久化类型）；
//! design §3.2 DDL（clips/notes/projects 的持久化投影）；DEC-15 + v1.3a（source）。

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
// Uuid 随公共 API 再导出（投影消费方构造/解析实体 id 无需自引 uuid crate）。
pub use uuid::Uuid;

/// 三态内容（REQ-CONTENT-3STATE；DB `pinboard_entries.kind` CHECK 值域 `clip/note/task`）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContentKind {
    Clip,
    Note,
    Task,
}

/// 8 类内容（REQ-CAPTURE-8TYPES；DB `clips.content_type` CHECK camelCase 值域）。
///
/// 逐变体显式 rename：`fileURL` 的 DB 值为「URL 全大写 + f 小写」，任何 rename_all 推导
/// 均无法同时满足 8 个字面值（如自动 camelCase 会产出 `fileUrl` ≠ CHECK），故全显式。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ContentType {
    #[serde(rename = "plainText")]
    PlainText,
    #[serde(rename = "richText")]
    RichText,
    #[serde(rename = "html")]
    Html,
    #[serde(rename = "image")]
    Image,
    #[serde(rename = "url")]
    Url,
    #[serde(rename = "fileURL")]
    FileUrl,
    #[serde(rename = "color")]
    Color,
    #[serde(rename = "unknown")]
    Unknown,
}

/// 8 类全集（枚举取值断言与 DB CHECK 值域逐一对账用）
pub const CONTENT_TYPES: [ContentType; 8] = [
    ContentType::PlainText,
    ContentType::RichText,
    ContentType::Html,
    ContentType::Image,
    ContentType::Url,
    ContentType::FileUrl,
    ContentType::Color,
    ContentType::Unknown,
];

impl ContentType {
    /// 与 DB CHECK 字面值对账（TASK-108 枚举取值全集断言）
    pub fn as_db_value(&self) -> &'static str {
        match self {
            ContentType::PlainText => "plainText",
            ContentType::RichText => "richText",
            ContentType::Html => "html",
            ContentType::Image => "image",
            ContentType::Url => "url",
            ContentType::FileUrl => "fileURL",
            ContentType::Color => "color",
            ContentType::Unknown => "unknown",
        }
    }

    /// DB CHECK 字面值反解（回填读路径与脏值承载 · 与 as_db_value 互逆；
    /// 未匹配 → None，调用方 fail-soft 跳过该行）。
    pub fn parse_db_value(s: &str) -> Option<Self> {
        match s {
            "plainText" => Some(Self::PlainText),
            "richText" => Some(Self::RichText),
            "html" => Some(Self::Html),
            "image" => Some(Self::Image),
            "url" => Some(Self::Url),
            "fileURL" => Some(Self::FileUrl),
            "color" => Some(Self::Color),
            "unknown" => Some(Self::Unknown),
            _ => None,
        }
    }
}

/// 新建实体 id（UUID v4）：app/宿主层构造投影实体的唯一入口（避免 app 直引
/// uuid crate——uuid 为 core 的受控依赖面）。
pub fn new_uuid() -> Uuid {
    Uuid::new_v4()
}

/// 来源（DEC-15 定形；v1.3a 扩展 Note 变体）。
///
/// JSON 形状：`{type:"clip", clipId, sourceApp, capturedAt}`；`{type:"note", refId, sourceApp, capturedAt}`。
/// DB 列承载：`source_kind/source_ref_id/source_app/source_captured_at`（design §3.2 todos）。
/// notes 侧沿用仅 clip 来源（design §3.1 不变量 4）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Source {
    Clip {
        #[serde(rename = "clipId")]
        clip_id: Uuid,
        #[serde(rename = "sourceApp")]
        source_app: Option<String>,
        #[serde(rename = "capturedAt")]
        captured_at: String,
    },
    Note {
        #[serde(rename = "refId")]
        ref_id: Uuid,
        #[serde(rename = "sourceApp")]
        source_app: Option<String>,
        #[serde(rename = "capturedAt")]
        captured_at: String,
    },
}

/// 指纹（REQ-STORE-FINGERPRINT；DB `notes` 对账基准）。
///
/// 偏差登记（DEC-63）：design §3.4 草案 `content_hash: Sha256`——sha256 计算归 files 层
/// （S2），core 仅承载 hex 结果串，避免 core 依赖 sha2。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fingerprint {
    /// sha256 hex（小写）
    pub content_hash: String,
    pub byte_count: u64,
}

/// DB `notes` 指针表投影（DEC-41：正文永不入 DB，DB 只存 path 指针与元数据）。
///
/// 偏差登记（DEC-63）：时间字段以 ISO8601 字符串承载（design 草案 DateTime 抽象免引入 chrono，
/// ISO 字典序=时间序）；`tags` 为 7 色标签名数组（design §3.2 `notes.tags`「JSON 数组」口径，
/// 草案中 `Vec<TagColor>` 与 DB 名称真值不符）；`fingerprint` 对应列在 DDL 中暂缺（T7-R5），
/// S2 TASK-201/207 承接。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NotePointer {
    pub uuid: Uuid,
    /// 相对 `files/` 的相对路径（`/` 分隔，跨平台稳定）
    pub path: PathBuf,
    /// ISO8601（与对账 mtime 基线分离，沿 EM savedAt 语义）
    pub created_at: String,
    pub modified_at: String,
    /// 指纹对账 1s 阈值基准
    pub saved_at: String,
    /// 7 色标签名（labels 表显示名=唯一真相）
    pub tags: Vec<String>,
    pub fingerprint: Option<Fingerprint>,
    /// DEC-15：来源 clip（可空；notes 侧仅 clip 来源）
    pub source: Option<Source>,
}

/// DB `clips` 表投影（8 类内容；design §3.2）
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Clip {
    pub id: Uuid,
    pub content_type: ContentType,
    /// 文本类内容；二进制类为 None（正文在 blobs）
    pub text_content: Option<String>,
    /// sha256 hex（10s 去重键）
    pub content_hash: String,
    pub source_app: Option<String>,
    pub size: i64,
    pub is_pinned: bool,
    /// 相对 data/blobs/；None = 纯文本条目
    pub blob_rel_path: Option<String>,
    /// 仅 image 类
    pub thumb_rel_path: Option<String>,
    /// 用户就地编辑文本时间（LWW 字段）
    pub edited_at: Option<String>,
    pub created_at: String,
}

/// DB `projects` 表投影。
///
/// 偏差登记（DEC-63）：REQ-PROJECT-FOLDER 的四扩展列（status/lead/startDate/targetDate）在
/// design §3.2 DDL 中缺列（T7-R3，重要级在案）且值域未定形——core v1 不虚构值域，
/// S2 TASK-207 随 DDL 补列一并扩展。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Project {
    pub id: Uuid,
    pub name: String,
    pub icon: Option<String>,
    pub description: Option<String>,
    pub sort_order: i64,
    pub created_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    const CHECK_CONTENT_TYPES: [&str; 8] = [
        "plainText",
        "richText",
        "html",
        "image",
        "url",
        "fileURL",
        "color",
        "unknown",
    ];

    #[test]
    fn content_type_covers_db_check_value_set() {
        // TASK-108：枚举取值全集断言（对应 DB CHECK 约束值域，design §3.2 clips）
        for (i, expected) in CHECK_CONTENT_TYPES.iter().enumerate() {
            assert_eq!(CONTENT_TYPES[i].as_db_value(), *expected);
            assert_eq!(
                serde_json::to_string(&CONTENT_TYPES[i]).unwrap(),
                format!("\"{expected}\"")
            );
        }
    }

    #[test]
    fn content_type_renames_are_explicit_not_autocamel() {
        // fileURL 必须逐变体显式 rename（自动变换存在产出 fileUrl ≠ CHECK 的风险）
        assert_eq!(
            serde_json::to_string(&ContentType::FileUrl).unwrap(),
            "\"fileURL\""
        );
        assert_eq!(
            serde_json::to_string(&ContentType::PlainText).unwrap(),
            "\"plainText\""
        );
        assert_eq!(
            serde_json::to_string(&ContentType::RichText).unwrap(),
            "\"richText\""
        );
    }

    #[test]
    fn content_type_roundtrip_all_variants() {
        for ct in CONTENT_TYPES {
            let json = serde_json::to_string(&ct).unwrap();
            let back: ContentType = serde_json::from_str(&json).expect("roundtrip");
            assert_eq!(back, ct);
        }
    }

    #[test]
    fn content_type_rejects_unknown_string() {
        assert!(serde_json::from_str::<ContentType>("\"fileUrl\"").is_err());
        assert!(serde_json::from_str::<ContentType>("\"bogus\"").is_err());
    }

    #[test]
    fn content_type_db_value_parse_roundtrip_all_variants() {
        // 回填读路径契约：as_db_value ↔ parse_db_value 互逆全覆盖 + 脏值拒识。
        for ct in CONTENT_TYPES {
            let db = ct.as_db_value();
            assert_eq!(ContentType::parse_db_value(db), Some(ct), "{db}");
        }
        assert_eq!(ContentType::parse_db_value("fileUrl"), None);
        assert_eq!(ContentType::parse_db_value(""), None);
    }

    #[test]
    fn new_uuid_is_v4_non_nil() {
        // app 宿主层实体 id 唯一入口：非 nil 且两次生成互异。
        let a = new_uuid();
        let b = new_uuid();
        assert_ne!(a, b);
        assert_ne!(a, Uuid::nil());
    }

    #[test]
    fn content_kind_matches_pinboard_entries_check() {
        // DB `pinboard_entries.kind` CHECK ('clip','note','task')（design §3.2）
        assert_eq!(
            serde_json::to_string(&ContentKind::Clip).unwrap(),
            "\"clip\""
        );
        assert_eq!(
            serde_json::to_string(&ContentKind::Note).unwrap(),
            "\"note\""
        );
        assert_eq!(
            serde_json::to_string(&ContentKind::Task).unwrap(),
            "\"task\""
        );
    }

    #[test]
    fn source_clip_matches_dec15_shape_and_roundtrips() {
        // DEC-15 形状逐键断言：{type:"clip", clipId, sourceApp, capturedAt}
        let json = r#"{"type":"clip","clipId":"0f9d1d27-7f0d-4f4a-9fd5-7c2a3ec2f001","sourceApp":"Safari","capturedAt":"2026-10-17T08:00:00Z"}"#;
        let src: Source = serde_json::from_str(json).expect("parse DEC-15 shape");
        let Source::Clip {
            clip_id,
            source_app,
            captured_at,
        } = &src
        else {
            panic!("expected Clip variant, got {src:?}");
        };
        assert_eq!(clip_id.to_string(), "0f9d1d27-7f0d-4f4a-9fd5-7c2a3ec2f001");
        assert_eq!(source_app.as_deref(), Some("Safari"));
        assert_eq!(captured_at, "2026-10-17T08:00:00Z");
        let out = serde_json::to_string(&src).unwrap();
        let back: Source = serde_json::from_str(&out).expect("roundtrip");
        assert_eq!(back, src);
    }

    #[test]
    fn source_note_v13a_roundtrip() {
        // v1.3a：todos source 扩展 type:"note"（【标记为任务】Note→Task 派生）
        let src = Source::Note {
            ref_id: Uuid::parse_str("2b1e4b7e-9a1c-4c6e-8a7b-5e6f7a8b9c0d").unwrap(),
            source_app: Some("Parchora".into()),
            captured_at: "2026-10-17T09:30:00Z".into(),
        };
        let json = serde_json::to_string(&src).unwrap();
        assert!(json.contains("\"type\":\"note\""));
        assert!(json.contains("\"refId\""));
        let back: Source = serde_json::from_str(&json).expect("roundtrip");
        assert_eq!(back, src);
    }

    #[test]
    fn clip_roundtrip_all_fields() {
        let clip = Clip {
            id: Uuid::nil(),
            content_type: ContentType::FileUrl,
            text_content: Some("IMG_0001.png".into()),
            content_hash: "ab".repeat(32),
            source_app: Some("Finder".into()),
            size: 1024,
            is_pinned: true,
            blob_rel_path: Some("blobs/<id>/raw".into()),
            thumb_rel_path: None,
            edited_at: Some("2026-10-17T08:00:00Z".into()),
            created_at: "2026-10-17T08:00:00Z".into(),
        };
        let back: Clip = serde_json::from_str(&serde_json::to_string(&clip).unwrap()).unwrap();
        assert_eq!(back, clip);
    }

    #[test]
    fn note_pointer_roundtrip() {
        let np = NotePointer {
            uuid: Uuid::nil(),
            path: PathBuf::from("速记/Untitled.md"),
            created_at: "2026-10-17T08:00:00Z".into(),
            modified_at: "2026-10-17T08:00:00Z".into(),
            saved_at: "2026-10-17T08:00:00Z".into(),
            tags: vec!["工作".into()],
            fingerprint: Some(Fingerprint {
                content_hash: "cd".repeat(32),
                byte_count: 64,
            }),
            source: Some(Source::Clip {
                clip_id: Uuid::nil(),
                source_app: None,
                captured_at: "2026-10-17T07:00:00Z".into(),
            }),
        };
        let back: NotePointer = serde_json::from_str(&serde_json::to_string(&np).unwrap()).unwrap();
        assert_eq!(back, np);
    }
}
