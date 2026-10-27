// NSApp 应用图标真机探针（第三十九轮 · 留档复现）。
//
// 用法：swift probe_icon.swift <pid> <out.png>
// 经 NSRunningApplication(pid).icon 导出运行中实例的应用图标 PNG，供像素采样
// 比对品牌图形（米色纸张/黑描边羽毛 vs 通用暗色可执行图标）。注意：Accessory
// 激活策略下该探针可能返回通用图标（icon 来源与 setApplicationIconImage 的同步
// 语义存在系统实现差异）——探针失败不作装配失败的断言，stderr 自证行为主证据。

import AppKit

guard CommandLine.arguments.count == 3 else {
    FileHandle.standardError.write("用法: swift probe_icon.swift <pid> <out.png>\n".data(using: .utf8)!)
    exit(2)
}
guard let pid = Int32(CommandLine.arguments[1]), pid > 0 else {
    FileHandle.standardError.write("pid 非法: \(CommandLine.arguments[1])\n".data(using: .utf8)!)
    exit(2)
}
let outPath = CommandLine.arguments[2]

guard let app = NSRunningApplication(processIdentifier: pid) else {
    FileHandle.standardError.write("NSRunningApplication(\(pid)) 为 nil（进程不在运行列表）\n".data(using: .utf8)!)
    exit(1)
}
guard let icon = app.icon else {
    FileHandle.standardError.write("app.icon 为 nil\n".data(using: .utf8)!)
    exit(1)
}
FileHandle.standardError.write("icon logical size: \(icon.size)\n".data(using: .utf8)!)

let px = 512
guard let rep = NSBitmapImageRep(
    bitmapDataPlanes: nil,
    pixelsWide: px, pixelsHigh: px,
    bitsPerSample: 8, samplesPerPixel: 4, hasAlpha: true, isPlanar: false,
    colorSpaceName: .deviceRGB, bytesPerRow: 0, bitsPerPixel: 0
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
icon.draw(
    in: NSRect(x: 0, y: 0, width: px, height: px),
    from: NSRect(origin: .zero, size: icon.size),
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
    print("OK \(outPath) \(png.count) bytes")
} catch {
    FileHandle.standardError.write("PNG 写入失败: \(error)\n".data(using: .utf8)!)
    exit(1)
}