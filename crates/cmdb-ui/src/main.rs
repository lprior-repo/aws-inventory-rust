//! Native Makepad viewer for the synthetic CMDB fixture.

pub use makepad_widgets;

use makepad_widgets::*;

mod app;
mod chrome;
mod dag;
mod data;
mod graph_view;
mod table;
mod templates;
mod ui;

use crate::app::App;

// No #![deny(missing_docs)]: app_main! (makepad 506ec05) expands to an undocumented pub fn app_main here.
app_main!(App);
