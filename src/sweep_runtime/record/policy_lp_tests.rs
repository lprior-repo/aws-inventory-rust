pub(crate) mod fixture;

use super::tag_observation;
use fixture::{TestResult, compiled, key, rows};
use inventory_core::{
    catalog::runtime::{PlanPurpose, RuntimeCatalog, TagEndpointSource},
    inventory::{Kind, Tag, TagCoverage},
};
use serde_json::json;
use std::collections::BTreeMap;

#[test]
fn given_pinpoint_configuration_name_when_list_and_detail_decode_then_key_and_inline_tags_are_preserved()
-> TestResult {
    // Given: names whose punctuation would expose accidental composite encoding or normalization.
    let bytes = compiled(
        "pinpoint-email",
        "2018-07-26",
        &["ListConfigurationSets", "GetConfigurationSet"],
        "AWS::PinpointEmail::ConfigurationSet",
    )?;
    let runtime = RuntimeCatalog::from_bytes(&bytes)?;
    let list = runtime
        .lookup("pinpoint-email", "ListConfigurationSets")
        .ok_or("list absent")?;
    let detail = runtime
        .lookup("pinpoint-email", "GetConfigurationSet")
        .ok_or("detail absent")?;
    let listed = rows(
        list,
        json!({"ConfigurationSets":["release:blue", "release|green"]}),
    )?;
    assert_eq!(listed.len(), 2);
    for (listed, name) in listed.iter().zip(["release:blue", "release|green"]) {
        // When: the real scalar-list and singleton response are decoded and selected.
        let described = rows(
            detail,
            json!({"ConfigurationSetName":name,"Tags":[{"Key":"environment","Value":"nightly"},{"Key":"empty","Value":""}]}),
        )?;
        assert_eq!(described.len(), 1);
        let row = described.first().ok_or("detail row absent")?;
        let list_key = key(list, listed, "us-east-1")?;
        let detail_key = key(detail, row, "us-east-1")?;
        // Then: one real resource key owns the complete detail tag snapshot.
        assert_eq!(list_key, detail_key);
        assert_eq!(detail_key.id.as_str(), name);
        let tags = tag_observation(
            row,
            &detail_key,
            detail.tags().ok_or("detail tag plan absent")?,
        )?;
        assert_eq!(tags.resource(), &list_key);
        assert_eq!(tags.coverage(), TagCoverage::Complete);
        assert_eq!(
            tags.tags(),
            [Tag::new("empty", "")?, Tag::new("environment", "nightly")?]
        );
    }
    Ok(())
}

#[test]
fn given_license_summary_when_detail_decodes_then_exact_arn_owns_populated_empty_and_missing_tags()
-> TestResult {
    // Given: the independent summary already contains the required detail ARN.
    let arn = "arn:aws:license-manager:us-east-1:123456789012:license-configuration:lic-123";
    let bytes = compiled(
        "license-manager",
        "2018-08-01",
        &["ListLicenseConfigurations", "GetLicenseConfiguration"],
        "AWS::LicenseManager::LicenseConfiguration",
    )?;
    let runtime = RuntimeCatalog::from_bytes(&bytes)?;
    let list = runtime
        .lookup("license-manager", "ListLicenseConfigurations")
        .ok_or("list absent")?;
    let detail = runtime
        .lookup("license-manager", "GetLicenseConfiguration")
        .ok_or("detail absent")?;
    let listed = rows(
        list,
        json!({"LicenseConfigurations":[{"LicenseConfigurationArn":arn}]}),
    )?;
    let list_key = key(list, listed.first().ok_or("summary absent")?, "us-east-1")?;
    let tagged = json!({"LicenseConfigurationArn":arn,"Tags":[{"Key":"owner","Value":"platform"}]});
    let empty = json!({"LicenseConfigurationArn":arn,"Tags":[]});
    let missing = json!({"LicenseConfigurationArn":arn});
    // When: populated, explicitly empty, and unobserved snapshots pass through production extraction.
    for (response, coverage, expected) in [
        (
            tagged,
            TagCoverage::Complete,
            vec![Tag::new("owner", "platform")?],
        ),
        (empty, TagCoverage::Complete, vec![]),
        (missing, TagCoverage::Unobserved, vec![]),
    ] {
        let described = rows(detail, response)?;
        assert_eq!(described.len(), 1);
        let row = described.first().ok_or("detail row absent")?;
        let detail_key = key(detail, row, "us-east-1")?;
        // Then: empty means removals, missing means no claim, and neither creates a second resource.
        assert_eq!(detail_key, list_key);
        assert_eq!(detail_key.id.as_str(), arn);
        let tags = tag_observation(
            row,
            &detail_key,
            detail.tags().ok_or("detail tag plan absent")?,
        )?;
        assert_eq!(tags.resource(), &list_key);
        assert_eq!(tags.coverage(), coverage);
        assert_eq!(tags.tags(), expected);
    }
    Ok(())
}

