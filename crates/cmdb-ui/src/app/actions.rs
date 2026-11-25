use super::*;
use crate::frame_time::FrameTimeRecorder;

impl MatchEvent for App {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        self.handle_tabs(cx, actions);
        self.handle_filters(cx, actions);
        self.handle_row_clicks(cx, actions);
        self.handle_graph_click(cx, actions);
    }

    fn handle_timer(&mut self, cx: &mut Cx, event: &TimerEvent) {
        if self.completion_poll.is_timer(event).is_some() {
            self.drain_worker(cx);
        }
    }
}

impl App {
    fn stop_worker(&mut self, cx: &mut Cx) {
        self.shutting_down = true;
        cx.stop_timer(std::mem::take(&mut self.completion_poll));
        drop(self.worker.take());
        self.retry_worker = false;
    }

    pub(super) fn start_worker_with_config(&mut self, cx: &mut Cx, config: WorkerConfig) {
        if self.shutting_down {
            return;
        }
        self.worker_config = Some(config.clone());
        self.graph_access_error = None;
        self.worker = Some(DataWorker::start(config, self.signal.clone()));
        self.retry_worker = false;
        if self.completion_poll.is_empty() {
            self.completion_poll = cx.start_interval(0.25);
        }
        if let Some(mut table) = self.table(cx).borrow_mut::<ResourceTable>() {
            table.set_loading(true);
        }
        self.refresh(cx);
    }

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
        if self.ui.button(cx, ids!(refresh_button)).clicked(actions) {
            self.request_page(cx);
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
        self.handle_tag_filter(cx, actions);
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

    pub(super) fn handle_tag_filter(&mut self, cx: &mut Cx, actions: &Actions) {
        let key = self.ui.text_input(cx, ids!(tag_key_input));
        let value = self.ui.text_input(cx, ids!(tag_value_input));
        let exact = self.ui.check_box(cx, ids!(tag_value_exact));
        if key.changed(actions).is_some()
            || value.changed(actions).is_some()
            || exact.changed(actions).is_some()
        {
            let value = value.text();
            let tag = tag_filter(&key.text(), exact.active(cx).then_some(value.as_str()));
            self.set_tag(cx, tag);
        }
    }

    pub(super) fn handle_graph_click(&mut self, cx: &mut Cx, actions: &Actions) {
        if self.frame_harness.is_some() {
            return;
        }
        let Some(id) = crate::graph_view::click_id(&self.ui, cx, actions) else {
            return;
        };
        if let Some(id) = id {
            self.select_graph_resource(cx, &id);
        } else {
            let changed = self
                .table(cx)
                .borrow_mut::<ResourceTable>()
                .is_some_and(|mut table| table.clear_selection(cx));
            if changed {
                let result = self
                    .worker
                    .as_ref()
                    .map_or(Ok(()), crate::worker::DataWorker::invalidate_graph);
                if let Err(error) = result {
                    self.apply_graph_event(cx, crate::data::Graph::default(), Some(error));
                }
                self.refresh(cx);
            }
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
        let dispatch = std::time::Instant::now();
        if matches!(event, Event::Shutdown) {
            self.shutdown_frame_harness(cx);
            self.stop_worker(cx);
            self.shutdown_frame_recorder();
            return;
        }
        if self.shutting_down {
            return;
        }
        self.prepare_event(cx, event);
        self.step_frame_harness(cx, event);
        self.match_event(cx, event);
        self.ui.handle_event(cx, event, &mut Scope::empty());
        self.finish_harness_event(cx, event, dispatch.elapsed());
    }
}

impl App {
    fn prepare_event(&mut self, cx: &mut Cx, event: &Event) {
        if matches!(event, Event::Startup) {
            self.start_frame_recorder();
            self.start_frame_harness(cx);
            self.start_worker(cx);
        }
        if matches!(event, Event::Signal) {
            self.drain_worker(cx);
        }
        if matches!(event, Event::Draw(_)) {
            self.record_frame(cx);
        }
    }

    fn start_frame_recorder(&mut self) {
        self.frame_time_recorder = match FrameTimeRecorder::from_env() {
            Ok(recorder) => Some(recorder),
            Err(error) => {
                eprintln!("{error}");
                None
            }
        };
    }

    fn shutdown_frame_recorder(&mut self) {
        if let Some(mut recorder) = self.frame_time_recorder.take() {
            recorder.shutdown();
        }
    }

    fn record_frame(&mut self, cx: &mut Cx) {
        let node_count = self
            .table(cx)
            .borrow::<ResourceTable>()
            .map_or(0, |table| table.graph().nodes.len());
        if let Some(recorder) = self.frame_time_recorder.as_mut() {
            recorder.frame(node_count);
        }
    }
}

pub(super) fn exact_filter(value: &str) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_owned())
}

pub(super) fn tag_filter(key: &str, value: Option<&str>) -> Option<cmdb_view::TagFilter> {
    (!key.is_empty()).then(|| cmdb_view::TagFilter {
        key: key.to_owned(),
        value: value.map(str::to_owned),
    })
}

pub(super) fn highlight_tab(cx: &mut Cx, mut button: ButtonRef) {
    script_apply_eval!(cx, button, { draw_text +: { color: mod.theme.color_text_hl } });
}

pub(super) fn dim_tab(cx: &mut Cx, mut button: ButtonRef) {
    script_apply_eval!(cx, button, { draw_text +: { color: mod.theme.color_text_meta } });
}

#[cfg(test)]
mod tests {
    use super::tag_filter;
    use cmdb_view::TagFilter;

    #[test]
    fn given_tag_value_mode_when_filtering_then_empty_exact_values_and_key_only_are_distinct() {
        assert_eq!(
            tag_filter("Environment", None),
            Some(TagFilter {
                key: "Environment".to_owned(),
                value: None,
            })
        );
        assert_eq!(
            tag_filter("Environment", Some("")),
            Some(TagFilter {
                key: "Environment".to_owned(),
                value: Some(String::new()),
            })
        );
        assert_eq!(tag_filter("", Some("production")), None);
    }

    #[test]
    fn given_significant_tag_whitespace_when_filtering_then_exact_identity_is_preserved() {
        let filter = tag_filter(" owner ", Some(" équipe "));
        assert_eq!(
            filter,
            Some(TagFilter {
                key: " owner ".to_owned(),
                value: Some(" équipe ".to_owned()),
            })
        );
    }
}

#[cfg(test)]
#[path = "integration_tests.rs"]
mod integration_tests;
