use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("invalid name: {value}")]
pub struct NameError {
    pub value: String,
}