#[test]
fn given_healthimaging_summary_when_arn_is_omitted_then_canonical_fallback_matches_detail_not_raw_id()
-> TestResult {
    // Given: Smithy requires datastoreId but makes the summary ARN optional.
    let id = "0123456789abcdef0123456789abcdef";
    let arn = format!("arn:aws:medical-imaging:us-east-1:123456789012:datastore/{id}");
    let bytes = compiled(
        "medical-imaging",
        "2023-07-19",
        &["ListDatastores", "GetDatastore"],
        "AWS::HealthImaging::Datastore",
    )?;
    let runtime = RuntimeCatalog::from_bytes(&bytes)?;
    let list = runtime
        .lookup("medical-imaging", "ListDatastores")
        .ok_or("list absent")?;
    let detail = runtime
        .lookup("medical-imaging", "GetDatastore")
        .ok_or("detail absent")?;
    let listed = rows(
        list,
        json!({"datastoreSummaries":[{"datastoreId":id,"datastoreName":"images","datastoreStatus":"ACTIVE"}]}),
    )?;
    let described = rows(
        detail,
        json!({"datastoreProperties":{"datastoreId":id,"datastoreName":"images","datastoreStatus":"ACTIVE","datastoreArn":arn}}),
    )?;
    // When: the summary lacks an ARN but the metadata detail has its real canonical ARN.
    let list_key = key(list, listed.first().ok_or("summary absent")?, "us-east-1")?;
    let detail_key = key(
        detail,
        described.first().ok_or("detail absent")?,
        "us-east-1",
    )?;
    // Then: the fallback is a scoped ARN and converges with the direct detail identity.
    assert_eq!(list_key, detail_key);
    assert_eq!(list_key.id.as_str(), arn);
    assert_eq!(list_key.kind, Kind::parse("AWS::HealthImaging::Datastore")?);
    // Optional ARNs in detail obey the same absent/null fallback contract as list summaries.
    for properties in [
        json!({"datastoreId":id}),
        json!({"datastoreId":id,"datastoreArn":null}),
    ] {
        let described = rows(detail, json!({"datastoreProperties":properties}))?;
        assert_eq!(
            key(
                detail,
                described.first().ok_or("fallback detail absent")?,
                "us-east-1"
            )?,
            list_key
        );
    }
    Ok(())
}

#[test]
fn given_healthimaging_explicit_arn_when_decoded_then_actual_partition_is_preserved_byte_for_byte()
-> TestResult {
    // Given: an actual ARN exists, so construction from restricted fallback context is unnecessary.
    let id = "0123456789abcdef0123456789abcdef";
    let arn = format!("arn:aws-cn:medical-imaging:cn-north-1:123456789012:datastore/{id}");
    let bytes = compiled(
        "medical-imaging",
        "2023-07-19",
        &["ListDatastores", "GetDatastore"],
        "AWS::HealthImaging::Datastore",
    )?;
    let runtime = RuntimeCatalog::from_bytes(&bytes)?;
    // When: both real modeled views carry the explicit ARN.
    for (name, response) in [
        (
            "ListDatastores",
            json!({"datastoreSummaries":[{"datastoreId":id,"datastoreName":"images","datastoreStatus":"ACTIVE","datastoreArn":arn}]}),
        ),
        (
            "GetDatastore",
            json!({"datastoreProperties":{"datastoreId":id,"datastoreName":"images","datastoreStatus":"ACTIVE","datastoreArn":arn}}),
        ),
    ] {
        let plan = runtime
            .lookup("medical-imaging", name)
            .ok_or("plan absent")?;
        let selected = rows(plan, response)?;
        assert_eq!(selected.len(), 1);
        // Then: the real ARN wins over any fallback construction and is never shortened to datastoreId.
        assert_eq!(
            key(plan, selected.first().ok_or("row absent")?, "cn-north-1")?
                .id
                .as_str(),
            arn
        );
    }
    Ok(())
}

