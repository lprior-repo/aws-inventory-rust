use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    ("CreateAssertion", Decision::Denied(DenyReason::Mutating)),
    ("CreateInputSource", Decision::Denied(DenyReason::Mutating)),
    ("CreatePolicy", Decision::Denied(DenyReason::Mutating)),
    ("CreateReport", Decision::Denied(DenyReason::Mutating)),
    ("CreateService", Decision::Denied(DenyReason::Mutating)),
    (
        "CreateServiceFunction",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateServiceFunctionResources",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateSystem", Decision::Denied(DenyReason::Mutating)),
    ("CreateTest", Decision::Denied(DenyReason::Mutating)),
    ("CreateUserJourney", Decision::Denied(DenyReason::Mutating)),
    ("DeleteAssertion", Decision::Denied(DenyReason::Mutating)),
    ("DeleteInputSource", Decision::Denied(DenyReason::Mutating)),
    ("DeletePolicy", Decision::Denied(DenyReason::Mutating)),
    ("DeleteService", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteServiceFunction",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteServiceFunctionResources",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteSystem", Decision::Denied(DenyReason::Mutating)),
    ("DeleteTest", Decision::Denied(DenyReason::Mutating)),
    ("DeleteTestSources", Decision::Denied(DenyReason::Mutating)),
    ("DeleteUserJourney", Decision::Denied(DenyReason::Mutating)),
    (
        "GetDependencyInsights",
        Decision::Deferred(
            "Required selectors serviceArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members overview, insights, status, createdAt, errorCode, errorMessage. Pinned purpose: Retrieves the dependency insights generated for a service. The response reports the current generation status; insights are populated once generation has completed. If generation failed, the response includes an error code, whose possible values are ",
        ),
    ),
    (
        "GetFailureModeFinding",
        Decision::Deferred(
            "Required selectors findingId, serviceArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members finding. Pinned purpose: Retrieves a finding by findingId.",
        ),
    ),
    (
        "GetPolicy",
        Decision::Deferred(
            "Required selectors policyArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members policy. Pinned purpose: Retrieves a resilience policy by ARN.",
        ),
    ),
    (
        "GetService",
        Decision::Deferred(
            "Required selectors serviceArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members service. Pinned purpose: Retrieves a service by ARN.",
        ),
    ),
    (
        "GetSystem",
        Decision::Deferred(
            "Required selectors systemArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members system. Pinned purpose: Retrieves a system by ARN.",
        ),
    ),
    (
        "GetTest",
        Decision::Deferred(
            "Required selectors testId, serviceArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members test. Pinned purpose: Retrieves a test by ID.",
        ),
    ),
    (
        "GetTestRun",
        Decision::Deferred(
            "Required selectors testRunId, serviceArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members testRun. Pinned purpose: Retrieves a test run by ID, including its status, results, and the configuration snapshotted when the run started.",
        ),
    ),
    (
        "GetTestTemplate",
        Decision::Deferred(
            "Required selectors testTemplateArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members testTemplate. Pinned purpose: Retrieves a resilience test template by ARN, including the parameters it accepts and the fault actions it runs.",
        ),
    ),
    (
        "GetUserJourney",
        Decision::Deferred(
            "Required selectors systemArn, userJourneyId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members userJourney. Pinned purpose: Retrieves a user journey.",
        ),
    ),
    ("ImportApp", Decision::Denied(DenyReason::Mutating)),
    ("ImportPolicy", Decision::Denied(DenyReason::Mutating)),
    (
        "ListAssertions",
        Decision::Deferred(
            "Required selectors serviceArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members assertions, nextToken. Pinned purpose: Lists resilience assertions for a service.",
        ),
    ),
    (
        "ListFailureModeAssessments",
        Decision::Deferred(
            "Required selectors serviceArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members assessmentSummaries, nextToken. Pinned purpose: Lists failure mode assessments.",
        ),
    ),
    (
        "ListFailureModeFindings",
        Decision::Deferred(
            "Required selectors serviceArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members findingsSummary, nextToken. Pinned purpose: List findings.",
        ),
    ),
    (
        "ListInputSources",
        Decision::Deferred(
            "Required selectors serviceArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members inputSourceSummaries, nextToken. Pinned purpose: Lists input sources for a service.",
        ),
    ),
    (
        "ListPolicyEvents",
        Decision::Deferred(
            "Required selectors policyArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members events, nextToken. Pinned purpose: Lists events for a resilience policy, including services that started or stopped using it, changes to cross-account sharing, and deletion of the policy.",
        ),
    ),
    (
        "ListReports",
        Decision::Deferred(
            "The optional-input response does not establish an immutable locally scoped owned-resource identity. Response members reportGenerationResults, nextToken. Pinned request/ownership contract: List reports for a service, or all reports owned by the account if serviceArn is not provided.",
        ),
    ),
    (
        "ListResolvedTestRunTargetResources",
        Decision::Deferred(
            "Required selectors testRunId, serviceArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members resolvedTargetResources, nextToken. Pinned purpose: Lists the AWS resources that AWS Fault Injection Service (AWS FIS) resolved as targets for a test run.",
        ),
    ),
    (
        "ListResources",
        Decision::Deferred(
            "Required selectors serviceArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members serviceFunctionId, serviceResources, nextToken. Pinned purpose: List resources.",
        ),
    ),
    (
        "ListServiceEvents",
        Decision::Deferred(
            "Required selectors serviceArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members events, nextToken. Pinned purpose: Lists events for a service.",
        ),
    ),
    (
        "ListServiceFunctions",
        Decision::Deferred(
            "Required selectors serviceArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members serviceFunctions, nextToken. Pinned purpose: Lists service functions for a service.",
        ),
    ),
    (
        "ListServiceTopologyEdges",
        Decision::Deferred(
            "Required selectors serviceArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members serviceTopologyEdgeSummaries, nextToken. Pinned purpose: Lists topology edges for a service.",
        ),
    ),
    (
        "ListSystemEvents",
        Decision::Deferred(
            "Required selectors systemArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members events, nextToken. Pinned purpose: Lists events for a system.",
        ),
    ),
    (
        "ListTagsForResource",
        Decision::Deferred(
            "Tag-only observation must join the originating resource identity with complete-snapshot semantics, not be persisted as a separate resource. Required request members: resourceArn. Pinned purpose: Lists the tags for a resource.",
        ),
    ),
    (
        "ListTestRunDependencies",
        Decision::Deferred(
            "Required selectors testRunId, serviceArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members dependencies, nextToken. Pinned purpose: Lists the dependencies that a test run blocked. Each dependency reflects the discovered classification captured when the run started, so results do not change if a dependency is reclassified after the run.",
        ),
    ),
    (
        "ListTestRunEvents",
        Decision::Deferred(
            "Required selectors testRunId, serviceArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members events, nextToken. Pinned purpose: Lists the events in a test run's timeline.",
        ),
    ),
    (
        "ListTestRunSourceEvents",
        Decision::Deferred(
            "Required selectors testRunId, serviceArn, sourceArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members testRunSourceEvents, nextToken. Pinned purpose: Lists the state-change events observed for a test run monitoring source. Events are returned for one source per call, in chronological order.",
        ),
    ),
    (
        "ListTestRunSources",
        Decision::Deferred(
            "Required selectors testRunId, serviceArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members testRunSources, nextToken. Pinned purpose: Lists the monitoring source snapshots captured for a test run, optionally filtered by type.",
        ),
    ),
    (
        "ListTestRuns",
        Decision::Deferred(
            "Required selectors serviceArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members testRuns, nextToken. Pinned purpose: Lists the runs of a test, or all test runs for a service.",
        ),
    ),
    (
        "ListTestSources",
        Decision::Deferred(
            "Required selectors testId, serviceArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members testSources, nextToken. Pinned purpose: Lists the monitoring sources attached to a test, optionally filtered by type.",
        ),
    ),
    (
        "ListTestTemplates",
        Decision::Denied(DenyReason::ReferenceData),
    ),
    (
        "ListTests",
        Decision::Deferred(
            "Required selectors serviceArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members tests, nextToken. Pinned purpose: Lists the tests configured for a service.",
        ),
    ),
    (
        "ListUserJourneys",
        Decision::Deferred(
            "Required selectors systemArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members userJourneySummaries, nextToken. Pinned purpose: Lists user journeys for a system.",
        ),
    ),
    ("PutTestSources", Decision::Denied(DenyReason::Mutating)),
    (
        "StartDependencyInsights",
        Decision::Denied(DenyReason::Mutating),
    ),
];
