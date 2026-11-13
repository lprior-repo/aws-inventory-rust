//! Organization account inventory and bounded root/OU membership traversal.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use aws_sdk_organizations::types::{Account, Child, ChildType, OrganizationalUnit, Root};
use aws_smithy_types::date_time::Format;
use futures_util::{StreamExt, future, stream};

use super::pages::{self, Identified, Listing, Page};
use super::{
    DiscoveryClient, DiscoveryCoverage, MAX_DISCOVERED_ACCOUNTS, MAX_DISCOVERED_UNITS,
    OrganizationAccount, OrganizationDiscovery, OrganizationMembership, OrganizationNode,
    bounded_text, data_error, ensure_running, optional_text, validate_account_id,
};
use crate::AwsScanError;
use crate::common::{required, sdk_error};

const MAX_ROOTS: usize = 32;
const MAX_PARENT_VISITS: usize = 10_032;
const MAX_DEPTH: u32 = 32;

impl Identified for OrganizationAccount {
    fn identifier(&self) -> &str {
        &self.id
    }
}
impl Identified for OrganizationNode {
    fn identifier(&self) -> &str {
        &self.id
    }
}

#[tracing::instrument(skip_all)]
pub(super) async fn collect(
    client: &DiscoveryClient<'_>,
    hierarchy: bool,
) -> OrganizationDiscovery {
    let sdk = aws_sdk_organizations::Client::new(client.config);
    let accounts = list_accounts(client, &sdk).await;
    let topology = if hierarchy {
        Some(collect_hierarchy(client, &sdk).await)
    } else {
        None
    };
    let (roots, organizational_units, memberships, hierarchy_coverage) = match topology {
        Some(value) => (
            value.roots,
            value.units,
            value.memberships,
            Some(value.coverage),
        ),
        None => (Vec::new(), Vec::new(), Vec::new(), None),
    };
    OrganizationDiscovery {
        accounts: accounts.items,
        account_coverage: accounts.coverage,
        hierarchy_requested: hierarchy,
        roots,
        organizational_units,
        memberships,
        hierarchy_coverage,
    }
}

#[tracing::instrument(skip_all)]
async fn list_accounts(
    client: &DiscoveryClient<'_>,
    sdk: &aws_sdk_organizations::Client,
) -> Listing<OrganizationAccount> {
    pages::collect(
        client,
        "organizations:ListAccounts",
        None,
        MAX_DISCOVERED_ACCOUNTS,
        |token| async move {
            let response = client
                .execute("organizations", true, || async {
                    sdk.list_accounts()
                        .set_next_token(token.clone())
                        .max_results(client.page_size.clamp(1, 20))
                        .send()
                        .await
                        .map_err(sdk_error)
                })
                .await?;
            Page::map(
                response.accounts.unwrap_or_default(),
                response.next_token,
                map_account,
            )
        },
    )
    .await
}

#[tracing::instrument(skip_all)]
async fn list_roots(
    client: &DiscoveryClient<'_>,
    sdk: &aws_sdk_organizations::Client,
) -> Listing<OrganizationNode> {
    pages::collect(
        client,
        "organizations:ListRoots",
        None,
        MAX_ROOTS,
        |token| async move {
            let response = client
                .execute("organizations", true, || async {
                    sdk.list_roots()
                        .set_next_token(token.clone())
                        .max_results(client.page_size.clamp(1, 20))
                        .send()
                        .await
                        .map_err(sdk_error)
                })
                .await?;
            Page::map(
                response.roots.unwrap_or_default(),
                response.next_token,
                map_root,
            )
        },
    )
    .await
}

#[tracing::instrument(skip_all, fields(parent = %parent.id))]
async fn list_units(
    client: &DiscoveryClient<'_>,
    sdk: &aws_sdk_organizations::Client,
    parent: &OrganizationNode,
    remaining: usize,
) -> Listing<OrganizationNode> {
    pages::collect(
        client,
        "organizations:ListOrganizationalUnitsForParent",
        Some(&parent.id),
        remaining,
        |token| async move {
            let response = client
                .execute("organizations", true, || async {
                    sdk.list_organizational_units_for_parent()
                        .parent_id(&parent.id)
                        .set_next_token(token.clone())
                        .max_results(client.page_size.clamp(1, 20))
                        .send()
                        .await
                        .map_err(sdk_error)
                })
                .await?;
            Page::map(
                response.organizational_units.unwrap_or_default(),
                response.next_token,
                |unit| map_unit(unit, &parent.id, parent.depth.saturating_add(1)),
            )
        },
    )
    .await
}

