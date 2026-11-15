mod config;
mod model;
mod runner;
mod scenario;
mod wire;
mod literal;

pub use runner::{run_data_scenario, run_scenario, run_seed};

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
