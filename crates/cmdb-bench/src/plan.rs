//! Per-account plan shared by the page writer and the edge writer.

use std::error::Error;

use inventory_core::{CollectionScope, Source};

use crate::layout;
use crate::options::Options;

pub struct AccountContext {
    pub account: usize,
    pub size: usize,
    pub account_id: String,
    pub region: &'static str,
    pub global: bool,
    pub payload_bytes: usize,
    pub scope: CollectionScope,
    pub vpc: Option<String>,
    pub security_group: Option<String>,
    pub edges: usize,
    pub resource_pages: usize,
    pub total_pages: usize,
}

pub fn account_context(
    account: usize,
    size: usize,
    options: &Options,
) -> Result<AccountContext, Box<dyn Error>> {
    let (account_id, region) = account_identity(account)?;
    let global = layout::is_global_account(options.seed, account, options.global_share)?;
    let edges = layout::edge_count(size, global)?;
    let (resource_pages, total_pages) = account_pages(size, edges)?;
    let (vpc, security_group) = account_hubs(account, size, global)?;
    Ok(AccountContext {
        account,
        size,
        scope: collection_scope(&account_id, region),
        account_id,
        region,
        global,
        payload_bytes: options.payload_bytes,
        vpc,
        security_group,
        edges,
        resource_pages,
        total_pages,
    })
}

fn account_identity(account: usize) -> Result<(String, &'static str), Box<dyn Error>> {
    let id = format!("{:012}", account.checked_add(1).ok_or("account overflow")?);
    Ok((id, layout::account_region(account)?))
}

fn account_pages(size: usize, edges: usize) -> Result<(usize, usize), Box<dyn Error>> {
    let resource_pages = layout::page_count(size)?;
    let total_pages = resource_pages
        .checked_add(layout::page_count(edges)?)
        .ok_or("page count overflow")?;
    Ok((resource_pages, total_pages))
}

fn account_hubs(
    account: usize,
    size: usize,
    global: bool,
) -> Result<(Option<String>, Option<String>), Box<dyn Error>> {
    let vpc = layout::hub_id(account, size, global, layout::vpc_index(global)?)?;
    let security_group =
        layout::hub_id(account, size, global, layout::security_group_index(global)?)?;
    Ok((vpc, security_group))
}

pub fn collection_scope(account_id: &str, region: &str) -> CollectionScope {
    CollectionScope {
        account_id: account_id.to_owned(),
        region: region.to_owned(),
        collector: "synthetic".to_owned(),
        parent_id: None,
        source: Source::Direct,
    }
}
