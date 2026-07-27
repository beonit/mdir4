use crate::runtime::job::{Deadline, JobControl, OperationId};

use super::location::LocationId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ViewGeneration(pub u64);

impl ViewGeneration {
    pub fn next(self) -> Self {
        Self(self.0.saturating_add(1))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SessionEpoch(pub u64);

impl SessionEpoch {
    pub fn next(self) -> Self {
        Self(self.0.saturating_add(1))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteRequest {
    pub location: LocationId,
    pub session_epoch: SessionEpoch,
    pub view_generation: ViewGeneration,
    pub operation_id: OperationId,
}

impl RemoteRequest {
    pub fn new(
        location: LocationId,
        session_epoch: SessionEpoch,
        view_generation: ViewGeneration,
        deadline: Deadline,
    ) -> (crate::runtime::job::CancelHandle, Self, JobControl) {
        let (cancel, control) = JobControl::new(deadline);
        let request = Self {
            location,
            session_epoch,
            view_generation,
            operation_id: control.operation_id,
        };
        (cancel, request, control)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteJobOutcome {
    Cancelled,
    TimedOut,
    Failed,
}

pub fn terminal_outcome(control: &JobControl) -> Option<RemoteJobOutcome> {
    if control.cancel.is_cancelled() {
        Some(RemoteJobOutcome::Cancelled)
    } else if control.deadline.expired() {
        Some(RemoteJobOutcome::TimedOut)
    } else {
        None
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteResult<T> {
    pub request: RemoteRequest,
    pub result: Result<T, RemoteJobOutcome>,
}

impl<T> RemoteResult<T> {
    pub fn applies_to(
        &self,
        location: &LocationId,
        session_epoch: SessionEpoch,
        view_generation: ViewGeneration,
    ) -> bool {
        self.request.location == *location
            && self.request.session_epoch == session_epoch
            && self.request.view_generation == view_generation
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn late_results_cannot_apply_after_a_generation_or_epoch_change() {
        let location = LocationId::new("dev").unwrap();
        let (_, request, _) = RemoteRequest::new(
            location.clone(),
            SessionEpoch(2),
            ViewGeneration(4),
            Deadline::after(Duration::from_secs(1)),
        );
        let result = RemoteResult {
            request,
            result: Ok(()),
        };

        assert!(result.applies_to(&location, SessionEpoch(2), ViewGeneration(4)));
        assert!(!result.applies_to(&location, SessionEpoch(3), ViewGeneration(4)));
        assert!(!result.applies_to(&location, SessionEpoch(2), ViewGeneration(5)));
    }

    #[test]
    fn cancellation_wins_over_deadline_and_is_explicit() {
        let (cancel, _, control) = RemoteRequest::new(
            LocationId::new("dev").unwrap(),
            SessionEpoch(1),
            ViewGeneration(1),
            Deadline::after(Duration::ZERO),
        );
        cancel.cancel();
        assert_eq!(
            terminal_outcome(&control),
            Some(RemoteJobOutcome::Cancelled)
        );
    }
}
