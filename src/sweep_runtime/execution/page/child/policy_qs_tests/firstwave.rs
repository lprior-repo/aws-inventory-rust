use super::*;

#[tokio::test]
async fn given_directory_buckets_when_express_control_xml_pages_resume_then_endpoint_signing_and_canonical_inventory_are_regional() -> TestResult {
    let catalog=fixture::catalog()?;
    let scenario=Scenario::new(&catalog,"directory-bucket-express-control","us-east-1").await?;
    scenario.seed(fixture::plan(&catalog,"s3","ListDirectoryBuckets")?).await?;
    scenario.run(&catalog,2,|plan,lease,wire| {
        let endpoint=routing::resolve(plan,&RegionId::parse("us-east-1")?)?;
        assert_eq!(endpoint.signing_region,"us-east-1");
        assert_eq!(endpoint.signing_name,"s3express");
        assert_eq!(wire.uri().host(),Some("s3express-control.us-east-1.amazonaws.com"));
        assert_eq!(wire.uri().path(),"/");
        let second=lease.token().is_some();
        assert_eq!(wire.uri().query().is_some_and(|query|query.contains("continuation-token=page-two")),second);
        let page=if second {"b"} else {"a"};
        let arn=if second {String::new()} else {"<BucketArn>arn:aws:s3express:us-east-1:123456789012:bucket/owned-a--use1-az4--x-s3</BucketArn>".to_owned()};
        let token=if second {""} else {"<ContinuationToken>page-two</ContinuationToken>"};
        Ok(format!("<ListAllMyDirectoryBucketsResult><Buckets><Bucket><Name>owned-{page}--use1-az4--x-s3</Name>{arn}</Bucket></Buckets>{token}</ListAllMyDirectoryBucketsResult>").into_bytes())
    }).await?;
    let saved=scenario.saved().await?;
    assert_eq!(saved,vec![("AWS::S3Express::DirectoryBucket".to_owned(),"arn:aws:s3express:us-east-1:123456789012:bucket/owned-a--use1-az4--x-s3".to_owned(),"us-east-1".to_owned()),("AWS::S3Express::DirectoryBucket".to_owned(),"owned-b--use1-az4--x-s3".to_owned(),"us-east-1".to_owned())]);
    Ok(())
}

#[tokio::test]
async fn given_all_owned_snapshot_classes_when_query_markers_resume_then_backup_snapshots_are_not_omitted_or_mixed_with_foreign_classes() -> TestResult {
    let catalog=fixture::catalog()?;
    let scenario=Scenario::new(&catalog,"owned-rds-snapshot-union","us-east-1").await?;
    scenario.seed(fixture::plan(&catalog,"rds","DescribeDBSnapshots")?).await?;
    let mut seen=BTreeSet::new();
    scenario.run(&catalog,6,|_,lease,wire| {
        let request=std::str::from_utf8(wire.body())?;
        let class=request.split('&').find_map(|pair|pair.strip_prefix("SnapshotType=")).ok_or("snapshot branch absent")?;
        assert!(matches!(class,"manual"|"automated"|"awsbackup"));
        assert!(!request.contains("IncludeShared")&&!request.contains("IncludePublic"));
        let second=lease.token().is_some();
        assert_eq!(request.split('&').any(|pair|pair=="Marker=page-two"),second);
        assert!(seen.insert((class.to_owned(),second)));
        let page=if second {"b"} else {"a"};
        let marker=if second {""} else {"<Marker>page-two</Marker>"};
        Ok(format!("<DescribeDBSnapshotsResponse><DescribeDBSnapshotsResult><DBSnapshots><DBSnapshot><DBSnapshotIdentifier>{class}-{page}</DBSnapshotIdentifier><DBSnapshotArn>arn:aws:rds:us-east-1:123456789012:snapshot:{class}-{page}</DBSnapshotArn><SnapshotType>{class}</SnapshotType><Status>available</Status></DBSnapshot></DBSnapshots>{marker}</DescribeDBSnapshotsResult></DescribeDBSnapshotsResponse>").into_bytes())
    }).await?;
    assert_eq!(seen.len(),6);
    let saved=scenario.saved().await?;
    assert_eq!(count(&saved,"AWS::RDS::DBSnapshot"),6);
    assert_eq!(saved.iter().filter(|(_,id,_)|id.contains(":snapshot:awsbackup-")).count(),2);
    Ok(())
}

