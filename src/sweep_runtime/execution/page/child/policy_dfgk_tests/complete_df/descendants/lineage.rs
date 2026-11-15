use super::*;

fn domain(name:&str)->String { format!("dzd_{}",name.repeat(12)) }
fn domain_arn(id:&str)->String {format!("arn:aws:datazone:us-east-1:123456789012:domain/{id}")}

#[tokio::test]
async fn given_two_domains_when_project_profile_pages_share_ids_then_request_domain_scopes_nodes_and_edges() -> TestResult {
    let mut nodes=Nodes::new(); let mut edges=Edges::new();
    for name in ["a","b"] {
        let domain=domain(name); let arn=domain_arn(&domain);
        node(&mut nodes,"AWS::DataZone::Domain",&arn);
        for page in ["first","second"] {
            let id=key(&[&domain,page]);
            node(&mut nodes,"AWS::DataZone::ProjectProfile",&id);
            edge(&mut edges,"AWS::DataZone::ProjectProfile",&id,"AWS::DataZone::Domain",&arn,"IN_DOMAIN");
        }
    }
    run("datazone",&["ListDomains"],5,datazone_response,nodes,edges).await
}
fn datazone_response(operation:&str,request:&Record,second:bool)->TestResult<(Json,Option<String>)> {
    match operation {
        "ListDomains"=>{
            assert!(!second);
            let items:Vec<_>=["a","b"].into_iter().map(|name|{let id=domain(name);json!({"id":id,"arn":domain_arn(&id),"name":name,"managedAccountId":"123456789012","status":"AVAILABLE","createdAt":1})}).collect();
            Ok((json!({"items":items}),Some("/v2/domains".to_owned())))
        }
        "ListProjectProfiles"=>{
            let domain=string(request,"domainIdentifier")?;
            assert!([self::domain("a"),self::domain("b")].iter().any(|expected|expected==domain));
            let body=json!({"items":[{"domainId":domain,"id":leaf(second),"name":"profile","createdBy":"owner"}]});
            Ok((paginated(body,second,"nextToken")?,Some(format!("/v2/domains/{domain}/project-profiles"))))
        }
        _=>Err(format!("unexpected DataZone operation {operation}").into()),
    }
}

const WORKFLOWS:&[(&str,&str,&str,&str)]=&[
    ("ListIdMappingWorkflows","idmappingworkflows","AWS::EntityResolution::IdMappingWorkflow","AWS::ApiOnly::EntityResolutionIdMappingJob"),
    ("ListMatchingWorkflows","matchingworkflows","AWS::EntityResolution::MatchingWorkflow","AWS::ApiOnly::EntityResolutionMatchingJob"),
];
fn workflow_arn(resource:&str,name:&str)->String {format!("arn:aws:entityresolution:us-east-1:123456789012:{resource}/{name}")}

#[tokio::test]
async fn given_matching_and_mapping_workflows_when_job_summary_pages_share_job_ids_then_both_families_retain_workflow_authority() -> TestResult {
    let mut nodes=Nodes::new();let mut edges=Edges::new();
    for (_,path,parent_kind,kind) in WORKFLOWS {
        let resource=path.strip_suffix('s').ok_or("workflow path missing plural suffix")?;
        for name in ["alpha","beta"] {
            let arn=workflow_arn(resource,name);
            node(&mut nodes,parent_kind,&arn);
            for page in ["first","second"] {
                let id=key(&[name,page]);node(&mut nodes,kind,&id);
                edge(&mut edges,kind,&id,parent_kind,&arn,"IN_WORKFLOW");
            }
        }
    }
    run("entityresolution",&["ListIdMappingWorkflows","ListMatchingWorkflows"],10,entity_response,nodes,edges).await
}
fn entity_response(operation:&str,request:&Record,second:bool)->TestResult<(Json,Option<String>)> {
    for (root,path,_,_) in WORKFLOWS {
        if operation==*root {
            assert!(!second);
            let resource=path.strip_suffix('s').ok_or("workflow resource absent")?;
            let summaries=["alpha","beta"].into_iter().map(|name|->TestResult<Json>{
                let mut row=json!({"workflowName":name,"workflowArn":workflow_arn(resource,name),"createdAt":1,"updatedAt":1});
                if operation=="ListMatchingWorkflows" {
                    row.as_object_mut().ok_or("workflow summary must be an object")?.insert("resolutionType".to_owned(),json!("RULE_MATCHING"));
                }
                Ok(row)
            }).collect::<TestResult<Vec<_>>>()?;
            return Ok((json!({"workflowSummaries":summaries}),Some(format!("/{path}"))));
        }
    }
    let path=match operation {"ListIdMappingJobs"=>"idmappingworkflows","ListMatchingJobs"=>"matchingworkflows",_=>return Err(format!("unexpected EntityResolution operation {operation}").into())};
    let name=string(request,"workflowName")?;assert!(["alpha","beta"].contains(&name));
    let body=json!({"jobs":[{"jobId":leaf(second),"status":"SUCCEEDED","startTime":1}]});
    Ok((paginated(body,second,"nextToken")?,Some(format!("/{path}/{name}/jobs"))))
}

