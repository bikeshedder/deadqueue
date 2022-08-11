//! # Deadqueue [![Latest Version](https://img.shields.io/crates/v/deadqueue.svg)](https://crates.io/crates/deadqueue) [![Build Status](https://travis-ci.org/bikeshedder/deadqueue.svg?branch=master)](https://travis-ci.org/bikeshedder/deadqueue)
//!
//! Deadqueue is a dead simple async queue with back pressure support.
//!
//! This crate provides three implementations:
//!
//! - Unlimited (`deadqueue::unlimited::Queue`)
//!   - Based on `crossbeam_queue::SegQueue`
//!   - Has unlimitied capacity and no back pressure on push
//!   - Enabled via the `unlimited` feature in your `Cargo.toml`
//!
//! - Resizable (`deadqueue::resizable::Queue`)
//!   - Based on `deadqueue::unlimited::Queue`
//!   - Has limited capacity with back pressure on push
//!   - Supports resizing
//!   - Enabled via the `resizable` feature in your `Cargo.toml`
//!
//! - Limited (`deadqueue::limited::Queue`)
//!   - Based on `crossbeam_queue::ArrayQueue`
//!   - Has limit capacity with back pressure on push
//!   - Does not support resizing
//!   - Enabled via the `limited` feature in your `Cargo.toml`
//!
//! ## Features
//!
//! | Feature | Description | Extra dependencies | Default |
//! | ------- | ----------- | ------------------ | ------- |
//! | `unlimited` | Enable unlimited queue implementation | – | yes |
//! | `resizable` | Enable resizable queue implementation | `deadqueue/unlimited` | yes |
//! | `limited` | Enable limited queue implementation | – | yes |
//!
//! ## Example
//!
//! ```rust
//! use std::sync::Arc;
//! use tokio::time::{sleep, Duration};
//!
//! const TASK_COUNT: usize = 1000;
//! const WORKER_COUNT: usize = 10;
//!
//! type TaskQueue = deadqueue::limited::Queue<usize>;
//!
//! #[tokio::main]
//! async fn main() {
//!     let queue = Arc::new(TaskQueue::new(TASK_COUNT));
//!     for i in 0..TASK_COUNT {
//!         queue.try_push(i).unwrap();
//!     }
//!     for worker in 0..WORKER_COUNT {
//!         let queue = queue.clone();
//!         tokio::spawn(async move {
//!             loop {
//!                 let task = queue.pop().await;
//!                 println!("worker[{}] processing task[{}] ...", worker, task);
//!             }
//!         });
//!     }
//!     while queue.len() > 0 {
//!         println!("Waiting for workers to finish...");
//!         sleep(Duration::from_millis(100)).await;
//!     }
//!     println!("All tasks done. :-)");
//! }
//! ```
//!
//! ## Reasons for yet another queue
//!
//! Deadqueue is by no means the only queue implementation available. It does things a little different and provides features that other implementations are lacking:
//!
//! - **Resizable queue.** Usually you have to pick between `limited` and `unlimited` queues. This crate features a `resizable` Queue which can be resized as needed. This is probably a big **unique selling point** of this crate.
//!
//! - **Introspection support.** The methods `.len()`, `.capacity()` and `.available()` provide access the current state of the queue.
//!
//! - **Fair scheduling.** Tasks calling `pop` will receive items in a first-come-first-serve fashion. This is mainly due to the use of `tokio::sync::Semaphore` which is fair by nature.
//!
//! - **One struct, not two.** The channels of `tokio`, `async_std` and `futures-intrusive` split the queue in two structs (`Sender` and `Receiver`) which makes the usage sligthly more complicated.
//!
//! - **Bring your own `Arc`.** Since there is no separation between `Sender` and `Receiver` there is also no need for an internal `Arc`. (All implementations that split the channel into a `Sender` and `Receiver` need some kind of `Arc` internally.)
//!
//! - **Fully concurrent access.** No need to wrap the `Receiver` part in a `Mutex`. All methods support concurrent accesswithout the need for an additional synchronization primitive.
//!
//! - **Support for `try__` methods.** The methods `try_push` and `try_pop` can be used to access the queue from non-blocking synchroneous code.
//!
//! ## Alternatives
//!
//! | Crate | Limitations | Documentation |
//! | --- | --- | --- |
//! | [`tokio`](https://crates.io/crates/tokio) | No resizable queue. No introspection support. Synchronization of `Receiver` needed. | [`tokio::sync::mpsc::channel`](https://docs.rs/tokio/latest/tokio/sync/mpsc/fn.channel.html), [`tokio::sync::mpsc::unbounded_channel`](https://docs.rs/tokio/latest/tokio/sync/mpsc/fn.unbounded_channel.html) |
//! | [`async-std`](https://crates.io/crates/async-std) | No resizable or unlimited queue. No introspection support. No `try_send` or `try_recv` methods. | [`async_std::sync::channel`](https://docs.rs/async-std/latest/async_std/sync/fn.channel.html) |
//! | [`futures`](https://crates.io/crates/futures) | No resizable queue. No introspection support. | [`futures::channel::mpsc::channel`](https://docs.rs/futures/0.3.1/futures/channel/mpsc/fn.channel.html), [`futures::channel::mpsc::unbounded`](https://docs.rs/futures/0.3.1/futures/channel/mpsc/fn.unbounded.html) |
//!
//! ## License
//!
//! Licensed under either of
//!
//! - Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0)>
//! - MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT)>
//!
//! at your option.
#![warn(missing_docs)]

use tokio::sync::watch;

mod atomic;

#[cfg(feature = "unlimited")]
pub mod unlimited;

#[cfg(feature = "resizable")]
pub mod resizable;

#[cfg(feature = "limited")]
pub mod limited;

/// Private type alias for notify_full and notify_empty
type Notifier = watch::Sender<()>;

/// Initialize the notify_full sender
fn new_notifier() -> Notifier {
    let (sender, _) = watch::channel(());
    sender
}
