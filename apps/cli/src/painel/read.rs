//! Leitura dos arquivos de `vostok/` para o painel: visão, specs, planos, tarefas, documentos e
//! configurações das evidências.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Component, Path, PathBuf};

use super::{
    json, json_string, live, not_found, percent_decode, query_value, safe_segment, text, Painel,
    Reply,
};
use crate::commands::plan::{plan_outdated, waves};
use crate::commands::task::{evidence_review_states, ReviewState};
use crate::harness::Harness;
use crate::models;
use crate::settings::{read_evidence_settings, SettingsSnapshot};
use crate::support::{config_get, frontmatter_get, sha256_hex, yaml_unescape};

/// Arquivos da tarefa em pasta mostrados no painel, na ordem das abas.
const TASK_FILES: &[&str] = &[
    "execucao",
    "verificacao",
    "criterios",
    "executivo",
    "tecnico",
    "registro",
];

/// Os únicos HTML que abrem na tela, e sem a origem do painel (RF-56).
const FINAL_DOCUMENTS: &[&str] = &["executivo.html", "tecnico.html", "evidencias.html"];
const FINAL_DOCUMENT_POLICY: &str = "sandbox allow-scripts allow-downloads";

/// Campos do front-matter da spec que o painel mostra.
const SPEC_FIELDS: &[&str] = &[
    "id",
    "title",
    "status",
    "type",
    "author",
    "created_at",
    "updated_at",
    "refined_at",
    "approved_at",
    "adjustments",
];

/// Quantas linhas do registro de execução entram na atividade recente da visão.
const ACTIVITY_LIMIT: usize = 20;

/// Cores padrão do tema (CB-05), usadas quando `config/documents.yml` não tem os grupos.
const DEFAULT_LIGHT: &[(&str, &str)] = &[
    ("color_bg", "#f7f5fb"),
    ("color_surface", "#ffffff"),
    ("color_ink", "#1d1729"),
    ("color_muted", "#6a6280"),
    ("color_line", "#e3ddee"),
    ("color_primary", "#6d28d9"),
    ("color_primary_ink", "#ffffff"),
];
const DEFAULT_DARK: &[(&str, &str)] = &[
    ("color_bg", "#1a1528"),
    ("color_surface", "#231d35"),
    ("color_ink", "#ece8f5"),
    ("color_muted", "#a39cb8"),
    ("color_line", "#3a3252"),
    ("color_primary", "#a78bfa"),
    ("color_primary_ink", "#150f22"),
];

struct SpecFile {
    id: String,
    status: String,
    file: PathBuf,
}

/// As pastas de `work/` resolvidas pela configuração do harness.
/// Como um arquivo de `vostok/` sai pela rota: onde está, o tipo e os cabeçalhos.
pub struct FileDelivery {
    pub path: PathBuf,
    pub content_type: &'static str,
    pub headers: Vec<(&'static str, String)>,
}

struct Layout<'a> {
    painel: &'a Painel,
    spec_dirs: Vec<PathBuf>,
    plans: PathBuf,
    tasks: PathBuf,
    archive: PathBuf,
}

impl<'a> Layout<'a> {
    fn new(painel: &'a Painel) -> Self {
        let harness = Harness {
            root: painel.root.clone(),
        };
        let specs = harness
            .path("specs")
            .unwrap_or_else(|_| painel.work.join("specs"));
        let config = harness.config_file("specs.yml");
        let spec_dirs = [("draft_dir", "draft"), ("refined_dir", "refined")]
            .iter()
            .map(|(key, fallback)| {
                specs.join(config_get(&config, key).unwrap_or_else(|_| fallback.to_string()))
            })
            .collect();
        Self {
            painel,
            spec_dirs,
            plans: harness
                .path("plans")
                .unwrap_or_else(|_| painel.work.join("plans")),
            tasks: harness
                .path("tasks")
                .unwrap_or_else(|_| painel.work.join("tasks")),
            archive: harness
                .path("archive")
                .unwrap_or_else(|_| painel.work.join(".archive")),
        }
    }

    fn harness(&self) -> Harness {
        Harness {
            root: self.painel.root.clone(),
        }
    }

    /// Caminho relativo à pasta do projeto, como o frontend o pede de volta.
    fn relative(&self, path: &Path) -> String {
        path.strip_prefix(&self.painel.project)
            .unwrap_or(path)
            .to_string_lossy()
            .into_owned()
    }
}

/// Responde um `GET /api/...`; `parts` são os segmentos do caminho, a partir de `api`.
/// `None` quando a rota não existe, e o servidor responde 404.
pub fn route(painel: &Painel, parts: &[&str], query: &str) -> Option<Reply> {
    let layout = Layout::new(painel);
    let reply = match parts {
        ["api", "visao"] => Ok(json(layout.overview())),
        ["api", "versao"] => Ok(json(format!(
            "{{\"versao\":\"{}\"}}",
            live::version(&painel.work)
        ))),
        ["api", "marca"] => Ok(json(layout.brand())),
        ["api", "documentos"] => Ok(json(layout.documents())),
        ["api", "specs", id] => layout.spec(id),
        ["api", "planos", id] => layout.plan(id),
        ["api", "tarefas", spec, task] => layout.task(spec, task),
        ["api", "configuracoes", "evidencias"] => layout.evidence_settings(),
        ["api", "arquivo"] => layout.file(&query_value(query, "caminho")),
        ["api", "fonte"] => layout.source(&query_value(query, "caminho")),
        // Documentos HTML abertos por caminho, para que os links relativos (assets/) funcionem.
        ["arquivos", rest @ ..] => {
            let relative: Vec<String> = rest.iter().map(|part| percent_decode(part)).collect();
            layout.file(&relative.join("/"))
        }
        _ => return None,
    };
    Some(reply.unwrap_or_else(|reply| reply))
}

/// `{"arquivos": ..., "valores": ...}` das configurações das evidências, no GET e no 200 do POST.
pub fn settings_json(snapshot: &SettingsSnapshot) -> String {
    snapshot.to_json().to_string()
}

/// A decisão de `Layout::file` sem ler o arquivo, para o servidor entregá-lo por partes.
/// `None` quando a rota não é de arquivo.
pub fn file_delivery(
    painel: &Painel,
    parts: &[&str],
    query: &str,
) -> Option<Result<FileDelivery, Reply>> {
    let relative = match parts {
        ["api", "arquivo"] => query_value(query, "caminho"),
        ["arquivos", rest @ ..] => {
            let decoded: Vec<String> = rest.iter().map(|part| percent_decode(part)).collect();
            decoded.join("/")
        }
        _ => return None,
    };
    Some(Layout::new(painel).delivery(&relative).map_err(nosniff))
}

