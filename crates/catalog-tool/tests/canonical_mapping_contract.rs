use catalog_tool::{endpoints, kind_map, names, overlay, runtime_catalog, smithy};
use inventory_core::{
    catalog::runtime::{BindingSource, IdentityPlan, ItemPlan, RuntimeCatalog},
    inventory::ScalarType,
};
use inventory_engine::inventory::{BinaryStore, SchemaRegistry};
use serde_json::json;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::{collections::BTreeMap, fs, path::Path};

#[path = "canonical_mapping_contract/domain.rs"]
mod domain;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

const KIND: &str = "AWS::Lambda::Function";
const TABLE: &str = "aws_lambda_function";
const FIELDS: &str = "CodeSize\nConfiguration\nFunctionArn\nFunctionName\nFunctions\nRuntime\nRuntimes\n";

// A bounded, local pinned-style model: both Lambda views select FunctionConfiguration.
// Scalar conflict fixtures deliberately replace those two views, not the canonical kind.
fn model(list_target: &str, detail_target: &str, root_detail: bool) -> serde_json::Value {
    let detail_output = if root_detail { detail_target } else { "n#GetOutput" };
    json!({"smithy":"2.0","shapes":{
        "n#Service":{"type":"service","version":"2015-03-31","operations":[{"target":"n#ListFunctions"},{"target":"n#GetFunctionConfiguration"}],"traits":{"aws.api#service":{"sdkId":"Lambda","endpointPrefix":"lambda","arnNamespace":"lambda"},"aws.auth#sigv4":{"name":"lambda"},"aws.protocols#restJson1":{}}},
        "n#ListFunctions":{"type":"operation","input":{"target":"n#ListInput"},"output":{"target":"n#ListOutput"},"traits":{"smithy.api#http":{"method":"GET","uri":"/2015-03-31/functions/"}}},
        "n#GetFunctionConfiguration":{"type":"operation","input":{"target":"n#GetInput"},"output":{"target":detail_output},"traits":{"smithy.api#http":{"method":"GET","uri":"/2015-03-31/functions/{FunctionName}"}}},
        "n#ListInput":{"type":"structure","members":{"Runtimes":{"target":"n#Runtimes"}}},
        "n#GetInput":{"type":"structure","members":{"FunctionName":{"target":if list_target == "n#FunctionConfiguration" { "smithy.api#String" } else { list_target },"traits":{"smithy.api#required":{},"smithy.api#httpLabel":{}}}}},
        "n#ListOutput":{"type":"structure","members":{"Functions":{"target":"n#Functions"}}},
        "n#GetOutput":{"type":"structure","members":{"Configuration":{"target":detail_target}}},
        "n#Functions":{"type":"list","member":{"target":list_target}},
        "n#IntegerItems":{"type":"list","member":{"target":"smithy.api#Long"}},
        "n#TextItems":{"type":"list","member":{"target":"smithy.api#String"}},
        "n#FunctionConfiguration":{"type":"structure","members":{"FunctionArn":{"target":"smithy.api#String"},"FunctionName":{"target":"smithy.api#String"},"CodeSize":{"target":"smithy.api#Long"},"Runtime":{"target":"n#Runtime"}}},
        "n#Runtime":{"type":"enum","members":{"PROVIDED":{"target":"smithy.api#Unit","traits":{"smithy.api#enumValue":"provided.al2023"}},"NODE":{"target":"smithy.api#Unit","traits":{"smithy.api#enumValue":"nodejs22.x"}}}},
        "n#Runtimes":{"type":"list","member":{"target":"n#Runtime"}}
    }})
}

fn arn(path: &str) -> Result<overlay::Identity, overlay::NameError> {
    Ok(overlay::Identity::Arn(overlay::MemberPath::selected(path)?))
}

fn key(path: &str) -> Result<overlay::Identity, overlay::NameError> {
    Ok(overlay::Identity::Key(vec![overlay::MemberPath::selected(path)?]))
}

