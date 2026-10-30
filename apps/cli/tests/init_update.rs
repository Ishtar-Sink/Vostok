//! Testes do `vostok init` e do `vostok update`: instalação, roteamento por ferramenta, manifesto,
//! atualização que preserva o projeto e os casos de borda (CT-03 a CT-09, CT-14, CT-19, CB-01,
//! CB-03 a CB-06 e CB-08).

mod common;

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use assert_cmd::Command;
use common::{failure_stderr, vostok_at, Project};
use tempfile::TempDir;
use vostok::embedded;
use vostok::manifest::Manifest;
use vostok::support::{config_get, config_list, frontmatter_get, sha256_hex};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const SUPPORTED: &str = "claude-code, codex, cursor";
const FORCE_INTERACTIVE: &str = "VOSTOK_FORCE_INTERACTIVE";
const DEFAULT_SKILL: &str = "vostok/skills/default/refine-spec/SKILL.md";
const CATALOG: &str = "vostok/context/skill-catalog.md";
const AGENTS_MD: &str = "> Este arquivo é o roteamento para o harness do projeto.\n> Execute `vostok context load` para carregar o contexto obrigatório.\n";
const CUSTOM_SKILL: &str = "---\nname: my-skill\ndescription: \"Faz algo útil. Use quando pedirem.\"\n---\n\n# Minha skill\n";

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|_| panic!("ler {}", path.display()))
}

fn write(path: &Path, content: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

fn stdout(command: &mut Command) -> String {
    let output = command.assert().success().get_output().stdout.clone();
    String::from_utf8(output).unwrap()
}

/// Todos os arquivos sob `dir`, com o conteúdo, para comparar a árvore antes e depois.
fn tree(dir: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut files = BTreeMap::new();
    collect(dir, dir, &mut files);
    files
}

fn collect(root: &Path, dir: &Path, files: &mut BTreeMap<PathBuf, Vec<u8>>) {
    for entry in fs::read_dir(dir).unwrap().flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(root, &path, files);
        } else {
            let relative = path.strip_prefix(root).unwrap().to_path_buf();
            files.insert(relative, fs::read(&path).unwrap());
        }
    }
}

/// Pasta temporária vazia, ainda sem o harness.
struct Workspace {
    dir: TempDir,
}

impl Workspace {
    fn new() -> Self {
        Self {
            dir: TempDir::new().expect("criar pasta temporária"),
        }
    }

    fn root(&self) -> &Path {
        self.dir.path()
    }

    fn path(&self, relative: &str) -> PathBuf {
        self.root().join(relative)
    }

    fn exists(&self, relative: &str) -> bool {
        self.path(relative).exists()
    }

    fn read(&self, relative: &str) -> String {
        read(&self.path(relative))
    }

    fn write(&self, relative: &str, content: &str) {
        write(&self.path(relative), content);
    }

    fn vostok(&self) -> Command {
        vostok_at(self.root())
    }

    fn init(&self, args: &[&str]) -> String {
        stdout(self.vostok().arg("init").args(args))
    }

    fn init_fails(&self, args: &[&str]) -> String {
        failure_stderr(self.vostok().arg("init").args(args))
    }

    fn interactive(&self, answers: &str) -> Command {
        let mut command = self.vostok();
        command
            .arg("init")
            .env(FORCE_INTERACTIVE, "1")
            .write_stdin(answers);
        command
    }

    fn update(&self) -> String {
        stdout(self.vostok().arg("update"))
    }

    fn config(&self, file: &str, key: &str) -> String {
        config_get(&self.path(&format!("vostok/config/{file}")), key).unwrap()
    }

    fn redirect_dirs(&self) -> Vec<String> {
        config_list(&self.path("vostok/config/skills.yml"), "redirect_dirs").unwrap()
    }

    fn embedded(&self, relative: &str) -> String {
        String::from_utf8(embedded::file(relative).unwrap().to_vec()).unwrap()
    }

    fn replace(&self, relative: &str, from: &str, to: &str) {
        let content = self.read(relative);
        assert!(content.contains(from), "{relative} não contém {from:?}");
        self.write(relative, &content.replacen(from, to, 1));
    }
}

// --- init ---