#[tracing::instrument(skip_all, fields(parent))]
async fn list_members(
    client: &DiscoveryClient<'_>,
    sdk: &aws_sdk_organizations::Client,
    parent: &str,
    remaining: usize,
) -> Listing<String> {
    pages::collect(
        client,
        "organizations:ListChildren",
        Some(parent),
        remaining,
        |token| async move {
            let response = client
                .execute("organizations", true, || async {
                    sdk.list_children()
                        .parent_id(parent)
                        .child_type(ChildType::Account)
                        .set_next_token(token.clone())
                        .max_results(client.page_size.clamp(1, 20))
                        .send()
                        .await
                        .map_err(sdk_error)
                })
                .await?;
            Page::map(
                response.children.unwrap_or_default(),
                response.next_token,
                map_member,
            )
        },
    )
    .await
}

fn map_account(account: Account) -> Result<OrganizationAccount, AwsScanError> {
    let id = required(account.id(), "organization account ID")?;
    validate_account_id(&id).map_err(|_| data_error("invalid organization account ID"))?;
    let state = optional_text(
        account.state().map(|state| state.as_str()),
        "account state",
        64,
    )?;
    let joined_at = account
        .joined_timestamp()
        .map(|time| {
            time.fmt(Format::DateTime)
                .map_err(|_| data_error("invalid organization join timestamp"))
        })
        .transpose()?;
    Ok(OrganizationAccount {
        eligible_for_scan: state.as_deref() == Some("ACTIVE"),
        id,
        arn: optional_text(account.arn(), "organization account ARN", 2_048)?,
        name: optional_text(account.name(), "organization account name", 512)?,
        state,
        joined_method: optional_text(
            account.joined_method().map(|method| method.as_str()),
            "join method",
            64,
        )?,
        joined_at,
    })
}

fn map_root(root: Root) -> Result<OrganizationNode, AwsScanError> {
    let id = required(root.id(), "organization root ID")?;
    validate_node_id(&id, "r-")?;
    Ok(OrganizationNode {
        id,
        arn: optional_text(root.arn(), "root ARN", 2_048)?,
        name: optional_text(root.name(), "root name", 512)?,
        parent_id: None,
        depth: 0,
    })
}

fn map_unit(
    unit: OrganizationalUnit,
    parent: &str,
    depth: u32,
) -> Result<OrganizationNode, AwsScanError> {
    let id = required(unit.id(), "organizational unit ID")?;
    validate_node_id(&id, "ou-")?;
    Ok(OrganizationNode {
        id,
        arn: optional_text(unit.arn(), "OU ARN", 2_048)?,
        name: optional_text(unit.name(), "OU name", 512)?,
        parent_id: Some(parent.to_owned()),
        depth,
    })
}

fn map_member(child: Child) -> Result<String, AwsScanError> {
    if child.r#type().is_none_or(|kind| kind.as_str() != "ACCOUNT") {
        return Err(data_error("ListChildren returned a non-account child"));
    }
    let id = required(child.id(), "organization member ID")?;
    validate_account_id(&id).map_err(|_| data_error("invalid organization member ID"))?;
    Ok(id)
}

fn validate_node_id(id: &str, prefix: &str) -> Result<(), AwsScanError> {
    bounded_text(id, "organization node ID", 100)?;
    let segment = |value: &str, minimum: usize| {
        (minimum..=32).contains(&value.len())
            && value
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
    };
    let valid = match (prefix, id.strip_prefix(prefix)) {
        ("r-", Some(root)) => segment(root, 4),
        ("ou-", Some(unit)) => unit
            .split_once('-')
            .is_some_and(|(root, suffix)| segment(root, 4) && segment(suffix, 8)),
        _ => false,
    };
    if !valid {
        return Err(data_error("invalid organization node ID"));
    }
    Ok(())
}

struct Hierarchy {
    roots: Vec<OrganizationNode>,
    units: Vec<OrganizationNode>,
    memberships: Vec<OrganizationMembership>,
    coverage: DiscoveryCoverage,
}

struct HierarchyCursor {
    result: Hierarchy,
    pending: VecDeque<OrganizationNode>,
    known_nodes: BTreeSet<String>,
    account_parents: BTreeMap<String, String>,
    visited: usize,
}

