import AppKit

/// Draggable handle between panels for resizing.
class PanelResizeHandle: NSView {
    var onDrag: ((CGFloat) -> Void)?
    private var dragStartX: CGFloat = 0
    private var isHovered = false
    private var isDragging = false
    private var trackingArea: NSTrackingArea?

    override func updateTrackingAreas() {
        super.updateTrackingAreas()
        if let ta = trackingArea { removeTrackingArea(ta) }
        let ta = NSTrackingArea(
            rect: bounds,
            options: [.mouseEnteredAndExited, .activeAlways, .cursorUpdate],
            owner: self, userInfo: nil
        )
        addTrackingArea(ta)
        trackingArea = ta
    }

    // resetCursorRects is more reliable than cursorUpdate in borderless windows
    override func resetCursorRects() {
        addCursorRect(bounds, cursor: .resizeLeftRight)
    }

    override func cursorUpdate(with event: NSEvent) {
        NSCursor.resizeLeftRight.set()
    }

    override func mouseEntered(with event: NSEvent) {
        isHovered = true
        needsDisplay = true
        NSCursor.resizeLeftRight.set()
        metalLog("PanelResizeHandle: mouseEntered frame=\(frame)")
    }

    override func mouseExited(with event: NSEvent) {
        isHovered = false
        needsDisplay = true
        if !isDragging {
            NSCursor.arrow.set()
        }
    }

    override func mouseDown(with event: NSEvent) {
        dragStartX = event.locationInWindow.x
        isDragging = true
        NSCursor.resizeLeftRight.push()
        metalLog("PanelResizeHandle: mouseDown x=\(dragStartX)")
    }

    override func mouseDragged(with event: NSEvent) {
        let delta = event.locationInWindow.x - dragStartX
        dragStartX = event.locationInWindow.x
        onDrag?(delta)
    }

    override func mouseUp(with event: NSEvent) {
        isDragging = false
        NSCursor.pop()
        metalLog("PanelResizeHandle: mouseUp")
    }

    override func draw(_ dirtyRect: NSRect) {
        let alpha: CGFloat = isHovered || isDragging ? 0.3 : 0.1
        NSColor(calibratedWhite: 1.0, alpha: alpha).setFill()
        // Full-height subtle background strip
        NSRect(x: bounds.midX - 1.5, y: 0, width: 3, height: bounds.height).fill()
        // Center knob indicator
        let knobAlpha: CGFloat = isHovered || isDragging ? 0.6 : 0.2
        NSColor(calibratedWhite: 1.0, alpha: knobAlpha).setFill()
        let knob = NSRect(x: bounds.midX - 1.5, y: bounds.midY - 20, width: 3, height: 40)
        knob.fill()
    }

    override func acceptsFirstMouse(for event: NSEvent?) -> Bool { true }
}
