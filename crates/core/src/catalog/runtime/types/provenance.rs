#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ModelSourceProvenance {
    Repository { commit: String, sha256: [u8; 32] },
    CompiledCatalog { sha256: [u8; 32] },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CatalogSourceProvenance {
    pub version: u32,
    pub models: ModelSourceProvenance,
    pub kind_map_version: u32,
    pub kind_map_sha256: [u8; 32],
    pub cfn_sha256: Option<[u8; 32]>,
    pub embedded_sources_sha256: [u8; 32],
    pub reviewed_inputs_sha256: [u8; 32],
}
