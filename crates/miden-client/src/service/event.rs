use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use miden_client_core::sync::SyncSummary;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HandlerId(u64);

impl HandlerId {
    pub fn next(counter: &AtomicU64) -> Self {
        Self(counter.fetch_add(1, Ordering::Relaxed))
    }
}

#[derive(Debug, Clone)]
pub struct SyncEvent {
    pub summary: SyncSummary,
    pub timestamp: Instant,
}

impl SyncEvent {
    pub fn new(summary: SyncSummary) -> Self {
        Self { summary, timestamp: Instant::now() }
    }
}