impl Layout<'_> {
    fn evidence_settings(&self) -> Result<Reply, Reply> {
        read_evidence_settings(&self.harness())
            .map(|snapshot| json(settings_json(&snapshot)))
            .map_err(|error| {
                text(
                    500,
                    &format!("não foi possível ler as configurações: {error}"),
                )
            })
    }

    /// Specs abertas com as tarefas, specs arquivadas e a atividade recente do registro de execução.
    fn overview(&self) -> String {
        let harness = self.harness();
        let open: Vec<String> = self
            .specs()
            .iter()
            .map(|spec| {
                let plan = self.plans.join(format!("{}.md", spec.id)).is_file();
                let plan_outdated = plan_outdated(&harness, &spec.id);
                format!(
                    "{{\"meta\":{},\"plan\":{plan},\"plan_outdated\":{plan_outdated},\"tasks\":[{}]}}",
                    frontmatter_json(&spec.file, SPEC_FIELDS),
                    self.task_summaries(&spec.id).join(",")
                )
            })
            .collect();
        let archived: Vec<String> = self
            .archived()
            .iter()
            .map(|(dir, file)| {
                let tasks = task_dirs(&dir.join("tasks")).len();
                format!(
                    "{{\"meta\":{},\"tasks_total\":{tasks},\"archived\":true}}",
                    frontmatter_json(file, SPEC_FIELDS)
                )
            })
            .collect();
        let project = self
            .painel
            .project
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default();
        format!(
            "{{\"project\":{},\"specs\":[{}],\"archived\":[{}],\"activity\":[{}]}}",
            json_string(&project),
            open.join(","),
            archived.join(","),
            self.activity().join(",")
        )
    }

    fn specs(&self) -> Vec<SpecFile> {
        let mut entries: Vec<SpecFile> = self
            .spec_dirs
            .iter()
            .flat_map(|dir| markdown_files(dir))
            .filter_map(|file| {
                let id = frontmatter_get(&file, "id").ok()?;
                let status = frontmatter_get(&file, "status").unwrap_or_default();
                Some(SpecFile { id, status, file })
            })
            .collect();
        // O rascunho e a refinada têm o mesmo ID: fica a refinada, que vem depois na ordem das pastas.
        entries.reverse();
        entries.sort_by(|a, b| a.id.cmp(&b.id));
        entries.dedup_by(|a, b| a.id == b.id);
        entries.retain(|spec| !spec.status.is_empty());
        entries
    }

    /// Specs arquivadas: `.archive/<SPEC>/spec.md`, da mais nova para a mais antiga.
    fn archived(&self) -> Vec<(PathBuf, PathBuf)> {
        let Ok(entries) = fs::read_dir(&self.archive) else {
            return Vec::new();
        };
        let mut dirs: Vec<(PathBuf, PathBuf)> = entries
            .flatten()
            .map(|entry| entry.path())
            .filter_map(|dir| {
                let file = dir.join("spec.md");
                file.is_file().then_some((dir, file))
            })
            .collect();
        dirs.sort();
        dirs.reverse();
        dirs
    }

    fn task_summaries(&self, spec: &str) -> Vec<String> {
        let tasks_dir = self.tasks.join(spec);
        let waves = computed_waves(&tasks_dir);
        let harness = self.harness();
        task_dirs(&tasks_dir)
            .iter()
            .map(|dir| task_summary(dir, &harness, &waves))
            .collect()
    }

    /// Últimas linhas do registro de execução das tarefas abertas, da mais nova para a mais antiga.
    fn activity(&self) -> Vec<String> {
        let mut rows = self.task_rows();
        sort_newest_first(&mut rows);
        rows.iter()
            .take(ACTIVITY_LIMIT)
            .map(ActivityRow::json)
            .collect()
    }

    fn task_rows(&self) -> Vec<ActivityRow> {
        let mut rows = Vec::new();
        for spec in self.specs() {
            rows.extend(task_rows_in(&self.tasks.join(&spec.id), &spec.id));
        }
        rows
    }

    /// Documentos finais das specs arquivadas (`.archive/<SPEC>/docs/*.html`).
    fn documents(&self) -> String {
        let items: Vec<String> = self
            .archived()
            .iter()
            .filter_map(|(dir, file)| {
                let docs: Vec<String> = ["executivo", "tecnico", "evidencias"]
                    .iter()
                    .map(|name| dir.join("docs").join(format!("{name}.html")))
                    .filter(|path| path.is_file())
                    .map(|path| {
                        let size = fs::metadata(&path).map(|meta| meta.len()).unwrap_or(0);
                        format!(
                            "{{\"name\":{},\"path\":{},\"size\":{size}}}",
                            json_string(&path.file_stem().unwrap_or_default().to_string_lossy()),
                            json_string(&self.relative(&path))
                        )
                    })
                    .collect();
                (!docs.is_empty()).then(|| {
                    format!(
                        "{{\"meta\":{},\"docs\":[{}]}}",
                        frontmatter_json(file, SPEC_FIELDS),
                        docs.join(",")
                    )
                })
            })
            .collect();
        format!("[{}]", items.join(","))
    }

    fn spec(&self, id: &str) -> Result<Reply, Reply> {
        let safe_id = safe_segment(id)?;
        let open = self
            .specs()
            .into_iter()
            .find(|spec| spec.id == safe_id)
            .map(|spec| spec.file);
        let archived = open.is_none();
        let file = open.unwrap_or_else(|| self.archive.join(safe_id).join("spec.md"));
        let markdown = fs::read_to_string(&file).map_err(|_| not_found())?;
        let plan = self.plans.join(format!("{safe_id}.md")).is_file();
        let plan_outdated = plan_outdated(&self.harness(), safe_id);
        let tasks_dir = if archived {
            self.archive.join(safe_id).join("tasks")
        } else {
            self.tasks.join(safe_id)
        };
        let waves = computed_waves(&tasks_dir);
        let harness = self.harness();
        let tasks: Vec<String> = task_dirs(&tasks_dir)
            .iter()
            .map(|dir| task_summary(dir, &harness, &waves))
            .collect();
        Ok(json(format!(
            "{{\"meta\":{},\"path\":{},\"plan\":{plan},\"plan_outdated\":{plan_outdated},\"archived\":{archived},\"markdown\":{},\"tasks\":[{}]}}",
            frontmatter_json(&file, SPEC_FIELDS),
            json_string(&self.relative(&file)),
            json_string(&strip_frontmatter(&markdown)),
            tasks.join(",")
        )))
    }

    fn plan(&self, id: &str) -> Result<Reply, Reply> {
        let file = self.plans.join(format!("{}.md", safe_segment(id)?));
        let markdown = fs::read_to_string(&file).map_err(|_| not_found())?;
        Ok(json(format!(
            "{{\"id\":{},\"path\":{},\"markdown\":{}}}",
            json_string(id),
            json_string(&self.relative(&file)),
            json_string(&strip_frontmatter(&markdown))
        )))
    }

    fn task(&self, spec: &str, task: &str) -> Result<Reply, Reply> {
        let spec = safe_segment(spec)?;
        let task = safe_segment(task)?;
        let open = self.tasks.join(spec).join(task);
        let archived = !open.is_dir();
        let dir = if archived {
            self.archive.join(spec).join("tasks").join(task)
        } else {
            open
        };
        if !dir.join("estado.yml").is_file() {
            return Err(not_found());
        }
        let files: Vec<String> = TASK_FILES
            .iter()
            .map(|name| {
                let content =
                    fs::read_to_string(dir.join(format!("{name}.md"))).unwrap_or_default();
                format!(
                    "{}:{}",
                    json_string(name),
                    json_string(&strip_html_comments(&content))
                )
            })
            .collect();
        let evidence: Vec<String> = evidence_files(&dir.join("evidencias"))
            .iter()
            .map(|file| {
                let size = fs::metadata(file).map(|meta| meta.len()).unwrap_or(0);
                format!(
                    "{{\"name\":{},\"path\":{},\"size\":{size}}}",
                    json_string(&file.file_name().unwrap_or_default().to_string_lossy()),
                    json_string(&self.relative(file))
                )
            })
            .collect();
        let verification = fs::read_to_string(dir.join("verificacao.md")).unwrap_or_default();
        let kinds = EvidenceKindNames::of(&self.harness());
        let baseline = estado_pairs(&dir.join("estado.yml"))
            .into_iter()
            .find(|(key, _)| key == "baseline_commit")
            .map(|(_, value)| registered(&value).to_string())
            .unwrap_or_default();
        let reviews: Vec<String> = evidence_review_states(&self.painel.root, &dir)
            .iter()
            .enumerate()
            .map(|(index, review)| {
                review_json(
                    index,
                    review,
                    &explanation(&verification, &kinds, &baseline, review),
                )
            })
            .collect();
        Ok(json(format!(
            "{{\"spec\":{},\"archived\":{archived},\"state\":{},\"files\":{{{}}},\"evidence\":[{}],\"reviews\":[{}]}}",
            json_string(spec),
            json_object(&with_task_facts(
                estado_pairs(&dir.join("estado.yml")),
                &dir,
                &self.harness(),
                &computed_waves(dir.parent().unwrap_or(dir.as_path()))
            )),
            files.join(","),
            evidence.join(","),
            reviews.join(",")
        )))
    }

    /// Um arquivo de dentro do harness (`vostok/`), para evidências e documentos.
    fn file(&self, relative: &str) -> Result<Reply, Reply> {
        let delivery = self.delivery(relative).map_err(nosniff)?;
        let body = fs::read(&delivery.path).map_err(|_| nosniff(not_found()))?;
        Ok(Reply {
            status: 200,
            content_type: delivery.content_type,
            body,
            headers: delivery.headers,
        })
    }

    /// Como o arquivo é entregue: baixar é o padrão, para que nenhum HTML ou SVG rode na origem
    /// do painel (RN-14).
    fn delivery(&self, relative: &str) -> Result<FileDelivery, Reply> {
        let path = Path::new(relative);
        let harness_dir = self
            .painel
            .root
            .strip_prefix(&self.painel.project)
            .unwrap_or(&self.painel.root);
        let inside = !relative.is_empty()
            && path
                .components()
                .all(|part| matches!(part, Component::Normal(_)))
            && path.starts_with(harness_dir);
        if !inside {
            return Err(text(400, "caminho fora de vostok/"));
        }
        let full = self.painel.project.join(path);
        let canonical = fs::canonicalize(&full).map_err(|_| not_found())?;
        if !canonical.starts_with(&self.painel.root) {
            return Err(text(400, "caminho fora de vostok/"));
        }
        let mut headers = vec![nosniff_header()];
        let content_type = if self.is_final_document(&canonical) {
            headers.push(("Content-Security-Policy", FINAL_DOCUMENT_POLICY.to_string()));
            "text/html; charset=utf-8"
        } else if let Some(viewable) = viewable_type(relative) {
            viewable
        } else {
            headers.push((
                "Content-Disposition",
                format!("attachment; filename=\"{}\"", clean_file_name(relative)),
            ));
            "application/octet-stream"
        };
        Ok(FileDelivery {
            path: canonical,
            content_type,
            headers,
        })
    }

    /// Reconhece pelo caminho canônico: um atalho `docs/executivo.html` que aponte para outro
    /// HTML sai da pasta de arquivo e não abre na tela.
    fn is_final_document(&self, canonical: &Path) -> bool {
        let Ok(archive) = fs::canonicalize(&self.archive) else {
            return false;
        };
        let Ok(inside) = canonical.strip_prefix(archive) else {
            return false;
        };
        let parts: Vec<_> = inside.iter().collect();
        matches!(parts[..], [_, docs, name]
            if docs == "docs" && FINAL_DOCUMENTS.iter().any(|document| name == *document))
    }

    /// Conteúdo e hash de um arquivo de `vostok/`, para abrir a edição com o controle otimista.
    fn source(&self, relative: &str) -> Result<Reply, Reply> {
        let reply = self.file(relative)?;
        let content = String::from_utf8_lossy(&reply.body).into_owned();
        Ok(json(format!(
            "{{\"conteudo\":{},\"hash\":{}}}",
            json_string(&content),
            json_string(&sha256_hex(&reply.body))
        )))
    }

    /// O grupo `brand`, o ícone embutido e as cores dos dois temas de `config/documents.yml`;
    /// sem os grupos, as cores padrão (CB-05).
    fn brand(&self) -> String {
        let config = self.painel.root.join("config/documents.yml");
        let content = fs::read_to_string(&config).unwrap_or_default();
        let group = |name: &str, defaults: &[(&str, &str)]| -> String {
            let mut pairs = yaml_group(&content, name);
            if pairs.is_empty() {
                pairs = defaults
                    .iter()
                    .map(|(key, value)| (key.to_string(), value.to_string()))
                    .collect();
            }
            json_object(&pairs)
        };
        let brand_pairs = yaml_group(&content, "brand");
        let brand = if brand_pairs.is_empty() {
            json_object(&[
                ("name".to_string(), String::new()),
                ("icon".to_string(), String::new()),
                ("primary".to_string(), String::new()),
                ("secondary".to_string(), String::new()),
            ])
        } else {
            json_object(&brand_pairs)
        };
        let icon = brand_pairs
            .iter()
            .find(|(key, _)| key == "icon")
            .map(|(_, value)| value.as_str())
            .unwrap_or("");
        format!(
            "{{\"brand\":{brand},\"icon_data\":{},\"light\":{},\"dark\":{}}}",
            json_string(&self.icon_data(icon)),
            group("light", DEFAULT_LIGHT),
            group("dark", DEFAULT_DARK)
        )
    }

    /// O ícone da marca como `data:` URI; vazio quando não há, está fora do projeto ou não abre.
    fn icon_data(&self, icon: &str) -> String {
        let path = Path::new(icon);
        let inside = !icon.is_empty()
            && path
                .components()
                .all(|part| matches!(part, Component::Normal(_)));
        if !inside {
            return String::new();
        }
        fs::read(self.painel.project.join(path))
            .map(|bytes| format!("data:{};base64,{}", icon_type(icon), base64(&bytes)))
            .unwrap_or_default()
    }
}

