#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    pub account: String,
    pub region: String,
    pub kind: String,
    pub id: String,
    pub name: String,
    pub key: String,
    pub record_bytes: Option<usize>,
    pub content_hash: Option<String>,
    pub tags: Vec<cmdb_view::ResourceTag>,
    pub tags_truncated: bool,
    pub tag_observation: Option<cmdb_view::StoredTagObservation>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Edge {
    pub from: String,
    pub to: String,
    pub label: String,
    pub provenance: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Graph {
    pub nodes: Vec<Row>,
    pub edges: Vec<Edge>,
    pub truncated: bool,
    pub tag_root: Option<String>,
    pub tag_affiliations: Vec<cmdb_view::TagAffiliation>,
    pub tags_truncated: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Filter {
    pub account: Option<String>,
    pub region: Option<String>,
    pub kind: Option<String>,
    pub query: String,
    pub tag: Option<cmdb_view::TagFilter>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Counts {
    pub accounts: u64,
    pub regions: u64,
    pub services: u64,
    pub resources: u64,
}

#[derive(Clone, Debug, Default)]
pub struct Data {
    rows: Vec<Row>,
    filter: Filter,
    selected: Option<Row>,
    graph: Graph,
    counts: Counts,
    total_matching: u64,
    loading: bool,
    error: Option<String>,
    graph_error: Option<String>,
    accounts: Vec<String>,
    regions: Vec<String>,
    kinds: Vec<String>,
}

impl Data {
    pub fn replace_page(&mut self, rows: Vec<Row>, total_matching: u64) {
        let previous = self.selected.as_ref().map(|row| row.key.clone());
        rows.iter().for_each(|row| {
            add_option(&mut self.accounts, &row.account);
            add_option(&mut self.regions, &row.region);
            add_option(&mut self.kinds, &row.kind);
        });
        self.rows = rows;
        self.total_matching = total_matching;
        self.selected = previous
            .and_then(|key| self.rows.iter().find(|row| row.key == key).cloned())
            .or_else(|| self.rows.first().cloned());
        self.graph = Graph::default();
        self.graph_error = None;
    }

    pub fn set_counts(&mut self, counts: Counts) {
        self.counts = counts;
    }

    pub fn set_loading(&mut self, loading: bool) {
        self.loading = loading;
    }

    pub fn set_error(&mut self, error: Option<String>) {
        self.error = error;
    }

    pub fn set_graph(&mut self, graph: Graph, error: Option<String>) {
        self.graph = graph;
        self.graph_error = error;
    }

    pub fn set_graph_error(&mut self, error: Option<String>) {
        self.graph_error = error;
    }

    pub fn visible_row(&self, index: usize) -> Option<&Row> {
        self.rows.get(index)
    }

    pub fn visible_len(&self) -> usize {
        self.rows.len()
    }

    pub fn total_len(&self) -> u64 {
        self.counts.resources
    }

    pub fn selected_row(&self) -> Option<&Row> {
        self.selected.as_ref()
    }

    pub fn select_visible(&mut self, index: usize) -> bool {
        let Some(row) = self.rows.get(index).cloned() else {
            return false;
        };
        self.select_row(row)
    }

    pub fn row_for_key(&self, key: &str) -> Option<&Row> {
        self.rows.iter().chain(self.graph.nodes.iter()).find(|row| row.key == key)
    }

    pub fn select_key(&mut self, key: &str) -> bool {
        let Some(row) = self.row_for_key(key).cloned() else {
            return false;
        };
        self.select_row(row)
    }

    pub fn select_row(&mut self, row: Row) -> bool {
        if self
            .selected
            .as_ref()
            .is_some_and(|selected| selected.key == row.key)
        {
            return false;
        }
        self.selected = Some(row);
        self.graph = Graph::default();
        self.graph_error = None;
        true
    }

    pub fn clear_selection(&mut self) -> bool {
        let changed = self.selected.take().is_some();
        self.graph = Graph::default();
        self.graph_error = None;
        changed
    }

    pub fn account_filter(&self) -> Option<&str> {
        self.filter.account.as_deref()
    }

    pub fn region_filter(&self) -> Option<&str> {
        self.filter.region.as_deref()
    }

    pub fn kind_filter(&self) -> Option<&str> {
        self.filter.kind.as_deref()
    }

    pub fn filter(&self) -> &Filter {
        &self.filter
    }

    pub fn set_filter(&mut self, filter: Filter) {
        self.filter = filter;
    }

    pub fn set_account(&mut self, value: Option<&str>) {
        self.filter.account = value.map(str::to_owned);
    }

    pub fn set_region(&mut self, value: Option<&str>) {
        self.filter.region = value.map(str::to_owned);
    }

    pub fn set_kind(&mut self, value: Option<&str>) {
        self.filter.kind = value.map(str::to_owned);
    }

    pub fn set_query(&mut self, value: &str) {
        self.filter.query = value.trim().to_owned();
    }

    pub fn set_tag(&mut self, value: Option<cmdb_view::TagFilter>) {
        self.filter.tag = value;
    }

    pub fn graph_status(&self) -> String {
        let status = match &self.graph_error {
            Some(error) => format!("Graph offline · {error}"),
            None => self.graph_summary(),
        };
        let status = if self.graph.truncated {
            format!("{status} · neighborhood truncated by query limits")
        } else {
            status
        };
        if self.graph.tags_truncated {
            format!("{status} · tag affiliations truncated by query limits")
        } else {
            status
        }
    }

    fn graph_summary(&self) -> String {
        if self.graph.nodes.is_empty() {
            return match self.selected {
                Some(_) => "No graph neighbors were recorded".to_owned(),
                None => "Select a resource to expand its graph neighbors".to_owned(),
            };
        }
        let edges = &self.graph.edges;
        let provenance = edges
            .first()
            .map_or("unknown", |edge| edge.provenance.as_str());
        let nodes = self.graph.nodes.len();
        let count = edges.len();
        format!("{nodes} nodes · {count} relationships · provenance: {provenance}")
    }

    pub fn visual_graph(&self) -> Graph {
        let mut graph = self.graph.clone();
        if let Some(selected) = self
            .selected
            .as_ref()
            .filter(|row| !graph.nodes.iter().any(|node| node.key == row.key))
        {
            graph.nodes.push(selected.clone());
        }
        graph
    }

    pub fn summary(&self) -> String {
        format!(
            "{} accounts · {} resources · {} regions · {} services",
            self.counts.accounts, self.counts.resources, self.counts.regions, self.counts.services
        )
    }

    pub fn status_line(&self) -> String {
        if self.loading {
            return "Loading inventory…".to_owned();
        }
        if let Some(error) = &self.error {
            return format!("Database error · {error}");
        }
        if self.total_len() == 0 {
            return "Database opened · no resources yet".to_owned();
        }
        format!(
            "{} matching · {} loaded",
            self.total_matching,
            self.rows.len()
        )
    }

    pub fn options(&self, facet: Facet) -> Vec<String> {
        match facet {
            Facet::Account => self.accounts.clone(),
            Facet::Region => self.regions.clone(),
            Facet::Kind => self.kinds.clone(),
        }
    }
}

fn add_option(values: &mut Vec<String>, value: &str) {
    if !values.iter().any(|item| item == value) {
        values.push(value.to_owned());
    }
}

#[derive(Clone, Copy)]
pub enum Facet {
    Account,
    Region,
    Kind,
}

#[cfg(test)]
mod tests;
