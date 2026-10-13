//! Comandos `rules`: regras do projeto e mapa de carregamento.
//! Reproduzem `validate-rules.sh` (`lib/rules.sh`), `init-harness.sh` e `adjust-rules.sh`.
//! O `load` segue o `adjust-rules.sh --load`, que tem o `unload` correspondente.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context, Result};
use clap::Subcommand;

use super::context::{is_any_item, is_blank_or_comment, is_map_item, map_item, map_key};
use crate::harness::Harness;
use crate::support::{
    config_get, exit_failure, fail, is_kebab_case, join_records, records, render, script_title,
    utc_date,
};

const ADJUST_USAGE: &str = "Uso: adjust-rules.sh --next-id <categoria> | --remove <categoria> <NN> | --create-category <categoria> <título> <propósito> | --load <categoria> always|<skill>[,<skill>...] | --unload <categoria> always|<skill>[,<skill>...]";

/// Pasta das regras do projeto, relativa à raiz do harness (a mesma citada em `required`).
const RULES_DIR: &str = "rules/project";
/// Template de categoria, relativo à pasta de templates.
const CATEGORY_TEMPLATE: &str = "rules/category.md";
const CONTEXT_MAP: &str = "config/context.yml";

#[derive(Debug, Subcommand)]
pub enum RulesCommand {
    /// Valida o mapa de contexto e os arquivos de regras de projeto.
    Validate,
    /// Lista os sinais do projeto que indicam categorias de regras.
    Detect,
    /// Cria ou atualiza uma categoria a partir de um arquivo de regras.
    Write {
        category: String,
        rules_file: String,
    },
    /// Mostra o que o `write` mudaria, sem gravar.
    Preview {
        category: String,
        rules_file: String,
    },
    /// Registra a categoria em `required` (`always`) ou nas skills indicadas.
    Load { category: String, target: String },
    /// Retira a categoria de `required` (`always`) ou das skills indicadas.
    Unload { category: String, target: String },
    /// Imprime o próximo ID livre da categoria.
    NextId { category: String },
    /// Marca uma regra como removida.
    Remove { category: String, number: String },
    /// Cria um arquivo de categoria vazio pelo template.
    CreateCategory {
        category: String,
        title: Option<String>,
        purpose: Option<String>,
    },
}

pub fn run(command: RulesCommand) -> Result<()> {
    let rules = Rules {
        harness: Harness::from_current_dir()?,
    };
    match command {
        RulesCommand::Validate => rules.validate(),
        RulesCommand::Detect => rules.detect(),
        RulesCommand::Write {
            category,
            rules_file,
        } => rules.write(&category, Path::new(&rules_file)),
        RulesCommand::Preview {
            category,
            rules_file,
        } => rules.preview(&category, Path::new(&rules_file)),
        RulesCommand::Load { category, target } => rules.map_edit(Edit::Add, &category, &target),
        RulesCommand::Unload { category, target } => {
            rules.map_edit(Edit::Remove, &category, &target)
        }
        RulesCommand::NextId { category } => rules.next_id(&category),
        RulesCommand::Remove { category, number } => rules.remove(&category, &number),
        RulesCommand::CreateCategory {
            category,
            title: Some(title),
            purpose: Some(purpose),
        } => rules.create_category(&category, &title, &purpose),
        RulesCommand::CreateCategory { .. } => bail!(ADJUST_USAGE),
    }
}

struct Rules {
    harness: Harness,
}

impl Rules {
    fn rules_dir(&self) -> PathBuf {
        self.harness.root.join(RULES_DIR)
    }

    fn category_path(&self, category: &str) -> PathBuf {
        self.rules_dir().join(format!("{category}.md"))
    }

    fn skill_dir(&self, scope: &str) -> Result<String> {
        config_get(
            &self.harness.config_file("skills.yml"),
            &format!("dirs.{scope}"),
        )
    }

    fn category_template(&self) -> Result<String> {
        let path = self.harness.path("templates")?.join(CATEGORY_TEMPLATE);
        read(&path)
    }

    // --- validate ---

    fn validate(&self) -> Result<()> {
        let mut problems = self.map_problems()?;
        let mut files = Vec::new();
        collect_markdown(&self.rules_dir(), &mut files);
        let mut files: Vec<String> = files
            .iter()
            .filter_map(|file| file.strip_prefix(&self.harness.root).ok())
            .map(|file| file.to_string_lossy().into_owned())
            .collect();
        files.sort_unstable();
        for file in files {
            let content = fs::read(self.harness.root.join(&file)).unwrap_or_default();
            problems.extend(category_problems(&file, &String::from_utf8_lossy(&content)));
        }
        if !problems.is_empty() {
            for problem in problems {
                println!("{problem}");
            }
            exit_failure();
        }
        println!("Regras e mapa válidos.");
        Ok(())
    }

