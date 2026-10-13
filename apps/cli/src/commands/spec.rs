//! Comandos `spec`: specs: criação, refinamento, ajuste e aprovação.
//! Reproduzem `create-new-spec.sh`, `list-specs.sh`, `create-refined-spec.sh`, `adjust-spec.sh` e
//! `approve-spec.sh`, com as mesmas mensagens, recusas e códigos de saída (SPEC-005, RN-03).

use std::collections::HashMap;
use std::fs;
use std::io::{self, BufRead, IsTerminal, Write};
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{bail, Result};
use clap::Subcommand;

use crate::harness::Harness;
use crate::support::{
    clean_title, config_block, config_get, config_list, config_set, frontmatter_get,
    frontmatter_lines, frontmatter_set, markdown_cell, markdown_section, markdown_table_rows,
    render, sha256_hex, slugify, template_leftovers, yaml_escape,
};

const REFINE_USAGE: &str =
    "Uso: create-refined-spec.sh <SPEC_ID> <tipo> | --validate <SPEC_ID> | --mark-refined <SPEC_ID>";
const ADJUST_USAGE: &str =
    "Uso: adjust-spec.sh --record-manual <SPEC_ID> | --validate <SPEC_ID> | --record <SPEC_ID> \"resumo\"";
const COMPONENT_ACTIONS: &str = "criar alterar remover";

#[derive(Debug, Subcommand)]
pub enum SpecCommand {
    /// Cria uma spec em rascunho.
    New { title: Option<String> },
    /// Lista as specs, opcionalmente filtradas por status.
    List { status: Option<String> },
    /// Cria a spec refinada com as seções do tipo.
    Refine {
        #[arg(allow_hyphen_values = true)]
        spec_id: Option<String>,
        kind: Option<String>,
    },
    /// Valida a spec refinada.
    Validate { spec_id: String },
    /// Grava o hash da refinada e marca o rascunho como refinado.
    MarkRefined { spec_id: String },
    /// Ajustes de uma spec refinada.
    #[command(subcommand)]
    Adjust(AdjustCommand),
    /// Aprovação de uma spec refinada.
    #[command(subcommand)]
    Approve(ApproveCommand),
}

