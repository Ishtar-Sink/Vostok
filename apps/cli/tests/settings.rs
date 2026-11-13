//! Testes das configurações das evidências (SPEC-052, TASK-01).

mod common;

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use common::Project;
use vostok::harness::Harness;
use vostok::settings::*;

const TASKS: &str = "tasks.yml";
const DOCUMENTS: &str = "documents.yml";

fn project() -> (Project, Harness) {
    let project = Project::new();
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packages/harness/config");
    for name in [TASKS, DOCUMENTS] {
        fs::copy(
            source.join(name),
            project.harness().join("config").join(name),
        )
        .expect("copiar o arquivo de configuração");
    }
    let harness = Harness::locate(project.root()).expect("localizar o harness");
    (project, harness)
}

fn config(harness: &Harness, name: &str) -> String {
    fs::read_to_string(harness.config_file(name)).expect("ler a configuração")
}

fn set(harness: &Harness, name: &str, content: &str) {
    fs::write(harness.config_file(name), content).expect("gravar a configuração");
}

fn snapshot(harness: &Harness) -> SettingsSnapshot {
    read_evidence_settings(harness).expect("ler as configurações")
}

fn request(snapshot: &SettingsSnapshot, values: &[(&str, SettingValue)]) -> SettingsRequest {
    SettingsRequest {
        values: values
            .iter()
            .map(|(key, value)| (key.to_string(), value.clone()))
            .collect(),
        files: snapshot.files.clone(),
    }
}

fn list(items: &[&str]) -> SettingValue {
    SettingValue::List(items.iter().map(|item| item.to_string()).collect())
}

fn text(value: &str) -> SettingValue {
    SettingValue::Text(value.to_string())
}

fn invalid(result: Result<SettingsSnapshot, SettingsError>) -> BTreeMap<String, String> {
    match result {
        Err(SettingsError::Invalid(errors)) => errors,
        other => panic!("esperava Invalid, veio {other:?}"),
    }
}

fn write(harness: &Harness, values: &[(&str, SettingValue)]) -> SettingsSnapshot {
    let current = snapshot(harness);
    write_evidence_settings(harness, &request(&current, values)).expect("gravar as configurações")
}

fn changed_pairs(before: &str, after: &str) -> Vec<(String, String)> {
    let before: Vec<&str> = before.lines().collect();
    let after: Vec<&str> = after.lines().collect();
    if before.len() != after.len() {
        panic!(
            "número de linhas diferente: {} antes, {} depois",
            before.len(),
            after.len()
        );
    }
    before
        .iter()
        .zip(&after)
        .filter(|(old, new)| old != new)
        .map(|(old, new)| (old.to_string(), new.to_string()))
        .collect()
}

fn diff_region(before: &str, after: &str) -> (Vec<String>, Vec<String>) {
    let before: Vec<&str> = before.lines().collect();
    let after: Vec<&str> = after.lines().collect();
    let shortest = before.len().min(after.len());
    let prefix = (0..shortest)
        .take_while(|&index| before[index] == after[index])
        .count();
    let suffix = (0..shortest - prefix)
        .take_while(|&index| before[before.len() - 1 - index] == after[after.len() - 1 - index])
        .count();
    let owned = |lines: &[&str]| lines.iter().map(|line| line.to_string()).collect();
    (
        owned(&before[prefix..before.len() - suffix]),
        owned(&after[prefix..after.len() - suffix]),
    )
}

/// Conteúdo dos dois arquivos, para conferir que nada foi gravado.
fn both(harness: &Harness) -> (String, String) {
    (config(harness, TASKS), config(harness, DOCUMENTS))
}

fn without_line(content: &str, prefix: &str) -> String {
    content
        .lines()
        .filter(|line| !line.starts_with(prefix))
        .map(|line| format!("{line}\n"))
        .collect()
}

