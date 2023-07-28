//! Limited queue implementation
//!

use std::{convert::TryInto, fmt::Debug};

use crossbeam_queue::ArrayQueue;
use tokio::sync::Semaphore;

use crate::atomic::Available;

/// Queue that is limited in size and does not support resizing.
///
/// This queue implementation has the following characteristics:
///
///   - Based on `crossbeam_queue::ArrayQueue`
///   - Has limit capacity with back pressure on push
///   - Does not support resizing
///   - Enabled via the `limited` feature in your `Cargo.toml`
pub struct Queue<T> {
    queue: ArrayQueue<T>,
    push_semaphore: Semaphore,
    pop_semaphore: Semaphore,
    available: Available,
}

impl<T> Debug for Queue<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Queue")
            .field("queue", &self.queue)
            .field("push_semaphore", &self.push_semaphore)
            .field("pop_semaphore", &self.pop_semaphore)
            .field("available", &self.available)
            .finish()
    }
}

impl<T> Queue<T> {
    /// Create new empty queue
    pub fn new(max_size: usize) -> Self {
        Self {
            queue: ArrayQueue::new(max_size),
            push_semaphore: Semaphore::new(max_size),
            pop_semaphore: Semaphore::new(0),
            available: Available::new(0),
        }
    }
    /// Get an item from the queue. If the queue is currently empty
    /// this method blocks until an item is available.
    pub async fn pop(&self) -> T {
        let txn = self.available.sub();
        let permit = self.pop_semaphore.acquire().await.unwrap();
        let item = self.queue.pop().unwrap();
        txn.commit();
        permit.forget();
        self.push_semaphore.add_permits(1);
        item
    }
    /// Try to get an item from the queue. If the queue is currently
    /// empty return None instead.
    pub fn try_pop(&self) -> Option<T> {
        let txn = self.available.sub();
        let permit = self.pop_semaphore.try_acquire().ok()?;
        let item = Some(self.queue.pop().unwrap());
        txn.commit();
        permit.forget();
        self.push_semaphore.add_permits(1);
        item
    }
    /// Push an item into the queue
    pub async fn push(&self, item: T) {
        let permit = self.push_semaphore.acquire().await.unwrap();
        self.available.add();
        self.queue.push(item).ok().unwrap();
        permit.forget();
        self.pop_semaphore.add_permits(1);
    }
    /// Try to push an item into the queue. If the queue is full
    /// the item is returned as `Err<T>`.
    pub fn try_push(&self, item: T) -> Result<(), T> {
        match self.push_semaphore.try_acquire() {
            Ok(permit) => {
                self.queue.push(item).ok().unwrap();
                permit.forget();
                self.pop_semaphore.add_permits(1);
                Ok(())
            }
            Err(_) => Err(item),
        }
    }
    /// Get capacity of the queue (maximum number of items queue can store)
    pub fn capacity(&self) -> usize {
        self.queue.capacity()
    }
    /// Get current length of queue (number of items currently stored)
    pub fn len(&self) -> usize {
        self.queue.len()
    }
    /// Returns `true` if the queue is empty.
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
    /// Returns `true` if the queue is full.
    pub fn is_full(&self) -> bool {
        self.queue.is_full()
    }
    /// The number of available items in the queue. If there are no
    /// items in the queue this number can become negative and stores the
    /// number of futures waiting for an item.
    pub fn available(&self) -> isize {
        self.available.get()
    }
    /// Await until the queue is full.
    pub async fn full(&self) {
        let capacity = self.capacity().try_into().unwrap();
        let _ = self.pop_semaphore.acquire_many(capacity).await.unwrap();
    }
    /// Await until the queue is empty.
    pub async fn empty(&self) {
        let capacity = self.capacity().try_into().unwrap();
        let _ = self.push_semaphore.acquire_many(capacity).await.unwrap();
    }
}

impl<T, I> From<I> for Queue<T>
where
    I: IntoIterator<Item = T>,
    <I as IntoIterator>::IntoIter: ExactSizeIterator,
{
    /// Create new queue from the given exact size iterator of objects.
    fn from(iter: I) -> Self {
        let iter = iter.into_iter();
        let size = iter.len();
        let queue = ArrayQueue::new(size);
        for obj in iter {
            queue.push(obj).ok().unwrap();
        }
        Queue {
            queue: ArrayQueue::new(size),
            push_semaphore: Semaphore::new(0),
            pop_semaphore: Semaphore::new(size),
            available: Available::new(size.try_into().unwrap()),
        }
    }
}
