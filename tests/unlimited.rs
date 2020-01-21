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
}
