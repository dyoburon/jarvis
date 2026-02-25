import Foundation
import QuartzCore

/// Master sequence controller.
/// Called each frame by the Renderer to update shader uniforms.
///
/// Phase 1 (0–9s):    Boot logs stream in, hex grid bg, orb fades in from 6s
/// Phase 2 (9–16s):   Music starts, orb ramps, voiceover plays, status text
/// Phase 3 (16–23s):  Music vibes, orb pulses
/// Phase 3b (23–27s): Collapse — orb shrinks, moves to bottom-right, bg goes transparent
/// Phase 4 (27s+):    Persistent mode (--jarvis) — orb stays, driven by stdin
class Timeline {
    private let hudRenderer: HUDTextRenderer
    private let audioEngine: AudioEngine
    private let basePath: String
    private let onComplete: () -> Void

    private var startTime: Double = 0
    private var started = false

    // One-shot triggers
    private var voiceoverPlayed = false
    private var musicStarted = false
    private var statusTextShown = false
    private var collapseStarted = false

    private let totalDuration: Double = 27.0

    /// When true, app persists after bootup instead of quitting
    var jarvisMode = false

    /// Audio level injected from stdin in jarvis mode
    /// Python already applies 70/30 EMA (same as vibetotext AudioEngine),
    /// so no additional smoothing needed here.
    var externalAudioLevel: Float = 0
    private var smoothedAudioLevel: Float = 0

    /// Current jarvis state: "listening", "speaking", "skill"
    var jarvisState: String = "listening"

    /// Whether we've entered Phase 4
    private var inPhase4 = false

    init(hudRenderer: HUDTextRenderer,
         audioEngine: AudioEngine,
         basePath: String,
         onComplete: @escaping () -> Void) {
        self.hudRenderer = hudRenderer
        self.audioEngine = audioEngine
        self.basePath = basePath
        self.onComplete = onComplete
    }

    func start() {
        startTime = CACurrentMediaTime()
        started = true
    }

    /// Called every frame by the Renderer. Drives all animation.
    func update(uniforms: inout Uniforms) {
        guard started else { return }
        let elapsed = CACurrentMediaTime() - startTime

        // Jarvis mode: skip bootup, go straight to persistent mode
        if jarvisMode {
            updateJarvis(elapsed: elapsed, uniforms: &uniforms)
            return
        }

        // Sequence complete
        if elapsed > totalDuration + 0.5 {
            onComplete()
            started = false
            return
        }

        uniforms.time = Float(elapsed)
        uniforms.audioLevel = audioEngine.currentLevel

        // Defaults
        uniforms.orbCenterX = 0
        uniforms.orbCenterY = 0
        uniforms.orbScale = 1.0
        uniforms.bgOpacity = 1.0
        uniforms.bgAlpha = 1.0

        if elapsed < 9.0 {
            updatePhase1(elapsed: elapsed, uniforms: &uniforms)
        } else if elapsed < 16.0 {
            updatePhase2(elapsed: elapsed, uniforms: &uniforms)
        } else {
            updatePhase3(elapsed: elapsed, uniforms: &uniforms)
        }
    }

    // ── Phase 1: Boot Logs (0–9s) ──

    private func updatePhase1(elapsed: Double, uniforms: inout Uniforms) {
        uniforms.hudOpacity = 1.0
        uniforms.intensity = 0
        uniforms.scanlineIntensity = 0.15
        uniforms.vignetteIntensity = 1.2

        // Hex grid fades in over first 2 seconds
        uniforms.bgOpacity = Float(min(1.0, elapsed / 2.0)) * 0.8

        // Orb starts fading in at 6s
        if elapsed > 6.0 {
            let t = Float(elapsed - 6.0) / 3.0  // 0→1 over 3 seconds
            uniforms.powerLevel = t * 0.3        // max 0.3 during phase 1
        } else {
            uniforms.powerLevel = 0
        }
    }

    // ── Phase 2: Music + Orb + Voiceover (9–16s) ──

    private func updatePhase2(elapsed: Double, uniforms: inout Uniforms) {
        let phaseT = Float(elapsed - 9.0)  // 0→7

        // Start music at the beginning of Phase 2
        if !musicStarted {
            musicStarted = true
            let url = URL(fileURLWithPath: "\(basePath)/metal-app/assets/audio/morning-alarm.mp3")
            audioEngine.playMusic(url: url)
        }

        // Fade out boot text quickly (0.5s)
        let textFade = max(0, 1.0 - phaseT * 2.0)

        // Orb ramps to full
        uniforms.powerLevel = min(1.0, 0.3 + phaseT * 0.28)

        uniforms.intensity = 0
        uniforms.scanlineIntensity = max(0.08, 0.15 - phaseT * 0.01)
        uniforms.vignetteIntensity = 1.2
        uniforms.bgOpacity = 0.8

        // Show status text after boot text fades
        if phaseT > 0.6 && !statusTextShown {
            statusTextShown = true
            hudRenderer.clearLines()
            hudRenderer.setStatusText("ALL SYSTEMS ONLINE")
        }

        // HUD opacity: boot text fading out, then status text fading in
        if phaseT < 0.6 {
            uniforms.hudOpacity = textFade
        } else {
            let statusFade = min(1.0, (phaseT - 0.6) * 1.5)
            uniforms.hudOpacity = statusFade * 0.8
        }

        // Play voiceover at 11s
        if elapsed > 11.0 && !voiceoverPlayed {
            voiceoverPlayed = true
            let url = URL(fileURLWithPath: "\(basePath)/metal-app/assets/audio/bootup-voice.mp3")
            audioEngine.playVoiceover(url: url)
        }
    }

