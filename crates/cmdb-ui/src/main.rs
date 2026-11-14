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

app_main!(App);
