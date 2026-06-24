//! Fixed-timestep accumulator and FPS tracking.
//!
//! Uses `web_time::Instant`, a drop-in replacement for `std::time::Instant`
//! that also works on `wasm32` (where `std::time::Instant` panics).

use web_time::Instant;

pub struct TimeStep {
    /// Seconds per fixed update (1.0 / target_ups).
    fixed_dt: f32,
    /// Real time since the last frame that has not yet been consumed by updates.
    accumulator: f32,
    /// Maximum real time to consume per frame, to avoid the "spiral of death"
    /// where slow frames queue ever more updates.
    max_frame_time: f32,

    last: Instant,
    start: Instant,

    /// Total elapsed wall-clock time in seconds since startup.
    total: f64,

    // FPS tracking (render frames per second, sampled once a second).
    fps: u32,
    frame_counter: u32,
    fps_timer: f32,
    /// DIAG: worst (longest) frame time seen in the current 1s window.
    worst_frame: f32,
}

impl TimeStep {
    pub fn new(target_ups: u32) -> Self {
        let fixed_dt = 1.0 / target_ups.max(1) as f32;
        let now = Instant::now();
        Self {
            fixed_dt,
            accumulator: 0.0,
            max_frame_time: 0.25,
            last: now,
            start: now,
            total: 0.0,
            fps: 0,
            frame_counter: 0,
            fps_timer: 0.0,
            worst_frame: 0.0,
        }
    }

    pub fn fixed_dt(&self) -> f32 {
        self.fixed_dt
    }

    /// Total elapsed time in seconds since startup.
    pub fn total(&self) -> f64 {
        self.total
    }

    pub fn fps(&self) -> u32 {
        self.fps
    }

    /// Advance real time by one render frame, feeding the accumulator. Returns
    /// the frame's real delta time in seconds (capped at `max_frame_time`).
    pub fn frame(&mut self) -> f32 {
        let now = Instant::now();
        let frame_time_raw = now.duration_since(self.last).as_secs_f32();
        self.last = now;
        self.total = now.duration_since(self.start).as_secs_f64();

        let frame_time = frame_time_raw.min(self.max_frame_time);
        self.accumulator += frame_time;

        // FPS sampling.
        self.frame_counter += 1;
        self.fps_timer += frame_time;
        if frame_time_raw > self.worst_frame {
            self.worst_frame = frame_time_raw;
        }
        if self.fps_timer >= 1.0 {
            self.fps = self.frame_counter;
            self.frame_counter = 0;
            self.fps_timer = 0.0;
            self.worst_frame = 0.0;
        }

        frame_time
    }

    /// Consume one fixed step from the accumulator if enough time is available.
    /// Call in a `while time.next_fixed_step() { game.update(...) }` loop.
    pub fn next_fixed_step(&mut self) -> bool {
        if self.accumulator >= self.fixed_dt {
            self.accumulator -= self.fixed_dt;
            true
        } else {
            false
        }
    }
}