    /// Caminhos de `required` e das skills existem, e cada skill do mapa existe.
    fn map_problems(&self) -> Result<Vec<String>> {
        let rel_default = self.skill_dir("default")?;
        let rel_custom = self.skill_dir("custom")?;
        let dir_default = self.harness.resolve(&rel_default);
        let dir_custom = self.harness.resolve(&rel_custom);
        let map = read(&self.harness.config_file("context.yml"))?;
        let mut problems = Vec::new();
        let mut seen: Vec<String> = Vec::new();
        for entry in map_entries(&map) {
            match entry {
                MapEntry::Required(path) => {
                    if !self.harness.resolve(&path).exists() {
                        problems.push(format!(
                            "{CONTEXT_MAP}: required: caminho inexistente: {path}"
                        ));
                    }
                }
                MapEntry::Skill(skill, path) => {
                    if !seen.contains(&skill) {
                        if !dir_default.join(&skill).is_dir() && !dir_custom.join(&skill).is_dir() {
                            problems.push(format!(
                                "{CONTEXT_MAP}: skills.{skill}: skill inexistente em {rel_default} e {rel_custom}"
                            ));
                        }
                        seen.push(skill.clone());
                    }
                    if let Some(path) = path.filter(|path| !path.is_empty()) {
                        if !self.harness.resolve(&path).exists() {
                            problems.push(format!(
                                "{CONTEXT_MAP}: skills.{skill}: caminho inexistente: {path}"
                            ));
                        }
                    }
                }
            }
        }
        Ok(problems)
    }

    // --- detect ---

    fn detect(&self) -> Result<()> {
        let root = self.harness.path("root")?;
        let mut signals = Vec::new();
        detect_in(&root, "", 1, &mut signals);
        signals.sort_unstable();
        signals.dedup();
        for signal in signals {
            println!("{signal}");
        }
        Ok(())
    }

    // --- write e preview ---

    fn write(&self, category: &str, input: &Path) -> Result<()> {
        let built = self.build_category(category, input)?;
        let target = self.category_path(category);
        fs::create_dir_all(self.rules_dir())?;
        if fs::read(&target).is_ok_and(|current| current == built.as_bytes()) {
            println!("Nenhuma mudança em {RULES_DIR}/{category}.md.");
            return Ok(());
        }
        write(&target, &built)?;
        println!("Categoria gravada: {RULES_DIR}/{category}.md");
        Ok(())
    }

    fn preview(&self, category: &str, input: &Path) -> Result<()> {
        let built = self.build_category(category, input)?;
        let target = self.category_path(category);
        match fs::read(&target) {
            Err(_) => {
                println!("Categoria nova: {RULES_DIR}/{category}.md");
                print!("{built}");
            }
            Ok(current) if current == built.as_bytes() => println!("Nenhuma mudança."),
            Ok(_) => print!("{}", unified_diff(&target, &built)?),
        }
        Ok(())
    }

    /// Conteúdo da categoria depois de aplicar o arquivo de regras, sem gravar.
    fn build_category(&self, category: &str, input: &Path) -> Result<String> {
        if !is_kebab_case(category) {
            bail!("categoria inválida: '{category}' (use kebab-case, ex.: code-style)");
        }
        if !input.is_file() {
            bail!("arquivo de regras não encontrado: {}", input.display());
        }
        let request = RulesRequest::parse(&read(input)?)?;
        if request.rules.is_empty() && request.removals.is_empty() {
            bail!(
                "o arquivo de regras não tem nenhuma regra: {}",
                input.display()
            );
        }
        let target = self.category_path(category);
        if target.is_file() {
            return Ok(update_category(category, &read(&target)?, &request));
        }
        if request.rules.is_empty() {
            bail!("a categoria '{category}' não existe: não há o que remover");
        }
        Ok(new_category(category, &self.category_template()?, request))
    }

    // --- load e unload ---

    fn map_edit(&self, edit: Edit, category: &str, targets: &str) -> Result<()> {
        self.existing_category(category)?;
        let path = format!("{RULES_DIR}/{category}.md");
        let targets: Vec<&str> = split_targets(targets);
        if targets.is_empty() {
            bail!("informe 'always' ou ao menos uma skill");
        }
        for target in &targets {
            if !is_kebab_case(target) {
                bail!("destino inválido '{target}': use 'always' ou o nome de uma skill");
            }
            if *target != "always" && !self.skill_exists(target)? {
                bail!(
                    "a skill '{target}' não existe em {} nem em {}",
                    self.skill_dir("default")?,
                    self.skill_dir("custom")?
                );
            }
        }
        let map_file = self.harness.config_file("context.yml");
        for target in targets {
            let entry = if target == "always" {
                "required"
            } else {
                target
            };
            let map = read(&map_file)?;
            match (edit, edit_map(&map, edit, entry, &path)?) {
                (Edit::Add, Some(updated)) => {
                    write(&map_file, &updated)?;
                    println!("Categoria '{category}' carregada em {target}.");
                }
                (Edit::Add, None) => {
                    println!("A categoria '{category}' já era carregada em {target}.")
                }
                (Edit::Remove, Some(updated)) => {
                    write(&map_file, &updated)?;
                    println!("Categoria '{category}' retirada de {target}.");
                }
                (Edit::Remove, None) => {
                    bail!("a categoria '{category}' não está carregada em {target}")
                }
            }
        }
        Ok(())
    }

