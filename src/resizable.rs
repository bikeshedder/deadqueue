//! Resizable queue implementation

use std::convert::TryInto;
use std::iter::FromIterator;
use std::sync::atomic::{AtomicUsize, Ordering};

use tokio::sync::{Mutex, Semaphore};

use crate::unlimited::Queue as UnlimitedQueue;

/// Queue that is limited in size and supports resizing.
///
/// This queue implementation has the following characteristics:
///
///   - Resizable (`deadqueue::resizable::Queue`)
///   - Based on `deadqueue::unlimited::Queue`
///   - Has limited capacity with back pressure on push
///   - Supports resizing
///   - Enabled via the `resizable` feature in your `Cargo.toml`
#[derive(Debug)]
pub struct Queue<T> {
    base: UnlimitedQueue<T>,
    capacity: AtomicUsize,
    push_semaphore: Semaphore,
    resize_mutex: Mutex<()>,
}

impl<T> Queue<T> {
    /// Create new empty queue
    pub fn new(max_size: usize) -> Self {
        Self {
            base: UnlimitedQueue::new(),
            capacity: AtomicUsize::new(max_size),
            push_semaphore: Semaphore::new(max_size),
            resize_mutex: Mutex::default(),
        }
    }
    /// Get an item from the queue. If the queue is currently empty
    /// this method blocks until an item is available.
    pub async fn pop(&self) -> T {
        let item = self.base.pop().await;
        self.push_semaphore.add_permits(1);
        item
    }
    /// Try to get an item from the queue. If the queue is currently
    /// empty return None instead.
    pub fn try_pop(&self) -> Option<T> {
        let item = self.base.try_pop();
        if item.is_some() {
            self.push_semaphore.add_permits(1);
        }
        item
    }
    /// Push an item into the queue
    pub async fn push(&self, item: T) {
        let permit = self.push_semaphore.acquire().await.unwrap();
        self.base.push(item);
        permit.forget();
    }
    /// Try to push an item to the queue. If the queue is currently
    /// full return the object as `Err<T>`.
    pub fn try_push(&self, item: T) -> Result<(), T> {
        match self.push_semaphore.try_acquire() {
            Ok(permit) => {
                self.base.push(item);
                permit.forget();
                Ok(())
            }
            Err(_) => Err(item),
        }
    }
    /// Get capacity of the queue
    pub fn capacity(&self) -> usize {
        self.capacity.load(Ordering::Relaxed)
    }
    /// Get current length of queue
    pub fn len(&self) -> usize {
        self.base.len()
    }
    /// The number of available items in the queue. If there are no
    /// items in the queue this number can become negative and stores the
    /// number of futures waiting for an item.
    pub fn available(&self) -> isize {
        self.base.available()
    }
    /// Resize queue. This increases or decreases the queue
    /// capacity accordingly.
    ///
    /// **Note:** Increasing the capacity of a queue happens without
    /// blocking unless a resize operation is already in progress.
    /// Decreasing the capacity can block if there are futures waiting to
    /// push items to the queue.
    pub async fn resize(&mut self, new_max_size: usize) {
        let _guard = self.resize_mutex.lock().await;
        if new_max_size > self.capacity() {
            let diff = new_max_size - self.capacity();
            self.capacity.fetch_add(diff, Ordering::Relaxed);
            self.push_semaphore.add_permits(diff);
        } else if new_max_size < self.capacity() {
            for _ in self.capacity()..new_max_size {
                let permit = self.push_semaphore.acquire().await.unwrap();
                self.capacity.fetch_sub(1, Ordering::Relaxed);
                permit.forget();
                self.base.pop().await;
            }
        }
    }
}

impl<T> FromIterator<T> for Queue<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let base = UnlimitedQueue::from_iter(iter);
        let len = base.len();
        Self {
            base,
            capacity: len.try_into().unwrap(),
            push_semaphore: Semaphore::new(0),
            resize_mutex: Mutex::default(),
        }
    }
}
