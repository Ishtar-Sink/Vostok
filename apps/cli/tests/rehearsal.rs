//! Testes do vostok task rehearse: ensaio das tarefas antes da aprovação (SPEC-077).

mod common;

use std::fs;

use common::{failure_stderr, Project};

fn gravar(project: &Project, relative: &str, content: &str) {
    let path = project.root().join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

fn ler(project: &Project, relative: &str) -> String {
    fs::read_to_string(project.root().join(relative)).unwrap_or_default()
}

fn projeto() -> Project {
    let project = Project::new();
    gravar(&project, "valor.txt", "a\n");
    project
}

fn tarefa(
    project: &Project,
    task: &str,
    status: &str,
    mudar: &str,
    baseline: &str,
    verificacao: &str,
    evidencias: &str,
) {
    let spec = "SPEC-001";
    let folder = format!("vostok/work/tasks/{spec}/{task}");
    let files = [
        ("estado.yml", format!("id: {task}\nspec: {spec}\ntitle: \"Tarefa\"\neffort: simples\ndepends_on: []\ncovers: [RF-01]\ntype: tarefa\nlabels: [{spec}, simples]\nstatus: {status}\ncontent_hash: null\nadjustments: 0\nadjusted_at: null\napproved_at: null\napproved_hash: null\nattempts: 0\ntests_result: null\nregression_result: null\nevidence_result: null\n")),
        ("execucao.md", format!("### Ler\n\n- `valor.txt`\n\n### Mudar\n\n{mudar}\n\n### Passos\n\n1. Crie o arquivo.\n")),
        ("verificacao.md", format!("### Baseline\n\n{baseline}\n\n### Verificação\n\n{verificacao}\n\n### Evidências\n\n{evidencias}\n")),
        ("criterios.md", "### Concluída quando\n\n- [ ] Pronto (RF-01).\n\n  > Como comprovar: ver o arquivo.\n".to_string()),
        ("executivo.md", "Resumo.\n".to_string()),
        ("tecnico.md", "Detalhes.\n".to_string()),
        ("registro.md", "## Registro de execução\n\n| # | Data | Etapa | Resultado | Detalhe |\n|---|---|---|---|---|\n\n## Histórico de ajustes\n\n| # | Data | Origem | Resumo |\n|---|---|---|---|\n".to_string()),
    ];
    for (name, content) in files {
        gravar(project, &format!("{folder}/{name}"), &content);
    }
}

const MUDAR: &str = "- `valor.txt` (alterar): valor novo.";
const BASELINE: &str = "- `cat valor.txt`";
const SEM_ROTULO: &str = "```bash\ntrue\n```";
const SEM_EVIDENCIA: &str = "Não se aplica.";

fn chave(project: &Project, bloco: &str) {
    let path = "vostok/config/main.yml";
    let mut kept: Vec<String> = Vec::new();
    let mut inside = false;
    for line in ler(project, path).lines() {
        if line == "# Ensaio das tarefas" || line.starts_with("# Ensaio das tarefas ") {
            continue;
        }
        if line.trim_end() == "rehearsal:" {
            inside = true;
            continue;
        }
        if inside && (line.starts_with(' ') || line.starts_with('\t')) {
            continue;
        }
        inside = false;
        kept.push(line.to_string());
    }
    let mut content = kept.join("\n");
    content.push('\n');
    if !bloco.is_empty() {
        content.push_str(&format!("\n{bloco}\n"));
    }
    gravar(project, path, &content);
}

fn ensaiar(project: &Project, args: &[&str]) -> String {
    let output = project
        .vostok()
        .args(["task", "rehearse"])
        .args(args)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    String::from_utf8(output).unwrap()
}

fn contador() -> tempfile::TempDir {
    let dir = tempfile::TempDir::new().unwrap();
    fs::write(dir.path().join("contagem.txt"), "").unwrap();
    dir
}

fn linhas(dir: &tempfile::TempDir) -> usize {
    fs::read_to_string(dir.path().join("contagem.txt"))
        .map(|text| text.lines().count())
        .unwrap_or(0)
}

fn conta(dir: &tempfile::TempDir) -> String {
    format!("- `echo x >> {}/contagem.txt`", dir.path().display())
}

fn existe(project: &Project, relative: &str) -> bool {
    project.root().join(relative).exists()
}

const T1: &str = "vostok/work/tasks/SPEC-001/TASK-01";
const T2: &str = "vostok/work/tasks/SPEC-001/TASK-02";

#[test]
fn ensaio_sem_problemas_imprime_nenhum_achado() {
    let project = projeto();
    let evidencia =
        "- `ok.txt` (automática): `echo pronto`\n  - Falha: se aparecer `FAILED`, quebrou.";
    for task in ["TASK-01", "TASK-02"] {
        tarefa(
            &project, task, "pending", MUDAR, BASELINE, SEM_ROTULO, evidencia,
        );
    }
    assert_eq!(ensaiar(&project, &["SPEC-001"]), "Nenhum achado.\n");
    for folder in [T1, T2] {
        assert!(existe(&project, &format!("{folder}/ensaio/ensaio.yml")));
        assert!(ler(&project, &format!("{folder}/ensaio/baseline/01.out"))
            .starts_with("$ cat valor.txt\nexit: 0\na\n"));
        assert!(existe(
            &project,
            &format!("{folder}/ensaio/evidencias/ok.txt")
        ));
        assert_eq!(
            ler(&project, &format!("{folder}/ensaio/achados.md")),
            "Nenhum achado.\n"
        );
        let record: serde_json::Value =
            serde_json::from_str(&ler(&project, &format!("{folder}/ensaio/ensaio.yml"))).unwrap();
        assert!(record["task"].is_string());
        assert!(record["tree_fingerprint"].is_string());
        assert!(record["date"].is_string());
        let commands = record["commands"].as_array().unwrap();
        assert_eq!(commands.len(), 2);
        assert_eq!(commands[0]["kind"], "baseline");
        assert_eq!(commands[0]["name"], "01");
        assert_eq!(commands[0]["file"], "baseline/01.out");
        assert_eq!(commands[1]["kind"], "evidencia");
        assert_eq!(commands[1]["name"], "ok.txt");
        assert_eq!(commands[1]["file"], "evidencias/ok.txt");
        for command in commands {
            assert!(command["command"].is_string());
            assert!(command["key"].is_string());
            assert!(command["exit_code"].is_number());
        }
    }
}

#[test]
fn ensaio_falha_na_saida_com_codigo_zero_e_defeito() {
    let project = projeto();
    let evidencia = "- `erros.txt` (automática): `echo '0 errors'`\n  - Falha: se aparecer `error`, a checagem achou erro.";
    tarefa(
        &project, "TASK-01", "pending", MUDAR, BASELINE, SEM_ROTULO, evidencia,
    );
    let saida = ensaiar(&project, &["SPEC-001", "TASK-01"]);
    assert!(
        saida.contains(
            "| TASK-01 | Evidência erros.txt | defeito | Texto de Falha aparece na saída com código 0 |"
        ),
        "{saida}"
    );
    assert!(saida.contains("`error` na linha: 0 errors"), "{saida}");
}

#[test]
fn ensaio_comando_que_quebra_e_a_conferir() {
    let project = projeto();
    let verificacao = "```bash\n# evidência: contagem.txt\ngrep -c naoexiste valor.txt\n```";
    let evidencia =
        "- `contagem.txt` (automática, da verificação)\n  - Falha: se aparecer `quebrou`, falhou.";
    tarefa(
        &project,
        "TASK-01",
        "pending",
        MUDAR,
        BASELINE,
        verificacao,
        evidencia,
    );
    let saida = ensaiar(&project, &["SPEC-001", "TASK-01"]);
    assert!(
        saida.contains(
            "| Verificação contagem.txt | a conferir | Comando termina com código diferente de 0 |"
        ),
        "{saida}"
    );
    assert!(saida.contains("exit: 1; fim da saída: 0"), "{saida}");
}

#[test]
fn ensaio_verificacao_que_ja_passa_e_a_conferir() {
    let project = projeto();
    let verificacao = "```bash\n# evidência: passa.txt\necho ok\n```";
    let evidencia =
        "- `passa.txt` (automática, da verificação)\n  - Falha: se aparecer `quebrou`, falhou.";
    tarefa(
        &project,
        "TASK-01",
        "pending",
        MUDAR,
        BASELINE,
        verificacao,
        evidencia,
    );
    let saida = ensaiar(&project, &["SPEC-001", "TASK-01"]);
    assert!(
        saida.contains(
            "| Verificação passa.txt | a conferir | Comando da Verificação já passa antes da mudança |"
        ),
        "{saida}"
    );
}

#[test]
fn ensaio_baseline_instavel_e_defeito() {
    let project = projeto();
    let baseline = "- `echo 'took 12ms'`\n- `printf 'test a ... ok\\ntest b ... ok\\n'`\n- `echo 'test result: ok. finished in 0.02s'`";
    tarefa(
        &project,
        "TASK-01",
        "pending",
        MUDAR,
        baseline,
        SEM_ROTULO,
        SEM_EVIDENCIA,
    );
    let saida = ensaiar(&project, &["SPEC-001", "TASK-01"]);
    assert!(
        saida.contains("| Baseline [1] | defeito | Saída do Baseline com duração ou horário |"),
        "{saida}"
    );
    assert!(
        saida.contains("| Baseline [2] | defeito | Lista de testes paralelos sem ordenação |"),
        "{saida}"
    );
    assert!(!saida.contains("Baseline [3]"), "{saida}");
}

#[test]
fn ensaio_baseline_que_mede_a_entrega_e_defeito() {
    let project = projeto();
    gravar(&project, "src/lib.txt", "x\n");
    tarefa(
        &project,
        "TASK-01",
        "pending",
        "- `src/novo/x.rs` (criar): arquivo novo.",
        "- `ls src`",
        SEM_ROTULO,
        SEM_EVIDENCIA,
    );
    let saida = ensaiar(&project, &["SPEC-001", "TASK-01"]);
    assert!(
        saida.contains("| Baseline [1] | defeito | Baseline cita a pasta de um caminho criar |"),
        "{saida}"
    );
    assert!(
        saida.contains("`src` é pasta de `src/novo/x.rs`"),
        "{saida}"
    );
}

#[test]
fn ensaio_alterar_inexistente_e_defeito() {
    let project = projeto();
    tarefa(
        &project,
        "TASK-01",
        "pending",
        "- `src/nao-existe.txt` (alterar): muda.",
        BASELINE,
        SEM_ROTULO,
        SEM_EVIDENCIA,
    );
    let saida = ensaiar(&project, &["SPEC-001", "TASK-01"]);
    assert!(
        saida.contains("| Mudar | defeito | Caminho alterar não existe |"),
        "{saida}"
    );
    assert!(
        saida.contains("`src/nao-existe.txt` não existe na raiz do projeto"),
        "{saida}"
    );
}

#[test]
fn ensaio_alterar_criado_por_outra_tarefa_nao_e_achado() {
    let project = projeto();
    tarefa(
        &project,
        "TASK-01",
        "pending",
        "- `src/gerado.txt` (criar): novo.",
        BASELINE,
        SEM_ROTULO,
        SEM_EVIDENCIA,
    );
    tarefa(
        &project,
        "TASK-02",
        "pending",
        "- `src/gerado.txt` (alterar): muda.",
        BASELINE,
        SEM_ROTULO,
        SEM_EVIDENCIA,
    );
    assert_eq!(ensaiar(&project, &["SPEC-001"]), "Nenhum achado.\n");
}

#[test]
fn ensaio_desligado_imprime_so_o_aviso() {
    let project = projeto();
    chave(&project, "rehearsal:\n  task: false");
    let dir = contador();
    tarefa(
        &project,
        "TASK-01",
        "pending",
        MUDAR,
        &conta(&dir),
        SEM_ROTULO,
        SEM_EVIDENCIA,
    );
    assert_eq!(
        ensaiar(&project, &["SPEC-001", "TASK-01"]),
        "Ensaio desligado para task.\n"
    );
    assert_eq!(linhas(&dir), 0);
    assert!(!existe(&project, &format!("{T1}/ensaio")));
}

#[test]
fn ensaio_chave_ausente_ou_vazia_roda() {
    let project = projeto();
    let dir = contador();
    tarefa(
        &project,
        "TASK-01",
        "pending",
        MUDAR,
        &conta(&dir),
        SEM_ROTULO,
        SEM_EVIDENCIA,
    );
    chave(&project, "");
    assert!(ensaiar(&project, &["SPEC-001", "TASK-01"]).contains("Nenhum achado."));
    chave(&project, "rehearsal:\n  task:");
    assert!(ensaiar(&project, &["SPEC-001", "TASK-01"]).contains("Nenhum achado."));
    assert_eq!(linhas(&dir), 2);
}

#[test]
fn ensaio_chave_invalida_e_recusada() {
    let project = projeto();
    chave(&project, "rehearsal:\n  task: talvez");
    tarefa(
        &project,
        "TASK-01",
        "pending",
        MUDAR,
        BASELINE,
        SEM_ROTULO,
        SEM_EVIDENCIA,
    );
    let stderr = failure_stderr(
        project
            .vostok()
            .args(["task", "rehearse", "SPEC-001", "TASK-01"]),
    );
    assert!(
        stderr.contains(
            "valor inválido em config/main.yml (rehearsal.task): 'talvez'; use true ou false"
        ),
        "{stderr}"
    );
}

#[test]
fn ensaio_nao_muda_a_tarefa() {
    let project = projeto();
    let verificacao = "```bash\n# evidência: passa.txt\necho ok\n```";
    let evidencia = "- `ok.txt` (automática): `echo pronto`\n  - Falha: se aparecer `FAILED`, quebrou.\n- `passa.txt` (automática, da verificação)\n  - Falha: se aparecer `quebrou`, falhou.";
    tarefa(
        &project,
        "TASK-01",
        "pending",
        MUDAR,
        BASELINE,
        verificacao,
        evidencia,
    );
    let estado = ler(&project, &format!("{T1}/estado.yml"));
    let registro = ler(&project, &format!("{T1}/registro.md"));
    ensaiar(&project, &["SPEC-001", "TASK-01"]);
    assert_eq!(ler(&project, &format!("{T1}/estado.yml")), estado);
    assert_eq!(ler(&project, &format!("{T1}/registro.md")), registro);
    assert!(!existe(&project, &format!("{T1}/evidencias")));
}

#[test]
fn ensaio_tarefa_iniciada_e_recusada_sem_rodar() {
    let project = projeto();
    let dir = contador();
    tarefa(
        &project,
        "TASK-01",
        "pending",
        MUDAR,
        &conta(&dir),
        SEM_ROTULO,
        SEM_EVIDENCIA,
    );
    tarefa(
        &project,
        "TASK-02",
        "in_progress",
        MUDAR,
        &conta(&dir),
        SEM_ROTULO,
        SEM_EVIDENCIA,
    );
    let stderr = failure_stderr(
        project
            .vostok()
            .args(["task", "rehearse", "SPEC-001", "TASK-01", "TASK-02"]),
    );
    assert!(
        stderr.contains("a tarefa TASK-02 já foi iniciada (status 'in_progress'); o ensaio vale só antes do vostok task start; nenhum comando foi rodado"),
        "{stderr}"
    );
    assert_eq!(linhas(&dir), 0);
    assert!(!existe(&project, &format!("{T1}/ensaio")));
    assert!(!existe(&project, &format!("{T2}/ensaio")));
}

#[test]
fn ensaio_sem_ids_pula_concluidas_e_iniciadas() {
    let project = projeto();
    let dir = contador();
    for (task, status) in [
        ("TASK-01", "pending"),
        ("TASK-02", "done"),
        ("TASK-03", "in_progress"),
    ] {
        tarefa(
            &project,
            task,
            status,
            MUDAR,
            &conta(&dir),
            SEM_ROTULO,
            SEM_EVIDENCIA,
        );
    }
    let output = project
        .vostok()
        .args(["task", "rehearse", "SPEC-001"])
        .assert()
        .success()
        .get_output()
        .clone();
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert_eq!(linhas(&dir), 1);
    assert!(existe(&project, &format!("{T1}/ensaio")));
    assert!(!existe(&project, &format!("{T2}/ensaio")));
    assert!(!existe(
        &project,
        "vostok/work/tasks/SPEC-001/TASK-03/ensaio"
    ));
    assert!(
        stderr.contains("- TASK-03: já iniciada (status 'in_progress'); fora do ensaio"),
        "{stderr}"
    );
}

#[test]
fn ensaio_tempo_limite_e_a_conferir() {
    let project = projeto();
    let evidencia = "- `lenta.txt` (automática): `sleep 5`\n  - Tempo-limite: 1";
    tarefa(
        &project, "TASK-01", "pending", MUDAR, BASELINE, SEM_ROTULO, evidencia,
    );
    let saida = ensaiar(&project, &["SPEC-001", "TASK-01"]);
    assert!(
        saida.contains(
            "| Evidência lenta.txt | a conferir | Comando termina com código diferente de 0 |"
        ),
        "{saida}"
    );
    assert!(saida.contains("exit: 124"), "{saida}");
    assert!(saida.contains("interrompida após 1 s"), "{saida}");
}

#[test]
fn ensaio_evidencia_sem_falha_so_confere_o_codigo() {
    let project = projeto();
    let evidencia = "- `sem.txt` (automática): `echo error; exit 3`";
    tarefa(
        &project, "TASK-01", "pending", MUDAR, BASELINE, SEM_ROTULO, evidencia,
    );
    let saida = ensaiar(&project, &["SPEC-001", "TASK-01"]);
    assert!(saida.contains("exit: 3"), "{saida}");
    assert!(!saida.contains("defeito"), "{saida}");
}

#[test]
fn ensaio_tarefa_sem_baseline_segue() {
    let project = projeto();
    let evidencia =
        "- `ok.txt` (automática): `echo pronto`\n  - Falha: se aparecer `FAILED`, quebrou.";
    tarefa(
        &project,
        "TASK-01",
        "pending",
        MUDAR,
        SEM_EVIDENCIA,
        SEM_ROTULO,
        evidencia,
    );
    assert_eq!(
        ensaiar(&project, &["SPEC-001", "TASK-01"]),
        "Nenhum achado.\n"
    );
    assert!(existe(&project, &format!("{T1}/ensaio/evidencias/ok.txt")));
    assert!(!existe(&project, &format!("{T1}/ensaio/baseline")));
}

#[test]
fn ensaio_depende_inexistente_nao_roda() {
    let project = projeto();
    let dir = contador();
    let evidencia = format!(
        "- `dep.txt` (automática): `echo x >> {}/contagem.txt`\n  - Depende: `assets/novo/`",
        dir.path().display()
    );
    tarefa(
        &project, "TASK-01", "pending", MUDAR, BASELINE, SEM_ROTULO, &evidencia,
    );
    let saida = ensaiar(&project, &["SPEC-001", "TASK-01"]);
    assert!(
        saida.contains(
            "| Evidência dep.txt | a conferir | Evidência não rodada: caminho ainda não existe |"
        ),
        "{saida}"
    );
    assert!(saida.contains("`assets/novo/` ainda não existe"), "{saida}");
    assert_eq!(linhas(&dir), 0);
}

#[test]
fn ensaio_nao_roda_print_manual_nem_diff() {
    let project = projeto();
    let dir = contador();
    let evidencia = format!(
        "- `tela.png` (print): `echo x >> {}/contagem.txt`\n- `valor.diff` (diff): `valor.txt`\n- `conferir.txt` (manual): abrir a tela",
        dir.path().display()
    );
    tarefa(
        &project, "TASK-01", "pending", MUDAR, BASELINE, SEM_ROTULO, &evidencia,
    );
    ensaiar(&project, &["SPEC-001", "TASK-01"]);
    assert_eq!(linhas(&dir), 0);
    let record: serde_json::Value =
        serde_json::from_str(&ler(&project, &format!("{T1}/ensaio/ensaio.yml"))).unwrap();
    let commands = record["commands"].as_array().unwrap();
    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0]["kind"], "baseline");
}
