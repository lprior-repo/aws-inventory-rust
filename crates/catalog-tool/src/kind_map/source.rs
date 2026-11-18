use super::{
    AdditionalEmissionMapping, BootstrapPurpose, CompoundRelationMapping, KindMap, KindMapError,
    ParentRelationMapping, RelationMapping, RelationTargetsMapping,
};
use crate::names::Exceptions;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CfnMapping<'a> {
    Mapped {
        cfn_type: &'a str,
        kind: &'a str,
        display_name_path: Option<&'a str>,
    },
    Unmapped {
        cfn_type: &'a str,
        reason: &'a str,
    },
}

impl<'a> CfnMapping<'a> {
    pub const fn canonical(kind: &'a str) -> Self {
        Self::Mapped {
            cfn_type: kind,
            kind,
            display_name_path: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ApiOnlyMapping<'a> {
    pub kind: &'a str,
    pub reason: &'a str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OperationMapping<'a> {
    pub service: &'a str,
    pub operation: &'a str,
    pub kind: &'a str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootstrapMapping<'a> {
    AccountIdentity {
        service: &'a str,
        operation: &'a str,
        account_path: &'a str,
    },
    RegionDiscovery {
        service: &'a str,
        operation: &'a str,
        regions_path: &'a str,
        region_name_path: &'a str,
        opt_in_status_path: Option<&'a str>,
    },
}

impl BootstrapMapping<'_> {
    pub(super) fn owned(self) -> super::bootstrap::BootstrapEntry {
        match self {
            Self::AccountIdentity {
                service,
                operation,
                account_path,
            } => super::bootstrap::BootstrapEntry {
                service: service.to_owned(),
                operation: operation.to_owned(),
                purpose: BootstrapPurpose::AccountIdentity {
                    account_path: account_path.to_owned(),
                },
            },
            Self::RegionDiscovery {
                service,
                operation,
                regions_path,
                region_name_path,
                opt_in_status_path,
            } => super::bootstrap::BootstrapEntry {
                service: service.to_owned(),
                operation: operation.to_owned(),
                purpose: BootstrapPurpose::RegionDiscovery {
                    regions_path: regions_path.to_owned(),
                    region_name_path: region_name_path.to_owned(),
                    opt_in_status_path: opt_in_status_path.map(str::to_owned),
                },
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MappingSource<'a> {
    pub version: u16,
    pub reviewer: &'a str,
    pub reviewed: &'a str,
    pub default_unmapped_reason: Option<&'a str>,
    pub cfn: &'a [CfnMapping<'a>],
    pub api_only: &'a [ApiOnlyMapping<'a>],
    pub operations: &'a [OperationMapping<'a>],
    pub relations: &'a [RelationMapping<'a>],
    pub parent_relations: &'a [ParentRelationMapping<'a>],
    pub compound_relations: &'a [CompoundRelationMapping<'a>],
    pub additional_emissions: &'a [AdditionalEmissionMapping<'a>],
    pub relation_targets: &'a [RelationTargetsMapping<'a>],
    pub bootstraps: &'a [BootstrapMapping<'a>],
}

impl Default for MappingSource<'_> {
    fn default() -> Self {
        Self {
            version: 1,
            reviewer: "Typed fixture",
            reviewed: "2026-11-15",
            default_unmapped_reason: None,
            cfn: &[],
            api_only: &[],
            operations: &[],
            relations: &[],
            parent_relations: &[],
            compound_relations: &[],
            additional_emissions: &[],
            relation_targets: &[],
            bootstraps: &[],
        }
    }
}

impl MappingSource<'_> {
    pub fn compile(&self, exceptions: &Exceptions) -> Result<KindMap, KindMapError> {
        super::construct::compile(self, exceptions)
    }

    pub fn compile_with_cfn(
        &self,
        cfn: &[u8],
        exceptions: &Exceptions,
    ) -> Result<KindMap, KindMapError> {
        let mut map = self.compile(exceptions)?;
        super::complete_cfn(&mut map, cfn)?;
        map.source_sha256 = super::fingerprint::digest(&map);
        Ok(map)
    }
}
