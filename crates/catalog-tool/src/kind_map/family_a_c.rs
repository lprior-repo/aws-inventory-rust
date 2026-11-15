use crate::kind_map::{RelationPath, RelationSource, RelationTarget};
use super::MappingSource;
mod completion_additions;

#[path = "family_a_c/cfn_0.rs"]
mod cfn_0;
#[path = "family_a_c/cfn_1.rs"]
mod cfn_1;
#[path = "family_a_c/cfn_2.rs"]
mod cfn_2;
#[path = "family_a_c/cfn_3.rs"]
mod cfn_3;
#[path = "family_a_c/cfn_4.rs"]
mod cfn_4;
#[path = "family_a_c/cfn_5.rs"]
mod cfn_5;
#[path = "family_a_c/cfn_6.rs"]
mod cfn_6;
#[path = "family_a_c/cfn_7.rs"]
mod cfn_7;
#[path = "family_a_c/cfn_8.rs"]
mod cfn_8;
#[path = "family_a_c/cfn_9.rs"]
mod cfn_9;
#[path = "family_a_c/api_only_0.rs"]
mod api_only_0;
#[path = "family_a_c/api_only_1.rs"]
mod api_only_1;
#[path = "family_a_c/api_only_2.rs"]
mod api_only_2;
#[path = "family_a_c/api_only_3.rs"]
mod api_only_3;
#[path = "family_a_c/api_only_4.rs"]
mod api_only_4;
#[path = "family_a_c/api_only_5.rs"]
mod api_only_5;
#[path = "family_a_c/api_only_6.rs"]
mod api_only_6;
#[path = "family_a_c/operations_0.rs"]
mod operations_0;
#[path = "family_a_c/operations_1.rs"]
mod operations_1;
#[path = "family_a_c/operations_2.rs"]
mod operations_2;
#[path = "family_a_c/operations_3.rs"]
mod operations_3;
#[path = "family_a_c/operations_4.rs"]
mod operations_4;
#[path = "family_a_c/operations_5.rs"]
mod operations_5;
#[path = "family_a_c/operations_6.rs"]
mod operations_6;
#[path = "family_a_c/operations_7.rs"]
mod operations_7;
#[path = "family_a_c/operations_8.rs"]
mod operations_8;
#[path = "family_a_c/operations_9.rs"]
mod operations_9;
#[path = "family_a_c/operations_10.rs"]
mod operations_10;
#[path = "family_a_c/operations_11.rs"]
mod operations_11;
#[path = "family_a_c/operations_12.rs"]
mod operations_12;
#[path = "family_a_c/operations_13.rs"]
mod operations_13;
#[path = "family_a_c/operations_14.rs"]
mod operations_14;
#[path = "family_a_c/operations_15.rs"]
mod operations_15;
#[path = "family_a_c/operations_16.rs"]
mod operations_16;
#[path = "family_a_c/operations_17.rs"]
mod operations_17;
#[path = "family_a_c/operations_18.rs"]
mod operations_18;
#[path = "family_a_c/operations_19.rs"]
mod operations_19;

pub(super) const SOURCES: &[MappingSource<'static>] = &[
    completion_additions::SOURCE,
    MappingSource { cfn: cfn_0::ENTRIES, ..super::EMPTY },
    MappingSource { cfn: cfn_1::ENTRIES, ..super::EMPTY },
    MappingSource { cfn: cfn_2::ENTRIES, ..super::EMPTY },
    MappingSource { cfn: cfn_3::ENTRIES, ..super::EMPTY },
    MappingSource { cfn: cfn_4::ENTRIES, ..super::EMPTY },
    MappingSource { cfn: cfn_5::ENTRIES, ..super::EMPTY },
    MappingSource { cfn: cfn_6::ENTRIES, ..super::EMPTY },
    MappingSource { cfn: cfn_7::ENTRIES, ..super::EMPTY },
    MappingSource { cfn: cfn_8::ENTRIES, ..super::EMPTY },
    MappingSource { cfn: cfn_9::ENTRIES, ..super::EMPTY },
    MappingSource { api_only: api_only_0::ENTRIES, ..super::EMPTY },
    MappingSource { api_only: api_only_1::ENTRIES, ..super::EMPTY },
    MappingSource { api_only: api_only_2::ENTRIES, ..super::EMPTY },
    MappingSource { api_only: api_only_3::ENTRIES, ..super::EMPTY },
    MappingSource { api_only: api_only_4::ENTRIES, ..super::EMPTY },
    MappingSource { api_only: api_only_5::ENTRIES, ..super::EMPTY },
    MappingSource { api_only: api_only_6::ENTRIES, ..super::EMPTY },
    MappingSource { operations: operations_0::ENTRIES, ..super::EMPTY },
    MappingSource { operations: operations_1::ENTRIES, ..super::EMPTY },
    MappingSource { operations: operations_2::ENTRIES, ..super::EMPTY },
    MappingSource { operations: operations_3::ENTRIES, ..super::EMPTY },
    MappingSource { operations: operations_4::ENTRIES, ..super::EMPTY },
    MappingSource { operations: operations_5::ENTRIES, ..super::EMPTY },
    MappingSource { operations: operations_6::ENTRIES, ..super::EMPTY },
    MappingSource { operations: operations_7::ENTRIES, ..super::EMPTY },
    MappingSource { operations: operations_8::ENTRIES, ..super::EMPTY },
    MappingSource { operations: operations_9::ENTRIES, ..super::EMPTY },
    MappingSource { operations: operations_10::ENTRIES, ..super::EMPTY },
    MappingSource { operations: operations_11::ENTRIES, ..super::EMPTY },
    MappingSource { operations: operations_12::ENTRIES, ..super::EMPTY },
    MappingSource { operations: operations_13::ENTRIES, ..super::EMPTY },
    MappingSource { operations: operations_14::ENTRIES, ..super::EMPTY },
    MappingSource { operations: operations_15::ENTRIES, ..super::EMPTY },
    MappingSource { operations: operations_16::ENTRIES, ..super::EMPTY },
    MappingSource { operations: operations_17::ENTRIES, ..super::EMPTY },
    MappingSource { operations: operations_18::ENTRIES, ..super::EMPTY },
    MappingSource { operations: operations_19::ENTRIES, ..super::EMPTY },
    MappingSource {
        relations: &[crate::kind_map::RelationMapping {
            source_kind: "AWS::AccessAnalyzer::ArchiveRule", source: RelationSource::Request(RelationPath::Fields("analyzerName")),
            target_kind: "AWS::AccessAnalyzer::Analyzer", target: RelationTarget::SelectedOutput(RelationPath::Fields("name")),
            relation: "IN_ANALYZER", scope: "account_region", guard: None,
        }],
        ..super::EMPTY
    },
];
