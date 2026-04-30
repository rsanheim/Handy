use log::info;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use crate::FILE_LOG_LEVEL;

static NEXT_TRACE_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug)]
pub struct PerfTrace {
    id: u64,
    started_at: Instant,
}

impl PerfTrace {
    pub fn new_if_enabled() -> Option<Self> {
        if perf_trace_env_enabled()
            || FILE_LOG_LEVEL.load(Ordering::Relaxed) >= log::LevelFilter::Trace as u8
        {
            Some(Self::new())
        } else {
            None
        }
    }

    pub fn new() -> Self {
        Self {
            id: NEXT_TRACE_ID.fetch_add(1, Ordering::Relaxed),
            started_at: Instant::now(),
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn elapsed_millis(&self) -> f64 {
        self.started_at.elapsed().as_secs_f64() * 1000.0
    }

    pub fn log_event(&self, event: &str) {
        info!(
            target: "perf.hot_path",
            "perf.hot_path trace_id={} event={} elapsed_ms={:.2}",
            self.id,
            event,
            self.elapsed_millis()
        );
    }

    pub fn log_detail(&self, event: &str, detail: impl std::fmt::Display) {
        info!(
            target: "perf.hot_path",
            "perf.hot_path trace_id={} event={} elapsed_ms={:.2} {}",
            self.id,
            event,
            self.elapsed_millis(),
            detail
        );
    }
}

fn perf_trace_env_enabled() -> bool {
    std::env::var("HANDY_PERF_TRACE")
        .map(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::PerfTrace;

    #[test]
    fn new_traces_get_distinct_ids() {
        let first = PerfTrace::new();
        let second = PerfTrace::new();

        assert_ne!(first.id(), second.id());
    }

    #[test]
    fn elapsed_millis_is_non_negative() {
        let trace = PerfTrace::new();

        assert!(trace.elapsed_millis() >= 0.0);
    }

    #[test]
    fn perf_trace_env_enabled_recognizes_truthy_values() {
        let current = std::env::var("HANDY_PERF_TRACE").ok();
        std::env::set_var("HANDY_PERF_TRACE", "yes");

        assert!(super::perf_trace_env_enabled());

        if let Some(value) = current {
            std::env::set_var("HANDY_PERF_TRACE", value);
        } else {
            std::env::remove_var("HANDY_PERF_TRACE");
        }
    }
}