#[test]
fn ct03_init_yes_installs_harness_routing_for_every_tool_and_manifest() {
    let workspace = Workspace::new();

    let out = workspace.init(&["--yes"]);

    assert!(out.contains(&format!("Harness {VERSION} instalado em vostok/")));
    for path in embedded::files() {
        assert!(workspace.exists(&format!("vostok/{path}")), "falta {path}");
    }
    assert_eq!(workspace.read("AGENTS.md"), AGENTS_MD);
    assert!(workspace.read("CLAUDE.md").contains("AGENTS.md"));
    for dir in [".claude/skills", ".agents/skills", ".cursor/skills"] {
        let redirect = workspace.read(&format!("{dir}/refine-spec/SKILL.md"));
        assert!(redirect.contains("`vostok/skills/default/refine-spec/SKILL.md`"));
    }
    assert_eq!(
        workspace.redirect_dirs(),
        [".claude/skills", ".agents/skills", ".cursor/skills"]
    );
    assert_eq!(workspace.config("main.yml", "paths.root"), "../");
    assert_eq!(workspace.config("main.yml", "paths.docs"), "../docs");
    assert_eq!(
        workspace.config("main.yml", "paths.spec_docs"),
        "../docs/specs"
    );
}

#[test]
fn ct03_manifest_records_version_and_hash_of_each_binary_file() {
    let workspace = Workspace::new();
    workspace.init(&["--yes"]);

    let manifest = workspace.read("vostok/.manifest");

    assert!(manifest.starts_with(&format!("version: {VERSION}\nfiles:\n")));
    let skill = "skills/default/refine-spec/SKILL.md";
    let hash = sha256_hex(embedded::file(skill).unwrap());
    assert!(manifest.contains(&format!("\n  {skill}: {hash}\n")));
    for project_file in [
        "config/main.yml",
        "rules/project/language.md",
        "memory/.gitkeep",
        "skills/custom/.gitkeep",
        "context/skill-catalog.md",
    ] {
        assert!(
            !manifest.contains(&format!("  {project_file}:")),
            "o manifesto não deveria ter {project_file}"
        );
    }
}

#[test]
fn ct04_interactive_init_records_answers_and_routes_only_codex() {
    let workspace = Workspace::new();

    let out = stdout(&mut workspace.interactive("documentation\n\n2\n"));

    assert!(out.contains("paths.docs"));
    assert!(out.contains("1) claude-code"));
    assert_eq!(
        workspace.config("main.yml", "paths.docs"),
        "../documentation"
    );
    assert_eq!(
        workspace.config("main.yml", "paths.spec_docs"),
        "../documentation/specs"
    );
    assert_eq!(workspace.redirect_dirs(), [".agents/skills"]);
    assert!(workspace.exists("AGENTS.md"));
    assert!(workspace.exists(".agents/skills/refine-spec/SKILL.md"));
    assert!(!workspace.exists("CLAUDE.md"));
    assert!(!workspace.exists(".claude/skills"));
    assert!(!workspace.exists(".cursor/skills"));
}

#[test]
fn ct04_interactive_init_accepts_tool_names_and_empty_answers_as_defaults() {
    let workspace = Workspace::new();

    stdout(&mut workspace.interactive("\n\nclaude-code, cursor\n"));

    assert_eq!(workspace.config("main.yml", "paths.docs"), "../docs");
    assert_eq!(
        workspace.config("main.yml", "paths.spec_docs"),
        "../docs/specs"
    );
    assert_eq!(
        workspace.redirect_dirs(),
        [".claude/skills", ".cursor/skills"]
    );
    assert!(workspace.exists("CLAUDE.md"));
    assert!(!workspace.exists(".agents/skills"));
}

#[test]
fn ct04_interactive_init_with_tools_flag_skips_the_tool_question() {
    let workspace = Workspace::new();

    let out = stdout(
        workspace
            .interactive("/srv/docs\n/srv/docs/specs\n")
            .args(["--tools", "cursor"]),
    );

    assert!(!out.contains("1) claude-code"));
    assert_eq!(workspace.config("main.yml", "paths.docs"), "/srv/docs");
    assert_eq!(
        workspace.config("main.yml", "paths.spec_docs"),
        "/srv/docs/specs"
    );
    assert_eq!(workspace.redirect_dirs(), [".cursor/skills"]);
}

