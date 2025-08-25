use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StopwatchErrorKind {
    AlreadyRunning,
    NotRunning,
    Invalid,
}

#[derive(Debug)]
pub struct StopwatchError(pub StopwatchErrorKind);

/// Simple stopwatch for measuring elapsed time.
pub struct Stopwatch {
    pub start_time: Option<Instant>,
    pub elapsed: Duration,
    pub running: bool,
}

impl Default for Stopwatch {
    fn default() -> Self {
        Self::new()
    }
}

impl Stopwatch {
    pub fn new() -> Self {
        Self { start_time: None, elapsed: Duration::ZERO, running: false }
    }

    /// Start the stopwatch if not already running.
    pub fn start(&mut self) -> Result<(), StopwatchError> {
        if self.running {
            return Err(StopwatchError(StopwatchErrorKind::AlreadyRunning));
        }
        self.start_time = Some(Instant::now());
        self.running = true;
        Ok(())
    }

    /// Stop the stopwatch and accumulate elapsed time.
    pub fn stop(&mut self) -> Result<(), StopwatchError> {
        if !self.running {
            return Err(StopwatchError(StopwatchErrorKind::NotRunning));
        }
        if let Some(start) = self.start_time.take() {
            self.elapsed += start.elapsed();
        }
        self.running = false;
        Ok(())
    }

    /// Reset the stopwatch to zero and stop it.
    pub fn reset(&mut self) {
        self.start_time = None;
        self.elapsed = Duration::ZERO;
        self.running = false;
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