impl HierarchyCursor {
    fn new(roots: Listing<OrganizationNode>) -> Self {
        let pending = roots.items.iter().cloned().collect();
        let known_nodes = roots.items.iter().map(|node| node.id.clone()).collect();
        let mut result = Hierarchy {
            roots: roots.items,
            units: Vec::new(),
            memberships: Vec::new(),
            coverage: roots.coverage,
        };
        if result.roots.is_empty() && result.coverage.complete {
            result.coverage.fail(
                "organizations:ListRoots",
                None,
                &data_error("organization hierarchy returned no roots"),
            );
        }
        Self {
            result,
            pending,
            known_nodes,
            account_parents: BTreeMap::new(),
            visited: 0,
        }
    }

    fn accept(
        &mut self,
        parent: &OrganizationNode,
        units: Listing<OrganizationNode>,
        members: Listing<String>,
    ) {
        self.result.coverage.merge(units.coverage);
        self.result.coverage.merge(members.coverage);
        for unit in units.items {
            self.accept_unit(parent, unit);
        }
        for account in members.items {
            self.accept_member(parent, account);
        }
        self.visited = self.visited.saturating_add(1);
    }

    fn accept_unit(&mut self, parent: &OrganizationNode, unit: OrganizationNode) {
        let operation = "organizations:ListOrganizationalUnitsForParent";
        let failure = if self.known_nodes.contains(&unit.id) {
            Some("organizational unit repeated or had multiple parents")
        } else if unit.depth > MAX_DEPTH {
            Some("organization hierarchy exceeds depth 32")
        } else if self.result.units.len() >= MAX_DISCOVERED_UNITS {
            Some("organization hierarchy exceeds unit limit")
        } else {
            None
        };
        if let Some(message) = failure {
            self.result
                .coverage
                .fail(operation, Some(&parent.id), &data_error(message));
            return;
        }
        self.known_nodes.insert(unit.id.clone());
        self.pending.push_back(unit.clone());
        self.result.units.push(unit);
    }

    fn accept_member(&mut self, parent: &OrganizationNode, account: String) {
        if self.account_parents.contains_key(&account)
            || self.result.memberships.len() >= MAX_DISCOVERED_ACCOUNTS
        {
            self.result.coverage.fail(
                "organizations:ListChildren",
                Some(&parent.id),
                &data_error("duplicate account membership or hierarchy account limit"),
            );
            return;
        }
        self.account_parents
            .insert(account.clone(), parent.id.clone());
        self.result.memberships.push(OrganizationMembership {
            account_id: account,
            parent_id: parent.id.clone(),
        });
    }
}

