use super::*;

#[tokio::test]
async fn given_services_networks_listeners_and_ip_or_lambda_groups_when_summary_pages_execute_then_both_log_parents_rules_and_optional_port_targets_persist() -> TestResult {
    let catalog = RuntimeCatalog::from_bytes(catalog::bytes()?)?;
    let mut scenario = Scenario::new(&catalog, "tz-lattice-completion").await?;
    scenario.seed(&catalog, "s3", &["ListBuckets"]).await?;
    let lease = scenario.store.claim(&scenario.sweep,1000,10000).await?.ok_or("S3 sink discovery work absent")?;
    let buckets = catalog.lookup("s3","ListBuckets").ok_or("S3 sink producer absent")?;
    let (request,wire) = scenario.request(buckets,&lease)?;
    assert_eq!(wire.method(),http::Method::GET);
    assert_eq!(wire.uri().host(),Some("s3.us-east-1.amazonaws.com"));
    scenario.apply_wire(&catalog,buckets,lease,&request,
        b"<ListAllMyBucketsResult xmlns=\"http://s3.amazonaws.com/doc/2006-03-01/\"><Owner><ID>owned-account</ID><DisplayName>inventory</DisplayName></Owner><Buckets><Bucket><Name>inventory-access-logs</Name><CreationDate>2026-01-01T00:00:00Z</CreationDate></Bucket></Buckets></ListAllMyBucketsResult>").await?;
    scenario = scenario.reopen().await?;
    scenario.assert_ids_at("AWS::S3::Bucket","global",vec!["arn:aws:s3:::inventory-access-logs".to_owned()]).await?;
    scenario.seed(&catalog, "vpc-lattice", &["ListServices", "ListServiceNetworks", "ListTargetGroups"]).await?;
    scenario.seed(&catalog, "cloudwatch-logs", &["DescribeLogGroups"]).await?;
    scenario.seed(&catalog, "firehose", &["ListDeliveryStreams"]).await?;
    let (scenario, seen) = scenario.drain(&catalog, "vpc-lattice", |plan, request, wire, resumed| {
        if plan.service_name() != "firehose" { assert_token(wire, "nextToken", resumed)?; }
        let p = if resumed { "2" } else { "1" };
        let row = match plan.operation_name() {
            "ListDeliveryStreams" => {
                let native: Json = serde_json::from_slice(wire.body())?;
                assert_eq!(native.get("ExclusiveStartDeliveryStreamName").map(Json::as_str),
                    if resumed { Some(Some("aws-waf-logs-inventory")) } else { None });
                return Ok(if resumed { json!({"DeliveryStreamNames":["z-final"],"HasMoreDeliveryStreams":false}) }
                    else { json!({"DeliveryStreamNames":["aws-waf-logs-archive","aws-waf-logs-inventory"],"HasMoreDeliveryStreams":true}) });
            }
            "DescribeLogGroups" => return Ok(json!({"logGroups":[{"logGroupName":"inventory","logGroupArn":"arn:aws:logs:us-east-1:123456789012:log-group:inventory","arn":"arn:aws:logs:us-east-1:123456789012:log-group:inventory:*"}]})),
            "ListServices" => return Ok(json!({"items":[{"id":"svc-one","arn":"arn:aws:vpc-lattice:us-east-1:123456789012:service/svc-one","name":"one"},{"id":"svc-two","arn":"arn:aws:vpc-lattice:us-east-1:123456789012:service/svc-two","name":"two"}]})),
            "ListServiceNetworks" => return Ok(json!({"items":[{"id":"sn-one","arn":"arn:aws:vpc-lattice:us-east-1:123456789012:servicenetwork/sn-one","name":"one"},{"id":"sn-two","arn":"arn:aws:vpc-lattice:us-east-1:123456789012:servicenetwork/sn-two","name":"two"}]})),
            "ListTargetGroups" => return Ok(json!({"items":[{"id":"tg-ip","arn":"arn:aws:vpc-lattice:us-east-1:123456789012:targetgroup/tg-ip","name":"ip","type":"IP"},{"id":"tg-lambda","arn":"arn:aws:vpc-lattice:us-east-1:123456789012:targetgroup/tg-lambda","name":"lambda","type":"LAMBDA"}]})),
            "ListListeners" => {
                let service = text(request,"serviceIdentifier")?;
                assert!(matches!(service,"svc-one"|"svc-two"));
                assert_eq!(wire.uri().path(),format!("/services/{service}/listeners"));
                return Ok(json!({"items":[{"id":"listener-shared","arn":format!("arn:aws:vpc-lattice:us-east-1:123456789012:service/{service}/listener/listener-shared"),"name":"listener","protocol":"HTTPS","port":443}]}));
            }
            "ListAccessLogSubscriptions" => {
                let resource = text(request,"resourceIdentifier")?;
                assert!(["service/svc-one","service/svc-two","servicenetwork/sn-one","servicenetwork/sn-two"].iter().any(|suffix| resource.ends_with(suffix)));
                assert_eq!(wire.uri().path(),"/accesslogsubscriptions");
                assert!(wire.uri().query().unwrap_or_default().contains("resourceIdentifier=arn%3Aaws%3Avpc-lattice"));
                let id = resource.rsplit('/').next().ok_or("resource ID absent")?;
                let destination = match id {
                    "svc-two" => "arn:aws:firehose:us-east-1:123456789012:deliverystream/aws-waf-logs-inventory",
                    "sn-one" => "arn:aws:s3:::inventory-access-logs",
                    _ => "arn:aws:logs:us-east-1:123456789012:log-group:inventory",
                };
                json!({"id":format!("als-{id}-{p}"),"arn":format!("arn:aws:vpc-lattice:us-east-1:123456789012:accesslogsubscription/als-{id}-{p}"),"resourceArn":resource,"resourceId":id,"destinationArn":destination})
            }
            "ListRules" => {
                let service = text(request,"serviceIdentifier")?;
                let listener = text(request,"listenerIdentifier")?;
                assert!(matches!(service,"svc-one"|"svc-two"));
                assert_eq!(listener,"listener-shared");
                assert_eq!(wire.uri().path(),format!("/services/{service}/listeners/listener-shared/rules"));
                json!({"id":format!("rule-{p}"),"arn":format!("arn:aws:vpc-lattice:us-east-1:123456789012:service/{service}/listener/listener-shared/rule/rule-{p}"),"name":"rule","priority":if resumed {2} else {1},"isDefault":false})
            }
            "ListTargets" => {
                let group = text(request,"targetGroupIdentifier")?;
                assert!(group.ends_with("/tg-ip") || group.ends_with("/tg-lambda"));
                assert_eq!(wire.method(),http::Method::POST);
                let native: Json = serde_json::from_slice(wire.body())?;
                assert_eq!(native.get("targets"), None, "target filter must not truncate registrations");
                assert!(wire.uri().path().starts_with("/targetgroups/arn"));
                assert!(wire.uri().path().ends_with("/listtargets"));
                if group.ends_with("/tg-ip") {
                    json!({"id":"192.0.2.1","port":if resumed {8443} else {443},"status":"HEALTHY"})
                } else {
                    json!({"id":format!("arn:aws:lambda:us-east-1:123456789012:function:target-{p}"),"status":"HEALTHY"})
                }
            }
            other => return Err(format!("unexpected VPC Lattice operation {other}").into()),
        };
        Ok(page("items",row,!resumed,"nextToken"))
    }).await?;
    assert_eq!(seen.get("ListAccessLogSubscriptions"),Some(&8));
    assert_eq!(seen.get("ListRules"),Some(&4));
    assert_eq!(seen.get("ListTargets"),Some(&4));
    let logs = ["svc-one","svc-two","sn-one","sn-two"].into_iter().flat_map(|id| ["1","2"].map(move |p| format!("arn:aws:vpc-lattice:us-east-1:123456789012:accesslogsubscription/als-{id}-{p}"))).collect();
    scenario.assert_ids("AWS::VpcLattice::AccessLogSubscription",logs).await?;
    let rules = ["svc-one","svc-two"].into_iter().flat_map(|service| ["1","2"].map(move |p| format!("arn:aws:vpc-lattice:us-east-1:123456789012:service/{service}/listener/listener-shared/rule/rule-{p}"))).collect();
    scenario.assert_ids("AWS::VpcLattice::Rule",rules).await?;
    let targets = vec![
        correlated(&["arn:aws:vpc-lattice:us-east-1:123456789012:targetgroup/tg-ip","192.0.2.1","443"]),
        correlated(&["arn:aws:vpc-lattice:us-east-1:123456789012:targetgroup/tg-ip","192.0.2.1","8443"]),
        correlated(&["arn:aws:vpc-lattice:us-east-1:123456789012:targetgroup/tg-lambda","arn:aws:lambda:us-east-1:123456789012:function:target-1"]),
        correlated(&["arn:aws:vpc-lattice:us-east-1:123456789012:targetgroup/tg-lambda","arn:aws:lambda:us-east-1:123456789012:function:target-2"]),
    ];
    let mut resources = Vec::new();
    let mut destinations = Vec::new();
    for (prefix, kind) in [("svc", "AWS::VpcLattice::Service"), ("sn", "AWS::VpcLattice::ServiceNetwork")] {
        for suffix in ["one", "two"] { for p in ["1", "2"] {
            let resource_type = if prefix == "svc" { "service" } else { "servicenetwork" };
            let id = format!("{prefix}-{suffix}");
            let from = format!("arn:aws:vpc-lattice:us-east-1:123456789012:accesslogsubscription/als-{id}-{p}");
            resources.push((from.clone(),kind.to_owned(),format!("arn:aws:vpc-lattice:us-east-1:123456789012:{resource_type}/{id}")));
            let (region,kind,to) = match id.as_str() {
                "svc-two" => ("us-east-1","AWS::KinesisFirehose::DeliveryStream","arn:aws:firehose:us-east-1:123456789012:deliverystream/aws-waf-logs-inventory"),
                "sn-one" => ("global","AWS::S3::Bucket","arn:aws:s3:::inventory-access-logs"),
                _ => ("us-east-1","AWS::Logs::LogGroup","arn:aws:logs:us-east-1:123456789012:log-group:inventory"),
            };
            destinations.push((from,region.to_owned(),kind.to_owned(),to.to_owned()));
        }}
    }
    scenario.assert_edges("AWS::VpcLattice::AccessLogSubscription","FOR_RESOURCE",resources).await?;
    scenario.assert_edge_regions("AWS::VpcLattice::AccessLogSubscription","LOGS_TO","us-east-1",destinations).await?;
    let mut listeners = Vec::new();
    let mut services = Vec::new();
    for service in ["svc-one","svc-two"] { for p in ["1","2"] {
        let from = format!("arn:aws:vpc-lattice:us-east-1:123456789012:service/{service}/listener/listener-shared/rule/rule-{p}");
        listeners.push((from.clone(),"AWS::VpcLattice::Listener".to_owned(),format!("arn:aws:vpc-lattice:us-east-1:123456789012:service/{service}/listener/listener-shared")));
        services.push((from,"AWS::VpcLattice::Service".to_owned(),format!("arn:aws:vpc-lattice:us-east-1:123456789012:service/{service}")));
    }}
    scenario.assert_edges("AWS::VpcLattice::Rule","IN_LISTENER",listeners).await?;
    scenario.assert_edges("AWS::VpcLattice::Rule","IN_SERVICE",services).await?;
    let group_edges = targets.iter().enumerate().map(|(index,from)|
        (from.clone(),"AWS::VpcLattice::TargetGroup".to_owned(),format!("arn:aws:vpc-lattice:us-east-1:123456789012:targetgroup/{}",if index < 2 { "tg-ip" } else { "tg-lambda" }))).collect();
    scenario.assert_edges("AWS::ApiOnly::VpcLatticeTargetRegistration","IN_TARGET_GROUP",group_edges).await?;
    scenario.assert_ids("AWS::ApiOnly::VpcLatticeTargetRegistration",targets).await?;
    Ok(())
}