fn five_kinds() -> SettingValue {
    list(&[
        "automática",
        "automática, da verificação",
        "diff",
        "manual",
        "print",
    ])
}

fn text_extensions_plus(extra: &[&str]) -> SettingValue {
    let mut items = vec!["out", "txt", "log", "md", "json"];
    items.extend_from_slice(extra);
    list(&items)
}

// --- Leitura ---

#[test]
fn leitura_traz_as_chaves_da_lista_fechada_e_o_sha256() {
    let (_project, harness) = project();
    let read = snapshot(&harness);
    let expected: BTreeMap<String, SettingValue> = [
        ("tasks.evidence_review_kinds", five_kinds()),
        ("tasks.evidence_baseline_timeout", SettingValue::Int(120)),
        ("tasks.evidence_title_max", SettingValue::Int(60)),
        ("tasks.persist_dir", text("")),
        ("tasks.persist_max_mb", SettingValue::Int(10)),
        ("tasks.persist_readme", text("README.md")),
        (
            "tasks.regression_ignore",
            list(&["Start at", "Duration", "finished in", "Time:"]),
        ),
        (
            "tasks.regression_alert",
            list(&["FAILED", "error", "warning", "panicked"]),
        ),
        ("documents.embed_max_mb", SettingValue::Int(10)),
        (
            "documents.text_extensions",
            list(&["out", "txt", "log", "md", "json"]),
        ),
        (
            "documents.image_extensions",
            list(&["png", "jpg", "jpeg", "gif", "webp", "svg"]),
        ),
        ("documents.video_extensions", list(&["mp4", "webm"])),
    ]
    .into_iter()
    .map(|(key, value)| (key.to_string(), value))
    .collect();
    assert_eq!(read.values, expected);
    for name in [TASKS, DOCUMENTS] {
        let bytes = fs::read(harness.config_file(name)).unwrap();
        assert_eq!(read.files[name], vostok::support::sha256_hex(&bytes));
    }
    assert_eq!(read.files.len(), 2);
}

#[test]
fn leitura_usa_o_padrao_do_harness_para_chave_ausente() {
    let (_project, harness) = project();
    let tasks = without_line(&config(&harness, TASKS), "evidence_review_kinds:");
    set(&harness, TASKS, &tasks);
    let documents = without_line(&config(&harness, DOCUMENTS), "embed_max_mb:");
    set(&harness, DOCUMENTS, &documents);
    let read = snapshot(&harness);
    assert_eq!(read.values["tasks.evidence_review_kinds"], five_kinds());
    assert_eq!(read.values["documents.embed_max_mb"], SettingValue::Int(10));
}

// --- Validação ---

#[test]
fn validacao_recusa_numeros_fora_da_faixa() {
    let (_project, harness) = project();
    let before = both(&harness);
    let current = snapshot(&harness);
    let errors = invalid(write_evidence_settings(
        &harness,
        &request(
            &current,
            &[
                ("tasks.evidence_baseline_timeout", SettingValue::Int(-1)),
                ("tasks.evidence_title_max", SettingValue::Int(500)),
            ],
        ),
    ));
    let expected: BTreeMap<String, String> = [
        (
            "tasks.evidence_baseline_timeout",
            "o valor deve estar entre 0 e 3600",
        ),
        (
            "tasks.evidence_title_max",
            "o valor deve estar entre 20 e 200",
        ),
    ]
    .into_iter()
    .map(|(key, message)| (key.to_string(), message.to_string()))
    .collect();
    assert_eq!(errors, expected);
    for (key, value, message) in [
        (
            "tasks.persist_max_mb",
            1025,
            "o valor deve estar entre 0 e 1024",
        ),
        (
            "documents.embed_max_mb",
            101,
            "o valor deve estar entre 0 e 100",
        ),
        (
            "tasks.evidence_title_max",
            19,
            "o valor deve estar entre 20 e 200",
        ),
    ] {
        let errors = invalid(write_evidence_settings(
            &harness,
            &request(&current, &[(key, SettingValue::Int(value))]),
        ));
        assert_eq!(errors.len(), 1, "{errors:?}");
        assert_eq!(errors[key], message);
    }
    assert_eq!(both(&harness), before);
}

