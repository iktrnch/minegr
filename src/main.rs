use std::io::{self, IsTerminal};
use std::process::ExitCode;

use clap::Parser;
use minegr::cli::Cli;
use minegr::ui::Ui;

fn main() -> ExitCode {
    let cli = Cli::parse();
    let ui = Ui::from_process(io::stdin().is_terminal(), io::stderr().is_terminal());

    match minegr::dispatch(cli, &ui) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            ui.error(&error.to_string());
            ExitCode::from(error.exit_code())
        }
    }
}
