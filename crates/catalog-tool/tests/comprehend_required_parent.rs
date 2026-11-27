#[path = "support/required_parent.rs"]
mod fixture;
use fixture::{Case, TestResult};

#[test]
fn given_flywheels_when_collecting_datasets_then_the_parent_arn_is_bound() -> TestResult {
    fixture::verify(Case {
        service: "comprehend",
        date: "2017-11-27",
        operation: "ListDatasets",
        parent: "ListFlywheels",
        input: "FlywheelArn",
        output: "FlywheelArn",
    })
}