#[test]
fn ct04_interactive_init_with_missing_answers_fails_without_installing() {
    let workspace = Workspace::new();

    let error = failure_stderr(&mut workspace.interactive("docs\n"));

    assert!(error.contains("resposta"), "{error}");
    assert!(!workspace.exists("vostok"));
}

#[test]
fn ct05_second_init_refuses_and_points_to_update() {
    let workspace = Workspace::new();
    workspace.init(&["--yes"]);
    workspace.write(DEFAULT_SKILL, "editada\n");
    let before = tree(workspace.root());

    let error = workspace.init_fails(&["--yes"]);

    assert!(error.contains("vostok update"), "{error}");
    assert_eq!(tree(workspace.root()), before);
}

#[test]
fn ct14_generated_agents_md_runs_context_load() {
    let workspace = Workspace::new();
    workspace.init(&["--yes", "--tools", "codex"]);

    assert!(workspace
        .read("AGENTS.md")
        .contains("Execute `vostok context load`"));
}

#[test]
fn ct19_only_claude_code_receives_routing_in_init_finalize_and_update() {
    let workspace = Workspace::new();
    workspace.init(&["--yes", "--tools", "claude-code"]);
    workspace.write("vostok/skills/custom/my-skill/SKILL.md", CUSTOM_SKILL);

    stdout(workspace.vostok().args(["skill", "finalize", "my-skill"]));
    fs::remove_dir_all(workspace.path(".claude/skills/refine-spec")).unwrap();
    workspace.update();

    assert_eq!(workspace.redirect_dirs(), [".claude/skills"]);
    assert!(workspace.exists("CLAUDE.md"));
    assert!(workspace.exists("AGENTS.md"));
    assert!(workspace.exists(".claude/skills/my-skill/SKILL.md"));
    assert!(workspace.exists(".claude/skills/refine-spec/SKILL.md"));
    assert!(!workspace.exists(".agents"));
    assert!(!workspace.exists(".cursor"));
}

#[test]
fn cb08_unsupported_tool_fails_listing_supported_without_installing() {
    let workspace = Workspace::new();

    let error = workspace.init_fails(&["--yes", "--tools", "codex,vim"]);

    assert!(error.contains("'vim'"), "{error}");
    assert!(error.contains(SUPPORTED), "{error}");
    assert!(!workspace.exists("vostok"));
    assert!(!workspace.exists("AGENTS.md"));
}

#[test]
fn cb08_empty_tool_list_fails_listing_supported_without_installing() {
    for tools in ["", " , "] {
        let workspace = Workspace::new();

        let error = workspace.init_fails(&["--yes", "--tools", tools]);

        assert!(error.contains("nenhuma ferramenta"), "{error}");
        assert!(error.contains(SUPPORTED), "{error}");
        assert!(!workspace.exists("vostok"));
    }
}

#[test]
fn cb08_interactive_invalid_tool_fails_without_installing() {
    for answer in ["vim", "4", ","] {
        let workspace = Workspace::new();

        let error = failure_stderr(&mut workspace.interactive(&format!("\n\n{answer}\n")));

        assert!(error.contains(SUPPORTED), "{answer}: {error}");
        assert!(!workspace.exists("vostok"));
    }
}

#[test]
fn cb06_init_without_terminal_and_without_yes_asks_for_yes() {
    let workspace = Workspace::new();

    let error = failure_stderr(workspace.vostok().arg("init").env_remove(FORCE_INTERACTIVE));

    assert!(error.contains("--yes"), "{error}");
    assert!(!workspace.exists("vostok"));
}

#[test]
fn cb01_init_keeps_foreign_agents_and_claude_as_local() {
    let workspace = Workspace::new();
    workspace.write("AGENTS.md", "# Instruções da equipe\n");
    workspace.write("CLAUDE.md", "# Claude da equipe\n");

    let out = workspace.init(&["--yes"]);

    assert_eq!(
        workspace.read("AGENTS.md.local"),
        "# Instruções da equipe\n"
    );
    assert_eq!(workspace.read("CLAUDE.md.local"), "# Claude da equipe\n");
    assert!(out.contains("AGENTS.md.local"), "{out}");
    assert!(out.contains("CLAUDE.md.local"), "{out}");
    assert_eq!(workspace.read("AGENTS.md"), AGENTS_MD);
}

