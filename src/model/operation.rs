use std::path::PathBuf;

pub use crate::runtime::job::OperationId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConflictDecision {
    Overwrite,
    OverwriteAll,
    Skip,
    SkipAll,
    Rename(PathBuf),
    Cancel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConflictPolicy {
    operation: OperationId,
    overwrite_all: bool,
    skip_all: bool,
}

impl ConflictPolicy {
    pub fn new(operation: OperationId) -> Self {
        Self {
            operation,
            overwrite_all: false,
            skip_all: false,
        }
    }

    pub fn remembered(&self, operation: OperationId) -> Option<ConflictDecision> {
        if operation != self.operation {
            return None;
        }
        if self.overwrite_all {
            Some(ConflictDecision::Overwrite)
        } else if self.skip_all {
            Some(ConflictDecision::Skip)
        } else {
            None
        }
    }

    pub fn apply(&mut self, decision: &ConflictDecision) {
        match decision {
            ConflictDecision::OverwriteAll => self.overwrite_all = true,
            ConflictDecision::SkipAll => self.skip_all = true,
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_decisions_are_scoped_to_one_operation() {
        let operation = OperationId::next();
        let mut policy = ConflictPolicy::new(operation);
        policy.apply(&ConflictDecision::OverwriteAll);
        assert_eq!(
            policy.remembered(operation),
            Some(ConflictDecision::Overwrite)
        );
        assert_eq!(policy.remembered(OperationId::next()), None);
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OperationSummary {
    pub succeeded: usize,
    pub failed: usize,
    pub skipped: usize,
    pub bytes: u64,
    pub first_error: Option<String>,
}