#[test]
fn validacao_aceita_os_limites_das_faixas() {
    let (_project, harness) = project();
    let keys = [
        "tasks.evidence_baseline_timeout",
        "tasks.evidence_title_max",
        "tasks.persist_max_mb",
        "documents.embed_max_mb",
    ];
    for limits in [[0, 20, 0, 0], [3600, 200, 1024, 100]] {
        let values: Vec<(&str, SettingValue)> = keys
            .iter()
            .zip(limits)
            .map(|(key, value)| (*key, SettingValue::Int(value)))
            .collect();
        write(&harness, &values);
        let read = snapshot(&harness);
        for (key, value) in &values {
            assert_eq!(&read.values[*key], value);
        }
    }
}

#[test]
fn validacao_recusa_chave_fora_da_lista() {
    let (_project, harness) = project();
    let before = both(&harness);
    let current = snapshot(&harness);
    for key in [
        "tasks.evidence_automatic",
        "tasks.max_attempts",
        "main.paths",
        "evidence_title_max",
    ] {
        let errors = invalid(write_evidence_settings(
            &harness,
            &request(&current, &[(key, SettingValue::Int(3))]),
        ));
        assert_eq!(
            errors[key], "chave fora da lista de configurações liberadas",
            "{key}"
        );
    }
    assert_eq!(both(&harness), before);
}

#[test]
fn validacao_recusa_tipo_de_valor_errado() {
    let (_project, harness) = project();
    let before = both(&harness);
    let current = snapshot(&harness);
    let errors = invalid(write_evidence_settings(
        &harness,
        &request(
            &current,
            &[
                ("tasks.evidence_title_max", text("60")),
                ("tasks.persist_dir", SettingValue::Int(1)),
                ("tasks.regression_alert", text("FAILED")),
            ],
        ),
    ));
    assert_eq!(
        errors["tasks.evidence_title_max"],
        "o valor deve ser um número inteiro"
    );
    assert_eq!(errors["tasks.persist_dir"], "o valor deve ser um texto");
    assert_eq!(
        errors["tasks.regression_alert"],
        "o valor deve ser uma lista de textos"
    );
    assert_eq!(both(&harness), before);
}

#[test]
fn validacao_recusa_tipos_de_conferencia_desconhecidos_ou_repetidos() {
    let (_project, harness) = project();
    let before = both(&harness);
    let current = snapshot(&harness);
    for (kinds, message) in [
        (
            list(&["revisão"]),
            "tipo de evidência desconhecido: revisão",
        ),
        (list(&["diff", "diff"]), "tipo repetido: diff"),
    ] {
        let errors = invalid(write_evidence_settings(
            &harness,
            &request(&current, &[("tasks.evidence_review_kinds", kinds)]),
        ));
        assert_eq!(errors["tasks.evidence_review_kinds"], message);
    }
    assert_eq!(both(&harness), before);
}

#[test]
fn validacao_recusa_persist_dir_invalido() {
    let (_project, harness) = project();
    let before = both(&harness);
    let current = snapshot(&harness);
    for (dir, message) in [
        ("docs/evidencias", "o caminho deve ter <SPEC>"),
        (
            "/tmp/<SPEC>",
            "o caminho deve ser relativo à raiz do projeto",
        ),
        (
            "C:\\docs\\<SPEC>",
            "o caminho deve ser relativo à raiz do projeto",
        ),
        ("docs/../<SPEC>", "o caminho não pode ter .."),
    ] {
        let errors = invalid(write_evidence_settings(
            &harness,
            &request(&current, &[("tasks.persist_dir", text(dir))]),
        ));
        assert_eq!(errors["tasks.persist_dir"], message, "{dir}");
    }
    assert_eq!(both(&harness), before);
}

