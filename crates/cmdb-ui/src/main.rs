pub use makepad_widgets;

use makepad_widgets::*;

mod app;
mod chrome;
mod cli;
mod dag;
mod data;
use cmdb_ui::frame_harness;
mod frame_time;
mod graph_view;
mod table;
mod templates;
mod ui;
mod worker;

use crate::app::App;
use crate::cli::Command;

app_main!(App, configure: configure_startup);

fn configure_startup(_cx: &mut Cx) {
    match cli::from_process_args() {
        Ok(Command::Run(config)) => cli::set_startup_config(config),
        Ok(Command::Help) => {
            print!("{}", cli::USAGE);
            std::process::exit(0);
        }
        Err(error) => {
            eprint!("cmdb-ui: {error}\n\n{}", cli::USAGE);
            std::process::exit(2);
        }
    }
}
