//! Cenários de referência do ciclo da tarefa: cada um roda o `vostok` num projeto novo e compara a
//! transcrição normalizada com a fixture em `fixtures/scenarios/<cenário>.txt`. Com
//! `VOSTOK_SCENARIOS_RECORD=1` a fixture é regravada em vez de comparada.

mod common;

use std::fs;
use std::path::PathBuf;

use common::Project;

const SPEC: &str = "SPEC-001";
const RECORD_VARIABLE: &str = "VOSTOK_SCENARIOS_RECORD";
const AUTOMATIC_EVIDENCE: &str = "- `saida.txt` (automática): `cat leitura.txt`\n  - Prova: o arquivo é lido.\n  - Trecho: a linha `ok`.\n  - Falha: arquivo ausente.";
const EXPECTING_EVIDENCE: &str = "- `saida.txt` (automática): `cat leitura.txt`\n  - Prova: o arquivo é lido.\n  - Trecho: a linha `ok`.\n  - Falha: arquivo ausente.\n  - Espera: ok";

// --- Acesso ao formato da tarefa: só estas funções o conhecem ---

/// Formato em que o cenário grava as tarefas. O resultado tem de ser o mesmo nos três.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Format {
    /// Arquivo único `TASK-NN.md`.
    Single,
    /// Pasta `TASK-NN/`, gravada direto.
    Folder,
    /// Arquivo único convertido com `vostok task migrate` antes do primeiro comando.
    Migrated,
}

const FORMATS: [Format; 3] = [Format::Single, Format::Folder, Format::Migrated];

/// Dados de uma tarefa de cenário; o que não é informado tem um valor padrão que passa em tudo.
struct TaskSource<'a> {
    id: &'a str,
    status: &'a str,
    depends_on: &'a str,
    baseline: &'a str,
    verification: &'a str,
    evidence: &'a str,
}

impl<'a> TaskSource<'a> {
    fn new(id: &'a str) -> Self {
        Self {
            id,
            status: "pending",
            depends_on: "[]",
            baseline: "- `echo base`",
            verification: "true",
            evidence: "Não se aplica.",
        }
    }
}

fn task_path(id: &str) -> String {
    format!("vostok/work/tasks/{SPEC}/{id}.md")
}

fn task_folder(id: &str) -> String {
    format!("vostok/work/tasks/{SPEC}/{id}")
}

/// A tarefa está em pasta (gravada assim ou já migrada).
fn in_folder(scenario: &Scenario, id: &str) -> bool {
    scenario
        .project
        .root()
        .join(task_folder(id))
        .join("estado.yml")
        .exists()
}

/// Arquivo da tarefa com os campos do estado.
fn state_path(scenario: &Scenario, id: &str) -> String {
    if in_folder(scenario, id) {
        format!("{}/estado.yml", task_folder(id))
    } else {
        task_path(id)
    }
}

/// Arquivo da tarefa que contém `text`: o arquivo único ou o arquivo da pasta com o trecho.
fn task_file_with(scenario: &Scenario, id: &str, text: &str) -> String {
    if !in_folder(scenario, id) {
        return task_path(id);
    }
    ["execucao.md", "verificacao.md", "criterios.md"]
        .iter()
        .map(|name| format!("{}/{name}", task_folder(id)))
        .find(|path| scenario.read(path).contains(text))
        .unwrap_or_else(|| panic!("'{text}' não está nos arquivos de {id}"))
}

/// Divide o texto da tarefa nos arquivos da pasta: o front-matter no estado e cada seção no seu
/// arquivo; o card fica de fora.
fn folder_files(text: &str) -> Vec<(&'static str, String)> {
    let between = |start: &str, end: &str| -> String {
        let from = text.find(start).unwrap();
        let to = end_of(text, from, end);
        format!("{}\n", text[from..to].trim_matches('\n'))
    };
    let state = text
        .strip_prefix("---\n")
        .and_then(|rest| rest.split_once("\n---\n"))
        .map(|(state, _)| format!("{state}\n"))
        .unwrap();
    vec![
        ("estado.yml", state),
        ("execucao.md", between("### Ler", "### Baseline")),
        (
            "verificacao.md",
            between("### Baseline", "### Concluída quando"),
        ),
        (
            "criterios.md",
            between("### Concluída quando", "## Descrição"),
        ),
        (
            "executivo.md",
            format!(
                "## Plano\n\n{}",
                between_lines(text, "### Resumo executivo", "### Detalhes técnicos")
            ),
        ),
        (
            "tecnico.md",
            format!(
                "## Plano\n\n```text\n## não é título\n```\n\n{}",
                between_lines(text, "### Detalhes técnicos", "### Card")
            ),
        ),
        ("registro.md", between("## Registro de execução", "\0")),
    ]
}

