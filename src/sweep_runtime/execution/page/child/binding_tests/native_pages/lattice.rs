use super::*;
use super::super::persistence::key;
use inventory_engine::inventory::WorkLease;
const OWNERS: [(&str, &str, &str, &str); 2] = [
    ("ListServices", "AWS::VpcLattice::Service", "service/svc-11111111111111111", "als-11111111111111111"),
    ("ListServiceNetworks", "AWS::VpcLattice::ServiceNetwork", "servicenetwork/sn-22222222222222222", "als-22222222222222222"),
];
fn arn(resource: &str) -> String { format!("arn:aws:vpc-lattice:us-east-1:123456789012:{resource}") }
fn escaped(text: &str) -> String { text.replace(':', "%3A").replace('/', "%2F") }

pub(super) fn page(plan: &CallablePlan<'_>, lease: &WorkLease, wire: &http::Request<bytes::Bytes>, _reverse: bool) -> TestResult<Json> {
    assert_eq!(wire.method(), http::Method::GET);
    assert!(wire.body().is_empty());
    let first = lease.token().is_none();
    let query = wire.uri().query().unwrap_or_default();
    assert_eq!(query.split('&').any(|pair| pair == "nextToken=native-next"), !first);
    if let Some((_, _, resource, _)) = OWNERS.iter().find(|(operation, _, _, _)| *operation == plan.operation_name()) {
        return Ok(if first { json!({"items":[],"nextToken":"native-next"}) }
            else { json!({"items":[{"arn":arn(resource),"id":"opaque-id","name":"resource"}]}) });
    }
    assert_eq!(plan.operation_name(), "ListAccessLogSubscriptions");
    let uri = wire.uri().to_string();
    let matching = OWNERS.iter().filter(|(_, _, resource, _)| uri.contains(&escaped(&arn(resource)))).collect::<Vec<_>>();
    assert_eq!(matching.len(), 1, "native URI did not bind exactly the originating producer ARN: {uri}");
    assert!(!uri.contains("opaque-id"));
    let (_, _, resource, subscription) = matching.first().copied().ok_or("native owner absent")?;
    Ok(if first { json!({"items":[],"nextToken":"native-next"}) }
        else { json!({"items":[{"arn":arn(&format!("accesslogsubscription/{subscription}")),"id":subscription,"resourceArn":arn(resource),"resourceId":"opaque-id"}]}) })
}

pub(super) fn expected() -> (BTreeSet<Resource>, BTreeSet<Edge>) {
    let mut resources = BTreeSet::new();
    let mut edges = BTreeSet::new();
    for (_, kind, resource, subscription) in OWNERS {
        let owner = key("us-east-1", kind, &arn(resource));
        let child = key("us-east-1", "AWS::VpcLattice::AccessLogSubscription", &arn(&format!("accesslogsubscription/{subscription}")));
        resources.insert(owner.clone());
        resources.insert(child.clone());
        edges.insert((child, "FOR_RESOURCE".to_owned(), owner));
    }
    (resources, edges)
}
