#[path = "app/actions.rs"]
mod actions;
use actions::*;
use makepad_widgets::makepad_platform::thread::SignalToUI;
use makepad_widgets::*;

use crate::data::{Facet, Filter};
use crate::table::ResourceTable;
use crate::worker::{DataWorker, WorkerConfig, WorkerEvent};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum ViewMode {
    Table,
    #[default]
    Graph,
}

#[derive(Script, ScriptHook)]
pub struct App {
    #[live]
    ui: WidgetRef,
    #[rust]
    view_mode: ViewMode,
    #[rust]
    worker: Option<DataWorker>,
    #[rust]
    signal: SignalToUI,
}

impl App {
    fn table(&self, cx: &mut Cx) -> WidgetRef {
        self.ui.widget(cx, ids!(table))
    }

    fn filter(&self, cx: &mut Cx) -> Filter {
        self.table(cx)
            .borrow::<ResourceTable>()
            .map_or_else(Filter::default, |table| table.filter())
    }

    fn set_account(&mut self, cx: &mut Cx, account: Option<String>) {
        let table = self.table(cx);
        if let Some(mut table) = table.borrow_mut::<ResourceTable>() {
            table.set_account(cx, account.as_deref());
        }
        self.request_page(cx);
    }

    fn set_region(&mut self, cx: &mut Cx, region: Option<String>) {
        let table = self.table(cx);
        if let Some(mut table) = table.borrow_mut::<ResourceTable>() {
            table.set_region(cx, region.as_deref());
        }
        self.request_page(cx);
    }

    fn set_kind(&mut self, cx: &mut Cx, kind: Option<String>) {
        let table = self.table(cx);
        if let Some(mut table) = table.borrow_mut::<ResourceTable>() {
            table.set_kind(cx, kind.as_deref());
        }
        self.request_page(cx);
    }

    fn set_query(&mut self, cx: &mut Cx, query: &str) {
        let table = self.table(cx);
        if let Some(mut table) = table.borrow_mut::<ResourceTable>() {
            table.set_query(cx, query);
        }
        self.request_page(cx);
    }

    fn request_page(&mut self, cx: &mut Cx) {
        let filter = self.filter(cx);
        let table = self.table(cx);
        if let Some(mut table) = table.borrow_mut::<ResourceTable>() {
            table.set_loading(true);
        }
        let result = match &self.worker {
            Some(worker) => worker.refresh(filter),
            None => Err("inventory worker is not running".to_owned()),
        };
        if let Err(error) = result
            && let Some(mut table) = table.borrow_mut::<ResourceTable>()
        {
            table.set_loading(false);
            table.set_error(Some(error));
        }
        self.refresh(cx);
    }

    fn request_expansion(&mut self, cx: &mut Cx) {
        let Some(row) = self
            .table(cx)
            .borrow::<ResourceTable>()
            .and_then(|table| table.selected_row())
        else {
            return;
        };
        if let Some(worker) = &self.worker {
            let request = cmdb_view::GraphNode {
                account: row.account,
                region: row.region,
                kind: row.kind,
                id: row.id,
            };
            if let Err(error) = worker.expand(request) {
                self.apply_graph_event(cx, crate::data::Graph::default(), Some(error));
            }
        }
    }

    fn refresh(&mut self, cx: &mut Cx) {
        crate::chrome::sync_chrome(&self.ui, cx);
        self.sync_view(cx);
    }

    fn set_view_mode(&mut self, cx: &mut Cx, mode: ViewMode) {
        if self.view_mode != mode {
            self.view_mode = mode;
            self.sync_view(cx);
        }
    }

