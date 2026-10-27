# gpui vendored 补丁（patches/）—— gpui-fast 线（DEC-59）

> 唯一必需 vendored 点：**park_offscreen 原语**（停放屏外/还原，3 files +54/−6，预算 <100 行 ✓；
> 真机首跑后按 DEC-60 锁纪律修复再生成，原为 +46/−6）。
> 基线：gpui-fast 0.1.8 + gpui-fast-macos 0.1.8（[冲突#9] 裁决 B · DEC-59，2026-10-17）。
> 历史：本文前身针对 gpui 0.2.2 原型（canBecomeKey 等 5 项免补丁结论仍有效）；0.2.2 原型
> 补丁内容保留于 `协作记录/spikes/gpui-window.md` 与 git 历史，不再作为应用对象。
> 证据全文：`docs/产品交付/统一产品方案/协作记录/spikes/{gpui-window.md, editor-component.md}`。

## 背景（为什么只有这 1 个补丁）

AGENTS.md 技术栈表预定 gpui vendored 小补丁覆盖三点：**停放屏外、从不 orderOut、canJoinAllSpaces**。
对 gpui-fast 0.1.8 树（registry 源码逐点核验，能力矩阵见 editor-component.md §3）：

| EdgeMark 需求 | gpui-fast 0.1.8 现状 | 结论 |
|---|---|---|
| 面板不夺焦点（NSPanel 非激活） | `WindowKind::PopUp` → NSPanel + `NSWindowStyleMaskNonactivatingPanel`（NonactivatingPanel L97/L1025） | 免补丁 |
| canJoinAllSpaces / 全屏辅助 | PopUp 分支原生 `CanJoinAllSpaces | FullScreenAuxiliary`（L1246-1247） | 免补丁 |
| 弹层层级 | `NSPopUpWindowLevel`（L105/L1240） | 免补丁 |
| PopUp 反激活 bug（虚假 becomeKey） | 上游已有 workaround（L3183-3196） | 免补丁 |
| canBecomeKeyWindow | `canBecomeKeyWindow` 注入 =yes（L438-445） | 免补丁 |
| **停放屏外 / 还原** | 无窗口 hide / 停放位置 API（`resize` 仅改尺寸，`activate` 无屏外路径） | **需补丁 0001** |
| （加分）父锚定弹窗 | 新增 `PopupOptions` API（core platform/popup.rs） | Peek 浮层直推能力 |

## 落地形态（已应用）

- vendored 树：`app/vendor/gpui-fast` + `app/vendor/gpui-fast-macos`（registry 源码复制，
  git pristine 基线提交 `135a88c`）；**补丁已应用于实树**。
- patch 文件：`0001-add-park-offscreen.patch`（由 `git diff` 从实树生成，非手写——
  补丁与实树恒一致）；三文件挂点：
  1. `gpui-fast/src/platform.rs`：`PlatformWindow` trait 的 `fn resize`（registry L1220）后
     插入 `fn park_offscreen(&mut self, park: bool)` **默认空实现**（其余后端零连带）；
  2. `gpui-fast/src/window.rs`：`pub fn resize`（L2912-2915）后插入公共包装层；
  3. `gpui-fast-macos/src/window.rs`：`MacWindowState` 增 `parked_frame: Option<Objc2NSRect>`
     字段（`activated_least_once` L713 后）+ init `None`（L1153 处）+ `fn park_offscreen`
     实现（`fn merge_all_windows` 前插入，模式＝`activate()` 同款 async + closed 守卫 +
     `Objc2NSWindow` 类型化调用；**锁纪律 DEC-60：objc 调用一律不得持锁执行**——
     `setFrame*` 同步回调 delegate（didMove/didResize）且 delegate 共用 MacWindowState 锁，
     持锁 objc 与 if-let 临时 guard 横跨 objc 均为禁用模式：park=`frame()` 先读 →
     `setFrameOrigin(-32000,-31000)` → 锁内落库；unpark=`take()` 先出锁 →
     `setFrame_display(frame, true)` 全量还原——签名核验自 objc2-app-kit 0.3.2
     `generated/NSWindow.rs` L827-852）。
- 工作区接入：`app/spikes/editor-spike/Cargo.toml` `[patch.crates-io]` 双条目指向 vendor 树；
  `app/Cargo.toml` exclude 增 `vendor`。
- 编译验证（macOS 27 宿主，CLT-only）：
  `cargo check` 温态 `Finished dev profile in 11.03s`；冷态独立 target 目录 40.64s，
  0 error（gpui-fast 2 条 unused 警告为上游 `fast/layers/reuse.rs` 既有，非补丁引入）。

## 复现 / 重打补丁步骤

```sh
# 以 pristine 基线重现实树（验证 patch 可复现性）
git -C app/vendor checkout -- .
git -C vendor apply ../spikes/gpui-spike/patches/0001-add-park-offscreen.patch   # 路径以实际仓布局为准
# 等价：直接 git -C vendor apply ../spikes/gpui-spike/patches/0001-add-park-offscreen.patch
git -C vendor diff --stat   # 应为 3 files, +54/-6（DEC-60 修复态；首版为 +46/-6）

# 升级 gpui-fast 版本时：换 registry 新版入 vendor → git diff 重新生成 patch → 人工复核挂点漂移
```

## runtime_shaders（构建依赖面，DEC-59/D-03）

`gpui-fast-apple` build.rs 默认调 `xcrun metal` 离线编译着色器（CLT 无 metal toolchain 时
fail——本机曾实证 exit 失败）。gpui-fast-platform 官方 feature `runtime_shaders`
（= `gpui_macos/runtime_shaders`）改走 MSL 拼装 + **运行期编译**，构建仅需 clang 链接。
editor-spike 已启用：`gpui-fast-platform = { version = "0.1.8", features = ["runtime_shaders"] }`。
产品 crate 接入时保持同一开关（登记于 DEC-59）；运行时 Metal 渲染路径不变。

## 已知取舍

1. **trait 默认空实现**：`park_offscreen` 给默认体，linux/windows/web 后端零连带改动。
   产品 macOS-only（DEC-36/38）；未来多平台再改显式实现（登记后再动）。
2. **停放点 (-32000, -31000)**：经典屏外惯例坐标；unpark 用 `setFrame:display:` 全量还原
   （位置+尺寸），不做部分还原。多屏/旋转屏边界行为列入 10.15.7 实机验证清单。
3. **线程模型**：与 fork `activate()` 同款——主线程 `foreground_executor.spawn` + closed
   守卫；spawn 前取短命锁三字段后即释放。**闭包内 objc 调用不持锁（DEC-60 真机实证修正）**：
   原实现持锁 setFrame 曾致两处重入死锁（park 持锁 setFrameOrigin、unpark
   if-let 临时 guard 横跨 setFrame_display），真机 selftest 复现并已修复——见
   gpui-window.md §4/§5 与 DEC-60。
4. **canBecomeKey 不打补丁**；若 Peek 交互需要「点击即持键」，追加一行
   `setBecomesKeyOnlyIfNeeded: NO`（PopUp alloc 分支）。默认 YES（NSPanel）与「预览面板
   不夺键」规格语义一致，暂不加，实机联调再决。
5. **锁版本与升级**：gpui-fast-* 0.1.x 锁版；升级 = 新 registry 源入 vendor → patch 重生成 →
   挂点人工复核（`git diff` 应仍命中三文件五挂点）→ 独立 PR。

## 真机验证证据（本机 macOS 27.0.1 宿主，CLT · DEC-60）

- `cargo run -- --selftest`（editor-spike）：全链 PASS——启动 → 1.06s PARK → 3.16s UNPARK →
  4.74s PASS、退出码 0，进程正常退出；两轮复跑一致；`--no-park` bisect（三段 timer
  链不碰窗口）证明调度栈健康，失败面锁定 park_offscreen 死锁（详见 DEC-60）。
- 死锁定位手段存档：`sample editor-spike 3 -file /tmp/spike-sample.txt`（主线程栈直达
  lock_slow→psynch_cvwait）；进程 CPU 0.1%（`ps aux`）排除渲染自旋。
- 10.15.7 实机验证清单（下节）转非阻塞复核——U-01 已经用户裁决以本机 27.0.1 替代跑收口（DEC-62，TQ14_SUBSTITUTE=1）；清单保留为真机复核载体，两腿判定规则不变。

## 10.15.7 实机验证清单（T-Q14；对齐 gpui-spike 与补丁行为，脚本 = scripts/catalina-check.sh）

- [ ] `cargo check`（editor-spike 载体）在 10.15.7 实机通过（runtime_shaders 下 CLT 即可）
- [ ] PopUp 面板（NSPanel）在前台其他 App 之上弹出且**主 App 不被激活**
      （菜单栏标题不变、前置 App 仍持键）
- [ ] 面板出现在所有 Space（CanJoinAllSpaces）且全屏 App 上可弹（FullScreenAuxiliary）
- [ ] `park_offscreen(true)` 后窗口从 Mission Control/屏幕均不可见；
      `park_offscreen(false)` 逐像素还原原位
- [ ] 停放期间再激活（makeKeyAndOrderFront 路径）无 orderOut 相关闪烁
- [ ] 密码管理器 1Password/键盘输入不受面板影响（规格：面板不夺焦点为红线场景）