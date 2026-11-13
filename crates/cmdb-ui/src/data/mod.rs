//! Filter state and the visible projection over the synthetic fixture.
//!
//! The fixture rows live in `fixture`; `Data` owns the filter, the indices the
//! filter keeps, and the current selection. A real source replaces
//! [`Data::synthetic`] without touching the widget tree.

mod fixture;

pub use fixture::{ACCOUNTS, KINDS, REGIONS, Row};
use fixture::{EDGES, ROWS};

/// One relationship between two resource ids.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Edge {
    pub from: &'static str,
    pub to: &'static str,
    pub label: &'static str,
}

/// The graph the DAG renderer consumes: the visible rows plus every edge whose
/// two endpoints are visible. Ids are [`Row::id`] values.
#[derive(Clone, Debug, Default)]
pub struct Graph {
    pub nodes: Vec<Row>,
    pub edges: Vec<Edge>,
}

/// The active filter, one selector per rail section plus the search box.
#[derive(Clone, Debug, Default)]
pub struct Filter {
    pub account: Option<&'static str>,
    pub region: Option<&'static str>,
    pub kind: Option<&'static str>,
    /// Lower-cased, trimmed search text; empty means "no text filter".
    pub query: String,
}

impl Filter {
    fn matches(&self, row: &Row) -> bool {
        self.account.is_none_or(|account| account == row.account)
            && self.region.is_none_or(|region| region == row.region)
            && self.kind.is_none_or(|kind| kind == row.kind)
            && (self.query.is_empty() || matches_query(row, &self.query))
    }
}

fn matches_query(row: &Row, query: &str) -> bool {
    row.id.to_lowercase().contains(query)
        || row.name.to_lowercase().contains(query)
        || row.kind.contains(query)
}

/// The table's backing state: the full fixture, the filter, and the indices of
/// rows the filter keeps visible.
#[derive(Clone, Debug)]
pub struct Data {
    rows: &'static [Row],
    filter: Filter,
    visible: Vec<usize>,
    selected: Option<usize>,
}

impl Data {
    /// Build the scaffold's dataset from the embedded fixture.
    pub fn synthetic() -> Self {
        let mut data = Self {
            rows: &ROWS,
            filter: Filter::default(),
            visible: Vec::with_capacity(ROWS.len()),
            selected: None,
        };
        data.rebuild();
        data
    }

    fn rebuild(&mut self) {
        self.visible = self
            .rows
            .iter()
            .enumerate()
            .filter(|(_, row)| self.filter.matches(row))
            .map(|(index, _)| index)
            .collect();
        if self
            .selected
            .is_some_and(|index| index >= self.visible.len())
        {
            self.selected = None;
        }
        if self.selected.is_none() && !self.visible.is_empty() {
            self.selected = Some(0);
        }
    }

    pub fn visible_len(&self) -> usize {
        self.visible.len()
    }

    pub fn total_len(&self) -> usize {
        self.rows.len()
    }

    /// The row at `index` of the *filtered* view.
    pub fn visible_row(&self, index: usize) -> Option<&Row> {
        self.visible.get(index).and_then(|row| self.rows.get(*row))
    }

    pub fn selected_row(&self) -> Option<&Row> {
        self.selected.and_then(|index| self.visible_row(index))
    }

    /// Position of a resource id in the filtered view.
    pub fn visible_index_of(&self, id: &str) -> Option<usize> {
        self.visible
            .iter()
            .position(|index| self.rows.get(*index).is_some_and(|row| row.id == id))
    }

    /// Clears the selection; returns whether anything changed.
    pub fn clear_selection(&mut self) -> bool {
        let changed = self.selected.is_some();
        self.selected = None;
        changed
    }

    pub fn select_visible(&mut self, index: usize) -> bool {
        let changed = self.selected != Some(index) && index < self.visible.len();
        if changed {
            self.selected = Some(index);
        }
        changed
    }

