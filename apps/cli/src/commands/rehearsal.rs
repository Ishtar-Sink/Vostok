//! Ensaio das tarefas antes da aprovação: conferências, tabela de achados e pasta `ensaio/` (SPEC-077).

use std::fs;
use std::path::Path;

use anyhow::{bail, Context, Result};
use serde_json::{json, Value};

use crate::support::{config_get, containing_line, snippets_of, unsearchable};

pub const DISABLED: &str = "Ensaio desligado para task.";
pub const NO_FINDINGS: &str = "Nenhum achado.";
pub const DIR: &str = "ensaio";
pub const RECORD_FILE: &str = "ensaio.yml";
pub const FINDINGS_FILE: &str = "achados.md";
pub const BASELINE: &str = "baseline";
pub const EVIDENCE: &str = "evidencia";
pub const VERIFICATION: &str = "verificacao";

pub const FALHA_PROBLEM: &str = "Texto de Falha aparece na saída com código 0";
pub const FALHA_FIX: &str = "Trocar o texto da Falha por um que só aparece quando a tarefa falha";
pub const EXIT_PROBLEM: &str = "Comando termina com código diferente de 0";
pub const EXIT_FIX: &str =
    "Conferir se o erro é esperado antes da mudança; senão, corrigir o comando";
pub const PASSES_PROBLEM: &str = "Comando da Verificação já passa antes da mudança";
pub const PASSES_FIX: &str = "Conferir se o comando e o filtro provam a mudança";
pub const DURATION_PROBLEM: &str = "Saída do Baseline com duração ou horário";
pub const DURATION_FIX: &str =
    "Filtrar a linha no comando ou acrescentar o trecho a regression_ignore";
pub const PARALLEL_PROBLEM: &str = "Lista de testes paralelos sem ordenação";
pub const PARALLEL_FIX: &str = "Acrescentar | sort ao comando";
pub const DELIVERY_PROBLEM: &str = "Baseline cita a pasta de um caminho criar";
pub const DELIVERY_FIX: &str =
    "Trocar o comando por um que não liste a pasta ou filtrar o caminho novo";
pub const ALTER_PROBLEM: &str = "Caminho alterar não existe";
pub const ALTER_FIX: &str = "Corrigir o caminho ou trocar a ação para criar";
pub const NOT_RUN_PROBLEM: &str = "Evidência não rodada: caminho ainda não existe";
pub const NOT_RUN_FIX: &str =
    "Conferir se o caminho é criado por esta tarefa ou por uma dependência";

/// Tipo do achado: defeito certo ou ponto a conferir.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Defect,
    ToCheck,
}

impl Kind {
    pub fn as_str(self) -> &'static str {
        match self {
            Kind::Defect => "defeito",
            Kind::ToCheck => "a conferir",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub task: String,
    pub place: String,
    pub kind: Kind,
    pub problem: String,
    pub proof: String,
    pub fix: String,
}

fn finding(
    task: &str,
    place: &str,
    kind: Kind,
    problem: &str,
    proof: String,
    fix: &str,
) -> Finding {
    Finding {
        task: task.to_string(),
        place: place.to_string(),
        kind,
        problem: problem.to_string(),
        proof,
        fix: fix.to_string(),
    }
}

/// Chave `rehearsal.task` de `config/main.yml`: ausente ou vazia vale ligado; true liga; false
/// desliga; outro valor é recusado.
pub fn enabled(main_yml: &Path) -> Result<bool> {
    let key = "rehearsal.task";
    match config_get(main_yml, key) {
        Err(_) => Ok(true),
        Ok(value) => match value.as_str() {
            "" | "true" => Ok(true),
            "false" => Ok(false),
            other => {
                bail!("valor inválido em config/main.yml ({key}): '{other}'; use true ou false")
            }
        },
    }
}

fn cell(text: &str) -> String {
    text.replace('|', "\\|")
        .replace("\r\n", " / ")
        .replace(['\n', '\r'], " / ")
}

/// Tabela dos achados, numerada de 1, ou `Nenhum achado.`; sempre termina com quebra de linha.
pub fn table(findings: &[Finding]) -> String {
    if findings.is_empty() {
        return format!("{NO_FINDINGS}\n");
    }
    let mut text = String::from(
        "| # | Tarefa | Local | Tipo | Problema | Prova | Correção sugerida |\n|---|---|---|---|---|---|---|\n",
    );
    for (index, item) in findings.iter().enumerate() {
        text.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} |\n",
            index + 1,
            cell(&item.task),
            cell(&item.place),
            item.kind.as_str(),
            cell(&item.problem),
            cell(&item.proof),
            cell(&item.fix),
        ));
    }
    text
}

