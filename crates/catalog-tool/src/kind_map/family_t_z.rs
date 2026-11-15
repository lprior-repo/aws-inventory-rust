use super::MappingSource;

#[path = "family_t_z/cfn_0.rs"]
mod cfn_0;
#[path = "family_t_z/cfn_1.rs"]
mod cfn_1;
#[path = "family_t_z/cfn_2.rs"]
mod cfn_2;
#[path = "family_t_z/cfn_3.rs"]
mod cfn_3;
#[path = "family_t_z/api_only_0.rs"]
mod api_only_0;
#[path = "family_t_z/api_only_1.rs"]
mod api_only_1;
#[path = "family_t_z/operations_0.rs"]
mod operations_0;
#[path = "family_t_z/operations_1.rs"]
mod operations_1;
#[path = "family_t_z/operations_2.rs"]
mod operations_2;
#[path = "family_t_z/operations_3.rs"]
mod operations_3;
#[path = "family_t_z/operations_4.rs"]
mod operations_4;
#[path = "family_t_z/operations_5.rs"]
mod operations_5;
#[path = "family_t_z/operations_6.rs"]
mod operations_6;
#[path = "family_t_z/operations_7.rs"]
mod operations_7;
#[path = "family_t_z/operations_8.rs"]
mod operations_8;
#[path = "family_t_z/relations_0.rs"]
mod relations_0;
mod completion;
mod completion_relations;

pub(super) const SOURCES: &[MappingSource<'static>] = &[
    MappingSource { cfn: cfn_0::ENTRIES, ..super::EMPTY },
    MappingSource { cfn: cfn_1::ENTRIES, ..super::EMPTY },
    MappingSource { cfn: cfn_2::ENTRIES, ..super::EMPTY },
    MappingSource { cfn: cfn_3::ENTRIES, ..super::EMPTY },
    MappingSource { api_only: api_only_0::ENTRIES, ..super::EMPTY },
    MappingSource { api_only: api_only_1::ENTRIES, ..super::EMPTY },
    MappingSource { operations: operations_0::ENTRIES, ..super::EMPTY },
    MappingSource { operations: operations_1::ENTRIES, ..super::EMPTY },
    MappingSource { operations: operations_2::ENTRIES, ..super::EMPTY },
    MappingSource { operations: operations_3::ENTRIES, ..super::EMPTY },
    MappingSource { operations: operations_4::ENTRIES, ..super::EMPTY },
    MappingSource { operations: operations_5::ENTRIES, ..super::EMPTY },
    MappingSource { operations: operations_6::ENTRIES, ..super::EMPTY },
    MappingSource { operations: operations_7::ENTRIES, ..super::EMPTY },
    MappingSource { operations: operations_8::ENTRIES, ..super::EMPTY },
    MappingSource { relations: relations_0::ENTRIES, ..super::EMPTY },
    MappingSource { cfn: completion::CFN, api_only: completion::API, operations: completion::OPERATIONS,
        relations: completion_relations::OUTPUT, parent_relations: completion_relations::PARENTS, ..super::EMPTY },
];