#[test]
fn given_healthimaging_canonical_owner_when_tag_endpoint_decodes_then_snapshot_targets_same_resource()
-> TestResult {
    // Given: the optional summary ARN is absent; the reviewed fallback still owns the real datastore.
    let id = "0123456789abcdef0123456789abcdef";
    let bytes = compiled(
        "medical-imaging",
        "2023-07-19",
        &["ListDatastores", "GetDatastore", "ListTagsForResource"],
        "AWS::HealthImaging::Datastore",
    )?;
    let runtime = RuntimeCatalog::from_bytes(&bytes)?;
    let list = runtime
        .lookup("medical-imaging", "ListDatastores")
        .ok_or("list absent")?;
    let tags = runtime
        .lookup("medical-imaging", "ListTagsForResource")
        .ok_or("tag endpoint absent")?;
    let PlanPurpose::TagEndpoint(endpoint) = tags.purpose() else {
        return Err("tag endpoint is not a tag facet".into());
    };
    let parent = endpoint.parents.first().ok_or("tag parent absent")?;
    assert_eq!(parent.operation.as_ref(), "ListDatastores");
    let request = parent.request.as_ref().ok_or("tag request absent")?;
    assert_eq!(request.source, TagEndpointSource::OwnerIdentity);
    assert!(request.parent.is_empty());
    assert_eq!(request.input.len(), 1);
    assert_eq!(
        tags.service()
            .field_name(*request.input.first().ok_or("tag input absent")?)?,
        "resourceArn"
    );
    let summaries = rows(list, json!({"datastoreSummaries":[{"datastoreId":id}]}))?;
    let owner = key(
        list,
        summaries.first().ok_or("summary absent")?,
        "us-east-1",
    )?;
    assert_eq!(
        owner.id.as_str(),
        format!("arn:aws:medical-imaging:us-east-1:123456789012:datastore/{id}")
    );
    // When: the production wire decoder and tag extractor consume populated and empty map snapshots.
    for (response, expected) in [
        (
            json!({"tags":{"owner":"radiology"}}),
            vec![Tag::new("owner", "radiology")?],
        ),
        (json!({"tags":{}}), vec![]),
    ] {
        let response_rows = rows(tags, response)?;
        let row = response_rows.first().ok_or("tag response absent")?;
        let snapshot = tag_observation(row, &owner, &endpoint.tags)?;
        // Then: no fabricated tag resource is created; explicit emptiness is authoritative.
        assert_eq!(snapshot.resource(), &owner);
        assert_eq!(snapshot.coverage(), TagCoverage::Complete);
        assert_eq!(snapshot.tags(), expected);
    }
    Ok(())
}

#[test]
fn given_healthimaging_present_bad_arn_when_decoded_then_canonical_fallback_is_not_used()
-> TestResult {
    let id = "0123456789abcdef0123456789abcdef";
    let bytes = compiled(
        "medical-imaging",
        "2023-07-19",
        &["ListDatastores", "GetDatastore"],
        "AWS::HealthImaging::Datastore",
    )?;
    let runtime = RuntimeCatalog::from_bytes(&bytes)?;
    for arn in ["", "not-an-arn"] {
        for (operation, response) in [
            (
                "ListDatastores",
                json!({"datastoreSummaries":[{"datastoreId":id,"datastoreArn":arn}]}),
            ),
            (
                "GetDatastore",
                json!({"datastoreProperties":{"datastoreId":id,"datastoreArn":arn}}),
            ),
        ] {
            let plan = runtime
                .lookup("medical-imaging", operation)
                .ok_or("operation absent")?;
            let selected = rows(plan, response)?;
            assert!(key(plan, selected.first().ok_or("row absent")?, "us-east-1").is_err());
        }
    }
    Ok(())
}

