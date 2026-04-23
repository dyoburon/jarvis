/**
 * FocusManager.swift
 *
 * Centralized focus management for ChatWebView panels.
 * Handles Tab cycling, Cmd+Tab restore, and deterministic focus tracking.
 *
 * @module Focus/FocusManager
 */

import AppKit
import Foundation

// =============================================================================
// TYPES
// =============================================================================

/// Focus state for a single panel
struct PanelFocusState {
    let index: Int
    var isFirstResponder: Bool = false
    var lastFocusTime: Date = .distantPast
}

/// Focus manager configuration
struct FocusConfig {
    var tabCyclingEnabled: Bool = true
    var restoreOnActivate: Bool = true
    var focusIndicatorEnabled: Bool = true
    var debugLogging: Bool = false
}

// =============================================================================
// FOCUS MANAGER
// =============================================================================

/// Manages focus state across all ChatWebView panels.
/// Provides deterministic focus tracking and Tab key cycling.
class FocusManager: NSObject {
    
    // MARK: - Properties
    
    private(set) var activePanel: Int = 0
    private var panelStates: [Int: PanelFocusState] = [:]
    private var config: FocusConfig
    private weak var window: NSWindow?
    private var lastActivateTime: Date = .distantPast
    
    /// Callback when focus changes: (fromPanel, toPanel)
    var onFocusChange: ((Int, Int) -> Void)?
    
    /// Callback to emit focus state to Python
    var onEmitFocus: ((Int) -> Void)?
    
    // MARK: - Initialization
    
    init(config: FocusConfig = FocusConfig()) {
        self.config = config
        super.init()
    }
    
    // MARK: - Configuration
    
    func updateConfig(_ newConfig: FocusConfig) {
        self.config = newConfig
    }
    
    func setWindow(_ window: NSWindow) {
        self.window = window
    }
    
    // MARK: - Panel Registration
    
    /// Register a panel with the focus manager
    func registerPanel(index: Int) {
        panelStates[index] = PanelFocusState(index: index)
        if config.debugLogging {
            metalLog("FocusManager: registered panel \(index)")
        }
    }
    
    /// Unregister a panel (when closed)
    func unregisterPanel(index: Int) {
        panelStates.removeValue(forKey: index)
        // Adjust active panel if needed
        if activePanel == index && panelStates.count > 0 {
            activePanel = panelStates.keys.sorted().first ?? 0
        }
        if config.debugLogging {
            metalLog("FocusManager: unregistered panel \(index), active now \(activePanel)")
        }
    }
    
    /// Clear all panels
    func clearAllPanels() {
        panelStates.removeAll()
        activePanel = 0
        if config.debugLogging {
            metalLog("FocusManager: cleared all panels")
        }
    }
    
    // MARK: - Focus Control
    
    /// Set the active panel and update focus state
    @discardableResult
    func setFocus(to newPanel: Int) -> Bool {
        guard panelStates[newPanel] != nil else {
            if config.debugLogging {
                metalLog("FocusManager: setFocus rejected — panel \(newPanel) not registered")
            }
            return false
        }
        
        let oldPanel = activePanel
        guard oldPanel != newPanel else { return true }
        
        activePanel = newPanel
        panelStates[newPanel]?.isFirstResponder = true
        panelStates[newPanel]?.lastFocusTime = Date()
        
        // Mark old panel as not first responder
        if oldPanel != newPanel {
            panelStates[oldPanel]?.isFirstResponder = false
        }
        
        if config.debugLogging {
            metalLog("FocusManager: focus changed \(oldPanel) → \(newPanel)")
        }
        
        // Notify listeners
        onFocusChange?(oldPanel, newPanel)
        onEmitFocus?(newPanel)
        
        return true
    }
    
    /// Cycle to next panel (Tab key behavior)
    func cycleToNextPanel(panelCount: Int) -> Int? {
        guard config.tabCyclingEnabled, panelCount > 1 else { return nil }
        
        let nextPanel = (activePanel + 1) % panelCount
        if setFocus(to: nextPanel) {
            return nextPanel
        }
        return nil
    }
    
    /// Cycle to previous panel (Shift+Tab behavior)
    func cycleToPreviousPanel(panelCount: Int) -> Int? {
        guard config.tabCyclingEnabled, panelCount > 1 else { return nil }
        
        let prevPanel = (activePanel - 1 + panelCount) % panelCount
        if setFocus(to: prevPanel) {
            return prevPanel
        }
        return nil
    }
    
    // MARK: - Window Activation Handling
    
    /// Called when the app/window becomes active (e.g., after Cmd+Tab back)
    func handleWindowActivate() {
        guard config.restoreOnActivate else { return }
        
        let now = Date()
        // Debounce: only restore if at least 0.1s since last activate
        guard now.timeIntervalSince(lastActivateTime) > 0.1 else { return }
        lastActivateTime = now
        
        if config.debugLogging {
            metalLog("FocusManager: window activated, restoring focus to panel \(activePanel)")
        }
        
        // Request focus restoration for the active panel
        onFocusChange?(activePanel, activePanel)
        onEmitFocus?(activePanel)
    }
    
    /// Called when a panel's WKWebView becomes first responder
    func notifyPanelFirstResponder(index: Int) {
        guard panelStates[index] != nil else { return }
        
        if activePanel != index {
            let oldPanel = activePanel
            activePanel = index
            panelStates[index]?.isFirstResponder = true
            panelStates[index]?.lastFocusTime = Date()
            
            if oldPanel >= 0, panelStates[oldPanel] != nil {
                panelStates[oldPanel]?.isFirstResponder = false
            }
            
            if config.debugLogging {
                metalLog("FocusManager: panel \(index) became first responder (was \(oldPanel))")
            }
        }
    }
    
    // MARK: - State Queries
    
    /// Check if a panel is currently active
    func isPanelActive(_ index: Int) -> Bool {
        return activePanel == index
    }
    
    /// Check if a panel is first responder
    func isPanelFirstResponder(_ index: Int) -> Bool {
        return panelStates[index]?.isFirstResponder ?? false
    }
    
    /// Get the last focus time for a panel
    func lastFocusTime(for index: Int) -> Date? {
        return panelStates[index]?.lastFocusTime
    }
    
    /// Get current active panel index
    func getActivePanel() -> Int {
        return activePanel
    }
    
    /// Get total registered panel count
    func getPanelCount() -> Int {
        return panelStates.count
    }
    
    // MARK: - Focus Emission to Python
    
    /// Emit current focus state to Python via stdout
    func emitFocusState() {
        onEmitFocus?(activePanel)
        
        // Also print to stdout for Python to read
        let json = "{\"type\":\"panel_focus\",\"panel\":\(activePanel)}"
        print(json)
        fflush(stdout)
    }
}
