/**
 * ChatWebView+Focus.swift
 *
 * Focus handling extension for ChatWebView.
 * Integrates with FocusManager for deterministic focus tracking.
 *
 * @module ChatWebView/ChatWebView+Focus
 */

import AppKit
import WebKit

// =============================================================================
// EXTENSION: FOCUS HANDLING
// =============================================================================

extension ChatWebView {
    
    // MARK: - Deterministic Focus Updates
    
    /// Update focus indicators on all panels.
    /// This is now deterministic based on FocusManager state.
    func updateFocusIndicatorsWithManager(_ manager: FocusManager) {
        let activeIdx = manager.getActivePanel()
        metalLog("updateFocusIndicators: activePanel=\(activeIdx) panelCount=\(panels.count)")
        
        for (i, wv) in panels.enumerated() {
            let isFocused = (i == activeIdx)
            let focused = isFocused ? "true" : "false"
            wv.evaluateJavaScript("setFocused(\(focused))", completionHandler: nil)
        }
        
        // Request first responder for the active panel
        requestFirstResponderForPanel(activeIdx, manager: manager)
    }
    
    /// Request first responder status for a specific panel.
    /// Includes delay to handle HTML loading timing.
    func requestFirstResponderForPanel(_ index: Int, manager: FocusManager) {
        guard index >= 0, index < panels.count else { return }
        
        let wv = panels[index]
        let capturedPanel = index
        
        // Delay to allow HTML to finish loading
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.3) { [weak wv] in
            guard let wv = wv else { return }
            
            // Only proceed if this is still the active panel
            guard manager.getActivePanel() == capturedPanel else {
                metalLog("requestFirstResponder: panel \(capturedPanel) no longer active, skipping")
                return
            }
            
            let before = wv.window?.firstResponder
            wv.window?.makeFirstResponder(wv)
            let after = wv.window?.firstResponder
            
            // Notify manager of first responder change
            if before !== after && after === wv {
                manager.notifyPanelFirstResponder(index: capturedPanel)
            }
            
            metalLog("requestFirstResponder: panel=\(capturedPanel) before=\(String(describing: before)) after=\(String(describing: after))")
        }
    }
    
    // MARK: - Tab Key Cycling
    
    /// Handle Tab key to cycle to next panel.
    /// Returns the new active panel index, or nil if cycling disabled.
    func handleTabKey(manager: FocusManager) -> Int? {
        let newPanel = manager.cycleToNextPanel(panelCount: panels.count)
        if let newPanel = newPanel {
            updateFocusIndicatorsWithManager(manager)
            // Emit focus change to Python
            print("{\"type\":\"panel_focus\",\"panel\":\(newPanel)}")
            fflush(stdout)
        }
        return newPanel
    }
    
    /// Handle Shift+Tab to cycle to previous panel.
    func handleShiftTabKey(manager: FocusManager) -> Int? {
        let newPanel = manager.cycleToPreviousPanel(panelCount: panels.count)
        if let newPanel = newPanel {
            updateFocusIndicatorsWithManager(manager)
            // Emit focus change to Python
            print("{\"type\":\"panel_focus\",\"panel\":\(newPanel)}")
            fflush(stdout)
        }
        return newPanel
    }
    
    // MARK: - Window Activation Restore
    
    /// Called when app/window becomes active after Cmd+Tab.
    /// Restores focus to the previously active panel.
    func restoreFocusOnActivate(manager: FocusManager) {
        let activeIdx = manager.getActivePanel()
        guard activeIdx >= 0, activeIdx < panels.count else { return }
        
        metalLog("restoreFocusOnActivate: restoring to panel \(activeIdx)")
        
        // Update JS focus indicators
        for (i, wv) in panels.enumerated() {
            let isFocused = (i == activeIdx)
            wv.evaluateJavaScript("setFocused(\(isFocused ? "true" : "false"))", completionHandler: nil)
        }
        
        // Make the panel first responder
        requestFirstResponderForPanel(activeIdx, manager: manager)
        
        // If fullscreen game is active, also restore game focus
        if fullscreenIframeActive {
            ensureWebViewFirstResponder()
            restoreGameFocus()
        }
    }
    
    // MARK: - Panel Click Focus
    
    /// Handle click on a panel to focus it.
    func handlePanelClick(_ index: Int, manager: FocusManager) {
        guard index >= 0, index < panels.count else { return }
        guard manager.getActivePanel() != index else { return }
        
        metalLog("handlePanelClick: focusing panel \(index)")
        manager.setFocus(to: index)
        updateFocusIndicatorsWithManager(manager)
        
        // Emit focus change to Python
        print("{\"type\":\"panel_focus\",\"panel\":\(index)}")
        fflush(stdout)
    }
    
    // MARK: - Panel Registration Helpers
    
    /// Register all current panels with the FocusManager.
    func registerPanelsWithManager(_ manager: FocusManager) {
        manager.clearAllPanels()
        for i in 0..<panels.count {
            manager.registerPanel(index: i)
        }
    }
    
    /// Register a new panel with the FocusManager.
    func registerNewPanel(_ index: Int, manager: FocusManager) {
        manager.registerPanel(index: index)
    }
    
    /// Unregister a panel from the FocusManager.
    func unregisterPanel(_ index: Int, manager: FocusManager) {
        manager.unregisterPanel(index: index)
    }
    
    // MARK: - Visual Focus Indicator
    
    /// Check if focus indicator should be shown.
    /// Can be configured via FocusConfig.
    func shouldShowFocusIndicator(_ manager: FocusManager) -> Bool {
        // This would read from FocusConfig if we expose it
        return true
    }
    
    /// Get the CSS border color for a panel based on focus state.
    static func focusBorderColor(isFocused: Bool) -> String {
        return isFocused
            ? "rgba(0,212,255,0.5)"   // Brighter cyan for focused
            : "rgba(0,212,255,0.12)"  // Dim for unfocused
    }
    
    /// Get the CSS box shadow for a panel based on focus state.
    static func focusBoxShadow(isFocused: Bool) -> String {
        return isFocused
            ? "inset 0 0 12px rgba(0,212,255,0.15)"  // Glow for focused
            : "none"                                  // No glow for unfocused
    }
}
