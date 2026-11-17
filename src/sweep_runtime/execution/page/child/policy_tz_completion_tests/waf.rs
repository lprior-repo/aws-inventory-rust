use super::*;

#[tokio::test]
async fn given_global_and_regional_classic_waf_when_logging_pages_and_both_association_types_execute_then_markers_scopes_and_bound_webacl_owners_are_durable() -> TestResult {
    let catalog = RuntimeCatalog::from_bytes(catalog::bytes()?)?;
    for service in ["waf","waf-regional"] {
        let scenario = Scenario::new(&catalog,&format!("tz-{service}-completion")).await?;
        scenario.seed_firehose(&catalog).await?;
        scenario.seed(&catalog,service,&["ListLoggingConfigurations","ListWebACLs"]).await?;
        let (scenario, seen) = scenario.drain(&catalog,service,|plan,request,wire,resumed| {
            let native: Json = serde_json::from_slice(wire.body())?;
            assert_eq!(wire.method(),http::Method::POST);
            match plan.operation_name() {
                "ListDeliveryStreams" => {
                    assert_eq!(native.get("DeliveryStreamType"),None,"stream type must not narrow inventory");
                    assert_eq!(native.get("ExclusiveStartDeliveryStreamName").map(Json::as_str),
                        if resumed { Some(Some("aws-waf-logs-inventory")) } else { None });
                    Ok(if resumed { json!({"DeliveryStreamNames":["z-final"],"HasMoreDeliveryStreams":false}) }
                        else { json!({"DeliveryStreamNames":["aws-waf-logs-archive","aws-waf-logs-inventory"],"HasMoreDeliveryStreams":true}) })
                }
                "ListLoggingConfigurations" => {
                    assert_token(wire,"NextMarker",resumed)?;
                    assert_eq!(native.get("Limit"),Some(&json!(100)));
                    assert_eq!(plan.scope(),if service == "waf" { PlanScope::Account } else { PlanScope::Region });
                    let region = if service == "waf" { "" } else { "us-east-1" };
                    let p = if resumed { "2" } else { "1" };
                    Ok(page("LoggingConfigurations",json!({"ResourceArn":format!("arn:aws:{service}:{region}:123456789012:webacl/acl-{p}"),"LogDestinationConfigs":["arn:aws:firehose:us-east-1:123456789012:deliverystream/aws-waf-logs-inventory"]}),!resumed,"NextMarker"))
                }
                "ListWebACLs" => Ok(json!({"WebACLs":[{"WebACLId":"acl-1","Name":"one"},{"WebACLId":"acl-2","Name":"two"}]})),
                "GetWebACL" => {
                    let acl = text(request,"WebACLId")?;
                    assert!(matches!(acl,"acl-1"|"acl-2"));
                    assert_eq!(native.get("WebACLId").and_then(Json::as_str),Some(acl));
                    let region = if service == "waf" { "" } else { "us-east-1" };
                    Ok(json!({"WebACL":{"WebACLId":acl,"WebACLArn":format!("arn:aws:{service}:{region}:123456789012:webacl/{acl}"),"Name":acl,"DefaultAction":{"Type":"ALLOW"},"Rules":[]}}))
                }
                "ListResourcesForWebACL" => {
                    let acl = text(request,"WebACLId")?;
                    let kind = text(request,"ResourceType")?;
                    assert!(matches!(acl,"acl-1"|"acl-2"));
                    assert!(matches!(kind,"APPLICATION_LOAD_BALANCER"|"API_GATEWAY"));
                    assert_eq!(native.get("WebACLId").and_then(Json::as_str),Some(acl));
                    assert_eq!(native.get("ResourceType").and_then(Json::as_str),Some(kind));
                    assert!(!resumed);
                    let arn = if kind == "APPLICATION_LOAD_BALANCER" { "arn:aws:elasticloadbalancing:us-east-1:123456789012:loadbalancer/app/shared/1234567890123456" } else { "arn:aws:apigateway:us-east-1::/restapis/shared/stages/prod" };
                    Ok(json!({"ResourceArns":[arn]}))
                }
                other => Err(format!("unexpected WAF operation {other}").into()),
            }
        }).await?;
        assert_eq!(seen.get("ListLoggingConfigurations"),Some(&2));
        let region = if service == "waf" { "" } else { "us-east-1" };
        let expected = ["1","2"].map(|p| format!("arn:aws:{service}:{region}:123456789012:webacl/acl-{p}")).to_vec();
        let logging_kind = if service == "waf" { "AWS::ApiOnly::WAFLoggingConfiguration" } else { "AWS::ApiOnly::WAFRegionalLoggingConfiguration" };
        let stored_region = if service == "waf" { "global" } else { "us-east-1" };
        scenario.assert_ids_at(logging_kind,stored_region,expected).await?;
        let web_acl_kind = if service == "waf" { "AWS::WAF::WebACL" } else { "AWS::WAFRegional::WebACL" };
        scenario.assert_ids_at(web_acl_kind,stored_region,vec!["acl-1".to_owned(),"acl-2".to_owned()]).await?;
        scenario.assert_edges_at(logging_kind,"FOR_WEB_ACL",stored_region,stored_region,
            ["1","2"].map(|p| (format!("arn:aws:{service}:{region}:123456789012:webacl/acl-{p}"),web_acl_kind.to_owned(),format!("acl-{p}"))).to_vec()).await?;
        scenario.assert_edges_at(logging_kind,"LOGS_TO",stored_region,"us-east-1",
            ["1","2"].map(|p| (format!("arn:aws:{service}:{region}:123456789012:webacl/acl-{p}"),
                "AWS::KinesisFirehose::DeliveryStream".to_owned(),
                "arn:aws:firehose:us-east-1:123456789012:deliverystream/aws-waf-logs-inventory".to_owned())).to_vec()).await?;
        assert_eq!(seen.get("ListDeliveryStreams"),Some(&2));
        scenario.assert_ids("AWS::KinesisFirehose::DeliveryStream",
            ["aws-waf-logs-archive","aws-waf-logs-inventory","z-final"].map(|name|
                format!("arn:aws:firehose:us-east-1:123456789012:deliverystream/{name}")).to_vec()).await?;
        if service == "waf-regional" {
            assert_eq!(seen.get("ListResourcesForWebACL"),Some(&4));
            let mut expected = Vec::new();
            let mut parents = Vec::new();
            for acl in ["acl-1","acl-2"] { for (kind,arn) in [
                ("APPLICATION_LOAD_BALANCER","arn:aws:elasticloadbalancing:us-east-1:123456789012:loadbalancer/app/shared/1234567890123456"),
                ("API_GATEWAY","arn:aws:apigateway:us-east-1::/restapis/shared/stages/prod"),
            ] {
                let id = correlated(&[acl,kind,arn]);
                expected.push(id.clone());
                parents.push((id,web_acl_kind.to_owned(),acl.to_owned()));
            }}
            scenario.assert_ids("AWS::WAFRegional::WebACLAssociation",expected).await?;
            scenario.assert_edges("AWS::WAFRegional::WebACLAssociation","FOR_WEB_ACL",parents).await?;
        }
    }
    Ok(())
}
