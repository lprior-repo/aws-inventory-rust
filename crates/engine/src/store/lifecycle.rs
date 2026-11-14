use super::{ScanStatus, StoreError};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum FinalScanStatus {
    Complete,
    Partial,
    Cancelled,
}

impl TryFrom<ScanStatus> for FinalScanStatus {
    type Error = StoreError;

    fn try_from(status: ScanStatus) -> Result<Self, Self::Error> {
        match status {
            ScanStatus::Complete => Ok(Self::Complete),
            ScanStatus::Partial => Ok(Self::Partial),
            ScanStatus::Cancelled => Ok(Self::Cancelled),
            ScanStatus::Running => Err(StoreError::InvalidInput("finish status cannot be running")),
        }
    }
}

impl From<FinalScanStatus> for ScanStatus {
    fn from(status: FinalScanStatus) -> Self {
        match status {
            FinalScanStatus::Complete => Self::Complete,
            FinalScanStatus::Partial => Self::Partial,
            FinalScanStatus::Cancelled => Self::Cancelled,
        }
    }
}

impl FinalScanStatus {
    pub(super) fn as_str(self) -> &'static str {
        ScanStatus::from(self).as_str()
    }
}

#[derive(Debug)]
pub(super) enum PersistedScan {
    Running {
        started_at: i64,
    },
    Finalized {
        started_at: i64,
        status: FinalScanStatus,
        finished_at: i64,
    },
}

impl PersistedScan {
    pub(super) fn parse(
        started_at: i64,
        status: &str,
        finished_at: Option<i64>,
    ) -> Result<Self, StoreError> {
        if started_at < 0 || finished_at.is_some_and(|end| end < started_at) {
            return Err(StoreError::Conflict("invalid persisted scan timestamps"));
        }
        match (status, finished_at) {
            ("running", None) => Ok(Self::Running { started_at }),
            ("complete", Some(end)) => {
                Ok(Self::finalized(started_at, FinalScanStatus::Complete, end))
            }
            ("partial", Some(end)) => {
                Ok(Self::finalized(started_at, FinalScanStatus::Partial, end))
            }
            ("cancelled", Some(end)) => {
                Ok(Self::finalized(started_at, FinalScanStatus::Cancelled, end))
            }
            _ => Err(StoreError::Conflict("invalid persisted scan lifecycle")),
        }
    }

    fn finalized(started_at: i64, status: FinalScanStatus, finished_at: i64) -> Self {
        Self::Finalized {
            started_at,
            status,
            finished_at,
        }
    }

    pub(super) fn started_at(&self) -> i64 {
        match self {
            Self::Running { started_at } | Self::Finalized { started_at, .. } => *started_at,
        }
    }

    pub(super) fn status(&self) -> ScanStatus {
        match self {
            Self::Running { .. } => ScanStatus::Running,
            Self::Finalized { status, .. } => (*status).into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn persisted_lifecycle_requires_exactly_the_timestamps_for_its_state() {
        assert!(matches!(
            PersistedScan::parse(10, "running", None),
            Ok(PersistedScan::Running { .. })
        ));
        for status in ["complete", "partial", "cancelled"] {
            assert!(matches!(
                PersistedScan::parse(10, status, Some(11)),
                Ok(PersistedScan::Finalized { .. })
            ));
            assert!(PersistedScan::parse(10, status, None).is_err());
            assert!(PersistedScan::parse(10, status, Some(9)).is_err());
        }
        assert!(PersistedScan::parse(10, "running", Some(11)).is_err());
        assert!(PersistedScan::parse(10, "unknown", None).is_err());
        assert!(PersistedScan::parse(-1, "running", None).is_err());
        assert!(FinalScanStatus::try_from(ScanStatus::Running).is_err());
    }
}