#[test]
fn cb01_init_without_claude_code_leaves_existing_claude_md_alone() {
    let workspace = Workspace::new();
    workspace.write("CLAUDE.md", "# Claude da equipe\n");

    workspace.init(&["--yes", "--tools", "codex"]);

    assert_eq!(workspace.read("CLAUDE.md"), "# Claude da equipe\n");
    assert!(!workspace.exists("CLAUDE.md.local"));
}

// --- update ---

fn installed(tools: &str) -> Workspace {
    let workspace = Workspace::new();
    workspace.init(&["--yes", "--tools", tools]);
    workspace
}

#[test]
fn ct06_update_preserves_project_files() {
    let workspace = installed("claude-code,codex");
    workspace.write("vostok/rules/project/language.md", "# Idioma do projeto\n");
    workspace.write("vostok/rules/project/stack.md", "# Stack\n");
    workspace.write("vostok/memory/decisions.md", "# Decisões\n");
    workspace.write("vostok/work/specs/draft/SPEC-001-x.md", "# Spec\n");
    workspace.write("vostok/.state/approved-commit", "abc\n");
    workspace.write("vostok/skills/custom/my-skill/SKILL.md", CUSTOM_SKILL);
    workspace.replace("vostok/config/sequences.yml", "specs: 1", "specs: 9");
    workspace.replace(
        "vostok/config/main.yml",
        "docs: ../docs\n",
        "docs: ../manual\n",
    );
    let project_files = [
        "vostok/rules/project/language.md",
        "vostok/rules/project/stack.md",
        "vostok/memory/decisions.md",
        "vostok/work/specs/draft/SPEC-001-x.md",
        "vostok/.state/approved-commit",
        "vostok/skills/custom/my-skill/SKILL.md",
        "vostok/config/sequences.yml",
        "vostok/config/main.yml",
        "vostok/config/skills.yml",
    ];
    let before: Vec<String> = project_files
        .iter()
        .map(|file| workspace.read(file))
        .collect();

    let out = workspace.update();

    let after: Vec<String> = project_files
        .iter()
        .map(|file| workspace.read(file))
        .collect();
    assert_eq!(after, before);
    assert!(!out.contains(".local"), "{out}");
    for file in project_files {
        assert!(!workspace.exists(&format!("{file}.local")), "{file}.local");
    }
}

#[test]
fn ct07_update_keeps_local_copy_of_changed_default_skill() {
    let workspace = installed("codex");
    workspace.write(DEFAULT_SKILL, "minha versão\n");

    let out = workspace.update();

    let local = format!("{DEFAULT_SKILL}.local");
    assert_eq!(workspace.read(&local), "minha versão\n");
    assert!(out.contains(&local), "{out}");
    assert_eq!(
        workspace.read(DEFAULT_SKILL),
        workspace.embedded("skills/default/refine-spec/SKILL.md")
    );
}

#[test]
fn ct07_update_overwrites_without_local_copy_when_only_the_binary_changed() {
    let workspace = installed("codex");
    let manifest_path = workspace.path("vostok/.manifest");
    let old = "versão anterior do binário\n";
    workspace.write(DEFAULT_SKILL, old);
    let mut manifest = Manifest::parse(&read(&manifest_path)).unwrap();
    manifest.files.insert(
        "skills/default/refine-spec/SKILL.md".to_string(),
        sha256_hex(old.as_bytes()),
    );
    manifest.version = "0.0.9".to_string();
    write(&manifest_path, &manifest.render());

    let out = workspace.update();

    assert!(!workspace.exists(&format!("{DEFAULT_SKILL}.local")));
    assert!(
        out.contains("vostok/skills/default/refine-spec/SKILL.md"),
        "{out}"
    );
    assert!(workspace
        .read("vostok/.manifest")
        .starts_with(&format!("version: {VERSION}\n")));
}

