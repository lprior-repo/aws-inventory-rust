pub mod command;
mod config;
pub mod draft;
mod file;
mod literal;
mod model;
mod report;
mod runner;
mod scenario;
mod wire;

const MAX_VALUE_BYTES: usize = 1_048_576;

pub use file::load;
pub use report::{SeedRunReport, StepOutcome};
pub use runner::{
    run_data_scenario, run_report, run_report_with_concurrency, run_scenario, run_seed,
};

pub use config::{SeedConfig, SeedError};
pub use scenario::{
    FieldPath, SeedIdentity, SeedIdentityEvidence, SeedInput, SeedInputValue, SeedNestedBinding,
    SeedOutputCheck, SeedRelationship, SeedScenario, SeedScope, SeedStep, SeedTemplate,
    SeedValuePart, StepId, default_scenario,
};

use inventory_core::inventory::{AccountId, Edge, Kind, Region, Relation, ResourceId, ResourceKey};

#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct SeedResourcePair {
    pub vpc_id: ResourceId,
    pub subnet_id: ResourceId,
    pub vpc_key: ResourceKey,
    pub subnet_key: ResourceKey,
    pub relationship: Relation,
    pub edge: Edge,
}

#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct SeedResourceIdentity {
    pub step: StepId,
    pub kind: Kind,
    pub id: ResourceId,
    pub key: ResourceKey,
    pub evidence: SeedIdentityEvidence,
}

#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct ScenarioSeedOracle {
    pub account_id: AccountId,
    pub region: Region,
    pub resources: Vec<SeedResourceIdentity>,
    pub edges: Vec<Edge>,
}
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct TypedSeedOracle {
    pub account_id: AccountId,
    pub region: Region,
    pub vpc_id: ResourceId,
    pub subnet_id: ResourceId,
    pub vpc_key: ResourceKey,
    pub subnet_key: ResourceKey,
    pub relationship: Relation,
    pub edge: Edge,
    pub pairs: Vec<SeedResourcePair>,
    pub resources: Vec<SeedResourceIdentity>,
    pub edges: Vec<Edge>,
}
