use super::*;
// A small Smithy REST-JSON service, not a replacement collector: the production
// compiler, request encoder, signer, TLS transport, response decoder and SQLite
// writer all run unchanged. The peer serves fixed pages selected by wire cursors.
const MODEL: &str = r#"{"smithy":"2.0","shapes":{
"n.pagination#Service":{"type":"service","version":"1","operations":[{"target":"n.pagination#GetCallerIdentity"},{"target":"n.pagination#DescribeRegions"},{"target":"n.pagination#ListItems"}],"traits":{"aws.api#service":{"sdkId":"Pagination","endpointPrefix":"pagination","arnNamespace":"pagination"},"aws.auth#sigv4":{"name":"pagination"},"aws.protocols#restJson1":{}}},
"n.pagination#GetCallerIdentity":{"type":"operation","output":{"target":"n.pagination#IdentityOutput"},"traits":{"smithy.api#http":{"method":"GET","uri":"/identity"}}},
"n.pagination#IdentityOutput":{"type":"structure","members":{"Account":{"target":"smithy.api#String"}}},
"n.pagination#DescribeRegions":{"type":"operation","output":{"target":"n.pagination#RegionsOutput"},"traits":{"smithy.api#http":{"method":"GET","uri":"/regions"}}},
"n.pagination#RegionsOutput":{"type":"structure","members":{"Regions":{"target":"n.pagination#RegionList"}}},
"n.pagination#RegionList":{"type":"list","member":{"target":"n.pagination#Region"}},
"n.pagination#Region":{"type":"structure","members":{"RegionName":{"target":"smithy.api#String"}}},
"n.pagination#ListItems":{"type":"operation","input":{"target":"n.pagination#ListInput"},"output":{"target":"n.pagination#ListOutput"},"traits":{"smithy.api#http":{"method":"GET","uri":"/items"}}},
"n.pagination#ListInput":{"type":"structure","members":{"StartName":{"target":"smithy.api#String","traits":{"smithy.api#httpQuery":"name"}},"StartType":{"target":"smithy.api#String","traits":{"smithy.api#httpQuery":"type"}},"StartIdentifier":{"target":"smithy.api#String","traits":{"smithy.api#httpQuery":"identifier"}}}},
"n.pagination#ListOutput":{"type":"structure","members":{"Items":{"target":"n.pagination#Items"},"NextName":{"target":"smithy.api#String"},"NextType":{"target":"smithy.api#String"},"NextIdentifier":{"target":"smithy.api#String"},"IsTruncated":{"target":"smithy.api#Boolean"}}},
"n.pagination#Items":{"type":"list","member":{"target":"smithy.api#String"}}
}}"#;

fn fixture_policy() -> Result<overlay::ServiceOverlay> {
    let plan = |scope, items, identity| -> Result<overlay::OperationOverlay> {
        let mut plan = overlay::OperationOverlay::default();
        plan.class = Some(overlay::OverlayClass::Enumerate);
        plan.scope = Some(scope);
        plan.items = Some(overlay::ItemSelector::parse(items)?);
        plan.identity = Some(overlay::Identity::Key(vec![overlay::MemberPath::selected(
            identity,
        )?]));
        plan.paginator = Some(overlay::Paginator::None);
        Ok(plan)
    };
    let mut items = plan(overlay::Scope::Region, "Items", "$")?;
    let pair = |input, output| -> Result<overlay::TokenPair> {
        Ok(overlay::TokenPair::new(
            overlay::MemberPath::new(input)?,
            overlay::MemberPath::new(output)?,
        ))
    };
    let mut optional = pair("StartIdentifier", "NextIdentifier")?;
    optional.optional = true;
    items.paginator = Some(overlay::Paginator::Tokens {
        pairs: vec![
            pair("StartName", "NextName")?,
            pair("StartType", "NextType")?,
            optional,
        ],
        truncated: Some(overlay::MemberPath::new("IsTruncated")?),
    });
    Ok(overlay::ServiceOverlay::new(
        "pagination",
        vec![
            (
                "GetCallerIdentity",
                plan(overlay::Scope::Account, "$", "Account")?,
            ),
            (
                "DescribeRegions",
                plan(overlay::Scope::Region, "Regions", "RegionName")?,
            ),
            ("ListItems", items),
        ],
    )?)
}