struct ActivityRow {
    date: String,
    kind: String,
    spec: String,
    task: String,
    target: String,
    number: usize,
    step: String,
    result: String,
    detail: String,
}

impl ActivityRow {
    fn sort_key(&self) -> (&str, &str, &str, &str, usize) {
        (
            &self.date,
            &self.kind,
            &self.spec,
            &self.target,
            self.number,
        )
    }

    fn json(&self) -> String {
        format!(
            "{{\"date\":{},\"type\":{},\"spec\":{},\"task\":{},\"target\":{},\"step\":{},\"result\":{},\"detail\":{}}}",
            json_string(&self.date),
            json_string(&self.kind),
            json_string(&self.spec),
            json_string(&self.task),
            json_string(&self.target),
            json_string(&self.step),
            json_string(&self.result),
            json_string(&self.detail)
        )
    }
}

fn sort_newest_first(rows: &mut [ActivityRow]) {
    rows.sort_by(|a, b| b.sort_key().cmp(&a.sort_key()));
}

/// As linhas do registro de cada tarefa de `<tasks_dir>`.
fn task_rows_in(tasks_dir: &Path, spec: &str) -> Vec<ActivityRow> {
    let mut rows = Vec::new();
    for dir in task_dirs(tasks_dir) {
        let task = dir.file_name().unwrap_or_default().to_string_lossy();
        let content = fs::read_to_string(dir.join("registro.md")).unwrap_or_default();
        rows.extend(activity_rows(&content, spec, &task));
    }
    rows
}

/// As linhas da tabela do registro de execução, sem o histórico de ajustes.
fn activity_rows(registro: &str, spec: &str, task: &str) -> Vec<ActivityRow> {
    let log = registro.split("## Histórico").next().unwrap_or("");
    log.lines()
        .filter(|line| line.starts_with("| ") && !line.starts_with("| #"))
        .filter_map(|line| {
            let cells: Vec<&str> = line.trim_matches('|').split('|').map(str::trim).collect();
            (cells.len() >= 5).then(|| ActivityRow {
                date: cells[1].to_string(),
                kind: "tarefa".to_string(),
                spec: spec.to_string(),
                task: task.to_string(),
                target: task.to_string(),
                number: cells[0].parse().unwrap_or(0),
                step: cells[2].to_string(),
                result: cells[3].to_string(),
                detail: cells[4].to_string(),
            })
        })
        .collect()
}

fn file_name_of(path: &Path) -> String {
    path.file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned()
}

fn json_object(pairs: &[(String, String)]) -> String {
    let items: Vec<String> = pairs
        .iter()
        .map(|(key, value)| format!("{}:{}", json_string(key), json_string(value)))
        .collect();
    format!("{{{}}}", items.join(","))
}

/// Campos escolhidos do front-matter de um arquivo, em JSON (texto, como no arquivo).
fn frontmatter_json(file: &Path, keys: &[&str]) -> String {
    let pairs: Vec<(String, String)> = keys
        .iter()
        .map(|key| {
            (
                key.to_string(),
                frontmatter_get(file, key).unwrap_or_default(),
            )
        })
        .collect();
    json_object(&pairs)
}

/// Um valor escalar de YAML sem as aspas, com o escape desfeito, e sem o comentário quando não tem aspas.
fn yaml_scalar(raw: &str) -> String {
    let value = raw.trim();
    let quoted_by =
        |quote: char| value.len() >= 2 && value.starts_with(quote) && value.ends_with(quote);
    if quoted_by('"') {
        yaml_unescape(&value[1..value.len() - 1])
    } else if quoted_by('\'') {
        value[1..value.len() - 1].replace("''", "'")
    } else {
        value.split(" #").next().unwrap_or("").trim().to_string()
    }
}

