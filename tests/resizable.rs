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

    #[tokio::test]
    async fn test_full() {
        let queue: Arc<Queue<usize>> = Arc::new(Queue::new(100));
        let barrier = Arc::new(tokio::sync::Barrier::new(2));
        let future_queue = queue.clone();
        let future_barrier = barrier.clone();
        let future = tokio::spawn(async move {
            future_barrier.wait().await;
            assert_ne!(future_queue.capacity(), future_queue.len());
            future_queue.wait_full().await;
        });
        barrier.wait().await;
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
        let barrier = Arc::new(tokio::sync::Barrier::new(2));
        let future_queue = queue.clone();
        let future_barrier = barrier.clone();
        let future = tokio::spawn(async move {
            future_barrier.wait().await;
            assert!(!future_queue.is_empty());
            future_queue.wait_empty().await;
        });
        barrier.wait().await;
        for _ in 0..100 {
            queue.pop().await;
        }
        future.await.unwrap();
        assert_eq!(queue.len(), 0);
    }

    #[tokio::test]
    async fn test_full_deadlock() {
        let queue: Arc<Queue<()>> = Arc::new(Queue::new(2));
        let barrier = Arc::new(tokio::sync::Barrier::new(2));
        let future_queue = queue.clone();
        let future_barrier = barrier.clone();
        let future = tokio::spawn(async move {
            future_barrier.wait().await;
            assert_ne!(future_queue.capacity(), future_queue.len());
            println!("Awaiting full queue");
            future_queue.wait_full().await;
        });
        barrier.wait().await;
        println!("Begin queue operations");
        queue.push(()).await;
        queue.pop().await;
        for _ in 0..2 {
            queue.push(()).await;
        }
        println!("End queue operations");
        future.await.unwrap();
        assert_eq!(queue.len(), 2);
    }

    #[tokio::test]
    async fn test_empty_deadlock() {
        let queue: Arc<Queue<()>> = Arc::new(Queue::new(3));
        for _ in 0..2 {
            queue.push(()).await;
        }
        let barrier = Arc::new(tokio::sync::Barrier::new(2));
        let future_queue = queue.clone();
        let future_barrier = barrier.clone();
        let future = tokio::spawn(async move {
            future_barrier.wait().await;
            assert!(!future_queue.is_empty());
            println!("Awaiting full queue");
            future_queue.wait_empty().await;
        });
        barrier.wait().await;
        println!("Begin queue operations");
        queue.push(()).await;
        for _ in 0..3 {
            queue.pop().await;
        }
        println!("End queue operations");
        future.await.unwrap();
        assert_eq!(queue.len(), 0);
    }

    #[test]
    fn test_debug() {
        struct NoDebug {}
        let queue: Queue<NoDebug> = Queue::new(1);
        format!("{:?}", queue);
    }

    #[tokio::test]
    async fn test_resize_enlarge() {
        let queue: Queue<usize> = Queue::new(0);
        queue.resize(1).await;
        assert_eq!(queue.capacity(), 1);
    }

    #[tokio::test]
    async fn test_resize_shrink() {
        let queue: Queue<usize> = Queue::new(2);
        queue.try_push(0).unwrap();
        queue.resize(1).await;
        assert_eq!(queue.capacity(), 1);
        assert_eq!(queue.len(), 1);
        assert_eq!(queue.try_push(42), Err(42));
        queue.resize(0).await;
        assert_eq!(queue.capacity(), 0);
        assert_eq!(queue.len(), 0);
        assert_eq!(queue.try_push(42), Err(42));
    }

    #[tokio::test]
    async fn test_is_full_basic() {
        let queue: Queue<usize> = Queue::new(2);
        assert!(!queue.is_full(), "Should be empty at construction");
        queue.push(1).await;
        assert!(
            !queue.is_full(),
            "Should not be full a one less than capacity"
        );
        queue.push(2).await;
        assert!(queue.is_full(), "Should now be full");
        let _ = queue.pop().await;
        assert!(!queue.is_full(), "Should no longer be full after pop");
    }
}
