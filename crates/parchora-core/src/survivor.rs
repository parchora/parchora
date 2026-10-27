//! C7 确定性收敛（TASK-103）。
//!
//! 锚点：design §2.10e（survivor 收敛为 files 指纹对账兜底的确定性保证，保留于 core 并维持单测）；
//! CB §4.3 / SRC-FEAT-CLIPBARA §2.11（SyncRecords.swift:209-214，UUID 字典序裁决）；
//! 决策口径：DEC-08（设计思想必须继承）/ DEC-37（推论不变）。

use uuid::Uuid;

/// C7：两端各自独立计算必得同一结果（无需通信）。
///
/// 方向取**字典序较大者**：CB 源码逆向文档仅载明「UUID 字典序裁决」未载明方向
/// （Clipbara-产品文档 §2.11、§3.4），方向不影响收敛性质（任一固定方向均满足交换律
/// 与确定性）；canonical hyphenated 串为小写定宽 hex，字典序与 nibble 序同构
/// ——假设登记见 DEC-63，真机/源码复核后如有出入仅需翻转比较符。
pub fn survivor_id(a: Uuid, b: Uuid) -> Uuid {
    // Uuid Ord = 16 字节字典序，与原 to_string 口径（canonical 小写定宽 hex）同构
    // ——省去每次比较两次 String 分配（行为等价 · 参数化单测锁定方向语义）。
    if a >= b {
        a
    } else {
        b
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 参数化对（覆盖首组/中间组/尾组与混合十六进制位序）
    const PAIRS: [(u128, u128); 5] = [
        (
            0x0000_0000_0000_0000_0000_0000_0000_0001,
            0x0000_0000_0000_0000_0000_0000_0000_0002,
        ),
        (
            0x0000_0000_0000_0000_0000_0000_0000_ffff,
            0x0000_0000_0000_0000_0000_0000_0001_0000,
        ),
        (
            0x1234_5678_9abc_def0_1234_5678_9abc_def0,
            0x1234_5678_9abc_def0_1234_5678_9abc_def1,
        ),
        (
            0xffff_ffff_ffff_ffff_ffff_ffff_ffff_fffe,
            0xffff_ffff_ffff_ffff_ffff_ffff_ffff_ffff,
        ),
        (
            0x8000_0000_0000_0000_0000_0000_0000_0000,
            0x7fff_ffff_ffff_ffff_ffff_ffff_ffff_ffff,
        ),
    ];

    #[test]
    fn parameterized_pairs_converge() {
        // TASK-103：参数化对收敛一致性——survivor(a,b) == survivor(b,a)，且两者必居其一
        for (x, y) in PAIRS {
            let a = Uuid::from_u128(x);
            let b = Uuid::from_u128(y);
            let ab = survivor_id(a, b);
            let ba = survivor_id(b, a);
            assert_eq!(ab, ba, "a={a} b={b}");
            assert!(ab == a || ab == b, "survivor 必为两端之一");
            assert_eq!(
                ab.to_string(),
                a.to_string().max(b.to_string()),
                "方向=字典序较大者"
            );
        }
    }

    #[test]
    fn equal_ids_converge_to_self() {
        let a = Uuid::from_u128(0xdead_beef_dead_beef_dead_beef_dead_beef);
        assert_eq!(survivor_id(a, a), a);
    }

    #[test]
    fn deterministic_across_repeated_calls() {
        let a = Uuid::from_u128(0x1);
        let b = Uuid::from_u128(0x2);
        let first = survivor_id(a, b);
        assert_eq!(survivor_id(a, b), first);
        assert_eq!(survivor_id(b, a), first);
        // 字典序较大者存活（"…0002" > "…0001"）
        assert_eq!(first, Uuid::from_u128(0x2));
    }
}
