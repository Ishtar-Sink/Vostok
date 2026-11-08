//! `vostok painel`: abre o painel local do harness da pasta, só em `127.0.0.1`.

use std::path::PathBuf;

use anyhow::Result;
use clap::Args;

#[derive(Debug, Args)]
pub struct PainelArgs {
    /// Porta de escuta em 127.0.0.1; 0 escolhe uma livre.
    #[arg(long, default_value_t = 0)]
    pub porta: u16,
    /// Pasta do projeto; o padrão é a pasta atual.
    #[arg(long)]
    pub pasta: Option<PathBuf>,
    /// Só imprime a URL, sem abrir o navegador.
    #[arg(long)]
    pub sem_navegador: bool,
}

#[cfg(not(feature = "painel"))]
pub fn run(_args: PainelArgs) -> Result<()> {
    anyhow::bail!(
        "este vostok foi compilado sem o painel; instale de novo com \
         `cargo install --path apps/cli --features painel`, com Node e pnpm no PATH para gerar o bundle"
    )
}

#[cfg(feature = "painel")]
pub fn run(args: PainelArgs) -> Result<()> {
    use std::fs;

    use anyhow::{anyhow, Context};

    use crate::harness::Harness;
    use crate::painel::{server, Painel};

    let start = match &args.pasta {
        Some(pasta) => fs::canonicalize(pasta)
            .with_context(|| format!("pasta inválida: {}", pasta.display()))?,
        None => crate::support::current_dir()?,
    };
    let harness = Harness::locate(&start).map_err(|_| {
        anyhow!(
            "nenhum vostok/config/main.yml encontrado em {} nem nas pastas acima; \
             abra o painel na pasta de um projeto com o harness instalado (`vostok init`)",
            start.display()
        )
    })?;
    let painel = Painel::new(&harness)?;
    if !server::HAS_BUNDLE {
        eprintln!("{}", server::BUNDLE_MISSING);
    }
    let project = painel.project.clone();
    server::serve(painel, args.porta, |url| {
        println!("Painel de {} em {url}", project.display());
        if !args.sem_navegador {
            open_browser(&url);
        }
    })
}

#[cfg(feature = "painel")]
fn open_browser(url: &str) {
    use std::process::Command;

    let mut command = if cfg!(target_os = "macos") {
        Command::new("open")
    } else if cfg!(windows) {
        let mut start = Command::new("cmd");
        start.args(["/c", "start", ""]);
        start
    } else {
        Command::new("xdg-open")
    };
    if command.arg(url).spawn().is_err() {
        eprintln!("Não foi possível abrir o navegador; acesse {url}");
    }
}
