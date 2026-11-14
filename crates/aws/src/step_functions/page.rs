use std::collections::BTreeSet;

use aws_sdk_sfn::operation::{
    list_activities::ListActivitiesOutput, list_state_machines::ListStateMachinesOutput,
};
use inventory_core::{Edge, Resource};

use crate::common::PageBody;

pub(super) const LIST_MAX_RESULTS: i32 = 1_000;

#[derive(Clone)]
pub(super) struct Observation {
    pub resource: Resource,
    pub edges: Vec<Edge>,
}

pub(super) fn page_limit(page_size: i32) -> i32 {
    page_size.clamp(1, LIST_MAX_RESULTS)
}

pub(super) fn state_machine_page_body(
    output: &ListStateMachinesOutput,
    observations: Vec<Observation>,
) -> PageBody {
    assemble(
        observations,
        output.next_token(),
        output.state_machines().len(),
    )
}

pub(super) fn activity_page_body(
    output: &ListActivitiesOutput,
    observations: Vec<Observation>,
) -> PageBody {
    assemble(observations, output.next_token(), output.activities().len())
}

fn assemble(observations: Vec<Observation>, token: Option<&str>, listed: usize) -> PageBody {
    let mut body = PageBody {
        next_token: token.map(str::to_owned),
        listed_count: listed,
        ..PageBody::default()
    };
    for observation in observations {
        body.resources.push(observation.resource);
        body.edges.extend(observation.edges);
    }
    body
}

pub(super) fn dedupe(observations: Vec<Observation>) -> Vec<Observation> {
    let mut seen = BTreeSet::new();
    observations
        .into_iter()
        .filter(|observation| seen.insert(observation.resource.id.clone()))
        .collect()
}
