use super::*;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct OrganizationAccount {
    pub id: String,
    pub arn: Option<String>,
    pub name: Option<String>,

    pub state: Option<String>,
    pub eligible_for_scan: bool,
    pub joined_method: Option<String>,
    pub joined_at: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct OrganizationNode {
    pub id: String,
    pub arn: Option<String>,
    pub name: Option<String>,
    pub parent_id: Option<String>,
    pub depth: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct OrganizationMembership {
    pub account_id: String,
    pub parent_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct OrganizationDiscovery {
    pub accounts: Vec<OrganizationAccount>,
    pub account_coverage: DiscoveryCoverage,
    pub hierarchy_requested: bool,
    pub roots: Vec<OrganizationNode>,
    pub organizational_units: Vec<OrganizationNode>,
    pub memberships: Vec<OrganizationMembership>,
    pub hierarchy_coverage: Option<DiscoveryCoverage>,
}

impl OrganizationDiscovery {
    pub fn complete(&self) -> bool {
        self.account_coverage.complete
            && self
                .hierarchy_coverage
                .as_ref()
                .is_none_or(|coverage| coverage.complete)
    }

    pub fn eligible_accounts(&self) -> impl Iterator<Item = &OrganizationAccount> {
        self.accounts
            .iter()
            .filter(|account| account.eligible_for_scan)
    }
}
