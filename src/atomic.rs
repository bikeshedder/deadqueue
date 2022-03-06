use std::sync::atomic::{AtomicIsize, Ordering};

#[derive(Debug)]
pub struct Available(AtomicIsize);

impl Available {
    pub fn new(value: isize) -> Self {
        Self(AtomicIsize::new(value))
    }
    pub fn sub(&self) -> TransactionSub {
        self.0.fetch_sub(1, Ordering::Relaxed);
        TransactionSub(&self.0)
    }
    pub fn add(&self) {
        self.0.fetch_add(1, Ordering::Relaxed);
    }
    pub fn get(&self) -> isize {
        self.0.load(Ordering::Relaxed)
    }
}

#[must_use]
pub struct TransactionSub<'a>(&'a AtomicIsize);

impl<'a> TransactionSub<'a> {
    pub fn commit(self) {
        std::mem::forget(self);
    }
}

impl<'a> Drop for TransactionSub<'a> {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::Relaxed);
    }
}
