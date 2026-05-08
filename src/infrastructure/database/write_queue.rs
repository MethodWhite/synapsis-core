//! Database Write Queue - Ordered, batched, non-saturating
//!
//! Prevents DB saturation from concurrent multi-agent writes.

use rusqlite::Connection;
use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Instant;

/// A pending write operation
pub struct WriteOp {
    pub sql: String,
    pub timestamp: Instant,
}

/// Write queue with backpressure — prevents DB saturation
pub struct WriteQueue {
    inner: Arc<(Mutex<VecDeque<WriteOp>>, Condvar)>,
    max_queue_size: usize,
    batch_size: usize,
}

impl WriteQueue {
    pub fn new() -> Self {
        Self {
            inner: Arc::new((Mutex::new(VecDeque::new()), Condvar::new())),
            max_queue_size: 1000,
            batch_size: 50,
        }
    }

    /// Submit a write. Drops if queue is full.
    pub fn submit(&self, op: WriteOp) -> bool {
        let (lock, cvar) = &*self.inner;
        let mut queued = false;

        {
            let mut queue = lock.lock().unwrap_or_else(|e| e.into_inner());
            if queue.len() < self.max_queue_size {
                queue.push_back(op);
                queued = true;
            }
        }

        if queued {
            cvar.notify_one();
        }
        queued
    }

    /// Execute pending writes as a batch in a transaction
    pub fn flush(&self, conn: &Connection) -> usize {
        let (lock, _cvar) = &*self.inner;
        let mut queue = lock.lock().unwrap_or_else(|e| e.into_inner());

        if queue.is_empty() {
            return 0;
        }

        let drain_count = std::cmp::min(queue.len(), self.batch_size);
        let batch: Vec<WriteOp> = queue.drain(..drain_count).collect();
        drop(queue);

        if batch.is_empty() {
            return 0;
        }

        let count = batch.len();
        if conn.execute_batch("BEGIN IMMEDIATE").is_err() {
            return 0;
        }

        for op in &batch {
            let _ = conn.execute_batch(&op.sql);
        }

        let _ = conn.execute_batch("COMMIT");
        count
    }

    pub fn pending_count(&self) -> usize {
        let (lock, _) = &*self.inner;
        lock.lock().unwrap_or_else(|e| e.into_inner()).len()
    }

    pub fn is_saturated(&self) -> bool {
        let (lock, _) = &*self.inner;
        lock.lock().unwrap_or_else(|e| e.into_inner()).len() >= self.max_queue_size
    }
}

impl Default for WriteQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for WriteQueue {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            max_queue_size: self.max_queue_size,
            batch_size: self.batch_size,
        }
    }
}
