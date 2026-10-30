//! Roteamento do projeto para as ferramentas de agente: o `AGENTS.md`, o arquivo de roteamento de
//! cada ferramenta, os redirecionamentos das skills, as definições de subagente das personas e o
//! catálogo (RF-06, RF-12, RF-19, RF-25).

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};

use crate::commands::skill::{skill_dir, update_catalog, write_redirects, Scope};
use crate::embedded;
use crate::harness::Harness;
use crate::manifest::catalog_path;
use crate::support::{config_get, config_list, frontmatter_get, yaml_escape};

/// Formato do arquivo de definição de subagente de uma ferramenta.
#[derive(Debug, PartialEq, Eq)]
pub enum AgentFormat {
    /// Markdown com front-matter YAML e o corpo como instruções.
    Markdown,
    /// TOML com `name`, `description` e `developer_instructions`.
    Toml,
}

/// Onde e como uma ferramenta lê as definições de subagente.
#[derive(Debug, PartialEq, Eq)]
pub struct AgentTarget {
    pub dir: &'static str,
    pub extension: &'static str,
    pub format: AgentFormat,
}

/// Ferramenta de agente suportada e o que ela recebe no projeto.
#[derive(Debug, PartialEq, Eq)]
pub struct Tool {
    pub name: &'static str,
    pub label: &'static str,
    /// Pasta dos redirecionamentos de skills, relativa à raiz do projeto.
    pub redirect_dir: &'static str,
    /// Arquivo de roteamento próprio da ferramenta, além do `AGENTS.md`.
    pub routing_file: Option<&'static str>,
    /// Pasta e formato das definições de subagente das personas, relativos à raiz do projeto;
    /// `None` quando o estudo não aponta a ferramenta como compatível (RF-12,
    /// `docs/estudos/multiagentes.md`).
    pub agents: Option<AgentTarget>,
    /// Ferramenta cuja pasta de definições esta também lê; com ela registrada, esta não recebe
    /// definições próprias, para não duplicar as personas.
    pub agents_read_from: Option<&'static str>,
}

pub const TOOLS: [Tool; 3] = [
    Tool {
        name: "claude-code",
        label: "Claude Code",
        redirect_dir: ".claude/skills",
        routing_file: Some("CLAUDE.md"),
        agents: Some(AgentTarget {
            dir: ".claude/agents",
            extension: "md",
            format: AgentFormat::Markdown,
        }),
        agents_read_from: None,
    },
    Tool {
        name: "codex",
        label: "Codex",
        redirect_dir: ".agents/skills",
        routing_file: None,
        agents: Some(AgentTarget {
            dir: ".codex/agents",
            extension: "toml",
            format: AgentFormat::Toml,
        }),
        agents_read_from: None,
    },
    Tool {
        name: "cursor",
        label: "Cursor",
        redirect_dir: ".cursor/skills",
        routing_file: None,
        agents: Some(AgentTarget {
            dir: ".cursor/agents",
            extension: "md",
            format: AgentFormat::Markdown,
        }),
        agents_read_from: Some("claude-code"),
    },
];

const AGENTS_FILE: &str = "AGENTS.md";
const AGENTS_CONTENT: &str = "> Este arquivo é o roteamento para o harness do projeto.\n> Execute `vostok context load` para carregar o contexto obrigatório.\n";
const TOOL_ROUTING_CONTENT: &str = "> Este arquivo é o roteamento para o harness do projeto.\n> Leia o arquivo `AGENTS.md` para carregar o harness.\n";
const LOCAL_SUFFIX: &str = ".local";
const SKILL_FILE: &str = "SKILL.md";
const PERSONAS_DIR: &str = "personas";
/// O Coordenador é o chat principal e não vira subagente.
const COORDINATOR: &str = "coordenador";
/// Identifica os arquivos gerados e evita colisão com os agentes do projeto.
const AGENT_PREFIX: &str = "vostok-";

/// Resultado da geração do roteamento.
#[derive(Debug, Default)]
pub struct Routing {
    /// Arquivos criados ou alterados.
    pub changed: Vec<PathBuf>,
    /// Versões anteriores guardadas como `.local`.
    pub kept_locals: Vec<PathBuf>,
    /// Skills customizadas com o nome de uma skill padrão.
    pub conflicts: Vec<String>,
    /// Definições de subagente criadas ou alteradas.
    pub agents: Vec<PathBuf>,
}