#[test]
fn given_license_tag_endpoint_when_decoded_then_authoritative_snapshot_preserves_discovered_configuration_owner()
-> TestResult {
    let arn = "arn:aws:license-manager:us-east-1:123456789012:license-configuration:lic-123";
    let bytes = compiled(
        "license-manager",
        "2018-08-01",
        &[
            "ListLicenseConfigurations",
            "GetLicenseConfiguration",
            "ListTagsForResource",
        ],
        "AWS::LicenseManager::LicenseConfiguration",
    )?;
    let runtime = RuntimeCatalog::from_bytes(&bytes)?;
    let list = runtime
        .lookup("license-manager", "ListLicenseConfigurations")
        .ok_or("list absent")?;
    let tags = runtime
        .lookup("license-manager", "ListTagsForResource")
        .ok_or("tag endpoint absent")?;
    let PlanPurpose::TagEndpoint(endpoint) = tags.purpose() else {
        return Err("standalone tags must be a facet".into());
    };
    let parent = endpoint.parents.first().ok_or("tag parent absent")?;
    assert_eq!(parent.operation.as_ref(), "ListLicenseConfigurations");
    let request = parent.request.as_ref().ok_or("tag request absent")?;
    assert_eq!(request.source, TagEndpointSource::Response);
    assert_eq!(
        list.service()
            .field_name(*request.parent.first().ok_or("parent ARN path absent")?)?,
        "LicenseConfigurationArn"
    );
    assert_eq!(
        tags.service()
            .field_name(*request.input.first().ok_or("request ARN path absent")?)?,
        "ResourceArn"
    );
    let summaries = rows(
        list,
        json!({"LicenseConfigurations":[{"LicenseConfigurationArn":arn}]}),
    )?;
    let owner = key(
        list,
        summaries.first().ok_or("summary absent")?,
        "us-east-1",
    )?;
    for (response, coverage, expected) in [
        (
            json!({"Tags":[{"Key":"owner","Value":"platform"}]}),
            TagCoverage::Complete,
            vec![Tag::new("owner", "platform")?],
        ),
        (json!({"Tags":[]}), TagCoverage::Complete, vec![]),
        (json!({}), TagCoverage::Unobserved, vec![]),
    ] {
        let response_rows = rows(tags, response)?;
        let row = response_rows.first().ok_or("tag response absent")?;
        let snapshot = tag_observation(row, &owner, &endpoint.tags)?;
        assert_eq!(snapshot.resource(), &owner);
        assert_eq!(snapshot.coverage(), coverage);
        assert_eq!(snapshot.tags(), expected);
    }
    Ok(())
}

#[test]
fn given_neptune_shared_cluster_api_when_runtime_request_is_encoded_then_engine_filter_reaches_query_wire()
-> TestResult {
    // Given: a real pinned Neptune model and reviewed structured fixed-input policy.
    let bytes = compiled(
        "neptune",
        "2014-10-31",
        &["DescribeDBClusters"],
        "AWS::Neptune::DBCluster",
    )?;
    let runtime = RuntimeCatalog::from_bytes(&bytes)?;
    let plan = runtime
        .lookup("neptune", "DescribeDBClusters")
        .ok_or("cluster list absent")?;
    // When: production root input construction and protocol selection serialize the request.
    let input = crate::sweep_runtime::input::root_bindings(plan, None, None)?;
    let request =
        crate::sweep_runtime::request::encode(plan, "https://fixture.example.test", &input, &[])?;
    let body = std::str::from_utf8(request.body())?;
    let pairs = body
        .split('&')
        .map(|pair| pair.split_once('=').ok_or("query pair missing value"))
        .collect::<Result<BTreeMap<_, _>, _>>()?;
    // Then: this is a native Query structure list, not JSON text or a service-ARN heuristic.
    assert_eq!(pairs.get("Action"), Some(&"DescribeDBClusters"));
    assert_eq!(pairs.get("Version"), Some(&"2014-10-31"));
    assert_eq!(pairs.get("Filters.Filter.1.Name"), Some(&"engine"));
    assert_eq!(
        pairs.get("Filters.Filter.1.Values.Value.1"),
        Some(&"neptune")
    );
    assert_eq!(pairs.len(), 4);
    Ok(())
}

