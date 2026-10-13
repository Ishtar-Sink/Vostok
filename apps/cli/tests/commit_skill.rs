//! Testes dos grupos `commit` e `skill`: equivalência com as fixtures dos scripts bash (CT-10) e
//! cenários de erro e de borda fora delas, como a escolha de ferramentas do `skill finalize` (RF-25).

mod common;

use std::fs;
use std::path::Path;
use std::process::Command as Process;

use assert_cmd::Command;
use common::golden::assert_group;
use common::{failure_stderr, Project};
use vostok::commands::skill::{self, Scope};
use vostok::harness::Harness;
use vostok::support::config_set;

const SKILLS_CONFIG: &str = "vostok/config/skills.yml";

#[test]
fn golden_commit() {
    assert_group("commit");
}

#[test]
fn golden_skill() {
    assert_group("skill");
}

/// Projeto temporário com o harness em `vostok/` e a raiz do projeto apontando para a pasta temporária.
fn project() -> Project {
    let project = Project::new();
    config_set(
        &project.harness().join("config/main.yml"),
        "paths.root",
        "../",
    )
    .unwrap();
    project
}

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

// --- skill ---

const VALID_SKILL: &str = "---\nname: my-skill\ndescription: \"Faz algo útil. Use quando pedirem.\"\nallowed-tools: Read\nmetadata:\n  version: \"1.0.0\"\n---\n\n# Minha skill\n";

fn custom_skill(project: &Project, name: &str, content: &str) {
    write(
        &project
            .harness()
            .join("skills/custom")
            .join(name)
            .join("SKILL.md"),
        content,
    );
}

fn set_redirect_dirs(project: &Project, dirs: &[&str]) {
    let path = project.root().join(SKILLS_CONFIG);
    let content = read(&path);
    let start = content.find("redirect_dirs:").unwrap();
    let end = content[start..].find("\n\n").unwrap() + start;
    let items: String = dirs.iter().map(|dir| format!("\n  - {dir}")).collect();
    let replaced = format!(
        "{}redirect_dirs:{items}{}",
        &content[..start],
        &content[end..]
    );
    fs::write(path, replaced).unwrap();
}

fn finalize_errors(project: &Project, name: &str) -> String {
    failure_stderr(project.vostok().args(["skill", "finalize", name]))
}

#[test]
fn rf25_finalize_writes_redirects_only_for_the_configured_tool() {
    let project = project();
    set_redirect_dirs(&project, &[".claude/skills"]);
    custom_skill(&project, "my-skill", VALID_SKILL);

    let out = stdout(project.vostok().args(["skill", "finalize", "my-skill"]));

    let redirect = project.root().join(".claude/skills/my-skill/SKILL.md");
    assert!(out.contains(&format!("Redirecionamento: {}", redirect.display())));
    assert!(!out.contains(".agents/skills"));
    assert!(!project.root().join(".agents/skills/my-skill").exists());
    assert_eq!(
        read(&redirect),
        "---\nname: my-skill\ndescription: \"Faz algo útil. Use quando pedirem.\"\nallowed-tools: Read\n---\n\n> Este é apenas um redirecionamento para a skill.\n> Leia e siga o arquivo `vostok/skills/custom/my-skill/SKILL.md`.\n"
    );
}

#[test]
fn rf25_finalize_without_tools_only_updates_the_catalog() {
    let project = project();
    set_redirect_dirs(&project, &[]);
    custom_skill(&project, "my-skill", VALID_SKILL);

    let out = stdout(project.vostok().args(["skill", "finalize", "my-skill"]));

    assert!(!out.contains("Redirecionamento"));
    assert!(!project.root().join(".claude").exists());
    assert!(
        read(&project.harness().join("context/skill-catalog.md")).contains(
            "| my-skill | Faz algo útil. Use quando pedirem. | `../skills/custom/my-skill` |"
        )
    );
}

#[test]
fn finalize_twice_keeps_a_single_catalog_row() {
    let project = project();
    custom_skill(&project, "my-skill", VALID_SKILL);
    stdout(project.vostok().args(["skill", "finalize", "my-skill"]));
    custom_skill(
        &project,
        "my-skill",
        &VALID_SKILL.replace("Faz algo útil.", "Faz outra coisa."),
    );
    stdout(project.vostok().args(["skill", "finalize", "my-skill"]));

    let catalog = read(&project.harness().join("context/skill-catalog.md"));
    assert_eq!(catalog.matches("| my-skill |").count(), 1);
    assert!(catalog.contains("Faz outra coisa."));
}

#[test]
fn public_api_generates_redirects_and_catalog_for_reuse() {
    let project = project();
    custom_skill(&project, "my-skill", VALID_SKILL);
    let harness = Harness::locate(project.root()).unwrap();
    let file = skill::skill_dir(&harness, Scope::Custom, "my-skill")
        .unwrap()
        .join("SKILL.md");

    let redirects = skill::write_redirects(&harness, "my-skill", &file).unwrap();
    let catalog = skill::update_catalog(&harness, "my-skill", Scope::Custom, &file).unwrap();

    assert_eq!(redirects.len(), 2);
    assert!(redirects.iter().all(|path| path.is_file()));
    assert!(read(&catalog).contains("| my-skill |"));
    assert_eq!(
        skill::find_scope(&harness, "my-skill").unwrap(),
        Some(Scope::Custom)
    );
    assert_eq!(skill::find_scope(&harness, "nao-existe").unwrap(), None);
}