#[test]
fn validacao_recusa_persist_readme_invalido() {
    let (_project, harness) = project();
    let before = both(&harness);
    let current = snapshot(&harness);
    for name in ["guia.txt", "docs/GUIA.md", ".md"] {
        let errors = invalid(write_evidence_settings(
            &harness,
            &request(&current, &[("tasks.persist_readme", text(name))]),
        ));
        assert_eq!(
            errors["tasks.persist_readme"], "o nome deve ser um arquivo .md, sem barra",
            "{name}"
        );
    }
    assert_eq!(both(&harness), before);
}

#[test]
fn validacao_recusa_trechos_vazios_repetidos_ou_com_aspas() {
    let (_project, harness) = project();
    let before = both(&harness);
    let current = snapshot(&harness);
    let quotes = "o trecho não pode ter aspas duplas nem espaço seguido de #";
    for (key, items, message) in [
        (
            "tasks.regression_ignore",
            list(&["", "x"]),
            "os trechos não podem ser vazios",
        ),
        (
            "tasks.regression_alert",
            list(&["FAILED", "FAILED"]),
            "trecho repetido: FAILED",
        ),
        ("tasks.regression_alert", list(&["a \"b\""]), quotes),
        ("tasks.regression_alert", list(&["a #b"]), quotes),
    ] {
        let errors = invalid(write_evidence_settings(
            &harness,
            &request(&current, &[(key, items)]),
        ));
        assert_eq!(errors[key], message);
    }
    assert_eq!(both(&harness), before);
}

#[test]
fn validacao_normaliza_e_recusa_extensoes() {
    let (_project, harness) = project();
    let before = both(&harness);
    let current = snapshot(&harness);
    let errors = invalid(write_evidence_settings(
        &harness,
        &request(
            &current,
            &[(
                "documents.text_extensions",
                text_extensions_plus(&["NDJSON", "png"]),
            )],
        ),
    ));
    let expected: BTreeMap<String, String> = [(
        "documents.text_extensions".to_string(),
        "extensão repetida: png, já está em documents.image_extensions".to_string(),
    )]
    .into();
    assert_eq!(errors, expected);
    assert_eq!(both(&harness), before);

    for (items, message) in [
        (
            text_extensions_plus(&["nd json"]),
            "extensão inválida: nd json; use só letras minúsculas e números",
        ),
        (list(&["txt", "TXT"]), "extensão repetida: txt"),
    ] {
        let errors = invalid(write_evidence_settings(
            &harness,
            &request(&current, &[("documents.text_extensions", items)]),
        ));
        assert_eq!(errors["documents.text_extensions"], message);
    }
    assert_eq!(both(&harness), before);

    write(
        &harness,
        &[(
            "documents.text_extensions",
            text_extensions_plus(&[".NDJSON"]),
        )],
    );
    assert_eq!(
        snapshot(&harness).values["documents.text_extensions"],
        text_extensions_plus(&["ndjson"])
    );
}

#[test]
fn validacao_recusa_pedido_sem_o_sha256_do_arquivo_lido() {
    let (_project, harness) = project();
    let before = both(&harness);
    let current = snapshot(&harness);
    let mut pedido = request(
        &current,
        &[("tasks.evidence_baseline_timeout", SettingValue::Int(300))],
    );
    pedido.files.remove(TASKS);
    let errors = invalid(write_evidence_settings(&harness, &pedido));
    let expected: BTreeMap<String, String> = [(
        "arquivos.tasks.yml".to_string(),
        "falta o sha256 lido do arquivo".to_string(),
    )]
    .into();
    assert_eq!(errors, expected);
    assert_eq!(both(&harness), before);
}

// --- Gravação ---

