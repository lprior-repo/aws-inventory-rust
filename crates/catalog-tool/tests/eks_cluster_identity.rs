use catalog_tool as fixture_catalog;
use inventory_core::catalog::runtime::{BindingSource, IdentityPart, IdentityPlan, RuntimeCatalog};

#[path = "../src/overlay/builtin/family_a_c/consumer_fixture.rs"]
mod model;

#[test]
fn given_eks_cluster_names_when_the_policy_compiles_then_the_key_matches_describe_cluster_arn()
-> model::TestResult {
    let compiled = model::compile(
        "eks",
        "2017-11-01",
        &["ListClusters", "DescribeCluster", "ListCapabilities"],
    )?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    let clusters = catalog
        .lookup("eks", "ListClusters")
        .ok_or("missing cluster list plan")?;
    assert_eq!(clusters.kind().as_str(), "AWS::EKS::Cluster");
    assert_eq!(
        clusters.identity(),
        &IdentityPlan::Template(
            vec![
                IdentityPart::Literal("arn:".to_owned()),
                IdentityPart::Partition,
                IdentityPart::Literal(":eks:".to_owned()),
                IdentityPart::Region,
                IdentityPart::Literal(":".to_owned()),
                IdentityPart::Account,
                IdentityPart::Literal(":cluster/".to_owned()),
                IdentityPart::Field(Box::new([])),
            ]
            .into_boxed_slice()
        )
    );
    let capabilities = catalog
        .lookup("eks", "ListCapabilities")
        .ok_or("missing capability list plan")?;
    let cluster_name = compiled.fields.id("clusterName").ok_or("missing field")?;
    assert!(capabilities.bindings().iter().any(|binding| {
        binding.path.as_ref() == [cluster_name]
            && matches!(
                &binding.source,
                BindingSource::Operation { service, operation, path }
                    if service.as_ref() == "eks" && operation.as_ref() == "ListClusters" && path.is_empty()
            )
    }));
    Ok(())
}
#[test]
fn given_listed_eks_cluster_when_detail_plan_compiles_then_secret_and_name_binding_are_preserved()
-> model::TestResult {
    let compiled = model::compile("eks", "2017-11-01", &["ListClusters", "DescribeCluster"])?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    let detail = catalog
        .lookup("eks", "DescribeCluster")
        .ok_or("missing cluster detail plan")?;
    assert_eq!(detail.kind().as_str(), "AWS::EKS::Cluster");
    let name = compiled
        .fields
        .id("name")
        .ok_or("missing cluster name field")?;
    assert!(detail.bindings().iter().any(|binding| {
        binding.path.as_ref() == [name]
            && matches!(
                &binding.source,
                BindingSource::Operation { service, operation, path }
                    if service.as_ref() == "eks" && operation.as_ref() == "ListClusters" && path.is_empty()
            )
    }));
    let activation_code = compiled
        .fields
        .id("activationCode")
        .ok_or("missing activation code field")?;
    assert!(detail.excluded_members().contains(activation_code));
    Ok(())
}
