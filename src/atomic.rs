use std::sync::atomic::{AtomicIsize, Ordering};

#[derive(Debug)]
pub struct Available(AtomicIsize);

impl Available {
    pub fn new(value: isize) -> Self {
        Self(AtomicIsize::new(value))
    }
    pub fn sub(&self) -> (TransactionSub, isize) {
        let new_len = self.0.fetch_sub(1, Ordering::Relaxed) - 1;
        (TransactionSub(&self.0), new_len)
    }
    pub fn add(&self) -> isize {
        self.0.fetch_add(1, Ordering::Relaxed) + 1
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