fn reviewed_overlay(list_identity: overlay::Identity, detail_identity: overlay::Identity, binding: &str, detail_items: &str, fixed: &[&str]) -> TestResult<overlay::ServiceOverlay> {
    let mut list = overlay::OperationOverlay::default();
    list.class = Some(overlay::OverlayClass::Enumerate);
    list.scope = Some(overlay::Scope::Region);
    list.items = Some(overlay::ItemSelector::parse("Functions")?);
    list.paginator = Some(overlay::Paginator::None);
    list.identity = Some(list_identity);
    if !fixed.is_empty() {
        list.fixed.insert(overlay::MemberPath::new("Runtimes")?,
            overlay::FixedValue::Strings(fixed.iter().map(|value| (*value).to_owned()).collect()));
    }
    let mut detail = overlay::OperationOverlay::default();
    detail.class = Some(overlay::OverlayClass::Expand);
    detail.scope = Some(overlay::Scope::Region);
    detail.items = Some(overlay::ItemSelector::parse(detail_items)?);
    detail.paginator = Some(overlay::Paginator::None);
    detail.identity = Some(detail_identity);
    detail.bind.insert(overlay::MemberPath::new("FunctionName")?,
        overlay::Binding::From {
            operation: overlay::OperationName::new("ListFunctions")?,
            path: overlay::MemberPath::selected(binding)?,
        });
    Ok(overlay::ServiceOverlay::new("lambda", vec![("ListFunctions", list), ("GetFunctionConfiguration", detail)])?)
}

struct Fixture {
    directory: tempfile::TempDir,
    policy: overlay::ServiceOverlay,
}

impl Fixture {
    fn path(&self) -> &Path { self.directory.path() }
    fn close(self) -> std::io::Result<()> { self.directory.close() }
    fn check(&self) -> Result<overlay::CheckedOverlays, overlay::CheckError> {
        overlay::check_policies(&self.path().join("source"), std::slice::from_ref(&self.policy),
            &catalog_tool::classes::DenyList::parse("").map_err(overlay::CheckError::from)?)
    }
}

fn fixture(model: &serde_json::Value, review: &overlay::ServiceOverlay) -> TestResult<Fixture> {
    let directory = tempfile::tempdir()?;
    let model_dir = directory.path().join("source/models/lambda/service/2015-03-31");
    fs::create_dir_all(&model_dir)?;
    fs::write(model_dir.join("lambda.json"), serde_json::to_vec(model)?)?;
    Ok(Fixture { directory, policy: review.clone() })
}

fn verified_endpoint() -> TestResult<endpoints::VerifiedEndpoint> {
    let properties = json!({"authSchemes":[{"name":"sigv4","signingName":"lambda","signingRegion":"us-east-1"}]});
    let document = json!({"version":"1.0","parameters":{"Region":{"type":"string"}},"rules":[{"type":"endpoint","conditions":[],"endpoint":{"url":"https://lambda.example.test","properties":properties}}]});
    let rules = endpoints::RuleSet::parse(Path::new("lambda-fixture-endpoint.json"), &document)?;
    let cases: Vec<endpoints::Case> = serde_json::from_value(json!([{"params":{"Region":"us-east-1"},"expect":{"endpoint":{"url":"https://lambda.example.test","properties":properties}}}]))?;
    rules.run_cases(&cases)?.verified_endpoint(Some("lambda"))
        .map_err(|error| format!("fixture endpoint evidence failed: {error:?}").into())
}

fn compile(root: &Fixture, display_path: Option<&str>) -> TestResult<Result<Vec<u8>, runtime_catalog::RuntimeCatalogCompileError>> {
    compile_mapping(root, display_path, false)
}

fn compile_mapping(root: &Fixture, display_path: Option<&str>, include_inactive: bool) -> TestResult<Result<Vec<u8>, runtime_catalog::RuntimeCatalogCompileError>> {
    let source = root.path().join("source");
    let checked = root.check()?;
    let fields = smithy::Registry::parse(FIELDS)?;
    let base = smithy::build_catalog(&source, &fields, "canonical-mapping-fixture")?;
    let mut cfn = vec![kind_map::CfnMapping::Mapped { cfn_type: KIND, kind: KIND, display_name_path: display_path }];
    let mut operations = vec![
        kind_map::OperationMapping { service: "lambda", operation: "ListFunctions", kind: KIND },
        kind_map::OperationMapping { service: "lambda", operation: "GetFunctionConfiguration", kind: KIND },
    ];
    if include_inactive {
        cfn.push(kind_map::CfnMapping::Mapped { cfn_type: "AWS::EC2::VPC", kind: "AWS::EC2::VPC", display_name_path: Some("MissingName") });
        operations.push(kind_map::OperationMapping { service: "ec2", operation: "DescribeVpcs", kind: "AWS::EC2::VPC" });
    }
    let kinds = kind_map::MappingSource {
        cfn: &cfn,
        operations: &operations,
        ..kind_map::MappingSource::default()
    }.compile(&names::Exceptions::none())?;
    let endpoints = BTreeMap::from([("lambda".to_owned(), verified_endpoint()?)]);
    Ok(runtime_catalog::augment(&base, &checked, &kinds, &endpoints))
}

