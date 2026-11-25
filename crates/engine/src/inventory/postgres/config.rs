use crate::inventory::StoreError;
use std::num::NonZeroUsize;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum DirectCopy {
    Disabled,
    Enabled,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PgWriteConfig {
    max_connections: u32,
    copy_min_rows: Option<NonZeroUsize>,
    direct_copy: DirectCopy,
}

impl Default for PgWriteConfig {
    fn default() -> Self {
        Self {
            max_connections: 8,
            copy_min_rows: None,
            direct_copy: DirectCopy::Disabled,
        }
    }
}

impl PgWriteConfig {
    pub fn new(max_connections: u32) -> Result<Self, StoreError> {
        if !(1..=64).contains(&max_connections) {
            return Err(StoreError::Invalid("PostgreSQL pool size must be 1..=64"));
        }
        Ok(Self {
            max_connections,
            copy_min_rows: None,
            direct_copy: DirectCopy::Disabled,
        })
    }

    pub fn max_connections(self) -> u32 {
        self.max_connections
    }

    #[must_use]
    pub fn with_copy_min_rows(self, rows: NonZeroUsize) -> Self {
        Self {
            copy_min_rows: Some(rows),
            ..self
        }
    }

    pub fn with_direct_copy(self) -> Result<Self, StoreError> {
        if self.copy_min_rows.is_none() {
            return Err(StoreError::Invalid(
                "direct PostgreSQL COPY requires copy_min_rows",
            ));
        }
        Ok(Self {
            direct_copy: DirectCopy::Enabled,
            ..self
        })
    }

    pub(super) fn copy_min_rows(self) -> Option<NonZeroUsize> {
        self.copy_min_rows
    }

    pub(super) fn direct_copy(self) -> DirectCopy {
        self.direct_copy
    }
}
