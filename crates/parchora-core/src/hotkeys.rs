//! 快捷键冲突检测（TASK-105 / M12）。
//!
//! 锚点：REQ-HOTKEY-CONFLICT / SCN-HOTKEY-CONFLICT-DETECT / SCN-HOTKEY-CONFLICT-NONBLOCKING /
//! SCN-HOTKEY-CONFLICT-SCOPE-GROUPED（DEC-32 作用域分组）；REQ-HOTKEY-DUAL-GLOBAL 场景
//! （全局键与任一本地键一致也提示）；SRC-BASE-EDGEMARK §2.4。
//!
//! 非阻断语义：本函数只返回提示清单（不阻断保存属 UI 层落点——保存动作由调用方继续执行）。

/// 修饰键位图（组合相等 = 位集合相等 + 键相等）
pub const MOD_CMD: u8 = 1 << 0;
pub const MOD_SHIFT: u8 = 1 << 1;
pub const MOD_OPT: u8 = 1 << 2;
pub const MOD_CTRL: u8 = 1 << 3;

/// 键值（组合键主键；`char` 按字面精确比较——大写/小写视为不同键，归一化由调用方约定）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
    Char(char),
    Space,
    Arrow(ArrowKey),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArrowKey {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KeyCombo {
    pub modifiers: u8,
    pub key: Key,
}

impl KeyCombo {
    pub const fn new(modifiers: u8, key: Key) -> Self {
        KeyCombo { modifiers, key }
    }
}

/// 可配置键位作用域（DEC-32 作用域分组；REQ-HOTKEY-CONFLICT 例：待办 ⌘⌥1~5 vs 集锦 ⌘1~9）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HotkeyScope {
    /// 全局双热键（⌃⇧Space / ⌘⇧V，可改）——与任一键位一致即提示（REQ-HOTKEY-DUAL-GLOBAL 场景）
    Global,
    /// 面板内跨视图通用键位（⌘N/⇧⌘N/⌘F/⌘P/⇧⌘C/⌘←/→ 等，按 DEC-58 分化后仍为面板键）
    Panel,
    /// 编辑器作用域（查找替换等编辑器专属可配置键）
    Editor,
    /// 集锦视图作用域（收集板 ⌘1~9 跳转等，DEC-57）
    CollectionView,
    /// 待办视图作用域（状态键 ⌘⌥1~5 等，DEC-32①）
    TodoView,
}

/// 待检测的一条可配置键位
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Binding {
    /// 键位标识（设置键的稳定 id，互报时回指）
    pub id: String,
    pub combo: KeyCombo,
    pub scope: HotkeyScope,
}

/// 冲突轮次：互比（可配置键位相互）/ 固定保留项对照
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictRound {
    Mutual,
    FixedReserved,
}

/// 一条冲突提示（纯提示；不阻断保存）
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Conflict {
    pub first_id: String,
    pub second_id: String,
    pub round: ConflictRound,
}

/// 固定保留项（REQ-HOTKEY-CONFLICT：⌘Z/⇧⌘Z/⌘B/⌘I/⌘E/⌘K/⇧⌘X，跨视图全作用域）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FixedReserved {
    pub id: &'static str,
    pub combo: KeyCombo,
}

const fn combo(modifiers: u8, key: char) -> KeyCombo {
    KeyCombo::new(modifiers, Key::Char(key))
}

pub const FIXED_RESERVED: [FixedReserved; 7] = [
    FixedReserved {
        id: "fixed.cmd_z",
        combo: combo(MOD_CMD, 'z'),
    },
    FixedReserved {
        id: "fixed.shift_cmd_z",
        combo: combo(MOD_CMD | MOD_SHIFT, 'z'),
    },
    FixedReserved {
        id: "fixed.cmd_b",
        combo: combo(MOD_CMD, 'b'),
    },
    FixedReserved {
        id: "fixed.cmd_i",
        combo: combo(MOD_CMD, 'i'),
    },
    FixedReserved {
        id: "fixed.cmd_e",
        combo: combo(MOD_CMD, 'e'),
    },
    FixedReserved {
        id: "fixed.cmd_k",
        combo: combo(MOD_CMD, 'k'),
    },
    FixedReserved {
        id: "fixed.shift_cmd_x",
        combo: combo(MOD_CMD | MOD_SHIFT, 'x'),
    },
];

/// 作用域相交判定（SCN-HOTKEY-CONFLICT-SCOPE-GROUPED / DEC-32）：
/// - 三个视图专属作用域两两不相交；同作用域相交；
/// - Global / Panel（跨上下文作用域）与任意作用域相交——全局键与任一本地键
///   一致即提示（REQ-HOTKEY-DUAL-GLOBAL 场景）；固定保留项本身为全作用域（第二轮常量处理）。
pub fn scopes_conflict(a: HotkeyScope, b: HotkeyScope) -> bool {
    use HotkeyScope::*;
    // 三个视图专属作用域两两不相交；其余（Global/Panel × 任意、同作用域之间）全部相交
    !matches!(
        (a, b),
        (Editor, CollectionView)
            | (CollectionView, Editor)
            | (Editor, TodoView)
            | (TodoView, Editor)
            | (CollectionView, TodoView)
            | (TodoView, CollectionView)
    )
}

