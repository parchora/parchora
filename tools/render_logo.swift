// Parchora 品牌 logo 离线渲染脚本（SVG → PNG · 第三十九轮）。
//
// 用法：swift render_logo.swift <input.svg> <output.png> [size]
// 产物：app/crates/parchora-app/assets/parchora-logo-512.png（经 include_bytes!
// 内嵌 parchora-app，platform::create_app_icon 经 NSData/NSImage 桥装配为
// NSApp.applicationIconImage）。渲染工具非构建依赖（规范 9 · 一次性脚本留档复现）。

import AppKit

guard CommandLine.arguments.count == 3 || CommandLine.arguments.count == 4 else {
    FileHandle.standardError.write("用法: swift render_logo.swift <in.svg> <out.png> [size]\n".data(using: .utf8)!)
    exit(2)
}
let svgPath = CommandLine.arguments[1]
let pngPath = CommandLine.arguments[2]
// 可选第 3 参：输出边长（像素）。缺省 512 保持既有行为；iconset 基底需 1024（第四十一轮 · DEC-92）。
let px: Int
if CommandLine.arguments.count == 4 {
    guard let parsed = Int(CommandLine.arguments[3]), parsed > 0 else {
        FileHandle.standardError.write("size 须为正整数: \(CommandLine.arguments[3])\n".data(using: .utf8)!)
        exit(2)
    }
    px = parsed
} else {
    px = 512
}

// NSImage 直读 SVG（CoreSVG 后端 · macOS 10.15+；本机 27.0.1 实证可用）。
guard let svg = NSImage(contentsOfFile: svgPath) else {
    FileHandle.standardError.write("NSImage 无法读取 SVG: \(svgPath)\n".data(using: .utf8)!)
    exit(1)
}

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
// 忠实原样缩放绘制（SVG 自含白底 rect，不做 template 化改色）。
svg.draw(
    in: NSRect(x: 0, y: 0, width: px, height: px),
    from: NSRect(origin: .zero, size: svg.size),
    operation: .sourceOver,
    fraction: 1.0
)
NSGraphicsContext.restoreGraphicsState()

guard let png = rep.representation(using: .png, properties: [:]) else {
    FileHandle.standardError.write("PNG 编码失败\n".data(using: .utf8)!)
    exit(1)
}
do {
    try png.write(to: URL(fileURLWithPath: pngPath))
} catch {
    FileHandle.standardError.write("PNG 写入失败: \(error)\n".data(using: .utf8)!)
    exit(1)
}
print("OK \(pngPath) \(png.count) bytes \(px)x\(px)")