use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerErrorKind {
    AlreadyRunning,
    NotRunning,
    Invalid,
}

#[derive(Debug)]
pub struct TimerError(pub TimerErrorKind);

#[derive(Debug, Clone, serde::Serialize)]
pub struct Lap {
    pub index: usize,
    // store lap time as milliseconds since start for simple serialization
    pub at_ms: u128,
    pub label: Option<String>,
}

/// Core timer with laps support.
pub struct Timer {
    start_time: Option<Instant>,
    elapsed: Duration,
    running: bool,
    laps: Vec<Lap>,
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}

impl Timer {
    pub fn new() -> Self {
        Self { start_time: None, elapsed: Duration::ZERO, running: false, laps: Vec::new() }
    }

    /// Start the timer if not already running.
    pub fn start(&mut self) -> Result<(), TimerError> {
        if self.running {
            return Err(TimerError(TimerErrorKind::AlreadyRunning));
        }
        self.start_time = Some(Instant::now());
        self.running = true;
        Ok(())
    }

    /// Stop the timer and accumulate elapsed time.
    pub fn stop(&mut self) -> Result<(), TimerError> {
        if !self.running {
            return Err(TimerError(TimerErrorKind::NotRunning));
        }
        if let Some(start) = self.start_time.take() {
            self.elapsed += start.elapsed();
        }
        self.running = false;
        Ok(())
    }

    /// Reset the timer to zero, stop it, and clear laps.
    pub fn reset(&mut self) {
        self.start_time = None;
        self.elapsed = Duration::ZERO;
        self.running = false;
        self.laps.clear();
    }

    /// Total elapsed time, including current running slice if any.
    pub fn elapsed(&self) -> Duration {
        if self.running {
            if let Some(start) = self.start_time {
                return self.elapsed + start.elapsed();
            }
        }
        self.elapsed
    }

    /// Record a lap at current elapsed time with optional label.
    pub fn lap(&mut self, label: Option<String>) -> Result<(), TimerError> {
        if !self.running {
            return Err(TimerError(TimerErrorKind::NotRunning));
        }
        let idx = self.laps.len() + 1;
        let at_ms = self.elapsed().as_millis();
        self.laps.push(Lap { index: idx, at_ms, label });
        Ok(())
    }

    pub fn laps(&self) -> &[Lap] { &self.laps }
}

/// Format a Duration as HH:MM:SS.mmm (integer milliseconds)
pub fn format_duration(d: Duration) -> String {
    let ms = d.as_millis();
    let h = ms / 3_600_000;
    let m = (ms % 3_600_000) / 60_000;
    let s = (ms % 60_000) / 1_000;
    let mm = ms % 1_000;
    format!("{:02}:{:02}:{:02}.{:03}", h, m, s, mm)
}
