use std::error::Error;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::Path;

use inventory_core::Resource;

use crate::dist;
use crate::edges::write_edges;
use crate::model::{AccountStats, MultiDigest, Oracle, PAGE_SIZE, canonical_resource};
use crate::options::Options;
use crate::pages::emit_page;
use crate::plan::{AccountContext, account_context};
use crate::records::{self, ResourceContext};
use crate::rng::Rng;
use crate::sink::EventSink;
use crate::skeleton::empty_oracle;

pub fn generate(root: &Path, options: Options) -> Result<Oracle, Box<dyn Error>> {
    options.validate()?;
    let sizes = dist::account_sizes(&options)?;
    fs::create_dir_all(root)?;
    let mut oracle = empty_oracle(&options)?;
    let (digest, duplicates, reorders) = write_events(root, &options, &sizes, &mut oracle)?;
    oracle.duplicates = duplicates;
    oracle.reorders = reorders;
    let deletions_path = root.join("deletions.jsonl");
    oracle.deletions = crate::deletions::write_deletions(&deletions_path, &options, &sizes)?;
    oracle.digest = digest.finish();
    fs::write(
        root.join("oracle.json"),
        serde_json::to_vec_pretty(&oracle)?,
    )?;
    Ok(oracle)
}

fn write_events(
    root: &Path,
    options: &Options,
    sizes: &[usize],
    oracle: &mut Oracle,
) -> Result<(MultiDigest, usize, usize), Box<dyn Error>> {
    let file = File::create(root.join("events.jsonl"))?;
    let mut sink = EventSink::new(BufWriter::new(file), options)?;
    let mut digest = MultiDigest::default();
    let mut rng = Rng::new(options.seed);
    for account in 0..options.accounts {
        let size = sizes.get(account).copied().ok_or("account size missing")?;
        generate_account(
            account,
            size,
            options,
            &mut rng,
            &mut digest,
            oracle,
            &mut sink,
        )?;
    }
    let (duplicates, reorders) = finish_sink(&mut sink)?;
    Ok((digest, duplicates, reorders))
}

fn finish_sink<W: Write>(sink: &mut EventSink<W>) -> Result<(usize, usize), Box<dyn Error>> {
    let counts = (sink.duplicates, sink.reorders);
    sink.finish()?;
    Ok(counts)
}

fn generate_account<W: Write>(
    account: usize,
    size: usize,
    options: &Options,
    rng: &mut Rng,
    digest: &mut MultiDigest,
    oracle: &mut Oracle,
    sink: &mut EventSink<W>,
) -> Result<(), Box<dyn Error>> {
    let context = account_context(account, size, options)?;
    write_resources(&context, rng, digest, oracle, sink)?;
    write_edges(&context, digest, sink)?;
    record_account(oracle, &context);
    Ok(())
}

fn write_resources<W: Write>(
    context: &AccountContext,
    rng: &mut Rng,
    digest: &mut MultiDigest,
    oracle: &mut Oracle,
    sink: &mut EventSink<W>,
) -> Result<(), Box<dyn Error>> {
    for page_index in 0..context.resource_pages {
        let start = page_index
            .checked_mul(PAGE_SIZE)
            .ok_or("page start overflow")?;
        let end = start.saturating_add(PAGE_SIZE).min(context.size);
        let resources = build_page(context, start, end, rng, digest, oracle)?;
        let number = page_index.saturating_add(1);
        emit_page(
            &context.scope,
            number,
            context.total_pages,
            resources,
            Vec::new(),
            sink,
        )?;
    }
    Ok(())
}

fn build_page(
    context: &AccountContext,
    start: usize,
    end: usize,
    rng: &mut Rng,
    digest: &mut MultiDigest,
    oracle: &mut Oracle,
) -> Result<Vec<Resource>, Box<dyn Error>> {
    let resource_context = resource_context(context);
    let mut resources = Vec::with_capacity(end.checked_sub(start).ok_or("page span")?);
    for index in start..end {
        let resource = records::build_resource(&resource_context, index, rng)?;
        digest.add(canonical_resource(&resource).as_bytes());
        count_resource(oracle, &resource);
        if context.global && index == 0 {
            oracle.global_services = oracle.global_services.saturating_add(1);
        }
        resources.push(resource);
    }
    Ok(resources)
}

fn resource_context(context: &AccountContext) -> ResourceContext<'_> {
    ResourceContext {
        account: context.account,
        account_id: &context.account_id,
        region: context.region,
        global: context.global,
        payload_bytes: context.payload_bytes,
        vpc: context.vpc.as_deref(),
    }
}

fn count_resource(oracle: &mut Oracle, resource: &Resource) {
    let entry = oracle.kinds.entry(resource.kind.clone()).or_insert(0);
    *entry = entry.saturating_add(1);
    let entry = oracle.regions.entry(resource.region.clone()).or_insert(0);
    *entry = entry.saturating_add(1);
}

fn record_account(oracle: &mut Oracle, context: &AccountContext) {
    oracle.resources = oracle.resources.saturating_add(context.size);
    oracle.edges = oracle.edges.saturating_add(context.edges);
    oracle.per_account.insert(
        context.account_id.clone(),
        AccountStats {
            resources: context.size,
            edges: context.edges,
        },
    );
}