fn assert_fixture_metadata(bytes: &[u8]) -> Result<()> {
    let catalog = RuntimeCatalog::from_bytes(bytes)?;
    assert_eq!(catalog.plans().len(), 3);
    let account = catalog
        .lookup("pagination", "GetCallerIdentity")
        .context("account bootstrap absent")?;
    assert!(matches!(account.purpose(),
        PlanPurpose::AccountIdentity { account_path } if account_path.as_ref() == [FieldId::new(1)?]));
    let regions = catalog
        .lookup("pagination", "DescribeRegions")
        .context("region bootstrap absent")?;
    assert!(matches!(regions.purpose(),
        PlanPurpose::RegionDiscovery { regions_path, region_name_path, opt_in_status_path }
        if regions_path.as_ref() == [FieldId::new(8)?]
            && region_name_path.as_ref() == [FieldId::new(7)?] && opt_in_status_path.is_none()));
    let items = catalog
        .lookup("pagination", "ListItems")
        .context("scalar plan absent")?;
    assert_eq!(items.purpose(), &PlanPurpose::Enumeration);
    let pagination = items.pagination();
    assert_eq!(pagination.tokens.len(), 3);
    pagination
        .tokens
        .iter()
        .zip([(10, 5, false), (11, 6, false), (9, 4, true)])
        .try_for_each(|(token, (input, output, optional))| -> Result<()> {
            assert_eq!(token.request_path.as_ref(), [FieldId::new(input)?]);
            assert_eq!(token.response_path.as_ref(), [FieldId::new(output)?]);
            assert_eq!(token.optional, optional);
            Ok(())
        })?;
    assert_eq!(
        pagination.truncated_path.as_deref(),
        Some([FieldId::new(2)?].as_slice())
    );
    Ok(())
}

const KINDS: kind_map::MappingSource<'static> = kind_map::MappingSource {
    version: 1,
    reviewed: "2026-11-15",
    reviewer: "Pagination fixture",
    default_unmapped_reason: None,
    cfn: &[],
    relations: &[],
    parent_relations: &[],
    compound_relations: &[],
    additional_emissions: &[],
    relation_targets: &[],
    api_only: &[
        kind_map::ApiOnlyMapping {
            kind: "AWS::ApiOnly::PaginationAccount",
            reason: "Bootstrap account",
        },
        kind_map::ApiOnlyMapping {
            kind: "AWS::ApiOnly::PaginationRegion",
            reason: "Bootstrap region",
        },
        kind_map::ApiOnlyMapping {
            kind: "AWS::ApiOnly::PaginationScalar",
            reason: "Scalar items",
        },
    ],
    operations: &[
        kind_map::OperationMapping {
            service: "pagination",
            operation: "GetCallerIdentity",
            kind: "AWS::ApiOnly::PaginationAccount",
        },
        kind_map::OperationMapping {
            service: "pagination",
            operation: "DescribeRegions",
            kind: "AWS::ApiOnly::PaginationRegion",
        },
        kind_map::OperationMapping {
            service: "pagination",
            operation: "ListItems",
            kind: "AWS::ApiOnly::PaginationScalar",
        },
    ],
    bootstraps: &[
        kind_map::BootstrapMapping::AccountIdentity {
            service: "pagination",
            operation: "GetCallerIdentity",
            account_path: "Account",
        },
        kind_map::BootstrapMapping::RegionDiscovery {
            service: "pagination",
            operation: "DescribeRegions",
            regions_path: "Regions",
            region_name_path: "RegionName",
            opt_in_status_path: None,
        },
    ],
};
pub(super) struct Fixture {
    pub(super) root: tempfile::TempDir,
    catalog: PathBuf,
    pub(super) database: PathBuf,
    credentials: PathBuf,
    config: PathBuf,
}