/// Texto de Falha que aparece numa saída com código 0 (RF-04).
pub fn falha_in_output(
    task: &str,
    place: &str,
    falha: &str,
    output: &[u8],
    exit: i32,
) -> Option<Finding> {
    if exit != 0 {
        return None;
    }
    let text = String::from_utf8_lossy(output);
    snippets_of(falha)
        .into_iter()
        .filter(|snippet| unsearchable(snippet).is_empty())
        .find_map(|snippet| {
            containing_line(&text, &snippet).map(|(_, line)| {
                finding(
                    task,
                    place,
                    Kind::Defect,
                    FALHA_PROBLEM,
                    format!("`{snippet}` na linha: {line}"),
                    FALHA_FIX,
                )
            })
        })
}

/// Comando que termina com código diferente de 0 (RF-05, CB-01).
pub fn failed_exit(task: &str, place: &str, exit: i32, output: &[u8]) -> Option<Finding> {
    if exit == 0 {
        return None;
    }
    let text = String::from_utf8_lossy(output);
    let lines: Vec<&str> = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect();
    let tail = lines[lines.len().saturating_sub(3)..].join(" / ");
    let end = if tail.is_empty() {
        "(sem saída)".to_string()
    } else {
        tail
    };
    Some(finding(
        task,
        place,
        Kind::ToCheck,
        EXIT_PROBLEM,
        format!("exit: {exit}; fim da saída: {end}"),
        EXIT_FIX,
    ))
}

/// Comando da Verificação que já passa antes da mudança (RF-06).
pub fn already_passes(task: &str, place: &str, exit: i32) -> Option<Finding> {
    (exit == 0).then(|| {
        finding(
            task,
            place,
            Kind::ToCheck,
            PASSES_PROBLEM,
            "exit: 0 antes da mudança".to_string(),
            PASSES_FIX,
        )
    })
}

/// Saída do Baseline com duração ou horário, ou lista de testes sem ordenação (RF-07).
pub fn unstable_baseline(
    task: &str,
    place: &str,
    command: &str,
    output: &[u8],
    ignore: &[String],
) -> Vec<Finding> {
    let text = String::from_utf8_lossy(output);
    let mut found = Vec::new();
    if let Some(line) = text.lines().find(|line| {
        !ignore.iter().any(|item| line.contains(item.as_str())) && duration_or_time(line)
    }) {
        found.push(finding(
            task,
            place,
            Kind::Defect,
            DURATION_PROBLEM,
            format!("linha: {}", line.trim()),
            DURATION_FIX,
        ));
    }
    let listed = text
        .lines()
        .filter(|line| line.starts_with("test ") && line.contains(" ... "))
        .count();
    let sorted = command
        .split(|c: char| !c.is_alphanumeric())
        .any(|word| word == "sort");
    if listed >= 2 && !sorted && !command.contains("--test-threads=1") {
        found.push(finding(
            task,
            place,
            Kind::Defect,
            PARALLEL_PROBLEM,
            format!("sem `sort` no comando; saída com {listed} linhas `test ...`"),
            PARALLEL_FIX,
        ));
    }
    found
}

