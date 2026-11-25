use super::consumer_fixture::{TestResult, compile};
use inventory_core::{
    catalog::runtime::{
        BindingSource, BindingValue, IdentityPlan, ItemPlan, RelationScope, RuntimeCatalog,
        TagPlan, TokenType,
    },
    record::FieldId,
};

fn field(fields: &crate::smithy::Registry, name: &str) -> TestResult<FieldId> {
    fields
        .id(name)
        .ok_or_else(|| format!("missing fixture field {name}").into())
}

type CursorCase = (
    &'static str,
    &'static str,
    &'static [(&'static str, &'static str, &'static str)],
);

#[test]
fn given_items_only_smithy_pagination_when_compiled_and_decoded_then_all_eight_real_cursors_survive()
-> TestResult {
    let groups: &[CursorCase] = &[
        (
            "accessanalyzer",
            "2019-11-01",
            &[("ListAnalyzers", "nextToken", "analyzers")],
        ),
        (
            "backupsearch",
            "2018-05-10",
            &[("ListSearchJobs", "NextToken", "SearchJobs")],
        ),
        (
            "backup-gateway",
            "2021-01-01",
            &[
                ("ListGateways", "NextToken", "Gateways"),
                ("ListHypervisors", "NextToken", "Hypervisors"),
                ("ListVirtualMachines", "NextToken", "VirtualMachines"),
            ],
        ),
        (
            "cleanrooms",
            "2022-02-17",
            &[
                ("ListCollaborations", "nextToken", "collaborationList"),
                (
                    "ListConfiguredTables",
                    "nextToken",
                    "configuredTableSummaries",
                ),
                ("ListMemberships", "nextToken", "membershipSummaries"),
            ],
        ),
    ];
    for (service, date, cases) in groups {
        let operations = cases
            .iter()
            .map(|(operation, _, _)| *operation)
            .collect::<Vec<_>>();
        let compiled = compile(service, date, &operations)?;
        let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
        assert_eq!(catalog.plans().len(), cases.len());
        for (operation, token_name, items) in *cases {
            let plan = catalog
                .lookup(service, operation)
                .ok_or("root plan missing")?;
            let token = plan
                .pagination()
                .tokens
                .first()
                .ok_or("continuation token missing")?;
            let token_id = field(&compiled.fields, token_name)?;
            assert_eq!(plan.pagination().tokens.len(), 1, "{service}.{operation}");
            assert_eq!(
                token.request_path.as_ref(),
                &[token_id],
                "{service}.{operation}"
            );
            assert_eq!(
                token.response_path.as_ref(),
                &[token_id],
                "{service}.{operation}"
            );
            assert_eq!(token.value_type, TokenType::String);
            assert!(!token.optional);
            assert_eq!(
                plan.items(),
                &ItemPlan::Path(vec![field(&compiled.fields, items)?].into_boxed_slice())
            );
        }
    }
    Ok(())
}

#[test]
fn given_domain_association_and_archive_views_when_compiled_then_each_pair_has_one_canonical_identity()
-> TestResult {
    let cases = [
        (
            "apigatewayv2",
            "2018-11-29",
            "GetDomainNames",
            "GetDomainName",
            "DomainNameArn",
            "DomainName",
            "AWS::ApiGatewayV2::DomainName",
            true,
        ),
        (
            "appconfig",
            "2019-10-09",
            "ListExtensionAssociations",
            "GetExtensionAssociation",
            "Id",
            "Id",
            "AWS::AppConfig::ExtensionAssociation",
            false,
        ),
        (
            "cloudwatch-events",
            "2015-10-07",
            "ListArchives",
            "DescribeArchive",
            "ArchiveName",
            "ArchiveName",
            "AWS::Events::Archive",
            false,
        ),
    ];
    for (service, date, list, detail, identity, source, kind, arn) in cases {
        let compiled = compile(service, date, &[list, detail])?;
        let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
        let root = catalog.lookup(service, list).ok_or("root missing")?;
        let expanded = catalog.lookup(service, detail).ok_or("detail missing")?;
        let path = vec![field(&compiled.fields, identity)?].into_boxed_slice();
        let expected = if arn {
            IdentityPlan::Arn(path)
        } else {
            IdentityPlan::Key(vec![path].into_boxed_slice())
        };
        assert_eq!(root.identity(), &expected, "{service}.{list}");
        assert_eq!(expanded.identity(), &expected, "{service}.{detail}");
        assert_eq!(
            (root.kind().as_str(), expanded.kind().as_str()),
            (kind, kind)
        );
        assert_eq!(
            catalog
                .schemas()
                .iter()
                .filter(|schema| schema.kind().as_str() == kind)
                .count(),
            1
        );
        let binding = expanded
            .bindings()
            .first()
            .ok_or("detail binding missing")?;
        let source_field = field(&compiled.fields, source)?;
        assert!(
            matches!(&binding.source, BindingSource::Operation { service: parent_service, operation, path }
            if parent_service.as_ref() == service && operation.as_ref() == list && path.as_ref() == [source_field])
        );
    }
    Ok(())
}

