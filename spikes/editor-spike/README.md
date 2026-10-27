# editor-spike —— TASK-002 编辑器组件调查

本 spike 仅作依赖图快照（`Cargo.lock`）与调查工作区，**无产品代码**。

## 结论与全文

调查报告（能力面 · 依赖 lineage · [冲突#9] 决策呈现 · IME 实机清单）：

→ `docs/产品交付/统一产品方案/协作记录/spikes/editor-component.md`

## 状态

- `cargo add gpui-component --features gpui-fast` + `gpui-fast-platform` 已解析（0.7.3 / 0.1.8 线）；
- 未编译（无需——结论均来自 registry 源码逐项核实，锚点见报告）；
- `src/main.rs` 为占位（cargo add 需要 manifest 可解析）。