import AppKit

/// Edge/corner zones for resize hit-testing.
enum ResizeZone: CustomStringConvertible {
    case none
    case north, south, east, west
    case northEast, northWest, southEast, southWest
    case titleBar

    var description: String {
        switch self {
        case .none: return "none"
        case .north: return "N"
        case .south: return "S"
        case .east: return "E"
        case .west: return "W"
        case .northEast: return "NE"
        case .northWest: return "NW"
        case .southEast: return "SE"
        case .southWest: return "SW"
        case .titleBar: return "titleBar"
        }
    }
}

/// Tracks the active interaction state.
enum InteractionState {
    case idle
    case dragging(panelIndex: Int, initialFrame: NSRect, mouseStart: NSPoint)
    case resizing(panelIndex: Int, zone: ResizeZone, initialFrame: NSRect, mouseStart: NSPoint)
}

/// Manages panel drag-to-reposition and edge/corner resize via NSEvent monitors.
class PanelInteractionManager {

    // MARK: - Configuration
    static let edgeThreshold: CGFloat = 6
    static let cornerThreshold: CGFloat = 10
    static let titleBarHeight: CGFloat = 35
    static let minPanelWidth: CGFloat = 200
    static let minPanelHeight: CGFloat = 150

    // MARK: - State
    private(set) var interactionState: InteractionState = .idle
    weak var chatWebView: ChatWebView?

    // MARK: - Event Monitor References
    private var mouseDownMonitor: Any?
    private var mouseDraggedMonitor: Any?
    private var mouseUpMonitor: Any?
    private var mouseMovedMonitor: Any?

    // Throttle mouseMoved logging
    private var lastMoveLogTime: Date = .distantPast

    // MARK: - Install / Uninstall

    func install() {
        mouseDownMonitor = NSEvent.addLocalMonitorForEvents(matching: .leftMouseDown) { [weak self] event in
            self?.handleMouseDown(event) ?? event
        }
        mouseDraggedMonitor = NSEvent.addLocalMonitorForEvents(matching: .leftMouseDragged) { [weak self] event in
            self?.handleMouseDragged(event) ?? event
        }
        mouseUpMonitor = NSEvent.addLocalMonitorForEvents(matching: .leftMouseUp) { [weak self] event in
            self?.handleMouseUp(event) ?? event
        }
        mouseMovedMonitor = NSEvent.addLocalMonitorForEvents(matching: .mouseMoved) { [weak self] event in
            self?.handleMouseMoved(event) ?? event
        }
        metalLog("PanelInteractionManager: installed event monitors")
    }

    func uninstall() {
        if let m = mouseDownMonitor { NSEvent.removeMonitor(m) }
        if let m = mouseDraggedMonitor { NSEvent.removeMonitor(m) }
        if let m = mouseUpMonitor { NSEvent.removeMonitor(m) }
        if let m = mouseMovedMonitor { NSEvent.removeMonitor(m) }
        mouseDownMonitor = nil
        mouseDraggedMonitor = nil
        mouseUpMonitor = nil
        mouseMovedMonitor = nil
        metalLog("PanelInteractionManager: uninstalled event monitors")
    }

    deinit {
        uninstall()
    }

    // MARK: - Guards

    private func shouldIgnore() -> Bool {
        guard let chat = chatWebView else { return true }
        if chat.panels.isEmpty { return true }
        if chat.fullscreenIframeActive { return true }
        return false
    }

    // MARK: - Mouse Down

    private func handleMouseDown(_ event: NSEvent) -> NSEvent? {
        guard !shouldIgnore() else { return event }
        let point = event.locationInWindow

        guard let (index, zone) = hitTest(point: point) else { return event }

        guard let chat = chatWebView else { return event }

        // First interaction: transition to free-form
        if !chat.isFreeFormLayout {
            chat.enterFreeFormLayout()
        }

        let frame = chat.panels[index].frame

        switch zone {
        case .titleBar:
            interactionState = .dragging(
                panelIndex: index,
                initialFrame: frame,
                mouseStart: point
            )
            // Bring panel to front
            if let superview = chat.panels[index].superview {
                superview.addSubview(chat.panels[index], positioned: .above, relativeTo: nil)
            }
            NSCursor.closedHand.push()
            metalLog("PanelInteraction: drag START panel=\(index) at (\(Int(point.x)),\(Int(point.y)))")
            return nil

        case .none:
            return event

        default:
            interactionState = .resizing(
                panelIndex: index,
                zone: zone,
                initialFrame: frame,
                mouseStart: point
            )
            cursorForZone(zone).push()
            metalLog("PanelInteraction: resize START panel=\(index) zone=\(zone) at (\(Int(point.x)),\(Int(point.y)))")
            return nil
        }
    }

