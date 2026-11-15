use super::{AccountId, InventoryError, Kind, Observation, Region, ResourceKey};
use std::num::NonZeroU16;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PageSize(NonZeroU16);

impl PageSize {
    pub fn new(value: u16) -> Result<Self, InventoryError> {
        let value = NonZeroU16::new(value)
            .filter(|size| size.get() <= 1000)
            .ok_or(InventoryError::PageSize)?;
        Ok(Self(value))
    }

    pub fn value(self) -> u16 {
        self.0.get()
    }
}

#[derive(Clone, Debug, Default)]
pub struct Filter {
    pub account: Option<AccountId>,
    pub region: Option<Region>,
    pub kind: Option<Kind>,
    search: String,
}

impl Filter {
    pub fn with_search(mut self, search: String) -> Result<Self, InventoryError> {
        if search.len() > 256 || search.contains('\0') {
            return Err(InventoryError::Search);
        }
        self.search = search;
        Ok(self)
    }

    pub fn search(&self) -> &str {
        &self.search
    }
}

#[derive(Clone, Debug)]
pub struct PageRequest {
    pub after: Option<ResourceKey>,
    pub size: PageSize,
}

#[derive(Debug)]
pub struct ResourcePage {
    pub rows: Vec<Observation>,
    pub next: Option<ResourceKey>,
    pub total: u64,
}
