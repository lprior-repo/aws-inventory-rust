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

#[path = "canonical_mapping_contract/scenarios.rs"]
mod scenarios;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

const KIND: &str = "AWS::Lambda::Function";
const TABLE: &str = "aws_lambda_function";
const FIELDS: &str =
    "CodeSize\nConfiguration\nFunctionArn\nFunctionName\nFunctions\nRuntime\nRuntimes\n";

fn model(list_target: &str, detail_target: &str, root_detail: bool) -> serde_json::Value {
    let detail_output = if root_detail {
        detail_target
    } else {
        "n#GetOutput"
    };
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
    Ok(overlay::Identity::Key(vec![overlay::MemberPath::selected(
        path,
    )?]))
}

fn reviewed_overlay(
    list_identity: overlay::Identity,
    detail_identity: overlay::Identity,
    binding: &str,
    detail_items: &str,
    fixed: &[&str],
) -> TestResult<overlay::ServiceOverlay> {
    let mut list = overlay::OperationOverlay::default();
    list.class = Some(overlay::OverlayClass::Enumerate);
    list.scope = Some(overlay::Scope::Region);
    list.items = Some(overlay::ItemSelector::parse("Functions")?);
    list.paginator = Some(overlay::Paginator::None);
    list.identity = Some(list_identity);
    if !fixed.is_empty() {
        list.fixed.insert(
            overlay::MemberPath::new("Runtimes")?,
            overlay::FixedValue::Strings(fixed.iter().map(|value| (*value).to_owned()).collect()),
        );
    }
    let mut detail = overlay::OperationOverlay::default();
    detail.class = Some(overlay::OverlayClass::Expand);
    detail.scope = Some(overlay::Scope::Region);
    detail.items = Some(overlay::ItemSelector::parse(detail_items)?);
    detail.paginator = Some(overlay::Paginator::None);
    detail.identity = Some(detail_identity);
    detail.bind.insert(
        overlay::MemberPath::new("FunctionName")?,
        overlay::Binding::From {
            operation: overlay::OperationName::new("ListFunctions")?,
            path: overlay::MemberPath::selected(binding)?,
        },
    );
    Ok(overlay::ServiceOverlay::new(
        "lambda",
        vec![
            ("ListFunctions", list),
            ("GetFunctionConfiguration", detail),
        ],
    )?)
}

struct Fixture {
    directory: tempfile::TempDir,
    policy: overlay::ServiceOverlay,
}

impl Fixture {
    fn path(&self) -> &Path {
        self.directory.path()
    }
    fn close(self) -> std::io::Result<()> {
        self.directory.close()
    }
    fn check(&self) -> Result<overlay::CheckedOverlays, Box<overlay::CheckError>> {
        overlay::check_policies(
            &self.path().join("source"),
            std::slice::from_ref(&self.policy),
            &catalog_tool::classes::DenyList::parse("")
                .map_err(|error| Box::new(overlay::CheckError::from(error)))?,
        )
        .map_err(Box::new)
    }
}

fn fixture(model: &serde_json::Value, review: &overlay::ServiceOverlay) -> TestResult<Fixture> {
    let directory = tempfile::tempdir()?;
    let model_dir = directory
        .path()
        .join("source/models/lambda/service/2015-03-31");
    fs::create_dir_all(&model_dir)?;
    fs::write(model_dir.join("lambda.json"), serde_json::to_vec(model)?)?;
    Ok(Fixture {
        directory,
        policy: review.clone(),
    })
}

fn verified_endpoint() -> TestResult<endpoints::VerifiedEndpoint> {
    let properties = json!({"authSchemes":[{"name":"sigv4","signingName":"lambda","signingRegion":"us-east-1"}]});
    let document = json!({"version":"1.0","parameters":{"Region":{"type":"string"}},"rules":[{"type":"endpoint","conditions":[],"endpoint":{"url":"https://lambda.example.test","properties":properties}}]});
    let rules = endpoints::RuleSet::parse(Path::new("lambda-fixture-endpoint.json"), &document)?;
    let cases: Vec<endpoints::Case> = serde_json::from_value(
        json!([{"params":{"Region":"us-east-1"},"expect":{"endpoint":{"url":"https://lambda.example.test","properties":properties}}}]),
    )?;
    rules
        .run_cases(&cases)?
        .verified_endpoint(Some("lambda"))
        .map_err(|error| format!("fixture endpoint evidence failed: {error:?}").into())
}

