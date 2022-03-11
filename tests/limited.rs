#[cfg(feature = "limited")]
mod tests {

    use std::sync::Arc;

    use deadqueue::limited::Queue;

    #[tokio::test]
    async fn test_basics() {
        let queue: Queue<usize> = Queue::new(2);
        assert_eq!(queue.len(), 0);
        assert!(queue.try_push(1).is_ok());
        assert_eq!(queue.len(), 1);
        assert!(queue.try_push(2).is_ok());
        assert_eq!(queue.len(), 2);
        assert!(queue.try_push(3).is_err());
        assert_eq!(queue.len(), 2);
        assert!(queue.try_pop().is_some());
        assert_eq!(queue.len(), 1);
        assert!(queue.try_push(3).is_ok());
        assert_eq!(queue.len(), 2);
    }

    #[tokio::test]
    async fn test_parallel() {
        let queue: Arc<Queue<usize>> = Arc::new(Queue::new(100));
        let mut futures = Vec::new();
        for _ in 0..100usize {
            let queue = queue.clone();
            futures.push(tokio::spawn(async move {
                for _ in 0..100usize {
                    queue.pop().await;
                }
            }));
        }
        for i in 0..10000 {
            queue.push(i).await;
        }
        for future in futures {
            future.await.unwrap();
        }
        assert_eq!(queue.len(), 0);
    }

    #[tokio::test]
    async fn test_full() {
        let queue: Arc<Queue<usize>> = Arc::new(Queue::new(100));
        let future_queue = queue.clone();
        let future = tokio::spawn(async move {
            future_queue.full().await;
        });
        for i in 0..100 {
            queue.push(i).await;
        }
        future.await.unwrap();
        assert_eq!(queue.len(), 100);
    }

    #[tokio::test]
    async fn test_empty() {
        let queue: Arc<Queue<usize>> = Arc::new(Queue::new(100));
        for i in 0..100 {
            queue.push(i).await;
        }
        let future_queue = queue.clone();
        let future = tokio::spawn(async move {
            future_queue.empty().await;
        });
        for _ in 0..100 {
            queue.pop().await;
        }
        future.await.unwrap();
        assert_eq!(queue.len(), 0);
    }

    #[test]
    fn test_debug() {
        struct NoDebug {}
        let queue: Queue<NoDebug> = Queue::new(1);
        format!("{:?}", queue);
    }
}