    fn skill_exists(&self, skill: &str) -> Result<bool> {
        for scope in ["default", "custom"] {
            if self
                .harness
                .resolve(&self.skill_dir(scope)?)
                .join(skill)
                .is_dir()
            {
                return Ok(true);
            }
        }
        Ok(false)
    }

    // --- next-id, remove e create-category ---

    /// Caminho da categoria; falha se o nome não estiver em kebab-case ou se ela não existir.
    fn existing_category(&self, category: &str) -> Result<PathBuf> {
        if !is_kebab_case(category) {
            bail!("categoria inválida '{category}': use kebab-case (ex.: code-style)");
        }
        let path = self.category_path(category);
        if !path.is_file() {
            bail!("a categoria '{category}' não existe: {RULES_DIR}/{category}.md");
        }
        Ok(path)
    }

    fn next_id(&self, category: &str) -> Result<()> {
        let path = self.existing_category(category)?;
        let next = highest_number(&read(&path)?, category) + 1;
        println!("project/{category}-{next:02}");
        Ok(())
    }

    fn remove(&self, category: &str, number: &str) -> Result<()> {
        if !is_kebab_case(category) {
            bail!("categoria inválida '{category}': use kebab-case (ex.: code-style)");
        }
        let valid = (1..=2).contains(&number.len()) && number.bytes().all(|b| b.is_ascii_digit());
        if !valid {
            bail!("número de regra inválido '{number}': use 1 ou 2 dígitos (ex.: 03)");
        }
        let path = self.existing_category(category)?;
        let number: u32 = number.parse()?;
        let id = format!("[project/{category}-{number:02}]: ");
        let content = read(&path)?;
        if !content.contains(&id) {
            bail!(
                "a regra {} não existe em {RULES_DIR}/{category}.md",
                id.trim_end_matches(": ")
            );
        }
        if content.lines().any(|line| is_removed_rule(line, &id)) {
            bail!("a regra {} já está removida", id.trim_end_matches(": "));
        }
        let today = utc_date();
        let lines: Vec<String> = records(&content)
            .into_iter()
            .map(|line| {
                if line.starts_with(&id) {
                    format!("{id}Removida em {today}.")
                } else {
                    line.to_string()
                }
            })
            .collect();
        write(&path, &join_records(&lines))?;
        println!(
            "Regra {} marcada como removida em {today}.",
            id.trim_end_matches(": ")
        );
        Ok(())
    }

    fn create_category(&self, category: &str, title: &str, purpose: &str) -> Result<()> {
        if !is_kebab_case(category) {
            bail!("categoria inválida '{category}': use kebab-case (ex.: code-style)");
        }
        let title = script_title(title).unwrap_or_else(|problem| fail(problem));
        if purpose.trim().is_empty() {
            bail!("o propósito está vazio");
        }
        if purpose.contains('\n') {
            bail!("o propósito deve ter uma linha só");
        }
        if purpose.contains("#{") {
            bail!("o propósito não pode conter '#{{', reservado para variáveis de template");
        }
        let path = self.category_path(category);
        if path.exists() {
            bail!("a categoria '{category}' já existe: {RULES_DIR}/{category}.md");
        }
        let template = without_leading_comment(&self.category_template()?);
        let content = render(
            &template,
            &HashMap::from([
                ("CATEGORY_TITLE", title),
                ("CATEGORY_PURPOSE", purpose.to_string()),
                ("CATEGORY_RULES", String::new()),
            ]),
        );
        fs::create_dir_all(self.rules_dir())?;
        write(&path, &content)?;
        println!("Categoria criada: {RULES_DIR}/{category}.md");
        Ok(())
    }
}

// --- Mapa de contexto ---