    // MARK: - Mouse Dragged

    private func handleMouseDragged(_ event: NSEvent) -> NSEvent? {
        let point = event.locationInWindow

        switch interactionState {
        case .dragging(let index, let initialFrame, let mouseStart):
            guard let chat = chatWebView, index < chat.panels.count else { return event }
            let dx = point.x - mouseStart.x
            let dy = point.y - mouseStart.y
            var newFrame = initialFrame
            newFrame.origin.x += dx
            newFrame.origin.y += dy

            // Clamp to parent bounds
            let parent = chat.parentFrame
            newFrame.origin.x = max(parent.minX, min(newFrame.origin.x, parent.maxX - newFrame.width))
            newFrame.origin.y = max(parent.minY, min(newFrame.origin.y, parent.maxY - newFrame.height))

            chat.panels[index].frame = newFrame
            if index < chat.panelFrames.count {
                chat.panelFrames[index] = newFrame
            }
            return nil

        case .resizing(let index, let zone, let initialFrame, let mouseStart):
            guard let chat = chatWebView, index < chat.panels.count else { return event }
            let newFrame = applyResize(zone: zone, currentMouse: point, mouseStart: mouseStart, initialFrame: initialFrame)
            chat.panels[index].frame = newFrame
            if index < chat.panelFrames.count {
                chat.panelFrames[index] = newFrame
            }
            return nil

        case .idle:
            return event
        }
    }

    // MARK: - Mouse Up

    private func handleMouseUp(_ event: NSEvent) -> NSEvent? {
        switch interactionState {
        case .dragging(let index, _, _):
            let frame = chatWebView?.panels[index].frame ?? .zero
            metalLog("PanelInteraction: drag END panel=\(index) frame=(\(Int(frame.origin.x)),\(Int(frame.origin.y)),\(Int(frame.width))x\(Int(frame.height)))")
            interactionState = .idle
            NSCursor.pop()
            return nil

        case .resizing(let index, let zone, _, _):
            let frame = chatWebView?.panels[index].frame ?? .zero
            metalLog("PanelInteraction: resize END panel=\(index) zone=\(zone) frame=(\(Int(frame.origin.x)),\(Int(frame.origin.y)),\(Int(frame.width))x\(Int(frame.height)))")
            interactionState = .idle
            NSCursor.pop()
            return nil

        case .idle:
            return event
        }
    }

    // MARK: - Mouse Moved (cursor updates)

    private func handleMouseMoved(_ event: NSEvent) -> NSEvent? {
        guard !shouldIgnore() else { return event }
        guard case .idle = interactionState else { return event }

        let point = event.locationInWindow
        if let (_, zone) = hitTest(point: point) {
            cursorForZone(zone).set()

            // Throttled logging (every 2 seconds)
            let now = Date()
            if now.timeIntervalSince(lastMoveLogTime) > 2.0 {
                lastMoveLogTime = now
                metalLog("PanelInteraction: hover zone=\(zone) at (\(Int(point.x)),\(Int(point.y)))")
            }
        } else {
            NSCursor.arrow.set()
        }
        return event  // always pass through mouseMoved
    }

    // MARK: - Resize Logic

    private func applyResize(zone: ResizeZone, currentMouse: NSPoint, mouseStart: NSPoint, initialFrame: NSRect) -> NSRect {
        let dx = currentMouse.x - mouseStart.x
        let dy = currentMouse.y - mouseStart.y
        var f = initialFrame

        // Horizontal
        switch zone {
        case .east, .northEast, .southEast:
            f.size.width = max(Self.minPanelWidth, initialFrame.width + dx)
        case .west, .northWest, .southWest:
            let newWidth = max(Self.minPanelWidth, initialFrame.width - dx)
            f.origin.x = initialFrame.maxX - newWidth
            f.size.width = newWidth
        default: break
        }

        // Vertical (macOS: y=0 is bottom, north = top = higher y)
        switch zone {
        case .north, .northEast, .northWest:
            f.size.height = max(Self.minPanelHeight, initialFrame.height + dy)
        case .south, .southEast, .southWest:
            let newHeight = max(Self.minPanelHeight, initialFrame.height - dy)
            f.origin.y = initialFrame.maxY - newHeight
            f.size.height = newHeight
        default: break
        }

        return f
    }

    // MARK: - Cursor

    func cursorForZone(_ zone: ResizeZone) -> NSCursor {
        switch zone {
        case .north, .south:         return .resizeUpDown
        case .east, .west:           return .resizeLeftRight
        case .northEast, .southWest: return .crosshair
        case .northWest, .southEast: return .crosshair
        case .titleBar:              return .openHand
        case .none:                  return .arrow
        }
    }
}
