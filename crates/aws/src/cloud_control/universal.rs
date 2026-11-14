use super::*;

mod outcomes;
mod selection;
pub(crate) use outcomes::collect;
pub use selection::{UniversalReport, digest, list_descriptors, report};

#[cfg(test)]
mod tests;
