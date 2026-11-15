/// Exact source layer used to assemble a runtime catalog, never the final artifact's own hash.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ModelSourceProvenance {
    Repository { commit: String, sha256: [u8; 32] },
    CompiledCatalog { sha256: [u8; 32] },
}

/// Versioned source pins. Artifact hashes belong to the consumer's outer receipt.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CatalogSourceProvenance {
    pub version: u32,
    pub models: ModelSourceProvenance,
    pub kind_map_version: u32,
    pub kind_map_sha256: [u8; 32],
    pub cfn_sha256: Option<[u8; 32]>,
    /// Build-time digest of the actual embedded policy, naming, denial, and kind-map sources.
    pub embedded_sources_sha256: [u8; 32],
    /// Digest of canonical compiled capability fields, excluding this receipt.
    pub reviewed_inputs_sha256: [u8; 32],
}