/// Persona especialista, lida do front-matter de `personas/<nome>.md`.
struct Persona {
    name: String,
    title: String,
    description: String,
}

/// Nomes das ferramentas suportadas, para as mensagens de erro.
pub fn supported_tools() -> String {
    TOOLS
        .iter()
        .map(|tool| tool.name)
        .collect::<Vec<_>>()
        .join(", ")
}

/// Ferramentas de uma lista separada por vírgulas ou espaços, na ordem da tabela, sem repetição (CB-08).
pub fn parse_tools(list: &str) -> Result<Vec<&'static Tool>> {
    let mut chosen = Vec::new();
    for name in list.split([',', ' ']).filter(|name| !name.is_empty()) {
        let tool = TOOLS.iter().find(|tool| tool.name == name).ok_or_else(|| {
            anyhow!(
                "ferramenta não suportada: '{name}'; as suportadas são: {}",
                supported_tools()
            )
        })?;
        chosen.push(tool);
    }
    if chosen.is_empty() {
        bail!(
            "nenhuma ferramenta escolhida; as suportadas são: {}",
            supported_tools()
        );
    }
    Ok(TOOLS.iter().filter(|tool| chosen.contains(tool)).collect())
}

/// Grava as pastas de redirecionamento das ferramentas em `redirect_dirs` do `config/skills.yml` (RF-25).
pub fn save_tools(harness: &Harness, tools: &[&Tool]) -> Result<()> {
    let path = harness.config_file("skills.yml");
    let content = read(&path)?;
    let mut lines = Vec::new();
    let mut found = false;
    let mut replacing = false;
    for line in content.lines() {
        if replacing && line.starts_with([' ', '\t']) {
            continue;
        }
        replacing = false;
        lines.push(line.to_string());
        if line.trim_end() == "redirect_dirs:" {
            found = true;
            replacing = true;
            lines.extend(
                tools
                    .iter()
                    .map(|tool| format!("  - {}   # {}", tool.redirect_dir, tool.label)),
            );
        }
    }
    if !found {
        bail!("Lista 'redirect_dirs' não encontrada em {}", path.display());
    }
    write(&path, &(lines.join("\n") + "\n"))
}

/// Raiz do projeto, de `paths.root` do `config/main.yml`.
pub fn project_root(harness: &Harness) -> Result<PathBuf> {
    let root = harness.path("root")?;
    crate::support::existing_dir(&root)
        .with_context(|| format!("raiz do projeto inexistente: {}", root.display()))
}

/// Gera o `AGENTS.md`, os arquivos de roteamento, os redirecionamentos e as definições de subagente
/// das personas das ferramentas registradas em `redirect_dirs` e atualiza o catálogo com as skills
/// instaladas.
pub fn generate(harness: &Harness) -> Result<Routing> {
    let root = project_root(harness)?;
    let redirect_dirs: Vec<String> =
        config_list(&harness.config_file("skills.yml"), "redirect_dirs")?
            .into_iter()
            .filter(|dir| !dir.is_empty())
            .collect();
    let catalog = harness.root.join(catalog_path(harness)?);
    let agent_dirs: Vec<PathBuf> = agent_targets(&registered_tools(&redirect_dirs))
        .iter()
        .map(|(_, target)| root.join(target.dir))
        .collect();
    let watched: Vec<PathBuf> = routing_files(&redirect_dirs)
        .map(|file| root.join(file))
        .chain(redirect_dirs.iter().map(|dir| root.join(dir)))
        .chain(agent_dirs.iter().cloned())
        .chain([catalog.clone()])
        .collect();
    let before = snapshot(&watched);

    let mut routing = Routing::default();
    for file in routing_files(&redirect_dirs) {
        let content = if file == AGENTS_FILE {
            AGENTS_CONTENT
        } else {
            TOOL_ROUTING_CONTENT
        };
        if let Some(local) = write_routing_file(&root.join(file), content)? {
            routing.kept_locals.push(local);
        }
    }
    restore_catalog(harness, &catalog)?;
    routing.conflicts = write_skills(harness)?;
    let agent_locals = write_agents(harness, &root, &redirect_dirs)?;

    let after = snapshot(&watched);
    for (path, content) in &after {
        if before.get(path) == Some(content) || agent_locals.contains(path) {
            continue;
        }
        if agent_dirs.iter().any(|dir| path.starts_with(dir)) {
            routing.agents.push(path.clone());
        } else {
            routing.changed.push(path.clone());
        }
    }
    routing.kept_locals.extend(agent_locals);
    Ok(routing)
}