#[derive(Debug, Subcommand)]
pub enum AdjustCommand {
    /// Registra edições feitas à mão.
    RecordManual { spec_id: String },
    /// Valida, sem alterar nada.
    Validate { spec_id: String },
    /// Registra o ajuste no histórico.
    Record {
        spec_id: String,
        summary: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
pub enum ApproveCommand {
    /// Lista os impedimentos, sem alterar nada.
    Validate { spec_id: String },
    /// Aprova a spec.
    Approve { spec_id: String },
}

pub fn run(command: SpecCommand) -> Result<()> {
    let specs = Specs {
        harness: Harness::from_current_dir()?,
    };
    match command {
        SpecCommand::New { title } => create_new(&specs, title.unwrap_or_default()),
        SpecCommand::List { status } => list(&specs, status.as_deref()),
        SpecCommand::Refine { spec_id, kind } => refine(&specs, spec_id, kind),
        SpecCommand::Validate { spec_id } => validate_refined(&specs, &spec_id),
        SpecCommand::MarkRefined { spec_id } => mark_refined(&specs, &spec_id),
        SpecCommand::Adjust(AdjustCommand::RecordManual { spec_id }) => {
            record_manual(&specs, &spec_id)
        }
        SpecCommand::Adjust(AdjustCommand::Validate { spec_id }) => {
            validate_adjustable(&specs, &spec_id)
        }
        SpecCommand::Adjust(AdjustCommand::Record { spec_id, summary }) => match summary {
            Some(summary) => record(&specs, &spec_id, &summary),
            None => bail!("{ADJUST_USAGE}"),
        },
        SpecCommand::Approve(ApproveCommand::Validate { spec_id }) => {
            validate_approval(&specs, &spec_id)
        }
        SpecCommand::Approve(ApproveCommand::Approve { spec_id }) => approve(&specs, &spec_id),
    }
}

/// Caminhos e configurações das specs, lidos de `config/main.yml` e `config/specs.yml`.
struct Specs {
    harness: Harness,
}

impl Specs {
    fn config(&self) -> PathBuf {
        self.harness.config_file("specs.yml")
    }

    fn setting(&self, key: &str) -> Result<String> {
        config_get(&self.config(), key)
    }

    fn setting_block(&self, key: &str) -> Result<String> {
        config_block(&self.config(), key)
    }

    fn status(&self, name: &str) -> Result<String> {
        self.setting(&format!("status.{name}"))
    }

    fn dir(&self) -> Result<PathBuf> {
        self.harness.path("specs")
    }

    fn draft_dir(&self) -> Result<PathBuf> {
        Ok(self.dir()?.join(self.setting("draft_dir")?))
    }

    fn refined_dir(&self) -> Result<PathBuf> {
        Ok(self.dir()?.join(self.setting("refined_dir")?))
    }

    fn templates_dir(&self) -> Result<PathBuf> {
        self.harness.path("templates")
    }

    fn section_template(&self, section: &str) -> Result<PathBuf> {
        Ok(self
            .templates_dir()?
            .join(self.setting("sections_dir")?)
            .join(format!("{section}.md")))
    }

    /// Título da seção, lido da primeira linha do template (`## Título`).
    fn section_heading(&self, section: &str) -> Result<String> {
        let content = fs::read_to_string(self.section_template(section)?).unwrap_or_default();
        let first = content.split('\n').next().unwrap_or_default();
        Ok(first.strip_prefix("## ").unwrap_or(first).to_string())
    }

    fn history_heading(&self) -> Result<String> {
        self.section_heading(&self.setting("adjustment_history_section")?)
    }

    /// Diretório do projeto (`paths.root`), normalizado como o `cd ... && pwd` dos scripts.
    fn project_root(&self) -> Result<PathBuf> {
        Ok(normalize_lexically(&self.harness.path("root")?))
    }
}

// --- new ---

fn create_new(specs: &Specs, title: String) -> Result<()> {
    let title = if title.is_empty() && io::stdin().is_terminal() {
        prompt("Título da spec: ")?
    } else {
        title
    };
    let title = match clean_title(&title) {
        Ok(title) => title,
        Err(error) => {
            eprintln!("{error}");
            bail!("título inválido. Uso: create-new-spec.sh \"Título da spec\"");
        }
    };
    let slug = slugify(&title);
    if slug.is_empty() {
        bail!("o título precisa ter ao menos uma letra ou número");
    }

    let config = specs.config();
    let sequences = specs.harness.config_file("sequences.yml");
    let prefix = specs.setting("id.prefix")?;
    let digits = specs.setting("id.digits")?;
    let sequence = config_get(&sequences, "specs")?;
    let Some(width) = parse_number(&digits) else {
        bail!("id.digits inválido em {}: '{digits}'", config.display());
    };
    let Some(number) = parse_number(&sequence) else {
        bail!("specs inválido em {}: '{sequence}'", sequences.display());
    };
    let spec_id = format!("{prefix}-{number:0width$}", width = width as usize);

    let draft_dir = specs.draft_dir()?;
    let template = specs.templates_dir()?.join(specs.setting("template")?);
    if !template.is_file() {
        bail!("template não encontrado: {}", template.display());
    }
    let file_name = render(
        &specs.setting("file_name")?,
        &HashMap::from([("SPEC_ID", spec_id.clone()), ("SPEC_SLUG", slug)]),
    );
    let target = draft_dir.join(file_name);
    if target.exists() {
        bail!("o arquivo já existe: {}", target.display());
    }

    let now = utc_now();
    let front_matter = render(
        &specs.setting_block("front_matter")?,
        &HashMap::from([
            ("SPEC_ID", spec_id.clone()),
            ("SPEC_TITLE", yaml_escape(&title)),
            ("SPEC_STATUS", specs.status("draft")?),
            ("AUTHOR", yaml_escape(&author(&specs.harness.root))),
            ("DATE", now[..10].to_string()),
            ("DATETIME", now.clone()),
        ]),
    );
    fs::create_dir_all(&draft_dir)?;
    let content = render_lines(
        &fs::read_to_string(&template)?,
        &HashMap::from([
            ("FRONT_MATTER", wrap_front_matter(&front_matter)),
            ("SPEC_ID", spec_id.clone()),
            ("SPEC_TITLE", title),
        ]),
    );
    fs::write(&target, content)?;

    config_set(&sequences, "specs", &(number + 1).to_string())?;
    println!("{spec_id} criada em {}", target.display());
    Ok(())
}

fn prompt(question: &str) -> Result<String> {
    eprint!("{question}");
    io::stderr().flush()?;
    let mut answer = String::new();
    io::stdin().lock().read_line(&mut answer)?;
    Ok(answer.trim_end_matches(['\n', '\r']).to_string())
}

fn parse_number(value: &str) -> Option<u64> {
    if value.is_empty() || !value.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    value.parse().ok()
}

/// Autor da spec: `user.name` do git ou, sem ele, a variável `USER`.
fn author(harness_root: &Path) -> String {
    let configured = Command::new("git")
        .arg("-C")
        .arg(harness_root)
        .args(["config", "user.name"])
        .stderr(Stdio::null())
        .output()
        .ok()
        .filter(|output| output.status.success());
    match configured {
        Some(output) => String::from_utf8_lossy(&output.stdout)
            .trim_end_matches('\n')
            .to_string(),
        None => std::env::var("USER").unwrap_or_default(),
    }
}

// --- list ---

fn list(specs: &Specs, status_filter: Option<&str>) -> Result<()> {
    let specs_dir = specs.dir()?;
    let prefix = format!("{}/", specs_dir.display());
    let mut rows = Vec::new();
    for file in markdown_files(&specs_dir) {
        let path = Path::new(&file);
        let Ok(id) = frontmatter_get(path, "id") else {
            continue;
        };
        let status = frontmatter_get(path, "status").unwrap_or_default();
        if status_filter.is_some_and(|wanted| !wanted.is_empty() && wanted != status) {
            continue;
        }
        let title = frontmatter_get(path, "title").unwrap_or_default();
        let relative = file.strip_prefix(&prefix).unwrap_or(&file);
        rows.push(format!(
            "| {} | {} | {} | {relative} |",
            table_cell(&id),
            table_cell(&title),
            table_cell(&status)
        ));
    }
    if rows.is_empty() {
        println!("Nenhuma spec encontrada.");
        return Ok(());
    }
    println!("| ID | Título | Status | Arquivo |");
    println!("|---|---|---|---|");
    for row in rows {
        println!("{row}");
    }
    Ok(())
}

fn table_cell(value: &str) -> String {
    value.replace('|', "\\|")
}

/// Arquivos `.md` da pasta e das subpastas, em ordem de bytes do caminho.
fn markdown_files(dir: &Path) -> Vec<String> {
    let mut files = Vec::new();
    let mut pending = vec![dir.to_path_buf()];
    while let Some(current) = pending.pop() {
        let Ok(entries) = fs::read_dir(&current) else {
            continue;
        };
        for entry in entries.flatten() {
            let Ok(kind) = entry.file_type() else {
                continue;
            };
            let path = entry.path();
            if kind.is_dir() {
                pending.push(path);
            } else if kind.is_file() && path.extension().is_some_and(|ext| ext == "md") {
                files.push(path.display().to_string());
            }
        }
    }
    files.sort();
    files
}

// --- refine, validate e mark-refined ---

fn refine(specs: &Specs, spec_id: Option<String>, kind: Option<String>) -> Result<()> {
    let spec_id = spec_id.unwrap_or_default();
    if spec_id.is_empty() || spec_id.starts_with('-') {
        bail!("{REFINE_USAGE}");
    }
    match kind.filter(|kind| !kind.is_empty()) {
        Some(kind) => create_refined(specs, &spec_id, &kind),
        None => bail!("informe o tipo. {REFINE_USAGE}"),
    }
}

fn create_refined(specs: &Specs, spec_id: &str, kind: &str) -> Result<()> {
    let sections = sections_for_type(specs, kind)?;
    let draft = find_draft(specs, spec_id)?;
    let draft_status = specs.status("draft")?;
    let status = frontmatter_get(&draft, "status")?;
    if status != draft_status {
        bail!("a spec {spec_id} está com status '{status}'; só specs com status '{draft_status}' podem ser refinadas");
    }

    let target = specs
        .refined_dir()?
        .join(draft.file_name().unwrap_or_default());
    if target.exists() {
        bail!(
            "a spec refinada já existe: {}. Preencha-a e rode: create-refined-spec.sh --validate {spec_id}",
            target.display()
        );
    }

    let template = specs
        .templates_dir()?
        .join(specs.setting("refined_template")?);
    if !template.is_file() {
        bail!("template não encontrado: {}", template.display());
    }
    let history = existing_section_template(specs, &specs.setting("adjustment_history_section")?)?;
    let mut content = String::new();
    for section in &sections {
        let section_template = existing_section_template(specs, section)?;
        content.push_str(read_trimmed(&section_template)?.as_str());
        content.push_str("\n\n");
    }

    let now = utc_now();
    let specs_dir = format!("{}/", specs.dir()?.display());
    let draft_path = draft.display().to_string();
    let source = draft_path.strip_prefix(&specs_dir).unwrap_or(&draft_path);
    let overrides = render(
        &specs.setting_block("refined_front_matter")?,
        &HashMap::from([
            ("SPEC_TYPE", kind.to_string()),
            ("SPEC_STATUS", specs.status("refined")?),
            ("SOURCE_PATH", yaml_escape(source)),
            ("DATE", now[..10].to_string()),
            ("DATETIME", now.clone()),
        ]),
    );
    let draft_content = fs::read_to_string(&draft)?;
    let base = frontmatter_lines(&draft_content).unwrap_or_default();
    let front_matter = merge_front_matter(&base, overrides.trim_end_matches('\n'));

    fs::create_dir_all(specs.refined_dir()?)?;
    let refined = render_lines(
        &fs::read_to_string(&template)?,
        &HashMap::from([
            ("FRONT_MATTER", wrap_front_matter(&front_matter)),
            ("SPEC_ID", spec_id.to_string()),
            ("SPEC_TITLE", frontmatter_get(&draft, "title")?),
            (
                "SECTIONS",
                content.strip_suffix("\n\n").unwrap_or(&content).to_string(),
            ),
            ("ADJUSTMENT_HISTORY", read_trimmed(&history)?),
        ]),
    );
    fs::write(&target, refined)?;
    println!(
        "Spec refinada {spec_id} ({kind}) criada em {}",
        target.display()
    );
    Ok(())
}

fn sections_for_type(specs: &Specs, kind: &str) -> Result<Vec<String>> {
    let types = config_list(&specs.config(), "types")?;
    if !types.iter().any(|known| known == kind) {
        let listed: String = types.iter().map(|known| format!("{known} ")).collect();
        bail!("tipo inválido '{kind}'. Tipos: {listed}");
    }
    config_list(&specs.config(), &format!("sections_{kind}"))
}

fn existing_section_template(specs: &Specs, section: &str) -> Result<PathBuf> {
    let template = specs.section_template(section)?;
    if !template.is_file() {
        bail!("template de seção não encontrado: {}", template.display());
    }
    Ok(template)
}

/// Conteúdo do arquivo sem as quebras de linha finais, como no `$(cat arquivo)` dos scripts.
fn read_trimmed(path: &Path) -> Result<String> {
    Ok(fs::read_to_string(path)?.trim_end_matches('\n').to_string())
}

fn wrap_front_matter(front_matter: &str) -> String {
    format!("---\n{}\n---", front_matter.trim_end_matches('\n'))
}

/// Preenche o template e termina cada linha com quebra, como o `render` em awk dos scripts.
fn render_lines(template: &str, values: &HashMap<&str, String>) -> String {
    let mut out = render(template, values);
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
    out
}

/// Junta dois front-matters: cada chave de `overrides` substitui a de mesmo nome em `base`, na
/// mesma posição e com todas as linhas do valor antigo; as chaves novas entram no fim.
fn merge_front_matter(base: &[&str], overrides: &str) -> String {
    let mut order = Vec::new();
    let mut line_of: HashMap<&str, &str> = HashMap::new();
    for line in overrides.split('\n') {
        if let Some(key) = front_matter_key(line) {
            order.push(key);
            line_of.insert(key, line);
        }
    }
    let mut out = Vec::new();
    let mut used = Vec::new();
    let mut skipping = false;
    for line in base {
        if let Some(key) = front_matter_key(line) {
            skipping = line_of.contains_key(key);
            if skipping {
                out.push(line_of[key]);
                used.push(key);
            }
        }
        if !skipping && !line.is_empty() {
            out.push(line);
        }
    }
    for key in order {
        if !used.contains(&key) {
            out.push(line_of[key]);
        }
    }
    out.join("\n")
}

fn front_matter_key(line: &str) -> Option<&str> {
    let end = line.find(|c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == '-'))?;
    (end > 0 && line[end..].starts_with(':')).then(|| &line[..end])
}

fn validate_refined(specs: &Specs, spec_id: &str) -> Result<()> {
    let refined = find_refined_to_mark(specs, spec_id)?;
    let errors = refined_errors(specs, &refined)?;
    if !errors.is_empty() {
        bail!(
            "a spec refinada {spec_id} tem problemas:\n{}",
            errors.join("\n")
        );
    }
    println!("Spec refinada {spec_id} válida: {}", refined.display());
    Ok(())
}

fn mark_refined(specs: &Specs, spec_id: &str) -> Result<()> {
    let refined = find_refined_to_mark(specs, spec_id)?;
    let draft = find_draft(specs, spec_id)?;
    let draft_status = specs.status("draft")?;
    let refined_status = specs.status("refined")?;
    let status = frontmatter_get(&draft, "status")?;
    if status != draft_status {
        bail!("o rascunho de {spec_id} já está com status '{status}'. Para mudar a refinada, use a skill adjust-spec");
    }
    let errors = refined_errors(specs, &refined)?;
    if !errors.is_empty() {
        bail!(
            "a spec refinada {spec_id} tem problemas:\n{}",
            errors.join("\n")
        );
    }

    let hash_field = specs.setting("content_hash_field")?;
    frontmatter_put(&refined, &hash_field, &content_hash(&refined)?)?;
    frontmatter_set(&draft, "status", &refined_status)?;
    println!(
        "Spec {spec_id} marcada como refinada: {}",
        refined.display()
    );
    println!(
        "Status do rascunho alterado para '{refined_status}': {}",
        draft.display()
    );
    Ok(())
}

fn find_draft(specs: &Specs, spec_id: &str) -> Result<PathBuf> {
    let draft_dir = specs.draft_dir()?;
    match find_by_id(&draft_dir, spec_id) {
        Some(draft) => Ok(draft),
        None => bail!("spec {spec_id} não encontrada em {}", draft_dir.display()),
    }
}

fn find_refined_to_mark(specs: &Specs, spec_id: &str) -> Result<PathBuf> {
    let refined_dir = specs.refined_dir()?;
    match find_by_id(&refined_dir, spec_id) {
        Some(refined) => Ok(refined),
        None => bail!(
            "spec refinada {spec_id} não encontrada em {}. Crie-a com: create-refined-spec.sh {spec_id} <tipo>",
            refined_dir.display()
        ),
    }
}

/// Primeiro `.md` da pasta, em ordem de nome, cujo front-matter tem o `id` informado.
fn find_by_id(dir: &Path, spec_id: &str) -> Option<PathBuf> {
    let mut files: Vec<PathBuf> = fs::read_dir(dir)
        .ok()?
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_file()
                && path.extension().is_some_and(|ext| ext == "md")
                && !path
                    .file_name()
                    .is_some_and(|name| name.to_string_lossy().starts_with('.'))
        })
        .collect();
    files.sort();
    files
        .into_iter()
        .find(|file| frontmatter_get(file, "id").is_ok_and(|id| id == spec_id))
}

