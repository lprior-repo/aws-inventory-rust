use super::*;

#[tokio::test]
async fn given_registered_and_deprecated_swf_domains_when_both_type_statuses_page_then_each_domain_keeps_its_own_types() -> TestResult {
    let catalog = fixture::catalog()?;
    let scenario = Scenario::new(&catalog, "swf-status-branches", "us-east-1").await?;
    scenario.seed(fixture::plan(&catalog,"swf","ListDomains")?).await?;
    let mut seen = BTreeSet::new();
    scenario.run(&catalog,36,|plan,lease,wire| {
        let request = body(wire)?;
        let status = text(&request,"registrationStatus")?;
        assert!(matches!(status,"REGISTERED"|"DEPRECATED"));
        let second = lease.token().is_some();
        assert_eq!(request.get("nextPageToken").and_then(Json::as_str),second.then_some("page-two"));
        let page = if second {"b"} else {"a"};
        if plan.operation_name() == "ListDomains" {
            let domain = format!("{status}-{page}");
            return paginated(json!({"domainInfos":[{"name":domain,"status":status,"arn":format!("arn:aws:swf:us-east-1:123456789012:/domain/{domain}")}]}),"nextPageToken",second);
        }
        let domain = text(&request,"domain")?;
        assert!(seen.insert((plan.operation_name().to_owned(),domain.to_owned(),status.to_owned(),second)));
        let member = if plan.operation_name() == "ListActivityTypes" {"activityType"} else {"workflowType"};
        paginated(json!({"typeInfos":[{member:{"name":"same-name","version":format!("{status}-{page}")},"status":status,"creationDate":0}]}),"nextPageToken",second)
    }).await?;
    assert_eq!(seen.len(),32);
    let saved = scenario.saved().await?;
    assert_eq!(count(&saved,"AWS::ApiOnly::SWFDomain"),4);
    for kind in ["AWS::ApiOnly::SWFActivityType","AWS::ApiOnly::SWFWorkflowType"] { assert_eq!(count(&saved,kind),16); }
    assert_eq!(scenario.edges().await?.iter().filter(|(_,_,_,_,relation)|relation=="REGISTERED_IN").count(),32);
    Ok(())
}

#[tokio::test]
async fn given_three_retention_types_and_local_ram_permissions_when_branch_pages_resume_then_no_filter_or_cursor_leaks() -> TestResult {
    let catalog = fixture::catalog()?;
    let scenario = Scenario::new(&catalog,"retention-and-local-permissions","us-east-1").await?;
    for (service,operation) in [("rbin","ListRules"),("ram","ListPermissions")] { scenario.seed(fixture::plan(&catalog,service,operation)?).await?; }
    let mut seen = BTreeSet::new();
    scenario.run(&catalog,8,|plan,lease,wire| {
        let request = body(wire)?;
        let second = lease.token().is_some();
        if plan.service_name()=="ram" {
            assert_eq!(text(&request,"resourceOwner")?,"SELF");
            assert_eq!(text(&request,"permissionType")?,"LOCAL");
            assert!(request.get("permissionStatus").is_none());
            assert_eq!(request.get("nextToken").and_then(Json::as_str),second.then_some("page-two"));
            let permissions = if second { vec![json!({"arn":"arn:aws:ram:us-east-1:123456789012:permission/active","status":"ATTACHABLE"}),json!({"arn":"arn:aws:ram:us-east-1:123456789012:permission/deprecated","status":"UNATTACHABLE"})] } else {vec![]};
            return paginated(json!({"permissions":permissions}),"nextToken",second);
        }
        let kind = text(&request,"ResourceType")?;
        assert!(matches!(kind,"EBS_VOLUME"|"EBS_SNAPSHOT"|"EC2_IMAGE"));
        assert!(request.get("ResourceTags").is_none() && request.get("LockState").is_none() && request.get("ExcludeResourceTags").is_none());
        assert_eq!(request.get("NextToken").and_then(Json::as_str),second.then_some("page-two"));
        assert!(seen.insert((kind.to_owned(),second)));
        let mut rules=vec![json!({"Identifier":format!("{kind}-a"),"RuleArn":format!("arn:aws:rbin:us-east-1:123456789012:rule/{kind}-a")})];
        if second {rules.push(json!({"Identifier":format!("{kind}-b"),"RuleArn":format!("arn:aws:rbin:us-east-1:123456789012:rule/{kind}-b")}));}
        paginated(json!({"Rules":rules}),"NextToken",second)
    }).await?;
    assert_eq!(seen.len(),6);
    let saved=scenario.saved().await?;
    assert_eq!(count(&saved,"AWS::Rbin::Rule"),6);
    assert_eq!(count(&saved,"AWS::RAM::Permission"),2);
    Ok(())
}

