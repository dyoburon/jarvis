import Foundation
import WebKit

/// Manages theme loading and CSS variable injection.
///
/// Loads theme YAML files from resources/themes/ and converts them
/// to CSS variables that can be injected into WKWebViews.
final class ThemeManager {
    static let shared = ThemeManager()

    // MARK: - Properties

    private(set) var currentTheme: String = "jarvis-dark"
    private var themeCache: [String: ThemeColors] = [:]
    private let basePath: String

    // MARK: - Structs

    struct ThemeColors {
        let primary: String
        let secondary: String
        let background: String
        let panelBg: String
        let text: String
        let textMuted: String
        let border: String
        let borderFocused: String
        let userText: String
        let toolRead: String
        let toolEdit: String
        let toolWrite: String
        let toolRun: String
        let toolSearch: String
        let success: String
        let warning: String
        let error: String

        let fontFamily: String
        let fontSize: Int
        let titleSize: Int
        let lineHeight: Double
    }

    // MARK: - Init

    init() {
        // Derive base path from binary location
        let binary = CommandLine.arguments[0]
        let url = URL(fileURLWithPath: binary).resolvingSymlinksInPath()
        basePath = url.deletingLastPathComponent()  // .build/debug/
            .deletingLastPathComponent()        // .build/
            .deletingLastPathComponent()        // metal-app/
            .deletingLastPathComponent()        // jarvis/
            .path

        // Load built-in themes
        preloadThemes()
    }

    // MARK: - Theme Loading

    private func preloadThemes() {
        let themes = [
            "jarvis-dark", "jarvis-light", "nord", "dracula",
            "catppuccin-mocha", "tokyo-night", "gruvbox-dark", "solarized-dark"
        ]
        for theme in themes {
            _ = loadTheme(name: theme)
        }
    }

    /// Load a theme by name from YAML file.
    func loadTheme(name: String) -> ThemeColors? {
        // Check cache first
        if let cached = themeCache[name] {
            return cached
        }

        let themePath = "\(basePath)/resources/themes/\(name).yaml"

        guard let content = FileManager.default.contents(atPath: themePath),
              let yamlString = String(data: content, encoding: .utf8) else {
            metalLog("ThemeManager: Failed to load theme file at \(themePath)")
            return getDefaultColors()
        }

        let colors = parseYAMLTheme(yamlString, name: name)
        themeCache[name] = colors
        return colors
    }

    /// Parse YAML theme file into ThemeColors struct.
    private func parseYAMLTheme(_ yaml: String, name: String) -> ThemeColors {
        var colors: [String: String] = defaultColorMap()
        var font: [String: Any] = defaultFontMap()

        // Simple YAML parsing (key: value)
        let lines = yaml.components(separatedBy: .newlines)
        var currentSection = ""
        var inColors = false
        var inFont = false

        for line in lines {
            let trimmed = line.trimmingCharacters(in: .whitespaces)

            // Skip comments and empty lines
            if trimmed.isEmpty || trimmed.hasPrefix("#") { continue }

            // Detect section headers
            if trimmed == "colors:" {
                inColors = true
                inFont = false
                continue
            } else if trimmed == "font:" {
                inFont = true
                inColors = false
                continue
            } else if !line.hasPrefix(" ") && trimmed.hasSuffix(":") {
                inColors = false
                inFont = false
                continue
            }

            // Parse key: value
            if let colonIndex = trimmed.firstIndex(of: ":") {
                let key = String(trimmed[..<colonIndex]).trimmingCharacters(in: .whitespaces)
                var value = String(trimmed[trimmed.index(after: colonIndex)...])
                    .trimmingCharacters(in: .whitespaces)

                // Remove quotes
                if value.hasPrefix("\"") && value.hasSuffix("\"") {
                    value = String(value.dropFirst().dropLast())
                }

                // Map to snake_case
                let snakeKey = key.replacingOccurrences(of: "-", with: "_")

                if inColors {
                    colors[snakeKey] = value
                } else if inFont {
                    font[snakeKey] = value
                }
            }
        }

        return ThemeColors(
            primary: colors["primary"] ?? "#00d4ff",
            secondary: colors["secondary"] ?? "#ff6b00",
            background: colors["background"] ?? "#000000",
            panelBg: colors["panel_bg"] ?? "rgba(0,0,0,0.93)",
            text: colors["text"] ?? "#f0ece4",
            textMuted: colors["text_muted"] ?? "#888888",
            border: colors["border"] ?? "rgba(0,212,255,0.12)",
            borderFocused: colors["border_focused"] ?? "rgba(0,212,255,0.5)",
            userText: colors["user_text"] ?? "rgba(140,190,220,0.65)",
            toolRead: colors["tool_read"] ?? "rgba(100,180,255,0.9)",
            toolEdit: colors["tool_edit"] ?? "rgba(255,180,80,0.9)",
            toolWrite: colors["tool_write"] ?? "rgba(255,180,80,0.9)",
            toolRun: colors["tool_run"] ?? "rgba(80,220,120,0.9)",
            toolSearch: colors["tool_search"] ?? "rgba(200,150,255,0.9)",
            success: colors["success"] ?? "#00ff88",
            warning: colors["warning"] ?? "#ff6b00",
            error: colors["error"] ?? "#ff4444",
            fontFamily: font["family"] as? String ?? "Menlo",
            fontSize: (font["size"] as? String).flatMap { Int($0) } ?? 13,
            titleSize: (font["title_size"] as? String).flatMap { Int($0) } ?? 15,
            lineHeight: (font["line_height"] as? String).flatMap { Double($0) } ?? 1.6
        )
    }