// --- Validação da refinada ---

/// Um problema por item. Vazio quando a refinada está válida.
fn refined_errors(specs: &Specs, file: &Path) -> Result<Vec<String>> {
    let content = fs::read_to_string(file)?;
    let mut errors: Vec<String> = template_leftovers(&content)
        .into_iter()
        .map(|(line, text)| format!("variável sem preencher: {line}:{text}"))
        .collect();

    let kind = frontmatter_get(file, "type").unwrap_or_default();
    if !config_list(&specs.config(), "types")?.contains(&kind) {
        errors.push(format!(
            "o campo 'type' do front-matter é inválido: '{kind}'"
        ));
        return Ok(errors);
    }
    let sections = config_list(&specs.config(), &format!("sections_{kind}"))?;
    let has = |name: &str| sections.iter().any(|section| section == name);
    let rows_of = |section: &str| -> Result<Vec<String>> {
        let heading = specs.section_heading(section)?;
        Ok(markdown_section(&content, &heading)
            .map(|text| markdown_table_rows(&text))
            .unwrap_or_default())
    };

    for section in &sections {
        let heading = specs.section_heading(section)?;
        let template = fs::read_to_string(specs.section_template(section)?).unwrap_or_default();
        match markdown_section(&content, &heading) {
            None => errors.push(format!("seção ausente: '## {heading}'")),
            Some(text) if template.lines().any(|line| line.starts_with('|')) => {
                if markdown_table_rows(&text).is_empty() {
                    errors.push(format!("a tabela da seção '{heading}' está vazia (use uma linha com '-' se não se aplica)"));
                }
            }
            Some(text) if text.trim().is_empty() => {
                errors.push(format!("a seção '{heading}' está vazia"));
            }
            Some(_) => {}
        }
    }

    if has("functional-requirements") {
        let mut rows = rows_of("functional-requirements")?;
        if rows.is_empty() {
            rows.push(String::new());
        }
        for row in rows {
            let requirement = markdown_cell(&row, 1);
            if requirement == "-" {
                continue;
            }
            if !is_requirement_id(&requirement) {
                errors.push(format!(
                    "ID de requisito funcional inválido: '{requirement}' (use RF-01, RF-02...)"
                ));
                continue;
            }
            for section in ["test-cases", "acceptance-criteria"] {
                if !has(section) {
                    continue;
                }
                let heading = specs.section_heading(section)?;
                let text = markdown_section(&content, &heading).unwrap_or_default();
                if !mentions(&text, &requirement) {
                    errors.push(format!("{requirement} não aparece em '{heading}'"));
                }
            }
        }
    }

    if has("synthesis") {
        let synthesis = rows_of("synthesis")?.join("\n");
        for section in ["thesis", "antithesis"] {
            if !has(section) {
                continue;
            }
            for row in rows_of(section)? {
                let argument = markdown_cell(&row, 1);
                if argument != "-" && !mentions(&synthesis, &argument) {
                    errors.push(format!(
                        "{argument} ('{}') não é considerado em nenhuma conclusão da síntese",
                        specs.section_heading(section)?
                    ));
                }
            }
        }
    }

    if has("affected-components") {
        let root = specs.project_root()?.display().to_string();
        for row in rows_of("affected-components")? {
            let path = markdown_cell(&row, 1).replace('`', "");
            let action = markdown_cell(&row, 2);
            if path == "-" {
                continue;
            }
            if !format!(" {COMPONENT_ACTIONS} ").contains(&format!(" {action} ")) {
                errors.push(format!(
                    "ação inválida para '{path}': '{action}' (use: {COMPONENT_ACTIONS})"
                ));
                continue;
            }
            let exists = Path::new(&format!("{root}/{path}")).exists();
            if action == "criar" && exists {
                errors.push(format!("'{path}' já existe; use a ação 'alterar'"));
            } else if action != "criar" && !exists {
                errors.push(format!(
                    "'{path}' não existe em {root}; use a ação 'criar' ou corrija o caminho"
                ));
            }
        }
    }
    Ok(errors)
}