/// Fim do trecho que começa em `from`: o início de `end`, ou o fim do texto.
fn end_of(text: &str, from: usize, end: &str) -> usize {
    text[from..]
        .find(end)
        .map_or(text.len(), |offset| from + offset)
}

/// Texto entre os dois títulos, sem eles.
fn between_lines(text: &str, start: &str, end: &str) -> String {
    let from = text.find(start).unwrap() + start.len();
    let to = end_of(text, from, end);
    format!("{}\n", text[from..to].trim_matches('\n'))
}

fn write_task(scenario: &mut Scenario, source: &TaskSource) {
    let TaskSource {
        id,
        status,
        depends_on,
        baseline,
        verification,
        evidence,
    } = source;
    let text = format!(
        "---\nid: {id}\nspec: {SPEC}\ntitle: \"Tarefa {id}\"\nprofile: mecanica\npersona: desenvolvedor\ndepends_on: {depends_on}\nwave: 1\ncovers: [RF-01]\ntype: tarefa\nlabels: [{SPEC}, mecanica]\nstatus: {status}\ncontent_hash: null\nadjustments: 0\nadjusted_at: null\napproved_at: null\napproved_hash: null\nattempts: 0\ntests_result: null\nregression_result: null\nevidence_result: null\n---\n\n# {id}: Tarefa {id}\n\n## Execução\n\n### Ler\n\n- `README.md`\n\n### Mudar\n\n- `feito.txt` (criar): arquivo novo.\n\n### Passos\n\n1. Crie o arquivo.\n\n### Baseline\n\n{baseline}\n\n### Verificação\n\n```bash\n{verification}\n```\n\n### Evidências\n\n{evidence}\n\n### Concluída quando\n\n- [ ] Pronto (RF-01).\n\n  > Como comprovar: ver o arquivo.\n\n## Descrição\n\n### Resumo executivo\n\nResumo.\n\n### Detalhes técnicos\n\nDetalhes.\n\n### Card\n\n- **Título:** Tarefa {id}\n- **Tipo:** tarefa\n- **Descrição:** Descrição.\n- **Critérios de aceite:**\n  - [ ] Pronto.\n- **Rótulos:** {SPEC}, mecanica\n- **Estimativa:** mecanica\n- **Bloqueada por:** -\n\n## Registro de execução\n\n| # | Data | Etapa | Resultado | Detalhe |\n|---|---|---|---|---|\n\n## Histórico de ajustes\n\n| # | Data | Origem | Resumo |\n|---|---|---|---|\n"
    );
    if scenario.format == Format::Folder {
        for (name, content) in folder_files(&text) {
            scenario.write(&format!("{}/{name}", task_folder(id)), &content);
        }
    } else {
        scenario.write(&task_path(id), &text);
    }
}

/// Valor de um campo do cabeçalho da tarefa.
fn state_of(scenario: &Scenario, id: &str, field: &str) -> String {
    scenario
        .read(&state_path(scenario, id))
        .lines()
        .find_map(|line| line.strip_prefix(&format!("{field}: ")))
        .unwrap_or("(ausente)")
        .to_string()
}

/// Linhas da tabela "Registro de execução", sem o cabeçalho.
fn record_rows(scenario: &Scenario, id: &str) -> Vec<String> {
    let text = if in_folder(scenario, id) {
        scenario.read(&format!("{}/registro.md", task_folder(id)))
    } else {
        scenario.read(&task_path(id))
    };
    let section = text.split("## Registro de execução").nth(1).unwrap_or("");
    section
        .lines()
        .skip_while(|line| !line.starts_with("|---"))
        .skip(1)
        .take_while(|line| line.starts_with('|'))
        .map(str::to_string)
        .collect()
}

/// Pasta das evidências da tarefa, relativa à raiz do projeto.
fn evidence_root(id: &str) -> String {
    format!("vostok/work/tasks/{SPEC}/evidence/{id}")
}

