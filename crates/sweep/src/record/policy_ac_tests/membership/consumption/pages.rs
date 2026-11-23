use super::*;
use inventory_core::catalog::runtime::CallablePlan;

pub(super) struct PageContext<'a, 'catalog> {
    pub(super) service: &'a str,
    pub(super) catalog: &'a RuntimeCatalog<'catalog>,
    pub(super) consumers: &'a ConsumerIndex<'a, 'catalog>,
    pub(super) registry: &'a catalog_tool::smithy::Registry,
    pub(super) parent: &'a CallablePlan<'catalog>,
    pub(super) account: &'a AccountId,
    pub(super) home: &'a RegionId,
    pub(super) region: &'a Region,
    pub(super) root_request: &'a Record,
    pub(super) owners: &'a BTreeMap<String, ResourceKey>,
    pub(super) parent_relation: &'a str,
    pub(super) cases: &'a [Case],
    pub(super) path: &'a std::path::Path,
    pub(super) sweep: &'a SweepId,
}

impl PageContext<'_, '_> {
    pub(super) async fn consume(
        self,
        mut store: BinaryStore,
        expected: &BTreeSet<(String, String)>,
    ) -> TestResult<(BinaryStore, BTreeSet<(String, String)>)> {
        let Self {
            service,
            catalog,
            consumers,
            registry,
            parent,
            account,
            home,
            region,
            root_request,
            owners,
            parent_relation,
            cases,
            path,
            sweep,
        } = self;
        let mut seen = BTreeSet::new();
        // When real child projection, durable work leases, cursor resumes and native serializers consume both pages.
        for at in 1..=expected.len() {
            let at = i64::try_from(at)?;
            let lease = store
                .claim(sweep, at, 10_000)
                .await?
                .ok_or("expected child page absent")?;
            let plan = catalog
                .lookup(service, lease.scope().operation().as_str())
                .ok_or("leased child absent")?;
            let case = cases
                .iter()
                .find(|case| case.operation == plan.operation_name())
                .ok_or("unreviewed child scheduled")?;
            let base = Record::from_bytes(lease.bindings().to_vec())?;
            let owner_value = text(&base, registry, case.parent_field)?;
            let owner = case
                .owners
                .iter()
                .find(|owner| owner.value == owner_value)
                .ok_or("child lost selected parent")?;
            assert_eq!(lease.scope().parent(), owners.get(&owner_value));
            let later = lease.token().is_some();
            let page = owner
                .pages
                .get(usize::from(later))
                .ok_or("page fixture absent")?;
            let cursor = Cursor::decode(lease.token())?;
            let fields = cursor.request_fields(plan.pagination())?;
            let actual_request = input::merge_input(&base, &fields)?;
            let wire = request::encode(plan, "https://fixture.example.test", &base, &fields)?;
            if service == "cognito-identity-provider" {
                let target = format!("AWSCognitoIdentityProviderService.{}", case.operation);
                assert_eq!(
                    wire.headers()
                        .get("x-amz-target")
                        .and_then(|value| value.to_str().ok()),
                    Some(target.as_str())
                );
            }
            page.wire.assert_matches(&wire)?;
            let response = decode(plan, &page.body)?;
            let ancestry = DiscoveryContext::decode(lease.context())?;
            let context = child::Context::new(plan, sweep, account, home, consumers);
            let (observations, children) = child::policy_dfgk_tests::project_children(
                catalog,
                &context,
                lease.scope(),
                &actual_request,
                &response,
                &ancestry,
            )?;
            assert_eq!(observations.len(), 1);
            assert_eq!(
                children.len(),
                0,
                "summary must not schedule content or secret details"
            );
            let observation = observations.first().ok_or("summary observation absent")?;
            assert_eq!(observation.key().kind.as_str(), case.kind);
            assert_eq!(observation.key().id.as_str(), page.id);
            assert!(
                seen.insert((case.kind.to_owned(), page.id.clone())),
                "duplicate or parent-colliding identity"
            );
            let actual_branch =
                crate::execution::routing::execution_branch(plan, lease.scope().branch())?;
            let mut selected = Vec::new();
            for row in record::item_records(&response, plan.items())? {
                if crate::row_policy::selected(
                    plan,
                    actual_branch,
                    &row,
                    &actual_request,
                    &response,
                )? {
                    selected.push(row);
                }
            }
            if case.requires_request_identity {
                let row = selected.first().ok_or("selected summary absent")?;
                assert!(
                    record::path_value(
                        row.root(),
                        &[registry
                            .id(case.parent_field)
                            .ok_or("request field absent")?]
                    )?
                    .is_none()
                );
                assert!(
                    record::resource_id_with_context(
                        row,
                        plan.identity(),
                        None,
                        &ancestry,
                        root_request
                    )
                    .is_err(),
                    "scoped identity accepted an unrelated root request"
                );
            }
            let unresolved = rows::references(
                catalog,
                plan,
                home,
                &selected,
                &observations,
                &actual_request,
                None,
            )?;
            let expected_reference_count = if matches!(
                case.operation,
                "ListConfiguredTableAssociations"
                    | "ListConfiguredAudienceModelAssociations"
                    | "ListIdNamespaceAssociations"
            ) {
                2
            } else {
                1
            };
            assert_eq!(
                unresolved.len(),
                expected_reference_count,
                "every reviewed endpoint must retain its join"
            );
            let parent_references = unresolved
                .iter()
                .filter(|reference| reference.target.kind() == parent.kind())
                .collect::<Vec<_>>();
            assert_eq!(parent_references.len(), 1);
            let reference = parent_references
                .first()
                .ok_or("reviewed parent reference absent")?;
            assert_eq!(&reference.from, observation.key());
            assert_eq!(
                reference.source,
                inventory_core::inventory::EdgeSource::ReviewedRule
            );
            assert_eq!(reference.target.kind(), parent.kind());
            assert_eq!(reference.target.value().as_str(), owner_value);
            assert_eq!(reference.target.account(), account);
            assert_eq!(reference.target.region(), region);
            assert_eq!(reference.relation.as_str(), parent_relation);
            let next = cursor.next(plan.pagination(), &response, &http::HeaderMap::new(), 10)?;
            assert_eq!(
                next.is_none(),
                later,
                "first-page cursor or terminal completion lost"
            );
            let done = next.is_none();
            let (next_token, continuation_digest) = next.map_or((None, None), |next| {
                (Some(next.cursor.into_bytes()), next.continuation_digest)
            });
            store
                .commit_page(PageCommit {
                    lease,
                    observations,
                    children,
                    unresolved,
                    edges: Vec::new(),
                    tags: Vec::new(),
                    next_token,
                    continuation_digest,
                    done,
                    observed_at_ms: at,
                })
                .await?;
            drop(store);
            store = BinaryStore::open(path).await?;
        }
        Ok((store, seen))
    }
}
