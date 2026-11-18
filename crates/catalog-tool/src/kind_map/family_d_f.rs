use super::MappingSource;

#[path = "family_d_f/complete_additions.rs"]
mod complete_additions;

#[path = "family_d_f/api_only_0.rs"]
mod api_only_0;
#[path = "family_d_f/api_only_1.rs"]
mod api_only_1;
#[path = "family_d_f/api_only_2.rs"]
mod api_only_2;
#[path = "family_d_f/cfn_0.rs"]
mod cfn_0;
#[path = "family_d_f/cfn_1.rs"]
mod cfn_1;
#[path = "family_d_f/cfn_2.rs"]
mod cfn_2;
#[path = "family_d_f/cfn_3.rs"]
mod cfn_3;
#[path = "family_d_f/cfn_4.rs"]
mod cfn_4;
#[path = "family_d_f/final_additions.rs"]
mod final_additions;
#[path = "family_d_f/operations_0.rs"]
mod operations_0;
#[path = "family_d_f/operations_1.rs"]
mod operations_1;
#[path = "family_d_f/operations_2.rs"]
mod operations_2;
#[path = "family_d_f/operations_3.rs"]
mod operations_3;
#[path = "family_d_f/operations_4.rs"]
mod operations_4;
#[path = "family_d_f/operations_5.rs"]
mod operations_5;
#[path = "family_d_f/operations_6.rs"]
mod operations_6;
#[path = "family_d_f/operations_7.rs"]
mod operations_7;
#[path = "family_d_f/relations_0.rs"]
mod relations_0;
#[path = "family_d_f/relations_1.rs"]
mod relations_1;

pub(super) const SOURCES: &[MappingSource<'static>] = &[
    complete_additions::SOURCE,
    final_additions::SOURCE,
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
        api_only: api_only_1::ENTRIES,
        ..super::EMPTY
    },
    MappingSource {
        api_only: api_only_2::ENTRIES,
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
        operations: operations_7::ENTRIES,
        ..super::EMPTY
    },
    MappingSource {
        relations: relations_0::ENTRIES,
        ..super::EMPTY
    },
    MappingSource {
        relations: relations_1::ENTRIES,
        ..super::EMPTY
    },
];