#[test]
fn ct08_update_restores_removed_config_keys_without_changing_the_others() {
    let workspace = installed("codex");
    workspace.replace(
        "vostok/config/main.yml",
        "docs: ../docs\n",
        "docs: ../manual\n",
    );
    let finalize = format!(
        "  finalize: {}",
        config_get(&workspace.path("vostok/config/main.yml"), "paths.finalize").unwrap()
    );
    let main = workspace.read("vostok/config/main.yml");
    let finalize_line = main
        .lines()
        .find(|line| line.starts_with(&finalize))
        .unwrap()
        .to_string();
    workspace.replace("vostok/config/main.yml", &format!("{finalize_line}\n"), "");
    workspace.replace("vostok/config/memory.yml", "search_max_results: 5\n", "");
    workspace.replace(
        "vostok/config/context.yml",
        "  approve-task:\n    - config/tasks.yml\n",
        "",
    );

    let out = workspace.update();

    assert!(out.contains("paths.finalize"), "{out}");
    assert!(out.contains("search_max_results"), "{out}");
    assert!(out.contains("skills.approve-task"), "{out}");
    assert_eq!(
        workspace.config("main.yml", "paths.finalize"),
        "./work/finalize"
    );
    assert_eq!(workspace.config("main.yml", "paths.docs"), "../manual");
    assert_eq!(workspace.config("main.yml", "paths.root"), "../");
    assert_eq!(workspace.config("memory.yml", "search_max_results"), "5");
    assert!(workspace
        .read("vostok/config/context.yml")
        .contains("  approve-task:\n    - config/tasks.yml\n"));
    assert_eq!(workspace.redirect_dirs(), [".agents/skills"]);
}

#[test]
fn ct08_update_recreates_a_missing_config_file() {
    let workspace = installed("codex");
    fs::remove_file(workspace.path("vostok/config/documents.yml")).unwrap();

    workspace.update();

    assert_eq!(
        workspace.read("vostok/config/documents.yml"),
        workspace.embedded("config/documents.yml")
    );
}

#[test]
fn ct09_update_regenerates_redirects_routing_and_catalog() {
    let workspace = installed("codex");
    fs::remove_dir_all(workspace.path(".agents/skills/refine-spec")).unwrap();
    fs::remove_file(workspace.path("AGENTS.md")).unwrap();
    let catalog = workspace.read(CATALOG);
    let row = catalog
        .lines()
        .find(|line| line.starts_with("| refine-spec |"))
        .unwrap()
        .to_string();
    workspace.replace(CATALOG, &format!("{row}\n"), "");

    let out = workspace.update();

    assert!(out.contains(".agents/skills/refine-spec/SKILL.md"), "{out}");
    assert!(workspace.exists(".agents/skills/refine-spec/SKILL.md"));
    assert_eq!(workspace.read("AGENTS.md"), AGENTS_MD);
    assert!(workspace.read(CATALOG).contains(&row));
}

#[test]
fn ct09_update_keeps_custom_skills_in_the_catalog() {
    let workspace = installed("codex");
    workspace.write("vostok/skills/custom/my-skill/SKILL.md", CUSTOM_SKILL);
    stdout(workspace.vostok().args(["skill", "finalize", "my-skill"]));

    workspace.update();

    assert!(workspace
        .read(CATALOG)
        .contains("| my-skill | Faz algo útil. Use quando pedirem. |"));
    assert!(!workspace.exists(&format!("{CATALOG}.local")));
}

#[test]
fn cb03_update_recreates_deleted_binary_file_without_local() {
    let workspace = installed("codex");
    fs::remove_file(workspace.path(DEFAULT_SKILL)).unwrap();

    let out = workspace.update();

    assert!(out.contains(DEFAULT_SKILL), "{out}");
    assert_eq!(
        workspace.read(DEFAULT_SKILL),
        workspace.embedded("skills/default/refine-spec/SKILL.md")
    );
    assert!(!workspace.exists(&format!("{DEFAULT_SKILL}.local")));
}

#[test]
fn cb04_update_with_the_same_binary_changes_nothing() {
    let workspace = installed("claude-code,codex,cursor");
    let before = tree(workspace.root());

    let out = workspace.update();

    assert!(out.contains("já está atualizado"), "{out}");
    assert_eq!(tree(workspace.root()), before);
}

#[test]
fn ct15_update_adds_the_coordinator_to_an_older_required_context() {
    let workspace = installed("codex");
    workspace.replace(
        "vostok/config/context.yml",
        "  - personas/coordenador.md\n",
        "",
    );

    let out = workspace.update();

    let context = workspace.read("vostok/config/context.yml");
    assert_eq!(context.matches("  - personas/coordenador.md\n").count(), 1);
    let required = config_list(&workspace.path("vostok/config/context.yml"), "required").unwrap();
    assert!(
        required.contains(&"personas/coordenador.md".to_string()),
        "{required:?}"
    );
    assert!(
        out.contains(
            "Configuração acrescentada: config/context.yml: required.personas/coordenador.md"
        ),
        "{out}"
    );
    assert!(workspace.update().contains("já está atualizado"));
}

