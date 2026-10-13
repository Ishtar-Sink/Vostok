use std::process::ExitCode;

use clap::Parser;

use vostok::cli::{self, Cli};

fn main() -> ExitCode {
    match cli::run(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Erro: {error:#}");
            ExitCode::FAILURE
        }
    }
}
