#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("syntax: {0}")]
    Syntax(String),
    #[error("shape or framing violation: {0}")]
    Shape(&'static str),
    #[error("resource bound exceeded")]
    Bound,
    #[error(transparent)]
    Record(#[from] inventory_core::record::RecordError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
pub type Result<T, E = Error> = std::result::Result<T, E>;
pub fn syntax(error: impl std::fmt::Display) -> Error {
    Error::Syntax(error.to_string())
}
pub fn number(value: usize) -> Result<f64> {
    Ok(f64::from(u32::try_from(value).map_err(|_| Error::Bound)?))
}
