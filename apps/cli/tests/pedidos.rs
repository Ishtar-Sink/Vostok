//! Testes de `vostok pedidos`: os pedidos do painel mudam de estado só pelo comando.

mod common;

use std::fs;
use std::path::PathBuf;

use common::{failure_stderr, Project};
use vostok::support::{config_set, frontmatter_get, markdown_section, utc_datetime_minutes_ago};

const STATE_FOLDERS: [&str; 4] = ["pendente", "em_andamento", "feito", "recusado"];

fn request_in(project: &Project, folder: &str, id: &str) -> PathBuf {
    project
        .harness()
        .join("work/pedidos")
        .join(folder)
        .join(format!("{id}.md"))
}

/// O arquivo do pedido, na subpasta em que estiver.
fn found(project: &Project, id: &str) -> PathBuf {
    STATE_FOLDERS
        .iter()
        .map(|folder| request_in(project, folder, id))
        .find(|path| path.is_file())
        .unwrap_or_else(|| panic!("pedido {id} não encontrado"))
}

fn request_text(id: &str, action: &str, state: &str) -> String {
    format!(
        "---\nid: {id}\nacao: {action}\nspec: SPEC-001\ntarefa: TASK-02\nestado: {state}\norigem: painel\ncriado_em: 2026-01-01T00:00:00Z\natualizado_em: 2026-01-01T00:00:00Z\n---\n\n## Pedido\n\nAjuste o texto.\n\n## Resultado\n\n"
    )
}

