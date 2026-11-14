use std::error::Error;
use std::io::Write;

use inventory_core::{CollectionEvent, CollectionPage, CollectionScope, Edge, Resource};

use crate::sink::EventSink;

pub fn emit_page<W: Write>(
    scope: &CollectionScope,
    number: usize,
    total: usize,
    resources: Vec<Resource>,
    edges: Vec<Edge>,
    sink: &mut EventSink<W>,
) -> Result<(), Box<dyn Error>> {
    let page = CollectionPage {
        scope: scope.clone(),
        page_number: u32::try_from(number)?,
        next_token: (number < total).then(|| format!("page-{number}")),
        resources,
        edges,
    };
    sink.emit(&CollectionEvent::Page(page))
}
