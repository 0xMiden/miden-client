#![allow(clippy::cast_possible_truncation)]

use std::time::{Duration, Instant};

/// Result of a single benchmark
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    /// Name of the benchmark
    pub name: String,
    /// Individual iteration durations
    pub iterations: Vec<Duration>,
    /// Size of output (for serialization benchmarks)
    pub output_size: Option<usize>,
    /// Additional metadata
    pub metadata: Option<String>,
}

impl BenchmarkResult {
    /// Creates a new benchmark result
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            iterations: Vec::new(),
            output_size: None,
            metadata: None,
        }
    }

    /// Adds an iteration duration
    pub fn add_iteration(&mut self, duration: Duration) {
        self.iterations.push(duration);
    }

    /// Sets the output size
    pub fn with_output_size(mut self, size: usize) -> Self {
        self.output_size = Some(size);
        self
    }

    /// Sets metadata
    pub fn with_metadata(mut self, metadata: impl Into<String>) -> Self {
        self.metadata = Some(metadata.into());
        self
    }

    /// Returns the mean duration
    pub fn mean(&self) -> Duration {
        if self.iterations.is_empty() {
            return Duration::ZERO;
        }
        let total: Duration = self.iterations.iter().sum();
        total / self.iterations.len() as u32
    }

    /// Returns the minimum duration
    pub fn min(&self) -> Duration {
        self.iterations.iter().min().copied().unwrap_or(Duration::ZERO)
    }

    /// Returns the maximum duration
    pub fn max(&self) -> Duration {
        self.iterations.iter().max().copied().unwrap_or(Duration::ZERO)
    }
}

/// Measures the execution time of an async function
pub async fn measure_time_async<F, Fut, T>(f: F) -> (T, Duration)
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = T>,
{
    let start = Instant::now();
    let result = f().await;
    let duration = start.elapsed();
    (result, duration)
}