fn write_at(path: &PathBuf, content: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

/// Grava o pedido na subpasta do estado, como o painel grava na versão com pastas por estado.
fn write_request(project: &Project, id: &str, action: &str, state: &str) -> PathBuf {
    let path = request_in(project, state, id);
    write_at(&path, &request_text(id, action, state));
    path
}

/// Arquivos `.md` de uma pasta, sem descer nas subpastas.
fn md_files(dir: &std::path::Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dir)
        .map(|entries| {
            entries
                .flatten()
                .filter(|entry| entry.path().is_file())
                .map(|entry| entry.file_name().to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default();
    names.sort();
    names
}

fn stdout_of(command: &mut assert_cmd::Command) -> String {
    String::from_utf8(command.assert().success().get_output().stdout.clone()).unwrap()
}

#[test]
fn listar_shows_pending_requests() {
    let project = Project::new();
    write_request(&project, "PED-002", "aprovar-tarefa", "feito");
    write_request(&project, "PED-001", "ajustar-tarefa", "pendente");

    let all = stdout_of(project.vostok().args(["pedidos", "listar"]));
    assert!(all.contains("| ID | Ação | Spec | Tarefa | Estado |"));
    assert!(all.contains("| PED-001 | ajustar-tarefa | SPEC-001 | TASK-02 | pendente |"));
    assert!(all.find("PED-001").unwrap() < all.find("PED-002").unwrap());

    let done = stdout_of(
        project
            .vostok()
            .args(["pedidos", "listar", "--estado", "feito"]),
    );
    assert!(done.contains("PED-002"));
    assert!(!done.contains("PED-001"));
}

#[test]
fn iniciar_moves_pending_to_em_andamento() {
    let project = Project::new();
    let path = write_request(&project, "PED-001", "ajustar-tarefa", "pendente");

    project
        .vostok()
        .args(["pedidos", "iniciar", "PED-001"])
        .assert()
        .success();

    assert!(!path.exists());
    let content = fs::read_to_string(found(&project, "PED-001")).unwrap();
    assert!(content.contains("estado: em_andamento"));
    assert!(!content.contains("atualizado_em: 2026-01-01T00:00:00Z"));
    assert!(content.contains("criado_em: 2026-01-01T00:00:00Z"));
}

#[test]
fn iniciar_refuses_a_request_not_pending() {
    let project = Project::new();
    let path = write_request(&project, "PED-001", "ajustar-tarefa", "em_andamento");
    let before = fs::read_to_string(&path).unwrap();

    let stderr = failure_stderr(project.vostok().args(["pedidos", "iniciar", "PED-001"]));

    assert!(stderr.contains("em_andamento"));
    assert_eq!(fs::read_to_string(path).unwrap(), before);
}

#[test]
fn concluir_writes_the_result() {
    let project = Project::new();
    let path = write_request(&project, "PED-001", "ajustar-tarefa", "em_andamento");

    project
        .vostok()
        .args(["pedidos", "concluir", "PED-001"])
        .write_stdin("Tarefa ajustada.\n")
        .assert()
        .success();

    assert!(!path.exists());
    let content = fs::read_to_string(found(&project, "PED-001")).unwrap();
    assert!(content.contains("estado: feito"));
    assert!(content.contains("## Resultado\n\nTarefa ajustada.\n"));
    assert!(content.contains("## Pedido\n\nAjuste o texto."));
}

#[test]
fn recusar_writes_the_reason() {
    let project = Project::new();
    let path = write_request(&project, "PED-001", "ajustar-tarefa", "em_andamento");

    project
        .vostok()
        .args(["pedidos", "recusar", "PED-001"])
        .write_stdin("Fora do escopo.")
        .assert()
        .success();

    assert!(!path.exists());
    let content = fs::read_to_string(found(&project, "PED-001")).unwrap();
    assert!(content.contains("estado: recusado"));
    assert!(content.contains("## Resultado\n\nFora do escopo.\n"));
}

#[test]
fn concluir_requires_em_andamento() {
    let project = Project::new();
    let path = write_request(&project, "PED-001", "ajustar-tarefa", "pendente");
    let before = fs::read_to_string(&path).unwrap();

    let stderr = failure_stderr(
        project
            .vostok()
            .args(["pedidos", "concluir", "PED-001"])
            .write_stdin("x"),
    );

    assert!(stderr.contains("pendente"));
    assert_eq!(fs::read_to_string(path).unwrap(), before);
}

#[test]
fn rejects_an_id_outside_the_pattern() {
    let project = Project::new();
    write_request(&project, "PED-001", "ajustar-tarefa", "pendente");

    failure_stderr(project.vostok().args(["pedidos", "iniciar", "../PED-001"]));
    failure_stderr(project.vostok().args(["pedidos", "iniciar", "PED-9"]));
}

#[test]
fn iniciar_fails_for_a_missing_request() {
    let project = Project::new();
    failure_stderr(project.vostok().args(["pedidos", "iniciar", "PED-001"]));
}

/// Pasta com um `codex` simulado: anota início e fim de cada execução no log e processa o pedido
/// citado na instrução pelo próprio `vostok pedidos`.
fn fake_codex(project: &Project, log: &std::path::Path) -> PathBuf {
    use std::os::unix::fs::PermissionsExt;

    let bin = project.root().join("bin");
    fs::create_dir_all(&bin).unwrap();
    let script = bin.join("codex");
    fs::write(
        &script,
        format!(
            "#!/bin/sh\nid=$(echo \"$*\" | grep -o 'PED-[0-9]*' | head -n 1)\necho \"inicio $id\" >> '{log}'\nsleep 0.3\ncd '{root}' && vostok pedidos iniciar \"$id\" >/dev/null && printf 'feito pelo codex' | vostok pedidos concluir \"$id\" >/dev/null\necho \"fim $id\" >> '{log}'\n",
            log = log.display(),
            root = project.root().display()
        ),
    )
    .unwrap();
    fs::set_permissions(&script, fs::Permissions::from_mode(0o755)).unwrap();
    bin
}

fn path_with(first: &std::path::Path) -> String {
    let vostok_dir = std::path::Path::new(env!("CARGO_BIN_EXE_vostok"))
        .parent()
        .unwrap()
        .to_path_buf();
    format!("{}:{}:/usr/bin:/bin", first.display(), vostok_dir.display())
}

fn state_of(project: &Project, id: &str) -> String {
    fs::read_to_string(found(project, id))
        .unwrap()
        .lines()
        .find_map(|line| line.strip_prefix("estado: ").map(str::to_string))
        .unwrap_or_default()
}

#[test]
fn vigiar_runs_codex_once_per_request_in_order() {
    let project = Project::new();
    write_request(&project, "PED-002", "ajustar-tarefa", "pendente");
    write_request(&project, "PED-001", "ajustar-tarefa", "pendente");
    let log = project.root().join("codex.log");
    let bin = fake_codex(&project, &log);

    let mut watcher = std::process::Command::new(env!("CARGO_BIN_EXE_vostok"))
        .args(["pedidos", "vigiar", "--agente", "codex", "--intervalo", "1"])
        .current_dir(project.root())
        .env("PATH", path_with(&bin))
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
    while ["PED-001", "PED-002"]
        .iter()
        .any(|id| state_of(&project, id) != "feito")
    {
        assert!(
            std::time::Instant::now() < deadline,
            "os pedidos não ficaram feitos a tempo"
        );
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    watcher.kill().unwrap();
    watcher.wait().unwrap();

    let lines: Vec<String> = fs::read_to_string(&log)
        .unwrap()
        .lines()
        .map(str::to_string)
        .collect();
    assert_eq!(
        lines,
        [
            "inicio PED-001",
            "fim PED-001",
            "inicio PED-002",
            "fim PED-002"
        ]
    );
}

#[test]
fn vigiar_refuses_a_request_codex_left_pending() {
    use std::os::unix::fs::PermissionsExt;

    let project = Project::new();
    write_request(&project, "PED-001", "ajustar-tarefa", "pendente");
    let bin = project.root().join("bin");
    fs::create_dir_all(&bin).unwrap();
    let script = bin.join("codex");
    fs::write(&script, "#!/bin/sh\nexit 0\n").unwrap();
    fs::set_permissions(&script, fs::Permissions::from_mode(0o755)).unwrap();

    let mut watcher = std::process::Command::new(env!("CARGO_BIN_EXE_vostok"))
        .args(["pedidos", "vigiar", "--agente", "codex", "--intervalo", "1"])
        .current_dir(project.root())
        .env("PATH", path_with(&bin))
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
    while state_of(&project, "PED-001") != "recusado" {
        assert!(
            std::time::Instant::now() < deadline,
            "o pedido não foi recusado"
        );
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    watcher.kill().unwrap();
    watcher.wait().unwrap();

    let path = found(&project, "PED-001");
    assert!(path.ends_with("recusado/PED-001.md"), "{}", path.display());
    let content = fs::read_to_string(path).unwrap();
    assert!(content.contains("o codex terminou sem processar o pedido"));
}

#[test]
fn vigiar_without_codex_stops_with_instructions() {
    let project = Project::new();
    let empty = project.root().join("vazio");
    fs::create_dir_all(&empty).unwrap();

    let stderr = failure_stderr(
        project
            .vostok()
            .env("PATH", empty.display().to_string())
            .args(["pedidos", "vigiar", "--agente", "codex"]),
    );

    assert!(stderr.contains("codex"), "{stderr}");
    assert!(stderr.contains("PATH"), "{stderr}");
}

#[test]
fn vigiar_rejects_an_unknown_agent() {
    let project = Project::new();
    let stderr = failure_stderr(
        project
            .vostok()
            .args(["pedidos", "vigiar", "--agente", "claude"]),
    );
    assert!(stderr.contains("codex"), "{stderr}");
}

#[test]
fn folders_start_and_finish_move_the_file() {
    let project = Project::new();
    write_request(&project, "PED-001", "ajustar-tarefa", "pendente");
    write_request(&project, "PED-002", "ajustar-tarefa", "pendente");
    let base = project.harness().join("work/pedidos");

    project
        .vostok()
        .args(["pedidos", "iniciar", "PED-001"])
        .assert()
        .success();
    assert_eq!(md_files(&base.join("pendente")), ["PED-002.md"]);
    assert_eq!(md_files(&base.join("em_andamento")), ["PED-001.md"]);

    project
        .vostok()
        .args(["pedidos", "concluir", "PED-001"])
        .write_stdin("ok")
        .assert()
        .success();
    assert!(md_files(&base.join("em_andamento")).is_empty());
    assert_eq!(md_files(&base.join("feito")), ["PED-001.md"]);
    assert!(fs::read_to_string(base.join("feito/PED-001.md"))
        .unwrap()
        .contains("estado: feito"));

    project
        .vostok()
        .args(["pedidos", "iniciar", "PED-002"])
        .assert()
        .success();
    project
        .vostok()
        .args(["pedidos", "recusar", "PED-002"])
        .write_stdin("não")
        .assert()
        .success();
    assert_eq!(md_files(&base.join("recusado")), ["PED-002.md"]);
    assert!(md_files(&base.join("pendente")).is_empty());
}

#[test]
fn folders_record_the_history() {
    let project = Project::new();
    write_request(&project, "PED-001", "ajustar-tarefa", "pendente");
    project
        .vostok()
        .args(["pedidos", "iniciar", "PED-001"])
        .assert()
        .success();
    project
        .vostok()
        .args(["pedidos", "concluir", "PED-001"])
        .write_stdin("Tarefa ajustada.")
        .assert()
        .success();

    let content = fs::read_to_string(found(&project, "PED-001")).unwrap();
    let history = content.find("## Histórico").expect("seção de histórico");
    let result = content.find("## Resultado").expect("seção de resultado");
    assert!(history < result, "{content}");
    assert!(
        content.contains("| Data | De | Para |\n|---|---|---|\n"),
        "{content}"
    );
    assert!(
        content.contains(" | pendente | em_andamento |"),
        "{content}"
    );
    assert!(content.contains(" | em_andamento | feito |"), "{content}");
    assert!(
        content.contains("## Pedido\n\nAjuste o texto."),
        "{content}"
    );
    assert!(
        content.ends_with("## Resultado\n\nTarefa ajustada.\n"),
        "{content}"
    );
}

#[test]
fn folders_list_pending_reads_only_pendente() {
    let project = Project::new();
    for (id, state) in [
        ("PED-001", "pendente"),
        ("PED-002", "em_andamento"),
        ("PED-003", "feito"),
        ("PED-004", "recusado"),
    ] {
        write_request(&project, id, "ajustar-tarefa", state);
    }
    write_at(
        &request_in(&project, "feito", "PED-099"),
        "sem front-matter\n",
    );

    let pending = stdout_of(
        project
            .vostok()
            .args(["pedidos", "listar", "--estado", "pendente"]),
    );
    assert!(pending.contains("| PED-001 |"), "{pending}");
    for id in ["PED-002", "PED-003", "PED-004", "PED-099"] {
        assert!(!pending.contains(id), "{pending}");
    }
}

#[test]
fn folders_migrate_loose_requests() {
    let project = Project::new();
    let base = project.harness().join("work/pedidos");
    let texts: Vec<(&str, &str, String)> = [
        ("PED-001", "pendente"),
        ("PED-002", "feito"),
        ("PED-003", "recusado"),
    ]
    .iter()
    .map(|(id, state)| (*id, *state, request_text(id, "ajustar-tarefa", state)))
    .collect();
    for (id, _, text) in &texts {
        write_at(&base.join(format!("{id}.md")), text);
    }

    project
        .vostok()
        .args(["pedidos", "listar"])
        .assert()
        .success();
    assert!(md_files(&base).is_empty(), "{:?}", md_files(&base));
    for (id, state, text) in &texts {
        assert_eq!(
            &fs::read_to_string(base.join(state).join(format!("{id}.md"))).unwrap(),
            text
        );
    }

    project
        .vostok()
        .args(["pedidos", "listar"])
        .assert()
        .success();
    for (id, state, text) in &texts {
        assert_eq!(
            &fs::read_to_string(base.join(state).join(format!("{id}.md"))).unwrap(),
            text
        );
    }
}

#[test]
fn folders_migrate_renamed_folders() {
    let project = Project::new();
    let base = project.harness().join("work/pedidos");
    write_request(&project, "PED-001", "ajustar-tarefa", "pendente");
    write_request(&project, "PED-002", "ajustar-tarefa", "feito");
    let main = project.harness().join("config/main.yml");
    let config = fs::read_to_string(&main)
        .unwrap()
        .replace("pending: pendente", "pending: pending")
        .replace("in_progress: em_andamento", "in_progress: in_progress")
        .replace("done: feito", "done: done")
        .replace("refused: recusado", "refused: refused");
    fs::write(&main, config).unwrap();

    project
        .vostok()
        .args(["pedidos", "listar"])
        .assert()
        .success();
    assert_eq!(md_files(&base.join("pending")), ["PED-001.md"]);
    assert_eq!(md_files(&base.join("done")), ["PED-002.md"]);
    assert!(!base.join("pendente").exists());
    assert!(!base.join("feito").exists());
}

#[test]
fn folders_follow_the_config() {
    let project = Project::new();
    let main = project.harness().join("config/main.yml");
    let config = fs::read_to_string(&main)
        .unwrap()
        .replace("requests: ./work/pedidos", "requests: ./work/requests")
        .replace("pending: pendente", "pending: pending")
        .replace("in_progress: em_andamento", "in_progress: in_progress")
        .replace("done: feito", "done: done")
        .replace("refused: recusado", "refused: refused");
    fs::write(&main, config).unwrap();
    let base = project.harness().join("work/requests");
    write_at(
        &base.join("pending/PED-001.md"),
        &request_text("PED-001", "ajustar-tarefa", "pendente"),
    );

    project
        .vostok()
        .args(["pedidos", "iniciar", "PED-001"])
        .assert()
        .success();
    assert_eq!(md_files(&base.join("in_progress")), ["PED-001.md"]);
    project
        .vostok()
        .args(["pedidos", "concluir", "PED-001"])
        .write_stdin("ok")
        .assert()
        .success();
    let done = base.join("done/PED-001.md");
    assert!(fs::read_to_string(done).unwrap().contains("estado: feito"));
    assert!(!project.harness().join("work/pedidos").exists());
}

#[test]
fn folders_keep_unknown_states_in_place() {
    let project = Project::new();
    let base = project.harness().join("work/pedidos");
    let loose = base.join("PED-005.md");
    write_at(&loose, &request_text("PED-005", "ajustar-tarefa", "outro"));
    write_request(&project, "PED-001", "ajustar-tarefa", "pendente");

    let all = stdout_of(project.vostok().args(["pedidos", "listar"]));
    assert!(loose.is_file());
    assert!(all.contains("| PED-001 |"), "{all}");
}

const OLD: &str = "2026-01-01T00:00:00Z";
const RECENT: &str = "2999-01-01T00:00:00Z";

fn write_front(project: &Project, relative: &str, front_matter: &str) {
    let path = project.harness().join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        path,
        format!("---\n{front_matter}---\n\n## Pedido\n\nTexto.\n"),
    )
    .unwrap();
}

fn prune_request(project: &Project, folder: &str, id: &str, spec: &str, updated: &str) {
    write_front(
        project,
        &format!("work/pedidos/{folder}/{id}.md"),
        &format!("id: {id}\nacao: ajustar-spec\nspec: {spec}\ntarefa: \nestado: {folder}\norigem: painel\ncriado_em: {updated}\natualizado_em: {updated}\n"),
    );
}

fn prune_fixture() -> Project {
    let project = Project::new();
    write_front(
        &project,
        "work/.archive/SPEC-001/pedidos/PED-001.md",
        "id: PED-001\nspec: SPEC-001\n",
    );
    write_front(
        &project,
        "work/.archive/SPEC-001/perguntas/P-001.md",
        "id: P-001\nspec: SPEC-001\n",
    );
    write_front(&project, "work/.archive/SPEC-001/avais/SPEC-001.log", "");
    prune_request(&project, "feito", "PED-002", "", OLD);
    prune_request(&project, "feito", "PED-003", "", RECENT);
    prune_request(&project, "feito", "PED-004", "SPEC-009", OLD);
    prune_request(&project, "pendente", "PED-005", "", OLD);
    write_front(
        &project,
        "work/perguntas/respondida/P-002.md",
        &format!(
            "id: P-002\npedido: PED-002\nspec: \ntask: \nstatus: respondida\ncreated_at: {OLD}\n"
        ),
    );
    write_front(
        &project,
        "work/avais/sem-spec.log",
        "d\tPED-002\th\tpainel\tfazer-commit\n",
    );
    project
}

const PRUNED: [&str; 4] = [
    "work/.archive/SPEC-001/pedidos/PED-001.md",
    "work/.archive/SPEC-001/perguntas/P-001.md",
    "work/pedidos/feito/PED-002.md",
    "work/perguntas/respondida/P-002.md",
];
const KEPT: [&str; 5] = [
    "work/pedidos/feito/PED-003.md",
    "work/pedidos/feito/PED-004.md",
    "work/pedidos/pendente/PED-005.md",
    "work/.archive/SPEC-001/avais/SPEC-001.log",
    "work/avais/sem-spec.log",
];

fn prune(project: &Project, args: &[&str]) -> String {
    let output = project
        .vostok()
        .args(["pedidos", "prune"])
        .args(args)
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn prune_sem_confirmar_so_lista() {
    let project = prune_fixture();
    let out = prune(&project, &[]);
    for listed in ["PED-001", "P-001", "PED-002", "P-002"] {
        assert!(out.contains(listed), "{listed}: {out}");
    }
    for absent in ["PED-003", "PED-004", "PED-005", ".log"] {
        assert!(!out.contains(absent), "{absent}: {out}");
    }
    assert!(out.contains("| Arquivo | Spec | Motivo |"), "{out}");
    assert!(out.contains("--confirmar"), "{out}");
    for path in PRUNED.iter().chain(KEPT.iter()) {
        assert!(project.harness().join(path).is_file(), "{path}");
    }
}

#[test]
fn prune_com_confirmar_apaga_e_mantem_os_avais() {
    let project = prune_fixture();
    let out = prune(&project, &["--confirmar"]);
    assert!(out.contains("4 arquivo(s) apagado(s)"), "{out}");
    assert!(out.contains("os avais ficam"), "{out}");
    for path in PRUNED {
        assert!(!project.harness().join(path).exists(), "{path}");
    }
    for path in KEPT {
        assert!(project.harness().join(path).is_file(), "{path}");
    }
}

#[test]
fn prune_leva_o_pedido_concluido_de_spec_arquivada() {
    let project = Project::new();
    write_front(&project, "work/.archive/SPEC-001/spec.md", "id: SPEC-001\n");
    prune_request(&project, "feito", "PED-006", "SPEC-001", RECENT);
    let out = prune(&project, &[]);
    assert!(out.contains("PED-006"), "{out}");
    assert!(out.contains("spec arquivada"), "{out}");
}

#[test]
fn prune_sem_nada_diz_que_nao_ha_o_que_limpar() {
    let project = Project::new();
    assert_eq!(prune(&project, &[]).trim(), "Não há o que limpar.");
}

// --- anexos da pergunta ---

fn write_attachment(project: &Project, relative: &str) {
    write_front(project, relative, "");
}

fn aged_folder(project: &Project, relative: &str, hours: u64) {
    let dir = project.harness().join(relative);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("01-a.txt"), "x").unwrap();
    let modified = std::time::SystemTime::now() - std::time::Duration::from_secs(hours * 3600);
    fs::File::open(&dir)
        .unwrap()
        .set_modified(modified)
        .unwrap();
}

#[test]
fn anexo_limpeza_apaga_a_pasta_do_pedido_antigo_sem_spec() {
    let project = Project::new();
    prune_request(&project, "feito", "PED-002", "", OLD);
    write_attachment(&project, "work/pedidos/anexos/PED-002/01-a.txt");

    let out = prune(&project, &["--confirmar"]);

    assert!(out.contains("anexos do pedido"), "{out}");
    assert!(!project
        .harness()
        .join("work/pedidos/feito/PED-002.md")
        .exists());
    assert!(!project
        .harness()
        .join("work/pedidos/anexos/PED-002")
        .exists());
}

#[test]
fn anexo_limpeza_apaga_a_pasta_do_pedido_arquivado_sem_sobra() {
    let project = Project::new();
    write_front(
        &project,
        "work/.archive/SPEC-001/pedidos/PED-001.md",
        "id: PED-001\nspec: SPEC-001\n",
    );
    write_attachment(
        &project,
        "work/.archive/SPEC-001/pedidos/anexos/PED-001/01-a.txt",
    );

    let out = prune(&project, &["--confirmar"]);

    assert!(out.contains("2 arquivo(s) apagado(s)"), "{out}");
    assert!(!project
        .harness()
        .join("work/.archive/SPEC-001/pedidos")
        .exists());
}

#[test]
fn anexo_limpeza_leva_a_pasta_de_pedido_de_spec_arquivada() {
    let project = Project::new();
    write_front(&project, "work/.archive/SPEC-001/spec.md", "id: SPEC-001\n");
    prune_request(&project, "feito", "PED-006", "SPEC-001", RECENT);
    write_attachment(&project, "work/pedidos/anexos/PED-006/01-a.txt");

    let out = prune(&project, &[]);

    assert!(
        out.contains("anexos/PED-006 | SPEC-001 | anexos do pedido"),
        "{out}"
    );
    assert!(out.contains("spec arquivada"), "{out}");
}

#[test]
fn anexo_limpeza_sem_confirmar_lista_e_nao_apaga() {
    let project = Project::new();
    prune_request(&project, "feito", "PED-002", "", OLD);
    write_attachment(&project, "work/pedidos/anexos/PED-002/01-a.txt");

    let out = prune(&project, &[]);

    assert!(
        out.contains("anexos/PED-002 | - | anexos do pedido"),
        "{out}"
    );
    assert!(out.contains("--confirmar"), "{out}");
    assert!(project
        .harness()
        .join("work/pedidos/anexos/PED-002/01-a.txt")
        .is_file());
}

#[test]
fn anexo_limpeza_so_leva_rascunho_de_mais_de_24_horas() {
    let project = Project::new();
    aged_folder(&project, "work/pedidos/anexos/.rascunho/velho", 25);
    aged_folder(&project, "work/pedidos/anexos/.rascunho/novo", 1);

    let listed = prune(&project, &[]);
    assert!(
        listed.contains(".rascunho/velho | - | rascunho com mais de 24 horas"),
        "{listed}"
    );
    assert!(!listed.contains(".rascunho/novo"), "{listed}");

    prune(&project, &["--confirmar"]);
    let drafts = project.harness().join("work/pedidos/anexos/.rascunho");
    assert!(!drafts.join("velho").exists());
    assert!(drafts.join("novo/01-a.txt").is_file());
}

#[test]
fn anexo_limpeza_sem_anexos_tem_a_mesma_saida() {
    let project = prune_fixture();
    let out = prune(&project, &[]);
    assert!(!out.contains("anexo"), "{out}");
    assert!(!out.contains("rascunho"), "{out}");
    assert_eq!(out.lines().filter(|line| line.starts_with('|')).count(), 6);
}

#[test]
fn anexo_pasta_segue_a_configuracao() {
    let project = Project::new();
    let main = project.harness().join("config/main.yml");
    let config = fs::read_to_string(&main)
        .unwrap()
        .replace("attachments: anexos", "attachments: arquivos");
    fs::write(&main, config).unwrap();
    prune_request(&project, "feito", "PED-002", "", OLD);
    write_attachment(&project, "work/pedidos/arquivos/PED-002/01-a.txt");
    write_attachment(&project, "work/pedidos/anexos/PED-002/01-a.txt");

    let out = prune(&project, &["--confirmar"]);

    assert!(out.contains("arquivos/PED-002"), "{out}");
    assert!(!project
        .harness()
        .join("work/pedidos/arquivos/PED-002")
        .exists());
    assert!(project
        .harness()
        .join("work/pedidos/anexos/PED-002/01-a.txt")
        .is_file());
}

#[test]
fn anexo_pasta_vazia_nao_some_na_migracao() {
    let project = Project::new();
    let attachments = project.harness().join("work/pedidos/anexos");
    fs::create_dir_all(&attachments).unwrap();
    write_request(&project, "PED-001", "ajustar-spec", "pendente");

    project
        .vostok()
        .args(["pedidos", "listar"])
        .assert()
        .success();

    assert!(attachments.is_dir());
}

// --- desfecho do pedido (SPEC-035) ---

fn concluir(project: &Project, args: &[&str]) -> assert_cmd::Command {
    let mut command = project.vostok();
    command
        .args(["pedidos", "concluir", "PED-001"])
        .args(args)
        .write_stdin("Resultado.\n");
    command
}

/// Conclui com opções inválidas: confere a mensagem e que o pedido ficou igual.
fn desfecho_recusado(args: &[&str], expected: &str) {
    let project = Project::new();
    let path = write_request(&project, "PED-001", "ajustar-spec", "em_andamento");
    let before = fs::read(&path).unwrap();

    let stderr = failure_stderr(&mut concluir(&project, args));

    assert!(stderr.contains(expected), "{stderr}");
    assert_eq!(fs::read(&path).unwrap(), before);
    assert_eq!(
        path,
        request_in(&project, "em_andamento", "PED-001"),
        "o pedido continua em andamento"
    );
}

#[test]
fn desfecho_pausado_grava_o_desfecho_e_as_saidas() {
    let project = Project::new();
    write_request(&project, "PED-001", "ajustar-spec", "em_andamento");

    concluir(
        &project,
        &[
            "--desfecho",
            "pausado",
            "--saida",
            "criar-spec",
            "--saida",
            "ajustar-spec",
        ],
    )
    .assert()
    .success();

    let content = fs::read_to_string(found(&project, "PED-001")).unwrap();
    assert!(content.contains("estado: feito"));
    assert!(content.contains("\ndesfecho: pausado\n"), "{content}");
    assert!(
        content.contains("\nsaidas: [criar-spec, ajustar-spec]\n"),
        "{content}"
    );
    assert!(!content.contains("\npergunta:"));
    assert!(content.contains("## Resultado\n\nResultado.\n"));
}

#[test]
fn desfecho_padrao_e_concluido_sem_saidas() {
    let project = Project::new();
    write_request(&project, "PED-001", "ajustar-spec", "em_andamento");

    concluir(&project, &[]).assert().success();

    let content = fs::read_to_string(found(&project, "PED-001")).unwrap();
    assert!(content.contains("\ndesfecho: concluido\n"), "{content}");
    assert!(content.contains("\nsaidas: []\n"), "{content}");
    assert!(!content.contains("\npergunta:"));
}

#[test]
fn desfecho_aguardando_pergunta_grava_a_pergunta() {
    let project = Project::new();
    write_request(&project, "PED-001", "ajustar-spec", "em_andamento");

    concluir(
        &project,
        &["--desfecho", "aguardando-pergunta", "--pergunta", "P-002"],
    )
    .assert()
    .success();

    let content = fs::read_to_string(found(&project, "PED-001")).unwrap();
    assert!(
        content.contains("\ndesfecho: aguardando-pergunta\n"),
        "{content}"
    );
    assert!(content.contains("\npergunta: P-002\n"), "{content}");
    assert!(content.contains("\nsaidas: []\n"), "{content}");
}

#[test]
fn desfecho_aguardando_sem_pergunta_e_recusado() {
    desfecho_recusado(&["--desfecho", "aguardando-pergunta"], "--pergunta");
}

#[test]
fn desfecho_pergunta_sem_aguardando_e_recusada() {
    desfecho_recusado(&["--pergunta", "P-002"], "aguardando-pergunta");
}

#[test]
fn desfecho_desconhecido_e_recusado() {
    desfecho_recusado(&["--desfecho", "recusado"], "desfecho inválido 'recusado'");
}

#[test]
fn desfecho_saida_desconhecida_e_recusada() {
    desfecho_recusado(&["--desfecho", "pausado", "--saida", "voar"], "'voar'");
}

#[test]
fn desfecho_pergunta_com_id_invalido_e_recusada() {
    desfecho_recusado(
        &["--desfecho", "aguardando-pergunta", "--pergunta", "2"],
        "P-NNN",
    );
}

// --- notas do agente (SPEC-038) ---

fn nota(project: &Project, args: &[&str]) -> assert_cmd::Command {
    let mut command = project.vostok();
    command.args(["pedidos", "nota"]).args(args);
    command
}

fn in_progress(project: &Project) -> PathBuf {
    write_request(project, "PED-001", "executar-tarefa", "em_andamento")
}

fn concluir_com(project: &Project, id: &str, text: &str) {
    project
        .vostok()
        .args(["pedidos", "concluir", id])
        .write_stdin(text)
        .assert()
        .success();
}

#[test]
fn nota_grava_os_campos_e_mantem_estado_e_atualizado_em() {
    let project = Project::new();
    let path = in_progress(&project);

    nota(&project, &["PED-001", "rodando os testes"])
        .assert()
        .success();

    assert_eq!(frontmatter_get(&path, "nota").unwrap(), "rodando os testes");
    let at = frontmatter_get(&path, "nota_em").unwrap();
    assert!(!at.is_empty());
    assert_eq!(frontmatter_get(&path, "estado").unwrap(), "em_andamento");
    assert_eq!(
        frontmatter_get(&path, "atualizado_em").unwrap(),
        "2026-01-01T00:00:00Z"
    );
    let content = fs::read_to_string(&path).unwrap();
    assert!(
        content.find("## Notas").unwrap() < content.find("## Resultado").unwrap(),
        "{content}"
    );
    assert!(
        content.contains(&format!("- {at} rodando os testes\n")),
        "{content}"
    );
}

#[test]
fn nota_com_detalhe_grava_as_linhas_recuadas() {
    let project = Project::new();
    let path = in_progress(&project);

    nota(
        &project,
        &["PED-001", "rodando", "--detalhe", "linha 1\nlinha 2"],
    )
    .assert()
    .success();

    let content = fs::read_to_string(&path).unwrap();
    assert!(
        content.contains(" rodando\n  linha 1\n  linha 2\n"),
        "{content}"
    );
}

#[test]
fn nota_sem_detalhe_acrescenta_so_a_linha() {
    let project = Project::new();
    let path = in_progress(&project);

    nota(&project, &["PED-001", "primeira"]).assert().success();
    nota(&project, &["PED-001", "segunda"]).assert().success();

    let content = fs::read_to_string(&path).unwrap();
    assert_eq!(content.matches("## Notas").count(), 1, "{content}");
    let section = markdown_section(&content, "Notas").unwrap();
    let lines: Vec<&str> = section.lines().filter(|line| !line.is_empty()).collect();
    assert_eq!(lines.len(), 2, "{section:?}");
    assert!(lines[0].ends_with(" primeira") && lines[1].ends_with(" segunda"));
    let inner = section.trim();
    assert!(!inner.contains("\n\n"), "{section:?}");
}

#[test]
fn nota_acrescenta_na_ordem_e_mantem_o_historico() {
    let project = Project::new();
    write_request(&project, "PED-001", "executar-tarefa", "pendente");

    project
        .vostok()
        .args(["pedidos", "iniciar", "PED-001"])
        .assert()
        .success();
    nota(&project, &["PED-001", "uma"]).assert().success();
    nota(&project, &["PED-001", "duas"]).assert().success();
    concluir_com(&project, "PED-001", "Feito.");

    let content = fs::read_to_string(found(&project, "PED-001")).unwrap();
    let history = markdown_section(&content, "Histórico").unwrap();
    assert!(history.contains("| pendente | em_andamento |"), "{history}");
    assert!(history.contains("| em_andamento | feito |"), "{history}");
    let notes = markdown_section(&content, "Notas").unwrap();
    assert!(
        notes.find(" uma").unwrap() < notes.find(" duas").unwrap(),
        "{notes}"
    );
}

#[test]
fn nota_recusa_pedido_fora_de_em_andamento() {
    let project = Project::new();
    for state in ["pendente", "feito", "recusado"] {
        let path = write_request(&project, "PED-001", "executar-tarefa", state);
        let before = fs::read(&path).unwrap();

        let stderr = failure_stderr(&mut nota(&project, &["PED-001", "x"]));

        assert!(stderr.contains(state), "{stderr}");
        assert_eq!(fs::read(&path).unwrap(), before);
        fs::remove_file(&path).unwrap();
    }
}

#[test]
fn nota_recusa_texto_vazio_com_quebra_de_linha_ou_longo() {
    let project = Project::new();
    let path = in_progress(&project);
    let before = fs::read(&path).unwrap();
    let long = "a".repeat(201);

    for text in ["", "   ", "a\nb", "a\rb", long.as_str()] {
        failure_stderr(&mut nota(&project, &["PED-001", text]));
    }

    assert_eq!(fs::read(&path).unwrap(), before);
    let content = fs::read_to_string(&path).unwrap();
    assert!(!content.contains("nota") && !content.contains("## Notas"));
}

#[test]
fn nota_recusa_detalhe_com_2001_caracteres() {
    let project = Project::new();
    let path = in_progress(&project);
    let before = fs::read(&path).unwrap();
    let detail = "a".repeat(2001);

    failure_stderr(&mut nota(&project, &["PED-001", "x", "--detalhe", &detail]));

    assert_eq!(fs::read(&path).unwrap(), before);
}

#[test]
fn nota_aceita_200_caracteres_e_detalhe_com_2000() {
    let project = Project::new();
    let path = in_progress(&project);
    let text = "ç".repeat(200);
    let detail = "ç".repeat(2000);

    nota(&project, &["PED-001", &text, "--detalhe", &detail])
        .assert()
        .success();

    assert_eq!(frontmatter_get(&path, "nota").unwrap(), text);
    let content = fs::read_to_string(&path).unwrap();
    assert!(
        content.contains(&format!("  {detail}\n")),
        "detalhe cortado"
    );
}

#[test]
fn nota_com_aspas_e_dois_pontos_volta_igual() {
    let project = Project::new();
    let path = in_progress(&project);

    for text in ["a: \"b\" # c", "|", "- com hífen"] {
        nota(&project, &["PED-001", text]).assert().success();
        assert_eq!(frontmatter_get(&path, "nota").unwrap(), text);
    }
}

#[test]
fn nota_com_titulo_no_detalhe_nao_cria_secao() {
    let project = Project::new();
    in_progress(&project);

    nota(&project, &["PED-001", "x", "--detalhe", "## Resultado"])
        .assert()
        .success();
    concluir_com(&project, "PED-001", "Feito.");

    let content = fs::read_to_string(found(&project, "PED-001")).unwrap();
    assert_eq!(
        markdown_section(&content, "Resultado").unwrap().trim(),
        "Feito."
    );
    assert!(content.contains("  ## Resultado"), "{content}");
    assert!(content.contains("## Pedido"), "{content}");
}

#[test]
fn nota_com_titulo_no_texto_nao_corta_o_pedido() {
    let project = Project::new();
    in_progress(&project);

    nota(&project, &["PED-001", "## Resultado"])
        .assert()
        .success();
    concluir_com(&project, "PED-001", "Feito.");

    let path = found(&project, "PED-001");
    assert_eq!(frontmatter_get(&path, "nota").unwrap(), "## Resultado");
    let content = fs::read_to_string(&path).unwrap();
    assert!(content.starts_with("---\nid: PED-001"), "{content}");
    assert!(content.contains("## Pedido"), "{content}");
}

#[test]
fn nota_com_cerca_no_detalhe_nao_esconde_o_resultado() {
    let project = Project::new();
    in_progress(&project);

    nota(&project, &["PED-001", "x", "--detalhe", "```"])
        .assert()
        .success();
    concluir_com(&project, "PED-001", "Feito.");

    let content = fs::read_to_string(found(&project, "PED-001")).unwrap();
    assert_eq!(
        markdown_section(&content, "Resultado").unwrap().trim(),
        "Feito."
    );
}

#[test]
fn nota_nao_deixa_arquivo_temporario() {
    let project = Project::new();
    in_progress(&project);

    nota(&project, &["PED-001", "x"]).assert().success();

    let dir = project.harness().join("work/pedidos/em_andamento");
    let names: Vec<String> = fs::read_dir(&dir)
        .unwrap()
        .flatten()
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(names, ["PED-001.md"]);
    let listed = stdout_of(project.vostok().args(["pedidos", "listar"]));
    assert_eq!(listed.matches("PED-").count(), 1, "{listed}");
}

#[test]
fn nota_id_invalido_ou_inexistente_e_recusado() {
    let project = Project::new();
    in_progress(&project);

    failure_stderr(&mut nota(&project, &["PED-1", "x"]));
    failure_stderr(&mut nota(&project, &["PED-999", "x"]));
}

#[test]
fn nota_nao_e_exigida_para_concluir() {
    let project = Project::new();
    in_progress(&project);

    concluir_com(&project, "PED-001", "Feito.");

    let content = fs::read_to_string(found(&project, "PED-001")).unwrap();
    assert!(!content.contains("## Notas"), "{content}");
    assert_eq!(
        markdown_section(&content, "Resultado").unwrap().trim(),
        "Feito."
    );
}

// --- rodada do loop (SPEC-035) ---

fn loop_file(project: &Project) -> PathBuf {
    project.harness().join("work/pedidos/.loop")
}

fn loop_value(project: &Project, key: &str) -> String {
    fs::read_to_string(loop_file(project))
        .unwrap_or_default()
        .lines()
        .find_map(|line| line.strip_prefix(&format!("{key}: ")).map(str::to_string))
        .unwrap_or_default()
}

fn round_of(project: &Project) -> String {
    loop_value(project, "ultima_rodada")
}

#[test]
fn rodada_grava_o_loop_com_os_padroes() {
    let project = Project::new();

    stdout_of(project.vostok().args(["pedidos", "rodada"]));

    let content = fs::read_to_string(loop_file(&project)).unwrap();
    assert_eq!(content.lines().count(), 3, "{content}");
    assert!(content.contains("intervalo: 60\n"), "{content}");
    assert!(content.contains("agente: claude\n"), "{content}");
    let round = round_of(&project);
    assert_eq!(round.len(), 20, "{round}");
    assert!(round.ends_with('Z'));
    assert_eq!(round.as_bytes()[10], b'T');
}

#[test]
fn rodada_grava_o_intervalo_e_o_agente() {
    let project = Project::new();

    stdout_of(project.vostok().args([
        "pedidos",
        "rodada",
        "--intervalo",
        "120",
        "--agente",
        "codex",
    ]));

    let content = fs::read_to_string(loop_file(&project)).unwrap();
    assert!(content.contains("intervalo: 120\n"), "{content}");
    assert!(content.contains("agente: codex\n"), "{content}");
}

#[test]
fn rodada_sobrescreve_a_anterior() {
    let project = Project::new();
    write_at(
        &loop_file(&project),
        "ultima_rodada: 2026-01-01T00:00:00Z\nintervalo: 30\nagente: codex\n",
    );

    stdout_of(project.vostok().args(["pedidos", "rodada"]));

    let content = fs::read_to_string(loop_file(&project)).unwrap();
    assert!(!content.contains("2026-01-01T00:00:00Z"), "{content}");
    assert!(!content.contains("intervalo: 30"), "{content}");
    assert_eq!(content.lines().count(), 3, "{content}");
}

#[test]
fn rodada_intervalo_zero_e_recusado() {
    let project = Project::new();

    let stderr = failure_stderr(
        project
            .vostok()
            .args(["pedidos", "rodada", "--intervalo", "0"]),
    );

    assert!(stderr.contains("--intervalo"), "{stderr}");
    assert!(!loop_file(&project).exists());
}

#[test]
fn rodada_nao_aparece_no_listar() {
    let project = Project::new();
    write_request(&project, "PED-001", "ajustar-spec", "pendente");

    stdout_of(project.vostok().args(["pedidos", "rodada"]));
    let listed = stdout_of(project.vostok().args(["pedidos", "listar"]));

    assert!(listed.contains("PED-001"));
    assert!(!listed.contains(".loop"));
    assert!(request_in(&project, "pendente", "PED-001").is_file());
}

#[test]
fn rodada_vigiar_grava_a_rodada_a_cada_volta() {
    let project = Project::new();
    let bin = fake_codex(&project, &project.root().join("codex.log"));

    let mut watcher = std::process::Command::new(env!("CARGO_BIN_EXE_vostok"))
        .args(["pedidos", "vigiar", "--agente", "codex", "--intervalo", "1"])
        .current_dir(project.root())
        .env("PATH", path_with(&bin))
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
    let pause = std::time::Duration::from_millis(100);
    while !(loop_value(&project, "intervalo") == "1" && loop_value(&project, "agente") == "codex") {
        assert!(
            std::time::Instant::now() < deadline,
            "o vigiar não gravou o loop a tempo"
        );
        std::thread::sleep(pause);
    }
    let first = round_of(&project);
    while round_of(&project) == first {
        assert!(
            std::time::Instant::now() < deadline,
            "a rodada não mudou a tempo"
        );
        std::thread::sleep(pause);
    }
    watcher.kill().unwrap();
    watcher.wait().unwrap();
}

fn main_yml(project: &Project) -> PathBuf {
    project.harness().join("config/main.yml")
}

fn set_warning_minutes(project: &Project, value: &str) {
    let path = main_yml(project);
    if config_set(&path, "request_note_warning_minutes", value).is_err() {
        let mut content = fs::read_to_string(&path).unwrap();
        content.push_str(&format!("\nrequest_note_warning_minutes: {value}\n"));
        fs::write(&path, content).unwrap();
    }
}

fn clear_warning_minutes(project: &Project) {
    let path = main_yml(project);
    let content = fs::read_to_string(&path).unwrap();
    let kept: Vec<&str> = content
        .lines()
        .filter(|line| !line.starts_with("request_note_warning_minutes"))
        .collect();
    fs::write(&path, kept.join("\n") + "\n").unwrap();
}

/// Pedido em andamento com `nota_em` (ou, sem ele, `atualizado_em`) de `minutes` minutos atrás.
fn stale_request(project: &Project, id: &str, minutes: u64, with_note: bool) -> String {
    let path = write_request(project, id, "executar-tarefa", "em_andamento");
    let at = utc_datetime_minutes_ago(minutes);
    let content = fs::read_to_string(&path).unwrap();
    let content = if with_note {
        content.replace(
            "atualizado_em: 2026-01-01T00:00:00Z\n",
            &format!("atualizado_em: 2026-01-01T00:00:00Z\nnota: andando\nnota_em: {at}\n"),
        )
    } else {
        content.replace("2026-01-01T00:00:00Z\n---", &format!("{at}\n---"))
    };
    fs::write(&path, content).unwrap();
    at
}

fn rodada(project: &Project) {
    stdout_of(project.vostok().args(["pedidos", "rodada"]));
}

#[test]
fn aviso_grava_o_pedido_sem_nota_nova() {
    let project = Project::new();
    let at = stale_request(&project, "PED-001", 16, true);

    rodada(&project);

    assert_eq!(loop_value(&project, "sem_nota_pedido"), "PED-001");
    assert_eq!(loop_value(&project, "sem_nota_desde"), at);
}

#[test]
fn aviso_conta_de_atualizado_em_sem_nota() {
    let project = Project::new();
    let at = stale_request(&project, "PED-001", 16, false);

    rodada(&project);

    assert_eq!(loop_value(&project, "sem_nota_pedido"), "PED-001");
    assert_eq!(loop_value(&project, "sem_nota_desde"), at);
}

#[test]
fn aviso_nao_aparece_antes_do_tempo() {
    let project = Project::new();
    stale_request(&project, "PED-001", 14, true);

    rodada(&project);

    let content = fs::read_to_string(loop_file(&project)).unwrap();
    assert!(!content.contains("sem_nota"), "{content}");
}

#[test]
fn aviso_sai_com_nota_nova() {
    let project = Project::new();
    stale_request(&project, "PED-001", 16, true);
    rodada(&project);
    assert_eq!(loop_value(&project, "sem_nota_pedido"), "PED-001");

    nota(&project, &["PED-001", "voltei"]).assert().success();
    rodada(&project);

    let content = fs::read_to_string(loop_file(&project)).unwrap();
    assert!(!content.contains("sem_nota"), "{content}");
}

#[test]
fn aviso_sai_sem_pedido_em_andamento() {
    let project = Project::new();
    stale_request(&project, "PED-001", 16, true);
    rodada(&project);
    assert_eq!(loop_value(&project, "sem_nota_pedido"), "PED-001");

    concluir_com(&project, "PED-001", "Feito.");
    rodada(&project);

    let content = fs::read_to_string(loop_file(&project)).unwrap();
    assert!(!content.contains("sem_nota"), "{content}");
}

#[test]
fn aviso_nao_muda_o_pedido() {
    let project = Project::new();
    stale_request(&project, "PED-001", 16, true);
    let path = found(&project, "PED-001");
    let before = fs::read(&path).unwrap();

    rodada(&project);

    assert_eq!(fs::read(&path).unwrap(), before);
}

#[test]
fn aviso_aponta_o_pedido_de_menor_id() {
    let project = Project::new();
    stale_request(&project, "PED-003", 20, true);
    stale_request(&project, "PED-002", 30, true);

    rodada(&project);

    assert_eq!(loop_value(&project, "sem_nota_pedido"), "PED-002");
}

#[test]
fn aviso_usa_o_tempo_configurado() {
    let project = Project::new();
    set_warning_minutes(&project, "1");
    stale_request(&project, "PED-001", 2, true);

    rodada(&project);

    assert_eq!(loop_value(&project, "sem_nota_pedido"), "PED-001");
}

#[test]
fn aviso_sem_a_chave_ou_com_valor_invalido_vale_15() {
    let project = Project::new();
    stale_request(&project, "PED-001", 14, true);
    let path = found(&project, "PED-001");
    let fresh = fs::read_to_string(&path).unwrap();
    let late_at = utc_datetime_minutes_ago(16);
    let late = fresh.replace(&frontmatter_get(&path, "nota_em").unwrap(), &late_at);

    for value in [None, Some("0"), Some("-1"), Some("abc")] {
        match value {
            None => clear_warning_minutes(&project),
            Some(value) => set_warning_minutes(&project, value),
        }
        fs::write(&path, &fresh).unwrap();
        rodada(&project);
        let content = fs::read_to_string(loop_file(&project)).unwrap();
        assert!(!content.contains("sem_nota"), "{value:?}: {content}");

        fs::write(&path, &late).unwrap();
        rodada(&project);
        assert_eq!(loop_value(&project, "sem_nota_desde"), late_at, "{value:?}");
    }
}

#[test]
fn pergunta_livre_passa_pelo_ciclo_da_cli() {
    let project = Project::new();
    write_request(&project, "PED-001", "perguntar-agente", "pendente");

    project
        .vostok()
        .args(["pedidos", "iniciar", "PED-001"])
        .assert()
        .success();
    let result = "Use CSV.\n\n## Detalhe\n\nSepare por virgula.\n";
    project
        .vostok()
        .args(["pedidos", "concluir", "PED-001"])
        .write_stdin(result)
        .assert()
        .success();

    let listed = stdout_of(project.vostok().args(["pedidos", "listar"]));
    assert!(
        listed.contains("| PED-001 | perguntar-agente |"),
        "{listed}"
    );
    assert!(listed.contains("| feito |"), "{listed}");
    let path = request_in(&project, "feito", "PED-001");
    assert!(path.is_file());
    let content = fs::read_to_string(path).unwrap();
    assert!(
        content.contains(&format!("## Resultado\n\n{result}")),
        "{content}"
    );
}

fn front_value(path: &std::path::Path, key: &str) -> String {
    frontmatter_get(path, key).unwrap_or_default()
}

#[test]
fn notificacao_gravada_ao_concluir() {
    let project = Project::new();
    write_request(&project, "PED-001", "ajustar-tarefa", "em_andamento");

    project
        .vostok()
        .args(["pedidos", "concluir", "PED-001"])
        .write_stdin("ok\n")
        .assert()
        .success();

    assert_eq!(
        front_value(&found(&project, "PED-001"), "notificacao"),
        "nao_lida"
    );
}

#[test]
fn notificacao_gravada_ao_recusar() {
    let project = Project::new();
    write_request(&project, "PED-001", "ajustar-tarefa", "em_andamento");

    project
        .vostok()
        .args(["pedidos", "recusar", "PED-001"])
        .write_stdin("sem motivo\n")
        .assert()
        .success();

    assert_eq!(
        front_value(&found(&project, "PED-001"), "notificacao"),
        "nao_lida"
    );
}

#[test]
fn notificacao_nao_gravada_ao_iniciar() {
    let project = Project::new();
    write_request(&project, "PED-001", "ajustar-tarefa", "pendente");

    project
        .vostok()
        .args(["pedidos", "iniciar", "PED-001"])
        .assert()
        .success();

    assert_eq!(front_value(&found(&project, "PED-001"), "notificacao"), "");
}

#[test]
fn notificacao_gravada_na_recusa_pelo_disparo() {
    use std::os::unix::fs::PermissionsExt;

    let project = Project::new();
    write_request(&project, "PED-001", "ajustar-tarefa", "pendente");
    let bin = project.root().join("bin");
    fs::create_dir_all(&bin).unwrap();
    let script = bin.join("codex");
    fs::write(&script, "#!/bin/sh\nexit 0\n").unwrap();
    fs::set_permissions(&script, fs::Permissions::from_mode(0o755)).unwrap();

    let mut watcher = std::process::Command::new(env!("CARGO_BIN_EXE_vostok"))
        .args(["pedidos", "vigiar", "--agente", "codex", "--intervalo", "1"])
        .current_dir(project.root())
        .env("PATH", path_with(&bin))
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
    while state_of(&project, "PED-001") != "recusado" {
        assert!(
            std::time::Instant::now() < deadline,
            "o pedido não foi recusado"
        );
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    watcher.kill().unwrap();
    watcher.wait().unwrap();

    assert_eq!(
        front_value(&found(&project, "PED-001"), "notificacao"),
        "nao_lida"
    );
}

#[test]
fn notificacao_nao_muda_a_saida_dos_comandos() {
    let project = Project::new();
    write_request(&project, "PED-001", "ajustar-tarefa", "pendente");
    write_request(&project, "PED-002", "ajustar-tarefa", "em_andamento");

    let started = stdout_of(project.vostok().args(["pedidos", "iniciar", "PED-001"]));
    let done = stdout_of(
        project
            .vostok()
            .args(["pedidos", "concluir", "PED-001"])
            .write_stdin("ok\n"),
    );
    let refused = stdout_of(
        project
            .vostok()
            .args(["pedidos", "recusar", "PED-002"])
            .write_stdin("não\n"),
    );

    assert_eq!(started, "Pedido PED-001: pendente -> em_andamento\n");
    assert_eq!(done, "Pedido PED-001: em_andamento -> feito\n");
    assert_eq!(refused, "Pedido PED-002: em_andamento -> recusado\n");
}

fn out_of_requests(args: &[&str], project: &Project) -> String {
    stdout_of(project.vostok().arg("pedidos").args(args))
}

/// Arquivos diretos de `dir` cujo texto contém `needle`, com `file` dentro de cada subpasta, se vier.
fn citing(dir: &std::path::Path, file: Option<&str>, needle: &str) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dir)
        .unwrap()
        .filter_map(|entry| {
            let path = entry.unwrap().path();
            let target = file.map_or(path.clone(), |name| path.join(name));
            let text = fs::read_to_string(target).unwrap_or_default();
            text.contains(needle)
                .then(|| path.file_name().unwrap().to_string_lossy().into_owned())
        })
        .collect();
    names.sort();
    names
}

fn repo(relative: &str) -> PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)
}

#[test]
fn ct58_pedido_por_comando_com_a_mesma_saida() {
    let project = Project::new();
    let _ = fs::remove_dir_all(project.harness().join("work/pedidos"));
    write_request(&project, "PED-001", "ajustar-tarefa", "pendente");
    write_request(&project, "PED-002", "ajustar-tarefa", "pendente");
    assert_eq!(
        out_of_requests(&["iniciar", "PED-001"], &project),
        "Pedido PED-001: pendente -> em_andamento\n"
    );
    assert_eq!(
        out_of_requests(&["iniciar", "PED-002"], &project),
        "Pedido PED-002: pendente -> em_andamento\n"
    );
    let mut concluir = project.vostok();
    concluir
        .args(["pedidos", "concluir", "PED-001"])
        .write_stdin("Pronto.");
    assert_eq!(
        stdout_of(&mut concluir),
        "Pedido PED-001: em_andamento -> feito\n"
    );
    let mut recusar = project.vostok();
    recusar
        .args(["pedidos", "recusar", "PED-002"])
        .write_stdin("Não dá.");
    assert_eq!(
        stdout_of(&mut recusar),
        "Pedido PED-002: em_andamento -> recusado\n"
    );
    assert_eq!(
        out_of_requests(&["listar"], &project),
        "| ID | Ação | Spec | Tarefa | Estado |\n|---|---|---|---|---|\n| PED-001 | ajustar-tarefa | SPEC-001 | TASK-02 | feito |\n| PED-002 | ajustar-tarefa | SPEC-001 | TASK-02 | recusado |\n"
    );
}

#[test]
fn ct58_so_pedidos_e_o_painel_citam_notificacao() {
    assert_eq!(
        citing(&repo("src/commands"), None, "notificacao"),
        ["pedidos.rs"]
    );
    assert!(citing(
        &repo("../../packages/harness/skills/default"),
        Some("SKILL.md"),
        "notificacao"
    )
    .is_empty());
}

#[test]
fn ct72_pergunta_sem_anexos_passa_sem_a_pasta() {
    let project = Project::new();
    let _ = fs::remove_dir_all(project.harness().join("work/pedidos"));
    write_request(&project, "PED-001", "perguntar-agente", "pendente");
    assert_eq!(
        out_of_requests(&["listar"], &project),
        "| ID | Ação | Spec | Tarefa | Estado |\n|---|---|---|---|---|\n| PED-001 | perguntar-agente | SPEC-001 | TASK-02 | pendente |\n"
    );
    assert_eq!(
        out_of_requests(&["iniciar", "PED-001"], &project),
        "Pedido PED-001: pendente -> em_andamento\n"
    );
    let mut concluir = project.vostok();
    concluir
        .args(["pedidos", "concluir", "PED-001"])
        .write_stdin("Resposta.");
    assert_eq!(
        stdout_of(&mut concluir),
        "Pedido PED-001: em_andamento -> feito\n"
    );
    project
        .vostok()
        .args(["pedidos", "prune"])
        .assert()
        .success();
    assert!(!project.harness().join("work/pedidos/anexos").exists());
}

#[test]
fn ct72_so_ask_agent_e_process_requests_citam_a_pasta() {
    let names = citing(
        &repo("../../packages/harness/skills/default"),
        Some("SKILL.md"),
        "## Anexos",
    );
    assert_eq!(names, ["ask-agent", "process-requests"]);
}