fn is_requirement_id(value: &str) -> bool {
    value
        .strip_prefix("RF-")
        .is_some_and(|number| !number.is_empty() && number.bytes().all(|b| b.is_ascii_digit()))
}

/// O ID aparece numa linha sem letra maiúscula, dígito ou hífen antes e sem dígito depois.
fn mentions(text: &str, id: &str) -> bool {
    text.lines().any(|line| {
        (0..=line.len())
            .filter(|&start| line.is_char_boundary(start) && line[start..].starts_with(id))
            .any(|start| {
                let before = line[..start].chars().next_back();
                let after = line[start + id.len()..].chars().next();
                before.is_none_or(|c| !(c.is_ascii_uppercase() || c.is_ascii_digit() || c == '-'))
                    && after.is_none_or(|c| !c.is_ascii_digit())
            })
    })
}

/// Pendências abertas: linhas da seção de pendências cuja primeira coluna não é `-`.
fn open_pending(specs: &Specs, file: &Path) -> Result<Vec<String>> {
    let content = fs::read_to_string(file)?;
    let heading = specs.setting("pending_heading")?;
    let rows = markdown_section(&content, &heading)
        .map(|text| markdown_table_rows(&text))
        .unwrap_or_default();
    Ok(rows
        .iter()
        .filter(|row| markdown_cell(row, 1) != "-")
        .map(|row| format!("{}: {}", markdown_cell(row, 2), markdown_cell(row, 3)))
        .collect())
}

