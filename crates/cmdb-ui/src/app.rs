use makepad_widgets::*;

use crate::data::{ACCOUNTS, KINDS, REGIONS};
use crate::table::ResourceTable;

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
}

impl App {
    fn table(&self, cx: &mut Cx) -> WidgetRef {
        self.ui.widget(cx, ids!(table))
    }

    fn set_account(&mut self, cx: &mut Cx, account: Option<&'static str>) {
        let table_ref = self.table(cx);
        {
            let Some(mut table) = table_ref.borrow_mut::<ResourceTable>() else {
                return;
            };
            table.set_account(cx, account);
        }
        self.refresh(cx);
    }

    fn set_region(&mut self, cx: &mut Cx, region: Option<&'static str>) {
        let table_ref = self.table(cx);
        {
            let Some(mut table) = table_ref.borrow_mut::<ResourceTable>() else {
                return;
            };
            table.set_region(cx, region);
        }
        self.refresh(cx);
    }

    fn set_kind(&mut self, cx: &mut Cx, kind: Option<&'static str>) {
        let table_ref = self.table(cx);
        {
            let Some(mut table) = table_ref.borrow_mut::<ResourceTable>() else {
                return;
            };
            table.set_kind(cx, kind);
        }
        self.refresh(cx);
    }

    fn set_query(&mut self, cx: &mut Cx, query: &str) {
        let table_ref = self.table(cx);
        {
            let Some(mut table) = table_ref.borrow_mut::<ResourceTable>() else {
                return;
            };
            table.set_query(cx, query);
        }
        self.refresh(cx);
    }

    fn account_filter(&self, cx: &mut Cx) -> Option<&'static str> {
        let table_ref = self.table(cx);
        table_ref
            .borrow::<ResourceTable>()
            .and_then(|table| table.account_filter())
    }

    fn region_filter(&self, cx: &mut Cx) -> Option<&'static str> {
        let table_ref = self.table(cx);
        table_ref
            .borrow::<ResourceTable>()
            .and_then(|table| table.region_filter())
    }

    fn kind_filter(&self, cx: &mut Cx) -> Option<&'static str> {
        let table_ref = self.table(cx);
        table_ref
            .borrow::<ResourceTable>()
            .and_then(|table| table.kind_filter())
    }

    fn toggle_account(&mut self, cx: &mut Cx, account: &'static str) {
        let active = self.account_filter(cx) == Some(account);
        self.set_account(cx, if active { None } else { Some(account) });
    }

    fn toggle_region(&mut self, cx: &mut Cx, region: &'static str) {
        let active = self.region_filter(cx) == Some(region);
        self.set_region(cx, if active { None } else { Some(region) });
    }

    fn toggle_kind(&mut self, cx: &mut Cx, kind: &'static str) {
        let active = self.kind_filter(cx) == Some(kind);
        self.set_kind(cx, if active { None } else { Some(kind) });
    }

    fn reset_filters(&mut self, cx: &mut Cx) {
        self.ui.text_input(cx, ids!(search_input)).set_text(cx, "");
        let table_ref = self.table(cx);
        {
            let Some(mut table) = table_ref.borrow_mut::<ResourceTable>() else {
                return;
            };
            table.set_account(cx, None);
            table.set_region(cx, None);
            table.set_kind(cx, None);
            table.set_query(cx, "");
        }
        self.refresh(cx);
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

    fn handle_accounts(&mut self, cx: &mut Cx, actions: &Actions) {
        let [prod, staging, sandbox] = ACCOUNTS;
        if self.ui.button(cx, ids!(prod_button)).clicked(actions) {
            self.toggle_account(cx, prod);
        }
        if self.ui.button(cx, ids!(staging_button)).clicked(actions) {
            self.toggle_account(cx, staging);
        }
        if self.ui.button(cx, ids!(sandbox_button)).clicked(actions) {
            self.toggle_account(cx, sandbox);
        }
    }

    fn handle_regions(&mut self, cx: &mut Cx, actions: &Actions) {
        let [us_east, eu_west] = REGIONS;
        if self.ui.button(cx, ids!(useast_button)).clicked(actions) {
            self.toggle_region(cx, us_east);
        }
        if self.ui.button(cx, ids!(euwest_button)).clicked(actions) {
            self.toggle_region(cx, eu_west);
        }
    }

    fn handle_kinds(&mut self, cx: &mut Cx, actions: &Actions) {
        let [ec2, s3, rds, lambda] = KINDS;
        if self.ui.button(cx, ids!(ec2_button)).clicked(actions) {
            self.toggle_kind(cx, ec2);
        }
        if self.ui.button(cx, ids!(s3_button)).clicked(actions) {
            self.toggle_kind(cx, s3);
        }
        if self.ui.button(cx, ids!(rds_button)).clicked(actions) {
            self.toggle_kind(cx, rds);
        }
        if self.ui.button(cx, ids!(lambda_button)).clicked(actions) {
            self.toggle_kind(cx, lambda);
        }
    }

    fn handle_row_clicks(&mut self, cx: &mut Cx, actions: &Actions) {
        let table_ref = self.table(cx);
        let changed = match table_ref.borrow_mut::<ResourceTable>() {
            Some(mut table) => table.apply_actions(cx, actions),
            None => false,
        };
        if changed {
            self.refresh(cx);
        }
    }
}

impl MatchEvent for App {
    fn handle_startup(&mut self, cx: &mut Cx) {
        self.refresh(cx);
    }

    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        if self.ui.button(cx, ids!(table_tab)).clicked(actions) {
            self.set_view_mode(cx, ViewMode::Table);
        }
        if self.ui.button(cx, ids!(graph_tab)).clicked(actions) {
            self.set_view_mode(cx, ViewMode::Graph);
        }
        if self.ui.button(cx, ids!(all_button)).clicked(actions) {
            self.reset_filters(cx);
        }
        self.handle_accounts(cx, actions);
        self.handle_regions(cx, actions);
        self.handle_kinds(cx, actions);

        if let Some(text) = self.ui.text_input(cx, ids!(search_input)).changed(actions) {
            self.set_query(cx, &text);
        }
        self.handle_row_clicks(cx, actions);

        if let Some(id) = crate::graph_view::click_id(&self.ui, cx, actions) {
            crate::graph_view::apply_selection(&self.ui, cx, id.as_deref());
            self.refresh(cx);
        }
    }
}

impl AppMain for App {
    fn script_mod(vm: &mut ScriptVm) -> ScriptValue {
        crate::makepad_widgets::script_mod(vm);
        crate::table::script_mod(vm);
        crate::dag::script_mod(vm);
        crate::templates::script_mod(vm);
        if !vm.is_reload() {
            script_eval!(vm, { mod.state = {visible_count: 0 graph_view: 0} true });
        }
        crate::ui::script_mod(vm)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        self.match_event(cx, event);
        self.ui.handle_event(cx, event, &mut Scope::empty());
    }
}

fn highlight_tab(cx: &mut Cx, mut button: ButtonRef) {
    script_apply_eval!(cx, button, { draw_text +: { color: mod.theme.color_text_hl } });
}

fn dim_tab(cx: &mut Cx, mut button: ButtonRef) {
    script_apply_eval!(cx, button, { draw_text +: { color: mod.theme.color_text_meta } });
}
