use super::{
    AdditionalEmissionMapping, BootstrapMapping, KindMap, KindMapError, MappingSource,
    ParentRelationMapping,
};

#[path = "builtin_api.rs"]
mod api;
#[path = "builtin_cfn.rs"]
mod cfn;
#[path = "family_a_c.rs"]
mod family_a_c;
#[path = "family_d_f.rs"]
mod family_d_f;
#[path = "family_g_k.rs"]
mod family_g_k;
#[path = "family_l_p.rs"]
mod family_l_p;
#[path = "family_q_s.rs"]
mod family_q_s;
#[path = "family_t_z.rs"]
mod family_t_z;
#[path = "builtin_operation.rs"]
mod operation;
#[path = "builtin_relation.rs"]
mod relation;

const EMPTY: MappingSource<'static> = MappingSource {
    version: 1,
    reviewed: "2026-11-15",
    reviewer: "Automated pinned-model review; no human approval",
    default_unmapped_reason: None,
    cfn: &[],
    api_only: &[],
    operations: &[],
    relations: &[],
    bootstraps: &[],
    relation_targets: &[],
    parent_relations: &[],
    compound_relations: &[],
    additional_emissions: &[],
};

pub(super) const SOURCE: MappingSource<'static> = MappingSource {
    version: 1,
    reviewer: "Automated model review; no human approval; limited explicit scope",
    reviewed: "2026-11-15",
    default_unmapped_reason: Some(
        "No individually reviewed canonical mapping; deliberately excluded from collection until a validated mapping and model schema review exists",
    ),
    cfn: cfn::ENTRIES,
    api_only: api::ENTRIES,
    operations: operation::ENTRIES,
    relations: relation::ENTRIES,
    relation_targets: &[],
    parent_relations: &[
        ParentRelationMapping {
            source_kind: "AWS::Route53::DNSSEC",
            target_kinds: &["AWS::Route53::HostedZone"],
            relation: "FOR_HOSTED_ZONE",
            scope: "account",
        },
        ParentRelationMapping {
            source_kind: "AWS::Route53::KeySigningKey",
            target_kinds: &["AWS::Route53::HostedZone"],
            relation: "FOR_HOSTED_ZONE",
            scope: "account",
        },
    ],
    compound_relations: &[],
    additional_emissions: &[AdditionalEmissionMapping {
        service: "route-53",
        operation: "GetDNSSEC",
        emitter_key: "key_signing_keys",
        kind: "AWS::Route53::KeySigningKey",
    }],
    bootstraps: &[
        BootstrapMapping::AccountIdentity {
            service: "sts",
            operation: "GetCallerIdentity",
            account_path: "Account",
        },
        BootstrapMapping::RegionDiscovery {
            service: "ec2",
            operation: "DescribeRegions",
            regions_path: "Regions",
            region_name_path: "RegionName",
            opt_in_status_path: Some("OptInStatus"),
        },
    ],
};

/// Compiles embedded reviewed mappings against the pinned CFN source.
///
/// # Errors
/// Returns an error for conflicting mappings or mismatched CFN source types.
pub fn embedded(cfn_source: &[u8]) -> Result<KindMap, KindMapError> {
    super::merge::compile(sources(), cfn_source)
}

pub(super) fn reviewed() -> Result<KindMap, KindMapError> {
    super::merge::reviewed(sources())
}

fn sources() -> impl Iterator<Item = super::MappingSource<'static>> + Clone {
    std::iter::once(SOURCE)
        .chain(family_a_c::SOURCES.iter().copied())
        .chain(family_d_f::SOURCES.iter().copied())
        .chain(family_g_k::SOURCES.iter().copied())
        .chain(family_l_p::SOURCES.iter().copied())
        .chain(family_q_s::SOURCES.iter().copied())
        .chain(family_t_z::SOURCES.iter().copied())
}

#[cfg(test)]
mod tests;
