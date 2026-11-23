use crate::inventory::StoreError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PgWriteConfig {
    max_connections: u32,
}

impl Default for PgWriteConfig {
    fn default() -> Self {
        Self { max_connections: 8 }
    }
}

impl PgWriteConfig {
    pub fn new(max_connections: u32) -> Result<Self, StoreError> {
        if !(1..=64).contains(&max_connections) {
            return Err(StoreError::Invalid("PostgreSQL pool size must be 1..=64"));
        }
        Ok(Self { max_connections })
    }

    pub fn max_connections(self) -> u32 {
        self.max_connections
    }
}