enum MapEntry {
    Required(String),
    /// Skill e um caminho dela (`None` na linha que abre a entrada).
    Skill(String, Option<String>),
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Block {
    Other,
    Required,
    Skills,
}

fn top_level_block(line: &str) -> Block {
    if line.starts_with("required:") {
        Block::Required
    } else if line.starts_with("skills:") {
        Block::Skills
    } else {
        Block::Other
    }
}

/// Linha que abre a entrada de uma skill: `^  [^ \t-][^:]*:`.
fn skill_heading(line: &str) -> Option<&str> {
    let rest = line.strip_prefix("  ")?;
    let first = rest.chars().next()?;
    (!matches!(first, ' ' | '\t' | '-') && rest.contains(':')).then(|| map_key(rest))
}

fn map_entries(map: &str) -> Vec<MapEntry> {
    let mut block = Block::Other;
    let mut skill: Option<String> = None;
    let mut entries = Vec::new();
    for line in map.lines().filter(|line| !is_blank_or_comment(line)) {
        if !line.starts_with([' ', '\t']) {
            block = top_level_block(line);
            skill = None;
            continue;
        }
        match block {
            Block::Required if is_any_item(line) => {
                entries.push(MapEntry::Required(map_item(line)))
            }
            Block::Skills => {
                if let Some(name) = skill_heading(line) {
                    skill = Some(name.to_string());
                    entries.push(MapEntry::Skill(name.to_string(), None));
                } else if let Some(name) = skill.as_ref().filter(|_| is_map_item(line)) {
                    entries.push(MapEntry::Skill(name.clone(), Some(map_item(line))));
                }
            }
            _ => {}
        }
    }
    entries
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Edit {
    Add,
    Remove,
}

/// Alvos separados por vírgula, como o `read -a` com `IFS=,`: o campo vazio do fim não conta.
fn split_targets(targets: &str) -> Vec<&str> {
    let line = targets.split('\n').next().unwrap_or("");
    let mut list: Vec<&str> = line.split(',').collect();
    if list.last() == Some(&"") {
        list.pop();
    }
    list
}

/// Inclui ou retira `path` da entrada `entry` (`required` ou uma skill) do mapa.
/// `None` quando não há o que mudar: o item já estava (inclusão) ou não estava (retirada).
fn edit_map(map: &str, edit: Edit, entry: &str, path: &str) -> Result<Option<String>> {
    let lines = records(map);
    let targets_required = entry == "required";
    let mut block = Block::Other;
    let mut skill = String::new();
    let mut head = None;
    let mut last = None;
    let mut skills_last = None;
    let mut found = None;
    for (index, line) in lines.iter().enumerate() {
        if is_blank_or_comment(line) {
            continue;
        }
        if !line.starts_with([' ', '\t']) {
            block = top_level_block(line);
            skill.clear();
            if block == Block::Required && targets_required {
                head = Some(index);
                last = Some(index);
            }
            if block == Block::Skills {
                skills_last = Some(index);
            }
            continue;
        }
        if block == Block::Skills {
            if let Some(name) = skill_heading(line) {
                skill = name.to_string();
                skills_last = Some(index);
                if skill == entry {
                    head = Some(index);
                    last = Some(index);
                }
                continue;
            }
            skills_last = Some(index);
        }
        let in_target = (block == Block::Required && targets_required)
            || (block == Block::Skills && skill == entry);
        if is_map_item(line) && in_target {
            last = Some(index);
            if map_item(line) == path {
                found = Some(index);
            }
        }
    }

    let mut out: Vec<String> = Vec::new();
    match edit {
        Edit::Add if found.is_some() => return Ok(None),
        Edit::Add if !targets_required && head.is_none() => {
            let Some(skills_last) = skills_last else {
                bail!("não foi possível alterar o mapa de contexto");
            };
            for (index, line) in lines.iter().enumerate() {
                out.push(line.to_string());
                if index == skills_last {
                    out.push(format!("  {entry}:"));
                    out.push(format!("    - {path}"));
                }
            }
        }
        Edit::Add => {
            let indent = if targets_required { "  - " } else { "    - " };
            for (index, line) in lines.iter().enumerate() {
                if Some(index) == head && !targets_required {
                    if let Some(opened) = open_empty_entry(line) {
                        out.push(opened);
                        out.push(format!("    - {path}"));
                        continue;
                    }
                }
                out.push(line.to_string());
                if Some(index) == last {
                    out.push(format!("{indent}{path}"));
                }
            }
        }
        Edit::Remove => {
            let Some(found) = found else {
                return Ok(None);
            };
            let remaining = match (head, last) {
                (Some(head), Some(last)) if !targets_required => (head + 1..=last)
                    .filter(|&index| index != found && is_map_item(lines[index]))
                    .count(),
                _ => 1,
            };
            for (index, line) in lines.iter().enumerate() {
                if index == found {
                    continue;
                }
                if Some(index) == head && remaining == 0 {
                    out.push(format!("{} []", strip_trailing_blank_and_comment(line)));
                } else {
                    out.push(line.to_string());
                }
            }
        }
    }
    Ok(Some(join_records(&out)))
}

/// `nome: []` vira `nome:`. O `adjust-rules.sh` gravava `nome::` (trocava ` []` por `:`); aqui vale
/// o formato do `init-harness.sh`, o mesmo das outras entradas.
fn open_empty_entry(line: &str) -> Option<String> {
    line.match_indices("[]").find_map(|(bracket, _)| {
        let after = line[bracket + 2..].trim_start_matches([' ', '\t']);
        (after.is_empty() || after.starts_with('#'))
            .then(|| line[..bracket].trim_end_matches([' ', '\t']).to_string())
    })
}

/// Remove `[ \t]*(#.*)?$`: espaços finais ou o primeiro comentário, com os espaços antes dele.
fn strip_trailing_blank_and_comment(line: &str) -> &str {
    let without_comment = line.find('#').map_or(line, |hash| &line[..hash]);
    without_comment.trim_end_matches([' ', '\t'])
}

// --- Arquivo de categoria ---

/// Problemas de um arquivo de `rules/project/`, como `<arquivo>:<linha>: <problema>`.
fn category_problems(file: &str, content: &str) -> Vec<String> {
    let base = file.rsplit('/').next().unwrap_or(file);
    let name = base.strip_suffix(".md").unwrap_or(base);
    let mut problems = Vec::new();
    let mut report =
        |line: usize, message: String| problems.push(format!("{file}:{line}: {message}"));
    if !(base.ends_with(".md") && is_kebab_case(name)) {
        report(1, format!("nome fora de kebab-case: {base}"));
    }
    let prefix = format!("project/{name}-");
    let format_error = || format!("ID fora do formato [project/{name}-NN]: ");
    let mut purpose = false;
    let mut warned = false;
    let mut seen_rule = false;
    let mut seen: Vec<String> = Vec::new();
    let mut expected = 0;
    let lines = records(content);
    for (index, line) in lines.iter().enumerate() {
        let number = index + 1;
        if number == 1 && !is_title(line) {
            report(
                1,
                "primeira linha deve ser o título \"# <título>\"".to_string(),
            );
        }
        if let Some((id, rest)) = rule_parts(line) {
            if !purpose && !warned {
                report(
                    number,
                    "falta a frase de propósito antes da primeira regra".to_string(),
                );
                warned = true;
            }
            let rest_ok = rest.len() > 3 && rest.starts_with("]: ");
            if !(rest_ok && (has_id_shape(id) || id.starts_with("project/"))) {
                report(number, format!("{}{id}", format_error()));
                continue;
            }
            let digits = id
                .strip_prefix(&prefix)
                .filter(|digits| digits.len() == 2 && digits.bytes().all(|b| b.is_ascii_digit()));
            let Some(digits) = digits else {
                report(number, format!("{}{id}", format_error()));
                continue;
            };
            if seen.iter().any(|known| known == id) {
                report(number, format!("ID repetido: {id}"));
                continue;
            }
            seen.push(id.to_string());
            let value: u32 = digits.parse().unwrap_or(0);
            if value != expected + 1 {
                report(
                    number,
                    format!(
                        "ID fora de sequência: esperado {:02}, encontrado {value:02}",
                        expected + 1
                    ),
                );
            }
            expected = value;
            seen_rule = true;
            continue;
        }
        let blank = line.trim_start_matches([' ', '\t']).is_empty();
        if number > 1 && !seen_rule && !blank && !line.starts_with('#') {
            purpose = true;
        }
    }
    if !lines.is_empty() && !purpose && !warned {
        report(
            lines.len(),
            "falta a frase de propósito antes da primeira regra".to_string(),
        );
    }
    problems
}

/// `^# [^ \t]`
fn is_title(line: &str) -> bool {
    line.strip_prefix("# ")
        .and_then(|rest| rest.chars().next())
        .is_some_and(|first| first != ' ' && first != '\t')
}

/// Linha de regra (`^\[[^]]*\]:`): o ID entre colchetes e o resto a partir do `]`.
fn rule_parts(line: &str) -> Option<(&str, &str)> {
    let inner = line.strip_prefix('[')?;
    let close = inner.find(']')?;
    let rest = &inner[close..];
    rest.starts_with("]:").then(|| (&inner[..close], rest))
}

/// `<escopo>/<nome>-NN`, com escopo sem `/` e nome em `[a-z0-9-]`.
fn has_id_shape(id: &str) -> bool {
    let Some((scope, name)) = id.split_once('/') else {
        return false;
    };
    let bytes = name.as_bytes();
    let well_formed = name
        .bytes()
        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-');
    !scope.is_empty()
        && well_formed
        && bytes.len() >= 4
        && bytes[bytes.len() - 3] == b'-'
        && bytes[bytes.len() - 2..].iter().all(u8::is_ascii_digit)
}

/// Maior número de regra da categoria, contando as removidas (`[project/<categoria>-NN]: `).
fn highest_number(content: &str, category: &str) -> u32 {
    let prefix = format!("[project/{category}-");
    content
        .lines()
        .filter_map(|line| line.strip_prefix(&prefix))
        .filter(|rest| rest.len() >= 5 && rest.is_char_boundary(2) && &rest[2..5] == "]: ")
        .filter_map(|rest| {
            let digits = &rest[..2];
            digits
                .bytes()
                .all(|b| b.is_ascii_digit())
                .then(|| digits.parse().ok())
                .flatten()
        })
        .max()
        .unwrap_or(0)
}

/// `^\[<id>\]: Removida em AAAA-MM-DD\.$`
fn is_removed_rule(line: &str, id: &str) -> bool {
    line.strip_prefix(id)
        .and_then(|rest| rest.strip_prefix("Removida em "))
        .and_then(|rest| rest.strip_suffix('.'))
        .is_some_and(|date| {
            let bytes = date.as_bytes();
            bytes.len() == 10
                && bytes.iter().enumerate().all(|(index, b)| match index {
                    4 | 7 => *b == b'-',
                    _ => b.is_ascii_digit(),
                })
        })
}

/// Sem o comentário `<!-- ... -->` de abertura do template, que só documenta o formato.
fn without_leading_comment(template: &str) -> String {
    let mut skipping = false;
    let mut started = false;
    let mut out = Vec::new();
    for line in records(template) {
        if !started && line.starts_with("<!--") {
            skipping = true;
        }
        if skipping {
            if line.contains("-->") {
                skipping = false;
                started = true;
            }
            continue;
        }
        started = true;
        out.push(line);
    }
    join_records(&out)
}

/// O arquivo de regras de `write` e `preview`.
struct RulesRequest {
    title: String,
    purpose: String,
    removals: Vec<u32>,
    rules: Vec<String>,
}

impl RulesRequest {
    fn parse(content: &str) -> Result<Self> {
        let mut request = Self {
            title: String::new(),
            purpose: String::new(),
            removals: Vec::new(),
            rules: Vec::new(),
        };
        for line in records(content) {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some(title) = line.strip_prefix("titulo:") {
                request.title = title.trim().to_string();
            } else if let Some(purpose) = line.strip_prefix("proposito:") {
                request.purpose = purpose.trim().to_string();
            } else if let Some(number) = line.strip_prefix("remover:") {
                let number: String = number.chars().filter(|c| !c.is_whitespace()).collect();
                if number.is_empty() || !number.bytes().all(|b| b.is_ascii_digit()) {
                    bail!("remover: espera o número da regra: '{line}'");
                }
                request.removals.push(number.parse()?);
            } else {
                let text = line.trim_start();
                let text = text.strip_prefix('-').map_or(text, str::trim_start).trim();
                if !text.is_empty() {
                    request.rules.push(text.to_string());
                }
            }
        }
        Ok(request)
    }
}

/// Categoria existente: marca as remoções e acrescenta as regras novas com os próximos IDs.
fn update_category(category: &str, current: &str, request: &RulesRequest) -> String {
    let prefix = format!("[project/{category}-");
    let number_of = |line: &str| -> Option<u32> {
        let rest = line.strip_prefix(&prefix)?;
        let leading: String = rest
            .chars()
            .take(2)
            .take_while(char::is_ascii_digit)
            .collect();
        Some(leading.parse().unwrap_or(0))
    };
    let mut next = records(current)
        .into_iter()
        .filter_map(number_of)
        .max()
        .unwrap_or(0)
        + 1;
    let today = utc_date();
    let mut out = join_records(
        &records(current)
            .into_iter()
            .map(|line| match number_of(line) {
                Some(number)
                    if request.removals.contains(&number) && !line.contains("]: Removida em ") =>
                {
                    format!("{prefix}{number:02}]: Removida em {today}.")
                }
                _ => line.to_string(),
            })
            .collect::<Vec<_>>(),
    );
    for text in &request.rules {
        let exists = out.lines().any(|line| {
            line.starts_with(&prefix)
                && line
                    .find("]: ")
                    .map_or(line.get(2..), |position| line.get(position + 3..))
                    == Some(text.as_str())
        });
        if exists {
            continue;
        }
        if !out.is_empty() && !out.ends_with('\n') {
            out.push('\n');
        }
        out.push_str(&format!("{prefix}{next:02}]: {text}\n"));
        next += 1;
    }
    out
}

/// Categoria nova pelo template, com as regras numeradas a partir de 01.
fn new_category(category: &str, template: &str, request: RulesRequest) -> String {
    let title = if request.title.is_empty() {
        category.to_string()
    } else {
        request.title
    };
    let purpose = if request.purpose.is_empty() {
        format!("Define as regras de {category} para o agente.")
    } else {
        request.purpose
    };
    let mut out = Vec::new();
    let mut skipping = false;
    for (index, line) in records(template).into_iter().enumerate() {
        if index == 0 && line.starts_with("<!--") {
            skipping = true;
        }
        if skipping {
            skipping = !line.contains("-->");
            continue;
        }
        match line {
            "# #{CATEGORY_TITLE}" => out.push(format!("# {title}")),
            "#{CATEGORY_PURPOSE}" => out.push(purpose.clone()),
            "#{CATEGORY_RULES}" => {
                out.extend(request.rules.iter().enumerate().map(|(position, rule)| {
                    format!("[project/{category}-{:02}]: {rule}", position + 1)
                }))
            }
            other => out.push(other.to_string()),
        }
    }
    join_records(&out)
}

/// Diferença unificada (`diff -u`) entre a categoria atual e a nova, pelo `diff` do sistema.
fn unified_diff(current: &Path, built: &str) -> Result<String> {
    let temp = std::env::temp_dir().join(format!("tmp.{}", unique_suffix()));
    write(&temp, built)?;
    let output = Command::new("diff")
        .arg("-u")
        .arg(current)
        .arg(&temp)
        .output();
    let _ = fs::remove_file(&temp);
    let output = output.context("não foi possível executar o diff")?;
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn unique_suffix() -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.subsec_nanos());
    format!("{}{nanos}", std::process::id())
}

// --- Sinais do projeto ---

/// Pastas e arquivos que a detecção não percorre.
const PRUNED: &[&str] = &[
    "node_modules",
    ".git",
    ".claude",
    ".agents",
    ".venv",
    "target",
    "dist",
    "build",
    "work",
];
const HARNESS_SOURCE: &str = "packages/harness";
const MAX_DEPTH: usize = 3;

fn detect_in(root: &Path, relative: &str, depth: usize, signals: &mut Vec<String>) {
    let Ok(entries) = fs::read_dir(root.join(relative)) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        let rel = if relative.is_empty() {
            name.clone()
        } else {
            format!("{relative}/{name}")
        };
        if PRUNED.contains(&name.as_str()) || rel == HARNESS_SOURCE {
            continue;
        }
        let Ok(kind) = entry.file_type() else {
            continue;
        };
        if kind.is_dir() {
            signals.extend(dir_signal(&rel, &name));
            if depth < MAX_DEPTH {
                detect_in(root, &rel, depth + 1, signals);
            }
        } else if kind.is_file() {
            signals.extend(file_signals(&root.join(&rel), &rel, &name));
        }
    }
}