    private func defaultColorMap() -> [String: String] {
        return [
            "primary": "#00d4ff",
            "secondary": "#ff6b00",
            "background": "#000000",
            "panel_bg": "rgba(0,0,0,0.93)",
            "text": "#f0ece4",
            "text_muted": "#888888",
            "border": "rgba(0,212,255,0.12)",
            "border_focused": "rgba(0,212,255,0.5)",
            "user_text": "rgba(140,190,220,0.65)",
            "tool_read": "rgba(100,180,255,0.9)",
            "tool_edit": "rgba(255,180,80,0.9)",
            "tool_write": "rgba(255,180,80,0.9)",
            "tool_run": "rgba(80,220,120,0.9)",
            "tool_search": "rgba(200,150,255,0.9)",
            "success": "#00ff88",
            "warning": "#ff6b00",
            "error": "#ff4444"
        ]
    }

    private func defaultFontMap() -> [String: Any] {
        return [
            "family": "Menlo",
            "size": "13",
            "title_size": "15",
            "line_height": "1.6"
        ]
    }

    private func getDefaultColors() -> ThemeColors {
        let map = defaultColorMap()
        return ThemeColors(
            primary: map["primary"]!,
            secondary: map["secondary"]!,
            background: map["background"]!,
            panelBg: map["panel_bg"]!,
            text: map["text"]!,
            textMuted: map["text_muted"]!,
            border: map["border"]!,
            borderFocused: map["border_focused"]!,
            userText: map["user_text"]!,
            toolRead: map["tool_read"]!,
            toolEdit: map["tool_edit"]!,
            toolWrite: map["tool_write"]!,
            toolRun: map["tool_run"]!,
            toolSearch: map["tool_search"]!,
            success: map["success"]!,
            warning: map["warning"]!,
            error: map["error"]!,
            fontFamily: "Menlo",
            fontSize: 13,
            titleSize: 15,
            lineHeight: 1.6
        )
    }

    // MARK: - CSS Generation

    /// Generate CSS variables string from theme colors.
    func generateCSSVariables(for themeName: String) -> String {
        let colors = loadTheme(name: themeName) ?? getDefaultColors()
        return generateCSSVariables(colors: colors)
    }

