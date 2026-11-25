use catalog_tool as fixture_catalog;
use inventory_core::{
    catalog::{
        Service, Shape, Target,
        runtime::{IdentityPart, IdentityPlan, RuntimeCatalog},
    },
    record::FieldId,
};

#[path = "../src/overlay/builtin/family_a_c/consumer_fixture.rs"]
mod model;

#[test]
fn given_codebuild_project_names_when_the_policy_compiles_then_the_key_matches_describe_project_arn()
-> model::TestResult {
    let compiled = model::compile(
        "codebuild",
        "2016-10-06",
        &["ListProjects", "BatchGetProjects"],
    )?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    let projects = catalog
        .lookup("codebuild", "ListProjects")
        .ok_or("missing project list plan")?;
    assert_eq!(projects.kind().as_str(), "AWS::CodeBuild::Project");
    assert_eq!(
        projects.identity(),
        &IdentityPlan::Template(
            vec![
                IdentityPart::Literal("arn:".to_owned()),
                IdentityPart::Partition,
                IdentityPart::Literal(":codebuild:".to_owned()),
                IdentityPart::Region,
                IdentityPart::Literal(":".to_owned()),
                IdentityPart::Account,
                IdentityPart::Literal(":project/".to_owned()),
                IdentityPart::Field(Box::new([])),
            ]
            .into_boxed_slice()
        )
    );
    assert!(catalog.lookup("codebuild", "BatchGetProjects").is_none());
    let service = projects.service();
    let output = response_shape(service, "BatchGetProjects")?;
    let rows_id = compiled
        .fields
        .id("projects")
        .ok_or("missing projects field")?;
    let arn_id = compiled.fields.id("arn").ok_or("missing ARN field")?;
    let rows = shape(service, member_target(output, rows_id)?)?;
    let row = shape(
        service,
        rows.element()?.ok_or("missing project list element")?,
    )?;
    assert_eq!(
        shape(service, member_target(row, arn_id)?)?.kind()?,
        "string"
    );
    Ok(())
}

#[test]
fn given_glue_blueprint_names_when_the_policy_compiles_then_the_key_uses_the_blueprint_arn_format()
-> model::TestResult {
    let compiled = model::compile("glue", "2017-03-31", &["ListBlueprints", "GetBlueprint"])?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    let blueprints = catalog
        .lookup("glue", "ListBlueprints")
        .ok_or("missing blueprint list plan")?;
    assert_eq!(blueprints.kind().as_str(), "AWS::Glue::Blueprint");
    assert_eq!(
        blueprints.identity(),
        &IdentityPlan::Template(
            vec![
                IdentityPart::Literal("arn:".to_owned()),
                IdentityPart::Partition,
                IdentityPart::Literal(":glue:".to_owned()),
                IdentityPart::Region,
                IdentityPart::Literal(":".to_owned()),
                IdentityPart::Account,
                IdentityPart::Literal(":blueprint/".to_owned()),
                IdentityPart::Field(Box::new([])),
            ]
            .into_boxed_slice()
        )
    );
    assert!(catalog.lookup("glue", "GetBlueprint").is_none());
    let service = blueprints.service();
    let output = response_shape(service, "GetBlueprint")?;
    let blueprint_id = compiled
        .fields
        .id("Blueprint")
        .ok_or("missing blueprint field")?;
    let name_id = compiled.fields.id("Name").ok_or("missing name field")?;
    let row = shape(service, member_target(output, blueprint_id)?)?;
    assert_eq!(
        shape(service, member_target(row, name_id)?)?.kind()?,
        "string"
    );
    Ok(())
}

fn response_shape<'a>(service: Service<'a>, name: &str) -> model::TestResult<Shape<'a>> {
    for operation in service.operations()? {
        let operation = operation?;
        if operation.name()? == name {
            return shape(service, operation.output()?);
        }
    }
    Err(format!("missing modeled operation {name}").into())
}

fn shape(service: Service<'_>, target: Target) -> model::TestResult<Shape<'_>> {
    let Target::Shape(index) = target else {
        return Err("expected a modeled shape".into());
    };
    Ok(service.shape(index)?.ok_or("missing modeled shape")?)
}

fn member_target(shape: Shape<'_>, id: FieldId) -> model::TestResult<Target> {
    for member in shape.members()? {
        let member = member?;
        if member.id == id {
            return Ok(member.target);
        }
    }
    Err("missing modeled member".into())
}
