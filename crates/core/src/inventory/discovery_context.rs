mod codec;
mod projection;

use super::{OperationId, ServiceId};
use crate::{
    catalog::runtime::{ContextProjectionPlan, ContextSource},
    record::{ContentHash, Exclusions, FieldId, Record, RecordError, ValueRef},
};

pub const MAX_DISCOVERY_FRAMES: usize = 32;
pub const MAX_DISCOVERY_CONTEXT_BYTES: usize = 1_048_576;

#[derive(Debug, thiserror::Error)]
pub enum ContextError {
    #[error(transparent)]
    Record(#[from] RecordError),
    #[error(transparent)]
    Inventory(#[from] super::InventoryError),
    #[error("discovery context exceeds the frame or serialized byte bound")]
    Bounds,
    #[error("discovery context is malformed or has an unsupported version")]
    Invalid,
    #[error("a required discovery context field is absent or has an incompatible shape")]
    Missing,
}

/// All fields are private: a frame can only be constructed by exact-path projection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiscoveryFrame {
    service: ServiceId,
    operation: OperationId,
    ordinal: u32,
    branch: Option<Box<str>>,
    selected: Record,
    response: Record,
    request: Record,
}

impl DiscoveryFrame {
    /// `selected` must already have the producer's reviewed privacy projection applied.
    pub fn project(
        service: ServiceId,
        operation: OperationId,
        ordinal: u32,
        plan: &ContextProjectionPlan,
        selected: &Record,
        response: &Record,
        request: &Record,
    ) -> Result<Self, ContextError> {
        Ok(Self {
            service,
            operation,
            ordinal,
            branch: None,
            selected: projection::project(selected, &plan.selected)?,
            response: projection::project(response, &plan.response)?,
            request: projection::project(request, &plan.request)?,
        })
    }

    pub fn with_branch(mut self, branch: Option<&str>) -> Result<Self, ContextError> {
        if branch.is_some_and(|name| {
            name.is_empty()
                || name.len() > 64
                || !name
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-')
        }) {
            return Err(ContextError::Invalid);
        }
        self.branch = branch.map(Box::from);
        Ok(self)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DiscoveryContext {
    frames: Vec<DiscoveryFrame>,
}

impl DiscoveryContext {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ContextError> {
        if bytes.is_empty() {
            return Ok(Self::empty());
        }
        codec::decode(bytes)
    }

    pub fn encode(&self) -> Result<Vec<u8>, ContextError> {
        codec::encode(self).map(Record::into_bytes)
    }

    /// Exact canonical wire size, computed without allocating or serializing.
    pub fn encoded_len(&self) -> Result<usize, ContextError> {
        codec::encoded_len(self, None)
    }

    /// Exact wire size after appending a frame, checked before cloning any ancestry.
    pub fn pushed_encoded_len(&self, frame: &DiscoveryFrame) -> Result<usize, ContextError> {
        codec::encoded_len(self, Some(frame))
    }

    pub fn push(&self, frame: DiscoveryFrame) -> Result<Self, ContextError> {
        self.pushed_encoded_len(&frame)?;
        let mut frames = self.frames.clone();
        frames.push(frame);
        let context = Self { frames };
        Ok(context)
    }

    pub fn discriminator(&self) -> Result<ContentHash, ContextError> {
        codec::encode(self)?
            .content_hash(&Exclusions::default())
            .map_err(ContextError::from)
    }

    pub fn value<'a>(
        &'a self,
        service: &str,
        operation: &str,
        source: ContextSource,
        path: &[FieldId],
    ) -> Result<Option<ValueRef<'a>>, ContextError> {
        let frame = self.frames.iter().rev().find(|frame| {
            frame.service.as_str() == service && frame.operation.as_str() == operation
        });
        let Some(frame) = frame else { return Ok(None) };
        let record = match source {
            ContextSource::SelectedItem => &frame.selected,
            ContextSource::Response => &frame.response,
            ContextSource::Request => &frame.request,
        };
        projection::value(record, path)
    }

    pub fn value_from_branch<'a>(
        &'a self,
        service: &str,
        operation: &str,
        branch: &str,
        source: ContextSource,
        path: &[FieldId],
    ) -> Result<Option<ValueRef<'a>>, ContextError> {
        let Some(frame) = self.frames.iter().rev().find(|frame| {
            frame.service.as_str() == service
                && frame.operation.as_str() == operation
                && frame.branch.as_deref() == Some(branch)
        }) else {
            return Ok(None);
        };
        let record = match source {
            ContextSource::SelectedItem => &frame.selected,
            ContextSource::Response => &frame.response,
            ContextSource::Request => &frame.request,
        };
        projection::value(record, path)
    }
}

#[cfg(test)]
mod tests;
