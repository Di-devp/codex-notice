import AppKit

let root = URL(fileURLWithPath: FileManager.default.currentDirectoryPath)
let iconPath = root.appendingPathComponent("src-tauri/icons/icon.png").path
let out16 = root.appendingPathComponent("docs/videos/covers/notice-cover-tech-16x9.png").path
let out9 = root.appendingPathComponent("docs/videos/covers/notice-cover-tech-9x16.png").path

let green = NSColor(calibratedRed: 0.27, green: 0.95, blue: 0.66, alpha: 1)
let deepGreen = NSColor(calibratedRed: 0.05, green: 0.22, blue: 0.17, alpha: 1)
let panel = NSColor(calibratedRed: 0.055, green: 0.065, blue: 0.07, alpha: 0.94)
let panel2 = NSColor(calibratedRed: 0.075, green: 0.08, blue: 0.09, alpha: 0.98)
let line = NSColor(calibratedRed: 0.17, green: 0.24, blue: 0.22, alpha: 0.9)
let muted = NSColor(calibratedRed: 0.68, green: 0.73, blue: 0.72, alpha: 1)
let white = NSColor(calibratedWhite: 0.96, alpha: 1)

extension NSBezierPath {
    convenience init(rounded rect: CGRect, radius: CGFloat) {
        self.init(roundedRect: rect, xRadius: radius, yRadius: radius)
    }
}

func fillRounded(_ rect: CGRect, radius: CGFloat, color: NSColor, stroke: NSColor? = nil, width: CGFloat = 1) {
    let path = NSBezierPath(rounded: rect, radius: radius)
    color.setFill()
    path.fill()
    if let stroke {
        stroke.setStroke()
        path.lineWidth = width
        path.stroke()
    }
}

func text(_ value: String, x: CGFloat, y: CGFloat, size: CGFloat, weight: NSFont.Weight = .regular, color: NSColor = white, maxWidth: CGFloat = 2000, align: NSTextAlignment = .left) {
    let paragraph = NSMutableParagraphStyle()
    paragraph.alignment = align
    paragraph.lineSpacing = size * 0.12
    let font = NSFont.systemFont(ofSize: size, weight: weight)
    let attrs: [NSAttributedString.Key: Any] = [
        .font: font,
        .foregroundColor: color,
        .paragraphStyle: paragraph
    ]
    let lineCount = CGFloat(value.split(separator: "\n", omittingEmptySubsequences: false).count)
    let height = size * (lineCount * 1.25 + 0.18)
    let attributed = NSAttributedString(string: value, attributes: attrs)
    attributed.draw(with: CGRect(x: x, y: y, width: maxWidth, height: height), options: [.usesLineFragmentOrigin, .usesFontLeading])
}

func drawGradientBackground(size: CGSize) {
    NSColor(calibratedRed: 0.015, green: 0.02, blue: 0.022, alpha: 1).setFill()
    CGRect(origin: .zero, size: size).fill()

    let glow = NSGradient(colors: [
        NSColor(calibratedRed: 0.00, green: 0.85, blue: 0.48, alpha: 0.42),
        NSColor(calibratedRed: 0.00, green: 0.36, blue: 0.28, alpha: 0.14),
        NSColor.clear
    ])!
    glow.draw(in: CGRect(x: size.width * 0.19, y: -size.height * 0.05, width: size.width * 0.7, height: size.height * 0.52), angle: 90)
    glow.draw(in: CGRect(x: -size.width * 0.15, y: size.height * 0.55, width: size.width * 0.55, height: size.height * 0.45), angle: -30)

    NSColor(calibratedWhite: 1, alpha: 0.035).setStroke()
    for i in stride(from: CGFloat(0), through: size.width, by: 80) {
        let path = NSBezierPath()
        path.move(to: CGPoint(x: i, y: 0))
        path.line(to: CGPoint(x: i + size.height * 0.18, y: size.height))
        path.lineWidth = 0.5
        path.stroke()
    }
}

func drawTrafficLight(x: CGFloat, y: CGFloat, scale: CGFloat) {
    let body = CGRect(x: x, y: y, width: 56 * scale, height: 150 * scale)
    fillRounded(body, radius: 24 * scale, color: NSColor(calibratedWhite: 0.02, alpha: 0.95), stroke: NSColor(calibratedWhite: 1, alpha: 0.08), width: 1.2 * scale)

    let lights: [(NSColor, CGFloat)] = [
        (NSColor(calibratedRed: 1, green: 0.19, blue: 0.18, alpha: 1), y + 105 * scale),
        (NSColor(calibratedRed: 1, green: 0.87, blue: 0.08, alpha: 1), y + 60 * scale),
        (NSColor(calibratedRed: 0.18, green: 0.95, blue: 0.35, alpha: 1), y + 15 * scale)
    ]
    for (color, ly) in lights {
        let shadow = NSShadow()
        shadow.shadowBlurRadius = 18 * scale
        shadow.shadowColor = color.withAlphaComponent(0.55)
        shadow.set()
        color.setFill()
        NSBezierPath(ovalIn: CGRect(x: x + 13 * scale, y: ly, width: 30 * scale, height: 30 * scale)).fill()
        NSShadow().set()
    }
}

