//! Comandos `commit`: commits liberados pelo usuário ([agent/boundaries-04]).
//!
//! A referência é a foto das refs locais (HEAD, branches e tags, sem `refs/remotes/` nem `refs/stash`)
//! no momento da última liberação, gravada em `paths.state`/commit-ack, fora do git. Commit novo é o
//! alcançável hoje e não alcançável pela foto; commit removido, o contrário.

use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

use anyhow::{anyhow, bail, Context, Result};
use clap::Subcommand;

use crate::harness::Harness;
use crate::support::sha256_hex;

const CHECK_USAGE: &str = "Uso: vostok commit check";
const USAGE: &str =
    "Uso: vostok commit show | approve <código> | commit [caminho...] < mensagem | revert <código>";
const STOP: &str = "PARE: responda com o template `blocker`, mostre os commits e pergunte ao usuário se os libera ou desfaz (skill approve-commit). Nunca libere nem desfaça commits por conta própria.";
const NOTHING_PENDING: &str = "Nenhum commit sem liberação.";
const TOKEN_LENGTH: usize = 12;

#[derive(Debug, Subcommand)]
pub enum CommitCommand {
    /// Recusa se houver commits sem liberação.
    Check {
        #[arg(hide = true)]
        extra: Vec<String>,
    },
    /// Lista os commits sem liberação e o código de liberação.
    Show,
    /// Libera os commits apontados.
    Approve { code: Option<String> },
    /// Desfaz os commits novos da branch atual.
    Revert { code: Option<String> },
    /// Faz um commit já liberado, com a mensagem pela entrada padrão.
    Commit { paths: Vec<String> },
}

pub fn run(command: CommitCommand) -> Result<()> {
    if let CommitCommand::Check { extra } = &command {
        if !extra.is_empty() {
            bail!(CHECK_USAGE);
        }
    }
    let repo = Repo::open(&Harness::from_current_dir()?)?;
    match command {
        CommitCommand::Check { .. } => check(&repo),
        CommitCommand::Show => repo.require_reference().and_then(|()| show(&repo)),
        CommitCommand::Approve { code } => repo
            .require_reference()
            .and_then(|()| approve(&repo, code.as_deref())),
        CommitCommand::Revert { code } => repo
            .require_reference()
            .and_then(|()| revert(&repo, code.as_deref())),
        CommitCommand::Commit { paths } => repo
            .require_reference()
            .and_then(|()| commit(&repo, &paths)),
    }
}

fn check(repo: &Repo) -> Result<()> {
    if !repo.has_git() {
        println!(
            "Sem repositório git em {}; nada a verificar.",
            repo.root.display()
        );
        return Ok(());
    }
    if !repo.ack.is_file() {
        repo.store()?;
        println!(
            "Referência de commits gravada na primeira verificação: {}",
            repo.ack.display()
        );
        return Ok(());
    }
    let errors = repo.ack_errors()?;
    if !errors.is_empty() {
        bail!(
            "a referência de commits liberados é inválida ({}):\n{}\n{STOP}",
            repo.ack.display(),
            errors.join("\n")
        );
    }
    let report = repo.report(&repo.changes()?)?;
    if !report.is_empty() {
        bail!(
            "há commits não liberados pelo usuário ([agent/boundaries-04]):\n{}\n{STOP}",
            report.join("\n")
        );
    }
    println!("{NOTHING_PENDING}");
    Ok(())
}

fn show(repo: &Repo) -> Result<()> {
    let errors = repo.ack_errors()?;
    if !errors.is_empty() {
        println!("Referência de commits inválida:");
        print_lines(&errors);
    } else {
        let report = repo.report(&repo.changes()?)?;
        if report.is_empty() {
            println!("{NOTHING_PENDING}");
        } else {
            print_lines(&report);
        }
    }
    println!("Código de liberação: {}", repo.token()?);
    Ok(())
}