#[test]
fn gravacao_troca_so_a_linha_da_lista_na_linha() {
    let (_project, harness) = project();
    let (tasks, documents) = both(&harness);
    write(
        &harness,
        &[(
            "tasks.evidence_review_kinds",
            list(&[
                "automática",
                "automática, da verificação",
                "manual",
                "print",
            ]),
        )],
    );
    assert_eq!(
        changed_pairs(&tasks, &config(&harness, TASKS)),
        vec![(
            "evidence_review_kinds: [automática, \"automática, da verificação\", diff, manual, print]"
                .to_string(),
            "evidence_review_kinds: [automática, \"automática, da verificação\", manual, print]"
                .to_string(),
        )]
    );
    assert_eq!(config(&harness, DOCUMENTS), documents);
}

#[test]
fn gravacao_mantem_a_lista_em_bloco() {
    let (_project, harness) = project();
    let (tasks, documents) = both(&harness);
    write(
        &harness,
        &[(
            "documents.text_extensions",
            text_extensions_plus(&["ndjson"]),
        )],
    );
    assert_eq!(
        diff_region(&documents, &config(&harness, DOCUMENTS)),
        (vec![], vec!["  - ndjson".to_string()])
    );
    write(
        &harness,
        &[(
            "tasks.regression_ignore",
            list(&["Start at", "Elapsed", "finished in", "Time:"]),
        )],
    );
    let after = config(&harness, TASKS);
    assert_eq!(
        diff_region(&tasks, &after),
        (
            vec!["  - Duration".to_string()],
            vec!["  - Elapsed".to_string()]
        )
    );
    assert!(after.lines().any(|line| line == "  - \"Time:\""));
}

#[test]
fn gravacao_troca_escalares_mantendo_o_resto() {
    let (_project, harness) = project();
    let tasks = config(&harness, TASKS);
    write(
        &harness,
        &[
            ("tasks.evidence_baseline_timeout", SettingValue::Int(300)),
            ("tasks.persist_dir", text("docs/evidencias/<SPEC>/<TASK>")),
            ("tasks.persist_readme", text("GUIA.md")),
        ],
    );
    let pair = |old: &str, new: &str| (old.to_string(), new.to_string());
    assert_eq!(
        changed_pairs(&tasks, &config(&harness, TASKS)),
        vec![
            pair(
                "evidence_baseline_timeout: 120",
                "evidence_baseline_timeout: 300"
            ),
            pair(
                "persist_dir: \"\"",
                "persist_dir: \"docs/evidencias/<SPEC>/<TASK>\""
            ),
            pair("persist_readme: README.md", "persist_readme: GUIA.md"),
        ]
    );
}

#[test]
fn gravacao_mantem_o_comentario_na_linha_do_valor() {
    let (_project, harness) = project();
    let tasks = config(&harness, TASKS).replace(
        "evidence_title_max: 60\n",
        "evidence_title_max: 60  # limite\n",
    );
    set(&harness, TASKS, &tasks);
    write(
        &harness,
        &[("tasks.evidence_title_max", SettingValue::Int(80))],
    );
    assert_eq!(
        changed_pairs(&tasks, &config(&harness, TASKS)),
        vec![(
            "evidence_title_max: 60  # limite".to_string(),
            "evidence_title_max: 80  # limite".to_string()
        )]
    );
}

#[test]
fn gravacao_mantem_comentario_entre_itens_da_lista() {
    let (_project, harness) = project();
    let tasks = config(&harness, TASKS).replace(
        "  - Start at\n  - Duration\n",
        "  - Start at\n  # nota\n  - Duration\n",
    );
    set(&harness, TASKS, &tasks);
    write(
        &harness,
        &[("tasks.regression_ignore", list(&["Start at", "Elapsed"]))],
    );
    let expected = tasks.replace(
        "regression_ignore:\n  - Start at\n  # nota\n  - Duration\n  - finished in\n  - \"Time:\"\n",
        "regression_ignore:\n  # nota\n  - Start at\n  - Elapsed\n",
    );
    assert_ne!(expected, tasks);
    assert_eq!(config(&harness, TASKS), expected);
}