/// SHA-256 do conteúdo depois do front-matter, sem `\r` no fim das linhas.
fn content_hash(file: &Path) -> Result<String> {
    Ok(sha256_hex(
        front_matter_body(&fs::read_to_string(file)?).as_bytes(),
    ))
}

fn front_matter_body(content: &str) -> String {
    let mut inside = false;
    let mut body = String::new();
    for (index, raw) in content.split_terminator('\n').enumerate() {
        let line = raw.strip_suffix('\r').unwrap_or(raw);
        if index == 0 && line == "---" {
            inside = true;
        } else if inside && line == "---" {
            inside = false;
        } else if !inside {
            body.push_str(line);
            body.push('\n');
        }
    }
    body
}

// --- adjust ---

fn find_adjustable(specs: &Specs, spec_id: &str) -> Result<PathBuf> {
    let refined_dir = specs.refined_dir()?;
    let Some(refined) = find_by_id(&refined_dir, spec_id) else {
        if find_by_id(&specs.draft_dir()?, spec_id).is_some() {
            bail!("a spec {spec_id} ainda não foi refinada; só specs refinadas são ajustadas");
        }
        bail!(
            "spec refinada {spec_id} não encontrada em {}",
            refined_dir.display()
        );
    };
    let status = frontmatter_get(&refined, "status").unwrap_or_default();
    let refined_status = specs.status("refined")?;
    let approved_status = specs.status("approved")?;
    if status == specs.status("planned")? {
        bail!("a spec {spec_id} está com status '{status}': as tarefas já foram criadas, e mudá-la exige um novo ciclo");
    }
    if status != refined_status && status != approved_status {
        bail!("a spec {spec_id} está com status '{status}'; só specs com status '{refined_status}' ou '{approved_status}' são ajustadas");
    }
    Ok(refined)
}