/// Pasta das evidências da tarefa em pasta.
fn folder_evidence_root(id: &str) -> String {
    format!("{}/evidencias", task_folder(id))
}

// --- Executor de cenário ---

struct Scenario {
    name: &'static str,
    format: Format,
    /// A spec ainda espera a conversão (variante migrada).
    pending_migration: bool,
    /// O cenário marca o ponto da conversão com `migrate_here`.
    deferred_migration: bool,
    project: Project,
    transcript: String,
    tasks: Vec<String>,
}

impl Scenario {
    fn new(name: &'static str, format: Format) -> Self {
        let mut scenario = Self {
            name,
            format,
            pending_migration: format == Format::Migrated,
            deferred_migration: false,
            project: Project::new(),
            transcript: String::new(),
            tasks: Vec::new(),
        };
        scenario.write(
            "vostok/work/specs/refined/SPEC-001-exemplo.md",
            "---\nid: SPEC-001\ntitle: Exemplo\nstatus: approved\n---\n\n# Exemplo\n",
        );
        scenario
    }

    fn write(&mut self, relative: &str, content: &str) {
        let path = self.project.root().join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }

    fn read(&self, relative: &str) -> String {
        fs::read_to_string(self.project.root().join(relative)).unwrap_or_default()
    }

    /// Troca `from` por `to` no arquivo; falha se `from` não existir, para o cenário não mentir.
    fn replace(&mut self, relative: &str, from: &str, to: &str) {
        let text = self.read(relative);
        assert!(text.contains(from), "'{from}' não está em {relative}");
        self.write(relative, &text.replacen(from, to, 1));
    }

    /// Cria a tarefa e a acompanha no `## estado` final.
    fn add_task(&mut self, source: &TaskSource) {
        write_task(self, source);
        self.tasks.push(source.id.to_string());
    }

    /// Na variante migrada, a conversão acontece no ponto marcado, e não antes do primeiro comando.
    fn defer_migration(&mut self) {
        self.deferred_migration = true;
    }

