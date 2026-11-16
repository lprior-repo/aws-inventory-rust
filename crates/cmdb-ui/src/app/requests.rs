use super::*;

impl App {
    pub(super) fn request_page(&mut self, cx: &mut Cx) {
        let filter = self.filter(cx);
        self.request_filter(cx, filter);
    }

    pub(super) fn request_filter(&mut self, cx: &mut Cx, filter: Filter) {
        if self.shutting_down {
            return;
        }
        if self.retry_worker || self.worker.as_ref().is_none_or(DataWorker::is_closed) {
            self.start_worker(cx);
        }
        let result = self.worker.as_ref().map_or_else(
            || Err("inventory worker is not running".to_owned()),
            |worker| worker.refresh(filter.clone()),
        );
        if result.is_err() {
            self.retry_worker = self.worker.as_ref().is_none_or(DataWorker::is_closed);
        }
        let accepted = self.table(cx).borrow_mut::<ResourceTable>().is_some_and(|mut table| {
            table.apply_filter_result(cx, filter, result)
        });
        if !accepted {
            let accepted_filter = self.filter(cx);
            self.sync_filter_inputs(cx, &accepted_filter);
        }
        self.refresh(cx);
    }

    pub(super) fn sync_filter_inputs(&self, cx: &mut Cx, filter: &Filter) {
        self.ui.text_input(cx, ids!(search_input)).set_text(cx, &filter.query);
        self.ui.text_input(cx, ids!(account_filter_input)).set_text(cx, filter.account.as_deref().map_or("", |value| value));
        self.ui.text_input(cx, ids!(region_filter_input)).set_text(cx, filter.region.as_deref().map_or("", |value| value));
        self.ui.text_input(cx, ids!(kind_filter_input)).set_text(cx, filter.kind.as_deref().map_or("", |value| value));
        let key = filter.tag.as_ref().map_or("", |tag| tag.key.as_str());
        let value = filter.tag.as_ref().and_then(|tag| tag.value.as_deref());
        self.ui.text_input(cx, ids!(tag_key_input)).set_text(cx, key);
        self.ui.text_input(cx, ids!(tag_value_input)).set_text(cx, value.map_or("", |value| value));
        self.ui.check_box(cx, ids!(tag_value_exact)).set_active(cx, value.is_some(), Animate::No);
    }

    pub(super) fn request_selection(&mut self, cx: &mut Cx, row: crate::data::Row) {
        if self.shutting_down {
            return;
        }
        let result = self.enqueue_expansion(&row);
        if let Some(mut table) = self.table(cx).borrow_mut::<ResourceTable>() {
            table.apply_selection_result(cx, row, result);
        }
        self.refresh(cx);
    }

    fn enqueue_expansion(&self, row: &crate::data::Row) -> Result<(), String> {
        let request = cmdb_view::GraphNode {
            account: row.account.clone(),
            region: row.region.clone(),
            kind: row.kind.clone(),
            id: row.id.clone(),
        };
        self.worker.as_ref().map_or_else(
            || Err("inventory worker is not running".to_owned()),
            |worker| worker.expand(request),
        )
    }
    pub(super) fn request_expansion(&mut self, cx: &mut Cx) {
        let Some(row) = self
            .table(cx)
            .borrow::<ResourceTable>()
            .and_then(|table| table.selected_row())
        else {
            let result = self.worker.as_ref().map_or(Ok(()), DataWorker::invalidate_graph);
            if let Err(error) = result {
                self.apply_graph_event(cx, crate::data::Graph::default(), Some(error));
            }
            return;
        };
        if let Err(error) = self.enqueue_expansion(&row)
            && let Some(mut table) = self.table(cx).borrow_mut::<ResourceTable>()
        {
            table.graph_request_failed(cx, error);
        }
    }
}