#[tokio::test]
async fn given_persisted_license_summary_when_discovered_detail_tags_become_empty_then_durable_tags_are_removed_without_duplicate_resource()
-> TestResult {
    // Given: actual compiled list/detail policies and a decoded summary already persisted in SQLite.
    let arn = "arn:aws:license-manager:us-east-1:123456789012:license-configuration:lic-123";
    let bytes = compiled(
        "license-manager",
        "2018-08-01",
        &["ListLicenseConfigurations", "GetLicenseConfiguration"],
        "AWS::LicenseManager::LicenseConfiguration",
    )?;
    let runtime = RuntimeCatalog::from_bytes(&bytes)?;
    let list = runtime
        .lookup("license-manager", "ListLicenseConfigurations")
        .ok_or("list absent")?;
    let detail = runtime
        .lookup("license-manager", "GetLicenseConfiguration")
        .ok_or("detail absent")?;
    let summary = rows(
        list,
        json!({"LicenseConfigurations":[{"LicenseConfigurationArn":arn}]}),
    )?
    .into_iter()
    .next()
    .ok_or("summary absent")?;
    let input = fixture::discovered_input(detail, &summary)?;
    let request =
        crate::sweep_runtime::request::encode(detail, "https://fixture.example.test", &input, &[])?;
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(request.body())?.get("LicenseConfigurationArn"),
        Some(&json!(arn))
    );
    let (_directory, store, database) =
        fixture::listed_store(&bytes, &runtime, list, summary).await?;
    // When/Then: the discovered metadata request first populates its exact owner's durable tags.
    fixture::assert_snapshot(
        &store,
        &database,
        detail,
        &input,
        (
            "license-tagged",
            11,
            json!({"LicenseConfigurationArn":arn,"Tags":[{"Key":"owner","Value":"platform"}]}),
            Some("owner=platform"),
        ),
    )
    .await?;
    // When/Then: the next authoritative empty snapshot removes tags, retaining exactly one resource.
    fixture::assert_snapshot(
        &store,
        &database,
        detail,
        &input,
        (
            "license-empty",
            21,
            json!({"LicenseConfigurationArn":arn,"Tags":[]}),
            None,
        ),
    )
    .await?;
    Ok(())
}

#[test]
fn given_crossaccount_oam_link_when_builtin_relation_projects_owner_then_sink_account_is_preserved_and_invalid_arns_fail_closed()
-> TestResult {
    use inventory_core::{
        inventory::{Observation, RegionId},
        record::Exclusions,
    };
    let bytes = compiled(
        "oam",
        "2022-06-10",
        &["ListLinks", "ListSinks"],
        "AWS::Oam::Link",
    )?;
    let runtime = RuntimeCatalog::from_bytes(&bytes)?;
    let plan = runtime.lookup("oam", "ListLinks").ok_or("links absent")?;
    let relation = runtime
        .relationships()
        .iter()
        .find(|relation| relation.relation.as_str() == "TARGETS_SINK")
        .ok_or("builtin sink relation absent")?;
    let sink = "arn:aws:oam:us-east-1:210987654321:sink/00000000-0000-0000-0000-000000000001";
    // Given: the production wire decoder produces an A-owned link pointing at monitoring account B.
    let row = rows(plan, json!({"Items":[{"Arn":"arn:aws:oam:us-east-1:123456789012:link/00000000-0000-0000-0000-000000000002","SinkArn":sink}]}))?
        .into_iter().next().ok_or("link row absent")?;
    let owner = key(plan, &row, "us-east-1")?;
    let observation = Observation::new(owner, row, &Exclusions::none(), None)?;
    let home = RegionId::parse("us-east-1")?;
    // When/Then: reviewed ARN ownership replaces source-account inheritance only for this relation.
    let (account, region) = crate::sweep_runtime::resource_projection::reference_owner(
        relation,
        &observation,
        sink,
        &home,
    )?;
    assert_eq!(account.as_str(), "210987654321");
    assert_eq!(region, observation.key().region);
    assert_ne!(account, observation.key().account);
    for invalid in [
        "not-an-arn",
        "arn:aws-cn:oam:us-east-1:210987654321:sink/00000000-0000-0000-0000-000000000001",
        "arn:aws:sns:us-east-1:210987654321:sink/00000000-0000-0000-0000-000000000001",
        "arn:aws:oam:us-east-1:210987654321:link/00000000-0000-0000-0000-000000000001",
        "arn:aws:oam:us-west-2:210987654321:sink/00000000-0000-0000-0000-000000000001",
    ] {
        assert!(
            crate::sweep_runtime::resource_projection::reference_owner(
                relation,
                &observation,
                invalid,
                &home
            )
            .is_err()
        );
    }
    Ok(())
}
