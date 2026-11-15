use super::*;

const CHANNEL:&str="arn:aws:sns:us-east-1:123456789012:inventory-notifications";
fn member(second:bool)->String {format!("arn:aws:ec2:us-east-1:123456789012:vpc/vpc-{}",if second {"22222222222222222"} else {"11111111111111111"})}

#[tokio::test]
async fn given_channel_and_two_resource_sets_when_membership_pages_share_uris_then_configuration_and_correlated_memberships_survive_reopen() -> TestResult {
    let mut nodes=Nodes::new();let mut edges=Edges::new();
    node(&mut nodes,"AWS::FMS::NotificationChannel",CHANNEL);
    for parent in ["setAlpha","setBeta"] {
        node(&mut nodes,"AWS::FMS::ResourceSet",parent);
        for second in [false,true] {
            let id=key(&[parent,&member(second)]);
            node(&mut nodes,"AWS::ApiOnly::FMSResourceSetMembership",&id);
            edge(&mut edges,"AWS::ApiOnly::FMSResourceSetMembership",&id,"AWS::FMS::ResourceSet",parent,"IN_RESOURCE_SET");
        }
    }
    run("fms",&["GetNotificationChannel","ListResourceSets"],6,fms_response,nodes,edges).await
}
fn fms_response(operation:&str,request:&Record,second:bool)->TestResult<(Json,Option<String>)> {
    let body=match operation {
        "GetNotificationChannel"=>{assert!(!second);return Ok((json!({"SnsTopicArn":CHANNEL,"SnsRoleName":"arn:aws:iam::123456789012:role/fms-notification"}),None));}
        "ListResourceSets"=>{assert!(!second);return Ok((json!({"ResourceSets":[{"Id":"setAlpha","Name":"alpha","ResourceSetStatus":"ACTIVE"},{"Id":"setBeta","Name":"beta","ResourceSetStatus":"ACTIVE"}]}),None));}
        "ListResourceSetResources"=>{
            let parent=string(request,"Identifier")?;assert!(["setAlpha","setBeta"].contains(&parent));
            json!({"Items":[{"URI":member(second),"AccountId":"123456789012"}]})
        }
        _=>return Err(format!("unexpected FMS operation {operation}").into()),
    };
    Ok((paginated(body,second,"NextToken")?,None))
}

fn detector_arn(name:&str)->String {format!("arn:aws:frauddetector:us-east-1:123456789012:detector/{name}")}
#[tokio::test]
async fn given_two_detectors_when_describe_version_summary_pages_resume_then_versions_are_not_collapsed_across_detectors() -> TestResult {
    let mut nodes=Nodes::new();let mut edges=Edges::new();
    for parent in ["alpha","beta"] {
        let arn=detector_arn(parent);node(&mut nodes,"AWS::FraudDetector::Detector",&arn);
        for version in ["1","2"] {
            let id=key(&[parent,version]);node(&mut nodes,"AWS::ApiOnly::FraudDetectorDetectorVersion",&id);
            edge(&mut edges,"AWS::ApiOnly::FraudDetectorDetectorVersion",&id,"AWS::FraudDetector::Detector",&arn,"IN_DETECTOR");
        }
    }
    run("frauddetector",&["GetDetectors"],5,fraud_response,nodes,edges).await
}
fn fraud_response(operation:&str,request:&Record,second:bool)->TestResult<(Json,Option<String>)> {
    match operation {
        "GetDetectors"=>{
            assert!(!second);
            let detectors:Vec<_>=["alpha","beta"].into_iter().map(|name|json!({"detectorId":name,"arn":detector_arn(name),"eventTypeName":"transaction","description":"detector metadata"})).collect();
            Ok((json!({"detectors":detectors}),None))
        }
        "DescribeDetector"=>{
            let detector=string(request,"detectorId")?;assert!(["alpha","beta"].contains(&detector));
            let body=json!({"detectorId":detector,"detectorVersionSummaries":[{"detectorVersionId":if second {"2"} else {"1"},"status":"ACTIVE","lastUpdatedTime":"2026-01-01T00:00:00Z"}]});
            Ok((paginated(body,second,"nextToken")?,None))
        }
        _=>Err(format!("unexpected FraudDetector operation {operation}").into()),
    }
}