/// Ferramentas de `TOOLS` cujo `redirect_dir` está em `redirect_dirs`.
fn registered_tools(redirect_dirs: &[String]) -> Vec<&'static Tool> {
    TOOLS
        .iter()
        .filter(|tool| redirect_dirs.iter().any(|dir| dir == tool.redirect_dir))
        .collect()
}

/// Ferramentas que recebem definições de subagente entre as registradas, com o destino de cada uma:
/// as compatíveis, menos as que já leem a pasta de outra ferramenta registrada (RF-12).
pub fn agent_targets(tools: &[&'static Tool]) -> Vec<(&'static Tool, &'static AgentTarget)> {
    tools
        .iter()
        .filter(|tool| {
            tool.agents_read_from
                .is_none_or(|other| !tools.iter().any(|registered| registered.name == other))
        })
        .filter_map(|tool| tool.agents.as_ref().map(|target| (*tool, target)))
        .collect()
}

/// Grava uma definição de subagente por persona especialista para cada ferramenta que as recebe;
/// uma versão diferente que já existia é guardada como `.local`, cujos caminhos são devolvidos.
fn write_agents(harness: &Harness, root: &Path, redirect_dirs: &[String]) -> Result<Vec<PathBuf>> {
    let targets = agent_targets(&registered_tools(redirect_dirs));
    if targets.is_empty() {
        return Ok(Vec::new());
    }
    let personas = personas(harness)?;
    let mut locals = Vec::new();
    for (_, target) in targets {
        for persona in &personas {
            let file = format!("{AGENT_PREFIX}{}.{}", persona.name, target.extension);
            let content = render_agent(&target.format, persona);
            if let Some(local) = write_routing_file(&root.join(target.dir).join(file), &content)? {
                locals.push(local);
            }
        }
    }
    Ok(locals)
}

/// Personas especialistas da pasta `personas` do harness, sem o Coordenador, em ordem alfabética.
fn personas(harness: &Harness) -> Result<Vec<Persona>> {
    let Ok(entries) = fs::read_dir(harness.root.join(PERSONAS_DIR)) else {
        return Ok(Vec::new());
    };
    let mut paths: Vec<PathBuf> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "md"))
        .filter(|path| path.file_stem().is_some_and(|stem| stem != COORDINATOR))
        .collect();
    paths.sort();
    paths
        .iter()
        .map(|path| {
            Ok(Persona {
                name: frontmatter_get(path, "name")?,
                title: frontmatter_get(path, "title")?,
                description: frontmatter_get(path, "description")?,
            })
        })
        .collect()
}

/// Definição de subagente da persona no formato da ferramenta. Sem `tools` e com o modelo da sessão:
/// o modelo é escolhido no disparo, pelo perfil da tarefa.
fn render_agent(format: &AgentFormat, persona: &Persona) -> String {
    let name = &persona.name;
    let instructions = format!(
        "Persona {} do harness Vostok, como subagente.\n\
         Antes de qualquer outra ação, execute `vostok context load --persona {name}` e siga a persona e o contexto carregados.\n\
         A tarefa, o que entregar e o modelo vêm da mensagem do Coordenador que disparou este subagente.\n",
        persona.title
    );
    // As aspas duplas do YAML e as strings básicas do TOML escapam `\` e `"` do mesmo jeito.
    let description = yaml_escape(&persona.description);
    match format {
        AgentFormat::Markdown => {
            let quoted: String = instructions
                .lines()
                .map(|line| format!("> {line}\n"))
                .collect();
            format!(
                "---\nname: {AGENT_PREFIX}{name}\ndescription: \"{description}\"\nmodel: inherit\n---\n\n{quoted}"
            )
        }
        AgentFormat::Toml => format!(
            "name = \"{AGENT_PREFIX}{name}\"\ndescription = \"{description}\"\ndeveloper_instructions = \"\"\"\n{instructions}\"\"\"\n"
        ),
    }
}