fn approve(repo: &Repo, code: Option<&str>) -> Result<()> {
    repo.require_token(code)?;
    let errors = repo.ack_errors()?;
    let count = if errors.is_empty() {
        repo.changes()?.len()
    } else {
        0
    };
    if errors.is_empty() && count == 0 {
        println!("{NOTHING_PENDING}");
        return Ok(());
    }
    repo.store()?;
    if errors.is_empty() {
        println!("Liberação gravada: {count} commit(s) novo(s) ou removido(s).");
    } else {
        println!("Referência de commits refeita a partir do estado atual.");
    }
    Ok(())
}

fn commit(repo: &Repo, paths: &[String]) -> Result<()> {
    if !repo.checked_changes()?.is_empty() {
        bail!("já há commits sem liberação; mostre-os ao usuário e libere ou desfaça antes de um commit novo");
    }
    let message = read_message()?;
    let before = repo.query(&["rev-parse", "-q", "--verify", "HEAD"])?;
    if paths.is_empty() {
        repo.commit(&message, &[])?;
    } else {
        // O `add` antes inclui arquivos novos e removidos; o commit com caminhos deixa o resto do stage como está.
        let mut add = vec!["add", "-A", "--"];
        add.extend(paths.iter().map(String::as_str));
        if !repo.run(&add)? {
            bail!("caminho inválido; nada foi commitado");
        }
        repo.commit(&message, paths)?;
    }
    let head = repo.query(&["rev-parse", "HEAD"])?;
    let parent = repo.query(&["rev-parse", "-q", "--verify", "HEAD^"])?;
    let single_new = matches!(repo.changes()?.as_slice(), [Change::New(hash)] if *hash == head);
    if !single_new || parent != before {
        bail!("o estado do repositório não corresponde a um único commit novo; nada foi liberado");
    }
    repo.store()?;
    println!(
        "Commit feito e liberado: {}",
        repo.query(&["log", "-1", "--format=%h %s"])?
    );
    Ok(())
}

fn read_message() -> Result<String> {
    let mut message = String::new();
    io::stdin()
        .read_to_string(&mut message)
        .context("não foi possível ler a mensagem de commit da entrada padrão")?;
    let message = message.trim_end_matches('\n').to_string();
    if message.chars().all(char::is_whitespace) {
        bail!("mensagem de commit vazia; passe-a pela entrada padrão");
    }
    Ok(message)
}

fn revert(repo: &Repo, code: Option<&str>) -> Result<()> {
    repo.require_token(code)?;
    let changes = repo.checked_changes()?;
    if changes.is_empty() {
        println!("{NOTHING_PENDING}");
        return Ok(());
    }
    if changes
        .iter()
        .any(|change| matches!(change, Change::Removed(_)))
    {
        bail!("há commits removidos; desfazer não os recupera. Peça ao usuário para decidir como seguir");
    }
    let branch = repo.query(&["symbolic-ref", "-q", "HEAD"])?;
    if branch.is_empty() {
        bail!("HEAD sem branch (detached); peça ao usuário para desfazer à mão");
    }
    let name = branch.strip_prefix("refs/heads/").unwrap_or(&branch);
    let base = repo.released_tip(&branch)?.ok_or_else(|| {
        anyhow!("a branch {name} não existia na liberação; peça ao usuário para desfazer à mão")
    })?;
    if !repo.succeeds(&["merge-base", "--is-ancestor", &base, "HEAD"])? {
        bail!("a branch {name} foi reescrita; peça ao usuário para desfazer à mão");
    }
    let news = sorted(
        changes
            .iter()
            .map(Change::hash)
            .map(str::to_string)
            .collect(),
    );
    let branch_news = sorted(repo.lines(&["rev-list", "HEAD", "--not", &base])?);
    if news != branch_news {
        bail!("há commits novos fora da branch {name}; peça ao usuário para desfazer à mão");
    }
    // Commits da branch que nenhum ref remoto contém; se faltar algum, ele já foi enviado.
    let unpushed = sorted(repo.lines(&["rev-list", "HEAD", "--not", &base, "--remotes"])?);
    if unpushed != branch_news {
        bail!("há commits já enviados ao remoto; desfazê-los exige reescrever o histórico compartilhado. Peça ao usuário para decidir");
    }
    if !repo.run(&["reset", "--mixed", "-q", &base])? {
        bail!("o git reset falhou; nada foi desfeito");
    }
    println!(
        "{} commit(s) desfeito(s) em {name}; as alterações continuam na pasta de trabalho, fora de stage.",
        news.len()
    );
    Ok(())
}

