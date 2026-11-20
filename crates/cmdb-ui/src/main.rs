pub use makepad_widgets;

use makepad_widgets::*;

mod app;
mod chrome;
mod cli;
mod dag;
mod data;
mod graph_view;
mod table;
mod templates;
mod ui;
mod worker;

use crate::app::App;
use crate::cli::Command;

app_main!(App, configure: configure_startup);

/// Reads the command line at startup, before any window is shown. `--help` prints usage and
/// exits 0. A bad command line prints the error with usage and exits 2. Both exit the process
/// here: a panic would be contained by the framework, which would keep the window open.
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