func drawStatusCard(_ rect: CGRect, compact: Bool = false) {
    let shadow = NSShadow()
    shadow.shadowBlurRadius = 28
    shadow.shadowOffset = CGSize(width: 0, height: -8)
    shadow.shadowColor = green.withAlphaComponent(0.22)
    shadow.set()
    fillRounded(rect, radius: 28, color: NSColor(calibratedRed: 0.04, green: 0.08, blue: 0.075, alpha: 0.92), stroke: green.withAlphaComponent(0.55), width: 1.5)
    NSShadow().set()

    let s = compact ? CGFloat(0.78) : CGFloat(1)
    drawTrafficLight(x: rect.minX + 34 * s, y: rect.minY + rect.height / 2 - 58 * s, scale: 0.86 * s)
    text("运行中", x: rect.minX + 118 * s, y: rect.minY + 38 * s, size: 42 * s, weight: .bold)
    text("↑ 1 个 Codex 会话运行中", x: rect.minX + 120 * s, y: rect.minY + 92 * s, size: 25 * s, weight: .semibold, color: muted)
}

func drawNoticeWindow(_ rect: CGRect, selected: String = "通知渠道", scale: CGFloat = 1) {
    let shadow = NSShadow()
    shadow.shadowBlurRadius = 36 * scale
    shadow.shadowOffset = CGSize(width: 0, height: -10 * scale)
    shadow.shadowColor = NSColor.black.withAlphaComponent(0.5)
    shadow.set()
    fillRounded(rect, radius: 32 * scale, color: panel, stroke: line, width: 2 * scale)
    NSShadow().set()

    let top = rect.maxY - 54 * scale
    for i in 0..<3 {
        NSColor(calibratedWhite: 0.33, alpha: 0.75).setFill()
        NSBezierPath(ovalIn: CGRect(x: rect.minX + 28 * scale + CGFloat(i) * 34 * scale, y: top + 18 * scale, width: 15 * scale, height: 15 * scale)).fill()
    }
    text("Notice", x: rect.minX + 132 * scale, y: top + 8 * scale, size: 22 * scale, weight: .bold, color: NSColor(calibratedWhite: 0.6, alpha: 1))

    let sidebarW = 300 * scale
    NSColor(calibratedRed: 0.025, green: 0.045, blue: 0.042, alpha: 0.92).setFill()
    CGRect(x: rect.minX, y: rect.minY, width: sidebarW, height: rect.height - 55 * scale).fill()
    line.setStroke()
    let divider = NSBezierPath()
    divider.move(to: CGPoint(x: rect.minX + sidebarW, y: rect.minY))
    divider.line(to: CGPoint(x: rect.minX + sidebarW, y: rect.maxY - 55 * scale))
    divider.lineWidth = 1 * scale
    divider.stroke()

    text("Notice", x: rect.minX + 38 * scale, y: rect.maxY - 136 * scale, size: 36 * scale, weight: .bold)
    text("开发者通知中心", x: rect.minX + 38 * scale, y: rect.maxY - 188 * scale, size: 19 * scale, weight: .medium, color: NSColor(calibratedRed: 0.56, green: 0.72, blue: 0.67, alpha: 1))

    let menu = ["仪表盘", "事件", "通知渠道", "接入源", "审批", "设置"]
    for (idx, item) in menu.enumerated() {
        let y = rect.maxY - (278 + CGFloat(idx) * 78) * scale
        if item == selected {
            fillRounded(CGRect(x: rect.minX + 28 * scale, y: y - 20 * scale, width: sidebarW - 56 * scale, height: 56 * scale), radius: 8 * scale, color: deepGreen.withAlphaComponent(0.9))
        }
        text(item, x: rect.minX + 60 * scale, y: y - 8 * scale, size: 24 * scale, weight: .bold, color: item == selected ? green : white.withAlphaComponent(0.92))
    }

    let contentX = rect.minX + sidebarW + 44 * scale
    text("通知渠道", x: contentX, y: rect.maxY - 150 * scale, size: 40 * scale, weight: .bold)
    let cardsY = rect.maxY - 330 * scale
    for i in 0..<3 {
        let card = CGRect(x: contentX + CGFloat(i) * 255 * scale, y: cardsY, width: 225 * scale, height: 130 * scale)
        fillRounded(card, radius: 10 * scale, color: panel2, stroke: NSColor(calibratedWhite: 1, alpha: 0.09), width: 1 * scale)
        let title = ["飞书", "通知策略", "发送状态"][i]
        let value = ["已配置", "关键节点", "正常"][i]
        text(title, x: card.minX + 24 * scale, y: card.maxY - 48 * scale, size: 20 * scale, weight: .bold, color: muted)
        text(value, x: card.minX + 24 * scale, y: card.maxY - 94 * scale, size: 26 * scale, weight: .bold, color: i == 0 ? green : white)
    }

    let wide = CGRect(x: contentX, y: rect.minY + 80 * scale, width: rect.width - sidebarW - 88 * scale, height: 190 * scale)
    fillRounded(wide, radius: 12 * scale, color: panel2, stroke: NSColor(calibratedWhite: 1, alpha: 0.09), width: 1 * scale)
    text("只在关键节点通知", x: wide.minX + 28 * scale, y: wide.maxY - 58 * scale, size: 28 * scale, weight: .bold)
    text("需要审批时提醒 · 任务成功结束后提醒 · 中间工具调用不刷屏", x: wide.minX + 28 * scale, y: wide.maxY - 112 * scale, size: 22 * scale, weight: .medium, color: muted, maxWidth: wide.width - 56 * scale)
}

