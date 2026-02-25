import Metal
import AppKit
import CoreText

/// Renders HUD text (boot logs, status text) into an MTLTexture via Core Text.
/// The texture is sampled in the Metal shader so text gets the full CRT treatment.
class HUDTextRenderer {
    let device: MTLDevice
    let texture: MTLTexture
    private let width: Int
    private let height: Int
    private let bytesPerRow: Int
    private var pixelData: [UInt8]

    private var lines: [String] = []
    private var statusText: String?
    private var topRightLines: [String] = []
    private var opacity: Float = 1.0
    private let maxLines = 18

    // Font metrics
    private let fontSize: CGFloat = 22
    private let statusFontSize: CGFloat = 28
    private let lineSpacing: CGFloat = 30
    private let marginX: CGFloat = 60
    private let marginY: CGFloat = 80

    init(device: MTLDevice, width: Int, height: Int) {
        self.device = device
        self.width = width
        self.height = height
        self.bytesPerRow = width * 4

        // Allocate pixel buffer
        pixelData = [UInt8](repeating: 0, count: bytesPerRow * height)

        // Create Metal texture
        let desc = MTLTextureDescriptor.texture2DDescriptor(
            pixelFormat: .rgba8Unorm,
            width: width,
            height: height,
            mipmapped: false
        )
        desc.usage = [.shaderRead]
        texture = device.makeTexture(descriptor: desc)!

        // Initial clear
        redraw()
    }

    func appendLine(_ line: String) {
        lines.append(line)
        if lines.count > maxLines {
            lines.removeFirst()
        }
        redraw()
    }

    func clearLines() {
        lines.removeAll()
        redraw()
    }

    func setStatusText(_ text: String?) {
        statusText = text
        redraw()
    }

    func setTopRightText(_ text: String?) {
        if let t = text {
            self.topRightLines = t.components(separatedBy: "\n")
        } else {
            self.topRightLines = []
        }
        redraw()
    }

    func setOpacity(_ opacity: Float) {
        self.opacity = opacity
        redraw()
    }

    private func redraw() {
        // Create CGContext
        let colorSpace = CGColorSpaceCreateDeviceRGB()
        guard let ctx = CGContext(
            data: &pixelData,
            width: width,
            height: height,
            bitsPerComponent: 8,
            bytesPerRow: bytesPerRow,
            space: colorSpace,
            bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue
        ) else { return }

        // Clear to transparent
        ctx.clear(CGRect(x: 0, y: 0, width: width, height: height))

        // Wrap in NSGraphicsContext with flipped coords (origin = top-left)
        let nsCtx = NSGraphicsContext(cgContext: ctx, flipped: true)
        NSGraphicsContext.saveGraphicsState()
        NSGraphicsContext.current = nsCtx

        let alpha = CGFloat(opacity)

        // Cyan HUD color
        let textColor = NSColor(
            red: 0.0, green: 0.83, blue: 1.0, alpha: alpha
        )
        let glowColor = NSColor(
            red: 0.0, green: 0.6, blue: 0.9, alpha: alpha * 0.6
        )

        let font = NSFont(name: "Menlo-Bold", size: fontSize)
            ?? NSFont.monospacedSystemFont(ofSize: fontSize, weight: .bold)

        // Draw boot log lines
        let shadow = NSShadow()
        shadow.shadowColor = glowColor
        shadow.shadowBlurRadius = 8
        shadow.shadowOffset = .zero

        let attrs: [NSAttributedString.Key: Any] = [
            .font: font,
            .foregroundColor: textColor,
            .shadow: shadow
        ]

        for (i, line) in lines.enumerated() {
            let y = marginY + CGFloat(i) * lineSpacing
            let str = NSAttributedString(string: line, attributes: attrs)
            str.draw(at: NSPoint(x: marginX, y: y))
        }

        // Draw top-right chat overlay — constrained to right 28% of screen
        if !topRightLines.isEmpty {
            let chatFontSize: CGFloat = 16
            let chatFont = NSFont(name: "Menlo", size: chatFontSize)
                ?? NSFont.monospacedSystemFont(ofSize: chatFontSize, weight: .regular)
            let chatColor = NSColor(
                red: 0.0, green: 0.83, blue: 1.0, alpha: alpha * 0.5
            )
            let chatShadow = NSShadow()
            chatShadow.shadowColor = glowColor
            chatShadow.shadowBlurRadius = 4
            chatShadow.shadowOffset = .zero

            let paraStyle = NSMutableParagraphStyle()
            paraStyle.alignment = .right
            paraStyle.lineBreakMode = .byWordWrapping

            let chatAttrs: [NSAttributedString.Key: Any] = [
                .font: chatFont,
                .foregroundColor: chatColor,
                .shadow: chatShadow,
                .paragraphStyle: paraStyle
            ]

            // Chat panels occupy left 72%; overlay stays in right 28%
            let overlayLeft = CGFloat(width) * 0.72 + marginX
            let overlayWidth = CGFloat(width) - overlayLeft - marginX
            let maxChat = min(topRightLines.count, 8)
            let visibleLines = topRightLines.suffix(maxChat)
            let text = visibleLines.joined(separator: "\n")
            let str = NSAttributedString(string: text, attributes: chatAttrs)
            let drawRect = CGRect(
                x: overlayLeft,
                y: marginY,
                width: overlayWidth,
                height: CGFloat(height) * 0.5
            )
            str.draw(with: drawRect, options: [.usesLineFragmentOrigin, .truncatesLastVisibleLine])
        }

        // Draw status text (centered, larger)
        if let status = statusText, alpha > 0 {
            let statusFont = NSFont(name: "Menlo-Bold", size: statusFontSize)
                ?? NSFont.monospacedSystemFont(ofSize: statusFontSize, weight: .bold)

            let statusShadow = NSShadow()
            statusShadow.shadowColor = glowColor
            statusShadow.shadowBlurRadius = 15
            statusShadow.shadowOffset = .zero

            let statusAttrs: [NSAttributedString.Key: Any] = [
                .font: statusFont,
                .foregroundColor: textColor,
                .shadow: statusShadow
            ]

            let str = NSAttributedString(string: status, attributes: statusAttrs)
            let size = str.size()
            let x = (CGFloat(width) - size.width) / 2
            let y = CGFloat(height) * 0.62  // below center (orb is at center)
            str.draw(at: NSPoint(x: x, y: y))
        }

        NSGraphicsContext.restoreGraphicsState()

        // Upload to Metal texture
        let region = MTLRegion(
            origin: MTLOrigin(x: 0, y: 0, z: 0),
            size: MTLSize(width: width, height: height, depth: 1)
        )
        texture.replace(
            region: region,
            mipmapLevel: 0,
            withBytes: pixelData,
            bytesPerRow: bytesPerRow
        )
    }
}
