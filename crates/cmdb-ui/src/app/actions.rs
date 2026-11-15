use super::*;

impl MatchEvent for App {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        self.handle_tabs(cx, actions);
        self.handle_filters(cx, actions);
        self.handle_row_clicks(cx, actions);
        self.handle_graph_click(cx, actions);
    }
}

impl App {
    pub(super) fn handle_tabs(&mut self, cx: &mut Cx, actions: &Actions) {
        if self.ui.button(cx, ids!(table_tab)).clicked(actions) {
            self.set_view_mode(cx, ViewMode::Table);
        }
        if self.ui.button(cx, ids!(graph_tab)).clicked(actions) {
            self.set_view_mode(cx, ViewMode::Graph);
        }
        if self.ui.button(cx, ids!(all_button)).clicked(actions) {
            self.reset_filters(cx);
        }
    }

    pub(super) fn handle_filters(&mut self, cx: &mut Cx, actions: &Actions) {
        self.handle_accounts(cx, actions);
        self.handle_regions(cx, actions);
        self.handle_kinds(cx, actions);
        self.handle_search(cx, actions);
        self.handle_exact_filters(cx, actions);
    }

    pub(super) fn handle_search(&mut self, cx: &mut Cx, actions: &Actions) {
        if let Some(text) = self.ui.text_input(cx, ids!(search_input)).changed(actions) {
            self.set_query(cx, &text);
        }
    }

    pub(super) fn handle_exact_filters(&mut self, cx: &mut Cx, actions: &Actions) {
        self.handle_account_filter(cx, actions);
        self.handle_region_filter(cx, actions);
        self.handle_kind_filter(cx, actions);
    }

    pub(super) fn handle_account_filter(&mut self, cx: &mut Cx, actions: &Actions) {
        if let Some(text) = self
            .ui
            .text_input(cx, ids!(account_filter_input))
            .changed(actions)
        {
            self.set_account(cx, exact_filter(&text));
        }
    }

    pub(super) fn handle_region_filter(&mut self, cx: &mut Cx, actions: &Actions) {
        if let Some(text) = self
            .ui
            .text_input(cx, ids!(region_filter_input))
            .changed(actions)
        {
            self.set_region(cx, exact_filter(&text));
        }
    }

    pub(super) fn handle_kind_filter(&mut self, cx: &mut Cx, actions: &Actions) {
        if let Some(text) = self
            .ui
            .text_input(cx, ids!(kind_filter_input))
            .changed(actions)
        {
            self.set_kind(cx, exact_filter(&text));
        }
    }

    pub(super) fn handle_graph_click(&mut self, cx: &mut Cx, actions: &Actions) {
        if let Some(id) = crate::graph_view::click_id(&self.ui, cx, actions) {
            crate::graph_view::apply_selection(&self.ui, cx, id.as_deref());
            self.request_expansion(cx);
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
        if matches!(event, Event::Startup) {
            self.start_worker(cx);
        }
        if matches!(event, Event::Signal) {
            self.drain_worker(cx);
        }
        self.match_event(cx, event);
        self.ui.handle_event(cx, event, &mut Scope::empty());
    }
}

pub(super) fn exact_filter(value: &str) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_owned())
}

pub(super) fn highlight_tab(cx: &mut Cx, mut button: ButtonRef) {
    script_apply_eval!(cx, button, { draw_text +: { color: mod.theme.color_text_hl } });
}

pub(super) fn dim_tab(cx: &mut Cx, mut button: ButtonRef) {
    script_apply_eval!(cx, button, { draw_text +: { color: mod.theme.color_text_meta } });
}
