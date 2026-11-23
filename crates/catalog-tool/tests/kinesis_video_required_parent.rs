#[path = "support/required_parent.rs"]
mod fixture;
use fixture::{Case, TestResult};

#[test]
fn given_mapped_resources_when_compiled_then_stream_arn_is_required() -> TestResult {
    fixture::verify(Case {
        service: "kinesis-video",
        date: "2017-09-30",
        operation: "DescribeMappedResourceConfiguration",
        parent: "ListStreams",
        input: "StreamARN",
        output: "StreamARN",
    })
}