fn dir_signal(rel: &str, name: &str) -> Option<String> {
    let (category, reason) = match name {
        "src" | "apps" | "packages" | "services" | "libs" | "lib" | "cmd" | "internal" => {
            ("architecture", "pasta de código")
        }
        "tests" | "test" | "__tests__" => ("testing", "pasta de testes"),
        "migrations" | "prisma" => ("database", "pasta de banco de dados"),
        "routes" | "controllers" => ("api", "pasta de rotas"),
        "components" | "pages" => ("frontend", "pasta de interface"),
        "workflows" if rel.ends_with(".github/workflows") => {
            ("development-methodology", "integração contínua")
        }
        ".husky" => ("development-methodology", "hooks de commit"),
        _ => return None,
    };
    Some(format!("{category}: {rel}: {reason}"))
}

const LINT_CONFIGS: &[&str] = &[
    ".eslintrc",
    ".eslintrc.*",
    "eslint.config.*",
    ".prettierrc",
    ".prettierrc.*",
    "prettier.config.*",
    ".editorconfig",
    "rustfmt.toml",
    ".rustfmt.toml",
    "ruff.toml",
    ".flake8",
    ".golangci.yml",
    "biome.json",
    ".swiftlint.yml",
];
const TEST_CONFIGS: &[&str] = &[
    "vitest.config.*",
    "jest.config.*",
    "pytest.ini",
    "playwright.config.*",
    "cypress.config.*",
    "phpunit.xml",
    "karma.conf.*",
];

