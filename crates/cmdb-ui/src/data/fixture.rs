//! The embedded synthetic CMDB fixture: 24 rows across 3 accounts, 2 regions
//! and 4 kinds. Deliberately static: the scaffold proves the screen, not the
//! collector.

/// One inventory resource as shown by the table.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Row {
    pub account: &'static str,
    pub region: &'static str,
    pub kind: &'static str,
    pub id: &'static str,
    pub name: &'static str,
}

/// Accounts the filter rail offers, in display order.
pub const ACCOUNTS: [&str; 3] = ["prod", "staging", "sandbox"];
/// Regions the filter rail offers, in display order.
pub const REGIONS: [&str; 2] = ["us-east-1", "eu-west-1"];
/// Resource kinds the filter rail offers, in display order.
pub const KINDS: [&str; 4] = ["ec2", "s3", "rds", "lambda"];

#[rustfmt::skip]
pub(super) const ROWS: [Row; 24] = [
    row("prod", "us-east-1", "ec2", "i-0a1b2c3d4e5f60001", "web-api-01"),
    row("prod", "us-east-1", "ec2", "i-0a1b2c3d4e5f60002", "web-api-02"),
    row("prod", "us-east-1", "s3", "prod-artifact-store", "artifact-store"),
    row("prod", "us-east-1", "rds", "db-prod-orders", "orders-primary"),
    row("prod", "us-east-1", "lambda", "prod-image-resize", "image-resize"),
    row("prod", "eu-west-1", "ec2", "i-0a1b2c3d4e5f60006", "web-api-eu-01"),
    row("prod", "eu-west-1", "s3", "prod-eu-backups", "eu-backups"),
    row("prod", "eu-west-1", "rds", "db-prod-sessions", "sessions-primary"),
    row("prod", "eu-west-1", "lambda", "prod-invoice-pdf", "invoice-pdf"),
    row("staging", "us-east-1", "ec2", "i-0a1b2c3d4e5f6000a", "staging-api-01"),
    row("staging", "us-east-1", "s3", "staging-assets", "staging-assets"),
    row("staging", "us-east-1", "rds", "db-staging-orders", "orders-staging"),
    row("staging", "us-east-1", "lambda", "staging-thumbnail", "thumbnail"),
    row("staging", "eu-west-1", "ec2", "i-0a1b2c3d4e5f6000e", "staging-worker-01"),
    row("staging", "eu-west-1", "s3", "staging-eu-logs", "eu-logs"),
    row("staging", "eu-west-1", "lambda", "staging-webhook", "webhook"),
    row("sandbox", "us-east-1", "ec2", "i-0a1b2c3d4e5f60011", "sandbox-lab-01"),
    row("sandbox", "us-east-1", "s3", "sandbox-scratch", "scratch"),
    row("sandbox", "us-east-1", "rds", "db-sandbox-playground", "playground"),
    row("sandbox", "us-east-1", "lambda", "sandbox-probe", "probe"),
    row("sandbox", "eu-west-1", "ec2", "i-0a1b2c3d4e5f60015", "sandbox-lab-eu"),
    row("sandbox", "eu-west-1", "s3", "sandbox-eu-scratch", "eu-scratch"),
    row("sandbox", "eu-west-1", "rds", "db-sandbox-eu-playground", "eu-playground"),
    row("sandbox", "eu-west-1", "lambda", "sandbox-eu-probe", "eu-probe"),
];

const fn row(
    account: &'static str,
    region: &'static str,
    kind: &'static str,
    id: &'static str,
    name: &'static str,
) -> Row {
    Row {
        account,
        region,
        kind,
        id,
        name,
    }
}

/// One synthetic relationship between two resource ids.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct EdgeLink {
    pub from: &'static str,
    pub to: &'static str,
    pub label: &'static str,
}

#[rustfmt::skip]
pub(super) const EDGES: [EdgeLink; 16] = [
    link("i-0a1b2c3d4e5f60001", "db-prod-orders", "reads"),
    link("i-0a1b2c3d4e5f60002", "db-prod-orders", "reads"),
    link("i-0a1b2c3d4e5f60001", "prod-artifact-store", "assets"),
    link("prod-image-resize", "prod-artifact-store", "writes"),
    link("prod-invoice-pdf", "prod-eu-backups", "writes"),
    link("i-0a1b2c3d4e5f60006", "db-prod-sessions", "sessions"),
    link("i-0a1b2c3d4e5f60006", "prod-eu-backups", "reads"),
    link("i-0a1b2c3d4e5f6000a", "db-staging-orders", "reads"),
    link("i-0a1b2c3d4e5f6000a", "staging-assets", "assets"),
    link("staging-thumbnail", "staging-assets", "writes"),
    link("staging-webhook", "staging-eu-logs", "writes"),
    link("i-0a1b2c3d4e5f6000e", "db-staging-orders", "reads"),
    link("i-0a1b2c3d4e5f60011", "db-sandbox-playground", "reads"),
    link("sandbox-probe", "sandbox-scratch", "writes"),
    link("sandbox-eu-probe", "sandbox-eu-scratch", "writes"),
    link("i-0a1b2c3d4e5f60015", "db-sandbox-eu-playground", "reads"),
];

const fn link(from: &'static str, to: &'static str, label: &'static str) -> EdgeLink {
    EdgeLink { from, to, label }
}

impl Row {
    /// Synthetic facet tags, one per kind: the detail rail shows them.
    pub fn tags(&self) -> &'static str {
        match self.kind {
            "ec2" => "compute · linux · asg",
            "s3" => "object-store · versioned",
            "rds" => "database · postgres",
            "lambda" => "serverless · nodejs",
            _ => "unclassified",
        }
    }

    /// Synthetic shape of the resource, one per kind.
    pub fn size(&self) -> &'static str {
        match self.kind {
            "ec2" => "t3.medium · 2 vCPU · 4 GiB",
            "s3" => "128 GiB · standard",
            "rds" => "db.r6g.large · 16 GiB",
            "lambda" => "512 MiB · 15 min",
            _ => "unknown",
        }
    }
}