/// Pares `chave: valor` de primeiro nível de um YAML plano, como o `estado.yml`.
fn estado_pairs(file: &Path) -> Vec<(String, String)> {
    fs::read_to_string(file)
        .unwrap_or_default()
        .lines()
        .filter(|line| !line.starts_with([' ', '#']))
        .filter_map(|line| line.split_once(':'))
        .map(|(key, value)| (key.trim().to_string(), yaml_scalar(value)))
        .collect()
}

/// O valor registrado, ou vazio para `""`, `-` e `null`, como o `task status` trata o campo.
fn registered(value: &str) -> &str {
    match value {
        "-" | "null" => "",
        value => value,
    }
}

/// Onda de cada tarefa da pasta, calculada pelas dependências como no `task status`.
fn computed_waves(tasks_dir: &Path) -> BTreeMap<String, usize> {
    let graph: Vec<(String, Vec<String>)> = task_dirs(tasks_dir)
        .iter()
        .map(|dir| {
            let depends_on = estado_pairs(&dir.join("estado.yml"))
                .into_iter()
                .find(|(key, _)| key == "depends_on")
                .map(|(_, value)| value)
                .unwrap_or_default();
            let ids = depends_on
                .replace(['[', ']'], "")
                .split(',')
                .map(|id| id.trim().to_string())
                .filter(|id| !id.is_empty())
                .collect();
            (file_name_of(dir), ids)
        })
        .collect();
    waves(&graph)
}

/// Acrescenta ao estado a onda, o perfil e a persona, sem inventar valor onde não há.
fn with_task_facts(
    mut estado: Vec<(String, String)>,
    dir: &Path,
    harness: &Harness,
    waves: &BTreeMap<String, usize>,
) -> Vec<(String, String)> {
    let registered_of = |estado: &[(String, String)], key: &str| {
        estado
            .iter()
            .find(|(name, _)| name == key)
            .map(|(_, value)| registered(value).to_string())
            .unwrap_or_default()
    };
    let wave = match registered_of(&estado, "wave") {
        wave if !wave.is_empty() => wave,
        _ => waves
            .get(&file_name_of(dir))
            .map(|wave| wave.to_string())
            .unwrap_or_default(),
    };
    let profile = match models::task_profile(harness, dir).as_str() {
        "-" => String::new(),
        profile => profile.to_string(),
    };
    let persona = registered_of(&estado, "persona");
    for (key, value) in [("wave", wave), ("profile", profile), ("persona", persona)] {
        match estado.iter_mut().find(|(name, _)| name == key) {
            Some(pair) => pair.1 = value,
            None => estado.push((key.to_string(), value)),
        }
    }
    estado
}

fn task_summary(dir: &Path, harness: &Harness, waves: &BTreeMap<String, usize>) -> String {
    let estado = with_task_facts(estado_pairs(&dir.join("estado.yml")), dir, harness, waves);
    let keys = [
        "id",
        "title",
        "status",
        "wave",
        "profile",
        "persona",
        "model",
        "depends_on",
        "covers",
        "type",
        "description",
        "check_result",
        "waiting_on",
        "waiting_reason",
        "closes_after",
    ];
    let waiting_keys = ["waiting_on", "waiting_reason", "closes_after"];
    let pairs: Vec<(String, String)> = keys
        .iter()
        .map(|key| {
            let value = estado
                .iter()
                .find(|(name, _)| name == key)
                .map(|(_, value)| value.clone())
                .unwrap_or_default();
            let value = if waiting_keys.contains(key) {
                registered(&value).to_string()
            } else {
                value
            };
            (key.to_string(), value)
        })
        .collect();
    json_object(&pairs)
}

/// Pares `chave: valor` de um grupo de primeiro nível, com as aspas e os comentários removidos.
fn yaml_group(content: &str, name: &str) -> Vec<(String, String)> {
    let header = format!("{name}:");
    let mut lines = content.lines().skip_while(|line| line.trim_end() != header);
    lines.next();
    lines
        .take_while(|line| line.is_empty() || line.starts_with(' ') || line.starts_with('#'))
        .filter_map(|line| {
            let (key, value) = line.trim().split_once(':')?;
            if key.starts_with('#') {
                return None;
            }
            let value = value
                .split(" #")
                .next()
                .unwrap_or("")
                .trim()
                .trim_matches('"');
            Some((key.to_string(), value.to_string()))
        })
        .collect()
}

fn markdown_files(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut files: Vec<PathBuf> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "md"))
        .collect();
    files.sort();
    files
}

fn task_dirs(spec_dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(spec_dir) else {
        return Vec::new();
    };
    let mut dirs: Vec<PathBuf> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.join("estado.yml").is_file())
        .collect();
    dirs.sort();
    dirs
}

/// Evidências da tarefa: os arquivos soltos de `evidencias/`, sem as subpastas.
fn evidence_files(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut files: Vec<PathBuf> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .collect();
    files.sort();
    files
}

fn strip_frontmatter(markdown: &str) -> String {
    if let Some(rest) = markdown.strip_prefix("---\n") {
        if let Some(end) = rest.find("\n---\n") {
            return rest[end + 5..].trim_start().to_string();
        }
    }
    markdown.to_string()
}

fn strip_html_comments(markdown: &str) -> String {
    let mut out = String::with_capacity(markdown.len());
    let mut rest = markdown;
    while let Some(start) = rest.find("<!--") {
        out.push_str(&rest[..start]);
        rest = match rest[start..].find("-->") {
            Some(end) => &rest[start + end + 3..],
            None => "",
        };
    }
    out.push_str(rest);
    out.trim().to_string()
}

/// Tipo do que abre na tela, pela extensão; o resto só baixa.
fn viewable_type(name: &str) -> Option<&'static str> {
    let extension = Path::new(name).extension()?.to_str()?.to_lowercase();
    let content_type = match extension.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "gif" => "image/gif",
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        "mov" => "video/quicktime",
        "md" | "txt" | "yml" | "log" | "out" | "diff" | "csv" => "text/plain; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" => "text/javascript; charset=utf-8",
        _ => return None,
    };
    Some(content_type)
}

/// Só ASCII: o nome vai no cabeçalho, e aspas ou quebras de linha o quebrariam (CB-40).
fn clean_file_name(relative: &str) -> String {
    let clean: String = relative
        .rsplit('/')
        .next()
        .unwrap_or_default()
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | ' ' | '-') {
                character
            } else {
                '_'
            }
        })
        .collect();
    if clean.is_empty() {
        "arquivo".to_string()
    } else {
        clean
    }
}

fn nosniff_header() -> (&'static str, String) {
    ("X-Content-Type-Options", "nosniff".to_string())
}

fn nosniff(mut reply: Reply) -> Reply {
    reply.headers.push(nosniff_header());
    reply
}

/// O ícone da marca vai embutido em `data:` e não passa por `Layout::file`; por isso aceita SVG.
fn icon_type(name: &str) -> &'static str {
    match Path::new(name).extension().and_then(|ext| ext.to_str()) {
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("webp") => "image/webp",
        _ => "application/octet-stream",
    }
}

fn base64(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let n = (chunk[0] as u32) << 16
            | (*chunk.get(1).unwrap_or(&0) as u32) << 8
            | *chunk.get(2).unwrap_or(&0) as u32;
        for (position, shift) in [18, 12, 6, 0].iter().enumerate() {
            if position <= chunk.len() {
                out.push(TABLE[(n >> shift & 63) as usize] as char);
            } else {
                out.push('=');
            }
        }
    }
    out
}

/// Como a evidência foi feita e como reproduzi-la, derivados do tipo e do comando (RF-13), e a
/// frase do subitem `Como:` (RF-14).
#[derive(Default)]
struct Explanation {
    command: String,
    tool: String,
    reproduce: String,
    how: String,
}

const HOW_FIELD: &str = "Como";
const MANUAL_TOOL: &str = "Anexada pelo usuário";
const DIFF_TOOL: &str = "git diff";
const PRINT_TOOL: &str = "Playwright (print)";
const VIDEO_TOOL: &str = "Playwright (vídeo)";

/// Nomes dos tipos de evidência e o rótulo da Verificação, lidos do `config/tasks.yml`.
struct EvidenceKindNames {
    manual: String,
    diff: String,
    label: String,
}