fn file_signals(path: &Path, rel: &str, name: &str) -> Vec<String> {
    let signal = |category: &str, reason: &str| format!("{category}: {rel}: {reason}");
    let stack = |language: &str| vec![signal("stack", language)];
    let matches_any = |patterns: &[&str]| patterns.iter().any(|pattern| glob_match(pattern, name));
    match name {
        "package.json" => package_json_signals(&read_lossy(path), &signal),
        "Cargo.toml" => stack("Rust"),
        "go.mod" => stack("Go"),
        "pyproject.toml" => pyproject_signals(&read_lossy(path), &signal),
        "requirements.txt" => stack("Python"),
        "pom.xml" | "build.gradle" | "build.gradle.kts" => stack("Java/Kotlin"),
        "Gemfile" => stack("Ruby"),
        "composer.json" => stack("PHP"),
        "Package.swift" => stack("Swift"),
        "pubspec.yaml" => stack("Dart/Flutter"),
        _ if name.ends_with(".csproj") => stack(".NET"),
        "Dockerfile" | "docker-compose.yml" | "docker-compose.yaml" => stack("Docker"),
        _ if matches_any(LINT_CONFIGS) => {
            vec![signal("code-style", "configuração de lint ou formatação")]
        }
        _ if matches_any(TEST_CONFIGS) => vec![signal("testing", "configuração de teste")],
        "schema.prisma" => vec![signal("database", "esquema do Prisma")],
        _ if matches_any(&["openapi.*", "swagger.*"]) => vec![signal("api", "contrato da API")],
        "CONTRIBUTING.md" | ".pre-commit-config.yaml" | ".gitlab-ci.yml" | "Jenkinsfile" => {
            vec![signal(
                "development-methodology",
                "processo de contribuição ou integração",
            )]
        }
        "SECURITY.md" | ".env.example" | ".gitleaks.toml" | ".snyk" | "dependabot.yml" => {
            vec![signal("security", "política ou verificação de segurança")]
        }
        _ => Vec::new(),
    }
}