fn record_manual(specs: &Specs, spec_id: &str) -> Result<()> {
    let refined = find_adjustable(specs, spec_id)?;
    let hash_field = specs.setting("content_hash_field")?;
    let stored = frontmatter_get(&refined, &hash_field).unwrap_or_default();
    if stored == content_hash(&refined)? {
        println!("Nenhuma edição manual.");
        return Ok(());
    }

    let summary = manual_summary(specs, &refined)?;
    let origin = specs.setting("adjustment_origins.manual")?;
    let count = record_adjustment(specs, &refined, &origin, &summary)?;
    println!("Ajuste manual {count} registrado em {spec_id}: {summary}");
    drop_approval(specs, &refined, spec_id)?;

    let errors = refined_errors(specs, &refined)?;
    if !errors.is_empty() {
        println!("Problemas de validação:\n{}", errors.join("\n"));
    }
    Ok(())
}

fn validate_adjustable(specs: &Specs, spec_id: &str) -> Result<()> {
    let refined = find_adjustable(specs, spec_id)?;
    let errors = refined_errors(specs, &refined)?;
    if !errors.is_empty() {
        bail!(
            "a spec refinada {spec_id} tem problemas:\n{}",
            errors.join("\n")
        );
    }
    println!("Spec refinada {spec_id} válida: {}", refined.display());
    Ok(())
}

fn record(specs: &Specs, spec_id: &str, summary: &str) -> Result<()> {
    let summary = clean_summary(summary)?;
    let refined = find_adjustable(specs, spec_id)?;
    let errors = refined_errors(specs, &refined)?;
    if !errors.is_empty() {
        bail!(
            "a spec refinada {spec_id} tem problemas; nenhum ajuste registrado:\n{}",
            errors.join("\n")
        );
    }
    let origin = specs.setting("adjustment_origins.agent")?;
    let count = record_adjustment(specs, &refined, &origin, &summary)?;
    println!(
        "Ajuste {count} registrado em {spec_id}: {}",
        refined.display()
    );
    drop_approval(specs, &refined, spec_id)
}

/// Resumo de um ajuste do agente: sem espaços nas pontas de cada linha e numa linha só.
fn clean_summary(summary: &str) -> Result<String> {
    let cleaned = summary
        .split('\n')
        .map(|line| line.trim_matches(|c: char| c.is_ascii_whitespace()))
        .collect::<Vec<_>>()
        .join("\n")
        .trim_end_matches('\n')
        .to_string();
    if cleaned.is_empty() {
        bail!("o resumo do ajuste está vazio");
    }
    if cleaned.contains('\n') {
        bail!("o resumo não pode ter quebra de linha; separe os itens com '; '");
    }
    Ok(cleaned)
}

/// Derruba a aprovação de uma spec aprovada e avisa. Não faz nada numa spec só refinada.
fn drop_approval(specs: &Specs, file: &Path, spec_id: &str) -> Result<()> {
    let status = frontmatter_get(file, "status").unwrap_or_default();
    if status != specs.status("approved")? {
        return Ok(());
    }
    let refined_status = specs.status("refined")?;
    frontmatter_set(file, "status", &refined_status)?;
    frontmatter_put(file, &specs.setting("approved_at_field")?, "null")?;
    frontmatter_put(file, &specs.setting("approved_hash_field")?, "null")?;
    println!("Aprovação de {spec_id} derrubada: o status voltou para '{refined_status}'.");
    Ok(())
}

fn manual_summary(specs: &Specs, file: &Path) -> Result<String> {
    let summary = specs.setting("manual_adjustment_summary")?;
    let sections = changed_sections(
        &fs::read_to_string(file)?,
        &changed_lines(file),
        &specs.history_heading()?,
    );
    Ok(if sections.is_empty() {
        summary
    } else {
        format!("{summary}: {}", sections.join(", "))
    })
}