/// Horário `H:MM:SS` ou número colado a uma unidade de tempo.
fn duration_or_time(line: &str) -> bool {
    let chars: Vec<char> = line.chars().collect();
    let digit = |index: usize| chars.get(index).is_some_and(char::is_ascii_digit);
    let is_alnum = |index: usize| chars.get(index).is_some_and(|c| c.is_alphanumeric());
    for start in 0..chars.len() {
        if !digit(start) || (start > 0 && is_alnum(start - 1)) {
            continue;
        }
        // horário
        let mut at = start;
        while digit(at) {
            at += 1;
        }
        if (1..=2).contains(&(at - start))
            && chars.get(at) == Some(&':')
            && digit(at + 1)
            && digit(at + 2)
            && chars.get(at + 3) == Some(&':')
            && digit(at + 4)
            && digit(at + 5)
            && !digit(at + 6)
        {
            return true;
        }
        // número com unidade
        let mut end = at;
        if matches!(chars.get(end), Some('.') | Some(',')) && digit(end + 1) {
            end += 1;
            while digit(end) {
                end += 1;
            }
        }
        let rest: String = chars[end..].iter().collect();
        for unit in ["min", "ms", "ns", "us", "µs", "s", "h"] {
            if let Some(after) = rest.strip_prefix(unit) {
                if !after.chars().next().is_some_and(char::is_alphanumeric) {
                    return true;
                }
            }
        }
    }
    false
}

/// Baseline que cita a pasta onde a tarefa cria um arquivo (RF-08).
pub fn measures_delivery(
    task: &str,
    place: &str,
    command: &str,
    created: &[String],
    existing_folders: &[String],
) -> Vec<Finding> {
    existing_folders
        .iter()
        .filter(|folder| super::task::cites_path(command, folder))
        .filter_map(|folder| {
            let prefix = format!("{folder}/");
            created
                .iter()
                .find(|path| path.starts_with(&prefix))
                .map(|path| {
                    finding(
                        task,
                        place,
                        Kind::Defect,
                        DELIVERY_PROBLEM,
                        format!("`{folder}` é pasta de `{path}`"),
                        DELIVERY_FIX,
                    )
                })
        })
        .collect()
}

/// Caminho `alterar` que não existe na raiz do projeto nem é criado por outra tarefa (RF-09).
pub fn missing_alter(
    task: &str,
    root: &Path,
    changes: &[(String, String)],
    created_elsewhere: &[String],
) -> Vec<Finding> {
    changes
        .iter()
        .filter(|(action, path)| {
            action == "alterar" && !root.join(path).exists() && !created_elsewhere.contains(path)
        })
        .map(|(_, path)| {
            finding(
                task,
                "Mudar",
                Kind::Defect,
                ALTER_PROBLEM,
                format!("`{path}` não existe na raiz do projeto"),
                ALTER_FIX,
            )
        })
        .collect()
}

