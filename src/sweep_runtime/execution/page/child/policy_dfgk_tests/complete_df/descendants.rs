use super::*;

mod lineage;
mod administration;

fn environment(name: &str) -> String { format!("env-{}", name.repeat(10)) }
fn environment_arn(id: &str) -> String { format!("arn:aws:evs:us-east-1:123456789012:environment/{id}") }
fn connector_secret(second:bool)->String {format!("arn:aws:secretsmanager:us-east-1:123456789012:secret:evs-{}-AbCdEf",leaf(second))}

#[tokio::test]
async fn given_two_evs_environments_when_numeric_vlan_host_and_connector_pages_run_then_colliding_local_ids_remain_distinct() -> TestResult {
    let mut nodes = Nodes::new();
    let mut edges = Edges::new();
    for second in [false,true] {node(&mut nodes,"AWS::SecretsManager::Secret",&connector_secret(second));}
    for name in ["a", "b"] {
        let environment = environment(name);
        let arn = environment_arn(&environment);
        node(&mut nodes,"AWS::EVS::Environment",&arn);
        for second in [false,true] {
            for (kind,local) in [("AWS::ApiOnly::EVSHost",format!("host-{}",leaf(second))),("AWS::ApiOnly::EVSConnector",format!("connector-{}",leaf(second))),("AWS::ApiOnly::EVSVlan",if second { "4094" } else { "1" }.to_owned())] {
                let id=key(&[&environment,&local]);
                node(&mut nodes,kind,&id);
                edge(&mut edges,kind,&id,"AWS::EVS::Environment",&arn,"IN_ENVIRONMENT");
            }
            let connector=key(&[&environment,&format!("connector-{}",leaf(second))]);
            edge(&mut edges,"AWS::ApiOnly::EVSConnector",&connector,"AWS::SecretsManager::Secret",&connector_secret(second),"USES_SECRET");
        }
    }
    run_with_dependencies("evs",&["ListEnvironments"],&[("secrets-manager","ListSecrets")],14,evs_response,nodes,edges).await
}

fn evs_response(operation: &str,request: &Record,second: bool) -> TestResult<(Json,Option<String>)> {
    if operation=="ListSecrets" {
        assert!(!second);
        let secrets:Vec<_>=[false,true].into_iter().map(|page|json!({"ARN":connector_secret(page),"Name":format!("evs-{}",leaf(page)),"Tags":[]})).collect();
        return Ok((json!({"SecretList":secrets}),None));
    }
    if operation=="ListEnvironments" {
        assert!(!second);
        let environments:Vec<_>=["a","b"].into_iter().map(|name| {
            let id=environment(name);
            json!({"environmentId":id,"environmentArn":environment_arn(&id),"environmentName":name,"environmentState":"CREATED"})
        }).collect();
        return Ok((json!({"environmentSummaries":environments}),None));
    }
    let environment=string(request,"environmentId")?;
    assert!([self::environment("a"),self::environment("b")].iter().any(|expected|expected==environment));
    let body=match operation {
        "ListEnvironmentHosts"=>json!({"environmentHosts":[{"hostName":format!("host-{}",leaf(second)),"ec2InstanceId":"i-0123456789abcdef0","hostState":"CREATED","ipAddress":"10.0.0.10"}]}),
        "ListEnvironmentConnectors"=>json!({"connectors":[{"environmentId":environment,"connectorId":format!("connector-{}",leaf(second)),"applianceFqdn":"connector.example.invalid","secretArn":connector_secret(second),"state":"CREATED"}]}),
        "ListEnvironmentVlans"=>json!({"environmentVlans":[{"vlanId":if second {4094} else {1},"cidr":"10.0.0.0/24","vlanState":"CREATED","subnetId":"subnet-0123456789abcdef0"}]}),
        _=>return Err(format!("unexpected EVS operation {operation}").into()),
    };
    Ok((paginated(body,second,"nextToken")?,None))
}

#[tokio::test]
async fn given_a_real_bound_vlan_page_when_identity_is_missing_noninteger_or_outside_smithy_integer_then_no_vlan_is_committed() -> TestResult {
    for (invalid,expected) in [
        (json!({}),Some("modeled identity path is absent")),
        (json!({"vlanId":null}),Some("modeled identity path is absent")),
        (json!({"vlanId":"1"}),None),
        (json!({"vlanId":1.5}),None),
        (json!({"vlanId":2147483648_i64}),Some("modeled identity integer is out of range")),
        (json!({"vlanId":-2147483649_i64}),Some("modeled identity integer is out of range")),
    ] {
        let catalog=catalog::catalog()?;
        let scenario=Scenario::new(&catalog,"invalid-native-evs-vlan").await?;
        scenario.seed(fixture::plan(&catalog,"evs","ListEnvironments")?).await?;
        let mut rejected=false;
        for _ in 0..13 {
            let lease=scenario.store.claim(&scenario.sweep,1001,10_000).await?.ok_or("bound VLAN work lost")?;
            let operation=lease.scope().operation().as_str().to_owned();
            let plan=fixture::plan(&catalog,"evs",&operation)?;
            let (request,_)=scenario.prepared_request(plan,&lease)?;
            if operation=="ListEnvironmentVlans" {
                assert!(lease.scope().parent().is_some());
                let error=match scenario.apply(&catalog,plan,lease,&request,json!({"environmentVlans":[invalid]})).await {
                    Ok(())=>return Err("invalid native VLAN identity committed successfully".into()),
                    Err(error)=>error,
                };
                if let Some(expected)=expected {
                    let Some(SweepError::Operation(actual))=error.downcast_ref::<SweepError>() else {return Err(format!("wrong native identity error: {error}").into())};
                    assert_eq!(*actual,expected);
                } else {
                    assert!(matches!(error.downcast_ref::<aws_wire::response::DecodeError>(),Some(aws_wire::response::DecodeError::Invalid("JSON value does not match modeled shape"))),"wrong native wire error: {error}");
                }
                rejected=true;
                break;
            }
            let (body,_)=evs_response(&operation,&request,lease.token().is_some())?;
            scenario.apply(&catalog,plan,lease,&request,body).await?;
        }
        assert!(rejected,"native VLAN request was never reached");
        let reader=scenario.reader().await?;
        let count:i64=sqlx::query_scalar("SELECT COUNT(*) FROM resources WHERE kind='AWS::ApiOnly::EVSVlan'").fetch_one(&reader).await?;
        assert_eq!(count,0);
        reader.close().await;
    }
    Ok(())
}
