mod fields;
mod last_item;
mod resume;
mod tokens;
#[cfg(test)]
mod tests;
use fields::*;
use inventory_core::{
    catalog::runtime::{PagePlan, TokenBinding, TokenPlacement, TokenSelection, TokenType},
    record::{Exclusions, FieldId, Key, Record, Value, ValueRef},
};

use crate::sweep_runtime::{SweepError, input, record};

const PAGES: u32 = 65_530;
const CURRENT: u32 = 65_532;
const PARTIAL: u32 = 65_533;
const MAX_TOKEN_RECORD: usize = 65_000;
const MAX_CURSOR: usize = 65_536;

#[derive(Default)]
pub(in crate::sweep_runtime) struct Cursor {
    pub pages: u32,
    pub current: Option<Record>,
    pub partial: bool,
}

pub(in crate::sweep_runtime) struct Advance {
    pub cursor: Record,
    pub continuation_digest: Option<[u8; 32]>,
}

impl Cursor {
    pub(in crate::sweep_runtime) fn decode(token: Option<&[u8]>) -> Result<Self, SweepError> {
        token.map_or_else(|| Ok(Self::default()), resume::decode)
    }

    pub(in crate::sweep_runtime) fn request_fields(
        &self,
        plan: &PagePlan,
    ) -> Result<Vec<input::InputField>, SweepError> {
        self.current.as_ref().map_or_else(
            || Ok(Vec::new()),
            |current| resume::request_fields(plan, current),
        )
    }

    pub(in crate::sweep_runtime) fn next(
        self,
        plan: &PagePlan,
        response: &Record,
        headers: &http::HeaderMap,
        limit: u32,
    ) -> Result<Option<Advance>, SweepError> {
        let Some(request) = tokens::request_record(plan, response, headers)? else {
            return Ok(None);
        };
        self.advance(request, limit).map(Some)
    }

    fn advance(self, request: Record, limit: u32) -> Result<Advance, SweepError> {
        let pages = self.pages.checked_add(1)
            .ok_or(SweepError::Operation("pagination page counter overflow"))?;
        let partial = pages >= limit || request.as_bytes().len() > MAX_TOKEN_RECORD;
        let continuation_digest = Some(token_digest(&request)?);
        let current = if partial { empty_record()? } else { request };
        let cursor = encode(pages, current.as_bytes(), partial)?;
        if cursor.as_bytes().len() > MAX_CURSOR {
            return Err(SweepError::Operation("pagination checkpoint exceeds durable cursor bound"));
        }
        Ok(Advance { cursor, continuation_digest })
    }
}

fn empty_record() -> Result<Record, SweepError> {
    Record::encode(&Value::Map(std::collections::BTreeMap::new())).map_err(SweepError::from)
}