/// Evidência que não rodou porque um caminho de `Requer` ou `Depende` ainda não existe (CB-04).
pub fn not_run(task: &str, place: &str, path: &str) -> Finding {
    finding(
        task,
        place,
        Kind::ToCheck,
        NOT_RUN_PROBLEM,
        format!("`{path}` ainda não existe"),
        NOT_RUN_FIX,
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recorded {
    pub kind: String,
    pub name: String,
    pub command: String,
    pub key: String,
    pub exit_code: i32,
    pub file: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Record {
    pub task: String,
    pub tree_fingerprint: String,
    pub date: String,
    pub commands: Vec<Recorded>,
}

impl Record {
    pub fn find(&self, kind: &str, name: &str, command: &str) -> Option<&Recorded> {
        self.commands
            .iter()
            .find(|item| item.kind == kind && item.name == name && item.command == command)
    }
}

/// Recria a pasta do ensaio com as saídas, o registro e os achados da tarefa (CB-05).
pub fn write(
    dir: &Path,
    record: &Record,
    outputs: &[(String, Vec<u8>)],
    findings: &str,
) -> Result<()> {
    if dir.exists() {
        fs::remove_dir_all(dir)
            .with_context(|| format!("não foi possível apagar {}", dir.display()))?;
    }
    fs::create_dir_all(dir)?;
    for (relative, bytes) in outputs {
        let path = dir.join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, bytes)
            .with_context(|| format!("não foi possível gravar {}", path.display()))?;
    }
    let commands: Vec<Value> = record
        .commands
        .iter()
        .map(|item| {
            json!({
                "kind": item.kind,
                "name": item.name,
                "command": item.command,
                "key": item.key,
                "exit_code": item.exit_code,
                "file": item.file,
            })
        })
        .collect();
    let value = json!({
        "task": record.task,
        "tree_fingerprint": record.tree_fingerprint,
        "date": record.date,
        "commands": commands,
    });
    fs::write(
        dir.join(RECORD_FILE),
        format!("{}\n", serde_json::to_string_pretty(&value)?),
    )?;
    fs::write(dir.join(FINDINGS_FILE), findings)?;
    Ok(())
}

/// Registro gravado; `None` sem o arquivo ou com JSON inválido.
pub fn read(dir: &Path) -> Option<Record> {
    let text = fs::read_to_string(dir.join(RECORD_FILE)).ok()?;
    let value: Value = serde_json::from_str(&text).ok()?;
    let text_of = |value: &Value, key: &str| value.get(key)?.as_str().map(str::to_string);
    let commands = value
        .get("commands")?
        .as_array()?
        .iter()
        .map(|item| {
            Some(Recorded {
                kind: text_of(item, "kind")?,
                name: text_of(item, "name")?,
                command: text_of(item, "command")?,
                key: text_of(item, "key")?,
                exit_code: i32::try_from(item.get("exit_code")?.as_i64()?).ok()?,
                file: text_of(item, "file")?,
            })
        })
        .collect::<Option<Vec<_>>>()?;
    Some(Record {
        task: text_of(&value, "task")?,
        tree_fingerprint: text_of(&value, "tree_fingerprint")?,
        date: text_of(&value, "date")?,
        commands,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duracao_e_horario_detectados() {
        for line in [
            "took 12ms",
            "Done in 1.52s",
            "[14:08:02] pronto",
            "elapsed 0,8s",
        ] {
            assert!(duration_or_time(line), "{line}");
        }
    }

    #[test]
    fn numeros_sem_unidade_nao_contam() {
        for line in [
            "3 passed",
            "versão 1.2.3",
            "abc12s",
            "12s3",
            "test a ... ok",
        ] {
            assert!(!duration_or_time(line), "{line}");
        }
    }

    #[test]
    fn tabela_escapa_a_barra() {
        let item = finding(
            "TASK-01",
            "Mudar",
            Kind::Defect,
            "a | b",
            "x\ny".to_string(),
            "fix",
        );
        let text = table(&[item]);
        assert!(text.contains("| 1 | TASK-01 | Mudar | defeito | a \\| b | x / y | fix |"));
        assert_eq!(table(&[]), "Nenhum achado.\n");
    }

    #[test]
    fn registro_grava_e_le() {
        let dir = tempfile::TempDir::new().unwrap();
        let target = dir.path().join("ensaio");
        let record = Record {
            task: "TASK-01".into(),
            tree_fingerprint: "abc".into(),
            date: "2026-11-19T00:00:00Z".into(),
            commands: vec![Recorded {
                kind: BASELINE.into(),
                name: "01".into(),
                command: "echo a".into(),
                key: "k".into(),
                exit_code: 0,
                file: "baseline/01.out".into(),
            }],
        };
        let outputs = vec![("baseline/01.out".to_string(), b"saida".to_vec())];
        write(&target, &record, &outputs, "Nenhum achado.\n").unwrap();
        assert_eq!(read(&target), Some(record.clone()));
        assert_eq!(fs::read(target.join("baseline/01.out")).unwrap(), b"saida");
        // regravar apaga o que havia
        write(&target, &record, &[], "x").unwrap();
        assert!(!target.join("baseline").exists());
        assert_eq!(read(&dir.path().join("nada")), None);
    }
}
