use super::MappingSource;
use crate::kind_map::{RelationPath, RelationSource};

#[path = "family_g_k/api_only_0.rs"]
mod api_only_0;
#[path = "family_g_k/cfn_0.rs"]
mod cfn_0;
#[path = "family_g_k/cfn_1.rs"]
mod cfn_1;
#[path = "family_g_k/cfn_2.rs"]
mod cfn_2;
#[path = "family_g_k/cfn_3.rs"]
mod cfn_3;
#[path = "family_g_k/cfn_4.rs"]
mod cfn_4;
#[path = "family_g_k/final_children.rs"]
mod final_children;
#[path = "family_g_k/final_gamelift_fleets.rs"]
mod final_gamelift_fleets;
#[path = "family_g_k/final_glue_expansion.rs"]
mod final_glue_expansion;
#[path = "family_g_k/final_integrations.rs"]
mod final_integrations;
#[path = "family_g_k/final_iot.rs"]
mod final_iot;
#[path = "family_g_k/final_ivs_roots.rs"]
mod final_ivs_roots;
#[path = "family_g_k/final_roots.rs"]
mod final_roots;
#[path = "family_g_k/final_sitewise.rs"]
mod final_sitewise;
#[path = "family_g_k/operations_0.rs"]
mod operations_0;
#[path = "family_g_k/operations_1.rs"]
mod operations_1;
#[path = "family_g_k/operations_2.rs"]
mod operations_2;
#[path = "family_g_k/operations_3.rs"]
mod operations_3;
#[path = "family_g_k/operations_4.rs"]
mod operations_4;
#[path = "family_g_k/operations_5.rs"]
mod operations_5;
#[path = "family_g_k/operations_6.rs"]
mod operations_6;
#[path = "family_g_k/relations_0.rs"]
mod relations_0;

pub(super) const SOURCES: &[MappingSource<'static>] = &[
    MappingSource {
        cfn: cfn_0::ENTRIES,
        ..super::EMPTY
    },
    MappingSource {
        cfn: cfn_1::ENTRIES,
        ..super::EMPTY
    },
    MappingSource {
        cfn: cfn_2::ENTRIES,
        ..super::EMPTY
    },
    MappingSource {
        cfn: cfn_3::ENTRIES,
        ..super::EMPTY
    },
    MappingSource {
        cfn: cfn_4::ENTRIES,
        ..super::EMPTY
    },
    MappingSource {
        api_only: api_only_0::ENTRIES,
        ..super::EMPTY
    },
    MappingSource {
        operations: operations_0::ENTRIES,
        ..super::EMPTY
    },
    MappingSource {
        operations: operations_1::ENTRIES,
        ..super::EMPTY
    },
    MappingSource {
        operations: operations_2::ENTRIES,
        ..super::EMPTY
    },
    MappingSource {
        operations: operations_3::ENTRIES,
        ..super::EMPTY
    },
    MappingSource {
        operations: operations_4::ENTRIES,
        ..super::EMPTY
    },
    MappingSource {
        operations: operations_5::ENTRIES,
        ..super::EMPTY
    },
    MappingSource {
        operations: operations_6::ENTRIES,
        ..super::EMPTY
    },
    MappingSource {
        relations: relations_0::ENTRIES,
        ..super::EMPTY
    },
    MappingSource {
        relations: relations_0::REQUEST_ENTRIES,
        ..super::EMPTY
    },
    MappingSource {
        cfn: final_iot::CFN,
        api_only: final_iot::API,
        operations: final_iot::OPERATIONS,
        relations: final_iot::REQUEST_RELATIONS,
        ..super::EMPTY
    },
    MappingSource {
        cfn: final_children::CFN,
        api_only: final_children::API_ONLY,
        operations: final_children::OPERATIONS,
        relations: final_children::REQUEST_RELATIONS,
        relation_targets: final_children::RELATION_TARGETS,
        ..super::EMPTY
    },
    MappingSource {
        cfn: final_integrations::CFN,
        api_only: final_integrations::API_ONLY,
        operations: final_integrations::OPERATIONS,
        relations: final_integrations::RELATIONS,
        ..super::EMPTY
    },
    MappingSource {
        relations: final_integrations::REQUEST_RELATIONS,
        ..super::EMPTY
    },
    MappingSource {
        cfn: final_sitewise::CFN,
        api_only: final_sitewise::API,
        operations: final_sitewise::OPERATIONS,
        relations: final_sitewise::RELATIONS,
        ..super::EMPTY
    },
    MappingSource {
        relations: final_sitewise::REQUEST_RELATIONS,
        ..super::EMPTY
    },
    MappingSource {
        cfn: final_roots::CFN,
        api_only: final_roots::API_ONLY,
        operations: final_roots::OPERATIONS,
        relations: final_roots::RELATIONS,
        ..super::EMPTY
    },
    MappingSource {
        cfn: final_gamelift_fleets::CFN,
        operations: final_gamelift_fleets::OPERATIONS,
        ..super::EMPTY
    },
    MappingSource {
        cfn: final_glue_expansion::CFN,
        operations: final_glue_expansion::OPERATIONS,
        ..super::EMPTY
    },
    MappingSource {
        cfn: final_ivs_roots::CFN,
        api_only: final_ivs_roots::API_ONLY,
        operations: final_ivs_roots::OPERATIONS,
        ..super::EMPTY
    },
    MappingSource {
        relation_targets: &[crate::kind_map::RelationTargetsMapping {
            source_kind: "AWS::MSK::VpcConnection",
            source: RelationSource::SelectedOutput(RelationPath::Fields("TargetClusterArn")),
            relation: "CONNECTS_TO",
            target_kinds: &["AWS::MSK::Cluster", "AWS::MSK::ServerlessCluster"],
        }],
        ..super::EMPTY
    },
];
