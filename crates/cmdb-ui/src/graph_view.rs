//! Wiring between the table's filtered graph and the DAG renderer.

use makepad_widgets::*;

use crate::dag::widget::{DagSelectAction, DagView};
use crate::data::{Graph, Row};
use crate::table::ResourceTable;

/// Push the filtered graph and the table's selected row into `DagView`.
pub(crate) fn sync(ui: &WidgetRef, cx: &mut Cx) {
    let Some((graph, selected)) = read_table(ui, cx) else {
        return;
    };
    feed_dag(ui, cx, &graph, selected);
    write_counts(ui, cx, &graph);
}

fn read_table(ui: &WidgetRef, cx: &mut Cx) -> Option<(Graph, Option<Row>)> {
    let table_ref = ui.widget(cx, ids!(table));
    let table = table_ref.borrow::<ResourceTable>()?;
    Some((table.graph(), table.selected_row()))
}

fn feed_dag(ui: &WidgetRef, cx: &mut Cx, graph: &Graph, selected: Option<Row>) {
    let dag_ref = ui.widget(cx, ids!(dag));
    if let Some(mut dag) = dag_ref.borrow_mut::<DagView>() {
        dag.set_graph(cx, graph);
        match selected {
            Some(row) => {
                dag.select_id(cx, row.id);
            }
            None => dag.clear_selection(cx),
        }
    }
}

fn write_counts(ui: &WidgetRef, cx: &mut Cx, graph: &Graph) {
    let fallback = (graph.nodes.len(), graph.edges.len(), 0, 0);
    let (nodes, edges, back, dropped) = match ui.widget(cx, ids!(dag)).borrow::<DagView>() {
        Some(dag) => (
            dag.node_count(),
            dag.edge_count(),
            dag.back_edge_count(),
            dag.dropped_nodes().saturating_add(dag.dropped_edges()),
        ),
        None => fallback,
    };
    let mut text = format!("{nodes} nodes · {edges} edges in the current filter");
    if back > 0 {
        text.push_str(&format!(" · {back} back edges"));
    }
    if dropped > 0 {
        text.push_str(&format!(" · {dropped} truncated"));
    }
    ui.label(cx, ids!(graph_counts)).set_text(cx, &text);
}

/// The DAG's click action, when this batch carries one. Outer `Some` means the
/// DAG posted; inner `None` means the canvas was clicked (clear selection).
pub(crate) fn click_id(ui: &WidgetRef, cx: &mut Cx, actions: &Actions) -> Option<Option<String>> {
    let uid = ui.widget(cx, ids!(dag)).widget_uid();
    let item = actions.find_widget_action(uid)?;
    Some(item.cast::<DagSelectAction>().id)
}

/// Select the table row matching the DAG id, or clear on an empty-canvas click.
pub(crate) fn apply_selection(ui: &WidgetRef, cx: &mut Cx, id: Option<&str>) -> bool {
    let table_ref = ui.widget(cx, ids!(table));
    let Some(mut table) = table_ref.borrow_mut::<ResourceTable>() else {
        return false;
    };
    match id {
        Some(id) => table.select_id(cx, id),
        None => table.clear_selection(cx),
    }
}
