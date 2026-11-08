//! Copia para `OUT_DIR/harness` só o conteúdo distribuível de `packages/harness` (RN-01),
//! que `src/embedded.rs` embute no binário. Com a feature `painel`, gera o bundle de `apps/painel`
//! quando pode, reconstrói ou recusa um bundle mais antigo que o código e liga o cfg
//! `painel_bundle` só com ele pronto.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;

/// Pastas e arquivos de primeiro nível que fazem parte do harness distribuído.
const TOP_LEVEL: &[&str] = &[
    ".gitignore",
    "config",
    "context",
    "personas",
    "rules",
    "skills",
    "templates",
];

/// O que o bundle do painel compila: um bundle mais antigo que algum deles está desatualizado.
const PAINEL_WATCHED: &[&str] = &["src", "index.html", "package.json"];

/// Categorias de regras de projeto que acompanham o harness; as demais são do projeto que o usa.
const DEFAULT_PROJECT_RULES: &[&str] = &["language.md", "naming-convention.md"];

fn main() {
    let manifest_dir = PathBuf::from(env("CARGO_MANIFEST_DIR"));
    let source = manifest_dir.join("../../packages/harness");
    let target = PathBuf::from(env("OUT_DIR")).join("harness");

    println!("cargo:rerun-if-changed={}", source.display());
    if target.exists() {
        fs::remove_dir_all(&target).expect("limpar a cópia anterior do harness");
    }
    for entry in TOP_LEVEL {
        let from = source.join(entry);
        if from.exists() {
            copy(&from, &target.join(entry), &source).expect("copiar o harness distribuível");
        }
    }
    for placeholder in ["memory/.gitkeep", "skills/custom/.gitkeep"] {
        write(&target.join(placeholder), "");
    }
    neutralize_sequences(&target.join("config/sequences.yml"));
    neutralize_context(&target.join("config/context.yml"), &target);
    prepare_painel_bundle(&manifest_dir.join("../painel"));
}

/// Sem Node e sem `dist`, o `vostok` compila sem o bundle e `vostok painel` explica como obtê-lo
/// (RF-12 da SPEC-029). Com um `dist` mais antigo que o código, reconstrói ou recusa o build, para
/// o binário nunca embutir telas velhas (RF-08 da SPEC-034).
fn prepare_painel_bundle(painel: &Path) {
    println!("cargo::rustc-check-cfg=cfg(painel_bundle)");
    if std::env::var_os("CARGO_FEATURE_PAINEL").is_none() {
        return;
    }
    for watched in PAINEL_WATCHED {
        println!("cargo:rerun-if-changed={}", painel.join(watched).display());
    }
    let index = painel.join("dist/index.html");
    if !index.is_file() {
        if let Err(reason) = build_painel(painel) {
            println!("cargo:warning=painel sem bundle: {reason}");
        }
    } else if bundle_is_outdated(painel, &index) {
        let rebuilt = build_painel(painel).and_then(|()| {
            index
                .is_file()
                .then_some(())
                .ok_or_else(|| "a reconstrução não gerou dist/index.html".to_string())
        });
        if let Err(reason) = rebuilt {
            panic!(
                "o bundle do painel em {} é mais antigo que o código de apps/painel e não foi reconstruído ({reason}); rode `pnpm --dir apps/painel build` e compile de novo",
                painel.join("dist").display()
            );
        }
    }
    if index.is_file() {
        println!("cargo:rustc-cfg=painel_bundle");
    }
}

/// Um `dist/index.html` que não dá para datar conta como em dia: na dúvida, o build não para.
fn bundle_is_outdated(painel: &Path, index: &Path) -> bool {
    let Ok(built) = fs::metadata(index).and_then(|metadata| metadata.modified()) else {
        return false;
    };
    let mut newest = None;
    for watched in PAINEL_WATCHED {
        newest_modification(&painel.join(watched), &mut newest);
    }
    newest.is_some_and(|source| built < source)
}

