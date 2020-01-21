# Deadqueue [![Latest Version](https://img.shields.io/crates/v/deadqueue.svg)](https://crates.io/crates/deadqueue) [![Build Status](https://travis-ci.org/bikeshedder/deadqueue.svg?branch=master)](https://travis-ci.org/bikeshedder/deadqueue)

Deadqueue is a dead simple async queue with back pressure support.

This crate provides three implementations:

- Unlimited (`deadqueue::unlimited::Queue`)
  - Based on `crossbeam_queue::SegQueue`
  - Has unlimitied capacity and no back pressure on push
  - Enabled via the `unlimited` feature in your `Cargo.toml`

- Resizable (`deadqueue::resizable::Queue`)
  - Based on `deadqueue::unlimited::Queue`
  - Has limited capacity with back pressure on push
  - Supports resizing
  - Enabled via the `resizable` feature in your `Cargo.toml`

- Limited (`deadqueue::limited::Queue`)
  - Based on `crossbeam_queue::ArrayQueue`
  - Has limit capacity with back pressure on push
  - Does not support resizing
  - Enabled via the `limited` feature in your `Cargo.toml`

## Features

| Feature | Description | Extra dependencies | Default |
| ------- | ----------- | ------------------ | ------- |
| `unlimited` | Enable unlimited queue implementation | – | yes |
| `resizable` | Enable resizable queue implementation | `deadqueue/unlimited` | yes |
| `limited` | Enable limited queue implementation | – | yes |

## Example

```rust,ignore
use std::sync::Arc;

const TASK_COUNT: usize = 1000;
const WORKER_COUNT: usize = 10;

type TaskQueue = deadqueue::limited::Queue<usize>;

#[tokio::main]
async fn main() {
    let queue = Arc::new(TaskQueue::new(10));
    for i in 0..TASK_COUNT {
        queue.push(i);
    }
    let mut futures = Vec::new();
    for _ in 0..WORKER_COUNT {
        let queue = queue.clone();
        futures.push(tokio::spawn(async move {
            let task = queue.pop().await;
            assert!(task > 1);
        }));
    }
    for future in futures {
        future.await;
    }
    assert_eq!(queue.len(), 0);
}
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0)>
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT)>

at your option.
