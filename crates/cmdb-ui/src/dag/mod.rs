//! Deterministic DAG view for the CMDB shell.
//!
//! - [`graph`] is the capped, deduplicated model.
//! - [`builder`] projects `crate::data::Graph` onto it.
//! - [`layout`] is a pure Sugiyama-style layered layout (cycle breaking,
//!   longest-path layers, barycenter ordering, bounded coordinates).
//! - [`widget`] is the Makepad renderer: one `DrawVector` batch plus text.
//!
//! Integration:
//! 1. `crate::dag::script_mod(vm);` in `AppMain::script_mod`.
//! 2. Place `DagView{ width: Fill height: Fill }` in the graph pane.
//! 3. `dag.set_graph(cx, &table.graph())` whenever the filter changes.

pub mod builder;
pub mod graph;
pub mod layout;
pub mod view;
pub mod widget;

#[cfg(test)]
mod tests;

/// Registers `mod.widgets.DagView` in the Splash widget catalogue.
pub fn script_mod(vm: &mut makepad_widgets::ScriptVm) {
    widget::script_mod(vm);
}
