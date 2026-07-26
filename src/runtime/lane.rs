use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Busy;

#[derive(Debug, Clone, Default)]
pub struct MutationCoordinator(Arc<Mutex<bool>>);

pub struct MutationLease {
    active: Arc<Mutex<bool>>,
}

impl MutationCoordinator {
    pub fn try_acquire(&self) -> Result<MutationLease, Busy> {
        let mut active = self.0.try_lock().map_err(|_| Busy)?;
        if *active {
            return Err(Busy);
        }
        *active = true;
        drop(active);
        Ok(MutationLease {
            active: self.0.clone(),
        })
    }
}

impl Drop for MutationLease {
    fn drop(&mut self) {
        if let Ok(mut active) = self.active.lock() {
            *active = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mutation_lane_is_non_blocking_and_releases_on_drop() {
        let coordinator = MutationCoordinator::default();
        let lease = coordinator.try_acquire().unwrap();
        assert!(coordinator.try_acquire().is_err());
        drop(lease);
        assert!(coordinator.try_acquire().is_ok());
    }
}
