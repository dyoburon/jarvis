import AVFoundation

/// Manages voiceover and music playback with real-time audio level metering
/// for driving audio-reactive shader uniforms.
class AudioEngine {
    private let engine = AVAudioEngine()
    private let voiceoverNode = AVAudioPlayerNode()
    private let musicNode = AVAudioPlayerNode()
    private var _currentLevel: Float = 0
    private var _smoothedLevel: Float = 0
    private var fadeTimer: Timer?

    init() {
        engine.attach(voiceoverNode)
        engine.attach(musicNode)
        engine.connect(voiceoverNode, to: engine.mainMixerNode, format: nil)
        engine.connect(musicNode, to: engine.mainMixerNode, format: nil)

        // Install tap for real-time level metering → drives orb pulsing
        let format = engine.mainMixerNode.outputFormat(forBus: 0)
        engine.mainMixerNode.installTap(
            onBus: 0,
            bufferSize: 1024,
            format: format
        ) { [weak self] buffer, _ in
            guard let channelData = buffer.floatChannelData?[0] else { return }
            let frameLength = Int(buffer.frameLength)
            var sum: Float = 0
            for i in 0..<frameLength {
                sum += channelData[i] * channelData[i]
            }
            let rms = sqrt(sum / Float(max(frameLength, 1)))
            // Scale up for visual responsiveness, clamp to 0-1
            let level = min(rms * 4.0, 1.0)
            DispatchQueue.main.async {
                guard let self = self else { return }
                // Noise floor: ignore levels below threshold
                let cleaned = level > 0.03 ? level : 0
                // Exponential moving average for smooth animation
                self._smoothedLevel = self._smoothedLevel * 0.7 + cleaned * 0.3
                self._currentLevel = self._smoothedLevel
            }
        }

        do {
            try engine.start()
        } catch {
            print("[Jarvis Audio] Engine start failed: \(error)")
        }
    }

    /// Current audio level (0-1) for driving shader uniforms
    var currentLevel: Float {
        return _currentLevel
    }

    func playVoiceover(url: URL) {
        guard FileManager.default.fileExists(atPath: url.path) else {
            print("[Jarvis Audio] Voiceover not found: \(url.path)")
            return
        }
        guard let file = try? AVAudioFile(forReading: url) else {
            print("[Jarvis Audio] Could not read voiceover file")
            return
        }
        voiceoverNode.scheduleFile(file, at: nil)
        voiceoverNode.play()
    }

    func playMusic(url: URL) {
        guard FileManager.default.fileExists(atPath: url.path) else {
            print("[Jarvis Audio] Music not found: \(url.path)")
            return
        }
        guard let file = try? AVAudioFile(forReading: url) else {
            print("[Jarvis Audio] Could not read music file")
            return
        }
        musicNode.volume = 0
        musicNode.scheduleFile(file, at: nil)
        musicNode.play()

        // Fade in over 1 second
        fadeVolume(node: musicNode, to: 1.0, duration: 1.0)
    }

    func fadeOutMusic(duration: TimeInterval) {
        fadeVolume(node: musicNode, to: 0.0, duration: duration)
    }

    private func fadeVolume(node: AVAudioPlayerNode, to target: Float, duration: TimeInterval) {
        fadeTimer?.invalidate()
        let steps = 30
        let interval = duration / Double(steps)
        let startVol = node.volume
        var step = 0

        fadeTimer = Timer.scheduledTimer(withTimeInterval: interval, repeats: true) { timer in
            step += 1
            let progress = Float(step) / Float(steps)
            node.volume = startVol + (target - startVol) * progress
            if step >= steps {
                timer.invalidate()
                node.volume = target
            }
        }
    }

    func stop() {
        fadeTimer?.invalidate()
        engine.mainMixerNode.removeTap(onBus: 0)
        voiceoverNode.stop()
        musicNode.stop()
        engine.stop()
    }
}