    /// Converte a spec para pasta, fora da transcrição. Sem efeito nas outras variantes.
    fn migrate_here(&mut self) {
        if !self.pending_migration {
            return;
        }
        self.pending_migration = false;
        let output = self
            .project
            .vostok()
            .args(["task", "migrate", SPEC])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "migração falhou: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    /// Troca `from` por `to` no arquivo da tarefa que tem o trecho.
    fn edit_task(&mut self, id: &str, from: &str, to: &str) {
        let path = task_file_with(self, id, from);
        self.replace(&path, from, to);
    }

    /// Roda `vostok <args>` e acrescenta o comando, o código de saída e as duas saídas.
    fn run(&mut self, args: &[&str]) {
        if !self.deferred_migration {
            self.migrate_here();
        }
        let output = self.project.vostok().args(args).output().unwrap();
        self.transcript
            .push_str(&format!("$ vostok {}\n", args.join(" ")));
        self.transcript
            .push_str(&format!("exit: {}\n", output.status.code().unwrap_or(-1)));
        self.section("stdout", &String::from_utf8_lossy(&output.stdout));
        self.section("stderr", &String::from_utf8_lossy(&output.stderr));
        self.transcript.push('\n');
    }

    fn section(&mut self, label: &str, text: &str) {
        if !text.is_empty() {
            self.transcript.push_str(&format!("--- {label}\n{text}"));
            if !text.ends_with('\n') {
                self.transcript.push('\n');
            }
        }
    }

    /// `vostok task <comando> SPEC-001 <id> [extra...]`.
    fn task(&mut self, command: &str, id: &str, extra: &[&str]) {
        let mut args = vec!["task", command, SPEC, id];
        args.extend_from_slice(extra);
        self.run(&args);
        if command == "start" {
            self.enrich_documents(id);
        }
    }

    /// Na tarefa em pasta, enriquece o executivo e o técnico depois do início, como a
    /// `finish-task` faz; fora da transcrição.
    fn enrich_documents(&mut self, id: &str) {
        if !in_folder(self, id) {
            return;
        }
        for name in ["executivo.md", "tecnico.md"] {
            let path = format!("{}/{name}", task_folder(id));
            let text = self.read(&path);
            self.write(&path, &format!("{text}\n## Feito\n\nEnriquecido.\n"));
        }
    }

    /// `vostok task <grupo> <comando> SPEC-001 <id>`.
    fn nested(&mut self, group: &str, command: &str, id: &str) {
        self.run(&["task", group, command, SPEC, id]);
    }

    /// Registra a edição manual e aprova a tarefa.
    fn approve(&mut self, id: &str) {
        self.nested("adjust", "record-manual", id);
        self.nested("approve", "approve", id);
    }

    fn mark_done_criteria(&mut self, id: &str) {
        self.edit_task(id, "- [ ] Pronto (RF-01).", "- [x] Pronto (RF-01).");
    }

    fn write_state(&mut self) {
        for id in self.tasks.clone() {
            let state = self.state_block(&id);
            self.transcript.push_str(&state);
        }
    }

    fn state_block(&self, id: &str) -> String {
        let approved = state_of(self, id, "approved_hash") != "null";
        let mut block = format!("## estado {id}\n");
        for field in [
            "status",
            "attempts",
            "tests_result",
            "regression_result",
            "evidence_result",
        ] {
            block.push_str(&format!("{field}: {}\n", state_of(self, id, field)));
        }
        block.push_str(&format!("approved_hash preenchido: {approved}\n\n"));
        block.push_str(&format!("## registro {id}\n"));
        for row in record_rows(self, id) {
            block.push_str(&format!("{row}\n"));
        }
        block.push_str(&format!("\n## evidências {id}\n"));
        for file in self.evidence_files(id) {
            block.push_str(&format!("{file}\n"));
        }
        block.push('\n');
        block
    }

    fn evidence_files(&self, id: &str) -> Vec<String> {
        let relative = if in_folder(self, id) {
            folder_evidence_root(id)
        } else {
            evidence_root(id)
        };
        let root = self.project.root().join(relative);
        let mut files = Vec::new();
        collect_files(&root, &root, &mut files);
        files.sort();
        files
    }

    /// Fecha o cenário: acrescenta o estado final e grava ou compara a fixture.
    fn finish(mut self) {
        self.write_state();
        let transcript = normalize(&self.transcript, self.project.root());
        let path = fixture_path(self.name);
        if std::env::var(RECORD_VARIABLE).as_deref() == Ok("1") {
            if self.format == Format::Single {
                fs::create_dir_all(path.parent().unwrap()).unwrap();
                fs::write(&path, transcript).unwrap();
            }
            return;
        }
        let expected = fs::read_to_string(&path).unwrap_or_else(|_| {
            panic!(
                "fixture ausente: {}; rode com {RECORD_VARIABLE}=1",
                path.display()
            )
        });
        assert!(
            expected == transcript,
            "cenário {} ({:?}) difere da fixture:\n{}",
            self.name,
            self.format,
            differing_lines(&expected, &transcript)
        );
    }
}

fn collect_files(base: &std::path::Path, dir: &std::path::Path, files: &mut Vec<String>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_files(base, &path, files);
        } else {
            files.push(
                path.strip_prefix(base)
                    .unwrap()
                    .to_string_lossy()
                    .into_owned(),
            );
        }
    }
}

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/scenarios")
        .join(format!("{name}.txt"))
}

fn differing_lines(expected: &str, actual: &str) -> String {
    let expected: Vec<&str> = expected.lines().collect();
    let actual: Vec<&str> = actual.lines().collect();
    let mut report = String::new();
    for index in 0..expected.len().max(actual.len()) {
        let (left, right) = (expected.get(index), actual.get(index));
        if left != right {
            report.push_str(&format!(
                "linha {}:\n  fixture: {}\n  obtido:  {}\n",
                index + 1,
                left.unwrap_or(&"(fim)"),
                right.unwrap_or(&"(fim)")
            ));
        }
    }
    report
}

// --- Normalização ---

fn normalize(text: &str, root: &std::path::Path) -> String {
    let mut text = text.to_string();
    for number in 1..=9 {
        let id = format!("TASK-0{number}");
        let root_text = root.to_str().unwrap();
        text = text
            .replace(&format!("{root_text}/{}", task_path(&id)), "<TAREFA>")
            .replace(&task_path(&id), "<TAREFA>")
            .replace(
                &format!("{root_text}/{}", evidence_root(&id)),
                "<EVIDENCIAS>",
            )
            .replace(&evidence_root(&id), "<EVIDENCIAS>")
            .replace(
                &format!("{root_text}/{}", folder_evidence_root(&id)),
                "<EVIDENCIAS>",
            )
            .replace(&folder_evidence_root(&id), "<EVIDENCIAS>")
            .replace(&format!("{id}/evidencias/"), &format!("evidence/{id}/"))
            .replace(&format!("{root_text}/{}", task_folder(&id)), "<TAREFA>")
            .replace(&task_folder(&id), "<TAREFA>");
    }
    let text = text.replace(root.to_str().unwrap(), "<ROOT>");
    let text = drop_edited_files(&text);
    let text = drop_migration_rows(&text);
    let text = drop_sources_documents(&text);
    replace_commit_header(&replace_hashes(&replace_dates(&text)))
}

