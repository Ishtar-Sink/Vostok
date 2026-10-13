use std::process::ExitCode;

use clap::Parser;

use vostok::cli::{self, Cli};

fn main() -> ExitCode {
    restore_default_sigpipe();
    match cli::run(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Erro: {error:#}");
            ExitCode::FAILURE
        }
    }
}

/// Com a saída fechada (ex.: `vostok context load | head`), encerra em silêncio como as ferramentas de
/// linha de comando, em vez do pânico do `println!`.
#[cfg(unix)]
fn restore_default_sigpipe() {
    // SAFETY: troca o tratamento de um sinal antes de qualquer outra thread existir.
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }
}

#[cfg(not(unix))]
fn restore_default_sigpipe() {}
