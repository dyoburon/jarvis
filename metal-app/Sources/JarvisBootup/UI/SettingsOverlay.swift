import AppKit
import WebKit

/// Fullscreen settings modal overlay for Jarvis.
///
/// Categories (matching the plan):
/// - Appearance: Theme, colors, fonts, opacity, layout
/// - Visualizer: Type, position, scale, reactivity
/// - Background: Mode, image/video path, blur
/// - Startup: Boot animation, fast start, on_ready
/// - Voice: Enable/disable, PTT/VAD, device
/// - Keybinds: All keyboard shortcuts
/// - Panels: History, persistence, focus
/// - Games: Enable/disable, custom paths
/// - About: Version, updates, logs
final class SettingsOverlay: NSView {
    
    // MARK: - Properties
    
    private let gearButton: NSButton
    private var overlayView: NSView?
    private var settingsWebView: WKWebView?
    private var isVisible = false
    
    /// Callback when settings are saved
    var onSettingsSaved: (() -> Void)?
    
    // MARK: - Init
    
    override init(frame frameRect: NSRect) {
        // Create gear button (settings icon)
        gearButton = NSButton(frame: NSRect(x: 0, y: 0, width: 36, height: 36))
        gearButton.bezelStyle = .regularSquare
        gearButton.image = NSImage(systemSymbolName: "gearshape.fill", accessibilityDescription: "Settings")
        gearButton.imageScaling = .scaleProportionallyDown
        gearButton.isBordered = false
        gearButton.toolTip = "Settings (Cmd+,)"
        
        super.init(frame: frameRect)
        
        // Style gear button
        gearButton.contentTintColor = NSColor(red: 0, green: 0.83, blue: 1, alpha: 0.7)
        gearButton.target = self
        gearButton.action = #selector(toggleSettings)
        
        addSubview(gearButton)
        
        // Create fullscreen overlay (hidden)
        createOverlay()
    }
    
    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }
    
    // MARK: - Layout
    
    override func layout() {
        super.layout()
        
        // Position gear button in top-right corner (with some offset for other UI)
        let margin: CGFloat = 16
        let topOffset: CGFloat = 50 // Offset for any top bar
        gearButton.frame = NSRect(
            x: bounds.width - gearButton.bounds.width - margin,
            y: bounds.height - gearButton.bounds.height - margin - topOffset,
            width: 36,
            height: 36
        )
        
        // Position fullscreen overlay
        overlayView?.frame = bounds
        settingsWebView?.frame = bounds
    }
    
    // MARK: - Toggle
    
    @objc func toggleSettings() {
        if isVisible {
            hideSettings()
        } else {
            showSettings()
        }
    }
    
    private func showSettings() {
        guard let overlay = overlayView else { return }
        isVisible = true
        
        if overlay.superview == nil {
            addSubview(overlay)
        }
        
        overlay.alphaValue = 0
        NSAnimationContext.runAnimationGroup { context in
            context.duration = 0.2
            context.timingFunction = CAMediaTimingFunction(name: .easeOut)
            overlay.animator().alphaValue = 1
        }
        
        loadSettingsContent()
    }
    
    @objc func hideSettings() {
        guard let overlay = overlayView else { return }
        isVisible = false
        
        NSAnimationContext.runAnimationGroup { context in
            context.duration = 0.15
            context.timingFunction = CAMediaTimingFunction(name: .easeIn)
            overlay.animator().alphaValue = 0
        } completionHandler: { [weak self] in
            self?.overlayView?.removeFromSuperview()
        }
    }
    
    // MARK: - Overlay Creation
    
    private func createOverlay() {
        // Fullscreen dark overlay
        let overlay = NSView(frame: bounds)
        overlay.wantsLayer = true
        overlay.layer?.backgroundColor = NSColor.black.withAlphaComponent(0.85).cgColor
        overlay.autoresizingMask = [.width, .height]
        
        // Close button (top-right)
        let closeButton = NSButton(frame: NSRect(x: 0, y: 0, width: 44, height: 44))
        closeButton.bezelStyle = .regularSquare
        closeButton.image = NSImage(systemSymbolName: "xmark.circle.fill", accessibilityDescription: "Close")
        closeButton.imageScaling = .scaleProportionallyDown
        closeButton.isBordered = false
        closeButton.contentTintColor = NSColor(red: 0, green: 0.83, blue: 1, alpha: 0.8)
        closeButton.target = self
        closeButton.action = #selector(hideSettings)
        closeButton.toolTip = "Close (Escape)"
        overlay.addSubview(closeButton)
        
        // Title
        let titleLabel = NSTextField(labelWithString: "SETTINGS")
        titleLabel.font = NSFont.systemFont(ofSize: 24, weight: .bold)
        titleLabel.textColor = NSColor(red: 0, green: 0.83, blue: 1, alpha: 1)
        titleLabel.alignment = .center
        titleLabel.frame = NSRect(x: 0, y: bounds.height - 60, width: bounds.width, height: 36)
        titleLabel.autoresizingMask = [.width, .minYMargin]
        overlay.addSubview(titleLabel)
        
        // WKWebView for settings content
        let config = WKWebViewConfiguration()
        config.userContentController = WKUserContentController()
        config.userContentController.add(self, name: "jarvisSettings")
        
        let webView = WKWebView(frame: NSRect(x: 100, y: 60, width: bounds.width - 200, height: bounds.height - 140), configuration: config)
        webView.navigationDelegate = self
        webView.autoresizingMask = [.width, .height]
        webView.setValue(false, forKey: "drawsBackground")
        overlay.addSubview(webView)
        self.settingsWebView = webView
        
        // Position close button
        closeButton.frame.origin = NSPoint(x: bounds.width - 60, y: bounds.height - 55)
        closeButton.autoresizingMask = [.minXMargin, .minYMargin]
        
        self.overlayView = overlay
    }
    
    // MARK: - Content Loading
    
    private func loadSettingsContent() {
        guard let webView = settingsWebView else { return }
        let html = generateSettingsHTML()
        webView.loadHTMLString(html, baseURL: nil)
    }
    
    private func generateSettingsHTML() -> String {
        let config = ConfigManager.shared
        let themes = ThemeManager.shared.availableThemes()
        let themeOptions = themes.map { "<option value=\"\($0.name)\">\($0.displayName)</option>" }.joined(separator: "\n")
        
        return """
        <!DOCTYPE html>
        <html>
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <style>
                * { box-sizing: border-box; margin: 0; padding: 0; }
                
                body {
                    font-family: 'SF Pro Text', -apple-system, BlinkMacSystemFont, sans-serif;
                    font-size: 13px;
                    color: #e0e0e0;
                    background: transparent;
                    padding: 0 20px;
                    overflow-y: auto;
                    height: 100%;
                }
                
                /* Two-column layout */
                .container {
                    display: flex;
                    gap: 40px;
                    height: 100%;
                }
                
                .sidebar {
                    width: 200px;
                    flex-shrink: 0;
                    position: sticky;
                    top: 0;
                }
                
                .content {
                    flex: 1;
                    min-width: 0;
                }
                
                /* Sidebar navigation */
                .nav-item {
                    padding: 10px 16px;
                    margin: 4px 0;
                    border-radius: 6px;
                    cursor: pointer;
                    color: #888;
                    font-size: 14px;
                    transition: all 0.15s;
                }
                
                .nav-item:hover {
                    background: rgba(0, 212, 255, 0.1);
                    color: #ccc;
                }
                
                .nav-item.active {
                    background: rgba(0, 212, 255, 0.2);
                    color: #00d4ff;
                }
                
                /* Section styling */
                .section {
                    display: none;
                    padding-bottom: 40px;
                }
                
                .section.active {
                    display: block;
                }
                
                .section-title {
                    font-size: 20px;
                    font-weight: 600;
                    color: #00d4ff;
                    margin-bottom: 20px;
                    padding-bottom: 10px;
                    border-bottom: 1px solid rgba(0, 212, 255, 0.2);
                }
                
                .group {
                    background: rgba(255, 255, 255, 0.03);
                    border: 1px solid rgba(255, 255, 255, 0.08);
                    border-radius: 8px;
                    padding: 16px;
                    margin-bottom: 16px;
                }
                
                .group-title {
                    font-size: 12px;
                    font-weight: 600;
                    color: #888;
                    text-transform: uppercase;
                    letter-spacing: 0.5px;
                    margin-bottom: 12px;
                }
                
                .row {
                    display: flex;
                    justify-content: space-between;
                    align-items: center;
                    padding: 8px 0;
                    border-bottom: 1px solid rgba(255, 255, 255, 0.05);
                }
                
                .row:last-child {
                    border-bottom: none;
                }
                
                .label {
                    color: #aaa;
                    font-size: 13px;
                }
                
                .value {
                    color: #e0e0e0;
                }
                
                /* Form controls */
                select, input[type="text"], input[type="number"] {
                    background: rgba(0, 0, 0, 0.4);
                    border: 1px solid rgba(0, 212, 255, 0.3);
                    border-radius: 6px;
                    padding: 6px 12px;
                    color: #e0e0e0;
                    font-size: 13px;
                    min-width: 150px;
                    outline: none;
                    transition: border-color 0.15s;
                }
                
                select:focus, input:focus {
                    border-color: #00d4ff;
                }
                
                select option {
                    background: #1a1a1a;
                    color: #e0e0e0;
                }
                
                input[type="checkbox"] {
                    width: 18px;
                    height: 18px;
                    accent-color: #00d4ff;
                    cursor: pointer;
                }
                
                input[type="range"] {
                    width: 150px;
                    accent-color: #00d4ff;
                }
                
                input[type="color"] {
                    width: 40px;
                    height: 28px;
                    border: 1px solid rgba(0, 212, 255, 0.3);
                    border-radius: 4px;
                    cursor: pointer;
                    background: transparent;
                }
                
                /* Toggle switch */
                .toggle {
                    position: relative;
                    width: 44px;
                    height: 24px;
                    cursor: pointer;
                }
                
                .toggle input {
                    opacity: 0;
                    width: 0;
                    height: 0;
                }
                
                .toggle-slider {
                    position: absolute;
                    top: 0;
                    left: 0;
                    right: 0;
                    bottom: 0;
                    background: rgba(255, 255, 255, 0.1);
                    border-radius: 24px;
                    transition: 0.2s;
                }
                
                .toggle-slider:before {
                    position: absolute;
                    content: "";
                    height: 18px;
                    width: 18px;
                    left: 3px;
                    bottom: 3px;
                    background: #888;
                    border-radius: 50%;
                    transition: 0.2s;
                }
                
                .toggle input:checked + .toggle-slider {
                    background: rgba(0, 212, 255, 0.4);
                }
                
                .toggle input:checked + .toggle-slider:before {
                    transform: translateX(20px);
                    background: #00d4ff;
                }
                
                /* Buttons */
                .buttons {
                    display: flex;
                    gap: 12px;
                    margin-top: 30px;
                    padding-top: 20px;
                    border-top: 1px solid rgba(255, 255, 255, 0.1);
                }
                
                button {
                    background: rgba(0, 212, 255, 0.15);
                    border: 1px solid rgba(0, 212, 255, 0.4);
                    border-radius: 6px;
                    padding: 10px 20px;
                    color: #00d4ff;
                    font-size: 13px;
                    font-weight: 500;
                    cursor: pointer;
                    transition: all 0.15s;
                }
                
                button:hover {
                    background: rgba(0, 212, 255, 0.25);
                    border-color: #00d4ff;
                }
                
                button.primary {
                    background: #00d4ff;
                    border-color: #00d4ff;
                    color: #000;
                }
                
                button.primary:hover {
                    background: #00b8e0;
                }
                
                button.danger {
                    background: transparent;
                    border-color: rgba(255, 68, 68, 0.4);
                    color: #ff6b6b;
                }
                
                button.danger:hover {
                    background: rgba(255, 68, 68, 0.15);
                    border-color: #ff6b6b;
                }
                
                /* Keybind recorder */
                .keybind-input {
                    background: rgba(0, 0, 0, 0.4);
                    border: 1px solid rgba(0, 212, 255, 0.3);
                    border-radius: 6px;
                    padding: 6px 12px;
                    color: #00d4ff;
                    font-family: monospace;
                    font-size: 12px;
                    min-width: 120px;
                    text-align: center;
                    cursor: pointer;
                }
                
                .keybind-input.recording {
                    border-color: #ff6b00;
                    animation: pulse 1s infinite;
                }
                
                @keyframes pulse {
                    0%, 100% { opacity: 1; }
                    50% { opacity: 0.6; }
                }
                
                /* About section */
                .about-logo {
                    font-size: 48px;
                    margin-bottom: 16px;
                }
                
                .about-version {
                    font-size: 16px;
                    color: #00d4ff;
                    margin-bottom: 8px;
                }
                
                .about-desc {
                    color: #888;
                    line-height: 1.6;
                }
                
                /* Theme preview */
                .theme-preview {
                    display: flex;
                    gap: 8px;
                    margin-top: 8px;
                }
                
                .theme-swatch {
                    width: 24px;
                    height: 24px;
                    border-radius: 4px;
                    border: 1px solid rgba(255, 255, 255, 0.2);
                }
            </style>
        </head>
        <body>
            <div class="container">
                <!-- Sidebar -->
                <div class="sidebar">
                    <div class="nav-item active" onclick="showSection('appearance')">🎨 Appearance</div>
                    <div class="nav-item" onclick="showSection('visualizer')">🔮 Visualizer</div>
                    <div class="nav-item" onclick="showSection('background')">🖼️ Background</div>
                    <div class="nav-item" onclick="showSection('startup')">🚀 Startup</div>
                    <div class="nav-item" onclick="showSection('voice')">🎤 Voice</div>
                    <div class="nav-item" onclick="showSection('keybinds')">⌨️ Keybinds</div>
                    <div class="nav-item" onclick="showSection('panels')">📋 Panels</div>
                    <div class="nav-item" onclick="showSection('games')">🎮 Games</div>
                    <div class="nav-item" onclick="showSection('about')">ℹ️ About</div>
                </div>
                
                <!-- Content -->
                <div class="content">
                    <!-- Appearance Section -->
                    <div id="section-appearance" class="section active">
                        <h2 class="section-title">Appearance</h2>
                        
                        <div class="group">
                            <div class="group-title">Theme</div>
                            <div class="row">
                                <span class="label">Theme</span>
                                <select id="theme" onchange="updateTheme()">
                                    \(themeOptions)
                                </select>
                            </div>
                        </div>
                        
                        <div class="group">
                            <div class="group-title">Colors</div>
                            <div class="row">
                                <span class="label">Primary Color</span>
                                <input type="color" id="color_primary" value="#00d4ff">
                            </div>
                            <div class="row">
                                <span class="label">Background</span>
                                <input type="color" id="color_background" value="#000000">
                            </div>
                            <div class="row">
                                <span class="label">Text Color</span>
                                <input type="color" id="color_text" value="#f0ece4">
                            </div>
                            <div class="row">
                                <span class="label">Border Color</span>
                                <input type="color" id="color_border" value="#00d4ff">
                            </div>
                        </div>
                        
                        <div class="group">
                            <div class="group-title">Typography</div>
                            <div class="row">
                                <span class="label">Font Family</span>
                                <select id="font_family">
                                    <option value="Menlo">Menlo</option>
                                    <option value="Monaco">Monaco</option>
                                    <option value="SF Mono">SF Mono</option>
                                    <option value="Fira Code">Fira Code</option>
                                    <option value="JetBrains Mono">JetBrains Mono</option>
                                </select>
                            </div>
                            <div class="row">
                                <span class="label">Font Size</span>
                                <input type="range" id="font_size" min="10" max="20" value="13" oninput="updateFontPreview()">
                                <span id="font_size_value" class="value">13px</span>
                            </div>
                            <div class="row">
                                <span class="label">Line Height</span>
                                <input type="range" id="line_height" min="1.2" max="2.0" step="0.1" value="1.6">
                                <span id="line_height_value" class="value">1.6</span>
                            </div>
                        </div>
                        
                        <div class="group">
                            <div class="group-title">Opacity</div>
                            <div class="row">
                                <span class="label">Panel Background</span>
                                <input type="range" id="opacity_panel" min="0.5" max="1.0" step="0.01" value="0.93">
                                <span id="opacity_panel_value" class="value">93%</span>
                            </div>
                            <div class="row">
                                <span class="label">Background</span>
                                <input type="range" id="opacity_background" min="0" max="1.0" step="0.01" value="1.0">
                                <span id="opacity_background_value" class="value">100%</span>
                            </div>
                        </div>
                        
                        <div class="buttons">
                            <button class="primary" onclick="saveSettings()">Save Changes</button>
                            <button onclick="resetSection('appearance')">Reset to Defaults</button>
                        </div>
                    </div>
                    
                    <!-- Visualizer Section -->
                    <div id="section-visualizer" class="section">
                        <h2 class="section-title">Visualizer</h2>
                        
                        <div class="group">
                            <div class="group-title">Type</div>
                            <div class="row">
                                <span class="label">Visualizer Type</span>
                                <select id="visualizer_type">
                                    <option value="orb">Orb (3D Sphere)</option>
                                    <option value="particle">Particles</option>
                                    <option value="waveform">Waveform</option>
                                    <option value="image">Image</option>
                                    <option value="video">Video</option>
                                    <option value="none">None (Disabled)</option>
                                </select>
                            </div>
                            <div class="row">
                                <span class="label">Enabled</span>
                                <label class="toggle">
                                    <input type="checkbox" id="visualizer_enabled" checked>
                                    <span class="toggle-slider"></span>
                                </label>
                            </div>
                        </div>
                        
                        <div class="group">
                            <div class="group-title">Position & Size</div>
                            <div class="row">
                                <span class="label">Scale</span>
                                <input type="range" id="visualizer_scale" min="0.1" max="2.0" step="0.1" value="1.0">
                                <span id="visualizer_scale_value" class="value">1.0x</span>
                            </div>
                            <div class="row">
                                <span class="label">Horizontal Offset</span>
                                <input type="range" id="visualizer_pos_x" min="-1" max="1" step="0.05" value="0">
                                <span id="visualizer_pos_x_value" class="value">0</span>
                            </div>
                            <div class="row">
                                <span class="label">Vertical Offset</span>
                                <input type="range" id="visualizer_pos_y" min="-1" max="1" step="0.05" value="0">
                                <span id="visualizer_pos_y_value" class="value">0</span>
                            </div>
                        </div>
                        
                        <div class="group">
                            <div class="group-title">Reactivity</div>
                            <div class="row">
                                <span class="label">React to Audio</span>
                                <label class="toggle">
                                    <input type="checkbox" id="visualizer_audio_react" checked>
                                    <span class="toggle-slider"></span>
                                </label>
                            </div>
                            <div class="row">
                                <span class="label">React to State</span>
                                <label class="toggle">
                                    <input type="checkbox" id="visualizer_state_react" checked>
                                    <span class="toggle-slider"></span>
                                </label>
                            </div>
                            <div class="row">
                                <span class="label">Bloom Intensity</span>
                                <input type="range" id="visualizer_bloom" min="0" max="2.0" step="0.1" value="1.0">
                                <span id="visualizer_bloom_value" class="value">1.0</span>
                            </div>
                        </div>
                        
                        <div class="buttons">
                            <button class="primary" onclick="saveSettings()">Save Changes</button>
                            <button onclick="resetSection('visualizer')">Reset to Defaults</button>
                        </div>
                    </div>
                    
                    <!-- Background Section -->
                    <div id="section-background" class="section">
                        <h2 class="section-title">Background</h2>
                        
                        <div class="group">
                            <div class="group-title">Mode</div>
                            <div class="row">
                                <span class="label">Background Type</span>
                                <select id="background_mode">
                                    <option value="hex_grid">Hex Grid (Default)</option>
                                    <option value="solid">Solid Color</option>
                                    <option value="gradient">Gradient</option>
                                    <option value="image">Image</option>
                                    <option value="video">Video</option>
                                    <option value="none">None (Transparent)</option>
                                </select>
                            </div>
                        </div>
                        
                        <div class="group">
                            <div class="group-title">Hex Grid Settings</div>
                            <div class="row">
                                <span class="label">Grid Color</span>
                                <input type="color" id="hexgrid_color" value="#00d4ff">
                            </div>
                            <div class="row">
                                <span class="label">Grid Opacity</span>
                                <input type="range" id="hexgrid_opacity" min="0" max="0.3" step="0.01" value="0.08">
                                <span id="hexgrid_opacity_value" class="value">8%</span>
                            </div>
                            <div class="row">
                                <span class="label">Animation Speed</span>
                                <input type="range" id="hexgrid_speed" min="0" max="3.0" step="0.1" value="1.0">
                                <span id="hexgrid_speed_value" class="value">1.0x</span>
                            </div>
                            <div class="row">
                                <span class="label">Glow Intensity</span>
                                <input type="range" id="hexgrid_glow" min="0" max="1.0" step="0.1" value="0.5">
                                <span id="hexgrid_glow_value" class="value">0.5</span>
                            </div>
                        </div>
                        
                        <div class="group">
                            <div class="group-title">Custom Background</div>
                            <div class="row">
                                <span class="label">Image/Video Path</span>
                                <input type="text" id="background_path" placeholder="/path/to/file.jpg" style="width: 250px;">
                            </div>
                            <div class="row">
                                <span class="label">Blur Amount</span>
                                <input type="range" id="background_blur" min="0" max="30" step="1" value="0">
                                <span id="background_blur_value" class="value">0px</span>
                            </div>
                        </div>
                        
                        <div class="buttons">
                            <button class="primary" onclick="saveSettings()">Save Changes</button>
                            <button onclick="resetSection('background')">Reset to Defaults</button>
                        </div>
                    </div>
                    
                    <!-- Startup Section -->
                    <div id="section-startup" class="section">
                        <h2 class="section-title">Startup</h2>
                        
                        <div class="group">
                            <div class="group-title">Boot Animation</div>
                            <div class="row">
                                <span class="label">Play Boot Animation</span>
                                <label class="toggle">
                                    <input type="checkbox" id="boot_enabled" checked>
                                    <span class="toggle-slider"></span>
                                </label>
                            </div>
                            <div class="row">
                                <span class="label">Boot Duration</span>
                                <input type="number" id="boot_duration" value="27" min="0" max="60" step="1">
                                <span class="value">seconds</span>
                            </div>
                            <div class="row">
                                <span class="label">Skip on Key Press</span>
                                <label class="toggle">
                                    <input type="checkbox" id="boot_skip_key" checked>
                                    <span class="toggle-slider"></span>
                                </label>
                            </div>
                            <div class="row">
                                <span class="label">Play Boot Music</span>
                                <label class="toggle">
                                    <input type="checkbox" id="boot_music" checked>
                                    <span class="toggle-slider"></span>
                                </label>
                            </div>
                        </div>
                        
                        <div class="group">
                            <div class="group-title">Fast Start</div>
                            <div class="row">
                                <span class="label">Enable Fast Start</span>
                                <label class="toggle">
                                    <input type="checkbox" id="fast_start">
                                    <span class="toggle-slider"></span>
                                </label>
                            </div>
                            <div class="row">
                                <span class="label">Fast Start Delay</span>
                                <input type="number" id="fast_start_delay" value="0.5" min="0" max="5" step="0.1">
                                <span class="value">seconds</span>
                            </div>
                        </div>
                        
                        <div class="group">
                            <div class="group-title">On Ready Action</div>
                            <div class="row">
                                <span class="label">Default Action</span>
                                <select id="on_ready_action">
                                    <option value="listening">Listening Mode</option>
                                    <option value="panels">Open Panels</option>
                                    <option value="chat">Launch Livechat</option>
                                    <option value="game">Launch Game</option>
                                    <option value="skill">Activate Skill</option>
                                </select>
                            </div>
                            <div class="row">
                                <span class="label">Default Panel Count</span>
                                <input type="number" id="default_panels" value="1" min="1" max="5">
                            </div>
                        </div>
                        
                        <div class="buttons">
                            <button class="primary" onclick="saveSettings()">Save Changes</button>
                            <button onclick="resetSection('startup')">Reset to Defaults</button>
                        </div>
                    </div>
                    
                    <!-- Voice Section -->
                    <div id="section-voice" class="section">
                        <h2 class="section-title">Voice</h2>
                        
                        <div class="group">
                            <div class="group-title">Voice Settings</div>
                            <div class="row">
                                <span class="label">Voice Enabled</span>
                                <label class="toggle">
                                    <input type="checkbox" id="voice_enabled" checked>
                                    <span class="toggle-slider"></span>
                                </label>
                            </div>
                            <div class="row">
                                <span class="label">Input Mode</span>
                                <select id="voice_mode">
                                    <option value="ptt">Push to Talk (PTT)</option>
                                    <option value="vad">Voice Activity Detection (VAD)</option>
                                </select>
                            </div>
                        </div>
                        
                        <div class="group">
                            <div class="group-title">Audio Device</div>
                            <div class="row">
                                <span class="label">Input Device</span>
                                <select id="voice_device">
                                    <option value="default">Default Microphone</option>
                                    <!-- Will be populated by Swift -->
                                </select>
                            </div>
                            <div class="row">
                                <span class="label">Sample Rate</span>
                                <select id="voice_sample_rate">
                                    <option value="16000">16 kHz</option>
                                    <option value="24000">24 kHz</option>
                                    <option value="48000">48 kHz</option>
                                </select>
                            </div>
                        </div>
                        
                        <div class="group">
                            <div class="group-title">Feedback</div>
                            <div class="row">
                                <span class="label">Sound Effects</span>
                                <label class="toggle">
                                    <input type="checkbox" id="voice_sounds" checked>
                                    <span class="toggle-slider"></span>
                                </label>
                            </div>
                            <div class="row">
                                <span class="label">Sound Volume</span>
                                <input type="range" id="voice_volume" min="0" max="1.0" step="0.1" value="0.5">
                                <span id="voice_volume_value" class="value">50%</span>
                            </div>
                        </div>
                        
                        <div class="buttons">
                            <button class="primary" onclick="saveSettings()">Save Changes</button>
                            <button onclick="resetSection('voice')">Reset to Defaults</button>
                        </div>
                    </div>
                    
                    <!-- Keybinds Section -->
                    <div id="section-keybinds" class="section">
                        <h2 class="section-title">Keybinds</h2>
                        
                        <div class="group">
                            <div class="group-title">Global Shortcuts</div>
                            <div class="row">
                                <span class="label">Push to Talk</span>
                                <div class="keybind-input" id="kb_ptt" onclick="recordKeybind('push_to_talk')">Option+Period</div>
                            </div>
                            <div class="row">
                                <span class="label">Open Assistant</span>
                                <div class="keybind-input" id="kb_assistant" onclick="recordKeybind('open_assistant')">Cmd+G</div>
                            </div>
                            <div class="row">
                                <span class="label">New Panel</span>
                                <div class="keybind-input" id="kb_new_panel" onclick="recordKeybind('new_panel')">Cmd+T</div>
                            </div>
                            <div class="row">
                                <span class="label">Close Panel</span>
                                <div class="keybind-input" id="kb_close_panel" onclick="recordKeybind('close_panel')">Escape+Escape</div>
                            </div>
                            <div class="row">
                                <span class="label">Toggle Fullscreen</span>
                                <div class="keybind-input" id="kb_fullscreen" onclick="recordKeybind('toggle_fullscreen')">Cmd+F</div>
                            </div>
                            <div class="row">
                                <span class="label">Open Settings</span>
                                <div class="keybind-input" id="kb_settings" onclick="recordKeybind('open_settings')">Cmd+,</div>
                            </div>
                        </div>
                        
                        <div class="group">
                            <div class="group-title">Panel Focus</div>
                            <div class="row">
                                <span class="label">Focus Panel 1</span>
                                <div class="keybind-input" id="kb_panel1" onclick="recordKeybind('focus_panel_1')">Cmd+1</div>
                            </div>
                            <div class="row">
                                <span class="label">Focus Panel 2</span>
                                <div class="keybind-input" id="kb_panel2" onclick="recordKeybind('focus_panel_2')">Cmd+2</div>
                            </div>
                            <div class="row">
                                <span class="label">Focus Panel 3</span>
                                <div class="keybind-input" id="kb_panel3" onclick="recordKeybind('focus_panel_3')">Cmd+3</div>
                            </div>
                            <div class="row">
                                <span class="label">Cycle Panels</span>
                                <div class="keybind-input" id="kb_cycle" onclick="recordKeybind('cycle_panels')">Tab</div>
                            </div>
                        </div>
                        
                        <div class="buttons">
                            <button class="primary" onclick="saveSettings()">Save Changes</button>
                            <button onclick="resetSection('keybinds')">Reset to Defaults</button>
                        </div>
                    </div>
                    
                    <!-- Panels Section -->
                    <div id="section-panels" class="section">
                        <h2 class="section-title">Panels</h2>
                        
                        <div class="group">
                            <div class="group-title">History</div>
                            <div class="row">
                                <span class="label">Save Chat History</span>
                                <label class="toggle">
                                    <input type="checkbox" id="history_enabled" checked>
                                    <span class="toggle-slider"></span>
                                </label>
                            </div>
                            <div class="row">
                                <span class="label">Max Messages per Panel</span>
                                <input type="number" id="history_max" value="1000" min="100" max="10000" step="100">
                            </div>
                            <div class="row">
                                <span class="label">Restore on Launch</span>
                                <label class="toggle">
                                    <input type="checkbox" id="history_restore" checked>
                                    <span class="toggle-slider"></span>
                                </label>
                            </div>
                        </div>
                        
                        <div class="group">
                            <div class="group-title">Layout</div>
                            <div class="row">
                                <span class="label">Max Panels</span>
                                <input type="number" id="max_panels" value="5" min="1" max="10">
                            </div>
                            <div class="row">
                                <span class="label">Default Panel Width</span>
                                <input type="range" id="panel_width" min="0.4" max="0.9" step="0.05" value="0.72">
                                <span id="panel_width_value" class="value">72%</span>
                            </div>
                            <div class="row">
                                <span class="label">Panel Gap</span>
                                <input type="number" id="panel_gap" value="2" min="0" max="20">
                                <span class="value">px</span>
                            </div>
                        </div>
                        
                        <div class="group">
                            <div class="group-title">Focus Behavior</div>
                            <div class="row">
                                <span class="label">Tab Cycling</span>
                                <label class="toggle">
                                    <input type="checkbox" id="tab_cycling" checked>
                                    <span class="toggle-slider"></span>
                                </label>
                            </div>
                            <div class="row">
                                <span class="label">Restore on Activate</span>
                                <label class="toggle">
                                    <input type="checkbox" id="focus_restore" checked>
                                    <span class="toggle-slider"></span>
                                </label>
                            </div>
                        </div>
                        
                        <div class="buttons">
                            <button class="primary" onclick="saveSettings()">Save Changes</button>
                            <button class="danger" onclick="clearHistory()">Clear All History</button>
                        </div>
                    </div>
                    
                    <!-- Games Section -->
                    <div id="section-games" class="section">
                        <h2 class="section-title">Games</h2>
                        
                        <div class="group">
                            <div class="group-title">Enabled Games</div>
                            <div class="row">
                                <span class="label">Wordle</span>
                                <label class="toggle"><input type="checkbox" checked><span class="toggle-slider"></span></label>
                            </div>
                            <div class="row">
                                <span class="label">Connections</span>
                                <label class="toggle"><input type="checkbox" checked><span class="toggle-slider"></span></label>
                            </div>
                            <div class="row">
                                <span class="label">Asteroids</span>
                                <label class="toggle"><input type="checkbox" checked><span class="toggle-slider"></span></label>
                            </div>
                            <div class="row">
                                <span class="label">Tetris</span>
                                <label class="toggle"><input type="checkbox" checked><span class="toggle-slider"></span></label>
                            </div>
                            <div class="row">
                                <span class="label">Pinball</span>
                                <label class="toggle"><input type="checkbox" checked><span class="toggle-slider"></span></label>
                            </div>
                            <div class="row">
                                <span class="label">Doodle Jump</span>
                                <label class="toggle"><input type="checkbox" checked><span class="toggle-slider"></span></label>
                            </div>
                            <div class="row">
                                <span class="label">Minesweeper</span>
                                <label class="toggle"><input type="checkbox" checked><span class="toggle-slider"></span></label>
                            </div>
                            <div class="row">
                                <span class="label">Subway Surfers</span>
                                <label class="toggle"><input type="checkbox" checked><span class="toggle-slider"></span></label>
                            </div>
                        </div>
                        
                        <div class="group">
                            <div class="group-title">Fullscreen Behavior</div>
                            <div class="row">
                                <span class="label">Keyboard Passthrough</span>
                                <label class="toggle">
                                    <input type="checkbox" id="game_keyboard" checked>
                                    <span class="toggle-slider"></span>
                                </label>
                            </div>
                            <div class="row">
                                <span class="label">Escape to Exit</span>
                                <label class="toggle">
                                    <input type="checkbox" id="game_escape" checked>
                                    <span class="toggle-slider"></span>
                                </label>
                            </div>
                        </div>
                        
                        <div class="buttons">
                            <button class="primary" onclick="saveSettings()">Save Changes</button>
                        </div>
                    </div>
                    
                    <!-- About Section -->
                    <div id="section-about" class="section">
                        <h2 class="section-title">About</h2>
                        
                        <div class="group" style="text-align: center; padding: 40px;">
                            <div class="about-logo">🔮</div>
                            <div class="about-version">Jarvis v1.0.0</div>
                            <div class="about-desc">
                                Personal AI Assistant with Metal-powered 3D visualization.<br>
                                Built with Swift, Python, and Metal.
                            </div>
                        </div>
                        
                        <div class="group">
                            <div class="group-title">Updates</div>
                            <div class="row">
                                <span class="label">Current Version</span>
                                <span class="value">1.0.0</span>
                            </div>
                            <div class="row">
                                <span class="label">Check Automatically</span>
                                <label class="toggle">
                                    <input type="checkbox" id="auto_update" checked>
                                    <span class="toggle-slider"></span>
                                </label>
                            </div>
                            <div class="row">
                                <span class="label">Update Channel</span>
                                <select id="update_channel">
                                    <option value="stable">Stable</option>
                                    <option value="beta">Beta</option>
                                </select>
                            </div>
                            <div class="row">
                                <button onclick="checkUpdates()" style="width: 100%;">Check for Updates</button>
                            </div>
                        </div>
                        
                        <div class="group">
                            <div class="group-title">Debug</div>
                            <div class="row">
                                <span class="label">Show FPS Counter</span>
                                <label class="toggle">
                                    <input type="checkbox" id="debug_fps">
                                    <span class="toggle-slider"></span>
                                </label>
                            </div>
                            <div class="row">
                                <span class="label">Log Level</span>
                                <select id="log_level">
                                    <option value="DEBUG">Debug</option>
                                    <option value="INFO" selected>Info</option>
                                    <option value="WARNING">Warning</option>
                                    <option value="ERROR">Error</option>
                                </select>
                            </div>
                            <div class="row">
                                <button class="danger" onclick="openLogs()" style="width: 100%;">Open Log Files</button>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
            
            <script>
                // Section navigation
                function showSection(name) {
                    document.querySelectorAll('.section').forEach(s => s.classList.remove('active'));
                    document.querySelectorAll('.nav-item').forEach(n => n.classList.remove('active'));
                    document.getElementById('section-' + name).classList.add('active');
                    event.target.classList.add('active');
                }
                
                // Range input previews
                document.querySelectorAll('input[type="range"]').forEach(input => {
                    input.addEventListener('input', function() {
                        const valueSpan = document.getElementById(this.id + '_value');
                        if (valueSpan) {
                            if (this.id.includes('opacity') || this.id.includes('volume')) {
                                valueSpan.textContent = Math.round(this.value * 100) + '%';
                            } else if (this.id.includes('size')) {
                                valueSpan.textContent = this.value + 'px';
                            } else if (this.id.includes('width') && this.id.includes('panel')) {
                                valueSpan.textContent = Math.round(this.value * 100) + '%';
                            } else {
                                valueSpan.textContent = this.value;
                            }
                        }
                    });
                });
                
                // Theme preview update
                function updateTheme() {
                    // Could show preview of theme colors
                }
                
                // Font preview
                function updateFontPreview() {
                    document.getElementById('font_size_value').textContent = 
                        document.getElementById('font_size').value + 'px';
                }
                
                // Keybind recording
                let recordingKeybind = null;
                
                function recordKeybind(action) {
                    if (recordingKeybind) {
                        document.getElementById('kb_' + recordingKeybind.replace('focus_panel_', 'panel').replace('cycle_panels', 'cycle')).classList.remove('recording');
                    }
                    recordingKeybind = action;
                    const elementId = action
                        .replace('push_to_talk', 'ptt')
                        .replace('open_assistant', 'assistant')
                        .replace('new_panel', 'new_panel')
                        .replace('close_panel', 'close_panel')
                        .replace('toggle_fullscreen', 'fullscreen')
                        .replace('open_settings', 'settings')
                        .replace('focus_panel_', 'panel')
                        .replace('cycle_panels', 'cycle');
                    document.getElementById('kb_' + elementId)?.classList.add('recording');
                }
                
                // Save settings
                function saveSettings() {
                    const settings = {
                        theme: { name: document.getElementById('theme').value },
                        colors: {
                            primary: document.getElementById('color_primary').value,
                            background: document.getElementById('color_background').value,
                            text: document.getElementById('color_text').value,
                            border: document.getElementById('color_border').value
                        },
                        font: {
                            family: document.getElementById('font_family').value,
                            size: parseInt(document.getElementById('font_size').value),
                            line_height: parseFloat(document.getElementById('line_height').value)
                        },
                        opacity: {
                            panel: parseFloat(document.getElementById('opacity_panel').value),
                            background: parseFloat(document.getElementById('opacity_background').value)
                        },
                        visualizer: {
                            type: document.getElementById('visualizer_type').value,
                            enabled: document.getElementById('visualizer_enabled').checked,
                            scale: parseFloat(document.getElementById('visualizer_scale').value),
                            position_x: parseFloat(document.getElementById('visualizer_pos_x').value),
                            position_y: parseFloat(document.getElementById('visualizer_pos_y').value),
                            pulse_with_audio: document.getElementById('visualizer_audio_react').checked,
                            bloom_intensity: parseFloat(document.getElementById('visualizer_bloom').value)
                        },
                        background: {
                            mode: document.getElementById('background_mode').value,
                            hex_grid: {
                                color: document.getElementById('hexgrid_color').value,
                                opacity: parseFloat(document.getElementById('hexgrid_opacity').value),
                                animation_speed: parseFloat(document.getElementById('hexgrid_speed').value),
                                glow_intensity: parseFloat(document.getElementById('hexgrid_glow').value)
                            }
                        },
                        startup: {
                            boot_animation: {
                                enabled: document.getElementById('boot_enabled').checked,
                                duration: parseFloat(document.getElementById('boot_duration').value),
                                skip_on_key: document.getElementById('boot_skip_key').checked,
                                music_enabled: document.getElementById('boot_music').checked
                            },
                            fast_start: {
                                enabled: document.getElementById('fast_start').checked,
                                delay: parseFloat(document.getElementById('fast_start_delay').value)
                            },
                            on_ready: {
                                action: document.getElementById('on_ready_action').value,
                                panels: { count: parseInt(document.getElementById('default_panels').value) }
                            }
                        },
                        voice: {
                            enabled: document.getElementById('voice_enabled').checked,
                            mode: document.getElementById('voice_mode').value,
                            sounds: {
                                enabled: document.getElementById('voice_sounds').checked,
                                volume: parseFloat(document.getElementById('voice_volume').value)
                            }
                        },
                        panels: {
                            history: {
                                enabled: document.getElementById('history_enabled').checked,
                                max_messages: parseInt(document.getElementById('history_max').value),
                                restore_on_launch: document.getElementById('history_restore').checked
                            },
                            max_panels: parseInt(document.getElementById('max_panels').value),
                            default_panel_width: parseFloat(document.getElementById('panel_width').value)
                        }
                    };
                    
                    window.webkit.messageHandlers.jarvisSettings.postMessage({
                        action: 'save',
                        settings: settings
                    });
                }
                
                function resetSection(section) {
                    if (confirm('Reset ' + section + ' settings to defaults?')) {
                        window.webkit.messageHandlers.jarvisSettings.postMessage({
                            action: 'reset_section',
                            section: section
                        });
                    }
                }
                
                function clearHistory() {
                    if (confirm('Clear all chat history? This cannot be undone.')) {
                        window.webkit.messageHandlers.jarvisSettings.postMessage({
                            action: 'clear_history'
                        });
                    }
                }
                
                function checkUpdates() {
                    window.webkit.messageHandlers.jarvisSettings.postMessage({
                        action: 'check_updates'
                    });
                }
                
                function openLogs() {
                    window.webkit.messageHandlers.jarvisSettings.postMessage({
                        action: 'open_logs'
                    });
                }
                
                // Load current settings
                function loadSettings(settings) {
                    // Theme
                    if (settings.theme) {
                        document.getElementById('theme').value = settings.theme.name || 'jarvis-dark';
                    }
                    
                    // Startup
                    if (settings.startup) {
                        document.getElementById('boot_enabled').checked = settings.startup.boot_animation?.enabled !== false;
                        document.getElementById('fast_start').checked = settings.startup.fast_start?.enabled || false;
                        document.getElementById('on_ready_action').value = settings.startup.on_ready?.action || 'listening';
                    }
                    
                    // Voice
                    if (settings.voice) {
                        document.getElementById('voice_enabled').checked = settings.voice.enabled !== false;
                        document.getElementById('voice_mode').value = settings.voice.mode || 'ptt';
                    }
                }
            </script>
        </body>
        </html>
        """
    }
    
    // MARK: - Message Handling
    
    private func handleMessage(_ message: [String: Any]) {
        guard let action = message["action"] as? String else { return }
        
        switch action {
        case "save":
            if let settings = message["settings"] as? [String: Any] {
                saveSettings(settings)
            }
        case "reset_section":
            if let section = message["section"] as? String {
                resetSection(section)
            }
        case "clear_history":
            clearHistory()
        case "check_updates":
            checkUpdates()
        case "open_logs":
            openLogs()
        default:
            break
        }
    }
    
    private func saveSettings(_ settings: [String: Any]) {
        // Send to Python to save to config.yaml
        if let jsonData = try? JSONSerialization.data(withJSONObject: settings),
           let json = String(data: jsonData, encoding: .utf8) {
            let message = "{\"type\":\"config_update\",\"settings\":\(json)}"
            print(message)
            fflush(stdout)
        }
        
        onSettingsSaved?()
        showToast("Settings saved. Restart required.")
    }
    
    private func resetSection(_ section: String) {
        let message = "{\"type\":\"config_reset_section\",\"section\":\"\(section)\"}"
        print(message)
        fflush(stdout)
        showToast("Reset \(section) to defaults.")
    }
    
    private func clearHistory() {
        let message = "{\"type\":\"config_clear_history\"}"
        print(message)
        fflush(stdout)
        showToast("Chat history cleared.")
    }
    
    private func checkUpdates() {
        // Sparkle update check
        showToast("Checking for updates...")
    }
    
    private func openLogs() {
        let logPath = "\(basePath)/jarvis.log"
        NSWorkspace.shared.open(URL(fileURLWithPath: logPath))
    }
    
    private func showToast(_ text: String) {
        let toast = NSView(frame: NSRect(x: 0, y: 0, width: 300, height: 44))
        toast.wantsLayer = true
        toast.layer?.backgroundColor = NSColor(red: 0, green: 0.15, blue: 0.2, alpha: 0.95).cgColor
        toast.layer?.cornerRadius = 8
        toast.layer?.borderColor = NSColor(red: 0, green: 0.83, blue: 1, alpha: 0.3).cgColor
        toast.layer?.borderWidth = 1
        
        let label = NSTextField(labelWithString: text)
        label.font = NSFont.systemFont(ofSize: 13)
        label.textColor = NSColor(red: 0, green: 0.83, blue: 1, alpha: 1)
        label.frame = NSRect(x: 16, y: 12, width: 268, height: 20)
        toast.addSubview(label)
        
        let toastX = (bounds.width - 300) / 2
        let toastY = bounds.height - 120
        toast.frame.origin = NSPoint(x: toastX, y: toastY)
        
        addSubview(toast)
        
        DispatchQueue.main.asyncAfter(deadline: .now() + 2.5) { [weak toast] in
            NSAnimationContext.runAnimationGroup { context in
                context.duration = 0.3
                toast?.animator().alphaValue = 0
            } completionHandler: {
                toast?.removeFromSuperview()
            }
        }
    }
}

// MARK: - WKNavigationDelegate

extension SettingsOverlay: WKNavigationDelegate {
    func webView(_ webView: WKWebView, didFinish navigation: WKNavigation!) {
        // Inject current settings
        let config = ConfigManager.shared
        let settings = """
        {
            "theme": { "name": "\(config.theme.name)" },
            "startup": {
                "boot_animation": { "enabled": \(config.startup.bootAnimation.enabled) },
                "fast_start": { "enabled": \(config.startup.fastStart.enabled) },
                "on_ready": { "action": "\(config.startup.onReady.action)" }
            },
            "voice": {
                "enabled": \(config.voice.enabled),
                "mode": "\(config.voice.mode)"
            }
        }
        """
        webView.evaluateJavaScript("loadSettings(\(settings))")
    }
}

// MARK: - WKScriptMessageHandler

extension SettingsOverlay: WKScriptMessageHandler {
    func userContentController(_ userContentController: WKUserContentController, didReceive message: WKScriptMessage) {
        if message.name == "jarvisSettings",
           let body = message.body as? [String: Any] {
            handleMessage(body)
        }
    }
}