/// RF-13: no formato de pasta, o `finalize sources` traz o executivo e o técnico de cada tarefa.
fn drop_sources_documents(text: &str) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some(start) = rest.find("**Resumo executivo**\n") {
        let end = rest[start..]
            .find("**Registro de execução**")
            .map_or(rest.len(), |offset| start + offset);
        out.push_str(&rest[..start]);
        rest = &rest[end..];
    }
    out.push_str(rest);
    out
}

/// RF-12: a spec migrada ganha no registro a linha da migração; ela sai e as linhas seguintes
/// voltam ao número que teriam sem ela.
fn drop_migration_rows(text: &str) -> String {
    let mut shift = 0;
    let mut lines = Vec::new();
    for line in text.lines() {
        if line.starts_with("| # ") || line.starts_with("## ") {
            shift = 0;
        }
        if line.contains("| migração | pasta | formato antigo convertido |") {
            shift += 1;
            continue;
        }
        let renumbered = line
            .strip_prefix("| ")
            .and_then(|rest| rest.split_once(" |"))
            .and_then(|(number, rest)| Some((number.parse::<usize>().ok()?, rest)))
            .filter(|_| shift > 0)
            .map(|(number, rest)| format!("| {} |{rest}", number - shift));
        lines.push(renumbered.unwrap_or_else(|| line.to_string()));
    }
    format!("{}\n", lines.join("\n"))
}

/// RF-07: no formato de pasta, a edição manual cita os arquivos editados
/// (`Edição manual: execucao.md`); a citação sai para comparar com o formato atual.
fn drop_edited_files(text: &str) -> String {
    let lines: Vec<String> = text
        .lines()
        .map(|line| {
            let Some(start) = line.find("Edição manual: ") else {
                return line.to_string();
            };
            let cited = start + "Edição manual".len();
            let end = line[cited..]
                .find(" |")
                .map_or(line.len(), |offset| cited + offset);
            format!("{}{}", &line[..cited], &line[end..])
        })
        .collect();
    format!("{}\n", lines.join("\n"))
}

/// Datas `AAAA-MM-DD` e horas ISO `AAAA-MM-DDTHH:MM:SSZ` viram `<DATA>`.
fn replace_dates(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::new();
    let mut index = 0;
    while index < chars.len() {
        if let Some(length) = date_length(&chars[index..]) {
            out.push_str("<DATA>");
            index += length;
        } else {
            out.push(chars[index]);
            index += 1;
        }
    }
    out
}

fn date_length(chars: &[char]) -> Option<usize> {
    const DATE: &str = "dddd-dd-dd";
    const TIME: &str = "Tdd:dd:ddZ";
    let matches = |pattern: &str, offset: usize| {
        pattern.chars().enumerate().all(|(position, kind)| {
            chars.get(offset + position).is_some_and(|c| match kind {
                'd' => c.is_ascii_digit(),
                other => *c == other || (other == '-' && *c == '-'),
            })
        })
    };
    if !matches(DATE, 0) {
        return None;
    }
    Some(if matches(TIME, DATE.len()) {
        DATE.len() + TIME.len()
    } else {
        DATE.len()
    })
}

/// Sequências de 64 hexadecimais viram `<HASH>`.
fn replace_hashes(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::new();
    let mut index = 0;
    while index < chars.len() {
        let run = chars[index..]
            .iter()
            .take_while(|c| c.is_ascii_hexdigit())
            .count();
        if run >= 64 {
            out.push_str("<HASH>");
            index += run;
        } else if run > 0 {
            out.extend(&chars[index..index + run]);
            index += run;
        } else {
            out.push(chars[index]);
            index += 1;
        }
    }
    out
}

