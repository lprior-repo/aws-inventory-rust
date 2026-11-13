//! Oracle schema, canonical record encodings, and order-independent state digests.
//!
//! State digests are multiset digests: each canonical record contributes one SHA-256
//! word-vector that is added lane-wise, so equal event multisets in any order (the
//! workload intentionally reorders events) still produce the same digest.

use std::collections::BTreeMap;

use inventory_core::{Edge, Resource};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const PAGE_SIZE: usize = 100;
pub const OBSERVED_AT: i64 = 1_700_000_000;
pub const REGIONS: [&str; 4] = ["us-east-1", "us-west-2", "eu-west-1", "ap-southeast-2"];
pub const KINDS: [&str; 5] = [
    "aws_ec2_instance",
    "aws_ec2_vpc",
    "aws_ec2_security_group",
    "aws_s3_bucket",
    "aws_iam_role",
];

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    Uniform,
    AssetsTotal,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct AccountStats {
    pub resources: usize,
    pub edges: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeletionOracle {
    pub count: usize,
    pub digest: String,
    pub per_account: BTreeMap<String, usize>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeletionRecord {
    pub account_id: String,
    pub resource_id: String,
    pub kind: String,
    pub deleted_at: i64,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Oracle {
    pub seed: u64,
    pub mode: Mode,
    pub accounts: usize,
    pub page_size: usize,
    pub resources: usize,
    pub edges: usize,
    pub kinds: BTreeMap<String, usize>,
    pub regions: BTreeMap<String, usize>,
    pub per_account: BTreeMap<String, AccountStats>,
    pub empty_regions: Vec<String>,
    pub global_services: usize,
    pub duplicates: usize,
    pub reorders: usize,
    pub deletions: DeletionOracle,
    pub digest: String,
}

/// Commutative SHA-256 multiset accumulator.
#[derive(Clone, Debug, Default)]
pub struct MultiDigest {
    lanes: [u64; 4],
}

impl MultiDigest {
    pub fn add(&mut self, bytes: &[u8]) {
        let digest = Sha256::digest(bytes);
        for (lane, chunk) in self.lanes.iter_mut().zip(digest.chunks(8)) {
            let mut word = [0u8; 8];
            for (slot, byte) in word.iter_mut().zip(chunk) {
                *slot = *byte;
            }
            *lane = lane.wrapping_add(u64::from_be_bytes(word));
        }
    }

    #[must_use]
    pub fn finish(self) -> String {
        let mut hasher = Sha256::new();
        for lane in self.lanes {
            hasher.update(lane.to_be_bytes());
        }
        hex(&hasher.finalize())
    }
}

pub fn canonical_resource(resource: &Resource) -> String {
    format!(
        "{}\t{}\t{}\t{}\t{}\t{}\n",
        resource.account_id,
        resource.region,
        resource.kind,
        resource.id,
        resource.observed_at,
        serde_json::to_string(&resource.properties).unwrap_or_default()
    )
}

pub fn canonical_edge(edge: &Edge) -> String {
    format!(
        "{}\t{}\t{}\t{}\n",
        edge.from_id, edge.to_id, edge.relation, edge.observed_at
    )
}

pub fn canonical_deletion(record: &DeletionRecord) -> String {
    format!(
        "{}\t{}\t{}\t{}\n",
        record.account_id, record.resource_id, record.kind, record.deleted_at
    )
}

pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