    fn sync_view(&mut self, cx: &mut Cx) {
        let graph = self.view_mode == ViewMode::Graph;
        self.ui.view(cx, ids!(table_pane)).set_visible(cx, !graph);
        self.ui.view(cx, ids!(graph_pane)).set_visible(cx, graph);
        let table_tab = self.ui.button(cx, ids!(table_tab));
        let graph_tab = self.ui.button(cx, ids!(graph_tab));
        if graph {
            dim_tab(cx, table_tab);
            highlight_tab(cx, graph_tab);
            crate::graph_view::sync(&self.ui, cx);
        } else {
            highlight_tab(cx, table_tab);
            dim_tab(cx, graph_tab);
        }
        let graph_flag = usize::from(graph);
        script_eval!(cx, { mod.state.graph_view = #(graph_flag) });
    }

    fn facet_value(&self, cx: &mut Cx, facet: Facet, index: usize) -> Option<String> {
        self.table(cx)
            .borrow::<ResourceTable>()?
            .options(facet)
            .get(index)
            .cloned()
    }

    fn toggle_account(&mut self, cx: &mut Cx, index: usize) {
        let Some(account) = self.facet_value(cx, Facet::Account, index) else {
            return;
        };
        let active = self.filter(cx).account == Some(account.clone());
        self.set_account(cx, (!active).then_some(account));
    }

    fn toggle_region(&mut self, cx: &mut Cx, index: usize) {
        let Some(region) = self.facet_value(cx, Facet::Region, index) else {
            return;
        };
        let active = self.filter(cx).region == Some(region.clone());
        self.set_region(cx, (!active).then_some(region));
    }

    fn toggle_kind(&mut self, cx: &mut Cx, index: usize) {
        let Some(kind) = self.facet_value(cx, Facet::Kind, index) else {
            return;
        };
        let active = self.filter(cx).kind == Some(kind.clone());
        self.set_kind(cx, (!active).then_some(kind));
    }

    fn reset_filters(&mut self, cx: &mut Cx) {
        self.ui.text_input(cx, ids!(search_input)).set_text(cx, "");
        self.ui
            .text_input(cx, ids!(account_filter_input))
            .set_text(cx, "");
        self.ui
            .text_input(cx, ids!(region_filter_input))
            .set_text(cx, "");
        self.ui
            .text_input(cx, ids!(kind_filter_input))
            .set_text(cx, "");
        let table = self.table(cx);
        if let Some(mut table) = table.borrow_mut::<ResourceTable>() {
            table.set_account(cx, None);
            table.set_region(cx, None);
            table.set_kind(cx, None);
            table.set_query(cx, "");
        }
        self.request_page(cx);
    }
    fn start_worker(&mut self, cx: &mut Cx) {
        self.worker = Some(DataWorker::start(
            WorkerConfig::from_process_args(),
            self.signal.clone(),
        ));
        if let Some(mut table) = self.table(cx).borrow_mut::<ResourceTable>() {
            table.set_loading(true);
        }
        self.refresh(cx);
    }

    fn drain_worker(&mut self, cx: &mut Cx) {
        let events = match &self.worker {
            Some(worker) => worker.drain(),
            None => Vec::new(),
        };
        events
            .into_iter()
            .for_each(|event| self.apply_worker_event(cx, event));
    }

    fn apply_worker_event(&mut self, cx: &mut Cx, event: WorkerEvent) {
        match event {
            WorkerEvent::Snapshot {
                counts,
                rows,
                total,
                error,
            } => self.apply_snapshot(cx, counts, rows, total, error),
            WorkerEvent::Graph { graph, error } => {
                self.apply_graph_event(cx, graph, error);
            }
        }
        self.refresh(cx);
    }

    fn apply_snapshot(
        &mut self,
        cx: &mut Cx,
        counts: crate::data::Counts,
        rows: Vec<crate::data::Row>,
        total: u64,
        error: Option<String>,
    ) {
        if let Some(mut table) = self.table(cx).borrow_mut::<ResourceTable>() {
            table.set_loading(false);
            table.set_error(error.clone());
            if error.is_none() {
                table.set_counts(counts);
                table.apply_page(cx, rows, total);
            }
        }
        if error.is_none() {
            self.request_expansion(cx);
        }
    }

    fn apply_graph_event(&mut self, cx: &mut Cx, graph: crate::data::Graph, error: Option<String>) {
        if let Some(mut table) = self.table(cx).borrow_mut::<ResourceTable>() {
            table.apply_graph(cx, graph, error);
        }
    }

    fn handle_accounts(&mut self, cx: &mut Cx, actions: &Actions) {
        if self.ui.button(cx, ids!(prod_button)).clicked(actions) {
            self.toggle_account(cx, 0);
        }
        if self.ui.button(cx, ids!(staging_button)).clicked(actions) {
            self.toggle_account(cx, 1);
        }
        if self.ui.button(cx, ids!(sandbox_button)).clicked(actions) {
            self.toggle_account(cx, 2);
        }
    }

    fn handle_regions(&mut self, cx: &mut Cx, actions: &Actions) {
        if self.ui.button(cx, ids!(useast_button)).clicked(actions) {
            self.toggle_region(cx, 0);
        }
        if self.ui.button(cx, ids!(euwest_button)).clicked(actions) {
            self.toggle_region(cx, 1);
        }
    }

    fn handle_kinds(&mut self, cx: &mut Cx, actions: &Actions) {
        if self.ui.button(cx, ids!(ec2_button)).clicked(actions) {
            self.toggle_kind(cx, 0);
        }
        if self.ui.button(cx, ids!(s3_button)).clicked(actions) {
            self.toggle_kind(cx, 1);
        }
        if self.ui.button(cx, ids!(rds_button)).clicked(actions) {
            self.toggle_kind(cx, 2);
        }
        if self.ui.button(cx, ids!(lambda_button)).clicked(actions) {
            self.toggle_kind(cx, 3);
        }
    }

    fn handle_row_clicks(&mut self, cx: &mut Cx, actions: &Actions) {
        let changed = self
            .table(cx)
            .borrow_mut::<ResourceTable>()
            .is_some_and(|mut table| table.apply_actions(cx, actions));
        if changed {
            self.request_expansion(cx);
            self.refresh(cx);
        }
    }
}
