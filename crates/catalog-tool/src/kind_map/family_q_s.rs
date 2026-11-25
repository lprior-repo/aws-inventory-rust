use super::MappingSource;

#[path = "family_q_s/api_only_0.rs"]
mod api_only_0;
#[path = "family_q_s/api_only_1.rs"]
mod api_only_1;
#[path = "family_q_s/api_only_2.rs"]
mod api_only_2;
#[path = "family_q_s/api_only_3.rs"]
mod api_only_3;
#[path = "family_q_s/api_only_4.rs"]
mod api_only_4;
#[path = "family_q_s/api_only_5.rs"]
mod api_only_5;
#[path = "family_q_s/api_only_6.rs"]
mod api_only_6;
#[path = "family_q_s/cfn_0.rs"]
mod cfn_0;
#[path = "family_q_s/cfn_1.rs"]
mod cfn_1;
#[path = "family_q_s/cfn_2.rs"]
mod cfn_2;
#[path = "family_q_s/cfn_3.rs"]
mod cfn_3;
#[path = "family_q_s/cfn_4.rs"]
mod cfn_4;
#[path = "family_q_s/cfn_5.rs"]
mod cfn_5;
#[path = "family_q_s/cfn_6.rs"]
mod cfn_6;
#[path = "family_q_s/cfn_7.rs"]
mod cfn_7;
#[path = "family_q_s/coverage.rs"]
mod coverage;
#[path = "family_q_s/operations_0.rs"]
mod operations_0;
#[path = "family_q_s/operations_1.rs"]
mod operations_1;
#[path = "family_q_s/operations_10.rs"]
mod operations_10;
#[path = "family_q_s/operations_11.rs"]
mod operations_11;
#[path = "family_q_s/operations_12.rs"]
mod operations_12;
#[path = "family_q_s/operations_13.rs"]
mod operations_13;
#[path = "family_q_s/operations_14.rs"]
mod operations_14;
#[path = "family_q_s/operations_2.rs"]
mod operations_2;
#[path = "family_q_s/operations_3.rs"]
mod operations_3;
#[path = "family_q_s/operations_4.rs"]
mod operations_4;
#[path = "family_q_s/operations_5.rs"]
mod operations_5;
#[path = "family_q_s/operations_6.rs"]
mod operations_6;
#[path = "family_q_s/operations_7.rs"]
mod operations_7;
#[path = "family_q_s/operations_8.rs"]
mod operations_8;
#[path = "family_q_s/operations_9.rs"]
mod operations_9;
#[path = "family_q_s/relations_0.rs"]
mod relations_0;
#[path = "family_q_s/sanitized_sns.rs"]
mod sanitized_sns;

pub(super) const SOURCES: &[MappingSource<'static>] = &[
    sanitized_sns::SOURCE,
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
        cfn: cfn_5::ENTRIES,
        ..super::EMPTY
    },
    MappingSource {
        cfn: cfn_6::ENTRIES,
        ..super::EMPTY
    },
    MappingSource {
        cfn: cfn_7::ENTRIES,
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
        api_only: api_only_3::ENTRIES,
        ..super::EMPTY
    },
    MappingSource {
        api_only: api_only_4::ENTRIES,
        ..super::EMPTY
    },
    MappingSource {
        api_only: api_only_5::ENTRIES,
        ..super::EMPTY
    },
    MappingSource {
        api_only: api_only_6::ENTRIES,
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
        operations: operations_8::ENTRIES,
        ..super::EMPTY
    },
    MappingSource {
        operations: operations_9::ENTRIES,
        ..super::EMPTY
    },
    MappingSource {
        operations: operations_10::ENTRIES,
        ..super::EMPTY
    },
    MappingSource {
        operations: operations_11::ENTRIES,
        ..super::EMPTY
    },
    MappingSource {
        operations: operations_12::ENTRIES,
        ..super::EMPTY
    },
    MappingSource {
        operations: operations_13::ENTRIES,
        ..super::EMPTY
    },
    MappingSource {
        operations: operations_14::ENTRIES,
        ..super::EMPTY
    },
    MappingSource {
        relations: relations_0::ENTRIES,
        ..super::EMPTY
    },
    coverage::SOURCE,
];