/// Linha inicial de cada trecho alterado desde o último commit. Vazio fora do git.
fn changed_lines(file: &Path) -> Vec<usize> {
    let (Some(dir), Some(name)) = (file.parent(), file.file_name()) else {
        return Vec::new();
    };
    let git = |args: &[&str]| {
        let mut command = Command::new("git");
        command.arg("-C").arg(dir).args(args).arg(name);
        command.stdin(Stdio::null()).stderr(Stdio::null());
        command
    };
    let tracked = git(&["ls-files", "--error-unmatch", "--"])
        .stdout(Stdio::null())
        .status()
        .is_ok_and(|status| status.success());
    if !tracked {
        return Vec::new();
    }
    let Ok(diff) = git(&["diff", "-U0", "HEAD", "--"]).output() else {
        return Vec::new();
    };
    String::from_utf8_lossy(&diff.stdout)
        .lines()
        .filter_map(hunk_start)
        .collect()
}

/// Linha inicial do lado novo de um cabeçalho `@@ -a,b +N,M @@`.
fn hunk_start(line: &str) -> Option<usize> {
    let (old, rest) = line.strip_prefix("@@ -")?.split_once(' ')?;
    if old.is_empty() || !old.bytes().all(|b| b.is_ascii_digit() || b == b',') {
        return None;
    }
    let rest = rest.strip_prefix('+')?;
    let end = rest
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(rest.len());
    rest[..end].parse().ok()
}

/// Seções (`## `) das linhas alteradas, sem o front-matter e sem a seção de histórico.
fn changed_sections(content: &str, changed: &[usize], skip: &str) -> Vec<String> {
    let mut in_front_matter = false;
    let mut in_code = false;
    let mut heading = "";
    let mut sections: Vec<String> = Vec::new();
    for (index, raw) in content.split_terminator('\n').enumerate() {
        let line = raw.strip_suffix('\r').unwrap_or(raw);
        if index == 0 && line == "---" {
            in_front_matter = true;
            continue;
        }
        if in_front_matter {
            in_front_matter = line != "---";
            continue;
        }
        if is_code_fence(line) {
            in_code = !in_code;
        }
        if !in_code {
            heading = line.strip_prefix("## ").unwrap_or(heading);
        }
        let counts = changed.contains(&(index + 1))
            && !heading.is_empty()
            && heading != skip
            && !sections.iter().any(|seen| seen == heading);
        if counts {
            sections.push(heading.to_string());
        }
    }
    sections
}

fn is_code_fence(line: &str) -> bool {
    line.trim_start_matches([' ', '\t']).starts_with("```")
}

/// Registra um ajuste: linha no histórico, contagem, datas e, por último, o hash do conteúdo.
/// Sem a seção de histórico, avisa e termina com erro sem alterar o arquivo.
fn record_adjustment(specs: &Specs, file: &Path, origin: &str, summary: &str) -> Result<u64> {
    let count = frontmatter_get(file, "adjustments")
        .ok()
        .and_then(|value| parse_number(&value))
        .unwrap_or(0)
        + 1;
    let now = utc_now();
    let heading = specs.history_heading()?;
    let row = format!(
        "| {count} | {} | {origin} | {} |",
        &now[..10],
        summary.replace('|', "\\|")
    );
    let Some(updated) = append_history_row(&fs::read_to_string(file)?, &heading, &row) else {
        eprintln!("seção ausente: '## {heading}'; nenhum ajuste registrado");
        std::process::exit(1);
    };
    fs::write(file, updated)?;
    frontmatter_put(file, "adjustments", &count.to_string())?;
    frontmatter_put(file, "adjusted_at", &now)?;
    frontmatter_put(file, "updated_at", &now)?;
    let hash_field = specs.setting("content_hash_field")?;
    frontmatter_put(file, &hash_field, &content_hash(file)?)?;
    Ok(count)
}

/// Acrescenta a linha depois da última linha de tabela da seção de histórico.
fn append_history_row(content: &str, heading: &str, row: &str) -> Option<String> {
    let heading = format!("## {heading}");
    let lines: Vec<&str> = content.split_terminator('\n').collect();
    let mut in_code = false;
    let mut inside = false;
    let mut at = None;
    for (index, raw) in lines.iter().enumerate() {
        let line = raw.strip_suffix('\r').unwrap_or(raw);
        if is_code_fence(line) {
            in_code = !in_code;
            continue;
        }
        if in_code {
            continue;
        }
        if line == heading {
            inside = true;
            at = Some(index);
            continue;
        }
        if inside && line.starts_with("## ") {
            inside = false;
        }
        if inside && line.starts_with('|') {
            at = Some(index);
        }
    }
    let at = at?;
    let mut out = String::new();
    for (index, line) in lines.iter().enumerate() {
        out.push_str(line);
        out.push('\n');
        if index == at {
            out.push_str(row);
            out.push('\n');
        }
    }
    Some(out)
}

