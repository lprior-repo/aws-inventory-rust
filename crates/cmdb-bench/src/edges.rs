//! Edge page emission for the synthetic workload.

use std::error::Error;
use std::io::Write;

use inventory_core::Edge;

use crate::layout;
use crate::model::{MultiDigest, PAGE_SIZE, canonical_edge};
use crate::pages::emit_page;
use crate::plan::AccountContext;
use crate::records;
use crate::sink::EventSink;

pub fn write_edges<W: Write>(
    context: &AccountContext,
    digest: &mut MultiDigest,
    sink: &mut EventSink<W>,
) -> Result<(), Box<dyn Error>> {
    let mut buffer = EdgeBuffer::new(context.resource_pages);
    for index in 0..context.size {
        push_instance_edges(context, index, &mut buffer, digest, sink)?;
    }
    buffer.flush(context, sink)?;
    if buffer.generated != context.edges {
        return Err(format!(
            "edge count mismatch: generated {}, expected {}",
            buffer.generated, context.edges
        )
        .into());
    }
    Ok(())
}

fn push_instance_edges<W: Write>(
    context: &AccountContext,
    index: usize,
    buffer: &mut EdgeBuffer,
    digest: &mut MultiDigest,
    sink: &mut EventSink<W>,
) -> Result<(), Box<dyn Error>> {
    let (kind, from_id) = layout::kind_and_id(context.account, index, context.global)?;
    if kind != layout::INSTANCE {
        return Ok(());
    }
    let targets = [context.vpc.as_deref(), context.security_group.as_deref()];
    for to_id in targets.into_iter().flatten() {
        buffer.push(context, digest, sink, &from_id, to_id)?;
    }
    Ok(())
}

struct EdgeBuffer {
    edges: Vec<Edge>,
    page: usize,
    generated: usize,
}

impl EdgeBuffer {
    fn new(first_page: usize) -> Self {
        Self {
            edges: Vec::with_capacity(PAGE_SIZE),
            page: first_page,
            generated: 0,
        }
    }

    fn push<W: Write>(
        &mut self,
        context: &AccountContext,
        digest: &mut MultiDigest,
        sink: &mut EventSink<W>,
        from_id: &str,
        to_id: &str,
    ) -> Result<(), Box<dyn Error>> {
        let edge = records::make_edge(&edge_context(context), from_id, to_id);
        digest.add(canonical_edge(&edge).as_bytes());
        self.generated = self.generated.checked_add(1).ok_or("edge overflow")?;
        self.edges.push(edge);
        if self.edges.len() == PAGE_SIZE {
            self.flush(context, sink)?;
        }
        Ok(())
    }

    fn flush<W: Write>(
        &mut self,
        context: &AccountContext,
        sink: &mut EventSink<W>,
    ) -> Result<(), Box<dyn Error>> {
        if self.edges.is_empty() {
            return Ok(());
        }
        self.page = self.page.checked_add(1).ok_or("page number overflow")?;
        let edges = std::mem::take(&mut self.edges);
        emit_page(
            &context.scope,
            self.page,
            context.total_pages,
            Vec::new(),
            edges,
            sink,
        )
    }
}

fn edge_context(context: &AccountContext) -> records::ResourceContext<'_> {
    records::ResourceContext {
        account: context.account,
        account_id: &context.account_id,
        region: context.region,
        global: context.global,
        payload_bytes: context.payload_bytes,
        vpc: context.vpc.as_deref(),
    }
}
