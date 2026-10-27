//! 写板代次登记（DEC-163 · REQ-CAPTURE-SELF-SKIP 自写板跳过 · 最小承载）。
//!
//! 本应用把文本写入剪贴板的每个写点（editor 围栏复制 chip / Peek 复制钮 ×2 /
//! 集锦右键复制 ×2 / zorite 引擎 Copy·Cut 经 host writer 桥）先经 `mark` 登记
//! 时刻与原文；app 层剪贴板捕获守护读回文本后以 `fresh_match`（窗口内且全文相
//! 等）判「自写板」→ 跳过不入拾记（DEC-46 语义延续）。
//!
//! 静态槽为进程级单例：mark 恒覆盖（单写者=UI 主线程），消费仅比对不修改——
//! 锁中毒按 miss 降级（宁多收一条不自写，不 panic）。
//!
//! 登记局限（DEC-163 台账）：gpui-component Textarea（源码态整篇 Copy·Cut）与
//! zorite-markdown 代码块复制钮为库内部直写 gpui 剪贴板、无 host writer 钩子，
//! 本机制不覆盖——该两条路径的自写捕获回归随 vendored 面扩展收口。

use std::sync::Mutex;
use std::time::{Duration, Instant};

/// 自写登记有效期：写板后超过此窗的登记不再视为自写源（错过一拍的捕获按
/// 「同内容 10s 判重」第二闸兜底 · DEC-163）。
const FRESH_MS: u64 = 1500;

/// 写板登记槽：(原文, 时刻)——单槽覆盖式（最近一次写板）。
static STAMP: Mutex<Option<(String, Instant)>> = Mutex::new(None);

/// 登记「本进程即将写该文本入剪贴板」（写点调用 · 文本须与实写一致）。
pub fn mark(text: &str) {
    if let Ok(mut slot) = STAMP.lock() {
        *slot = Some((text.to_string(), Instant::now()));
    }
}

/// 捕获侧判定：`text` 与登记原文一致且登记新鲜 → 自写板（跳过）。
pub fn fresh_match(text: &str) -> bool {
    let Ok(slot) = STAMP.lock() else {
        return false;
    };
    matches!(slot.as_ref(), Some((t, at)) if t == text && at.elapsed() < Duration::from_millis(FRESH_MS))
}
