#!/bin/sh
# catalina-check.sh —— T-Q14 运行时腿自检（macOS 10.15.7 实机执行）
# 载体（DEC-59，2026-10-17）：gpui-fast 0.1.8 线 vendored —— spike = app/spikes/editor-spike。
# 用法: ./scripts/catalina-check.sh [spike目录]   # 默认脚本所在仓的 app/spikes/editor-spike
# 替代跑: TQ14_SUBSTITUTE=1 ./scripts/catalina-check.sh   # 非 10.15 本机替代证据（DEC-62），真机复核义务不豁免
# 输出: 全部日志同时落 catalina-check.out（回填给会话用）；任一硬性项失败则退出码 1。
# 注：gpui-fast 线启用 runtime_shaders（gpui-fast-platform feature），构建走 MSL 运行期
#     编译——不再依赖离线 metal 工具链与完整 Xcode，CLT 即即可编译（DEC-59/D-03）。
set -u
DEVELOPER_DIR="${DEVELOPER_DIR:-$(xcode-select -p 2>/dev/null || true)}"

SPIKE_DIR="${1:-$(cd "$(dirname "$0")/../../.." && pwd)/spikes/editor-spike}"
OUT="$(pwd)/catalina-check.out"
: > "$OUT"

log() { printf '%s\n' "$*" | tee -a "$OUT"; }
check() { # check <硬性|提示> <名称> <命令...>
  label="$1"; name="$2"; shift 2
  if "$@" >>"$OUT" 2>&1; then
    log "[PASS] ($label) $name"; return 0
  fi
  log "[FAIL] ($label) $name —— 详见 catalina-check.out"; return 1
}

log "==== Parchora T-Q14 Catalina 自检 $(date) ===="
log "[env] SPIKE_DIR=$SPIKE_DIR"

# 1. 系统版本（硬性：10.15.x Catalina）
sw_vers >>"$OUT" 2>&1
PRODVER=$(sw_vers -productVersion 2>/dev/null)
case "$PRODVER" in
  10.15.*) log "[PASS] (硬性) macOS=$PRODVER" ;;
  *)
    if [ "${TQ14_SUBSTITUTE:-0}" != "1" ]; then
      log "[FAIL] (硬性) macOS=$PRODVER 非 10.15.x —— T-Q14 结论变更，停止后续"; exit 1
    fi
    log "[OVERRIDE] (替代跑) macOS=$PRODVER 非 10.15.x —— TQ14_SUBSTITUTE=1 本机替代证据（DEC-62）；不构成 10.15 兼容性结论" ;;
esac

# 2. 架构与工具链（提示：intel/arm64 都允许，结论分开登记）
uname -m >>"$OUT" 2>&1; log "[INFO] arch=$(uname -m) cargo=$(command -v cargo || echo MISSING)"

# 3. 完整 Xcode / Metal toolchain（提示级：runtime_shaders 下非必需，仅记录环境形态）
if [ -d /Applications/Xcode.app/Contents/Developer ]; then
  export DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer
  log "[PASS] (提示) Xcode.app 存在，DEVELOPER_DIR 已指向之"
else
  log "[INFO] (提示) 未找到完整 Xcode —— 正常（runtime_shaders 下 CLT 即可，DEC-59）"
fi
check 提示 "xcodebuild 可用（缺不影响编译）" xcodebuild -version
check 提示 "metal 工具链可用（缺不影响 runtime_shaders 编译）" xcrun -f metal
check 提示 "clang 链接器可用（cargo 必需）" cc --version

# 4. spike 编译（硬性：T-Q14 主判据；全新 target 首次可能数分钟）
if [ -d "$SPIKE_DIR" ]; then
  (
    cd "$SPIKE_DIR" || exit 1
    log "[RUN ] cargo check (DEVELOPER_DIR=$DEVELOPER_DIR)"
    if env DEVELOPER_DIR="$DEVELOPER_DIR" cargo check 2>&1 | tee -a "$OUT"; then
      log "[PASS] (硬性) editor-spike cargo check（10.15.7 SDK，gpui-fast vendored 链）"
    else
      log "[FAIL] (硬性) editor-spike cargo check —— 详见上方错误段"
      exit 1
    fi
  ) || FAIL_SPIKE=1
else
  log "[FAIL] (硬性) SPIKE_DIR 不存在: $SPIKE_DIR"
  FAIL_SPIKE=1
fi

# 5. 汇总
log "==== 汇总 ===="
log "macOS=$PRODVER arch=$(uname -m) spike_check=${FAIL_SPIKE:-0}（0=通过）"
log "后续：cargo run 探针 + 补丁行为清单（app/spikes/gpui-spike/patches/README §10.15.7 实机验证清单）"
[ "${FAIL_SPIKE:-0}" -eq 0 ] || exit 1
exit 0