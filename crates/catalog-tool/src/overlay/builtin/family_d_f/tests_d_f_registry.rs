use crate::classes::{DenyList, DenyReason};
use crate::overlay::{
    self, OperationName, OverlayClass, ServiceOverlay,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn operation<'a>(
    policies: &'a [crate::overlay::ServiceOverlay],
    service: &str,
    name: &str,
) -> Result<&'a crate::overlay::OperationOverlay, Box<dyn std::error::Error>> {
    policies
        .iter()
        .find(|policy| policy.service() == service)
        .and_then(|policy| {
            policy
                .operations()
                .get(&OperationName::new(name).ok()?)
        })
        .ok_or_else(|| format!("missing policy {service}/{name}").into())
}

const SERVICES: &[(&str, usize)] = &[
    ("data-pipeline", 19),
    ("database-migration-service", 119),
    ("databrew", 44),
    ("dataexchange", 37),
    ("datasync", 53),
    ("datazone", 195),
    ("dax", 21),
    ("deadline", 127),
    ("detective", 29),
    ("device-farm", 77),
    ("devops-agent", 63),
    ("devops-guru", 31),
    ("direct-connect", 73),
    ("directory-service-data", 17),
    ("directory-service", 80),
    ("dlm", 8),
    ("docdb-elastic", 19),
    ("docdb", 56),
    ("drs", 70),
    ("dsql", 16),
    ("dynamodb-streams", 4),
    ("ebs", 6),
    ("ec2-instance-connect", 2),
    ("ecr-public", 23),
    ("ecr", 62),
    ("efs", 31),
    ("eks-auth", 1),
    ("elastic-beanstalk", 47),
    ("elastic-load-balancing", 29),
    ("elasticache", 77),
    ("elasticsearch-service", 52),
    ("elementalinference", 24),
    ("emr-containers", 25),
    ("emr-serverless", 22),
    ("emr", 65),
    ("endusermessaging", 27),
    ("entityresolution", 38),
    ("eventbridge", 57),
    ("eventbridgev2", 25),
    ("evs", 24),
    ("firehose", 12),
    ("fis", 26),
    ("fms", 42),
    ("forecast", 63),
    ("forecastquery", 2),
    ("frauddetector", 73),
    ("freetier", 5),
    ("fsx", 48),
];

#[test]
fn public_registry_dispositions_cover_every_pinned_d_f_service_and_operation() -> TestResult {
    // Given the exact pinned service/operation counts, excluding the first-wave services.
    let policies = overlay::embedded()?;
    let mut total = 0;
    let mut callable = 0;
    // When consumers read the public embedded registry.
    for (service, expected_count) in [
        ("data-pipeline", 19),
        ("database-migration-service", 119),
        ("databrew", 44),
        ("dataexchange", 37),
        ("datasync", 53),
        ("datazone", 195),
        ("dax", 21),
        ("deadline", 127),
        ("detective", 29),
        ("device-farm", 77),
        ("devops-agent", 63),
        ("devops-guru", 31),
        ("direct-connect", 73),
        ("directory-service-data", 17),
        ("directory-service", 80),
        ("dlm", 8),
        ("docdb-elastic", 19),
        ("docdb", 56),
        ("drs", 70),
        ("dsql", 16),
        ("dynamodb-streams", 4),
        ("ebs", 6),
        ("ec2-instance-connect", 2),
        ("ecr-public", 23),
        ("ecr", 62),
        ("efs", 31),
        ("eks-auth", 1),
        ("elastic-beanstalk", 47),
        ("elastic-load-balancing", 29),
        ("elasticache", 77),
        ("elasticsearch-service", 52),
        ("elementalinference", 24),
        ("emr-containers", 25),
        ("emr-serverless", 22),
        ("emr", 65),
        ("endusermessaging", 27),
        ("entityresolution", 38),
        ("eventbridge", 57),
        ("eventbridgev2", 25),
        ("evs", 24),
        ("firehose", 12),
        ("fis", 26),
        ("fms", 42),
        ("forecast", 63),
        ("forecastquery", 2),
        ("frauddetector", 73),
        ("freetier", 5),
        ("fsx", 48),
    ] {
        let policy = policies
            .iter()
            .find(|p| p.service() == service)
            .ok_or(service)?;
        assert_eq!(policy.operations().len(), expected_count, "{service}");
        total += policy.operations().len();
        // Then every operation has an explicit disposition, never inferred read-name approval.
        for (operation, plan) in policy.operations().iter() {
            match plan.class {
                Some(OverlayClass::Enumerate | OverlayClass::Expand) => {
                    callable += 1;
                    if plan.items.is_none() {
                        eprintln!("DMS missing items: {operation}");
                    }
                    assert_ne!(plan.items, None);
                    if plan.identity.is_none() {
                        eprintln!("DMS missing identity: {operation}");
                    }
                    assert_ne!(plan.identity, None);
                    if plan.paginator.is_none() {
                        eprintln!("DMS missing paginator: {operation}");
                    }
                    assert_ne!(plan.paginator, None);
                    assert_ne!(plan.scope, None);
                }
                Some(OverlayClass::Deny(_)) => assert_eq!(plan.items, None),
                Some(OverlayClass::Defer) => {
                    assert_ne!(plan.defer_reason.as_deref().unwrap_or_default(), "");
                    assert_eq!(plan.items, None);
                    assert_eq!(plan.identity, None);
                }
                other => return Err(format!("unexpected {service} disposition: {other:?}").into()),
            }
        }
    }
    assert_eq!(total, 2066);
    assert_eq!(callable, 213);
    Ok(())
}