impl EvidenceKindNames {
    fn of(harness: &Harness) -> Self {
        let config = harness.config_file("tasks.yml");
        let get = |key: &str, fallback: &str| {
            config_get(&config, key).unwrap_or_else(|_| fallback.to_string())
        };
        Self {
            manual: get("evidence_manual", "manual"),
            diff: get("evidence_diff", "diff"),
            label: get("verify_label", "# evidência:"),
        }
    }
}

/// Linhas do item da evidência `name` em "Evidências": a linha sem recuo e as recuadas abaixo dela.
fn evidence_lines<'t>(verification: &'t str, name: &str) -> Vec<&'t str> {
    let start = format!("- `{name}`");
    let mut lines = verification
        .lines()
        .skip_while(|line| !line.starts_with(&start));
    let Some(first) = lines.next() else {
        return Vec::new();
    };
    std::iter::once(first)
        .chain(lines.take_while(|line| crate::support::is_indented(line)))
        .collect()
}

/// Comando logo abaixo de `<label> <name>` na Verificação, com as continuações em `\`; a mesma
/// regra de `labeled_commands` do `task.rs`.
fn labeled_command(verification: &str, label: &str, name: &str) -> String {
    let marker = format!("{label} {name}");
    let lines: Vec<&str> = verification.lines().collect();
    let Some(position) = lines.iter().position(|line| line.trim() == marker) else {
        return String::new();
    };
    let mut command = Vec::new();
    for line in &lines[position + 1..] {
        let opens = command.is_empty();
        if opens && (line.trim().is_empty() || line.trim_start().starts_with('#')) {
            break;
        }
        command.push(*line);
        if !line.trim_end().ends_with('\\') {
            break;
        }
    }
    command.join("\n").trim().to_string()
}

/// Corta `text` nos separadores que estão fora de aspas e de parênteses.
fn top_level_split(text: &str, separators: &[&str]) -> Vec<String> {
    let mut pieces = Vec::new();
    let mut current = String::new();
    let (mut single, mut double, mut depth) = (false, false, 0usize);
    let mut index = 0;
    while index < text.len() {
        let rest = &text[index..];
        let character = rest.chars().next().unwrap_or_default();
        if !single && !double && depth == 0 {
            if let Some(separator) = separators.iter().find(|sep| rest.starts_with(**sep)) {
                pieces.push(std::mem::take(&mut current));
                index += separator.len();
                continue;
            }
        }
        match character {
            '\'' if !double => single = !single,
            '"' if !single => double = !double,
            '(' if !single && !double => depth += 1,
            ')' if !single && !double => depth = depth.saturating_sub(1),
            _ => {}
        }
        current.push(character);
        index += character.len_utf8();
    }
    pieces.push(current);
    pieces
        .into_iter()
        .map(|piece| piece.trim().to_string())
        .filter(|piece| !piece.is_empty())
        .collect()
}

fn is_assignment(word: &str) -> bool {
    let Some((name, _)) = word.split_once('=') else {
        return false;
    };
    let mut characters = name.chars();
    characters
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic() || first == '_')
        && characters.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Primeiro programa do comando, depois de variáveis e de `cd`; vazio quando o primeiro não é um
/// nome reconhecível, como `"$B"` ou `!` (CB-12).
fn program_of(command: &str) -> String {
    for piece in top_level_split(command, &["&&", "||", ";", "|", "\n"]) {
        let words = top_level_split(&piece, &[" ", "\t"]);
        let Some(first) = words.iter().find(|word| !is_assignment(word)) else {
            continue;
        };
        if first == "cd" {
            continue;
        }
        let recognizable = !first.starts_with('-')
            && first
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '/' | '+' | '-'));
        if !recognizable {
            return String::new();
        }
        return first.rsplit('/').next().unwrap_or_default().to_string();
    }
    String::new()
}

fn explanation(
    verification: &str,
    kinds: &EvidenceKindNames,
    baseline: &str,
    review: &ReviewState,
) -> Explanation {
    let lines = evidence_lines(verification, &review.name);
    let declared = lines
        .first()
        .and_then(|line| crate::commands::task::evidence_item(line))
        .map(|(_, _, rest)| rest.trim().trim_matches('`').to_string())
        .unwrap_or_default();
    let command = if declared.is_empty() {
        labeled_command(verification, &kinds.label, &review.name)
    } else {
        declared
    };
    let how = lines
        .iter()
        .skip(1)
        .filter_map(|line| crate::support::evidence_subitem(line))
        .find(|(field, _)| *field == HOW_FIELD)
        .map(|(_, text)| text.trim().to_string())
        .unwrap_or_default();
    let tool = if review.kind == kinds.manual {
        MANUAL_TOOL.to_string()
    } else if review.kind == kinds.diff {
        DIFF_TOOL.to_string()
    } else if review.name.ends_with(".webm") {
        VIDEO_TOOL.to_string()
    } else if review.name.ends_with(".png") {
        PRINT_TOOL.to_string()
    } else {
        program_of(&command)
    };
    let reproduce = if review.kind == kinds.manual {
        String::new()
    } else if review.kind == kinds.diff {
        let base = if baseline.is_empty() {
            "HEAD"
        } else {
            baseline
        };
        format!("git diff {base} -- {command}")
    } else {
        command.clone()
    };
    Explanation {
        command,
        tool,
        reproduce,
        how,
    }
}