async fn assert_canonical_sqlite(database: &Path, bytes: &[u8], catalog: &RuntimeCatalog<'_>) -> TestResult {
    let registry = SchemaRegistry::new(bytes, catalog.schemas().to_vec())?;
    let store = BinaryStore::create(database, registry).await?;
    let reader = SqlitePoolOptions::new().max_connections(1)
        .connect_with(SqliteConnectOptions::new().filename(database).read_only(true)).await?;
    let kinds: Vec<(String, String)> = sqlx::query_as("SELECT kind,table_name FROM kinds ORDER BY kind")
        .fetch_all(&reader).await?;
    assert_eq!(kinds, vec![(KIND.to_owned(), TABLE.to_owned())]);
    let operations: Vec<(String, String)> = sqlx::query_as("SELECT service,operation FROM catalog_operations ORDER BY operation")
        .fetch_all(&reader).await?;
    assert_eq!(operations, vec![("lambda".to_owned(), "GetFunctionConfiguration".to_owned()), ("lambda".to_owned(), "ListFunctions".to_owned())]);
    let physical_columns: Vec<(String, String)> = sqlx::query_as("SELECT name,type FROM pragma_table_info(?1) ORDER BY cid")
        .bind(TABLE).fetch_all(&reader).await?;
    let expected_columns = [("account", "TEXT"), ("region", "TEXT"), ("id", "TEXT"),
        ("content_hash", "BLOB"), ("first_seen", "INTEGER"), ("last_changed", "INTEGER"),
        ("record", "BLOB"), ("f_4", "TEXT")].map(|(name, ty)| (name.to_owned(), ty.to_owned())).to_vec();
    assert_eq!(physical_columns, expected_columns);
    reader.close().await;
    drop(store);
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn given_list_and_singular_structure_when_compiled_then_one_canonical_identity_and_sqlite_schema_are_consumable() -> TestResult {
    // Given: real operation names share one CFN mapping; neither gets an API-only alias.
    for root_detail in [false, true] {
        let detail_items = if root_detail { "$" } else { "Configuration" };
        let review = reviewed_overlay(arn("FunctionArn")?, arn("FunctionArn")?, "FunctionArn", detail_items, &[])?;
        let root = fixture(&model("n#FunctionConfiguration", "n#FunctionConfiguration", root_detail), &review)?;

        // When: the public compiler and production artifact decoder consume the review.
        let bytes = compile(&root, Some("FunctionName"))??;
        let catalog = RuntimeCatalog::from_bytes(&bytes)?;
        let fields = smithy::Registry::parse(FIELDS)?;
        let arn = fields.id("FunctionArn").ok_or("FunctionArn registry entry missing")?;
        let name = fields.id("FunctionName").ok_or("FunctionName registry entry missing")?;
        let functions = fields.id("Functions").ok_or("Functions registry entry missing")?;
        let configuration = fields.id("Configuration").ok_or("Configuration registry entry missing")?;
        let list = catalog.lookup("lambda", "ListFunctions").ok_or("list plan missing")?;
        let detail = catalog.lookup("lambda", "GetFunctionConfiguration").ok_or("detail plan missing")?;

        // Then: wrapper selection is distinct, but identity and schema stay item-relative.
        assert_eq!(catalog.plans().len(), 2);
        assert_eq!((list.kind().as_str(), detail.kind().as_str()), (KIND, KIND));
        assert_eq!(list.identity(), &IdentityPlan::Arn(vec![arn].into_boxed_slice()));
        assert_eq!(detail.identity(), list.identity());
        assert_eq!(list.items(), &ItemPlan::Path(vec![functions].into_boxed_slice()));
        let expected_detail = if root_detail { ItemPlan::Root } else { ItemPlan::Path(vec![configuration].into_boxed_slice()) };
        assert_eq!(detail.items(), &expected_detail);
        let binding = detail.bindings().first().ok_or("detail binding missing")?;
        assert_eq!(detail.bindings().len(), 1);
        assert!(matches!(&binding.source, BindingSource::Operation { service, operation, path }
            if service.as_ref() == "lambda" && operation.as_ref() == "ListFunctions" && path.as_ref() == [arn]));
        assert_eq!(catalog.schemas().len(), 1);
        let schema = catalog.schemas().first().ok_or("canonical schema missing")?;
        assert_eq!((schema.kind().as_str(), schema.table().as_str()), (KIND, TABLE));
        let columns = schema.columns().iter().map(|column| (column.path(), column.scalar())).collect::<Vec<_>>();
        assert_eq!(columns, vec![(std::slice::from_ref(&name), ScalarType::Text)]);

        // The real SQLite compiler must create exactly one table with the expected type.
        assert_canonical_sqlite(&root.path().join("canonical.sqlite"), &bytes, &catalog).await?;
        root.close()?;
    }
    Ok(())
}

#[test]
fn given_compatible_scalar_views_of_one_kind_when_compiled_then_one_selected_scalar_type_is_retained() -> TestResult {
    // Given: two reviewed operations select the same scalar type for the same CFN kind.
    for (list_target, detail_target, scalar) in [
        ("smithy.api#String", "n#TextItems", ScalarType::Text),
        ("smithy.api#Long", "n#IntegerItems", ScalarType::Integer),
    ] {
        let review = reviewed_overlay(key("$")?, key("$")?, "$", "Configuration", &[])?;
        let root = fixture(&model(list_target, detail_target, false), &review)?;

        // When: the same pipeline used by the incompatible-view regression compiles them.
        let bytes = compile(&root, None)??;
        let catalog = RuntimeCatalog::from_bytes(&bytes)?;

        // Then: neither plan is dropped and the shared schema has one exact scalar column.
        assert_eq!(catalog.plans().len(), 2);
        assert_eq!(catalog.schemas().len(), 1);
        let schema = catalog.schemas().first().ok_or("canonical scalar schema missing")?;
        assert_eq!(schema.kind().as_str(), KIND);
        let selected = schema.columns().iter().filter(|column| column.path().is_empty())
            .map(|column| (column.name(), column.scalar())).collect::<Vec<_>>();
        assert_eq!(selected, vec![("value", scalar)]);
        root.close()?;
    }
    Ok(())
}

#[test]
fn given_incompatible_scalar_views_of_one_kind_when_compiled_then_capability_error_rejects_schema_merge() -> TestResult {
    // Given: both assignments of scalar types must reject a text/integer identity domain split.
    for (list_target, detail_target) in [("smithy.api#String", "n#IntegerItems"), ("smithy.api#Long", "n#TextItems")] {
        let review = reviewed_overlay(key("$")?, key("$")?, "$", "Configuration", &[])?;
        let root = fixture(&model(list_target, detail_target, false), &review)?;

        // When: semantic overlay checking succeeds and compilation unifies canonical views.
        let result = compile(&root, None)?;

        // Then: the established capability variant and diagnostic reject the schema merge.
        let error = match result {
            Err(error) => error,
            Ok(_) => return Err("incompatible scalar views compiled successfully".into()),
        };
        assert!(matches!(&error, runtime_catalog::RuntimeCatalogCompileError::Capability));
        assert_eq!(error.to_string(), "runtime capability metadata is invalid or unsupported");
        root.close()?;
    }
    Ok(())
}

#[test]
fn given_unknown_enum_in_fixed_list_when_reviewed_then_precompile_gate_reports_exact_invalid_value() -> TestResult {
    // Given: the second value is a Smithy enum identifier, not its declared wire literal.
    for runtimes in [["provided.al2023", "NODE"], ["unknown", "nodejs22.x"]] {
        let review = reviewed_overlay(arn("FunctionArn")?, arn("FunctionArn")?, "FunctionArn", "Configuration", &runtimes)?;
        let root = fixture(&model("n#FunctionConfiguration", "n#FunctionConfiguration", false), &review)?;

        // When: only the public precompile semantic gate runs; no artifact is built.
        let violations = match root.check() {
            Err(overlay::CheckError::Violations(violations)) => violations,
            Err(other) => return Err(other.into()),
            Ok(_) => return Err("unknown enum fixed-list value passed precompile gate".into()),
        };

        // Then: no successful callable review escapes and the exact field is diagnosed.
        let diagnostics = violations.iter().map(|violation| (violation.service(), violation.operation(), violation.key(), violation.value(), violation.rule())).collect::<Vec<_>>();
        assert_eq!(diagnostics, vec![
            ("lambda", "GetFunctionConfiguration", "bind.from", "ListFunctions", overlay::ViolationRule::BindingSourceNotCallable),
            ("lambda", "ListFunctions", "fixed", "Runtimes", overlay::ViolationRule::InvalidValue),
        ]);
        root.close()?;
    }
    Ok(())
}