#[test]
fn gravacao_lista_vazia_fica_legivel() {
    let (_project, harness) = project();
    write(
        &harness,
        &[
            ("tasks.evidence_review_kinds", list(&[])),
            ("tasks.regression_alert", list(&[])),
        ],
    );
    let tasks = config(&harness, TASKS);
    let lines: Vec<&str> = tasks.lines().collect();
    assert!(lines.contains(&"evidence_review_kinds: []"));
    let alert = lines
        .iter()
        .position(|line| *line == "regression_alert:")
        .expect("cabeçalho de regression_alert");
    assert!(!lines[alert + 1].trim_start().starts_with("- "));
    let read = snapshot(&harness);
    assert_eq!(read.values["tasks.evidence_review_kinds"], list(&[]));
    assert_eq!(read.values["tasks.regression_alert"], list(&[]));
    assert_eq!(
        vostok::support::config_list(&harness.config_file(TASKS), "regression_alert").unwrap(),
        Vec::<String>::new()
    );
}

#[test]
fn gravacao_acrescenta_chave_ausente_no_fim() {
    let (_project, harness) = project();
    let tasks = without_line(&config(&harness, TASKS), "evidence_review_kinds:");
    set(&harness, TASKS, &tasks);
    write(
        &harness,
        &[("tasks.evidence_review_kinds", list(&["manual"]))],
    );
    let after = config(&harness, TASKS);
    let added = after
        .strip_prefix(&tasks)
        .expect("o arquivo de antes continua como prefixo");
    assert!(!tasks.ends_with("\n\n"));
    assert_eq!(added, "\nevidence_review_kinds:\n  - manual\n");
    assert_eq!(
        snapshot(&harness).values["tasks.evidence_review_kinds"],
        list(&["manual"])
    );
}

#[test]
fn gravacao_valor_igual_nao_muda_o_arquivo() {
    let (_project, harness) = project();
    let before = both(&harness);
    let current = snapshot(&harness);
    let values: Vec<(&str, SettingValue)> = current
        .values
        .iter()
        .map(|(key, value)| (key.as_str(), value.clone()))
        .collect();
    let result =
        write_evidence_settings(&harness, &request(&current, &values)).expect("gravar igual");
    assert_eq!(both(&harness), before);
    assert_eq!(result.files, current.files);
}

#[test]
fn gravacao_devolve_valores_e_sha256_novos_sem_arquivo_tmp() {
    let (_project, harness) = project();
    let current = snapshot(&harness);
    let result = write(
        &harness,
        &[("tasks.evidence_baseline_timeout", SettingValue::Int(300))],
    );
    assert_eq!(result, snapshot(&harness));
    assert_ne!(result.files[TASKS], current.files[TASKS]);
    assert_eq!(result.files[DOCUMENTS], current.files[DOCUMENTS]);
    let leftovers: Vec<String> = fs::read_dir(harness.config_file(""))
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|name| name.ends_with(".tmp"))
        .collect();
    assert!(leftovers.is_empty(), "{leftovers:?}");
}

// --- Conflito ---

#[test]
fn conflito_quando_o_arquivo_muda_depois_da_leitura() {
    let (_project, harness) = project();
    let current = snapshot(&harness);
    let edited = format!("{}# editado à mão\n", config(&harness, TASKS));
    set(&harness, TASKS, &edited);
    let result = write_evidence_settings(
        &harness,
        &request(
            &current,
            &[("tasks.evidence_baseline_timeout", SettingValue::Int(300))],
        ),
    );
    match result {
        Err(SettingsError::Conflict { file }) => assert_eq!(file, TASKS),
        other => panic!("esperava Conflict, veio {other:?}"),
    }
    let after = config(&harness, TASKS);
    assert_eq!(after, edited);
    assert!(!after.contains("evidence_baseline_timeout: 300"));
}