#[tokio::test]
async fn given_public_and_private_zones_when_one_dnssec_response_per_public_zone_arrives_then_status_and_keys_share_the_bound_parent() -> TestResult {
    let catalog=fixture::catalog()?;
    let scenario=Scenario::new(&catalog,"zone-qualified-dnssec-keys","us-east-1").await?;
    scenario.seed(fixture::plan(&catalog,"route-53","ListHostedZones")?).await?;
    scenario.seed(fixture::plan(&catalog,"kms","ListKeys")?).await?;
    let mut zones=BTreeSet::new();
    scenario.run(&catalog,4,|plan,_,wire| {
        match plan.operation_name() {
            "ListHostedZones"=>Ok("<ListHostedZonesResponse><HostedZones><HostedZone><Id>/hostedzone/ZA</Id><Name>a.example.test.</Name><CallerReference>a</CallerReference><Config><PrivateZone>false</PrivateZone></Config></HostedZone><HostedZone><Id>/hostedzone/ZB</Id><Name>b.example.test.</Name><CallerReference>b</CallerReference><Config><PrivateZone>false</PrivateZone></Config></HostedZone><HostedZone><Id>/hostedzone/ZPRIVATE</Id><Name>internal.example.test.</Name><CallerReference>private</CallerReference><Config><PrivateZone>true</PrivateZone></Config></HostedZone></HostedZones><IsTruncated>false</IsTruncated><MaxItems>100</MaxItems></ListHostedZonesResponse>".as_bytes().to_vec()),
            "ListKeys"=>bytes(json!({"Keys":[{"KeyId":"11111111-2222-3333-4444-555555555555","KeyArn":"arn:aws:kms:us-east-1:123456789012:key/11111111-2222-3333-4444-555555555555"}]})),
            "GetDNSSEC"=>{
                let path=wire.uri().path();
                assert!(matches!(path,"/2013-04-01/hostedzone/ZA/dnssec"|"/2013-04-01/hostedzone/ZB/dnssec"));
                assert!(zones.insert(path.to_owned()));
                // AWS API response syntax uses KeySigningKey, not an invented top-level ARN.
                Ok("<GetDNSSECResponse><Status><ServeSignature>SIGNING</ServeSignature></Status><KeySigningKeys><KeySigningKey><Name>same-key-name</Name><KmsArn>arn:aws:kms:us-east-1:123456789012:key/11111111-2222-3333-4444-555555555555</KmsArn><Status>ACTIVE</Status><PublicKey>PUBLIC-DNSSEC-MATERIAL</PublicKey><DSRecord>PUBLIC-DS</DSRecord><DNSKEYRecord>PUBLIC-DNSKEY</DNSKEYRecord></KeySigningKey></KeySigningKeys></GetDNSSECResponse>".as_bytes().to_vec())
            }
            _=>Err("unreviewed DNSSEC operation".into()),
        }
    }).await?;
    assert_eq!(zones.len(),2);
    let saved=scenario.saved().await?;
    assert_eq!(count(&saved,"AWS::Route53::HostedZone"),3);
    assert_eq!(count(&saved,"AWS::Route53::DNSSEC"),2);
    assert_eq!(count(&saved,"AWS::Route53::KeySigningKey"),2);
    let edges=scenario.edges().await?;
    assert_eq!(edges.iter().filter(|(kind,_,_,_,relation)|kind=="AWS::Route53::KeySigningKey"&&relation=="USES_KEY").count(),2);
    assert_eq!(edges.iter().filter(|(kind,_,_,_,relation)|kind=="AWS::Route53::KeySigningKey"&&relation=="FOR_HOSTED_ZONE").count(),2);
    assert_eq!(edges.iter().filter(|(kind,_,_,_,relation)|kind=="AWS::Route53::DNSSEC"&&relation=="FOR_HOSTED_ZONE").count(),2);
    for zone in ["ZA","ZB"] {
        for kind in ["AWS::Route53::DNSSEC","AWS::Route53::KeySigningKey"] {
            let source=saved.iter().find(|(candidate,id,_)|candidate==kind&&id.contains(zone)).ok_or("zone-qualified DNSSEC resource is missing")?;
            assert!(edges.iter().any(|(candidate,id,target_kind,target,relation)|candidate==kind&&id==&source.1&&target_kind=="AWS::Route53::HostedZone"&&target==&format!("/hostedzone/{zone}")&&relation=="FOR_HOSTED_ZONE"));
        }
    }
    Ok(())
}

#[tokio::test]
async fn given_assertion_and_gating_rules_in_two_panels_when_native_union_pages_resume_then_both_arms_persist_under_canonical_safety_rule() -> TestResult {
    let catalog=fixture::catalog()?;
    let scenario=Scenario::new(&catalog,"arc-safety-rule-union","us-east-1").await?;
    scenario.seed(fixture::plan(&catalog,"route53-recovery-control-config","ListControlPanels")?).await?;
    scenario.run(&catalog,5,|plan,lease,wire| {
        if plan.operation_name()=="ListControlPanels" {return bytes(json!({"ControlPanels":[{"ControlPanelArn":panel("a"),"Name":"a"},{"ControlPanelArn":panel("b"),"Name":"b"}]}));}
        let parent=lease.scope().parent().ok_or("control panel owner lost")?;
        assert!(wire.uri().path().contains("controlpanel%2F")||wire.uri().path().contains("controlpanel/"));
        let second=lease.token().is_some();
        let arm=if second {"GATING"} else {"ASSERTION"};
        let mut rule=json!({"SafetyRuleArn":format!("arn:aws:route53-recovery-control::123456789012:safety-rule/{}-{arm}",if parent.id.as_str().ends_with("/a") {"a"} else {"b"}),"ControlPanelArn":parent.id.as_str(),"Name":arm,"RuleConfig":{"Type":"ATLEAST","Threshold":1,"Inverted":false},"Status":"DEPLOYED","WaitPeriodMs":5000});
        let fields=rule.as_object_mut().ok_or("rule not object")?;
        if second {fields.insert("GatingControls".to_owned(),json!(["arn:aws:route53-recovery-control::123456789012:controlpanel/a/routingcontrol/gate"]));fields.insert("TargetControls".to_owned(),json!(["arn:aws:route53-recovery-control::123456789012:controlpanel/a/routingcontrol/target"]));}
        else {fields.insert("AssertedControls".to_owned(),json!(["arn:aws:route53-recovery-control::123456789012:controlpanel/a/routingcontrol/asserted"]));}
        paginated(json!({"SafetyRules":[{arm:rule}]}),"NextToken",second)
    }).await?;
    assert_eq!(count(&scenario.saved().await?,"AWS::Route53RecoveryControl::SafetyRule"),4);
    assert_eq!(scenario.edges().await?.iter().filter(|(kind,_,_,_,relation)|kind=="AWS::Route53RecoveryControl::SafetyRule"&&relation=="FOR_CONTROL_PANEL").count(),4);
    Ok(())
}
fn panel(id:&str)->String {format!("arn:aws:route53-recovery-control::123456789012:controlpanel/{id}")}
