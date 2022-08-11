//! Unlimited queue implementation

use std::convert::TryInto;
use std::fmt::Debug;
use std::iter::FromIterator;

use crossbeam_queue::SegQueue;
use tokio::sync::Semaphore;

use crate::atomic::Available;
use crate::Notifier;

/// Queue that is unlimited in size.
///
/// This queue implementation has the following characteristics:
///
///   - Based on `crossbeam_queue::SegQueue`
///   - Has unlimitied capacity and no back pressure on push
///   - Enabled via the `unlimited` feature in your `Cargo.toml`
pub struct Queue<T> {
    queue: SegQueue<T>,
    semaphore: Semaphore,
    available: Available,
    notify_empty: Notifier,
}

impl<T> Queue<T> {
    /// Create new empty queue
    pub fn new() -> Self {
        Self::default()
    }

    /// Get an item from the queue. If the queue is currently empty
    /// this method blocks until an item is available.
    pub async fn pop(&self) -> T {
        let txn = self.available.sub();
        let permit = self.semaphore.acquire().await.unwrap();
        txn.commit();
        //txn.commit();
        // FIXME must be used
        let item = self.queue.pop().unwrap();
        permit.forget();
        self.notify_if_empty();
        item
    }
    /// Try to get an item from the queue. If the queue is currently
    /// empty return None instead.
    pub fn try_pop(&self) -> Option<T> {
        let txn = self.available.sub();
        let permit = self.semaphore.try_acquire().ok()?;
        let item = self.queue.pop().unwrap();
        txn.commit();
        permit.forget();
        self.notify_if_empty();
        Some(item)
    }
    /// Push an item into the queue
    pub fn push(&self, item: T) {
        self.queue.push(item);
        self.semaphore.add_permits(1);
        self.available.add();
    }
    /// Get current length of queue (number of items currently stored).
    pub fn len(&self) -> usize {
        self.queue.len()
    }
    /// Returns `true` if the queue is empty.
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
    /// Get available count. This is the difference between the current
    /// queue length and the number of tasks waiting for an item of the
    /// queue.
    pub fn available(&self) -> isize {
        self.available.get()
    }
    /// Notify any callers awaiting empty()
    fn notify_if_empty(&self) {
        if self.is_empty() {
            self.notify_empty.send_replace(());
        }
    }
    /// Await until the queue is empty
    pub async fn wait_empty(&self) {
        if self.is_empty() {
            return;
        }
        self.notify_empty.subscribe().changed().await.unwrap()
    }
}

impl<T> Debug for Queue<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Queue")
            .field("queue", &self.queue)
            .field("semaphore", &self.semaphore)
            .field("available", &self.available)
            .field("empty", &self.notify_empty)
            .finish()
    }
}

impl<T> Default for Queue<T> {
    fn default() -> Self {
        Self {
            queue: SegQueue::new(),
            semaphore: Semaphore::new(0),
            available: Available::new(0),
            notify_empty: crate::new_notifier(),
        }
    }
}

impl<T> FromIterator<T> for Queue<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let queue = SegQueue::new();
        for item in iter {
            queue.push(item);
        }
        let size = queue.len();
        Self {
            queue,
            semaphore: Semaphore::new(size),
            available: Available::new(size.try_into().unwrap()),
            ..Self::default()
        }
    }
}