/// 两轮冲突检测（REQ-HOTKEY-CONFLICT）：
/// 第一轮可配置键位互比（同组合且作用域相交）；第二轮对照 7 条固定保留项（全作用域）。
/// 输出按扫描序排列（确定性）；结论仅为提示清单，调用方继续保存.
pub fn detect_conflicts(bindings: &[Binding]) -> Vec<Conflict> {
    let mut out = Vec::new();

    // 第一轮：互比
    for i in 0..bindings.len() {
        for j in (i + 1)..bindings.len() {
            if bindings[i].combo == bindings[j].combo
                && scopes_conflict(bindings[i].scope, bindings[j].scope)
            {
                out.push(Conflict {
                    first_id: bindings[i].id.clone(),
                    second_id: bindings[j].id.clone(),
                    round: ConflictRound::Mutual,
                });
            }
        }
    }

    // 第二轮：对照固定保留项（固定项=全作用域，逐条绑定逐固定项）
    for binding in bindings {
        for fixed in FIXED_RESERVED.iter() {
            if binding.combo == fixed.combo {
                out.push(Conflict {
                    first_id: binding.id.clone(),
                    second_id: fixed.id.to_string(),
                    round: ConflictRound::FixedReserved,
                });
            }
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn binding(id: &str, modifiers: u8, key: char, scope: HotkeyScope) -> Binding {
        Binding {
            id: id.into(),
            combo: combo(modifiers, key),
            scope,
        }
    }

    #[test]
    fn mutual_conflict_same_scope_same_combo() {
        // SCN-HOTKEY-CONFLICT-DETECT：两个可配置本地键设为相同组合 → 互比冲突
        let bindings = vec![
            binding("global2", MOD_CMD | MOD_SHIFT, 'v', HotkeyScope::Global),
            binding("panel.new", MOD_CMD | MOD_SHIFT, 'v', HotkeyScope::Panel),
        ];
        let conflicts = detect_conflicts(&bindings);
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].round, ConflictRound::Mutual);
        assert_eq!(conflicts[0].first_id, "global2");
        assert_eq!(conflicts[0].second_id, "panel.new");
    }

    #[test]
    fn global_conflicts_with_any_local_key() {
        // REQ-HOTKEY-DUAL-GLOBAL 场景：全局键与任一本地键一致 → 提示
        let bindings = vec![
            binding("global.v", MOD_CMD | MOD_SHIFT, 'v', HotkeyScope::Global),
            binding("todo.jump", MOD_CMD | MOD_SHIFT, 'v', HotkeyScope::TodoView),
        ];
        let conflicts = detect_conflicts(&bindings);
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].round, ConflictRound::Mutual);
    }

    #[test]
    fn disjoint_view_scopes_do_not_cross_report() {
        // SCN-HOTKEY-CONFLICT-SCOPE-GROUPED：待办状态键与收集板 tab 键同组合不互报
        let bindings = vec![
            binding("todo.status", MOD_CMD | MOD_OPT, '1', HotkeyScope::TodoView),
            binding(
                "collection.tab",
                MOD_CMD | MOD_OPT,
                '1',
                HotkeyScope::CollectionView,
            ),
            binding("editor.find", MOD_CMD | MOD_OPT, '1', HotkeyScope::Editor),
        ];
        assert!(detect_conflicts(&bindings).is_empty());
    }

    #[test]
    fn fixed_reserved_conflict_regardless_of_scope() {
        // 固定保留项=全作用域：任一作用域键位撞 ⌘B / ⇧⌘X → 固定项冲突提示
        let bindings = vec![
            binding("panel.bold", MOD_CMD, 'b', HotkeyScope::Panel),
            binding("todo.cut", MOD_CMD | MOD_SHIFT, 'x', HotkeyScope::TodoView),
        ];
        let conflicts = detect_conflicts(&bindings);
        assert_eq!(conflicts.len(), 2);
        assert!(conflicts
            .iter()
            .all(|c| c.round == ConflictRound::FixedReserved));
        let ids: Vec<&str> = conflicts.iter().map(|c| c.second_id.as_str()).collect();
        assert!(ids.contains(&"fixed.cmd_b"));
        assert!(ids.contains(&"fixed.shift_cmd_x"));
    }

    #[test]
    fn all_seven_fixed_items_are_detectable() {
        let bindings: Vec<Binding> = FIXED_RESERVED
            .iter()
            .enumerate()
            .map(|(i, f)| Binding {
                id: format!("cfg{i}"),
                combo: f.combo,
                scope: HotkeyScope::Panel,
            })
            .collect();
        let conflicts = detect_conflicts(&bindings);
        assert_eq!(conflicts.len(), 7);
    }

    #[test]
    fn clean_binding_set_has_no_conflict() {
        let bindings = vec![
            binding(
                "global.space",
                MOD_CTRL | MOD_SHIFT,
                ' ',
                HotkeyScope::Global,
            ),
            binding(
                "todo.status1",
                MOD_CMD | MOD_OPT,
                '1',
                HotkeyScope::TodoView,
            ),
            binding("collection.tab1", MOD_CMD, '1', HotkeyScope::CollectionView),
        ];
        assert!(detect_conflicts(&bindings).is_empty());
    }

    #[test]
    fn output_is_deterministic_in_scan_order() {
        let bindings = vec![
            binding("a", MOD_CMD, 'z', HotkeyScope::Panel), // 撞 fixed.cmd_z
            binding("b", MOD_CMD, 'z', HotkeyScope::Panel), // 与 a 互比 + 撞 fixed
            binding("c", MOD_CMD | MOD_SHIFT, 'z', HotkeyScope::CollectionView), // 撞 fixed.shift_cmd_z
        ];
        let conflicts = detect_conflicts(&bindings);
        // 互比 1 条（a-b）+ 固定项 3 条（a、b 撞 undo；c 撞 shift_undo）
        assert_eq!(conflicts.len(), 4);
        assert_eq!(
            conflicts[0],
            Conflict {
                first_id: "a".into(),
                second_id: "b".into(),
                round: ConflictRound::Mutual,
            }
        );
        assert!(conflicts[1..3].iter().all(|c| c.second_id == "fixed.cmd_z"));
        assert_eq!(conflicts[3].second_id, "fixed.shift_cmd_z");
    }
}
