//
//  BackgroundManager.swift
//  JarvisBootup
//
//  Creates and manages the active background based on configuration.
//

import Foundation

/// Manages the active background instance
class BackgroundManager {
    
    // MARK: - Singleton
    
    static let shared = BackgroundManager()
    
    // MARK: - Properties
    
    private(set) var activeBackground: Background
    private var config: BackgroundConfig = BackgroundConfig()
    
    // MARK: - Initialization
    
    private init() {
        // Initialize with default hex grid background
        activeBackground = HexGridBackground(config: config.hexGrid)
        metalLog("BackgroundManager: Initialized with hex_grid background (opacity: \(config.hexGrid.opacity))")
    }
    
    // MARK: - Configuration
    
    /// Update configuration and recreate background if needed
    func updateConfig(_ config: BackgroundConfig) {
        let typeChanged = self.config.mode != config.mode
        self.config = config
        
        if typeChanged || activeBackground is NullBackground {
            createBackground()
        }
        
        // Apply current config to background
        activeBackground.opacity = Float(config.hexGrid.opacity)
    }
    
    /// Create the appropriate background for the current mode
    private func createBackground() {
        switch config.mode {
        case "hex_grid":
            activeBackground = HexGridBackground(config: config.hexGrid)
        case "solid":
            activeBackground = SolidBackground(color: config.solidColor)
        case "gradient":
            activeBackground = GradientBackground(config: config.gradient)
        case "image":
            activeBackground = ImageBackground(config: config.image)
        case "video":
            activeBackground = VideoBackground(config: config.video)
        case "none":
            activeBackground = NullBackground()
        default:
            // Default to hex_grid for unknown types
            activeBackground = HexGridBackground(config: config.hexGrid)
        }
        
        metalLog("BackgroundManager: Created \(config.mode) background")
    }
    
    // MARK: - Updates
    
    /// Update with time delta
    func update(deltaTime: Float) {
        activeBackground.update(deltaTime: deltaTime)
    }
    
    // MARK: - Accessors
    
    /// Get hex grid color for renderer
    var hexGridColor: String {
        config.hexGrid.color
    }
    
    /// Get hex grid opacity for renderer
    var hexGridOpacity: Float {
        Float(config.hexGrid.opacity)
    }
    
    /// Get hex grid animation speed for renderer
    var hexGridAnimationSpeed: Float {
        Float(config.hexGrid.animationSpeed)
    }
}

// MARK: - Hex Grid Background

/// Animated hex grid background (the default Jarvis background)
class HexGridBackground: Background {
    var isVisible: Bool { opacity > 0 }
    var opacity: Float = 0.08
    
    private var config: HexGridBackgroundConfig
    private var time: Float = 0
    
    init(config: HexGridBackgroundConfig) {
        self.config = config
        self.opacity = Float(config.opacity)
    }
    
    func update(deltaTime: Float) {
        time += deltaTime * Float(config.animationSpeed)
        // Animation is handled in the shader
    }
}

// MARK: - Solid Background

/// Solid color background
class SolidBackground: Background {
    var isVisible: Bool { opacity > 0 }
    var opacity: Float = 1.0
    
    private let color: String
    
    init(color: String) {
        self.color = color
    }
    
    func update(deltaTime: Float) {}
}

// MARK: - Gradient Background

/// Gradient background
class GradientBackground: Background {
    var isVisible: Bool { opacity > 0 }
    var opacity: Float = 1.0
    
    private var config: GradientBackgroundConfig
    
    init(config: GradientBackgroundConfig) {
        self.config = config
    }
    
    func update(deltaTime: Float) {}
}

// MARK: - Image Background (Placeholder)

/// Static image background
class ImageBackground: Background {
    var isVisible: Bool { opacity > 0 && !config.path.isEmpty }
    var opacity: Float = 1.0
    
    private var config: ImageBackgroundConfig
    
    init(config: ImageBackgroundConfig) {
        self.config = config
        self.opacity = Float(config.opacity)
    }
    
    func update(deltaTime: Float) {}
}

// MARK: - Video Background (Placeholder)

/// Looping video background
class VideoBackground: Background {
    var isVisible: Bool { opacity > 0 && !config.path.isEmpty }
    var opacity: Float = 1.0
    
    private var config: VideoBackgroundConfig
    
    init(config: VideoBackgroundConfig) {
        self.config = config
    }
    
    func update(deltaTime: Float) {}
}
