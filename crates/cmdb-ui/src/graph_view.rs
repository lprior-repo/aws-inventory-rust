use makepad_widgets::*;

use crate::dag::widget::{DagSelectAction, DagView};
use crate::data::{Graph, Row};
use crate::table::ResourceTable;

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
                dag.select_id(cx, &row.key);
            }
            None => dag.clear_selection(cx),
        }
    }
}

fn write_counts(ui: &WidgetRef, cx: &mut Cx, graph: &Graph) {
    let (nodes, edges, back, dropped) = dag_counts(ui, cx, graph);
    let status = ui
        .widget(cx, ids!(table))
        .borrow::<ResourceTable>()
        .map_or_else(
            || "Graph view unavailable".to_owned(),
            |table| table.graph_status(),
        );
    let mut text = if graph.nodes.is_empty() {
        status
    } else {
        format!("{nodes} visual nodes · {edges} visual edges · {status}")
    };
    if let Some(affiliation) = graph.tag_affiliations.first() {
        text.push_str(&format!(
            " · {} live tag affiliations · provenance: {}",
            graph.tag_affiliations.len(),
            affiliation.provenance,
        ));
    }
    if back > 0 {
        text.push_str(&format!(" · {back} back edges"));
    }
    if dropped > 0 {
        text.push_str(&format!(" · {dropped} truncated"));
    }
    ui.label(cx, ids!(graph_counts)).set_text(cx, &text);
}

fn dag_counts(ui: &WidgetRef, cx: &mut Cx, graph: &Graph) -> (usize, usize, usize, usize) {
    match ui.widget(cx, ids!(dag)).borrow::<DagView>() {
        Some(dag) => (
            dag.node_count(),
            dag.edge_count(),
            dag.back_edge_count(),
            dag.dropped_nodes().saturating_add(dag.dropped_edges()),
        ),
        None => (graph.nodes.len(), graph.edges.len(), 0, 0),
    }
}

pub(crate) fn click_id(ui: &WidgetRef, cx: &mut Cx, actions: &Actions) -> Option<Option<String>> {
    let uid = ui.widget(cx, ids!(dag)).widget_uid();
    let item = actions.find_widget_action(uid)?;
    Some(item.cast::<DagSelectAction>().id)
}
