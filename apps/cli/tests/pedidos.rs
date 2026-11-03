//! Testes de `vostok pedidos`: os pedidos do painel mudam de estado só pelo comando.

mod common;

use std::fs;
use std::path::PathBuf;

use common::{failure_stderr, Project};

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