fn package_json_signals(content: &str, signal: &dyn Fn(&str, &str) -> String) -> Vec<String> {
    let has_any = |names: &[&str]| {
        names
            .iter()
            .any(|name| content.contains(&format!("\"{name}\"")))
    };
    let mut signals = vec![signal("stack", "Node.js")];
    let checks: [(&[&str], &str, &str); 6] = [
        (&["typescript"], "stack", "TypeScript"),
        (
            &["eslint", "prettier", "@biomejs/biome"],
            "code-style",
            "eslint/prettier/biome nas dependências",
        ),
        (
            &["vitest", "jest", "mocha", "playwright", "cypress"],
            "testing",
            "framework de teste nas dependências",
        ),
        (
            &[
                "prisma",
                "@prisma/client",
                "typeorm",
                "sequelize",
                "mongoose",
                "knex",
                "drizzle-orm",
            ],
            "database",
            "ORM ou driver de banco nas dependências",
        ),
        (
            &["react", "vue", "svelte", "@angular/core", "next", "nuxt"],
            "frontend",
            "framework de interface nas dependências",
        ),
        (
            &["express", "fastify", "koa", "hono", "@nestjs/core"],
            "api",
            "framework HTTP nas dependências",
        ),
    ];
    for (names, category, reason) in checks {
        if has_any(names) {
            signals.push(signal(category, reason));
        }
    }
    signals
}