    /// Generate CSS variables string from ThemeColors struct.
    func generateCSSVariables(colors: ThemeColors) -> String {
        return """
        :root {
            --color-primary: \(colors.primary);
            --color-secondary: \(colors.secondary);
            --color-background: \(colors.background);
            --color-panel-bg: \(colors.panelBg);
            --color-text: \(colors.text);
            --color-text-muted: \(colors.textMuted);
            --color-border: \(colors.border);
            --color-border-focused: \(colors.borderFocused);
            --color-user-text: \(colors.userText);
            --color-tool-read: \(colors.toolRead);
            --color-tool-edit: \(colors.toolEdit);
            --color-tool-write: \(colors.toolWrite);
            --color-tool-run: \(colors.toolRun);
            --color-tool-search: \(colors.toolSearch);
            --color-success: \(colors.success);
            --color-warning: \(colors.warning);
            --color-error: \(colors.error);
            --font-family: \(colors.fontFamily), Monaco, 'Courier New', monospace;
            --font-size: \(colors.fontSize)px;
            --font-title-size: \(colors.titleSize)px;
            --line-height: \(colors.lineHeight);
        }
        """
    }

    /// Generate full CSS for injection into WKWebView.
    func generateThemeCSS(for themeName: String) -> String {
        let colors = loadTheme(name: themeName) ?? getDefaultColors()
        let vars = generateCSSVariables(colors: colors)

        return """
        <style id="jarvis-theme-vars">
        \(vars)
        </style>
        """
    }

    // MARK: - Injection

    /// Inject theme CSS into a WKWebView.
    func injectTheme(into webView: WKWebView, themeName: String? = nil) {
        let theme = themeName ?? currentTheme
        let css = generateCSSVariables(for: theme)

        // Remove old theme vars first
        let removeScript = """
        (function() {
            var old = document.getElementById('jarvis-theme-vars');
            if (old) old.remove();
        })();
        """

        // Inject new vars
        let injectScript = """
        (function() {
            var style = document.createElement('style');
            style.id = 'jarvis-theme-vars';
            style.textContent = `\(css)`;
            document.head.appendChild(style);
        })();
        """

        webView.evaluateJavaScript(removeScript) { _, _ in
            webView.evaluateJavaScript(injectScript) { _, error in
                if let error = error {
                    metalLog("ThemeManager: Failed to inject theme: \(error.localizedDescription)")
                } else {
                    metalLog("ThemeManager: Injected theme '\(theme)'")
                }
            }
        }
    }

    /// Update current theme and inject into all web views.
    func setTheme(_ name: String) {
        currentTheme = name
        metalLog("ThemeManager: Theme changed to '\(name)'")
        // Note: Actual injection happens via injectTheme calls from ChatWebView
    }

    // MARK: - Available Themes

    /// Get list of available theme names.
    func availableThemes() -> [ThemeInfo] {
        return [
            ThemeInfo(name: "jarvis-dark", displayName: "Jarvis Dark", description: "The signature Jarvis theme with cyan accents"),
            ThemeInfo(name: "jarvis-light", displayName: "Jarvis Light", description: "Light theme with cyan accents"),
            ThemeInfo(name: "nord", displayName: "Nord", description: "Arctic, bluish color palette"),
            ThemeInfo(name: "dracula", displayName: "Dracula", description: "Dark theme with purple accents"),
            ThemeInfo(name: "catppuccin-mocha", displayName: "Catppuccin Mocha", description: "Soothing pastel theme"),
            ThemeInfo(name: "tokyo-night", displayName: "Tokyo Night", description: "Clean dark theme with blue accents"),
            ThemeInfo(name: "gruvbox-dark", displayName: "Gruvbox Dark", description: "Retro groove color palette"),
            ThemeInfo(name: "solarized-dark", displayName: "Solarized Dark", description: "Precision color scheme")
        ]
    }

    struct ThemeInfo {
        let name: String
        let displayName: String
        let description: String
    }

    // MARK: - ConfigManager Integration

    /// Load theme from ConfigManager.
    func loadFromConfig() {
        let configTheme = ConfigManager.shared.theme.name
        if !configTheme.isEmpty {
            currentTheme = configTheme
            metalLog("ThemeManager: Loaded theme '\(configTheme)' from config")
        }
    }
}