#[test]
fn finalize_of_unknown_skill_fails() {
    let project = project();
    assert_eq!(
        finalize_errors(&project, "nao-existe"),
        "Erro: skill 'nao-existe' não encontrada\n"
    );
}

#[test]
fn finalize_reports_every_description_problem() {
    let project = project();
    let long = "a".repeat(1025);
    custom_skill(
        &project,
        "long-desc",
        &format!("---\nname: long-desc\ndescription: \"{long}\"\n---\n"),
    );
    custom_skill(
        &project,
        "xml-desc",
        "---\nname: xml-desc\ndescription: \"Usa <tag> no texto\"\n---\n",
    );
    custom_skill(&project, "no-desc", "---\nname: no-desc\n---\n");

    assert!(finalize_errors(&project, "long-desc")
        .contains("o campo 'description' tem 1025 caracteres; o máximo é 1024"));
    assert!(finalize_errors(&project, "xml-desc")
        .contains("o campo 'description' não pode conter tags XML"));
    assert!(finalize_errors(&project, "no-desc")
        .contains("o campo 'description' está vazio ou ausente"));
}

#[test]
fn finalize_rejects_long_skill_and_missing_metadata_paths() {
    let project = project();
    let body = "linha\n".repeat(500);
    custom_skill(
        &project,
        "my-skill",
        &(VALID_SKILL.replace(
            "  version: \"1.0.0\"\n",
            "  version: \"1.0.0\"\n  scripts: \"scripts/a.sh config/main.yml\"\n",
        ) + &body),
    );

    let errors = finalize_errors(&project, "my-skill");

    assert!(errors.starts_with("Erro: a skill 'my-skill' tem problemas:\n"));
    assert!(errors.contains("o SKILL.md tem 510 linhas; o máximo é 500"));
    assert!(errors.contains("metadata.scripts: 'scripts/a.sh' não existe na raiz do harness"));
    assert!(!errors.contains("config/main.yml"));
    assert!(!project.root().join(".claude").exists());
}

#[test]
fn finalize_fails_without_the_catalog_section() {
    let project = project();
    custom_skill(&project, "my-skill", VALID_SKILL);
    write(
        &project.harness().join("context/skill-catalog.md"),
        "# Catálogo\n\n## Padrão\n\n| a | b | c |\n|---|---|---|\n",
    );

    let errors = finalize_errors(&project, "my-skill");

    assert!(errors.contains("seção '## Customizadas' com tabela não encontrada em"));
}

#[test]
fn create_rejects_long_and_reserved_names() {
    let project = project();
    let long = "a".repeat(65);
    assert_eq!(
        failure_stderr(project.vostok().args(["skill", "create", &long])),
        format!("Erro: o nome '{long}' passa de 64 caracteres\n")
    );
    assert!(
        failure_stderr(project.vostok().args(["skill", "create", "my-anthropic"]))
            .contains("palavras reservadas")
    );
    for invalid in ["-a", "a--b", "a-", "Ab"] {
        assert!(
            !project
                .vostok()
                .args(["skill", "create", "--", invalid])
                .output()
                .unwrap()
                .status
                .success(),
            "nome aceito: {invalid}"
        );
    }
}

#[test]
fn create_default_skill_and_its_context_entry() {
    let project = project();

    let out = stdout(
        project
            .vostok()
            .args(["skill", "create", "--default", "new-one"]),
    );

    let file = project.harness().join("skills/default/new-one/SKILL.md");
    assert!(out.ends_with(&format!("Skill 'new-one' criada em {}\n", file.display())));
    assert!(read(&file).starts_with("---\nname: new-one\n"));
    let context = read(&project.harness().join("config/context.yml"));
    assert_eq!(context.matches("\n  new-one: []\n").count(), 1);
    assert!(
        failure_stderr(project.vostok().args(["skill", "create", "new-one"]))
            .contains("a skill 'new-one' já existe em")
    );
}

#[test]
fn create_adds_the_skills_block_when_the_context_has_none() {
    let project = project();
    let context = project.harness().join("config/context.yml");
    write(&context, "required:\n  - rules/agent/boundaries.md\n");

    stdout(project.vostok().args(["skill", "create", "new-one"]));

    assert_eq!(
        read(&context),
        "required:\n  - rules/agent/boundaries.md\n\n# Regras e arquivos carregados por cada skill (`load-extra-context.sh --skill <nome>`).\nskills:\n  new-one: []\n"
    );
}

#[test]
fn create_fails_without_the_context_map() {
    let project = project();
    fs::remove_file(project.harness().join("config/context.yml")).unwrap();

    assert!(
        failure_stderr(project.vostok().args(["skill", "create", "new-one"]))
            .contains("mapa de contexto não encontrado")
    );
}

