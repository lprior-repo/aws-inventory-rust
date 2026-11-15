use super::MappingSource;

#[path = "family_l_p/completion_additions.rs"]
mod completion_additions;
#[path = "family_l_p/final_additions.rs"]
mod final_additions;
#[path = "family_l_p/cfn_0.rs"]
mod cfn_0;
#[path = "family_l_p/cfn_1.rs"]
mod cfn_1;
#[path = "family_l_p/cfn_2.rs"]
mod cfn_2;
#[path = "family_l_p/cfn_3.rs"]
mod cfn_3;
#[path = "family_l_p/cfn_4.rs"]
mod cfn_4;
#[path = "family_l_p/cfn_5.rs"]
mod cfn_5;
#[path = "family_l_p/cfn_6.rs"]
mod cfn_6;
#[path = "family_l_p/cfn_7.rs"]
mod cfn_7;
#[path = "family_l_p/cfn_8.rs"]
mod cfn_8;
#[path = "family_l_p/cfn_9.rs"]
mod cfn_9;
#[path = "family_l_p/api_only_0.rs"]
mod api_only_0;
#[path = "family_l_p/api_only_1.rs"]
mod api_only_1;
#[path = "family_l_p/api_only_2.rs"]
mod api_only_2;
#[path = "family_l_p/operations_0.rs"]
mod operations_0;
#[path = "family_l_p/operations_1.rs"]
mod operations_1;
#[path = "family_l_p/operations_2.rs"]
mod operations_2;
#[path = "family_l_p/operations_3.rs"]
mod operations_3;
#[path = "family_l_p/operations_4.rs"]
mod operations_4;
#[path = "family_l_p/operations_5.rs"]
mod operations_5;
#[path = "family_l_p/operations_6.rs"]
mod operations_6;
#[path = "family_l_p/operations_7.rs"]
mod operations_7;
#[path = "family_l_p/operations_8.rs"]
mod operations_8;
#[path = "family_l_p/operations_9.rs"]
mod operations_9;
#[path = "family_l_p/operations_10.rs"]
mod operations_10;
#[path = "family_l_p/operations_11.rs"]
mod operations_11;
#[path = "family_l_p/relations_0.rs"]
mod relations_0;

pub(super) const SOURCES: &[MappingSource<'static>] = &[
    completion_additions::SOURCE,
    final_additions::SOURCE,
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
    MappingSource { relations: relations_0::ENTRIES, ..super::EMPTY },
];