/// A linha `# commit: <hash> [...]` do cabeçalho dos logs vira `# commit: <COMMIT>`.
fn replace_commit_header(text: &str) -> String {
    let lines: Vec<String> = text
        .lines()
        .map(|line| {
            if line.starts_with("# commit: ") && line != "# commit: -" {
                "# commit: <COMMIT>".to_string()
            } else {
                line.to_string()
            }
        })
        .collect();
    format!("{}\n", lines.join("\n"))
}

// --- Cenários ---

fn passing_task<'a>(id: &'a str, verification: &'a str) -> TaskSource<'a> {
    let mut source = TaskSource::new(id);
    source.verification = verification;
    source
}

#[test]
fn scenario_full_flow() {
    for format in FORMATS {
        let mut s = Scenario::new("full-flow", format);
        s.add_task(&TaskSource {
            evidence: AUTOMATIC_EVIDENCE,
            verification: "echo verificado",
            ..TaskSource::new("TASK-01")
        });
        s.add_task(&TaskSource {
            depends_on: "[TASK-01]",
            verification: "echo verificado de novo",
            ..TaskSource::new("TASK-02")
        });
        s.approve("TASK-01");
        s.approve("TASK-02");
        for id in ["TASK-01", "TASK-02"] {
            s.task("start", id, &[]);
            s.mark_done_criteria(id);
            s.write("leitura.txt", "ok\n");
            s.task("tests", id, &[]);
            s.task("regression", id, &[]);
            s.task("evidence", id, &[]);
            s.task("finish", id, &[]);
        }
        s.run(&["finalize", "sources", SPEC]);
        s.finish();
    }
}

#[test]
fn scenario_failing_tests_block_and_resume() {
    for format in FORMATS {
        let mut s = Scenario::new("failing-tests-block-and-resume", format);
        s.add_task(&passing_task("TASK-01", "test -f feito.txt"));
        s.approve("TASK-01");
        s.task("start", "TASK-01", &[]);
        s.mark_done_criteria("TASK-01");
        s.task("tests", "TASK-01", &[]);
        s.task("tests", "TASK-01", &[]);
        s.task("tests", "TASK-01", &[]);
        s.write("feito.txt", "ok\n");
        s.task("resume", "TASK-01", &["criei o arquivo que faltava"]);
        s.task("tests", "TASK-01", &[]);
        s.task("regression", "TASK-01", &[]);
        s.task("evidence", "TASK-01", &[]);
        s.task("finish", "TASK-01", &[]);
        s.finish();
    }
}

#[test]
fn scenario_regression_block_and_accept() {
    for format in FORMATS {
        let mut s = Scenario::new("regression-block-and-accept", format);
        s.defer_migration();
        s.write("valor.txt", "a\n");
        s.add_task(&TaskSource {
            baseline: "- `cat valor.txt`",
            ..passing_task("TASK-01", "true")
        });
        s.approve("TASK-01");
        s.task("start", "TASK-01", &[]);
        s.migrate_here();
        s.mark_done_criteria("TASK-01");
        s.write("valor.txt", "b\n");
        s.task("tests", "TASK-01", &[]);
        s.task("regression", "TASK-01", &[]);
        s.task(
            "resume",
            "TASK-01",
            &["--accept-regression", "1", "diferença esperada"],
        );
        s.task("tests", "TASK-01", &[]);
        s.task("regression", "TASK-01", &[]);
        s.task("evidence", "TASK-01", &[]);
        s.task("finish", "TASK-01", &[]);
        s.finish();
    }
}

#[test]
fn scenario_regression_growth_passes() {
    for format in FORMATS {
        let mut s = Scenario::new("regression-growth-passes", format);
        s.write("lista.txt", "um\n");
        s.add_task(&TaskSource {
            baseline: "- `cat lista.txt`",
            ..passing_task("TASK-01", "true")
        });
        s.approve("TASK-01");
        s.task("start", "TASK-01", &[]);
        s.mark_done_criteria("TASK-01");
        s.write("lista.txt", "um\ndois\n");
        s.task("tests", "TASK-01", &[]);
        s.task("regression", "TASK-01", &[]);
        s.task("evidence", "TASK-01", &[]);
        s.task("finish", "TASK-01", &[]);
        s.finish();
    }
}