#[test]
fn cb05_update_preserves_custom_skill_named_like_a_new_default_skill() {
    let workspace = installed("codex");
    fs::remove_dir_all(workspace.path("vostok/skills/default/refine-spec")).unwrap();
    let custom = CUSTOM_SKILL.replace("my-skill", "refine-spec");
    workspace.write("vostok/skills/custom/refine-spec/SKILL.md", &custom);

    let out = workspace.update();

    assert!(out.contains("Aviso"), "{out}");
    assert!(out.contains("'refine-spec'"), "{out}");
    assert_eq!(
        workspace.read("vostok/skills/custom/refine-spec/SKILL.md"),
        custom
    );
    assert!(workspace.exists(DEFAULT_SKILL));
    assert!(workspace
        .read(".agents/skills/refine-spec/SKILL.md")
        .contains("vostok/skills/custom/refine-spec/SKILL.md"));
}

#[test]
fn update_without_manifest_fails_pointing_to_init() {
    let project = Project::new();

    let error = failure_stderr(project.vostok().arg("update"));

    assert!(error.contains(".manifest"), "{error}");
    assert!(error.contains("vostok init"), "{error}");
}

#[test]
fn update_outside_a_project_fails_pointing_to_init() {
    let workspace = Workspace::new();

    let error = failure_stderr(workspace.vostok().arg("update"));

    assert!(error.contains("vostok init"), "{error}");
}

// --- subagentes (CT-08, RF-12) ---

const SPECIALISTS: [&str; 7] = [
    "planejador",
    "arquiteto",
    "engenheiro",
    "desenvolvedor",
    "qa-lead",
    "ui-ux-designer",
    "qa",
];
const ARCHITECT_AGENT: &str = ".claude/agents/vostok-arquiteto.md";

#[test]
fn ct08_init_generates_a_subagent_per_specialist_persona_for_claude_code() {
    let workspace = Workspace::new();

    let out = workspace.init(&["--yes", "--tools", "claude-code"]);

    for name in SPECIALISTS {
        let agent = workspace.read(&format!(".claude/agents/vostok-{name}.md"));
        assert!(
            agent.starts_with(&format!("---\nname: vostok-{name}\ndescription: \"")),
            "{agent}"
        );
        assert!(
            agent.contains(&format!("vostok context load --persona {name}")),
            "{agent}"
        );
    }
    assert!(!workspace.exists(".claude/agents/vostok-coordenador.md"));
    assert!(
        out.contains("Subagentes: .claude/agents (7 personas)"),
        "{out}"
    );
}

#[test]
fn ct08_subagent_description_comes_from_the_persona() {
    let workspace = installed("claude-code");

    let persona = workspace.path("vostok/personas/arquiteto.md");
    let expected = frontmatter_get(&persona, "description").unwrap();

    assert_eq!(
        workspace.read("vostok/personas/arquiteto.md"),
        workspace.embedded("personas/arquiteto.md")
    );
    assert_eq!(
        frontmatter_get(&workspace.path(ARCHITECT_AGENT), "description").unwrap(),
        expected
    );
    assert_eq!(
        frontmatter_get(&workspace.path(ARCHITECT_AGENT), "model").unwrap(),
        "inherit"
    );
}

#[test]
fn ct08_init_generates_subagents_only_for_registered_tools() {
    let workspace = installed("codex");

    assert!(!workspace.exists(".claude/agents"));
    assert!(!workspace.exists(".cursor/agents"));
}

#[test]
fn ct08_codex_subagents_follow_the_study_format() {
    let workspace = Workspace::new();

    let out = workspace.init(&["--yes", "--tools", "codex"]);

    let description = frontmatter_get(
        &workspace.path("vostok/personas/arquiteto.md"),
        "description",
    )
    .unwrap();
    let agent = workspace.read(".codex/agents/vostok-arquiteto.toml");
    assert!(
        agent.starts_with(&format!(
            "name = \"vostok-arquiteto\"\ndescription = \"{description}\"\ndeveloper_instructions = \"\"\"\n"
        )),
        "{agent}"
    );
    assert!(agent.contains("vostok context load --persona arquiteto"));
    assert!(agent.ends_with("\"\"\"\n"), "{agent}");
    for name in SPECIALISTS {
        assert!(workspace.exists(&format!(".codex/agents/vostok-{name}.toml")));
    }
    assert!(!workspace.exists(".codex/agents/vostok-coordenador.toml"));
    assert!(
        out.contains("Subagentes: .codex/agents (7 personas)"),
        "{out}"
    );
}