fn print_lines(lines: &[String]) {
    for line in lines {
        println!("{line}");
    }
}

fn sorted(mut values: Vec<String>) -> Vec<String> {
    values.sort();
    values
}

#[derive(Debug, PartialEq, Eq)]
enum Change {
    New(String),
    Removed(String),
}

impl Change {
    fn hash(&self) -> &str {
        match self {
            Change::New(hash) | Change::Removed(hash) => hash,
        }
    }
}

/// O repositório git da raiz do projeto e o arquivo com a foto das refs liberadas.
struct Repo {
    root: PathBuf,
    ack: PathBuf,
}

impl Repo {
    fn open(harness: &Harness) -> Result<Self> {
        let root = harness.path("root")?;
        let root = root
            .canonicalize()
            .with_context(|| format!("raiz do projeto inexistente: {}", root.display()))?;
        Ok(Self {
            root,
            ack: harness.path("state")?.join("commit-ack"),
        })
    }

    fn require_reference(&self) -> Result<()> {
        if !self.has_git() {
            bail!("sem repositório git em {}", self.root.display());
        }
        if !self.ack.is_file() {
            bail!("ainda não há referência de commits; execute vostok commit check");
        }
        Ok(())
    }

    fn git(&self, args: &[&str]) -> Command {
        let mut command = Command::new("git");
        command.arg("-C").arg(&self.root).args(args);
        command
    }

    fn has_git(&self) -> bool {
        self.git(&["rev-parse", "--git-dir"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|status| status.success())
    }

    /// Saída do git sem as quebras de linha do fim; vazia se o comando falhar.
    fn query(&self, args: &[&str]) -> Result<String> {
        let output = self
            .git(args)
            .stderr(Stdio::null())
            .output()
            .context("não foi possível executar o git")?;
        if !output.status.success() {
            return Ok(String::new());
        }
        Ok(String::from_utf8_lossy(&output.stdout)
            .trim_end_matches('\n')
            .to_string())
    }

    fn lines(&self, args: &[&str]) -> Result<Vec<String>> {
        let output = self
            .git(args)
            .output()
            .context("não foi possível executar o git")?;
        if !output.status.success() {
            bail!(
                "git {} falhou: {}",
                args.join(" "),
                String::from_utf8_lossy(&output.stderr).trim_end()
            );
        }
        Ok(String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter(|line| !line.is_empty())
            .map(str::to_string)
            .collect())
    }

    fn succeeds(&self, args: &[&str]) -> Result<bool> {
        let status = self
            .git(args)
            .stdout(Stdio::null())
            .status()
            .context("não foi possível executar o git")?;
        Ok(status.success())
    }

    fn is_commit(&self, hash: &str) -> Result<bool> {
        let status = self
            .git(&["cat-file", "-e", &format!("{hash}^{{commit}}")])
            .stderr(Stdio::null())
            .status()
            .context("não foi possível executar o git")?;
        Ok(status.success())
    }

    /// Executa o git com a saída à vista do usuário, como nos scripts.
    fn run(&self, args: &[&str]) -> Result<bool> {
        io::stdout().flush()?;
        let status = self
            .git(args)
            .stdin(Stdio::null())
            .status()
            .context("não foi possível executar o git")?;
        Ok(status.success())
    }

    fn commit(&self, message: &str, paths: &[String]) -> Result<()> {
        let mut args = vec!["commit", "-q", "-F", "-"];
        if !paths.is_empty() {
            args.push("--");
            args.extend(paths.iter().map(String::as_str));
        }
        io::stdout().flush()?;
        let mut child = self
            .git(&args)
            .stdin(Stdio::piped())
            .spawn()
            .context("não foi possível executar o git")?;
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(format!("{message}\n").as_bytes()).ok();
        }
        if !child.wait()?.success() {
            bail!("o commit falhou; nada foi liberado");
        }
        Ok(())
    }