#[test]
fn given_enabled_baselines_when_compiled_then_child_baseline_inclusion_is_an_outbound_binding()
-> TestResult {
    let compiled = compile("controltower", "2018-05-10", &["ListEnabledBaselines"])?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    let plan = catalog
        .lookup("controltower", "ListEnabledBaselines")
        .ok_or("baseline plan missing")?;
    let include = field(&compiled.fields, "includeChildren")?;
    assert_eq!(plan.bindings().len(), 1);
    let binding = plan.bindings().first().ok_or("includeChildren missing")?;
    assert_eq!(binding.path.as_ref(), &[include]);
    assert_eq!(
        binding.source,
        BindingSource::Fixed(BindingValue::Boolean(true))
    );
    assert_eq!(
        plan.identity(),
        &IdentityPlan::Arn(vec![field(&compiled.fields, "arn")?].into_boxed_slice())
    );
    Ok(())
}

#[test]
fn given_inference_profiles_when_compiled_then_only_application_profiles_are_requested_as_resources()
-> TestResult {
    let compiled = compile("bedrock", "2023-04-20", &["ListInferenceProfiles"])?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    let plan = catalog
        .lookup("bedrock", "ListInferenceProfiles")
        .ok_or("profile plan missing")?;
    assert_eq!(catalog.plans().len(), 1);
    assert_eq!(
        plan.kind().as_str(),
        "AWS::Bedrock::ApplicationInferenceProfile"
    );
    assert_eq!(
        plan.identity(),
        &IdentityPlan::Arn(
            vec![field(&compiled.fields, "inferenceProfileArn")?].into_boxed_slice()
        )
    );
    let binding = plan
        .bindings()
        .first()
        .ok_or("APPLICATION filter missing")?;
    assert_eq!(plan.bindings().len(), 1);
    assert_eq!(
        binding.path.as_ref(),
        &[field(&compiled.fields, "typeEquals")?]
    );
    assert_eq!(
        binding.source,
        BindingSource::Fixed(BindingValue::String("APPLICATION".to_owned()))
    );
    assert_eq!(plan.pagination().tokens.len(), 1);
    let cursor = plan
        .pagination()
        .tokens
        .first()
        .ok_or("profile cursor missing")?;
    assert_eq!(
        cursor.request_path.as_ref(),
        &[field(&compiled.fields, "nextToken")?]
    );
    assert_eq!(cursor.response_path, cursor.request_path);
    let schema = catalog
        .schemas()
        .iter()
        .find(|schema| schema.kind().as_str() == plan.kind().as_str())
        .ok_or("profile schema missing")?;
    assert_eq!(
        schema.table().as_str(),
        "aws_bedrock_application_inference_profile"
    );
    Ok(())
}

#[test]
fn given_portal_summaries_when_compiled_then_configuration_tags_and_cursor_are_consumable()
-> TestResult {
    let compiled = compile("apigatewayv2", "2018-11-29", &["ListPortals"])?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    let plan = catalog
        .lookup("apigatewayv2", "ListPortals")
        .ok_or("portal root missing")?;
    assert_eq!(plan.kind().as_str(), "AWS::ApiOnly::ApiGatewayV2Portal");
    assert_eq!(
        plan.identity(),
        &IdentityPlan::Arn(vec![field(&compiled.fields, "PortalArn")?].into_boxed_slice())
    );
    assert_eq!(
        plan.tags(),
        Some(&TagPlan::Map {
            path: vec![field(&compiled.fields, "Tags")?].into_boxed_slice()
        })
    );
    assert_eq!(plan.pagination().tokens.len(), 1);
    let cursor = plan
        .pagination()
        .tokens
        .first()
        .ok_or("portal cursor missing")?;
    assert_eq!(
        cursor.request_path.as_ref(),
        &[field(&compiled.fields, "NextToken")?]
    );
    assert_eq!(cursor.response_path, cursor.request_path);
    Ok(())
}

#[test]
fn given_archives_and_buses_when_compiled_then_exact_account_region_dependency_survives()
-> TestResult {
    let compiled = compile(
        "cloudwatch-events",
        "2015-10-07",
        &[
            "ListArchives",
            "DescribeArchive",
            "ListEventBuses",
            "DescribeEventBus",
        ],
    )?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    let matching = catalog
        .relationships()
        .iter()
        .filter(|relation| {
            relation.source_kind.as_str() == "AWS::Events::Archive"
                && relation.target_kind.as_str() == "AWS::Events::EventBus"
        })
        .collect::<Vec<_>>();
    assert_eq!(matching.len(), 1);
    let relation = matching.first().ok_or("archive bus relation missing")?;
    assert_eq!(
        relation.source_path.as_ref(),
        &[field(&compiled.fields, "EventSourceArn")?]
    );
    assert_eq!(
        relation.target_path.as_ref(),
        &[field(&compiled.fields, "Arn")?]
    );
    assert_eq!(relation.relation.as_str(), "ON_EVENT_BUS");
    assert_eq!(relation.scope, RelationScope::AccountRegion);
    Ok(())
}
