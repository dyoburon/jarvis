import AppKit

extension ChatWebView {
    func relayoutPanels() {
        let count = panels.count
        guard count > 0 else { return }

        // Free-form mode: apply stored frames directly
        if isFreeFormLayout {
            for (i, wv) in panels.enumerated() {
                if i < panelFrames.count {
                    wv.frame = panelFrames[i]
                }
            }
            metalLog("relayoutPanels: freeForm applied \(count) frames")
            return
        }

        let gap: CGFloat = 2
        let handleWidth: CGFloat = 10
        let lf = layoutFrame  // tiled layout uses 72% width; drag/resize uses full parentFrame

        if count <= 3 {
            // Single row: use width ratios for resizable panels
            ensureRatios(count: count)
            let totalHandleWidth = handleWidth * CGFloat(count - 1)
            let availableWidth = lf.width - totalHandleWidth
            var x = lf.origin.x
            for (i, wv) in panels.enumerated() {
                let w = availableWidth * panelWidthRatios[i]
                wv.frame = NSRect(
                    x: x,
                    y: lf.origin.y,
                    width: w,
                    height: lf.height
                )
                x += w
                // Position resize handle after this panel (except the last)
                if i < resizeHandles.count {
                    let handle = resizeHandles[i]
                    handle.frame = NSRect(
                        x: x,
                        y: lf.origin.y,
                        width: handleWidth,
                        height: lf.height
                    )
                    // Force tracking area + cursor rect refresh after frame change
                    handle.updateTrackingAreas()
                    handle.window?.invalidateCursorRects(for: handle)
                    x += handleWidth
                }
            }
        } else {
            // 2 rows, 3 columns — fill left-to-right, top-to-bottom
            let cols = 3
            let colW = (lf.width - gap * CGFloat(cols - 1)) / CGFloat(cols)
            let rowH = (lf.height - gap) / 2
            for (i, wv) in panels.enumerated() {
                let col = i % cols
                let row = i / cols  // 0 = top row, 1 = bottom row
                // macOS: y=0 is bottom, so top row has higher y
                let y = row == 0
                    ? lf.origin.y + rowH + gap
                    : lf.origin.y
                wv.frame = NSRect(
                    x: lf.origin.x + CGFloat(col) * (colW + gap),
                    y: y,
                    width: colW,
                    height: rowH
                )
            }
        }
    }

    /// Ensure panelWidthRatios has the right count, resetting to equal if needed.
    func ensureRatios(count: Int) {
        if panelWidthRatios.count != count {
            panelWidthRatios = Array(repeating: 1.0 / CGFloat(count), count: count)
        }
    }

    /// Rebuild resize handles to match current panel count.
    func rebuildResizeHandles() {
        for h in resizeHandles { h.removeFromSuperview() }
        resizeHandles.removeAll()

        // No between-panel handles in free-form mode
        guard !isFreeFormLayout else { return }

        let count = panels.count
        guard count > 1, count <= 3, let parent = parentView else { return }

        let handleWidth: CGFloat = 10
        let minRatio: CGFloat = 0.1

        for i in 0..<(count - 1) {
            let handle = PanelResizeHandle()
            handle.frame = .zero
            let leftIndex = i
            let rightIndex = i + 1
            handle.onDrag = { [weak self] delta in
                guard let self = self else { return }
                let totalHandleWidth = handleWidth * CGFloat(self.panels.count - 1)
                let availableWidth = self.layoutFrame.width - totalHandleWidth
                guard availableWidth > 0 else { return }
                let deltaRatio = delta / availableWidth
                var newLeft = self.panelWidthRatios[leftIndex] + deltaRatio
                var newRight = self.panelWidthRatios[rightIndex] - deltaRatio
                // Enforce minimum width
                if newLeft < minRatio {
                    newRight -= (minRatio - newLeft)
                    newLeft = minRatio
                }
                if newRight < minRatio {
                    newLeft -= (minRatio - newRight)
                    newRight = minRatio
                }
                self.panelWidthRatios[leftIndex] = newLeft
                self.panelWidthRatios[rightIndex] = newRight
                self.relayoutPanels()
            }
            parent.addSubview(handle, positioned: .above, relativeTo: nil)
            resizeHandles.append(handle)
        }
    }

    /// Snapshot current panel frames for free-form editing.
    func captureCurrentFrames() {
        panelFrames = panels.map { $0.frame }
        let desc = panelFrames.map { "(\(Int($0.origin.x)),\(Int($0.origin.y)) \(Int($0.width))x\(Int($0.height)))" }.joined(separator: ", ")
        metalLog("captureCurrentFrames: [\(desc)]")
    }

    /// Transition from tiled to free-form layout.
    func enterFreeFormLayout() {
        guard !isFreeFormLayout else { return }
        captureCurrentFrames()
        isFreeFormLayout = true
        // Remove tiled resize handles; interaction manager takes over
        for h in resizeHandles { h.removeFromSuperview() }
        resizeHandles.removeAll()
        metalLog("enterFreeFormLayout: transitioned with \(panels.count) panels")
    }

    /// Reset to tiled layout.
    func resetToTiledLayout() {
        isFreeFormLayout = false
        panelFrames.removeAll()
        panelWidthRatios = Array(repeating: 1.0 / CGFloat(panels.count), count: panels.count)
        rebuildResizeHandles()
        relayoutPanels()
        metalLog("resetToTiledLayout: reset \(panels.count) panels to grid")
    }

    func removeAllPanels() {
        for h in resizeHandles { h.removeFromSuperview() }
        resizeHandles.removeAll()
        panelWidthRatios.removeAll()
        panelFrames.removeAll()
        isFreeFormLayout = false
        for wv in panels { wv.removeFromSuperview() }
        panels.removeAll()
    }
}
