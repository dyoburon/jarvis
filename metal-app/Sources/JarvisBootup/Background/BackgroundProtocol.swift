//
//  BackgroundProtocol.swift
//  JarvisBootup
//
//  Protocol for background abstraction.
//  Allows different background types (hex_grid, solid, image, video, gradient, none).
//

import Foundation
import simd

// MARK: - Background Protocol

/// Protocol for all background types
protocol Background {
    /// Whether this background is currently visible
    var isVisible: Bool { get }
    
    /// Current opacity (0-1)
    var opacity: Float { get set }
    
    /// Update with time delta
    func update(deltaTime: Float)
}

// MARK: - Null Background

/// A transparent background
class NullBackground: Background {
    var isVisible: Bool { false }
    var opacity: Float = 0.0 { didSet {} }
    
    func update(deltaTime: Float) {}
}
