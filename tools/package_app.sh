#!/bin/bash
# Parchora .app bundle 打包脚本（第四十一轮 · DEC-92）。
#
# 用法：bash tools/package_app.sh [debug|release] [品牌源路径]
#   - 第 1 参：构建档位，默认 release（DEC-197：分发瘦身——debug 档产物 ~127MB
#     不用于分发；调试打包可用显式 `bash tools/package_app.sh debug`）；产物取对应
#     target 目录的二进制。release 档装配后 strip 符号表（debug 档保留供调试）。
#   - 第 2 参：品牌源（.png 或 .svg），默认 ../parchora_logo_color.png（DEC-160
#     用户交付彩色稿；DEC-92 初版默认 ../docs/parchora_logo.svg 退役为留档可选源）；
#     PNG 源经 pad_logo.swift 透明垫方成正方形基底（DEC-112）。
# 产物：app/dist/Parchora.app
#
# 链路：品牌源（PNG →（pad_logo.swift 垫方 · 默认）｜ SVG →（render_logo.swift
#       可选 size=1024））→ 基底 PNG →（sips 降采样）→ AppIcon.iconset 全尺寸 →（iconutil）→
#       AppIcon.icns → 装配 bundle → plutil -lint → ad-hoc codesign → verify。
# 渲染/打包工具均非构建依赖（规范 9 · 一次性脚本留档复现）。
# 单实例锁：二进制命名 parchora-app 与裸二进制共享 temp flock，互斥语义不变。

set -euo pipefail
cd "$(dirname "$0")/.."

CONFIG="${1:-release}"
BRAND_SRC="${2:-../parchora_logo_color.png}"

case "$CONFIG" in
  debug)   TARGET_DIR="target/debug" ;;
  release) TARGET_DIR="target/release" ;;
  *) echo "参数只支持 debug|release，收到: $CONFIG" >&2; exit 2 ;;
esac

# workspace.package.version（app/Cargo.toml）→ Info.plist 版本号。
VERSION=$(awk '$1=="version" {gsub(/"/,"",$3); print $3; exit}' Cargo.toml)
if [ -z "$VERSION" ]; then
  echo "无法从 Cargo.toml 解析版本号" >&2
  exit 1
fi

echo "== [1/6] cargo build -p parchora-app ($CONFIG) =="
if [ "$CONFIG" = "release" ]; then
  cargo build -p parchora-app --release
else
  cargo build -p parchora-app
fi

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

echo "== [2/6] 生成 1024 基底 PNG（品牌源：$BRAND_SRC）=="
case "${BRAND_SRC##*.}" in
  png|PNG)
    # PNG 源：透明垫方成正方形居中（CoreGraphics/NSImage 自绘保 alpha ·
    # sips padColor 仅不透明不可用 · DEC-112）。
    swift tools/pad_logo.swift "$BRAND_SRC" "$WORK/icon_1024.png" 1024
    ;;
  svg|SVG)
    swift tools/render_logo.swift "$BRAND_SRC" "$WORK/icon_1024.png" 1024
    ;;
  *)
    echo "品牌源须为 .svg/.png，收到: $BRAND_SRC" >&2
    exit 2
    ;;
esac

echo "== [3/6] 生成 iconset 全尺寸 =="
ICONSET="$WORK/AppIcon.iconset"
mkdir -p "$ICONSET"
for dim in 16 32 128 256 512; do
  sips -z "$dim" "$dim" "$WORK/icon_1024.png" --out "$ICONSET/icon_${dim}x${dim}.png" >/dev/null
  dbl=$((dim * 2))
  if [ "$dbl" -le 1024 ]; then
    sips -z "$dbl" "$dbl" "$WORK/icon_1024.png" --out "$ICONSET/icon_${dim}x${dim}@2x.png" >/dev/null
  fi
done
iconutil -c icns "$ICONSET" -o "$WORK/AppIcon.icns"

echo "== [4/6] 装配 dist/Parchora.app =="
APP="dist/Parchora.app"
rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp "$TARGET_DIR/parchora-app" "$APP/Contents/MacOS/parchora-app"
if [ "$CONFIG" = "release" ]; then
  # strip 符号表（DEC-197：release __LINKEDIT 8.1→<1MB）；debug 档保留符号供 lldb 调试
  strip "$APP/Contents/MacOS/parchora-app"
fi
cp "$WORK/AppIcon.icns" "$APP/Contents/Resources/AppIcon.icns"
printf 'APPL????' > "$APP/Contents/PkgInfo"
cat > "$APP/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key><string>parchora-app</string>
    <key>CFBundleIdentifier</key><string>com.parchora.app</string>
    <key>CFBundleName</key><string>Parchora</string>
    <key>CFBundlePackageType</key><string>APPL</string>
    <key>CFBundleInfoDictionaryVersion</key><string>6.0</string>
    <key>CFBundleIconFile</key><string>AppIcon</string>
    <key>CFBundleShortVersionString</key><string>$VERSION</string>
    <key>CFBundleVersion</key><string>$VERSION</string>
    <key>LSUIElement</key><true/>
    <key>NSHighResolutionCapable</key><true/>
    <key>LSMinimumSystemVersion</key><string>10.15.7</string>
</dict>
</plist>
PLIST

echo "== [5/6] plutil -lint =="
plutil -lint "$APP/Contents/Info.plist"

echo "== [6/6] ad-hoc codesign + verify =="
codesign --force --sign - "$APP"
codesign --verify --strict "$APP"
codesign --display --entitlements - "$APP" >/dev/null 2>&1 || true

echo "OK dist/Parchora.app (version=$VERSION, config=$CONFIG)"