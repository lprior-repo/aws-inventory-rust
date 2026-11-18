use crate::data::{Counts, Data, Filter, Graph, Row};

pub(super) fn apply_filter_enqueue_result(
    data: &mut Data,
    filter: Filter,
    result: Result<(), String>,
) -> bool {
    match result {
        Ok(()) => {
            data.set_filter(filter);
            data.set_error(None);
            data.set_loading(true);
            true
        }
        Err(error) => {
            data.set_loading(false);
            data.set_error(Some(error));
            false
        }
    }
}

pub(super) fn apply_selection_enqueue_result(
    data: &mut Data,
    row: Row,
    result: Result<(), String>,
) -> bool {
    match result {
        Ok(()) => data.select_row(row),
        Err(error) => {
            data.set_graph_error(Some(error));
            false
        }
    }
}

pub(super) fn apply_snapshot_data(
    data: &mut Data,
    counts: Counts,
    rows: Vec<Row>,
    total_matching: u64,
    error: Option<String>,
) {
    data.set_loading(false);
    if error.is_some() {
        data.set_counts(Counts::default());
        data.replace_page(Vec::new(), 0);
    } else {
        data.set_counts(counts);
        data.replace_page(rows, total_matching);
    }
    data.set_graph(Graph::default(), None);
    data.set_error(error);
}

pub(super) fn apply_graph_data(data: &mut Data, graph: Graph, error: Option<String>) -> bool {
    if graph.tag_root.as_deref().is_some_and(|root| {
        data.selected_row()
            .is_none_or(|selected| selected.key != root)
    }) {
        return false;
    }
    let graph = if error.is_some() {
        Graph::default()
    } else {
        graph
    };
    data.set_graph(graph, error);
    true
}

#[cfg(test)]
#[path = "../app/state_tests.rs"]
mod state_tests;

#[cfg(test)]
#[path = "../app/request_state_tests.rs"]
mod request_state_tests;