impl Fixture {
    pub(super) fn create() -> Result<Self> {
        let root = tempfile::tempdir()?;
        let source = root.path().join("source");
        let models = source.join("models/pagination/service/2026-01-01");
        std::fs::create_dir_all(&models)?;
        std::fs::write(models.join("model.json"), MODEL)?;
        let checked = overlay::check_policies(
            &source,
            &[fixture_policy()?],
            &catalog_tool::classes::DenyList::builtin(),
        )?;
        let registry = smithy::Registry::parse(
            "Account\nIsTruncated\nItems\nNextIdentifier\nNextName\nNextType\nRegionName\nRegions\nStartIdentifier\nStartName\nStartType\n",
        )?;
        let base = smithy::build_catalog(&source, &registry, "pagination-consumer-fixture")?;
        let kinds = KINDS.compile(&names::Exceptions::none())?;
        let endpoints = BTreeMap::from([("pagination".to_owned(), verified_endpoint()?)]);
        let catalog = root.path().join("fixture.catalog");
        let bytes = runtime_catalog::augment(&base, &checked, &kinds, &endpoints)?;
        assert_fixture_metadata(&bytes)?;
        std::fs::write(&catalog, bytes)?;
        let credentials = root.path().join("credentials");
        let config = root.path().join("config");
        std::fs::write(
            &credentials,
            "[pagination]\naws_access_key_id=AKIAPAGINATIONCONTRACT\naws_secret_access_key=local-only-pagination-secret\n",
        )?;
        std::fs::write(&config, "[profile pagination]\nregion=us-east-1\n")?;
        let database = root.path().join("inventory.sqlite");
        Ok(Self {
            root,
            catalog,
            database,
            credentials,
            config,
        })
    }

    pub(super) fn spawn(&self, peer: &Peer, resume: bool) -> Result<Child> {
        let mut command = Command::new(env!("CARGO_BIN_EXE_aws-inventory"));
        command
            .env_clear()
            .env("HOME", self.root.path())
            .env("AWS_CONFIG_FILE", &self.config)
            .env("AWS_SHARED_CREDENTIALS_FILE", &self.credentials)
            .env("AWS_EC2_METADATA_DISABLED", "true")
            .args(["sweep", "--database"])
            .arg(&self.database)
            .arg("--catalog")
            .arg(&self.catalog)
            .args(["--profile", "pagination", "--region", REGION, "--endpoint"])
            .arg(&peer.endpoint)
            .arg("--ca-bundle")
            .arg(&peer.ca)
            .args(["--max-pages", "20", "--concurrency", "1"])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        if resume {
            command.arg("--resume");
        }
        command.spawn().context("launching public sweep binary")
    }

    pub(super) async fn collect(&self, peer: &Peer, resume: bool) -> Result<Output> {
        let mut child = self.spawn(peer, resume)?;
        match tokio::time::timeout(DEADLINE, child.wait()).await {
            Ok(status) => {
                status?;
            }
            Err(deadline) => {
                child
                    .kill()
                    .await
                    .context("killing and reaping timed-out sweep")?;
                return Err(deadline.into());
            }
        }
        Ok(tokio::time::timeout(DEADLINE, child.wait_with_output()).await??)
    }
}

fn verified_endpoint() -> Result<endpoints::VerifiedEndpoint> {
    let properties =
        json!({"authSchemes":[{"name":"sigv4","signingName":"pagination","signingRegion":REGION}]});
    let document = json!({"version":"1.0","parameters":{"Region":{"type":"string"}},"rules":[{"type":"endpoint","conditions":[],"endpoint":{"url":"https://pagination.invalid","properties":properties}}]});
    let rules = endpoints::RuleSet::parse(Path::new("pagination-endpoint.json"), &document)?;
    let cases = serde_json::from_value::<Vec<endpoints::Case>>(
        json!([{"params":{"Region":REGION},"expect":{"endpoint":{"url":"https://pagination.invalid","properties":properties}}}]),
    )?;
    rules
        .run_cases(&cases)?
        .verified_endpoint(Some("pagination"))
        .map_err(|error| anyhow::anyhow!("fixture endpoint derivation failed: {error:?}"))
}
