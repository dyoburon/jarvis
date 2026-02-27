import AppKit

extension PanelInteractionManager {

    /// Determine which panel and zone a point falls on.
    /// Checks corners first (they overlap edges), then edges, then title bar.
    /// Returns nil if the point is in a panel interior (event passes to WKWebView).
    func hitTest(point: NSPoint) -> (panelIndex: Int, zone: ResizeZone)? {
        guard let chat = chatWebView else { return nil }

        // Iterate in reverse so topmost (last-added/last-focused) panel wins
        for i in stride(from: chat.panels.count - 1, through: 0, by: -1) {
            let panel = chat.panels[i]
            let frame = panel.convert(panel.bounds, to: nil)  // window coordinates

            // Check if point is near this panel at all (expanded by edge threshold)
            let expanded = frame.insetBy(dx: -Self.edgeThreshold, dy: -Self.edgeThreshold)
            guard expanded.contains(point) else { continue }

            let ct = Self.cornerThreshold
            let et = Self.edgeThreshold

            // Corner checks (higher priority — they overlap edge zones)
            // NE corner: top-right
            if point.x >= frame.maxX - ct && point.y >= frame.maxY - ct {
                return (i, .northEast)
            }
            // NW corner: top-left
            if point.x <= frame.minX + ct && point.y >= frame.maxY - ct {
                return (i, .northWest)
            }
            // SE corner: bottom-right
            if point.x >= frame.maxX - ct && point.y <= frame.minY + ct {
                return (i, .southEast)
            }
            // SW corner: bottom-left
            if point.x <= frame.minX + ct && point.y <= frame.minY + ct {
                return (i, .southWest)
            }

            // Edge checks (exclude corner regions)
            // East edge
            if point.x >= frame.maxX - et && point.y > frame.minY + ct && point.y < frame.maxY - ct {
                return (i, .east)
            }
            // West edge
            if point.x <= frame.minX + et && point.y > frame.minY + ct && point.y < frame.maxY - ct {
                return (i, .west)
            }
            // North edge (top)
            if point.y >= frame.maxY - et && point.x > frame.minX + ct && point.x < frame.maxX - ct {
                return (i, .north)
            }
            // South edge (bottom)
            if point.y <= frame.minY + et && point.x > frame.minX + ct && point.x < frame.maxX - ct {
                return (i, .south)
            }

            // Title bar: top 35px of the panel, excluding edge/corner zones
            // macOS y=0 is bottom, so title bar is at the top = frame.maxY side
            let titleBarRect = NSRect(
                x: frame.minX + ct,
                y: frame.maxY - Self.titleBarHeight,
                width: frame.width - 2 * ct,
                height: Self.titleBarHeight - et
            )
            if titleBarRect.contains(point) {
                return (i, .titleBar)
            }

            // Point is inside panel interior — let WKWebView handle it
            if frame.contains(point) {
                return nil
            }
        }

        return nil
    }
}