/// A data da própria pasta conta: apagar um arquivo de `src` também deixa o bundle velho.
fn newest_modification(path: &Path, newest: &mut Option<SystemTime>) {
    let Ok(metadata) = fs::metadata(path) else {
        return;
    };
    if let Ok(modified) = metadata.modified() {
        if newest.is_none_or(|current| modified > current) {
            *newest = Some(modified);
        }
    }
    if metadata.is_dir() {
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                newest_modification(&entry.path(), newest);
            }
        }
    }
}

fn build_painel(painel: &Path) -> Result<(), String> {
    if !painel.join("package.json").is_file() {
        return Err(format!("{} não tem package.json", painel.display()));
    }
    for args in [&["install", "--frozen-lockfile"][..], &["build"][..]] {
        let status = Command::new("pnpm")
            .arg("--dir")
            .arg(painel)
            .args(args)
            .stdout(std::io::stderr())
            .status()
            .map_err(|error| format!("pnpm indisponível ({error})"))?;
        if !status.success() {
            return Err(format!("`pnpm {}` falhou ({status})", args.join(" ")));
        }
    }
    Ok(())
}

fn env(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("variável {name} ausente no build"))
}

fn copy(from: &Path, to: &Path, root: &Path) -> io::Result<()> {
    if !is_distributable(from, root) {
        return Ok(());
    }
    if from.is_dir() {
        fs::create_dir_all(to)?;
        for entry in fs::read_dir(from)? {
            let entry = entry?;
            copy(&entry.path(), &to.join(entry.file_name()), root)?;
        }
        return Ok(());
    }
    if let Some(parent) = to.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(from, to).map(|_| ())
}

fn is_distributable(path: &Path, root: &Path) -> bool {
    let relative = path.strip_prefix(root).unwrap_or(path);
    let parts: Vec<String> = relative
        .components()
        .map(|part| part.as_os_str().to_string_lossy().into_owned())
        .collect();
    if parts.iter().any(|part| part == "scripts") {
        return false;
    }
    match parts.as_slice() {
        [first, second, ..] if first == "skills" => second == "default",
        [first, second, name] if first == "rules" && second == "project" => {
            DEFAULT_PROJECT_RULES.contains(&name.as_str())
        }
        _ => true,
    }
}

/// As sequências de IDs recomeçam em 1 num projeto novo.
fn neutralize_sequences(path: &Path) {
    let Ok(content) = fs::read_to_string(path) else {
        return;
    };
    let lines: Vec<String> = content
        .lines()
        .map(|line| match line.split_once(':') {
            Some((key, _))
                if !line.starts_with('#') && !line.starts_with(' ') && !key.is_empty() =>
            {
                format!("{key}: 1")
            }
            _ => line.to_string(),
        })
        .collect();
    write(path, &(lines.join("\n") + "\n"));
}

/// Tira do mapa de contexto as regras de projeto que não acompanham o harness.
fn neutralize_context(path: &Path, harness: &Path) {
    let Ok(content) = fs::read_to_string(path) else {
        return;
    };
    let kept: Vec<&str> = content
        .lines()
        .filter(|line| {
            let item = line.trim_start();
            match item.strip_prefix("- rules/project/") {
                Some(rest) => {
                    let name = rest.split_whitespace().next().unwrap_or(rest);
                    harness.join("rules/project").join(name).exists()
                }
                None => true,
            }
        })
        .collect();
    let mut result = Vec::new();
    for (index, line) in kept.iter().enumerate() {
        let is_empty_entry = line.starts_with("  ")
            && !line.starts_with("    ")
            && line.trim_end().ends_with(':')
            && !kept
                .get(index + 1)
                .is_some_and(|next| next.starts_with("    - "));
        if !is_empty_entry {
            result.push(*line);
        }
    }
    write(path, &(result.join("\n") + "\n"));
}

fn write(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("criar pasta no OUT_DIR");
    }
    fs::write(path, content).expect("gravar no OUT_DIR");
}
