#[path = "support/required_parent.rs"]
mod fixture;
use fixture::{Case, TestResult};

#[test]
fn given_exchange_status_when_compiled_then_reserved_node_id_is_required() -> TestResult {
    fixture::verify(Case {
        service: "redshift",
        date: "2012-12-01",
        operation: "DescribeReservedNodeExchangeStatus",
        parent: "DescribeReservedNodes",
        input: "ReservedNodeId",
        output: "ReservedNodeId",
    })
}