    /// Foto atual: uma linha "hash ref" por ref que aponta (direto ou por tag) para um commit, ordenada pela ref.
    fn snapshot(&self) -> Result<String> {
        let mut entries = Vec::new();
        let head = self.query(&["rev-parse", "-q", "--verify", "HEAD^{commit}"])?;
        if !head.is_empty() {
            entries.push(("HEAD".to_string(), head));
        }
        let refs = self.query(&[
            "for-each-ref",
            "--format=%(objecttype) %(objectname) %(*objecttype) %(*objectname) %(refname)",
        ])?;
        entries.extend(refs.lines().filter_map(commit_ref));
        entries.sort_by(|(a_ref, a_hash), (b_ref, b_hash)| {
            a_ref
                .cmp(b_ref)
                .then_with(|| format!("{a_hash} {a_ref}").cmp(&format!("{b_hash} {b_ref}")))
        });
        Ok(entries
            .iter()
            .map(|(name, hash)| format!("{hash} {name}\n"))
            .collect())
    }

    /// Código curto que identifica a foto atual; a liberação exige o código mostrado na verificação.
    fn token(&self) -> Result<String> {
        let mut token = sha256_hex(self.snapshot()?.as_bytes());
        token.truncate(TOKEN_LENGTH);
        Ok(token)
    }

    fn require_token(&self, code: Option<&str>) -> Result<()> {
        let code = code
            .filter(|code| !code.is_empty())
            .ok_or_else(|| anyhow!(USAGE))?;
        let token = self.token()?;
        if code != token {
            bail!("o repositório mudou desde a verificação (código atual: {token}); mostre os commits ao usuário de novo");
        }
        Ok(())
    }

    fn read_ack(&self) -> Result<String> {
        fs::read_to_string(&self.ack)
            .with_context(|| format!("não foi possível ler {}", self.ack.display()))
    }

    /// Problemas da referência, um por linha: linha fora do formato ou commit inexistente.
    fn ack_errors(&self) -> Result<Vec<String>> {
        let mut errors = Vec::new();
        for line in self.read_ack()?.lines().filter(|line| !line.is_empty()) {
            match line.split_once(' ') {
                Some((hash, name)) if is_commit_hash(hash) && is_ref_name(name) => {
                    if !self.is_commit(hash)? {
                        errors.push(format!(
                            "commit liberado não existe mais no repositório: {line}"
                        ));
                    }
                }
                _ => errors.push(format!("linha fora do formato 'hash ref': {line}")),
            }
        }
        Ok(errors)
    }

    fn checked_changes(&self) -> Result<Vec<Change>> {
        let errors = self.ack_errors()?;
        if !errors.is_empty() {
            bail!(
                "a referência de commits é inválida; ela só pode ser refeita com vostok commit approve:\n{}",
                errors.join("\n")
            );
        }
        self.changes()
    }

    /// Commits novos e removidos em relação à referência, os novos primeiro.
    fn changes(&self) -> Result<Vec<Change>> {
        let current = tips(&self.snapshot()?);
        let released = tips(&self.read_ack()?);
        let mut changes: Vec<Change> = self
            .between(&current, &released)?
            .into_iter()
            .map(Change::New)
            .collect();
        changes.extend(
            self.between(&released, &current)?
                .into_iter()
                .map(Change::Removed),
        );
        Ok(changes)
    }