/// Conferência de uma evidência com o ID `EV-NN` pela posição do item em "Evidências", com a
/// origem, o hash atual e a explicação.
fn review_json(index: usize, review: &ReviewState, explanation: &Explanation) -> String {
    format!(
        "{{\"id\":{},\"name\":{},\"kind\":{},\"status\":{},\"reason\":{},\"reviewed_at\":{},\"origin\":{},\"sha256\":{},\"command\":{},\"tool\":{},\"reproduce\":{},\"how\":{}}}",
        json_string(&format!("EV-{:02}", index + 1)),
        json_string(&review.name),
        json_string(&review.kind),
        json_string(review.status),
        json_string(&review.reason),
        json_string(&review.reviewed_at),
        json_string(&review.origin),
        json_string(&review.sha256),
        json_string(&explanation.command),
        json_string(&explanation.tool),
        json_string(&explanation.reproduce),
        json_string(&explanation.how)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::SettingValue;
    use tempfile::TempDir;

    const MAIN_YML: &str = "paths:\n  specs: ./work/specs\n  plans: ./work/plans\n  tasks: ./work/tasks\n  archive: ./work/.archive\n";
    const SPECS_YML: &str = "draft_dir: draft\nrefined_dir: refined\n";
    const REGISTRO: &str = "## Registro de execução\n\n| # | Data | Etapa | Resultado | Detalhe |\n|---|---|---|---|---|\n| 1 | 2026-11-02 | início | in_progress | começou |\n\n## Histórico\n\n| 1 | 2020-01-01 | x | y | z |\n";

    fn write(project: &Path, relative: &str, content: &str) {
        let path = project.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }

    fn spec_md(id: &str, status: &str) -> String {
        format!("---\nid: {id}\ntitle: \"Título {id}\"\nstatus: {status}\n---\n\n# Corpo {id}\n")
    }

    fn project() -> (TempDir, Painel) {
        let dir = TempDir::new().unwrap();
        let p = dir.path();
        write(p, "vostok/config/main.yml", MAIN_YML);
        write(p, "vostok/config/specs.yml", SPECS_YML);
        write(
            p,
            "vostok/work/specs/refined/SPEC-001.md",
            &spec_md("SPEC-001", "approved"),
        );
        write(
            p,
            "vostok/work/plans/SPEC-001.md",
            "---\nspec: SPEC-001\n---\nplano aberto\n",
        );
        write(
            p,
            "vostok/work/tasks/SPEC-001/TASK-01/estado.yml",
            "id: TASK-01\ntitle: \"Primeira\"\nstatus: in_progress\nwave: 1\n",
        );
        write(
            p,
            "vostok/work/tasks/SPEC-001/TASK-01/execucao.md",
            "visível<!-- oculto -->\n",
        );
        write(
            p,
            "vostok/work/tasks/SPEC-001/TASK-01/registro.md",
            REGISTRO,
        );
        write(
            p,
            "vostok/work/tasks/SPEC-001/TASK-01/evidencias/saida.txt",
            "ok",
        );
        write(
            p,
            "vostok/work/tasks/SPEC-001/TASK-01/evidencias/baseline/b.txt",
            "b",
        );
        write(
            p,
            "vostok/work/.archive/SPEC-000/spec.md",
            &spec_md("SPEC-000", "archived"),
        );
        write(
            p,
            "vostok/work/.archive/SPEC-000/tasks/TASK-01/estado.yml",
            "id: TASK-01\nstatus: done\n",
        );
        write(
            p,
            "vostok/work/.archive/SPEC-000/docs/executivo.html",
            "<html></html>",
        );
        write(p, "outro/segredo.txt", "fora");
        let painel = Painel::new(&Harness {
            root: p.join("vostok"),
        })
        .unwrap();
        (dir, painel)
    }

    fn get(painel: &Painel, target: &str) -> Reply {
        let (path, query) = target.split_once('?').unwrap_or((target, ""));
        let parts: Vec<&str> = path.trim_matches('/').split('/').collect();
        route(painel, &parts, query).unwrap_or_else(not_found)
    }

    fn body(reply: &Reply) -> String {
        String::from_utf8(reply.body.clone()).unwrap()
    }

    #[test]
    fn quoted_task_title_comes_without_escapes() {
        let (dir, painel) = project();
        write(
            dir.path(),
            "vostok/work/tasks/SPEC-001/TASK-02/estado.yml",
            r#"id: TASK-02
title: "Separar a coluna \"Em validação\" no quadro"
description: "Cor em \"Em validação\" e barra \\ literal"
status: pending
wave: 1
"#,
        );
        let spec = body(&get(&painel, "/api/specs/SPEC-001"));
        let task = body(&get(&painel, "/api/tarefas/SPEC-001/TASK-02"));
        let title = r#""title":"Separar a coluna \"Em validação\" no quadro""#;
        assert!(spec.contains(title), "{spec}");
        assert!(task.contains(title), "{task}");
        assert!(
            task.contains(r#""description":"Cor em \"Em validação\" e barra \\ literal""#),
            "{task}"
        );
        for reply in [&spec, &task] {
            assert!(!reply.contains(r#"\\\"Em"#), "{reply}");
        }
    }

    #[test]
    fn espera_visao_traz_os_campos_de_espera() {
        let (dir, painel) = project();
        write(
            dir.path(),
            "vostok/work/tasks/SPEC-001/TASK-02/estado.yml",
            "id: TASK-02\ntitle: \"Segunda\"\nstatus: blocked\nwave: 1\nwaiting_on: [TASK-03]\nwaiting_reason: \"espera o comando novo\"\ncloses_after: [TASK-12]\n",
        );
        let visao = body(&get(&painel, "/api/visao"));
        assert!(visao.contains(r#""waiting_on":"[TASK-03]""#), "{visao}");
        assert!(
            visao.contains(r#""waiting_reason":"espera o comando novo""#),
            "{visao}"
        );
        assert!(visao.contains(r#""closes_after":"[TASK-12]""#), "{visao}");
    }

    #[test]
    fn espera_campos_vazios_ou_null_saem_vazios() {
        let (dir, painel) = project();
        write(
            dir.path(),
            "vostok/work/tasks/SPEC-001/TASK-02/estado.yml",
            "id: TASK-02\ntitle: \"Segunda\"\nstatus: pending\nwave: 1\nwaiting_on: []\nwaiting_reason: null\n",
        );
        let visao = body(&get(&painel, "/api/visao"));
        assert!(visao.contains(r#""waiting_reason":"""#), "{visao}");
        assert!(visao.contains(r#""closes_after":"""#), "{visao}");
        assert!(visao.contains(r#""waiting_on":"""#), "{visao}");
    }

    #[test]
    fn campos_calculados_na_tarefa_arquivada() {
        let (_dir, painel) = project();
        let spec = body(&get(&painel, "/api/specs/SPEC-000"));
        let task = body(&get(&painel, "/api/tarefas/SPEC-000/TASK-01"));
        for reply in [&spec, &task] {
            assert!(reply.contains("\"wave\":\"1\""), "{reply}");
            assert!(reply.contains("\"persona\":\"\""), "{reply}");
        }
    }

    #[test]
    fn overview_lists_open_and_archived_specs_and_activity() {
        let (_dir, painel) = project();
        let reply = get(&painel, "/api/visao");
        let text = body(&reply);
        assert_eq!(reply.status, 200);
        assert!(
            text.contains("\"specs\":[{\"meta\":{\"id\":\"SPEC-001\""),
            "{text}"
        );
        assert!(text.contains("\"plan\":true"));
        assert!(text.contains("\"id\":\"TASK-01\""));
        assert!(text.contains("\"archived\":[{\"meta\":{\"id\":\"SPEC-000\""));
        assert!(text.contains("\"tasks_total\":1"));
        assert!(text.contains("\"step\":\"início\""));
        assert!(
            !text.contains("\"step\":\"x\""),
            "o histórico não é atividade"
        );
    }

    #[test]
    fn overview_of_a_folder_without_work_is_empty() {
        let dir = TempDir::new().unwrap();
        write(dir.path(), "vostok/config/main.yml", MAIN_YML);
        let painel = Painel::new(&Harness {
            root: dir.path().join("vostok"),
        })
        .unwrap();
        let text = body(&get(&painel, "/api/visao"));
        assert!(text.contains("\"specs\":[]"));
        assert!(text.contains("\"archived\":[]"));
    }

    #[test]
    fn spec_returns_content_and_metadata() {
        let (_dir, painel) = project();
        let text = body(&get(&painel, "/api/specs/SPEC-001"));
        assert!(text.contains("\"title\":\"Título SPEC-001\""));
        assert!(text.contains("# Corpo SPEC-001"));
        assert!(!text.contains("status: approved"), "sem o front-matter");
        assert!(text.contains("\"archived\":false"));
        assert!(text.contains("\"path\":\"vostok/work/specs/refined/SPEC-001.md\""));
        assert!(text.contains("\"id\":\"TASK-01\""));
    }

    #[test]
    fn archived_spec_comes_with_archived_true() {
        let (_dir, painel) = project();
        let text = body(&get(&painel, "/api/specs/SPEC-000"));
        assert!(text.contains("\"archived\":true"), "{text}");
        assert!(text.contains("# Corpo SPEC-000"));
        let task = body(&get(&painel, "/api/tarefas/SPEC-000/TASK-01"));
        assert!(task.contains("\"archived\":true"));
    }

    #[test]
    fn missing_or_unsafe_ids_are_404() {
        let (_dir, painel) = project();
        for target in [
            "/api/specs/SPEC-999",
            "/api/specs/..",
            "/api/planos/SPEC-999",
            "/api/planos/%2e%2e",
            "/api/tarefas/SPEC-001/TASK-99",
            "/api/tarefas/SPEC-001/..",
            "/api/rota-inexistente",
        ] {
            assert_eq!(get(&painel, target).status, 404, "{target}");
        }
    }

    #[test]
    fn plan_returns_markdown_without_front_matter() {
        let (_dir, painel) = project();
        let text = body(&get(&painel, "/api/planos/SPEC-001"));
        assert!(text.contains("plano aberto"));
        assert!(!text.contains("spec: SPEC-001"));
    }

    #[test]
    fn task_returns_parts_state_and_evidence_without_subfolders() {
        let (_dir, painel) = project();
        let text = body(&get(&painel, "/api/tarefas/SPEC-001/TASK-01"));
        assert!(text.contains("\"state\":{\"id\":\"TASK-01\""));
        assert!(
            text.contains("\"execucao\":\"visível\""),
            "comentários HTML removidos: {text}"
        );
        assert!(text.contains("\"name\":\"saida.txt\""));
        assert!(
            text.contains("\"path\":\"vostok/work/tasks/SPEC-001/TASK-01/evidencias/saida.txt\"")
        );
        assert!(!text.contains("b.txt"));
        assert!(text.contains("\"archived\":false"));
    }

    #[test]
    fn documents_lists_html_of_archived_specs() {
        let (_dir, painel) = project();
        let text = body(&get(&painel, "/api/documentos"));
        assert!(text.contains("\"name\":\"executivo\""));
        assert!(text.contains("vostok/work/.archive/SPEC-000/docs/executivo.html"));
    }

    #[test]
    fn brand_without_groups_uses_default_tokens() {
        let (_dir, painel) = project();
        let text = body(&get(&painel, "/api/marca"));
        assert!(
            text.contains("\"light\":{\"color_bg\":\"#f7f5fb\""),
            "{text}"
        );
        assert!(text.contains("\"dark\":{\"color_bg\":\"#1a1528\""));
        assert!(text.contains("\"brand\":{\"name\":\"\""));
        assert!(text.contains("\"icon_data\":\"\""));
    }

    #[test]
    fn brand_reads_groups_and_embeds_the_icon() {
        let (dir, painel) = project();
        write(dir.path(), "vostok/config/documents.yml", "light:\n  color_bg: \"#fff\"\ndark:\n  color_bg: \"#000\"  # escuro\nbrand:\n  name: \"Acme\"\n  icon: \"logo.svg\"\n  primary: \"\"\n  secondary: \"\"\n");
        write(dir.path(), "logo.svg", "<svg/>");
        let text = body(&get(&painel, "/api/marca"));
        assert!(text.contains("\"light\":{\"color_bg\":\"#fff\"}"), "{text}");
        assert!(text.contains("\"dark\":{\"color_bg\":\"#000\"}"));
        assert!(text.contains("\"name\":\"Acme\""));
        assert!(text.contains("\"icon_data\":\"data:image/svg+xml;base64,PHN2Zy8+\""));
    }

    #[test]
    fn brand_icon_outside_the_project_is_not_embedded() {
        let (dir, painel) = project();
        write(
            dir.path(),
            "vostok/config/documents.yml",
            "brand:\n  name: \"A\"\n  icon: \"../x.svg\"\n",
        );
        assert!(body(&get(&painel, "/api/marca")).contains("\"icon_data\":\"\""));
    }

    #[test]
    fn source_returns_content_and_sha256() {
        let (_dir, painel) = project();
        let text = body(&get(
            &painel,
            "/api/fonte?caminho=vostok/work/plans/SPEC-001.md",
        ));
        assert!(text.contains("plano aberto"));
        let hash = sha256_hex(b"---\nspec: SPEC-001\n---\nplano aberto\n");
        assert!(text.contains(&format!("\"hash\":\"{hash}\"")));
    }

    #[test]
    fn file_serves_inside_vostok_and_refuses_outside_or_traversal() {
        let (_dir, painel) = project();
        let ok = get(
            &painel,
            "/api/arquivo?caminho=vostok/work/.archive/SPEC-000/docs/executivo.html",
        );
        assert_eq!(ok.status, 200);
        assert_eq!(ok.content_type, "text/html; charset=utf-8");
        for caminho in [
            "outro/segredo.txt",
            "vostok/../outro/segredo.txt",
            "vostok%2F..%2Foutro%2Fsegredo.txt",
            "/etc/passwd",
            "",
        ] {
            let reply = get(&painel, &format!("/api/arquivo?caminho={caminho}"));
            assert_eq!(reply.status, 400, "{caminho}");
        }
        assert_eq!(
            get(&painel, "/api/arquivo?caminho=vostok/nada.md").status,
            404
        );
    }

    #[cfg(unix)]
    #[test]
    fn file_refuses_a_symlink_that_leaves_vostok() {
        let (dir, painel) = project();
        std::os::unix::fs::symlink(
            dir.path().join("outro/segredo.txt"),
            dir.path().join("vostok/atalho.txt"),
        )
        .unwrap();
        assert_eq!(
            get(&painel, "/api/arquivo?caminho=vostok/atalho.txt").status,
            400
        );
    }

    #[test]
    fn fonte_de_html_continua_em_json() {
        let (dir, painel) = project();
        write(dir.path(), "vostok/work/pagina.html", "<h1>oi</h1>");
        let reply = get(&painel, "/api/fonte?caminho=vostok/work/pagina.html");
        assert_eq!(reply.status, 200);
        assert_eq!(reply.content_type, "application/json; charset=utf-8");
        assert!(reply.headers.is_empty(), "{:?}", reply.headers);
        let text = body(&reply);
        assert!(text.contains("\"conteudo\":\"<h1>oi</h1>\""), "{text}");
        assert!(
            text.contains(&format!("\"hash\":\"{}\"", sha256_hex(b"<h1>oi</h1>"))),
            "{text}"
        );
    }

    #[test]
    fn arquivos_route_opens_files_by_path() {
        let (_dir, painel) = project();
        let reply = get(
            &painel,
            "/arquivos/vostok/work/.archive/SPEC-000/docs/executivo.html",
        );
        assert_eq!(reply.status, 200);
        assert_eq!(get(&painel, "/arquivos/outro/segredo.txt").status, 400);
    }

    #[test]
    fn version_route_uses_live_version() {
        let (_dir, painel) = project();
        let text = body(&get(&painel, "/api/versao"));
        assert!(text.starts_with("{\"versao\":\""));
    }

    #[test]
    fn data_is_read_on_every_request() {
        let (dir, painel) = project();
        assert!(!body(&get(&painel, "/api/visao")).contains("SPEC-002"));
        write(
            dir.path(),
            "vostok/work/specs/refined/SPEC-002.md",
            &spec_md("SPEC-002", "refined"),
        );
        assert!(body(&get(&painel, "/api/visao")).contains("SPEC-002"));
    }

    #[test]
    fn refined_spec_wins_over_draft_with_the_same_id() {
        let (dir, painel) = project();
        write(
            dir.path(),
            "vostok/work/specs/draft/SPEC-001.md",
            &spec_md("SPEC-001", "draft"),
        );
        let text = body(&get(&painel, "/api/visao"));
        assert_eq!(text.matches("\"id\":\"SPEC-001\"").count(), 1);
        assert!(text.contains("\"status\":\"approved\""));
    }

    #[test]
    fn base64_pads_like_the_standard() {
        assert_eq!(base64(b"a"), "YQ==");
        assert_eq!(base64(b"ab"), "YWI=");
        assert_eq!(base64(b"abc"), "YWJj");
    }

    #[test]
    fn strip_html_comments_handles_unclosed_comment() {
        assert_eq!(strip_html_comments("a<!-- b --> c"), "a c");
        assert_eq!(strip_html_comments("a<!-- b"), "a");
    }

    #[test]
    fn revisao_traz_o_id_pela_posicao() {
        use crate::commands::task::ReviewState;
        let review = ReviewState {
            name: "tela.png".into(),
            kind: "print".into(),
            status: "a conferir",
            reason: String::new(),
            reviewed_at: String::new(),
            origin: String::new(),
            sha256: String::new(),
        };
        let first = review_json(0, &review, &Explanation::default());
        assert!(first.contains("\"reason\":\"\",\"reviewed_at\":\"\""));
        assert!(first.contains("\"id\":\"EV-01\"") && first.contains("\"name\":\"tela.png\""));
        assert!(review_json(9, &review, &Explanation::default()).contains("\"id\":\"EV-10\""));
    }

    const EXPLAIN_TASKS_CONFIG: &str = "status:\n  pending: pending\n  approved: approved\n  in_progress: in_progress\n  validating: validating\n  blocked: blocked\n  done: done\nexecution_section: Execução\nevidence_subsection: Evidências\nrecord_section: Registro de execução\nevidence_from_verify: \"automática, da verificação\"\nevidence_review_kinds: [automática]\nverify_label: \"# evidência:\"\nevidence_manual: manual\nevidence_diff: diff\n";
    const ESTADO_COM_BASELINE: &str =
        "id: TASK-01\ntitle: \"Primeira\"\nstatus: in_progress\nwave: 1\nbaseline_commit: abc1234\n";
    const TASK_DIR: &str = "vostok/work/tasks/SPEC-001/TASK-01";

    fn explained(verification: &str, estado: &str, files: &[(&str, &str)]) -> String {
        let (dir, painel) = project();
        write(dir.path(), "vostok/config/tasks.yml", EXPLAIN_TASKS_CONFIG);
        write(
            dir.path(),
            &format!("{TASK_DIR}/verificacao.md"),
            verification,
        );
        write(dir.path(), &format!("{TASK_DIR}/estado.yml"), estado);
        for (relative, content) in files {
            write(dir.path(), &format!("{TASK_DIR}/{relative}"), content);
        }
        body(&get(&painel, "/api/tarefas/SPEC-001/TASK-01"))
    }

    fn review_of<'a>(body: &'a str, name: &str) -> &'a str {
        let marker = format!("\"name\":{}", json_string(name));
        let reviews = body
            .split_once("\"reviews\":[")
            .map_or(body, |(_, rest)| rest);
        reviews
            .split("{\"id\":")
            .find(|part| part.contains(&marker))
            .unwrap_or_else(|| panic!("sem a evidência {name} em {body}"))
    }

    fn evidences(items: &str) -> String {
        format!("### Evidências\n\n{items}")
    }

    fn assert_review(body: &str, name: &str, fragments: &[&str]) {
        let review = review_of(body, name);
        for fragment in fragments {
            assert!(review.contains(fragment), "faltou {fragment} em {review}");
        }
    }

    #[test]
    fn explicacao_de_cada_tipo_de_evidencia() {
        let verification = "### Verificação\n\n```bash\n# evidência: log.txt\npnpm --dir apps/painel exec vitest run src/a.test.ts\n```\n\n### Evidências\n\n- `log.txt` (automática, da verificação)\n- `cargo.txt` (automática): `cargo test --test task -- review`\n- `skill.diff` (diff): `packages/harness/skills/default/x/SKILL.md`\n- `tela.png` (print): `bash apps/painel/scripts/print.sh /tarefas tela.png`\n- `fluxo.webm` (print): `bash apps/painel/scripts/print.sh /tarefas fluxo.webm`\n- `foto.jpg` (manual): foto do aparelho.\n";
        let body = explained(verification, ESTADO_COM_BASELINE, &[]);
        assert_review(
            &body,
            "log.txt",
            &[
                r#""command":"pnpm --dir apps/painel exec vitest run src/a.test.ts","tool":"pnpm","reproduce":"pnpm --dir apps/painel exec vitest run src/a.test.ts","how":"""#,
            ],
        );
        assert_review(
            &body,
            "cargo.txt",
            &[r#""tool":"cargo","reproduce":"cargo test --test task -- review""#],
        );
        assert_review(
            &body,
            "skill.diff",
            &[
                r#""command":"packages/harness/skills/default/x/SKILL.md","tool":"git diff","reproduce":"git diff abc1234 -- packages/harness/skills/default/x/SKILL.md""#,
            ],
        );
        assert_review(
            &body,
            "tela.png",
            &[
                r#""tool":"Playwright (print)","reproduce":"bash apps/painel/scripts/print.sh /tarefas tela.png""#,
            ],
        );
        assert_review(&body, "fluxo.webm", &[r#""tool":"Playwright (vídeo)""#]);
        assert_review(
            &body,
            "foto.jpg",
            &[r#""command":"foto do aparelho.","tool":"Anexada pelo usuário","reproduce":"""#],
        );
    }

    #[test]
    fn explicacao_traz_a_frase_do_como() {
        let body = explained(
            &evidences("- `gera.txt` (automática): `bash scripts/monta.sh`\n  - Título: Fixture montada\n  - Como: monta a fixture com duas tarefas e roda a conferência\n- `sem.txt` (automática): `cargo test`\n"),
            ESTADO_COM_BASELINE,
            &[],
        );
        assert_review(
            &body,
            "gera.txt",
            &[
                r#""tool":"bash","reproduce":"bash scripts/monta.sh","how":"monta a fixture com duas tarefas e roda a conferência""#,
            ],
        );
        assert_review(&body, "sem.txt", &[r#""how":"""#]);
    }

    #[test]
    fn explicacao_sem_programa_reconhecivel_fica_so_com_o_comando() {
        let body = explained(
            &evidences("- `a.txt` (automática): `\"$B\" task evidence SPEC-001 TASK-01`\n- `b.txt` (automática): `! grep -n x a.txt`\n"),
            ESTADO_COM_BASELINE,
            &[],
        );
        assert_review(
            &body,
            "a.txt",
            &[r#""tool":"","reproduce":"\"$B\" task evidence SPEC-001 TASK-01""#],
        );
        assert_review(
            &body,
            "b.txt",
            &[r#""tool":"","reproduce":"! grep -n x a.txt""#],
        );
    }

    #[test]
    fn explicacao_acha_o_programa_depois_de_variaveis_e_cd() {
        let body = explained(
            &evidences("- `a.txt` (automática): `FOO=1 cargo test`\n- `b.txt` (automática): `cd apps/cli && cargo test`\n- `c.txt` (automática): `D=\"$(mktemp -d)\" && bash x.sh \"$D\"`\n- `d.txt` (automática): `./target/debug/vostok task status`\n"),
            ESTADO_COM_BASELINE,
            &[],
        );
        for (name, tool) in [
            ("a.txt", "cargo"),
            ("b.txt", "cargo"),
            ("c.txt", "bash"),
            ("d.txt", "vostok"),
        ] {
            assert_review(&body, name, &[&format!("\"tool\":\"{tool}\"")]);
        }
    }

    #[test]
    fn explicacao_do_diff_sem_baseline_usa_o_head() {
        let body = explained(
            &evidences("- `a.diff` (diff): `docs/a.md`\n"),
            "id: TASK-01\nstatus: in_progress\n",
            &[],
        );
        assert_review(
            &body,
            "a.diff",
            &[r#""reproduce":"git diff HEAD -- docs/a.md""#],
        );
    }

    #[test]
    fn revisao_traz_origem_hash_e_recusada() {
        let h = |text: &str| sha256_hex(text.as_bytes())[..8].to_string();
        let reviews = format!(
            "| ID | Arquivo | Estado | sha256 | Data | Origem | Texto |\n|---|---|---|---|---|---|---|\n| EV-01 | a.txt | conferida | {} | 2026-11-13T14:02:11Z | painel | ok |\n| EV-02 | b.txt | recusada | {} | 2026-11-13T14:03:40Z | chat | falta o total |\n",
            h("a"),
            h("b")
        );
        let body = explained(
            &evidences("- `a.txt` (automática): `echo a`\n- `b.txt` (automática): `echo b`\n- `c.txt` (automática): `echo c`\n"),
            ESTADO_COM_BASELINE,
            &[
                ("evidencias/a.txt", "a"),
                ("evidencias/b.txt", "b"),
                ("evidencias/c.txt", "c"),
                ("conferencias.md", &reviews),
            ],
        );
        assert_review(
            &body,
            "a.txt",
            &[
                r#""status":"conferida""#,
                r#""origin":"painel""#,
                &format!("\"sha256\":\"{}\"", h("a")),
            ],
        );
        assert_review(
            &body,
            "b.txt",
            &[
                r#""status":"recusada""#,
                r#""reason":"falta o total""#,
                r#""origin":"chat""#,
            ],
        );
        assert_review(
            &body,
            "c.txt",
            &[
                r#""status":"a conferir""#,
                r#""origin":"""#,
                &format!("\"sha256\":\"{}\"", h("c")),
            ],
        );
    }

    #[test]
    fn configuracoes_json_traz_arquivos_e_valores() {
        let snapshot = SettingsSnapshot {
            files: BTreeMap::from([
                ("documents.yml".to_string(), "d".to_string()),
                ("tasks.yml".to_string(), "t".to_string()),
            ]),
            values: BTreeMap::from([
                (
                    "tasks.evidence_baseline_timeout".to_string(),
                    SettingValue::Int(120),
                ),
                (
                    "tasks.persist_dir".to_string(),
                    SettingValue::Text(String::new()),
                ),
                (
                    "tasks.regression_alert".to_string(),
                    SettingValue::List(vec!["FAILED".into(), "error".into()]),
                ),
            ]),
        };
        assert_eq!(
            settings_json(&snapshot),
            r#"{"arquivos":{"documents.yml":"d","tasks.yml":"t"},"valores":{"tasks.evidence_baseline_timeout":120,"tasks.persist_dir":"","tasks.regression_alert":["FAILED","error"]}}"#
        );
    }
}
