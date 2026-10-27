// Parchora 应用图标 PNG 源垫方脚本（PNG → 正方形 PNG · 第六十三轮 DEC-112）。
//
// 用法：swift pad_logo.swift <input.png> <output.png> [size]
// 语义：非正方形 PNG 源按 max(宽,高) 透明垫方、内容等比居中，再输出 size×size
//       基底（iconset 需正方形）；alpha 通道全程保留（内容垫方处为透明）。
// 产物：app/dist/Parchora.app 装配用 iconset 基底（经 tools/package_app.sh）与
//       crates/parchora-app/assets/parchora-logo-512.png（NSApp 内嵌图标）。
// 渲染/打包工具均非构建依赖（规范 9 · 一次性脚本留档复现）；sips --padToHeightWidth
// 的 padColor 仅支持不透明色，故自绘透明垫方。

import AppKit

guard CommandLine.arguments.count == 3 || CommandLine.arguments.count == 4 else {
    FileHandle.standardError.write("用法: swift pad_logo.swift <in.png> <out.png> [size]\n".data(using: .utf8)!)
    exit(2)
}
let inPath = CommandLine.arguments[1]
let outPath = CommandLine.arguments[2]
// 可选第 3 参：输出边长（像素）。缺省 1024 为 iconset 基底口径（DEC-92）。
let px: Int
if CommandLine.arguments.count == 4 {
    guard let parsed = Int(CommandLine.arguments[3]), parsed > 0 else {
        FileHandle.standardError.write("size 须为正整数: \(CommandLine.arguments[3])\n".data(using: .utf8)!)
        exit(2)
    }
    px = parsed
} else {
    px = 1024
}

guard let image = NSImage(contentsOfFile: inPath) else {
    FileHandle.standardError.write("NSImage 无法读取 PNG: \(inPath)\n".data(using: .utf8)!)
    exit(1)
}
// 用像素尺寸（非 NSImage.size 点值）计算垫方，免疫源 DPI 标注差异。
guard let firstRep = image.representations.first else {
    FileHandle.standardError.write("NSImage 无位图表示: \(inPath)\n".data(using: .utf8)!)
    exit(1)
}
let srcW = max(1, firstRep.pixelsWide)
let srcH = max(1, firstRep.pixelsHigh)
let side = max(srcW, srcH)

guard let rep = NSBitmapImageRep(
    bitmapDataPlanes: nil,
    pixelsWide: px,
    pixelsHigh: px,
    bitsPerSample: 8,
    samplesPerPixel: 4,
    hasAlpha: true,
    isPlanar: false,
    colorSpaceName: .deviceRGB,
    bytesPerRow: 0,
    bitsPerPixel: 0
) else {
    FileHandle.standardError.write("NSBitmapImageRep 构造失败\n".data(using: .utf8)!)
    exit(1)
}

NSGraphicsContext.saveGraphicsState()
guard let ctx = NSGraphicsContext(bitmapImageRep: rep) else {
    FileHandle.standardError.write("NSGraphicsContext 构造失败\n".data(using: .utf8)!)
    exit(1)
}
NSGraphicsContext.current = ctx
// 内容等比居中：画布 px×px，内容边 = px × 源边/side（画布初始透明，alpha 保真）。
let drawW = Double(px) * Double(srcW) / Double(side)
let drawH = Double(px) * Double(srcH) / Double(side)
image.draw(
    in: NSRect(x: (Double(px) - drawW) / 2.0, y: (Double(px) - drawH) / 2.0,
               width: drawW, height: drawH),
    from: NSRect(origin: .zero, size: image.size),
    operation: .sourceOver,
    fraction: 1.0
)
NSGraphicsContext.restoreGraphicsState()

guard let png = rep.representation(using: .png, properties: [:]) else {
    FileHandle.standardError.write("PNG 编码失败\n".data(using: .utf8)!)
    exit(1)
}
do {
    try png.write(to: URL(fileURLWithPath: outPath))
} catch {
    FileHandle.standardError.write("PNG 写入失败: \(error)\n".data(using: .utf8)!)
    exit(1)
}
print("OK \(outPath) \(png.count) bytes \(px)x\(px) 内容 \(Int(drawW))x\(Int(drawH)) 居中 (源 \(srcW)x\(srcH))")