/// Como `frontmatter_set`, mas acrescenta a chave no fim do front-matter se ela não existir.
fn frontmatter_put(path: &Path, key: &str, value: &str) -> Result<()> {
    if frontmatter_set(path, key, value).is_ok() {
        return Ok(());
    }
    let content = fs::read_to_string(path)?;
    let mut inside = false;
    let mut found = false;
    let mut out = String::new();
    for (index, raw) in content.split_terminator('\n').enumerate() {
        let line = raw.strip_suffix('\r').unwrap_or(raw);
        if index == 0 && line == "---" {
            inside = true;
        } else if inside && line == "---" {
            out.push_str(&format!("{key}: {value}\n"));
            inside = false;
            found = true;
        }
        out.push_str(raw);
        out.push('\n');
    }
    if !found {
        bail!("Front-matter não encontrado em {}", path.display());
    }
    fs::write(path, out)?;
    Ok(())
}

// --- approve ---

fn find_approvable(specs: &Specs, spec_id: &str) -> Result<PathBuf> {
    let refined_dir = specs.refined_dir()?;
    match find_by_id(&refined_dir, spec_id) {
        Some(refined) => Ok(refined),
        None if find_by_id(&specs.draft_dir()?, spec_id).is_some() => {
            bail!("a spec {spec_id} ainda não foi refinada; só specs refinadas são aprovadas")
        }
        None => bail!(
            "spec refinada {spec_id} não encontrada em {}",
            refined_dir.display()
        ),
    }
}

/// Um impedimento por item. Vazio quando a spec pode ser aprovada.
fn approval_blockers(specs: &Specs, spec_id: &str, file: &Path) -> Result<Vec<String>> {
    let status = frontmatter_get(file, "status").unwrap_or_default();
    let refined_status = specs.status("refined")?;
    if status == specs.status("approved")? {
        return Ok(vec![format!("a spec {spec_id} já está aprovada")]);
    }
    if status != refined_status {
        return Ok(vec![format!(
            "a spec {spec_id} está com status '{status}'; só specs com status '{refined_status}' são aprovadas"
        )]);
    }

    let mut blockers: Vec<String> = refined_errors(specs, file)?
        .into_iter()
        .map(|error| format!("validação: {error}"))
        .collect();
    blockers.extend(
        open_pending(specs, file)?
            .into_iter()
            .map(|pending| format!("pendência aberta: {pending}")),
    );
    let stored = frontmatter_get(file, &specs.setting("content_hash_field")?).unwrap_or_default();
    if stored != content_hash(file)? {
        blockers.push(format!(
            "edição manual sem registro; registre-a com: adjust-spec.sh --record-manual {spec_id}"
        ));
    }
    Ok(blockers)
}

fn validate_approval(specs: &Specs, spec_id: &str) -> Result<()> {
    let refined = find_approvable(specs, spec_id)?;
    let blockers = approval_blockers(specs, spec_id, &refined)?;
    if !blockers.is_empty() {
        bail!(
            "a spec {spec_id} não pode ser aprovada:\n{}",
            blockers.join("\n")
        );
    }
    println!("Spec {spec_id} pode ser aprovada: {}", refined.display());
    Ok(())
}

fn approve(specs: &Specs, spec_id: &str) -> Result<()> {
    let refined = find_approvable(specs, spec_id)?;
    let blockers = approval_blockers(specs, spec_id, &refined)?;
    if !blockers.is_empty() {
        bail!(
            "a spec {spec_id} não pode ser aprovada; nada foi alterado:\n{}",
            blockers.join("\n")
        );
    }

    let now = utc_now();
    let hash = frontmatter_get(&refined, &specs.setting("content_hash_field")?)?;
    frontmatter_set(&refined, "status", &specs.status("approved")?)?;
    frontmatter_put(&refined, &specs.setting("approved_at_field")?, &now)?;
    frontmatter_put(&refined, &specs.setting("approved_hash_field")?, &hash)?;
    frontmatter_put(&refined, "updated_at", &now)?;
    println!("Spec {spec_id} aprovada: {}", refined.display());

    // O rascunho só é removido depois de a aprovação estar gravada.
    let draft_dir = specs.draft_dir()?;
    match find_by_id(&draft_dir, spec_id) {
        Some(draft) => {
            if fs::remove_file(&draft).is_err() {
                bail!(
                    "a aprovação foi gravada, mas o rascunho não pôde ser removido; remova-o à mão: {}",
                    draft.display()
                );
            }
            println!("Rascunho removido: {}", draft.display());
        }
        None => println!(
            "Rascunho de {spec_id} não encontrado em {}; nada a remover.",
            draft_dir.display()
        ),
    }
    Ok(())
}

// --- Caminhos e datas ---

/// Resolve `.` e `..` sem seguir links, como o `cd` do shell.
fn normalize_lexically(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            other => out.push(other),
        }
    }
    out
}

/// Data e hora atuais em UTC, no formato `AAAA-MM-DDTHH:MM:SSZ`.
fn utc_now() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs());
    let (year, month, day) = civil_from_days((seconds / 86_400) as i64);
    let time = seconds % 86_400;
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        time / 3600,
        time % 3600 / 60,
        time % 60
    )
}

/// Dias desde 1970-01-01 para (ano, mês, dia), pelo algoritmo de Howard Hinnant.
fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let shifted = days + 719_468;
    let era = shifted.div_euclid(146_097);
    let day_of_era = shifted - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted_month = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * shifted_month + 2) / 5 + 1;
    let month = if shifted_month < 10 {
        shifted_month + 3
    } else {
        shifted_month - 9
    };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    (year, month, day)
}
