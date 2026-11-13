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

#[cfg(test)]
mod tests;

mod listing;
use listing::{list_accounts, list_members, list_roots, list_units};
mod mapping;
use mapping::{map_account, map_member, map_root, map_unit};
mod hierarchy;
use hierarchy::collect_hierarchy;

#[cfg(test)]
use mapping::validate_node_id;
