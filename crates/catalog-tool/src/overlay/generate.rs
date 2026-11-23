mod derive;
mod identity;
mod items;
mod pagination;

use super::OperationOverlay;

pub(crate) type Derived = Result<OperationOverlay, String>;

#[cfg(test)]
mod tests;
