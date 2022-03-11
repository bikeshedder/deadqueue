#[cfg(feature = "resizable")]
mod tests {

    use std::sync::Arc;

    use deadqueue::resizable::Queue;

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
        let queue: Arc<Queue<usize>> = Arc::new(Queue::new(10));
        let mut futures = Vec::new();
        for _ in 0..100usize {
            let queue = queue.clone();
            futures.push(tokio::spawn(async move {
                for _ in 0..100usize {
                    queue.pop().await;
                }
            }));
        }
        for _ in 0..100usize {
            let queue = queue.clone();
            futures.push(tokio::spawn(async move {
                for i in 0..100usize {
                    queue.push(i).await;
                }
            }));
        }
        for future in futures {
            future.await.unwrap();
        }
        assert_eq!(queue.len(), 0);
    }

    #[test]
    fn test_debug() {
        struct NoDebug {}
        let queue: Queue<NoDebug> = Queue::new(1);
        format!("{:?}", queue);
    }
}
