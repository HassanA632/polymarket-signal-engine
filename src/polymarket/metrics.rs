use std::time::Duration;

#[derive(Debug, Default)]
pub struct LatencyMetrics {
    message_count: u64,
    total_latency: Duration,
    min_latency: Option<Duration>,
    max_latency: Option<Duration>,
}

impl LatencyMetrics {
    pub fn record(&mut self, latency: Duration) {
        self.message_count += 1;
        self.total_latency += latency;

        self.min_latency = Some(match self.min_latency {
            Some(current_min) => current_min.min(latency),
            None => latency,
        });

        self.max_latency = Some(match self.max_latency {
            Some(current_max) => current_max.max(latency),
            None => latency,
        });
    }

    pub fn message_count(&self) -> u64 {
        self.message_count
    }

    pub fn should_report(&self, interval: u64) -> bool {
        self.message_count > 0 && self.message_count % interval == 0
    }

    pub fn display_summary(&self) {
        let avg_latency = self.average_latency();

        println!(
            "METRICS messages={} avg={} min={} max={}",
            self.message_count,
            format_duration(avg_latency),
            format_duration(self.min_latency.unwrap_or_default()),
            format_duration(self.max_latency.unwrap_or_default()),
        );
    }

    fn average_latency(&self) -> Duration {
        if self.message_count == 0 {
            return Duration::ZERO;
        }

        Duration::from_nanos((self.total_latency.as_nanos() / self.message_count as u128) as u64)
    }
}

fn format_duration(duration: Duration) -> String {
    if duration.as_micros() < 1_000 {
        format!("{}µs", duration.as_micros())
    } else {
        format!("{:.2}ms", duration.as_secs_f64() * 1_000.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_message_count() {
        let mut metrics = LatencyMetrics::default();

        metrics.record(Duration::from_micros(100));
        metrics.record(Duration::from_micros(200));

        assert_eq!(metrics.message_count(), 2);
    }

    #[test]
    fn reports_at_interval() {
        let mut metrics = LatencyMetrics::default();

        for _ in 0..9 {
            metrics.record(Duration::from_micros(100));
        }

        assert!(!metrics.should_report(10));

        metrics.record(Duration::from_micros(100));

        assert!(metrics.should_report(10));
    }

    #[test]
    fn does_not_report_when_empty() {
        let metrics = LatencyMetrics::default();

        assert!(!metrics.should_report(10));
    }

    #[test]
    fn handles_zero_messages_average() {
        let metrics = LatencyMetrics::default();

        assert_eq!(metrics.average_latency(), Duration::ZERO);
    }

    #[test]
    fn calculates_average_latency() {
        let mut metrics = LatencyMetrics::default();

        metrics.record(Duration::from_micros(100));
        metrics.record(Duration::from_micros(300));

        assert_eq!(metrics.average_latency(), Duration::from_micros(200));
    }
}
