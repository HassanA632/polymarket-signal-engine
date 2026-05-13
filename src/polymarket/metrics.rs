use std::time::Duration;

use crate::polymarket::signals::MarketSignal;

#[derive(Debug, Default)]
pub struct LatencyMetrics {
    message_count: u64,
    total_latency: Duration,
    min_latency: Option<Duration>,
    max_latency: Option<Duration>,
    latencies: Vec<Duration>,
}

impl LatencyMetrics {
    pub fn record(&mut self, latency: Duration) {
        self.message_count += 1;
        self.total_latency += latency;
        self.latencies.push(latency);

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
            "METRICS messages={} avg={} min={} max={} p50={} p95={} p99={}",
            self.message_count,
            format_duration(avg_latency),
            format_duration(self.min_latency.unwrap_or_default()),
            format_duration(self.max_latency.unwrap_or_default()),
            format_duration(self.percentile(50.0)),
            format_duration(self.percentile(95.0)),
            format_duration(self.percentile(99.0)),
        );
    }

    fn average_latency(&self) -> Duration {
        if self.message_count == 0 {
            return Duration::ZERO;
        }

        Duration::from_nanos((self.total_latency.as_nanos() / self.message_count as u128) as u64)
    }

    fn percentile(&self, percentile: f64) -> Duration {
        if self.latencies.is_empty() {
            return Duration::ZERO;
        }

        let mut sorted_latencies = self.latencies.clone();
        sorted_latencies.sort();

        let rank = (percentile / 100.0) * (sorted_latencies.len().saturating_sub(1) as f64);
        let index = rank.ceil() as usize;

        sorted_latencies[index]
    }
}

#[derive(Debug, Default)]
pub struct SignalMetrics {
    total: u64,
    tight_spread: u64,
    spread_tightened: u64,
    price_move: u64,
    large_trade: u64,
}

impl SignalMetrics {
    pub fn record(&mut self, signal: &MarketSignal) {
        self.total += 1;

        match signal {
            MarketSignal::TightSpread { .. } => {
                self.tight_spread += 1;
            }
            MarketSignal::SpreadTightened { .. } => {
                self.spread_tightened += 1;
            }
            MarketSignal::PriceMoveUp { .. } | MarketSignal::PriceMoveDown { .. } => {
                self.price_move += 1;
            }
            MarketSignal::LargeTrade { .. } => {
                self.large_trade += 1;
            }
        }
    }

    pub fn display_summary(&self) {
        println!(
            "SIGNAL_METRICS total={} tight_spread={} spread_tightened={} price_move={} large_trade={}",
            self.total, self.tight_spread, self.spread_tightened, self.price_move, self.large_trade
        );
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

    #[test]
    fn calculates_p50_latency() {
        let mut metrics = LatencyMetrics::default();

        metrics.record(Duration::from_micros(100));
        metrics.record(Duration::from_micros(200));
        metrics.record(Duration::from_micros(300));

        assert_eq!(metrics.percentile(50.0), Duration::from_micros(200));
    }

    #[test]
    fn calculates_p95_latency() {
        let mut metrics = LatencyMetrics::default();

        for value in 1..=100 {
            metrics.record(Duration::from_micros(value));
        }

        assert_eq!(metrics.percentile(95.0), Duration::from_micros(96));
    }

    #[test]
    fn calculates_p99_latency() {
        let mut metrics = LatencyMetrics::default();

        for value in 1..=100 {
            metrics.record(Duration::from_micros(value));
        }

        assert_eq!(metrics.percentile(99.0), Duration::from_micros(100));
    }

    #[test]
    fn percentile_returns_zero_when_empty() {
        let metrics = LatencyMetrics::default();

        assert_eq!(metrics.percentile(95.0), Duration::ZERO);
    }

    #[test]
    fn records_tight_spread_signal() {
        let mut metrics = SignalMetrics::default();

        let signal = MarketSignal::TightSpread {
            token_id: "token-1".to_string(),
            best_bid: "0.42".to_string(),
            best_ask: "0.43".to_string(),
            spread: "0.01".to_string(),
        };

        metrics.record(&signal);

        assert_eq!(metrics.total, 1);
        assert_eq!(metrics.tight_spread, 1);
        assert_eq!(metrics.spread_tightened, 0);
        assert_eq!(metrics.price_move, 0);
        assert_eq!(metrics.large_trade, 0);
    }

    #[test]
    fn combines_price_move_up_and_down() {
        let mut metrics = SignalMetrics::default();

        let up = MarketSignal::PriceMoveUp {
            token_id: "token-1".to_string(),
            previous_bid: "0.40".to_string(),
            current_bid: "0.43".to_string(),
            change: "0.0300".to_string(),
        };

        let down = MarketSignal::PriceMoveDown {
            token_id: "token-1".to_string(),
            previous_bid: "0.43".to_string(),
            current_bid: "0.40".to_string(),
            change: "-0.0300".to_string(),
        };

        metrics.record(&up);
        metrics.record(&down);

        assert_eq!(metrics.total, 2);
        assert_eq!(metrics.price_move, 2);
    }

    #[test]
    fn records_all_signal_categories() {
        let mut metrics = SignalMetrics::default();

        metrics.record(&MarketSignal::TightSpread {
            token_id: "token-1".to_string(),
            best_bid: "0.42".to_string(),
            best_ask: "0.43".to_string(),
            spread: "0.01".to_string(),
        });

        metrics.record(&MarketSignal::SpreadTightened {
            token_id: "token-1".to_string(),
            previous_spread: "0.04".to_string(),
            current_spread: "0.02".to_string(),
        });

        metrics.record(&MarketSignal::PriceMoveUp {
            token_id: "token-1".to_string(),
            previous_bid: "0.40".to_string(),
            current_bid: "0.43".to_string(),
            change: "0.0300".to_string(),
        });

        metrics.record(&MarketSignal::LargeTrade {
            token_id: "token-1".to_string(),
            side: "BUY".to_string(),
            price: "0.43".to_string(),
            size: "750".to_string(),
        });

        assert_eq!(metrics.total, 4);
        assert_eq!(metrics.tight_spread, 1);
        assert_eq!(metrics.spread_tightened, 1);
        assert_eq!(metrics.price_move, 1);
        assert_eq!(metrics.large_trade, 1);
    }
}