const EXPERIMENTS:&[(&str,&str,&str,&str)]=&[
    ("ListExperimentTemplates","experiment-template","AWS::FIS::ExperimentTemplate","AWS::FIS::TargetAccountConfiguration"),
    ("ListExperiments","experiment","AWS::ApiOnly::FISExperiment","AWS::ApiOnly::FISExperimentTargetAccountConfiguration"),
];
fn fis_id(template:bool,name:&str)->String {format!("{}{}",if template {"EXT"} else {"EXP"},name.repeat(13))}
fn fis_arn(resource:&str,id:&str)->String {format!("arn:aws:fis:us-east-1:123456789012:{resource}/{id}")}

#[tokio::test]
async fn given_template_and_running_experiment_families_when_target_account_pages_resume_then_owned_configuration_not_foreign_accounts_is_stored() -> TestResult {
    let mut nodes=Nodes::new();let mut edges=Edges::new();
    for (operation,resource,parent_kind,kind) in EXPERIMENTS {
        let template=*operation=="ListExperimentTemplates";
        for name in ["a","b"] {
            let parent=fis_id(template,name);let arn=fis_arn(resource,&parent);node(&mut nodes,parent_kind,&arn);
            for account in ["222222222222","333333333333"] {
                let id=key(&[&parent,account]);node(&mut nodes,kind,&id);
                edge(&mut edges,kind,&id,parent_kind,&arn,if template {"IN_TEMPLATE"} else {"IN_EXPERIMENT"});
            }
        }
    }
    run("fis",&["ListExperimentTemplates","ListExperiments"],10,fis_response,nodes,edges).await
}
fn fis_response(operation:&str,request:&Record,second:bool)->TestResult<(Json,Option<String>)> {
    for (root,resource,_,_) in EXPERIMENTS {
        if operation==*root {
            assert!(!second);let template=*root=="ListExperimentTemplates";
            let rows:Vec<_>=["a","b"].into_iter().map(|name|{let id=fis_id(template,name);json!({"id":id,"arn":fis_arn(resource,&id),"tags":{"owner":"inventory"}})}).collect();
            return if template {Ok((json!({"experimentTemplates":rows}),Some("/experimentTemplates".to_owned())))} else {Ok((json!({"experiments":rows}),Some("/experiments".to_owned())))};
        }
    }
    let (field,path,template)=match operation {
        "ListTargetAccountConfigurations"=>("experimentTemplateId","experimentTemplates",true),
        "ListExperimentTargetAccountConfigurations"=>("experimentId","experiments",false),
        _=>return Err(format!("unexpected FIS operation {operation}").into()),
    };
    let parent=string(request,field)?;assert!([fis_id(template,"a"),fis_id(template,"b")].iter().any(|id|id==parent));
    let account=if second {"333333333333"} else {"222222222222"};
    let body=json!({"targetAccountConfigurations":[{"accountId":account,"roleArn":format!("arn:aws:iam::{account}:role/fis-target"),"description":"owned target-account configuration"}]});
    Ok((paginated(body,second,"nextToken")?,Some(format!("/{path}/{parent}/targetAccountConfigurations"))))
}
