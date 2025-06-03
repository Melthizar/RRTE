use std::time::{Duration, Instant};

/// Time tracking for the engine
#[derive(Debug, Clone)]
pub struct Time {
    start_time: Instant,
    last_frame: Instant,
    current_frame: Instant,
    delta_time: Duration,
    time_scale: f32,
}

impl Time {
    /// Create a new time tracker
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            start_time: now,
            last_frame: now,
            current_frame: now,
            delta_time: Duration::ZERO,
            time_scale: 1.0,
        }
    }

    /// Update the time for a new frame
    pub fn update(&mut self) {
        self.last_frame = self.current_frame;
        self.current_frame = Instant::now();
        self.delta_time = self.current_frame - self.last_frame;
    }

    /// Get delta time in seconds
    pub fn delta_time(&self) -> f32 {
        self.delta_time.as_secs_f32() * self.time_scale
    }

    /// Get total elapsed time since engine start
    pub fn elapsed_time(&self) -> f32 {
        (self.current_frame - self.start_time).as_secs_f32()
    }

    /// Get frames per second
    pub fn fps(&self) -> f32 {
        if self.delta_time.as_secs_f32() > 0.0 {
            1.0 / self.delta_time.as_secs_f32()
        } else {
            0.0
        }
    }

    /// Set time scale (1.0 = normal speed)
    pub fn set_time_scale(&mut self, scale: f32) {
        self.time_scale = scale.max(0.0);
    }

    /// Get current time scale
    pub fn time_scale(&self) -> f32 {
        self.time_scale
    }

    /// Start/reset the timer
    pub fn start(&mut self) {
        let now = Instant::now();
        self.start_time = now;
        self.last_frame = now;
        self.current_frame = now;
        self.delta_time = Duration::ZERO;
    }
}

impl Default for Time {
    fn default() -> Self {
        Self::new()
    }
}