    pub fn account_filter(&self) -> Option<&'static str> {
        self.filter.account
    }

    pub fn region_filter(&self) -> Option<&'static str> {
        self.filter.region
    }

    pub fn kind_filter(&self) -> Option<&'static str> {
        self.filter.kind
    }

    pub fn set_account(&mut self, account: Option<&'static str>) {
        if self.filter.account != account {
            self.filter.account = account;
            self.rebuild();
        }
    }

    pub fn set_region(&mut self, region: Option<&'static str>) {
        if self.filter.region != region {
            self.filter.region = region;
            self.rebuild();
        }
    }

    pub fn set_kind(&mut self, kind: Option<&'static str>) {
        if self.filter.kind != kind {
            self.filter.kind = kind;
            self.rebuild();
        }
    }

    pub fn set_query(&mut self, query: &str) {
        let query = query.trim().to_lowercase();
        if self.filter.query != query {
            self.filter.query = query;
            self.rebuild();
        }
    }

    /// Human-readable description of the active selectors.
    pub fn filter_label(&self) -> String {
        let mut parts: Vec<&str> = Vec::new();
        if let Some(account) = self.filter.account {
            parts.push(account);
        }
        if let Some(region) = self.filter.region {
            parts.push(region);
        }
        if let Some(kind) = self.filter.kind {
            parts.push(kind);
        }
        if !self.filter.query.is_empty() {
            parts.push(&self.filter.query);
        }
        if parts.is_empty() {
            "All resources".to_string()
        } else {
            parts.join(" · ")
        }
    }

    /// Title-bar summary, derived from the fixture.
    pub fn summary(&self) -> String {
        let accounts = ACCOUNTS
            .iter()
            .filter(|it| self.account_count(it) > 0)
            .count();
        let regions = REGIONS
            .iter()
            .filter(|it| self.region_count(it) > 0)
            .count();
        format!(
            "{} accounts · {} resources · {} edges · {} regions · synthetic fixture",
            accounts,
            self.total_len(),
            self.edge_count(),
            regions
        )
    }

    /// One-line toolbar count: visible rows, plus the active selectors.
    pub fn status_line(&self) -> String {
        let shown = self.visible_len();
        let label = self.filter_label();
        if label == "All resources" {
            format!("{shown} shown")
        } else {
            format!("{shown} shown · {label}")
        }
    }

    /// Whether a resource id is in the filtered view.
    fn visible_id(&self, id: &str) -> bool {
        self.visible
            .iter()
            .any(|index| self.rows.get(*index).is_some_and(|row| row.id == id))
    }

    /// Edges whose two endpoints are both visible.
    pub fn edge_count(&self) -> usize {
        EDGES
            .iter()
            .filter(|edge| self.visible_id(edge.from) && self.visible_id(edge.to))
            .count()
    }

    /// The node/edge graph the DAG renderer draws, scoped to the filter.
    pub fn graph(&self) -> Graph {
        let nodes = self
            .visible
            .iter()
            .filter_map(|index| self.rows.get(*index).copied())
            .collect();
        let edges = EDGES
            .iter()
            .filter(|edge| self.visible_id(edge.from) && self.visible_id(edge.to))
            .map(|edge| Edge {
                from: edge.from,
                to: edge.to,
                label: edge.label,
            })
            .collect();
        Graph { nodes, edges }
    }

    /// Count of rows in an account, independent of the active filter.
    pub fn account_count(&self, account: &str) -> usize {
        self.rows
            .iter()
            .filter(|row| row.account == account)
            .count()
    }

    pub fn region_count(&self, region: &str) -> usize {
        self.rows.iter().filter(|row| row.region == region).count()
    }

    pub fn kind_count(&self, kind: &str) -> usize {
        self.rows.iter().filter(|row| row.kind == kind).count()
    }
}

impl Default for Data {
    fn default() -> Self {
        Self::synthetic()
    }
}

#[cfg(test)]
mod tests;
