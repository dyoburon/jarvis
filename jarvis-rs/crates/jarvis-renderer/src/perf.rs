//! Frame timing and performance monitoring.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Tracks frame durations for FPS calculation.
pub struct FrameTimer {
    frame_times: VecDeque<Duration>,
    last_frame: Instant,
    max_samples: usize,
}

impl FrameTimer {
    /// Create a new frame timer with a default 120-sample rolling window.
    pub fn new() -> Self {
        Self {
            frame_times: VecDeque::new(),
            last_frame: Instant::now(),
            max_samples: 120,
        }
    }

    /// Record the start of a new frame. Call this once per frame.
    pub fn begin_frame(&mut self) {
        let now = Instant::now();
        let dt = now - self.last_frame;
        self.last_frame = now;
        self.frame_times.push_back(dt);
        if self.frame_times.len() > self.max_samples {
            self.frame_times.pop_front();
        }
    }

    /// Average frames per second over the sample window.
    pub fn fps(&self) -> f64 {
        if self.frame_times.is_empty() {
            return 0.0;
        }
        let total: f64 = self.frame_times.iter().map(|d| d.as_secs_f64()).sum();
        if total <= 0.0 {
            return 0.0;
        }
        self.frame_times.len() as f64 / total
    }

    /// Average frame time in milliseconds.
    pub fn frame_time_ms(&self) -> f64 {
        if self.frame_times.is_empty() {
            return 0.0;
        }
        let total: f64 = self.frame_times.iter().map(|d| d.as_secs_f64()).sum();
        (total / self.frame_times.len() as f64) * 1000.0
    }

    /// Number of frame samples currently stored.
    pub fn sample_count(&self) -> usize {
        self.frame_times.len()
    }
}

impl Default for FrameTimer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_fps_is_zero() {
        let timer = FrameTimer::new();
        assert_eq!(timer.fps(), 0.0);
        assert_eq!(timer.frame_time_ms(), 0.0);
    }

    #[test]
    fn fps_after_frames() {
        let mut timer = FrameTimer::new();
        // Simulate 10 frames at ~16ms (60fps)
        for _ in 0..10 {
            std::thread::sleep(Duration::from_millis(1));
            timer.begin_frame();
        }
        // FPS should be some positive number
        assert!(timer.fps() > 0.0);
        assert!(timer.frame_time_ms() > 0.0);
        assert_eq!(timer.sample_count(), 10);
    }

    #[test]
    fn max_samples_respected() {
        let mut timer = FrameTimer::new();
        for _ in 0..200 {
            timer.begin_frame();
        }
        assert!(timer.sample_count() <= 120);
    }
}