#[test]
fn ct08_cursor_subagents_follow_the_study_format() {
    let workspace = installed("cursor");

    let agent = workspace.read(".cursor/agents/vostok-arquiteto.md");

    assert!(
        agent.starts_with("---\nname: vostok-arquiteto\ndescription: \""),
        "{agent}"
    );
    assert!(agent.contains("\nmodel: inherit\n---\n"), "{agent}");
    assert!(agent.contains("vostok context load --persona arquiteto"));
}

#[test]
fn ct08_cursor_reuses_the_agents_of_another_registered_tool() {
    let with_claude = Workspace::new();
    let out = with_claude.init(&["--yes", "--tools", "claude-code,cursor"]);
    assert!(with_claude.exists(ARCHITECT_AGENT));
    assert!(!with_claude.exists(".cursor/agents"));
    assert!(!out.contains("Subagentes: .cursor/agents"), "{out}");

    let alone = Workspace::new();
    let out = alone.init(&["--yes", "--tools", "cursor"]);
    assert!(alone.exists(".cursor/agents/vostok-arquiteto.md"));
    assert!(
        out.contains("Subagentes: .cursor/agents (7 personas)"),
        "{out}"
    );
}

#[test]
fn ct08_update_recreates_a_removed_subagent() {
    let workspace = installed("claude-code");
    let generated = workspace.read(ARCHITECT_AGENT);
    fs::remove_file(workspace.path(ARCHITECT_AGENT)).unwrap();

    let out = workspace.update();

    assert_eq!(workspace.read(ARCHITECT_AGENT), generated);
    assert!(
        out.contains(&format!("Subagente regenerado: {ARCHITECT_AGENT}")),
        "{out}"
    );
    assert!(workspace.update().contains("já está atualizado"));
}

#[test]
fn ct08_update_keeps_a_changed_subagent_as_local() {
    let workspace = installed("claude-code");
    let generated = workspace.read(ARCHITECT_AGENT);
    workspace.write(ARCHITECT_AGENT, "minha versão\n");

    let out = workspace.update();

    assert_eq!(
        workspace.read(&format!("{ARCHITECT_AGENT}.local")),
        "minha versão\n"
    );
    assert_eq!(workspace.read(ARCHITECT_AGENT), generated);
    assert!(
        out.contains(&format!("Versão local guardada: {ARCHITECT_AGENT}.local")),
        "{out}"
    );
    assert!(
        !out.contains(&format!("Subagente regenerado: {ARCHITECT_AGENT}.local")),
        "{out}"
    );
}

#[test]
fn ct08_update_preserves_the_agents_of_the_project() {
    let workspace = installed("claude-code");
    let own = ".claude/agents/meu-agente.md";
    workspace.write(own, "---\nname: meu-agente\n---\n\nMeu agente.\n");

    let out = workspace.update();

    assert_eq!(
        workspace.read(own),
        "---\nname: meu-agente\n---\n\nMeu agente.\n"
    );
    assert!(!workspace.exists(&format!("{own}.local")));
    assert!(out.contains("já está atualizado"), "{out}");
}

// --- manifesto ---

#[test]
fn manifest_round_trips_its_format() {
    let content = "version: 0.1.0\nfiles:\n  a/b.md: 1f\n  c.md: 2e\n";

    let manifest = Manifest::parse(content).unwrap();

    assert_eq!(manifest.version, "0.1.0");
    assert_eq!(manifest.files.get("a/b.md").map(String::as_str), Some("1f"));
    assert_eq!(manifest.render(), content);
}

#[test]
fn manifest_rejects_malformed_content() {
    for content in [
        "",
        "files:\n  a.md: 1f\n",
        "version: 0.1.0\nfiles:\n  a.md\n",
        "version: 0.1.0\nfiles:\n  a.md: \n",
        "version: 0.1.0\nextra: 1\n",
    ] {
        assert!(Manifest::parse(content).is_err(), "{content:?}");
    }
}
