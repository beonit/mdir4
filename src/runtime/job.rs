use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU64, Ordering},
};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OperationId(pub u64);

static NEXT_OPERATION_ID: AtomicU64 = AtomicU64::new(1);

impl OperationId {
    pub fn next() -> Self {
        Self(NEXT_OPERATION_ID.fetch_add(1, Ordering::Relaxed))
    }
}

#[derive(Debug, Clone, Default)]
pub struct CancelHandle(Arc<AtomicBool>);

#[derive(Debug, Clone)]
pub struct CancelToken(Arc<AtomicBool>);

pub fn cancellation_pair() -> (CancelHandle, CancelToken) {
    let shared = Arc::new(AtomicBool::new(false));
    (CancelHandle(shared.clone()), CancelToken(shared))
}

#[derive(Debug, Clone, Copy)]
pub struct Deadline(Instant);

impl Deadline {
    pub fn after(duration: Duration) -> Self {
        Self(Instant::now() + duration)
    }

    pub fn expired(self) -> bool {
        Instant::now() >= self.0
    }
}

impl CancelHandle {
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Release);
    }
}

impl CancelToken {
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cancellation_is_shared_and_operation_ids_are_unique() {
        let (handle, token) = cancellation_pair();
        assert!(!token.is_cancelled());
        handle.cancel();
        assert!(token.is_cancelled());
        assert_ne!(OperationId::next(), OperationId::next());
        assert!(Deadline::after(Duration::ZERO).expired());
    }
}