func drawAppIcon(x: CGFloat, y: CGFloat, size: CGFloat) {
    guard let icon = NSImage(contentsOfFile: iconPath) else { return }
    let rect = CGRect(x: x, y: y, width: size, height: size)
    let shadow = NSShadow()
    shadow.shadowBlurRadius = 22
    shadow.shadowOffset = CGSize(width: 0, height: -4)
    shadow.shadowColor = NSColor.black.withAlphaComponent(0.45)
    shadow.set()
    fillRounded(rect, radius: size * 0.22, color: NSColor.white.withAlphaComponent(0.96), stroke: NSColor.white.withAlphaComponent(0.25), width: 1)
    NSShadow().set()
    NSGraphicsContext.saveGraphicsState()
    NSBezierPath(rounded: rect, radius: size * 0.22).addClip()
    icon.draw(in: rect)
    NSGraphicsContext.restoreGraphicsState()
}

func save(_ image: NSImage, to path: String) {
    guard let tiff = image.tiffRepresentation,
          let rep = NSBitmapImageRep(data: tiff),
          let data = rep.representation(using: .png, properties: [:]) else {
        fatalError("Cannot encode PNG")
    }
    try! data.write(to: URL(fileURLWithPath: path))
}

func render16x9() {
    let size = CGSize(width: 1920, height: 1080)
    let image = NSImage(size: size)
    image.lockFocus()
    drawGradientBackground(size: size)

    fillRounded(CGRect(x: 86, y: 910, width: 410, height: 72), radius: 36, color: deepGreen.withAlphaComponent(0.5), stroke: green.withAlphaComponent(0.55), width: 1.5)
    text("</>  开发者效率工具", x: 132, y: 925, size: 30, weight: .bold, color: green)
    text("Codex任务状态", x: 86, y: 682, size: 112, weight: .heavy)
    text("一眼看清", x: 86, y: 552, size: 128, weight: .heavy, color: NSColor(calibratedRed: 0.78, green: 1, blue: 0.9, alpha: 1))
    text("红 黄 绿 灯提醒 · 飞书关键通知", x: 90, y: 500, size: 40, weight: .bold, color: muted)

    drawStatusCard(CGRect(x: 610, y: 554, width: 620, height: 170))
    drawNoticeWindow(CGRect(x: 660, y: 110, width: 1120, height: 460), selected: "通知渠道", scale: 0.82)

    drawAppIcon(x: 120, y: 130, size: 112)
    text("Notice", x: 260, y: 150, size: 62, weight: .heavy)
    text("少一点打扰，多一点确定感", x: 122, y: 76, size: 34, weight: .medium, color: muted)

    image.unlockFocus()
    save(image, to: out16)
}

func render9x16() {
    let size = CGSize(width: 1080, height: 1920)
    let image = NSImage(size: size)
    image.lockFocus()
    drawGradientBackground(size: size)

    fillRounded(CGRect(x: 72, y: 1720, width: 415, height: 72), radius: 36, color: deepGreen.withAlphaComponent(0.5), stroke: green.withAlphaComponent(0.55), width: 1.5)
    text("</>  开发者效率工具", x: 118, y: 1735, size: 31, weight: .bold, color: green)
    text("Codex任务状态", x: 70, y: 1436, size: 100, weight: .heavy)
    text("一眼看清", x: 70, y: 1308, size: 118, weight: .heavy, color: NSColor(calibratedRed: 0.78, green: 1, blue: 0.9, alpha: 1))
    text("红 黄 绿 灯提醒 · 飞书关键通知", x: 74, y: 1244, size: 39, weight: .bold, color: muted)

    drawStatusCard(CGRect(x: 190, y: 1012, width: 700, height: 190))
    drawNoticeWindow(CGRect(x: 58, y: 398, width: 964, height: 540), selected: "通知渠道", scale: 0.7)

    drawAppIcon(x: 310, y: 188, size: 116)
    text("Notice", x: 450, y: 206, size: 70, weight: .heavy)
    text("少一点打扰，多一点确定感", x: 0, y: 108, size: 35, weight: .medium, color: muted, maxWidth: 1080, align: .center)

    image.unlockFocus()
    save(image, to: out9)
}

render16x9()
render9x16()
print("Generated:")
print(out16)
print(out9)
