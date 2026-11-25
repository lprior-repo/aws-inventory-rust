pub mod cbor;
mod defaults;
pub mod errors;
mod headers;
pub mod json;
mod timestamp;
pub mod xml;

use crate::request::Protocol;
use http::{HeaderMap, StatusCode};
use inventory_core::{
    catalog::{CatalogError, Operation, Service},
    record::{BuildError, MAX_DEPTH, MAX_RECORD_BYTES, RecordError},
};
use std::collections::TryReserveError;

const MAX_NODES: usize = 65_536;

#[derive(Clone, Copy, Debug)]
pub struct DecodeLimits {
    body_bytes: usize,
    nodes: usize,
    depth: usize,
    preserve_fields: bool,
    excluded_members: inventory_core::record::MemberExclusions,
}

impl DecodeLimits {
    pub fn new(body_bytes: usize, nodes: usize, depth: usize) -> Result<Self, DecodeError> {
        if body_bytes == 0 || body_bytes > MAX_RECORD_BYTES {
            return Err(DecodeError::Limit("response body"));
        }
        if nodes == 0 || nodes > MAX_NODES {
            return Err(DecodeError::Limit("response nodes"));
        }
        if depth == 0 || depth > MAX_DEPTH {
            return Err(DecodeError::Limit("response depth"));
        }
        Ok(Self {
            body_bytes,
            nodes,
            depth,
            preserve_fields: false,
            excluded_members: inventory_core::record::MemberExclusions::default(),
        })
    }

    pub fn body_bytes(self) -> usize {
        self.body_bytes
    }

    pub fn nodes(self) -> usize {
        self.nodes
    }

    pub fn depth(self) -> usize {
        self.depth
    }

    /// Retains absent fields as absent instead of synthesizing SDK defaults.
    /// Inventory snapshots require observed tag values and pagination evidence.
    pub fn preserve_field_presence(mut self) -> Self {
        self.preserve_fields = true;
        self
    }

    /// Skip modeled member names before decoding values, including nested occurrences.
    pub fn excluding_members(mut self, members: inventory_core::record::MemberExclusions) -> Self {
        self.excluded_members = members;
        self
    }
}

impl Default for DecodeLimits {
    fn default() -> Self {
        Self {
            body_bytes: MAX_RECORD_BYTES,
            nodes: MAX_NODES,
            depth: MAX_DEPTH,
            preserve_fields: false,
            excluded_members: inventory_core::record::MemberExclusions::default(),
        }
    }
}

#[derive(Clone, Copy)]
pub struct ResponseContext<'a> {
    pub service: Service<'a>,
    pub operation: Operation<'a>,
    pub protocol: Protocol,
    pub body: &'a [u8],
    pub headers: &'a HeaderMap,
    pub status: StatusCode,
    pub limits: DecodeLimits,
}

#[derive(Debug, thiserror::Error)]
pub enum DecodeError {
    #[error(transparent)]
    Catalog(#[from] CatalogError),
    #[error(transparent)]
    Record(#[from] RecordError),
    #[error(transparent)]
    Service(#[from] errors::ServiceError),
    #[error(transparent)]
    Builder(#[from] BuildError),
    #[error(transparent)]
    Allocation(#[from] TryReserveError),
    #[error("response limit exceeded: {0}")]
    Limit(&'static str),
    #[error("invalid response: {0}")]
    Invalid(&'static str),
    #[error("unsupported response: {0}")]
    Unsupported(&'static str),
}

pub fn decode(context: ResponseContext<'_>) -> Result<inventory_core::record::Record, DecodeError> {
    if context.body.len() > context.limits.body_bytes() {
        return Err(DecodeError::Limit("response body"));
    }
    if !context.status.is_success() {
        return Err(errors::decode(context)?.into());
    }
    match context.protocol {
        Protocol::AwsJson10 | Protocol::AwsJson11 | Protocol::RestJson1 => json::decode(context),
        Protocol::AwsQuery | Protocol::Ec2Query | Protocol::RestXml => xml::decode(context),
        Protocol::RpcV2Cbor => cbor::decode(context),
    }
}
