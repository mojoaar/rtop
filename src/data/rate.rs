use std::time::Duration;

pub fn rate_per_sec(current: u64, previous: u64, elapsed: Duration) -> f64 {
    let secs = elapsed.as_secs_f64();
    if secs <= 0.0 || current < previous {
        0.0
    } else {
        (current - previous) as f64 / secs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn computes_bytes_per_sec() {
        assert_eq!(rate_per_sec(1024, 0, Duration::from_secs(1)), 1024.0);
    }

    #[test]
    fn counter_reset_yields_zero() {
        assert_eq!(rate_per_sec(10, 9000, Duration::from_secs(1)), 0.0);
    }

    #[test]
    fn zero_elapsed_yields_zero() {
        assert_eq!(rate_per_sec(100, 0, Duration::from_millis(0)), 0.0);
    }
}
