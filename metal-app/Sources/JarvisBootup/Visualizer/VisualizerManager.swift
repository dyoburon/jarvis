//
//  VisualizerManager.swift
//  JarvisBootup
//
//  Creates and manages the active visualizer based on configuration.
//

import Foundation
import simd

/// Manages the active visualizer instance
class VisualizerManager {
    
    // MARK: - Singleton
    
    static let shared = VisualizerManager()
    
    // MARK: - Properties
    
    private(set) var activeVisualizer: Visualizer = NullVisualizer()
    private var config: VisualizerConfig = VisualizerConfig()
    
    // MARK: - Initialization
    
    private init() {}
    
    // MARK: - Configuration
    
    /// Update configuration and recreate visualizer if needed
    func updateConfig(_ config: VisualizerConfig) {
        let typeChanged = self.config.type != config.type
        self.config = config
        
        if typeChanged || activeVisualizer is NullVisualizer {
            createVisualizer()
        }
        
        // Apply current config to visualizer
        activeVisualizer.scale = Float(config.scale)
        activeVisualizer.intensity = Float(config.orb.intensityBase)
        activeVisualizer.position = simd_float2(Float(config.positionX), Float(config.positionY))
    }
    
    /// Create the appropriate visualizer for the current type
    private func createVisualizer() {
        switch config.type {
        case "orb":
            // The orb is handled by the existing Renderer - we'll use a wrapper
            activeVisualizer = OrbVisualizer(config: config.orb)
        case "particle":
            activeVisualizer = ParticleVisualizer(config: config.particle)
        case "waveform":
            activeVisualizer = WaveformVisualizer(config: config.waveform)
        case "none":
            activeVisualizer = NullVisualizer()
        default:
            // Default to orb for unknown types
            activeVisualizer = OrbVisualizer(config: config.orb)
        }
        
        metalLog("VisualizerManager: Created \(config.type) visualizer, enabled=\(config.enabled)")
    }
    
    // MARK: - State Management
    
    /// Apply state-specific overrides to the visualizer
    func setState(_ state: VisualizerState) {
        guard config.reactToState else { return }
        
        let stateConfig: VisualizerStateConfig
        switch state {
        case .listening:
            stateConfig = config.stateListening
        case .speaking:
            stateConfig = config.stateSpeaking
        case .skill:
            stateConfig = config.stateSkill
        case .chat:
            stateConfig = config.stateChat
        case .idle:
            stateConfig = config.stateIdle
        }
        
        // Apply overrides
        activeVisualizer.scale = Float(stateConfig.scale)
        activeVisualizer.intensity = Float(stateConfig.intensity)
        
        if let posX = stateConfig.positionX, let posY = stateConfig.positionY {
            activeVisualizer.position = simd_float2(Float(posX), Float(posY))
        }
        
        activeVisualizer.applyState(state)
    }
    
    // MARK: - Updates
    
    /// Update audio level for reactive visualizers
    func updateAudioLevel(_ level: Float) {
        guard config.enabled && config.reactToAudio else { return }
        activeVisualizer.updateAudioLevel(level)
    }
    
    /// Update with time delta
    func update(deltaTime: Float) {
        guard config.enabled else { return }
        activeVisualizer.update(deltaTime: deltaTime)
    }
}

// MARK: - Orb Visualizer Wrapper

/// Wraps the existing orb rendering with the Visualizer protocol
class OrbVisualizer: Visualizer {
    var isVisible: Bool { true }
    
    var scale: Float = 1.0 {
        didSet {
            // Will be applied via uniforms in Renderer
        }
    }
    
    var intensity: Float = 1.0 {
        didSet {
            // Will be applied via uniforms in Renderer
        }
    }
    
    var position: simd_float2 = simd_float2(0, 0) {
        didSet {
            // Will be applied via uniforms in Renderer
        }
    }
    
    private var config: OrbVisualizerConfig
    
    init(config: OrbVisualizerConfig) {
        self.config = config
        self.scale = Float(config.intensityBase)
    }
    
    func updateAudioLevel(_ level: Float) {
        // Audio reactivity handled by existing Renderer
    }
    
    func update(deltaTime: Float) {
        // Animation handled by existing Renderer
    }
    
    func applyState(_ state: VisualizerState) {
        // State changes handled by setState in VisualizerManager
    }
}

// MARK: - Particle Visualizer (Placeholder)

/// Particle system visualizer (to be implemented)
class ParticleVisualizer: Visualizer {
    var isVisible: Bool { true }
    var scale: Float = 1.0
    var intensity: Float = 1.0
    var position: simd_float2 = simd_float2(0, 0)
    
    private var config: ParticleVisualizerConfig
    
    init(config: ParticleVisualizerConfig) {
        self.config = config
    }
    
    func updateAudioLevel(_ level: Float) {}
    func update(deltaTime: Float) {}
    func applyState(_ state: VisualizerState) {}
}

// MARK: - Waveform Visualizer (Placeholder)

/// Audio waveform visualizer (to be implemented)
class WaveformVisualizer: Visualizer {
    var isVisible: Bool { true }
    var scale: Float = 1.0
    var intensity: Float = 1.0
    var position: simd_float2 = simd_float2(0, 0)
    
    private var config: WaveformVisualizerConfig
    
    init(config: WaveformVisualizerConfig) {
        self.config = config
    }
    
    func updateAudioLevel(_ level: Float) {}
    func update(deltaTime: Float) {}
    func applyState(_ state: VisualizerState) {}
}
