//! 收集板 9 色确定性分配（TASK-107 / M4）。
//!
//! 锚点：REQ-BOARD-COLOR / SCN-BOARD-COLOR-DETERMINISTIC：「按 (createdAt, id) 排序索引
//! 从 9 色中确定性分配，不新增存储字段；数据复制与导出导入后颜色分配保持一致」。
//!
//! 取舍（DEC-63）：core 只承载「排序 → 索引（0..=8）」规则；9 色视觉值域归品牌/UI 层
//! （品牌设计方案 §3.2 九色 hex 表 + REQ-TAG-7COLORS 同源常量，规范 7 审批线），
//! design §3.4 草案 `pinboard_color(&[Pinboard]) -> TagColor` 为单值签名、与逐板确定性
//! 分配相矛盾，实现细化为平行索引数组 `pinboard_colors`。

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 收集板色板容量（REQ-BOARD-COLOR：9 色）
pub const PINBOARD_COLOR_COUNT: usize = 9;

/// DB `pinboards` 表投影（design §3.2）
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pinboard {
    pub id: Uuid,
    pub name: String,
    /// tab 展示顺序（REQ-BOARD-TAB-ORDER 重排持久化；不参与取色）
    pub sort_order: i64,
    /// ISO8601（字典序=时间序；取色排序键第一分量）
    pub created_at: String,
}

/// 按 `(created_at, id)` 排序索引取色：返回与输入同序的色索引向量（0..=8；
/// 板数超过 9 时对 9 取模循环——超 9 行为规格未载明，假设登记见 DEC-63）。
///
/// 结果只依赖集合本身（created_at/id），与输入顺序无关 → 导出导入后颜色一致；
/// `sort_order`（tab 展示顺序）变更不改变取色。
pub fn pinboard_colors(pinboards: &[Pinboard]) -> Vec<u8> {
    if pinboards.is_empty() {
        return Vec::new();
    }
    let mut order: Vec<usize> = (0..pinboards.len()).collect();
    order.sort_by(|&i, &j| {
        pinboards[i]
            .created_at
            .cmp(&pinboards[j].created_at)
            .then_with(|| pinboards[i].id.cmp(&pinboards[j].id))
    });
    let mut colors = vec![0u8; pinboards.len()];
    for (position, &input_index) in order.iter().enumerate() {
        colors[input_index] = (position % PINBOARD_COLOR_COUNT) as u8;
    }
    colors
}

#[cfg(test)]
mod tests {
    use super::*;

    fn board(id: u128, created_at: &str) -> Pinboard {
        Pinboard {
            id: Uuid::from_u128(id),
            name: format!("板{id}"),
            sort_order: 0,
            created_at: created_at.into(),
        }
    }

    #[test]
    fn deterministic_allocation_by_created_at_then_id() {
        // SCN-BOARD-COLOR-DETERMINISTIC：含 createdAt 相同、仅 id 不同的情形
        let boards = vec![
            board(3, "2026-10-17T08:00:00Z"),
            board(1, "2026-10-17T07:00:00Z"),
            board(2, "2026-10-17T08:00:00Z"),
        ];
        let colors = pinboard_colors(&boards);
        // 排序后：(07:00, id=1) → 0；(08:00, id=2) → 1；(08:00, id=3) → 2
        assert_eq!(colors, vec![2, 0, 1]);
    }

    #[test]
    fn allocation_is_repeatable() {
        let boards = vec![
            board(9, "2026-10-17T08:00:00Z"),
            board(5, "2026-10-17T08:00:00Z"),
        ];
        let first = pinboard_colors(&boards);
        let second = pinboard_colors(&boards);
        assert_eq!(first, second);
        // 同 createdAt → id 兜底序：id 5 < id 9
        assert_eq!(first, vec![1, 0]);
    }

    #[test]
    fn permuted_input_keeps_same_color_per_board() {
        // 多板一致性：输入顺序不影响每板的色（导出导入后颜色一致）
        let boards = vec![
            board(1, "2026-10-17T07:00:00Z"),
            board(2, "2026-10-17T08:00:00Z"),
            board(3, "2026-10-17T09:00:00Z"),
        ];
        let baseline = pinboard_colors(&boards);
        assert_eq!(baseline, vec![0, 1, 2]);

        let reversed = vec![boards[2].clone(), boards[1].clone(), boards[0].clone()];
        let shuffled = pinboard_colors(&reversed);
        assert_eq!(shuffled, vec![2, 1, 0]);
    }

    #[test]
    fn wraps_after_nine_boards() {
        let boards: Vec<Pinboard> = (1..=12_u32)
            .map(|i| board(i as u128, &format!("2026-10-17T{i:02}:00:00Z")))
            .collect();
        let colors = pinboard_colors(&boards);
        assert_eq!(colors.len(), 12);
        // 前 9 板 0..=8，后 3 板循环回 0..=2
        //（注意：created_at 必须零填充定宽 ISO8601，否则字典序≠时间序——本用例曾以
        //  0{i} 未填充产生 T010 < T01 的反例，实为测试数据错误，约定登记见 DEC-63）
        assert_eq!(colors[..9], [0, 1, 2, 3, 4, 5, 6, 7, 8]);
        assert_eq!(colors[9..], [0, 1, 2]);
    }

    #[test]
    fn empty_input_is_empty() {
        assert!(pinboard_colors(&[]).is_empty());
    }

    #[test]
    fn tab_reorder_does_not_change_color() {
        // 取色只看 (createdAt, id)；displayOrder 变更不影响分配
        let mut a = board(1, "2026-10-17T07:00:00Z");
        let mut b = board(2, "2026-10-17T08:00:00Z");
        a.sort_order = 5;
        b.sort_order = 9;
        let colors = pinboard_colors(&[a, b]);
        assert_eq!(colors, vec![0, 1]);
    }
}
