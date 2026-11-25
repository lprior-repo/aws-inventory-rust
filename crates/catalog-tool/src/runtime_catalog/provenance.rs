use super::{RuntimeCatalogCompileError, encode};
use inventory_core::{
    catalog::{Catalog, runtime::ModelSourceProvenance},
    record::Value,
};
use sha2::{Digest, Sha256};

include!(concat!(env!("OUT_DIR"), "/embedded_sources_sha256.rs"));

pub struct CatalogSources {
    models: ModelSourceProvenance,
    cfn_sha256: Option<[u8; 32]>,
}

impl CatalogSources {
    pub fn repository(commit: String, models_sha256: [u8; 32], cfn_sha256: [u8; 32]) -> Self {
        Self {
            models: ModelSourceProvenance::Repository {
                commit,
                sha256: models_sha256,
            },
            cfn_sha256: Some(cfn_sha256),
        }
    }

    pub(super) fn compiled(base_bytes: &[u8]) -> Self {
        Self {
            models: ModelSourceProvenance::CompiledCatalog {
                sha256: Sha256::digest(base_bytes).into(),
            },
            cfn_sha256: None,
        }
    }

    pub(super) fn validate(&self, catalog: Catalog<'_>) -> Result<(), RuntimeCatalogCompileError> {
        if let ModelSourceProvenance::Repository { commit, .. } = &self.models
            && (commit != catalog.models_commit()? || self.cfn_sha256.is_none())
        {
            return Err(RuntimeCatalogCompileError::Path {
                path: "provenance validation".to_owned(),
            });
        }
        Ok(())
    }
}

pub(super) fn encode(
    sources: &CatalogSources,
    kinds: &crate::kind_map::KindMap,
    reviewed_inputs_sha256: [u8; 32],
) -> Result<Value, RuntimeCatalogCompileError> {
    let models = match &sources.models {
        ModelSourceProvenance::Repository { commit, sha256 } => encode::map([
            (1, Value::Int(0)),
            (2, Value::Bytes(sha256.to_vec())),
            (3, Value::Text(commit.clone())),
        ])?,
        ModelSourceProvenance::CompiledCatalog { sha256 } => {
            encode::map([(1, Value::Int(1)), (2, Value::Bytes(sha256.to_vec()))])?
        }
    };
    encode::map([
        (1, Value::Int(1)),
        (2, models),
        (3, Value::Int(i64::from(kinds.source_version()))),
        (4, Value::Bytes(kinds.source_sha256().to_vec())),
        (
            5,
            sources
                .cfn_sha256
                .map_or(Value::Null, |sha| Value::Bytes(sha.to_vec())),
        ),
        (6, Value::Bytes(EMBEDDED_SOURCES_SHA256.to_vec())),
        (7, Value::Bytes(reviewed_inputs_sha256.to_vec())),
    ])
}
