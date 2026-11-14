use super::{Suite, find_operation, model::compile_service, run_case};
use inventory_core::catalog::Catalog;
use serde_json::Value as Json;
use std::collections::BTreeSet;

#[derive(Default)]
struct Ledger {
    clients: BTreeSet<String>,
    servers: BTreeSet<String>,
    events: usize,
    failures: Vec<String>,
}

pub(super) fn run_suite(suite: Suite) -> Vec<String> {
    let fixture: Json = serde_json::from_str(suite.fixture).expect("published fixture JSON");
    let shapes = fixture["shapes"].as_object().expect("fixture shapes");
    let mut ledger = Ledger::default();
    for service_id in service_ids(shapes) {
        run_service(suite, &fixture, shapes, &service_id, &mut ledger);
    }
    let mut failures = verify_ledger(suite, &ledger);
    failures.append(&mut ledger.failures);
    failures
}

fn service_ids(shapes: &serde_json::Map<String, Json>) -> Vec<String> {
    shapes
        .iter()
        .filter_map(|(id, shape)| (shape["type"] == "service").then(|| id.clone()))
        .collect()
}

fn run_service(
    suite: Suite,
    fixture: &Json,
    shapes: &serde_json::Map<String, Json>,
    service_id: &str,
    ledger: &mut Ledger,
) {
    let Some(service_shape) = protocol_service(suite, shapes, service_id) else {
        return;
    };
    let bytes = compile_service(fixture, service_id);
    let catalog = Catalog::borrow(&bytes).expect("compiled fixture catalog");
    let name = service_name(service_id);
    let service = catalog
        .service(&name)
        .expect("service lookup")
        .expect("compiled service");
    for reference in service_shape["operations"]
        .as_array()
        .expect("service operation list")
    {
        process_operation(suite, shapes, service, reference, ledger);
    }
}

fn protocol_service<'a>(
    suite: Suite,
    shapes: &'a serde_json::Map<String, Json>,
    service_id: &str,
) -> Option<&'a Json> {
    let shape = shapes.get(service_id)?;
    shape["traits"].get(suite.protocol_id).map(|_| shape)
}

fn service_name(service_id: &str) -> String {
    service_id
        .rsplit('#')
        .next()
        .expect("service name")
        .replace('.', "-")
}

fn process_operation(
    suite: Suite,
    shapes: &serde_json::Map<String, Json>,
    service: inventory_core::catalog::Service<'_>,
    reference: &Json,
    ledger: &mut Ledger,
) {
    let operation_id = reference["target"].as_str().expect("operation target");
    let name = operation_id.rsplit('#').next().expect("operation name");
    let operation = find_operation(service, name);
    let operation_shape = &shapes[operation_id];
    let cases = operation_shape["traits"]["smithy.test#httpRequestTests"].as_array();
    if let Some(cases) = cases {
        run_cases(suite, service, operation, operation_id, name, cases, ledger);
    }
    ledger.events += operation_shape["traits"]["smithy.test#eventStreamTests"]
        .as_array()
        .map_or(0, Vec::len);
}

fn run_cases(
    suite: Suite,
    service: inventory_core::catalog::Service<'_>,
    operation: inventory_core::catalog::Operation<'_>,
    operation_id: &str,
    operation_name: &str,
    cases: &[Json],
    ledger: &mut Ledger,
) {
    for case in cases {
        if case["appliesTo"] == "server" {
            record_server(case, ledger);
        } else {
            run_client_case(
                suite,
                service,
                operation,
                operation_id,
                operation_name,
                case,
                ledger,
            );
        }
    }
}

fn record_server(case: &Json, ledger: &mut Ledger) {
    ledger
        .servers
        .insert(case["id"].as_str().expect("server case id").to_owned());
}

fn run_client_case(
    suite: Suite,
    service: inventory_core::catalog::Service<'_>,
    operation: inventory_core::catalog::Operation<'_>,
    operation_id: &str,
    operation_name: &str,
    case: &Json,
    ledger: &mut Ledger,
) {
    assert!(
        case["appliesTo"]
            .as_str()
            .map_or(true, |value| value == "client" || value == "both")
    );
    let id = format!(
        "{operation_id}::{}",
        case["id"].as_str().expect("client case id")
    );
    assert!(
        ledger.clients.insert(id.clone()),
        "duplicate fixture ledger entry {id}"
    );
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        run_case(suite, service, operation, operation_name, case)
    }));
    if outcome.is_err() {
        ledger.failures.push(format!("{} {id}", suite.name));
    }
}

fn verify_ledger(suite: Suite, ledger: &Ledger) -> Vec<String> {
    let name = suite.name;
    let servers = suite
        .servers
        .iter()
        .map(|id| (*id).to_owned())
        .collect::<BTreeSet<_>>();
    let unmatched = ledger.servers.symmetric_difference(&servers);
    [
        (ledger.clients.len() != suite.clients)
            .then(|| format!("{name} client ledger has {}", ledger.clients.len())),
        (ledger.servers != servers).then(|| {
            format!(
                "{name} server ledger differs: {:?}",
                unmatched.collect::<Vec<_>>()
            )
        }),
        (ledger.events != suite.events)
            .then(|| format!("{name} event-stream ledger has {}", ledger.events)),
    ]
    .into_iter()
    .flatten()
    .collect()
}