fn compile(
    root: &Fixture,
    display_path: Option<&str>,
) -> TestResult<Result<Vec<u8>, runtime_catalog::RuntimeCatalogCompileError>> {
    compile_mapping(root, display_path, false)
}

fn compile_mapping(
    root: &Fixture,
    display_path: Option<&str>,
    include_inactive: bool,
) -> TestResult<Result<Vec<u8>, runtime_catalog::RuntimeCatalogCompileError>> {
    let source = root.path().join("source");
    let checked = root.check()?;
    let fields = smithy::Registry::parse(FIELDS)?;
    let base = smithy::build_catalog(&source, &fields, "canonical-mapping-fixture")?;
    let mut cfn = vec![kind_map::CfnMapping::Mapped {
        cfn_type: KIND,
        kind: KIND,
        display_name_path: display_path,
    }];
    let mut operations = vec![
        kind_map::OperationMapping {
            service: "lambda",
            operation: "ListFunctions",
            kind: KIND,
        },
        kind_map::OperationMapping {
            service: "lambda",
            operation: "GetFunctionConfiguration",
            kind: KIND,
        },
    ];
    if include_inactive {
        cfn.push(kind_map::CfnMapping::Mapped {
            cfn_type: "AWS::EC2::VPC",
            kind: "AWS::EC2::VPC",
            display_name_path: Some("MissingName"),
        });
        operations.push(kind_map::OperationMapping {
            service: "ec2",
            operation: "DescribeVpcs",
            kind: "AWS::EC2::VPC",
        });
    }
    let kinds = kind_map::MappingSource {
        cfn: &cfn,
        operations: &operations,
        ..kind_map::MappingSource::default()
    }
    .compile(&names::Exceptions::none())?;
    let endpoints = BTreeMap::from([("lambda".to_owned(), verified_endpoint()?)]);
    Ok(runtime_catalog::augment(
        &base, &checked, &kinds, &endpoints,
    ))
}

async fn assert_canonical_sqlite(
    database: &Path,
    bytes: &[u8],
    catalog: &RuntimeCatalog<'_>,
) -> TestResult {
    let registry = SchemaRegistry::new(bytes, catalog.schemas().to_vec())?;
    let store = BinaryStore::create(database, registry).await?;
    let reader = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            SqliteConnectOptions::new()
                .filename(database)
                .read_only(true),
        )
        .await?;
    let kinds: Vec<(String, String)> =
        sqlx::query_as("SELECT kind,table_name FROM kinds ORDER BY kind")
            .fetch_all(&reader)
            .await?;
    assert_eq!(kinds, vec![(KIND.to_owned(), TABLE.to_owned())]);
    let operations: Vec<(String, String)> =
        sqlx::query_as("SELECT service,operation FROM catalog_operations ORDER BY operation")
            .fetch_all(&reader)
            .await?;
    assert_eq!(
        operations,
        vec![
            ("lambda".to_owned(), "GetFunctionConfiguration".to_owned()),
            ("lambda".to_owned(), "ListFunctions".to_owned())
        ]
    );
    let physical_columns: Vec<(String, String)> =
        sqlx::query_as("SELECT name,type FROM pragma_table_info(?1) ORDER BY cid")
            .bind(TABLE)
            .fetch_all(&reader)
            .await?;
    let expected_columns = [
        ("account", "TEXT"),
        ("region", "TEXT"),
        ("id", "TEXT"),
        ("content_hash", "BLOB"),
        ("first_seen", "INTEGER"),
        ("last_changed", "INTEGER"),
        ("record", "BLOB"),
        ("f_4", "TEXT"),
    ]
    .map(|(name, ty)| (name.to_owned(), ty.to_owned()))
    .to_vec();
    assert_eq!(physical_columns, expected_columns);
    reader.close().await;
    drop(store);
    Ok(())
}