#[test]
fn conflito_so_vale_para_os_arquivos_das_chaves() {
    let (_project, harness) = project();
    let current = snapshot(&harness);
    let edited = format!("{}# editado à mão\n", config(&harness, DOCUMENTS));
    set(&harness, DOCUMENTS, &edited);
    let result = write_evidence_settings(
        &harness,
        &request(
            &current,
            &[("tasks.evidence_baseline_timeout", SettingValue::Int(300))],
        ),
    )
    .expect("gravar só o tasks.yml");
    assert_eq!(
        result.values["tasks.evidence_baseline_timeout"],
        SettingValue::Int(300)
    );
    assert_eq!(config(&harness, DOCUMENTS), edited);
}

// --- Tudo ou nada ---

#[test]
fn tudo_ou_nada_um_valor_invalido_nao_grava_nenhum() {
    let (_project, harness) = project();
    let before = both(&harness);
    let current = snapshot(&harness);
    let errors = invalid(write_evidence_settings(
        &harness,
        &request(
            &current,
            &[
                ("tasks.evidence_baseline_timeout", SettingValue::Int(300)),
                ("documents.embed_max_mb", SettingValue::Int(500)),
            ],
        ),
    ));
    assert_eq!(
        errors.keys().collect::<Vec<_>>(),
        vec!["documents.embed_max_mb"]
    );
    assert_eq!(both(&harness), before);
}

#[test]
fn tudo_ou_nada_conflito_num_arquivo_nao_grava_o_outro() {
    let (_project, harness) = project();
    let current = snapshot(&harness);
    let tasks = config(&harness, TASKS);
    set(
        &harness,
        DOCUMENTS,
        &format!("{}# editado à mão\n", config(&harness, DOCUMENTS)),
    );
    let result = write_evidence_settings(
        &harness,
        &request(
            &current,
            &[
                ("tasks.evidence_baseline_timeout", SettingValue::Int(300)),
                ("documents.embed_max_mb", SettingValue::Int(20)),
            ],
        ),
    );
    match result {
        Err(SettingsError::Conflict { file }) => assert_eq!(file, DOCUMENTS),
        other => panic!("esperava Conflict, veio {other:?}"),
    }
    assert_eq!(config(&harness, TASKS), tasks);
}

#[test]
fn tudo_ou_nada_devolve_os_erros_de_todas_as_chaves() {
    let (_project, harness) = project();
    let before = both(&harness);
    let current = snapshot(&harness);
    let errors = invalid(write_evidence_settings(
        &harness,
        &request(
            &current,
            &[
                ("tasks.evidence_baseline_timeout", SettingValue::Int(-1)),
                ("tasks.evidence_title_max", SettingValue::Int(5)),
                ("tasks.persist_readme", text("x")),
                ("documents.video_extensions", list(&["a b"])),
            ],
        ),
    ));
    assert_eq!(
        errors.keys().collect::<Vec<_>>(),
        vec![
            "documents.video_extensions",
            "tasks.evidence_baseline_timeout",
            "tasks.evidence_title_max",
            "tasks.persist_readme",
        ]
    );
    assert_eq!(both(&harness), before);
}

#[test]
fn tudo_ou_nada_grava_os_dois_arquivos_juntos() {
    let (_project, harness) = project();
    let before = both(&harness);
    let result = write(
        &harness,
        &[
            ("tasks.evidence_title_max", SettingValue::Int(80)),
            ("documents.embed_max_mb", SettingValue::Int(20)),
        ],
    );
    let after = both(&harness);
    assert_ne!(after.0, before.0);
    assert_ne!(after.1, before.1);
    assert_eq!(
        result.values["tasks.evidence_title_max"],
        SettingValue::Int(80)
    );
    assert_eq!(
        result.values["documents.embed_max_mb"],
        SettingValue::Int(20)
    );
}