    // ── Phase 3: Music Vibes + Collapse (16–27s) ──
    //
    // 16–23s: Music plays, orb vibes with audio
    // 23–27s: Collapse — orb shrinks & moves, background goes transparent

    private func updatePhase3(elapsed: Double, uniforms: inout Uniforms) {
        let phaseT = Float(elapsed - 16.0)  // 0→11

        // Clear HUD on first frame of Phase 3
        if statusTextShown {
            statusTextShown = false
            hudRenderer.setStatusText(nil)
            hudRenderer.setOpacity(0)
        }

        uniforms.hudOpacity = 0
        uniforms.scanlineIntensity = 0.08

        // Orb at full power during music
        uniforms.powerLevel = 1.0
        // Gentle intensity pulse with music
        uniforms.intensity = min(0.6, phaseT * 0.15)

        uniforms.bgOpacity = 0.8
        uniforms.vignetteIntensity = 1.0

        // ── Collapse animation (starts at 23s = phaseT 7) ──
        let collapseStart: Float = 7.0
        let collapseDuration: Float = 4.0

        if phaseT > collapseStart {
            let collapseT = (phaseT - collapseStart) / collapseDuration  // 0→1

            // Smooth ease-in-out
            let ease = collapseT * collapseT * (3.0 - 2.0 * collapseT)

            // Orb shrinks
            uniforms.orbScale = max(0.12, 1.0 - ease * 0.88)

            // Orb moves to bottom-right
            uniforms.orbCenterX = ease * 0.38
            uniforms.orbCenterY = ease * 0.38

            // Hex grid fades out
            uniforms.bgOpacity = max(0, 0.8 * (1.0 - ease))

            // Background becomes transparent
            uniforms.bgAlpha = max(0, 1.0 - ease)

            // Vignette closes in slightly
            uniforms.vignetteIntensity = 1.0 + ease * 1.5

            // Intensity settles
            uniforms.intensity = max(0, uniforms.intensity * (1.0 - ease * 0.5))

            // Fade music
            if !collapseStarted {
                collapseStarted = true
                audioEngine.fadeOutMusic(duration: Double(collapseDuration))
            }

            // Final fade — orb dims in last second
            if collapseT > 0.75 {
                let finalFade = (collapseT - 0.75) / 0.25
                uniforms.powerLevel = max(0, 1.0 - finalFade)
            }
        }
    }

    // ── Jarvis Persistent Mode ──
    //
    // Full-screen orb + hex grid, driven by stdin commands from Python.
    // - Orb centered, full size, pulses with audio
    // - HUD text shows transcripts (top-left) and skill output
    // - Quick 2s fade-in on launch

    private func updateJarvis(elapsed: Double, uniforms: inout Uniforms) {
        smoothedAudioLevel = smoothedAudioLevel * 0.5 + externalAudioLevel * 0.5
        uniforms.time = Float(elapsed)
        uniforms.audioLevel = smoothedAudioLevel

        // Orb centered, full size
        uniforms.orbCenterX = 0
        uniforms.orbCenterY = 0
        uniforms.orbScale = 1.0

        // Fade in over first 2 seconds
        let fadeIn = Float(min(1.0, elapsed / 2.0))

        // Hex grid background (dimmer so sphere stands out)
        uniforms.bgOpacity = fadeIn * (jarvisState == "chat" ? 1.2 : 1.0)
        uniforms.bgAlpha = 1.0

        uniforms.scanlineIntensity = 0.03
        uniforms.vignetteIntensity = 0.25

        // Orb brightness based on state
        switch jarvisState {
        case "speaking":
            uniforms.powerLevel = fadeIn * (1.4 + smoothedAudioLevel * 0.6)
            uniforms.intensity = 0.5 + smoothedAudioLevel * 0.5
        case "skill":
            uniforms.powerLevel = fadeIn * 1.2
            uniforms.intensity = 0.3
        case "chat":
            // Orb sits below chat panels in the hex grid area
            uniforms.orbCenterX = 0.10
            uniforms.orbCenterY = 0.30
            uniforms.orbScale = 0.55
            uniforms.powerLevel = fadeIn * 1.3
            uniforms.intensity = 0.5
        default: // listening
            uniforms.powerLevel = fadeIn * (1.0 + smoothedAudioLevel * 0.6)
            uniforms.intensity = 0.2 + smoothedAudioLevel * 0.4
        }

        // HUD text — always visible for transcripts, brighter for skills/chat
        switch jarvisState {
        case "skill", "chat":
            uniforms.hudOpacity = fadeIn * 1.0
        case "speaking":
            uniforms.hudOpacity = fadeIn * 0.8
        default:
            uniforms.hudOpacity = fadeIn * 0.6
        }
    }
}
