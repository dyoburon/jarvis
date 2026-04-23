//
//  VisualizerProtocol.swift
//  JarvisBootup
//
//  Protocol for visualizer abstraction.
//  Allows different visualization types (orb, image, video, particle, waveform, none).
//

import Foundation
import simd

// MARK: - Visualizer Protocol

/// Protocol for all visualizer types
protocol Visualizer {
    /// Whether this visualizer is currently visible
    var isVisible: Bool { get }
    
    /// Current scale multiplier
    var scale: Float { get set }
    
    /// Current intensity multiplier
    var intensity: Float { get set }
    
    /// Current position offset (-1 to 1)
    var position: simd_float2 { get set }
    
    /// Update with audio level (0-1)
    func updateAudioLevel(_ level: Float)
    
    /// Update with time delta
    func update(deltaTime: Float)
    
    /// Apply state-specific overrides
    func applyState(_ state: VisualizerState)
}

// MARK: - Visualizer State

enum VisualizerState: String {
    case listening
    case speaking
    case skill
    case chat
    case idle
}

// MARK: - Null Visualizer

/// A no-op visualizer for when visualizer is disabled
class NullVisualizer: Visualizer {
    var isVisible: Bool { false }
    var scale: Float = 1.0 { didSet {} }
    var intensity: Float = 1.0 { didSet {} }
    var position: simd_float2 = simd_float2(0, 0) { didSet {} }
    
    func updateAudioLevel(_ level: Float) {}
    func update(deltaTime: Float) {}
    func applyState(_ state: VisualizerState) {}
}