#[tokio::test]
async fn given_another_home_region_when_global_resolver_native_pages_execute_then_all_six_kinds_use_the_ohio_control_plane() -> TestResult {
    let catalog=fixture::catalog()?;
    let scenario=Scenario::new(&catalog,"global-resolver-fixed-control-plane","eu-west-1").await?;
    for operation in ["ListGlobalResolvers","ListAccessSources","ListFirewallDomainLists","ListHostedZoneAssociations"] { scenario.seed(fixture::plan(&catalog,"route53globalresolver",operation)?).await?; }
    scenario.run(&catalog,20,|plan,lease,wire| {
        let actual_region=routing::request_region(plan,lease.scope().region(),&scenario.home,lease.scope().branch())?;
        let endpoint=routing::resolve(plan,&actual_region)?;
        assert_eq!(endpoint.signing_region,"us-east-2");
        assert!(wire.uri().host().is_some_and(|host|host.contains(".us-east-2.")));
        assert_eq!(lease.scope().region(),&Region::Global);
        let second=lease.token().is_some();
        if second {assert!(wire.uri().query().is_some_and(|query|query.contains("next_token=page-two")));}
        resolver_response(plan.operation_name(),wire,second)
    }).await?;
    let saved=scenario.saved().await?;
    for (kind,expected) in [("GlobalResolver",2),("AccessSource",2),("FirewallDomainList",2),("HostedZoneAssociation",2),("DnsView",4),("FirewallRule",8)] {
        assert_eq!(count(&saved,&format!("AWS::Route53GlobalResolver::{kind}")),expected);
    }
    assert!(saved.iter().all(|(_,_,region)|region==Region::Global.as_str()));
    let edges=scenario.edges().await?;
    assert_eq!(edges.iter().filter(|(_,_,_,_,relation)|relation=="FOR_GLOBAL_RESOLVER").count(),6);
    assert_eq!(edges.iter().filter(|(_,_,_,_,relation)|relation=="FOR_DNS_VIEW").count(),10);
    assert_eq!(edges.iter().filter(|(_,_,_,_,relation)|relation=="USES_DOMAIN_LIST").count(),8);
    Ok(())
}

fn resolver_response(operation:&str,wire:&Wire,second:bool)->TestResult<Vec<u8>> {
    let page=if second {"b"} else {"a"};
    let value=match operation {
        "ListGlobalResolvers" => json!({"globalResolvers":[{"id":format!("resolver-{page}"),"arn":resolver_arn("global-resolver",&format!("resolver-{page}")),"clientToken":format!("resolver-{page}"),"dnsName":"example.test","name":page,"regions":["us-east-1","eu-west-1"],"createdAt":"2026-01-01T00:00:00Z","updatedAt":"2026-01-01T00:00:00Z","status":"OPERATIONAL","ipv4Addresses":["192.0.2.1"]}]}),
        "ListAccessSources" => json!({"accessSources":[{"arn":resolver_arn("access-source",&format!("source-{page}")),"cidr":"192.0.2.0/24","createdAt":"2026-01-01T00:00:00Z","updatedAt":"2026-01-01T00:00:00Z","id":format!("source-{page}"),"ipAddressType":"IPV4","dnsViewId":format!("resolver-a-view-{page}"),"protocol":"DO53","status":"OPERATIONAL"}]}),
        "ListFirewallDomainLists" => json!({"firewallDomainLists":[{"arn":resolver_arn("firewall-domain-list",&format!("list-{page}")),"globalResolverId":"resolver-a","createdAt":"2026-01-01T00:00:00Z","updatedAt":"2026-01-01T00:00:00Z","id":format!("list-{page}"),"name":page,"status":"OPERATIONAL"}]}),
        "ListHostedZoneAssociations" => json!({"hostedZoneAssociations":[{"id":format!("association-{page}"),"resourceArn":resolver_arn("dns-view","shared"),"hostedZoneId":format!("Z{page}"),"hostedZoneName":"example.test","name":page,"createdAt":"2026-01-01T00:00:00Z","updatedAt":"2026-01-01T00:00:00Z","status":"OPERATIONAL"}]}),
        "ListDNSViews" => {
            let resolver=if wire.uri().path().contains("resolver-a") {"resolver-a"} else {"resolver-b"};
            assert!(wire.uri().path().contains(resolver));
            let id=format!("{resolver}-view-{page}");
            json!({"dnsViews":[{"id":id,"arn":resolver_arn("dns-view",&id),"clientToken":id,"dnssecValidation":"ENABLED","ednsClientSubnet":"DISABLED","firewallRulesFailOpen":"DISABLED","name":id,"globalResolverId":resolver,"createdAt":"2026-01-01T00:00:00Z","updatedAt":"2026-01-01T00:00:00Z","status":"OPERATIONAL"}]})
        }
        "ListFirewallRules" => {
            let query=wire.uri().query().ok_or("DNS view selector absent")?;
            let view=query.split('&').find_map(|pair|pair.strip_prefix("dnsview_id=")).ok_or("DNS view request correlation absent")?;
            json!({"firewallRules":[{"action":"ALLOW","createdAt":"2026-01-01T00:00:00Z","updatedAt":"2026-01-01T00:00:00Z","id":format!("{view}-rule-{page}"),"name":page,"priority":100,"dnsViewId":view,"firewallDomainListId":format!("list-{page}"),"status":"OPERATIONAL"}]})
        }
        _=>return Err("unreviewed global resolver call".into()),
    };
    paginated(value,"nextToken",second)
}
fn resolver_arn(resource:&str,id:&str)->String {format!("arn:aws:route53globalresolver:us-east-2:123456789012:{resource}/{id}")}
