//! Resizable queue implementation

use std::convert::TryInto;
use std::fmt::Debug;
use std::iter::FromIterator;
use std::sync::atomic::{AtomicUsize, Ordering};

use tokio::sync::{Mutex, Semaphore, watch};

use crate::unlimited::Queue as UnlimitedQueue;

/// Private type alias for notify_full
type FullNotifier = watch::Sender<()>;

/// Queue that is limited in size and supports resizing.
///
/// This queue implementation has the following characteristics:
///
///   - Resizable (`deadqueue::resizable::Queue`)
///   - Based on `deadqueue::unlimited::Queue`
///   - Has limited capacity with back pressure on push
///   - Supports resizing
///   - Enabled via the `resizable` feature in your `Cargo.toml`
pub struct Queue<T> {
    queue: UnlimitedQueue<T>,
    capacity: AtomicUsize,
    push_semaphore: Semaphore,
    resize_mutex: Mutex<()>,
    notify_full: FullNotifier,
}

impl<T> Queue<T> {
    /// Create new empty queue
    pub fn new(max_size: usize) -> Self {
        Self {
            queue: UnlimitedQueue::new(),
            capacity: AtomicUsize::new(max_size),
            push_semaphore: Semaphore::new(max_size),
            resize_mutex: Mutex::default(),
            notify_full: Self::new_notify_full(),
        }
    }
    /// Get an item from the queue. If the queue is currently empty
    /// this method blocks until an item is available.
    pub async fn pop(&self) -> T {
        let item = self.queue.pop().await;
        self.push_semaphore.add_permits(1);
        item
    }
    /// Try to get an item from the queue. If the queue is currently
    /// empty return None instead.
    pub fn try_pop(&self) -> Option<T> {
        let item = self.queue.try_pop();
        if item.is_some() {
            self.push_semaphore.add_permits(1);
        }
        item
    }
    /// Push an item into the queue
    pub async fn push(&self, item: T) {
        let permit = self.push_semaphore.acquire().await.unwrap();
        self.queue.push(item);
        permit.forget();
        self.notify_if_full();
    }
    /// Try to push an item to the queue. If the queue is currently
    /// full return the object as `Err<T>`.
    pub fn try_push(&self, item: T) -> Result<(), T> {
        match self.push_semaphore.try_acquire() {
            Ok(permit) => {
                self.queue.push(item);
                permit.forget();
                self.notify_if_full();
                Ok(())
            }
            Err(_) => Err(item),
        }
    }
    /// Get capacity of the queue (maximum number of items queue can store).
    pub fn capacity(&self) -> usize {
        self.capacity.load(Ordering::Relaxed)
    }
    /// Get current length of queue (number of items currently stored).
    pub fn len(&self) -> usize {
        self.queue.len()
    }
    /// Returns `true` if the queue is empty.
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
    /// Returns `true` if the queue is full.
    /// **Note:** this can give an incorrect result if a simultaneous push/pop
    /// and resize ocurr while this function is executing. try_push() is the
    /// reccomended and safer mechanism in most circumstances. This method
    /// is provided as a convenience API.
    pub fn is_full(&self) -> bool {
        self.len() >= self.capacity()
    }
    /// The number of available items in the queue. If there are no
    /// items in the queue this number can become negative and stores the
    /// number of futures waiting for an item.
    pub fn available(&self) -> isize {
        self.queue.available()
    }
    /// Initialize the notify_full sender
    fn new_notify_full() -> FullNotifier {
        let (sender, _) = watch::channel(());
        sender
    }
    /// Check if the queue is full and notify any waiters
    fn notify_if_full(&self) {
        if self.push_semaphore.available_permits() == 0 {
            self.notify_full.send_replace(());
        }
    }
    /// Await until the queue is full.
    pub async fn full(&self) {
        if self.len() == self.capacity() {
            return;
        }
        self.notify_full.subscribe().changed().await.unwrap();
    }
    /// Await until the queue is empty.
    pub async fn empty(&self) {
        loop {
            let capacity = self.capacity().try_into().unwrap();
            let _permit = self.push_semaphore.acquire_many(capacity).await.unwrap();
            // Check that queue is empty in case it has been resized
            // larger such that acquire_many succeeded even with items
            // still in the queue
            if self.is_empty() {
                break;
            }
            // _permit dropped at end of loop
        }
    }
    /// Resize queue. This increases or decreases the queue
    /// capacity accordingly.
    ///
    /// **Note:** Increasing the capacity of a queue happens without
    /// blocking unless a resize operation is already in progress.
    /// Decreasing the capacity can block if there are futures waiting to
    /// push items to the queue.
    pub async fn resize(&self, target_capacity: usize) {
        let _guard = self.resize_mutex.lock().await;
        match target_capacity.cmp(&self.capacity()) {
            std::cmp::Ordering::Greater => {
                let diff = target_capacity - self.capacity();
                self.capacity.fetch_add(diff, Ordering::Relaxed);
                self.push_semaphore.add_permits(diff);
            }
            std::cmp::Ordering::Less => {
                // Shrinking the queue is a bit more involved
                // as there are two cases that need to be covered.
                for _ in target_capacity..self.capacity() {
                    tokio::select! {
                        biased;
                        // If there are push permits available consume
                        // them first making sure no new items are pushed
                        // to the queue.
                        push_permit = self.push_semaphore.acquire() => {
                            push_permit.unwrap().forget();
                        }
                        // If the queue contains more elements than the
                        // target capacity those need to be removed from
                        // the queue.
                        // Important: Call `self.queue.pop` and not
                        // `self.pop` as the former would add permits to
                        // the `push_semaphore` which we don't want to
                        // happen since the queue is being shrinked.
                        _ = self.queue.pop() => {}
                    };
                    self.capacity.fetch_sub(1, Ordering::Relaxed);
                }
                // Currently this is a no-op because this resize() function
                // borrows mutably, so there can be no callers awaiting
                // `self.full()`, because they would hold an immutable reference.
                //
                // The code in self.resize() doesn't require mutability, though.
                // If the API changes in a future version, this call should remain
                // here so any callers awaiting `self.full()` will be notified
                // if the queue is resized smaller such that it becomes full.
                self.notify_if_full();
            }
            _ => {}
        }
    }
}

impl<T> Debug for Queue<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Queue")
            .field("queue", &self.queue)
            .field("capacity", &self.capacity)
            .field("push_semaphore", &self.push_semaphore)
            .field("resize_mutex", &self.resize_mutex)
            .finish()
    }
}

impl<T> FromIterator<T> for Queue<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let queue = UnlimitedQueue::from_iter(iter);
        let len = queue.len();
        Self {
            queue,
            capacity: len.try_into().unwrap(),
            push_semaphore: Semaphore::new(0),
            resize_mutex: Mutex::default(),
            notify_full: Self::new_notify_full(),
        }
    }
}