#[test]
fn script_creation_errors_and_executable_file() {
    let project = project();
    custom_skill(&project, "my-skill", VALID_SKILL);

    assert_eq!(
        failure_stderr(
            project
                .vostok()
                .args(["skill", "script", "nao-existe", "a"])
        ),
        "Erro: skill 'nao-existe' não encontrada\n"
    );
    assert_eq!(
        failure_stderr(
            project
                .vostok()
                .args(["skill", "script", "my-skill", "Bad"])
        ),
        "Erro: nome de script inválido 'Bad': use kebab-case (ex.: list-items)\n"
    );
    stdout(
        project
            .vostok()
            .args(["skill", "script", "my-skill", "list-items"]),
    );
    let script = project
        .harness()
        .join("skills/custom/my-skill/scripts/list-items.sh");
    assert!(read(&script).contains("list-items.sh"));
    assert_executable(&script);
    assert!(failure_stderr(
        project
            .vostok()
            .args(["skill", "script", "my-skill", "list-items"])
    )
    .contains("o script já existe:"));
}

#[cfg(unix)]
fn assert_executable(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    assert_ne!(fs::metadata(path).unwrap().permissions().mode() & 0o111, 0);
}

#[cfg(not(unix))]
fn assert_executable(_path: &Path) {}

// --- commit ---

fn git(root: &Path, args: &[&str]) {
    let status = Process::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .envs(git_identity())
        .output()
        .unwrap()
        .status;
    assert!(status.success(), "git {args:?}");
}

fn git_identity() -> [(&'static str, &'static str); 4] {
    [
        ("GIT_AUTHOR_NAME", "T"),
        ("GIT_AUTHOR_EMAIL", "t@t"),
        ("GIT_COMMITTER_NAME", "T"),
        ("GIT_COMMITTER_EMAIL", "t@t"),
    ]
}

fn commit_cmd(project: &Project, args: &[&str]) -> Command {
    let mut command = project.vostok();
    command.arg("commit").args(args).envs(git_identity());
    command
}

fn git_project() -> Project {
    let project = project();
    git(project.root(), &["init", "-q", "-b", "main"]);
    project
}

#[test]
fn commands_outside_git_and_without_reference() {
    let project = project();
    let root = project.root().display().to_string();

    assert_eq!(
        stdout(&mut commit_cmd(&project, &["check"])),
        format!("Sem repositório git em {root}; nada a verificar.\n")
    );
    assert_eq!(
        failure_stderr(&mut commit_cmd(&project, &["revert", "abc"])),
        format!("Erro: sem repositório git em {root}\n")
    );

    git(project.root(), &["init", "-q", "-b", "main"]);
    assert_eq!(
        failure_stderr(&mut commit_cmd(&project, &["approve", "abc"])),
        "Erro: ainda não há referência de commits; execute check-commits.sh\n"
    );
}

#[test]
fn approve_and_revert_require_a_code() {
    let project = git_project();
    stdout(&mut commit_cmd(&project, &["check"]));

    for args in [&["approve"][..], &["revert"], &["approve", ""]] {
        assert!(
            failure_stderr(&mut commit_cmd(&project, args)).starts_with("Erro: Uso: "),
            "{args:?}"
        );
    }
}

#[test]
fn first_commit_of_an_empty_repository_is_released() {
    let project = git_project();
    stdout(&mut commit_cmd(&project, &["check"]));
    write(&project.root().join("a.txt"), "a\n");

    let out = stdout(commit_cmd(&project, &["commit", "a.txt"]).write_stdin("feat: a\n\n\n"));

    assert!(out.starts_with("Commit feito e liberado: "));
    assert!(out.ends_with(" feat: a\n"));
    assert_eq!(
        stdout(&mut commit_cmd(&project, &["check"])),
        "Nenhum commit sem liberação.\n"
    );
}

#[test]
fn show_and_approve_of_a_clean_repository() {
    let project = git_project();
    stdout(&mut commit_cmd(&project, &["check"]));

    let shown = stdout(&mut commit_cmd(&project, &["show"]));
    let code = shown
        .lines()
        .last()
        .unwrap()
        .strip_prefix("Código de liberação: ")
        .unwrap()
        .to_string();

    assert_eq!(code.len(), 12);
    assert_eq!(
        stdout(&mut commit_cmd(&project, &["approve", &code])),
        "Nenhum commit sem liberação.\n"
    );
    assert_eq!(
        stdout(&mut commit_cmd(&project, &["revert", &code])),
        "Nenhum commit sem liberação.\n"
    );
}

#[test]
fn revert_refuses_a_branch_missing_from_the_reference() {
    let project = git_project();
    stdout(&mut commit_cmd(&project, &["check"]));
    write(&project.root().join("a.txt"), "a\n");
    git(project.root(), &["add", "a.txt"]);
    git(project.root(), &["commit", "-q", "-m", "a"]);
    let shown = stdout(&mut commit_cmd(&project, &["show"]));
    let code = shown.lines().last().unwrap().rsplit(' ').next().unwrap();

    assert_eq!(
        failure_stderr(&mut commit_cmd(&project, &["revert", code])),
        "Erro: a branch main não existia na liberação; peça ao usuário para desfazer à mão\n"
    );
}