fn pyproject_signals(content: &str, signal: &dyn Fn(&str, &str) -> String) -> Vec<String> {
    let lower = content.to_lowercase();
    let has_any = |text: &str, names: &[&str]| names.iter().any(|name| text.contains(name));
    let mut signals = vec![signal("stack", "Python")];
    if has_any(content, &["ruff", "black", "flake8"]) {
        signals.push(signal("code-style", "ruff/black/flake8"));
    }
    if has_any(content, &["pytest"]) {
        signals.push(signal("testing", "pytest"));
    }
    if has_any(&lower, &["sqlalchemy", "django"]) {
        signals.push(signal("database", "ORM"));
    }
    if has_any(&lower, &["fastapi", "flask", "django"]) {
        signals.push(signal("api", "framework HTTP"));
    }
    signals
}

/// Padrão de nome com `*` como coringa, como nos `case` do bash.
fn glob_match(pattern: &str, name: &str) -> bool {
    let parts: Vec<&str> = pattern.split('*').collect();
    let (first, rest) = parts
        .split_first()
        .expect("split devolve ao menos uma parte");
    let Some(mut remaining) = name.strip_prefix(first) else {
        return false;
    };
    let Some((last, middle)) = rest.split_last() else {
        return remaining.is_empty();
    };
    for part in middle {
        match remaining.find(part) {
            Some(position) => remaining = &remaining[position + part.len()..],
            None => return false,
        }
    }
    remaining.ends_with(last)
}

/// Arquivos `.md` de uma pasta e das subpastas.
fn collect_markdown(dir: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let Ok(kind) = entry.file_type() else {
            continue;
        };
        let path = entry.path();
        if kind.is_dir() {
            collect_markdown(&path, files);
        } else if kind.is_file() && entry.file_name().to_string_lossy().ends_with(".md") {
            files.push(path);
        }
    }
}

fn read_lossy(path: &Path) -> String {
    String::from_utf8_lossy(&fs::read(path).unwrap_or_default()).into_owned()
}

fn read(path: &Path) -> Result<String> {
    fs::read_to_string(path).with_context(|| format!("não foi possível ler {}", path.display()))
}

fn write(path: &Path, content: &str) -> Result<()> {
    fs::write(path, content).with_context(|| format!("não foi possível gravar {}", path.display()))
}
