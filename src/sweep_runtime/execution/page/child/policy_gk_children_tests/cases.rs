mod arn_scope;
mod guard_identity;
mod greengrass;
mod kafka;
mod kendra_keyspaces;

use super::*;

pub(super) fn response(case: &Case, parent: &Parent, page: u32) -> TestResult<Json> {
    let local = match page { 0 | 1 => "shared", 2 => "later", _ => return Err("unexpected test page".into()) };
    let rows = if page == 0 { Vec::new() } else { vec![item(case, parent, local)?] };
    let token = if case.service == "kafka" { "nextToken" } else { case.token };
    Ok(match page {
        0 => json!({(case.items): rows, (token): "empty-continuation"}),
        1 => json!({(case.items): rows, (token): "data-continuation"}),
        2 => json!({(case.items): rows}),
        _ => return Err("unexpected test page".into()),
    })
}

fn item(case: &Case, parent: &Parent, local: &str) -> TestResult<Json> {
    let id = expected_id(case, parent, local);
    Ok(match (case.service, case.operation) {
        ("guardduty", "ListFilters") | ("keyspaces", "ListTypes") => json!(local),
        ("guardduty", "ListPublishingDestinations") => json!({"DestinationId":local,"DestinationType":"S3","Status":"PUBLISHING"}),
        ("identitystore", "ListGroups") => json!({"IdentityStoreId":parent.binding,"GroupId":group_id(parent, local),
            "GroupArn":id,"Revision":"17","DisplayName":local,"Description":"engineering group", "CreatedBy":"system"}),
        ("greengrass", _) => json!({"Arn":id,"Id":parent.binding,"Version":local,"CreationTimestamp":"1700000000000"}),
        ("kafka", "ListTopics") => json!({"topicArn":id,"topicName":local,"partitionCount":4,"replicationFactor":3,"outOfSyncReplicaCount":0}),
        ("kafka", "ListChannels") => json!({"channelArn":id,"channelName":local,"status":"ACTIVE",
            "creationTime":"2026-01-01T00:00:00Z","destinationType":"S3"}),
        ("kafka", "ListConfigurationRevisions") => json!({"revision":revision(local),"creationTime":"2026-01-01T00:00:00Z","description":"immutable metadata"}),
        ("kendra", "ListExperiences") => json!({"Id":local,"Name":"hosted search","Status":"ACTIVE",
            "Endpoints":[{"EndpointType":"HOME","Endpoint":"https://search.example.test/home"}]}),
        ("kendra", "ListFeaturedResultsSets") => json!({"FeaturedResultsSetId":local,"FeaturedResultsSetName":"metadata only","Status":"ACTIVE","CreationTimestamp":1700000000}),
        ("kendra", "ListDataSources") => json!({"Id":local,"Name":"safe connector summary","Type":"S3","Status":"ACTIVE"}),
        ("kendra", "ListFaqs") => json!({"Id":local,"Name":"safe faq summary","Status":"ACTIVE","FileFormat":"CSV"}),
        ("kendra", "ListQuerySuggestionsBlockLists") => json!({"Id":local,"Name":"safe blocklist summary","Status":"ACTIVE","ItemCount":5}),
        ("kendra", "ListThesauri") => json!({"Id":local,"Name":"safe thesaurus summary","Status":"ACTIVE"}),
        ("kendra", "ListAccessControlConfigurations") => json!({"Id":local}),
        _ => return Err("missing exact child response scenario".into()),
    })
}

pub(super) fn expected_id(case: &Case, parent: &Parent, local: &str) -> String {
    match (case.service, case.operation) {
        ("identitystore", _) => format!("arn:aws:identitystore:::group/{}", group_id(parent, local)),
        ("greengrass", _) => format!("{}/versions/{local}", parent.id),
        ("kafka", "ListTopics") => format!("{}/{local}", parent.id.replace(":cluster/", ":topic/")),
        ("kafka", "ListChannels") => format!("{}/{local}", parent.id.replace(":cluster/", ":channel/")),
        ("kafka", "ListConfigurationRevisions") => qualified(&parent.binding, &revision(local).to_string()),
        _ => qualified(&parent.binding, local),
    }
}

fn qualified(parent: &str, local: &str) -> String {
    format!("{}:{parent}|{}:{local}", parent.len(), local.len())
}

fn revision(local: &str) -> i64 { if local == "shared" { 1 } else { 2 } }

fn group_id(parent: &Parent, local: &str) -> String {
    let prefix = parent.binding.trim_start_matches("d-");
    format!("{prefix}-00000000-0000-0000-0000-00000000000{}", revision(local))
}

fn assert_text(record: &Record, field: &str, expected: &str) -> TestResult {
    let actual = record.root().get_name(field)?.ok_or_else(|| format!("stored metadata omitted {field}"))?.to_value()?;
    assert_eq!(actual, Value::Text(expected.to_owned()), "{field}");
    Ok(())
}

pub(super) fn assert_metadata(case: &Case, id: &str, record: &Record) -> TestResult {
    let local = if id.ends_with("later") || id.ends_with("1:2") || id.ends_with("000000000002") { "later" } else { "shared" };
    match (case.service, case.operation) {
        ("guardduty", "ListFilters") | ("keyspaces", "ListTypes") => {
            assert_eq!(record::path_value(record.root(), &[])?.ok_or("scalar value missing")?.to_value()?, Value::Text(local.to_owned()));
        }
        ("guardduty", "ListPublishingDestinations") => {
            assert_text(record, "DestinationId", local)?;
            assert_text(record, "Status", "PUBLISHING")?;
        }
        ("identitystore", _) => {
            assert_text(record, "GroupArn", id)?;
            assert_text(record, "DisplayName", local)?;
            assert_text(record, "Description", "engineering group")?;
            assert_text(record, "Revision", "17")?;
        }
        ("greengrass", _) => {
            assert_text(record, "Arn", id)?;
            assert_text(record, "Version", local)?;
            assert!(record.root().get_name("Id")?.is_some(), "parent Id metadata was dropped");
            assert!(record.root().get_name("Definition")?.is_none(), "forbidden definition body was persisted");
        }
        ("kafka", "ListTopics") => { assert_text(record, "TopicArn", id)?; assert_text(record, "TopicName", local)?; }
        ("kafka", "ListChannels") => { assert_text(record, "ChannelArn", id)?; assert_text(record, "ChannelName", local)?; }
        ("kafka", "ListConfigurationRevisions") => {
            assert_eq!(record.root().get_name("Revision")?.ok_or("stored revision absent")?.to_value()?, Value::Int(revision(local)));
            assert_text(record, "Description", "immutable metadata")?;
            assert!(record.root().get_name("ServerProperties")?.is_none(), "forbidden configuration body was persisted");
        }
        ("kendra", "ListFeaturedResultsSets") => { assert_text(record, "FeaturedResultsSetId", local)?; }
        ("kendra", "ListExperiences") => {
            assert_text(record, "Id", local)?;
            assert!(matches!(record.root().get_name("Endpoints")?, Some(ValueRef::List(_))), "hosted HOME endpoint metadata was lost");
        }
        ("kendra", "ListAccessControlConfigurations") => { assert_text(record, "Id", local)?; }
        ("kendra", _) => { assert_text(record, "Id", local)?; assert_text(record, "Status", "ACTIVE")?; }
        _ => return Err("missing metadata assertion scenario".into()),
    }
    Ok(())
}
