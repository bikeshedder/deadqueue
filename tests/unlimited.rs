#[cfg(feature = "unlimited")]
mod tests {

    use std::iter::FromIterator;
    use std::sync::Arc;

    use deadqueue::unlimited::Queue;

    #[tokio::test]
    async fn test_basics() {
        let queue: Queue<usize> = Queue::from_iter(vec![1, 2, 3, 4, 5]);
        for _ in 0..5 {
            assert!(queue.try_pop().is_some());
        }
        assert!(queue.try_pop().is_none());
    }

    #[tokio::test]
    async fn test_pop() {
        let queue: Queue<usize> = Queue::from_iter(vec![1, 2, 3, 4, 5]);
        for i in 0..5 {
            assert_eq!(queue.len(), 5 - i);
            queue.pop().await;
        }
        assert_eq!(queue.len(), 0);
        assert_eq!(queue.available(), 0);
    }

    #[tokio::test]
    async fn test_parallel() {
        let queue: Arc<Queue<usize>> = Arc::new(Queue::new());
        let mut futures = Vec::new();
        for i in 0..10000usize {
            queue.push(i);
        }
        assert_eq!(queue.len(), 10000);
        assert_eq!(queue.available(), 10000);
        for _ in 0..100usize {
            let queue = queue.clone();
            futures.push(tokio::spawn(async move {
                for _ in 0..100usize {
                    queue.pop().await;
                }
            }));
        }
        for future in futures {
            future.await.unwrap();
        }
        assert_eq!(queue.len(), 0);
        assert_eq!(queue.available(), 0);
    }

    #[tokio::test]
    async fn test_empty() {
        let queue: Arc<Queue<usize>> = Arc::new(Queue::new());
        for i in 0..100 {
            queue.push(i);
        }
        let barrier = Arc::new(tokio::sync::Barrier::new(2));
        let future_queue = queue.clone();
        let future_barrier = barrier.clone();
        let future = tokio::spawn(async move {
            future_barrier.wait().await;
            assert!(!future_queue.is_empty());
            future_queue.empty().await;
        });
        // Slightly delay the pop operations
        // to ensure that the spawned task has
        // time to start
        barrier.wait().await;
        for _ in 0..100 {
            queue.pop().await;
        }
        future.await.unwrap();
        assert_eq!(queue.len(), 0);
    }

    #[test]
    fn test_debug() {
        struct NoDebug {}
        let queue: Queue<NoDebug> = Queue::new();
        format!("{:?}", queue);
    }
}