#[tracing::instrument(skip_all)]
async fn collect_hierarchy(
    client: &DiscoveryClient<'_>,
    sdk: &aws_sdk_organizations::Client,
) -> Hierarchy {
    let roots = list_roots(client, sdk).await;
    let traversal = stream::unfold(Some(HierarchyCursor::new(roots)), |state| async move {
        let mut cursor = state?;
        if let Err(error) = ensure_running(client.executor) {
            cursor
                .result
                .coverage
                .fail("organizations:Hierarchy", None, &error);
            return Some((Some(cursor.result), None));
        }
        if cursor.pending.is_empty() {
            return Some((Some(cursor.result), None));
        }
        if cursor.visited >= MAX_PARENT_VISITS {
            cursor.result.coverage.fail(
                "organizations:Hierarchy",
                None,
                &data_error("hierarchy parent visit bound exceeded"),
            );
            return Some((Some(cursor.result), None));
        }
        let parent = cursor.pending.pop_front()?;
        let (units, members) = tokio::join!(
            list_units(
                client,
                sdk,
                &parent,
                MAX_DISCOVERED_UNITS.saturating_sub(cursor.result.units.len())
            ),
            list_members(
                client,
                sdk,
                &parent.id,
                MAX_DISCOVERED_ACCOUNTS.saturating_sub(cursor.result.memberships.len())
            )
        );
        cursor.accept(&parent, units, members);
        Some((None, Some(cursor)))
    });
    traversal
        .filter_map(future::ready)
        .fold(None, |_, result| future::ready(Some(result)))
        .await
        .unwrap_or_else(|| {
            let mut coverage = DiscoveryCoverage::success();
            coverage.fail(
                "organizations:Hierarchy",
                None,
                &data_error("hierarchy has no terminal outcome"),
            );
            Hierarchy {
                roots: Vec::new(),
                units: Vec::new(),
                memberships: Vec::new(),
                coverage,
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use aws_sdk_organizations::types::AccountState;

    #[test]
    fn account_state_controls_eligibility_and_missing_state_fails_closed()
    -> Result<(), AwsScanError> {
        let active = Account::builder()
            .id("123456789012")
            .name("Production")
            .state(AccountState::Active)
            .build();
        let suspended = Account::builder()
            .id("223456789012")
            .state(AccountState::Suspended)
            .build();
        let unknown = Account::builder().id("323456789012").build();
        assert!(map_account(active)?.eligible_for_scan);
        assert!(!map_account(suspended)?.eligible_for_scan);
        assert!(!map_account(unknown)?.eligible_for_scan);
        Ok(())
    }

    #[test]
    fn malformed_account_id_is_rejected_and_email_is_not_retained() -> Result<(), AwsScanError> {
        assert!(map_account(Account::builder().id("bad").build()).is_err());
        let account = map_account(
            Account::builder()
                .id("123456789012")
                .email("sensitive@example.test")
                .state(AccountState::Active)
                .build(),
        )?;
        let json = serde_json::to_value(account).map_err(|error| data_error(&error.to_string()))?;
        assert!(json.get("email").is_none());
        Ok(())
    }

    #[test]
    fn root_unit_and_membership_id_types_are_checked() -> Result<(), AwsScanError> {
        assert!(map_root(Root::builder().id("ou-test-unit").build()).is_err());
        assert!(map_root(Root::builder().id("r-").build()).is_err());
        assert!(map_root(Root::builder().id("r-a").build()).is_err());
        assert!(map_root(Root::builder().id("r-root-extra").build()).is_err());
        assert!(validate_node_id("ou-root-a", "ou-").is_err());
        assert!(validate_node_id("ou-root-12345678-extra", "ou-").is_err());
        validate_node_id("ou-root-12345678", "ou-")?;
        assert!(
            map_unit(
                OrganizationalUnit::builder().id("r-root").build(),
                "r-root",
                1
            )
            .is_err()
        );
        assert!(
            map_member(
                Child::builder()
                    .id("123456789012")
                    .r#type(ChildType::OrganizationalUnit)
                    .build()
            )
            .is_err()
        );
        assert_eq!(
            map_member(
                Child::builder()
                    .id("123456789012")
                    .r#type(ChildType::Account)
                    .build()
            )?,
            "123456789012"
        );
        Ok(())
    }

    fn root() -> OrganizationNode {
        OrganizationNode {
            id: "r-root".into(),
            arn: None,
            name: Some("Root".into()),
            parent_id: None,
            depth: 0,
        }
    }

    #[test]
    fn hierarchy_rejects_cycles_multiple_parents_and_duplicate_memberships() {
        let mut cursor = HierarchyCursor::new(Listing {
            items: vec![root()],
            coverage: DiscoveryCoverage::success(),
        });
        let parent = root();
        let unit = OrganizationNode {
            id: "ou-root-production".into(),
            arn: None,
            name: None,
            parent_id: Some(parent.id.clone()),
            depth: 1,
        };
        cursor.accept_unit(&parent, unit.clone());
        cursor.accept_unit(&parent, unit);
        cursor.accept_member(&parent, "123456789012".into());
        cursor.accept_member(&parent, "123456789012".into());
        assert_eq!(cursor.result.units.len(), 1);
        assert_eq!(cursor.result.memberships.len(), 1);
        assert_eq!(cursor.result.coverage.failures.len(), 2);
        assert!(!cursor.result.coverage.complete);
    }

    #[test]
    fn empty_roots_and_depth_limit_are_explicit_partial_coverage() {
        let empty = HierarchyCursor::new(Listing {
            items: vec![],
            coverage: DiscoveryCoverage::success(),
        });
        assert!(!empty.result.coverage.complete);
        let mut cursor = HierarchyCursor::new(Listing {
            items: vec![root()],
            coverage: DiscoveryCoverage::success(),
        });
        cursor.accept_unit(
            &root(),
            OrganizationNode {
                id: "ou-root-deep".into(),
                arn: None,
                name: None,
                parent_id: Some("r-root".into()),
                depth: MAX_DEPTH.saturating_add(1),
            },
        );
        assert!(cursor.result.units.is_empty());
        assert!(!cursor.result.coverage.complete);
    }
}