/// `AGENTS.md`, sempre, e o arquivo de roteamento de cada ferramenta registrada que tem um.
fn routing_files(redirect_dirs: &[String]) -> impl Iterator<Item = &'static str> + '_ {
    let tool_files = TOOLS
        .iter()
        .filter(|tool| redirect_dirs.iter().any(|dir| dir == tool.redirect_dir))
        .filter_map(|tool| tool.routing_file);
    [AGENTS_FILE].into_iter().chain(tool_files)
}

/// Grava o arquivo de roteamento; uma versão diferente que já existia é guardada como `.local` (CB-01).
fn write_routing_file(path: &Path, content: &str) -> Result<Option<PathBuf>> {
    let local = match fs::read(path) {
        Ok(current) if current == content.as_bytes() => return Ok(None),
        Ok(_) => Some(keep_local(path)?),
        Err(_) => None,
    };
    write(path, content)?;
    Ok(local)
}

/// Copia o arquivo para `<arquivo>.local` e devolve o caminho da cópia.
pub fn keep_local(path: &Path) -> Result<PathBuf> {
    let mut local = path.as_os_str().to_owned();
    local.push(LOCAL_SUFFIX);
    let local = PathBuf::from(local);
    fs::copy(path, &local)
        .with_context(|| format!("não foi possível guardar {}", local.display()))?;
    Ok(local)
}

/// Recria o catálogo a partir do binário quando ele foi apagado.
fn restore_catalog(harness: &Harness, catalog: &Path) -> Result<()> {
    if catalog.is_file() {
        return Ok(());
    }
    let relative = catalog_path(harness)?;
    let content = embedded::file(&relative)
        .ok_or_else(|| anyhow!("catálogo de skills não encontrado: {}", catalog.display()))?;
    write(catalog, &String::from_utf8_lossy(content))
}

/// Redirecionamentos e linhas do catálogo de todas as skills instaladas. Uma skill customizada com o
/// nome de uma padrão prevalece no redirecionamento; os nomes em conflito são devolvidos (CB-05).
fn write_skills(harness: &Harness) -> Result<Vec<String>> {
    let defaults = installed_skills(harness, Scope::Default)?;
    let customs = installed_skills(harness, Scope::Custom)?;
    let conflicts: Vec<String> = defaults
        .iter()
        .filter(|name| customs.contains(name))
        .cloned()
        .collect();
    for (scope, names) in [(Scope::Default, &defaults), (Scope::Custom, &customs)] {
        for name in names {
            let file = skill_dir(harness, scope, name)?.join(SKILL_FILE);
            if scope == Scope::Default && conflicts.contains(name) {
                update_catalog(harness, name, scope, &file)?;
                continue;
            }
            write_redirects(harness, name, &file)?;
            update_catalog(harness, name, scope, &file)?;
        }
    }
    Ok(conflicts)
}

/// Nomes das skills com `SKILL.md` na pasta do escopo, em ordem alfabética.
fn installed_skills(harness: &Harness, scope: Scope) -> Result<Vec<String>> {
    let dir = config_get(
        &harness.config_file("skills.yml"),
        &format!("dirs.{}", scope.key()),
    )?;
    let Ok(entries) = fs::read_dir(harness.resolve(&dir)) else {
        return Ok(Vec::new());
    };
    let mut names: Vec<String> = entries
        .flatten()
        .filter(|entry| entry.path().join(SKILL_FILE).is_file())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    Ok(names)
}

/// Conteúdo de cada arquivo sob os caminhos (arquivos ou pastas), para saber o que mudou.
fn snapshot(paths: &[PathBuf]) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut files = BTreeMap::new();
    for path in paths {
        collect(path, &mut files);
    }
    files
}

fn collect(path: &Path, files: &mut BTreeMap<PathBuf, Vec<u8>>) {
    if path.is_dir() {
        for entry in fs::read_dir(path).into_iter().flatten().flatten() {
            collect(&entry.path(), files);
        }
    } else if let Ok(content) = fs::read(path) {
        files.insert(path.to_path_buf(), content);
    }
}

fn read(path: &Path) -> Result<String> {
    fs::read_to_string(path).with_context(|| format!("não foi possível ler {}", path.display()))
}

fn write(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("não foi possível criar {}", parent.display()))?;
    }
    fs::write(path, content).with_context(|| format!("não foi possível gravar {}", path.display()))
}