    /// Commits alcançáveis por `include` e não por `exclude`, do mais novo ao mais antigo.
    fn between(&self, include: &[String], exclude: &[String]) -> Result<Vec<String>> {
        if include.is_empty() {
            return Ok(Vec::new());
        }
        let mut args = vec!["rev-list", "--topo-order"];
        args.extend(include.iter().map(String::as_str));
        args.push("--not");
        args.extend(exclude.iter().map(String::as_str));
        self.lines(&args)
    }

    /// Descrição de um commit: hash curto, assunto, arquivos e se já está num ref remoto.
    fn describe(&self, hash: &str) -> Result<String> {
        let files = self
            .query(&[
                "log",
                "-1",
                "--format=",
                "--name-only",
                "-m",
                "--first-parent",
                hash,
            ])?
            .lines()
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join(", ");
        let files = if files.is_empty() { "-" } else { &files };
        let pushed = !self
            .query(&["branch", "-r", "--contains", hash])?
            .is_empty();
        let remote = if pushed {
            " [já enviado ao remoto]"
        } else {
            ""
        };
        let summary = self.query(&["log", "-1", "--format=%h %s", hash])?;
        Ok(format!("- {summary} (arquivos: {files}){remote}"))
    }

    /// Relatório dos commits não liberados; vazio se não houver nenhum.
    fn report(&self, changes: &[Change]) -> Result<Vec<String>> {
        let mut report = Vec::new();
        let news: Vec<&Change> = changes
            .iter()
            .filter(|change| matches!(change, Change::New(_)))
            .collect();
        let removed: Vec<&Change> = changes
            .iter()
            .filter(|change| matches!(change, Change::Removed(_)))
            .collect();
        if !news.is_empty() {
            report.push("Commits novos:".to_string());
            for change in news {
                report.push(self.describe(change.hash())?);
            }
        }
        if !removed.is_empty() {
            report.push("Commits removidos (estavam liberados e saíram do histórico):".to_string());
            for change in removed {
                report.push(self.describe(change.hash())?);
            }
        }
        Ok(report)
    }

    /// Ponta liberada de uma ref, pela foto gravada.
    fn released_tip(&self, name: &str) -> Result<Option<String>> {
        Ok(self.read_ack()?.lines().find_map(|line| {
            let mut fields = line.split_whitespace();
            let hash = fields.next()?;
            (fields.next()? == name).then(|| hash.to_string())
        }))
    }

    /// Grava a foto atual como liberada.
    fn store(&self) -> Result<()> {
        if let Some(parent) = self.ack.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("não foi possível criar {}", parent.display()))?;
        }
        let temporary = self.ack.with_extension("tmp");
        fs::write(&temporary, self.snapshot()?)
            .with_context(|| format!("não foi possível gravar {}", temporary.display()))?;
        fs::rename(&temporary, &self.ack)
            .with_context(|| format!("não foi possível gravar {}", self.ack.display()))
    }
}

/// `(ref, hash)` de uma linha do `for-each-ref` que aponta para um commit, direto ou por tag.
fn commit_ref(line: &str) -> Option<(String, String)> {
    let fields: Vec<&str> = line.split_whitespace().collect();
    let name = *fields.last()?;
    if name.starts_with("refs/remotes/") || name == "refs/stash" {
        return None;
    }
    let hash = match fields.as_slice() {
        ["commit", hash, ..] => hash,
        ["tag", _, "commit", hash, _] => hash,
        _ => return None,
    };
    Some((name.to_string(), hash.to_string()))
}

/// Pontas distintas de uma foto, ordenadas.
fn tips(snapshot: &str) -> Vec<String> {
    let mut tips: Vec<String> = snapshot
        .lines()
        .map(|line| line.split(' ').next().unwrap_or_default().to_string())
        .filter(|tip| !tip.is_empty())
        .collect();
    tips.sort();
    tips.dedup();
    tips
}

fn is_commit_hash(value: &str) -> bool {
    matches!(value.len(), 40 | 64)
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn is_ref_name(value: &str) -> bool {
    !value.is_empty() && !value.chars().any(char::is_whitespace)
}
