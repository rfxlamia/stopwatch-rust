use stopwatch_rust::{format_duration, Stopwatch, StopwatchErrorKind};

#[test]
fn stop_without_start_should_error() {
    let mut sw = Stopwatch::new();
    let err = sw.stop().unwrap_err();
    assert_eq!(err.0, StopwatchErrorKind::NotRunning);
}

#[test]
fn double_start_should_error() {
    let mut sw = Stopwatch::new();
    sw.start().unwrap();
    let err = sw.start().unwrap_err();
    assert_eq!(err.0, StopwatchErrorKind::AlreadyRunning);
}

#[test]
fn reset_while_running_sets_zero_and_not_running() {
    let mut sw = Stopwatch::new();
    sw.start().unwrap();
    sw.reset();
    assert_eq!(sw.elapsed().as_millis(), 0);
}

#[test]
fn format_is_stable_in_ms() {
    assert_eq!(format_duration(std::time::Duration::from_millis(1)), "00:00:00.001");
    assert_eq!(format_duration(std::time::Duration::from_millis(10)), "00:00:00.010");
}