#[test]
fn scenario_failing_evidence() {
    for format in FORMATS {
        let mut s = Scenario::new("failing-evidence", format);
        s.add_task(&TaskSource {
            evidence: EXPECTING_EVIDENCE,
            ..passing_task("TASK-01", "true")
        });
        s.approve("TASK-01");
        s.task("start", "TASK-01", &[]);
        s.mark_done_criteria("TASK-01");
        s.write("leitura.txt", "outro\n");
        s.task("tests", "TASK-01", &[]);
        s.task("regression", "TASK-01", &[]);
        s.task("evidence", "TASK-01", &[]);
        s.task("finish", "TASK-01", &[]);
        s.finish();
    }
}

#[test]
fn scenario_requires_and_timeout() {
    for format in FORMATS {
        let mut s = Scenario::new("requires-and-timeout", format);
        let evidence = "- `esperada.txt` (automática): `test -s feito.txt && echo pronto`\n  - Prova: o arquivo novo é lido.\n  - Trecho: o conteúdo.\n  - Falha: arquivo ausente.\n  - Requer: `feito.txt`\n- `lenta.txt` (automática): `echo começo; sleep 5; echo fim`\n  - Prova: o comando roda.\n  - Trecho: começo.\n  - Falha: vazio.\n  - Tempo-limite: 1";
        s.add_task(&TaskSource {
            evidence,
            ..passing_task("TASK-01", "true")
        });
        s.approve("TASK-01");
        s.task("start", "TASK-01", &[]);
        s.mark_done_criteria("TASK-01");
        s.write("feito.txt", "pronto\n");
        s.task("tests", "TASK-01", &[]);
        s.task("regression", "TASK-01", &[]);
        s.task("evidence", "TASK-01", &[]);
        s.task("finish", "TASK-01", &[]);
        s.finish();
    }
}

#[test]
fn scenario_start_refused() {
    for format in FORMATS {
        let mut s = Scenario::new("start-refused", format);
        s.add_task(&passing_task("TASK-01", "true"));
        s.add_task(&TaskSource {
            depends_on: "[TASK-01]",
            ..passing_task("TASK-02", "true")
        });
        s.approve("TASK-02");
        s.task("start", "TASK-02", &[]);
        s.approve("TASK-01");
        s.edit_task("TASK-01", "1. Crie o arquivo.", "1. Crie outro arquivo.");
        s.task("start", "TASK-01", &[]);
        s.finish();
    }
}

#[test]
fn scenario_finish_refused() {
    for format in FORMATS {
        let mut s = Scenario::new("finish-refused", format);
        s.add_task(&passing_task("TASK-01", "true"));
        s.approve("TASK-01");
        s.task("start", "TASK-01", &[]);
        s.task("finish", "TASK-01", &[]);
        s.task("tests", "TASK-01", &[]);
        s.task("regression", "TASK-01", &[]);
        s.task("evidence", "TASK-01", &[]);
        s.task("finish", "TASK-01", &[]);
        s.mark_done_criteria("TASK-01");
        s.task("finish", "TASK-01", &[]);
        s.finish();
    }
}

#[test]
fn scenario_manual_edit_and_adjust() {
    for format in FORMATS {
        let mut s = Scenario::new("manual-edit-and-adjust", format);
        s.add_task(&passing_task("TASK-01", "true"));
        s.approve("TASK-01");
        s.edit_task(
            "TASK-01",
            "1. Crie o arquivo.",
            "1. Crie o arquivo com calma.",
        );
        s.nested("adjust", "validate", "TASK-01");
        s.task("start", "TASK-01", &[]);
        s.nested("adjust", "record-manual", "TASK-01");
        s.task("start", "TASK-01", &[]);
        s.nested("approve", "approve", "TASK-01");
        s.task("start", "TASK-01", &[]);
        s.edit_task(
            "TASK-01",
            "1. Crie o arquivo com calma.",
            "1. Crie o arquivo e confira.",
        );
        s.nested("adjust", "record-manual", "TASK-01");
        s.run(&[
            "task",
            "adjust",
            "record",
            SPEC,
            "TASK-01",
            "passo ajustado durante a execução",
        ]);
        s.finish();
    }
}

#[test]
fn scenario_tests_without_baseline() {
    for format in FORMATS {
        let mut s = Scenario::new("tests-without-baseline", format);
        s.add_task(&TaskSource {
            status: "in_progress",
            ..passing_task("TASK-01", "true")
        });
        s.task("tests", "TASK-01", &[]);
        s.task("baseline", "TASK-01", &[]);
        s.task("tests", "TASK-01", &[]);
        s.finish();
    }
}
