//! Leitura dos arquivos de `vostok/` para o painel: visão, specs, planos, tarefas e documentos.

use std::fs;
use std::path::{Component, Path, PathBuf};

use super::{
    json, json_string, live, not_found, percent_decode, query_value, safe_segment, text, Painel,
    Reply,
};
use crate::harness::Harness;
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

/// Quantas atividades `GET /api/atividades` devolve quando o pedido não traz `limite`.
const DEFAULT_ACTIVITIES: usize = 15;

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
        ["api", "atividades"] => Ok(json(layout.activities(query))),
        ["api", "marca"] => Ok(json(layout.brand())),
        ["api", "documentos"] => Ok(json(layout.documents())),
        ["api", "configuracoes"] => Ok(json(layout.settings())),
        ["api", "specs", id] => layout.spec(id),
        ["api", "planos", id] => layout.plan(id),
        ["api", "tarefas", spec, task] => layout.task(spec, task),
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

impl Layout<'_> {
    /// Specs abertas com as tarefas, specs arquivadas e a atividade recente do registro de execução.
    fn overview(&self) -> String {
        let open: Vec<String> = self
            .specs()
            .iter()
            .map(|spec| {
                let plan = self.plans.join(format!("{}.md", spec.id)).is_file();
                format!(
                    "{{\"meta\":{},\"plan\":{plan},\"tasks\":[{}]}}",
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
            "{{\"project\":{},\"specs\":[{}],\"archived\":[{}],\"activity\":[{}],\"questions\":[{}]}}",
            json_string(&project),
            open.join(","),
            archived.join(","),
            self.activity().join(","),
            self.questions().join(",")
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
        task_dirs(&self.tasks.join(spec))
            .iter()
            .map(|dir| task_summary(dir))
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

    /// `GET /api/atividades`: tarefas (abertas e arquivadas), specs, planos, pedidos e avais,
    /// filtrados por `tipo` e `spec`, da mais nova para a mais antiga.
    fn activities(&self, query: &str) -> String {
        let kind = query_value(query, "tipo");
        let spec = query_value(query, "spec");
        let limit = query_value(query, "limite")
            .parse::<usize>()
            .unwrap_or(DEFAULT_ACTIVITIES);
        let mut rows = self.task_rows();
        rows.extend(self.archived_task_rows());
        rows.extend(self.adjustment_rows());
        rows.extend(self.request_rows());
        rows.extend(self.approval_rows());
        rows.retain(|row| {
            (kind.is_empty() || row.kind == kind) && (spec.is_empty() || row.spec == spec)
        });
        sort_newest_first(&mut rows);
        let items: Vec<String> = rows.iter().take(limit).map(ActivityRow::json).collect();
        format!("[{}]", items.join(","))
    }

    fn task_rows(&self) -> Vec<ActivityRow> {
        let mut rows = Vec::new();
        for spec in self.specs() {
            rows.extend(task_rows_in(&self.tasks.join(&spec.id), &spec.id));
        }
        rows
    }

    fn archived_task_rows(&self) -> Vec<ActivityRow> {
        let mut rows = Vec::new();
        for (dir, _) in self.archived() {
            let spec = dir.file_name().unwrap_or_default().to_string_lossy();
            rows.extend(task_rows_in(&dir.join("tasks"), &spec));
        }
        rows
    }

    /// Histórico de ajustes das specs (abertas e arquivadas) e dos planos.
    fn adjustment_rows(&self) -> Vec<ActivityRow> {
        let mut rows = Vec::new();
        let specs = self.specs().into_iter().map(|spec| (spec.id, spec.file));
        let archived = self.archived().into_iter().map(|(dir, file)| {
            (
                dir.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned(),
                file,
            )
        });
        for (id, file) in specs.chain(archived) {
            rows.extend(adjustment_rows_in(&file, "spec", &id));
        }
        for file in markdown_files(&self.plans) {
            let id = file
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();
            rows.extend(adjustment_rows_in(&file, "plano", &id));
        }
        rows
    }

    /// Cada pedido entra com a criação e, se mudou de estado, com o estado atual.
    fn request_rows(&self) -> Vec<ActivityRow> {
        let mut rows = Vec::new();
        for file in markdown_files(&self.painel.work.join("pedidos")) {
            let field = |key| frontmatter_get(&file, key).unwrap_or_default();
            let (id, created, updated) = (field("id"), field("criado_em"), field("atualizado_em"));
            if id.is_empty() || created.is_empty() {
                continue;
            }
            let row = |date: &str, step: &str, result: String| ActivityRow {
                date: date.to_string(),
                kind: "pedido".to_string(),
                spec: field("spec"),
                task: String::new(),
                target: id.clone(),
                number: 0,
                step: step.to_string(),
                result,
                detail: field("acao"),
            };
            rows.push(row(&created, "criado", "pendente".to_string()));
            if !updated.is_empty() && updated != created {
                rows.push(row(&updated, "estado", field("estado")));
            }
        }
        rows
    }

    /// O `avais.log`, sem as linhas malformadas.
    fn approval_rows(&self) -> Vec<ActivityRow> {
        let Ok((avais, _)) = crate::aval::read(&self.painel.work) else {
            return Vec::new();
        };
        avais
            .into_iter()
            .map(|aval| ActivityRow {
                spec: aval
                    .id
                    .split('/')
                    .next()
                    .filter(|head| head.starts_with("SPEC-"))
                    .unwrap_or_default()
                    .to_string(),
                date: aval.date,
                kind: "aval".to_string(),
                task: String::new(),
                target: aval.id,
                number: 0,
                step: aval.action,
                result: aval.origin,
                detail: aval.hash,
            })
            .collect()
    }

    /// Perguntas abertas do agente para o usuário (`work/perguntas/*.md`): front-matter com `id`,
    /// `spec`, `task`, `status` e `created_at`, e o corpo com `## Pergunta` e `## Opções`.
    fn questions(&self) -> Vec<String> {
        let mut files = markdown_files(&self.painel.work.join("perguntas"));
        files.sort();
        files
            .iter()
            .filter(|file| frontmatter_get(file, "status").is_ok_and(|status| status == "aberta"))
            .map(|file| {
                let body = strip_frontmatter(&fs::read_to_string(file).unwrap_or_default());
                format!(
                    "{{\"meta\":{},\"path\":{},\"markdown\":{}}}",
                    frontmatter_json(
                        file,
                        &["id", "spec", "task", "status", "created_at", "author"]
                    ),
                    json_string(&self.relative(file)),
                    json_string(&body)
                )
            })
            .collect()
    }

    /// Os arquivos de `config/` do harness, com o texto inteiro: o painel os mostra por grupo, com
    /// os comentários como ajuda.
    fn settings(&self) -> String {
        let mut files: Vec<PathBuf> = fs::read_dir(self.painel.root.join("config"))
            .map(|entries| {
                entries
                    .flatten()
                    .map(|entry| entry.path())
                    .filter(|path| path.extension().is_some_and(|ext| ext == "yml"))
                    .collect()
            })
            .unwrap_or_default();
        files.sort();
        let items: Vec<String> = files
            .iter()
            .map(|file| {
                format!(
                    "{{\"name\":{},\"path\":{},\"content\":{}}}",
                    json_string(&file.file_name().unwrap_or_default().to_string_lossy()),
                    json_string(&self.relative(file)),
                    json_string(&fs::read_to_string(file).unwrap_or_default())
                )
            })
            .collect();
        format!("[{}]", items.join(","))
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
        let tasks_dir = if archived {
            self.archive.join(safe_id).join("tasks")
        } else {
            self.tasks.join(safe_id)
        };
        let tasks: Vec<String> = task_dirs(&tasks_dir)
            .iter()
            .map(|dir| task_summary(dir))
            .collect();
        Ok(json(format!(
            "{{\"meta\":{},\"path\":{},\"plan\":{plan},\"archived\":{archived},\"markdown\":{},\"tasks\":[{}]}}",
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
        Ok(json(format!(
            "{{\"spec\":{},\"archived\":{archived},\"state\":{},\"files\":{{{}}},\"evidence\":[{}]}}",
            json_string(spec),
            yaml_json(&dir.join("estado.yml")),
            files.join(","),
            evidence.join(",")
        )))
    }

    /// Um arquivo de dentro do harness (`vostok/`), para evidências e documentos.
    fn file(&self, relative: &str) -> Result<Reply, Reply> {
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
        let body = fs::read(canonical).map_err(|_| not_found())?;
        Ok(Reply {
            status: 200,
            content_type: content_type(relative),
            body,
        })
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
            .map(|bytes| {
                let mime = content_type(icon).split(';').next().unwrap_or("");
                format!("data:{mime};base64,{}", base64(&bytes))
            })
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

/// As linhas da seção "Histórico de ajustes" (`# | Data | Origem | Resumo`) de uma spec ou plano.
fn adjustment_rows_in(file: &Path, kind: &str, id: &str) -> Vec<ActivityRow> {
    let content = fs::read_to_string(file).unwrap_or_default();
    let Some((_, history)) = content.split_once("## Histórico de ajustes") else {
        return Vec::new();
    };
    history
        .lines()
        .take_while(|line| !line.starts_with("## "))
        .filter(|line| line.starts_with("| ") && !line.starts_with("| #"))
        .filter_map(|line| {
            let cells: Vec<&str> = line.trim_matches('|').split('|').map(str::trim).collect();
            (cells.len() >= 4).then(|| ActivityRow {
                date: cells[1].to_string(),
                kind: kind.to_string(),
                spec: id.to_string(),
                task: String::new(),
                target: id.to_string(),
                number: cells[0].parse().unwrap_or(0),
                step: "ajuste".to_string(),
                result: cells[2].to_string(),
                detail: cells[3].to_string(),
            })
        })
        .collect()
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

/// Um YAML plano (`chave: valor`, como o `estado.yml`) em JSON, com os valores como texto.
fn yaml_json(file: &Path) -> String {
    json_object(&estado_pairs(file))
}

fn task_summary(dir: &Path) -> String {
    let estado = estado_pairs(&dir.join("estado.yml"));
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
    ];
    let pairs: Vec<(String, String)> = keys
        .iter()
        .map(|key| {
            let value = estado
                .iter()
                .find(|(name, _)| name == key)
                .map(|(_, value)| value.clone())
                .unwrap_or_default();
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

fn content_type(name: &str) -> &'static str {
    match Path::new(name).extension().and_then(|ext| ext.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("md" | "txt" | "yml" | "log") => "text/plain; charset=utf-8",
        Some("json") => "application/json; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "text/javascript; charset=utf-8",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("webp") => "image/webp",
        Some("gif") => "image/gif",
        Some("mp4") => "video/mp4",
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

#[cfg(test)]
mod tests {
    use super::*;
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
        write(
            p,
            "vostok/work/perguntas/P-1.md",
            "---\nid: P-1\nspec: SPEC-001\nstatus: aberta\n---\n## Pergunta\nQual?\n",
        );
        write(
            p,
            "vostok/work/perguntas/P-2.md",
            "---\nid: P-2\nstatus: respondida\n---\n",
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

    fn activity_project() -> (TempDir, Painel) {
        let (dir, painel) = project();
        let p = dir.path();
        write(
            p,
            "vostok/work/.archive/SPEC-000/tasks/TASK-01/registro.md",
            "| # | Data | Etapa | Resultado | Detalhe |\n|---|---|---|---|---|\n| 1 | 2026-10-01 | início | in_progress | arquivada |\n",
        );
        write(
            p,
            "vostok/work/specs/refined/SPEC-001.md",
            "---\nid: SPEC-001\ntitle: \"T\"\nstatus: approved\n---\n\n## Histórico de ajustes\n\n| # | Data | Origem | Resumo |\n|---|---|---|---|\n| 1 | 2026-11-01 | chat | ajuste da spec |\n",
        );
        write(
            p,
            "vostok/work/plans/SPEC-001.md",
            "---\nspec: SPEC-001\n---\n## Histórico de ajustes\n\n| # | Data | Origem | Resumo |\n|---|---|---|---|\n| 1 | 2026-11-02 | chat | ajuste do plano |\n",
        );
        write(
            p,
            "vostok/work/pedidos/PED-001.md",
            "---\nid: PED-001\nacao: ajustar-tarefa\nspec: SPEC-001\ntarefa: TASK-01\nestado: feito\norigem: painel\ncriado_em: 2026-11-03T10:00:00Z\natualizado_em: 2026-11-03T11:00:00Z\n---\n",
        );
        write(
            p,
            "vostok/work/avais.log",
            "2026-11-03T12:00:00Z\tSPEC-001\tabc\tpainel\taprovar\n",
        );
        (dir, painel)
    }

    fn types_of(reply: &Reply) -> Vec<String> {
        body(reply)
            .split("\"type\":\"")
            .skip(1)
            .map(|rest| rest.split('"').next().unwrap().to_string())
            .collect()
    }

    #[test]
    fn atividades_reunem_todas_as_fontes() {
        let (_dir, painel) = activity_project();
        let reply = get(&painel, "/api/atividades?limite=50");
        assert_eq!(reply.status, 200);
        let types = types_of(&reply);
        for expected in ["tarefa", "spec", "plano", "pedido", "aval"] {
            assert!(types.iter().any(|t| t == expected), "{expected}: {types:?}");
        }
        assert_eq!(types[0], "aval");
        assert_eq!(types.last().unwrap(), "tarefa");
        let json = body(&reply);
        assert!(json.contains("arquivada"));
        let dates: Vec<&str> = json
            .split("\"date\":\"")
            .skip(1)
            .map(|rest| rest.split('"').next().unwrap())
            .collect();
        assert!(dates.windows(2).all(|pair| pair[0] >= pair[1]), "{dates:?}");
    }

    #[test]
    fn atividades_filtram_por_tipo_spec_e_limite() {
        let (_dir, painel) = activity_project();
        let only_specs = get(&painel, "/api/atividades?tipo=spec");
        assert_eq!(types_of(&only_specs), vec!["spec"]);
        let of_spec_000 = body(&get(&painel, "/api/atividades?spec=SPEC-000&limite=50"));
        assert!(of_spec_000.contains("arquivada") && !of_spec_000.contains("ajuste da spec"));
        assert_eq!(types_of(&get(&painel, "/api/atividades?limite=2")).len(), 2);
        assert_eq!(body(&get(&painel, "/api/atividades?tipo=nada")), "[]");
        assert!(types_of(&get(&painel, "/api/atividades?limite=abc")).len() <= 15);
    }

    #[test]
    fn avais_malformados_sao_ignorados() {
        let (dir, painel) = activity_project();
        write(
            dir.path(),
            "vostok/work/avais.log",
            "curta\tlinha\n\n2026-11-03T12:00:00Z\tSPEC-001\tabc\tpainel\n",
        );
        let reply = get(&painel, "/api/atividades?tipo=aval");
        assert_eq!(reply.status, 200);
        assert_eq!(types_of(&reply), vec!["aval"]);
    }

    #[test]
    fn atividades_respondem_em_ate_500_ms() {
        let (dir, painel) = project();
        let mut registro =
            String::from("| # | Data | Etapa | Resultado | Detalhe |\n|---|---|---|---|---|\n");
        for number in 1..=1000 {
            registro.push_str(&format!(
                "| {number} | 2026-11-02 | etapa | ok | d{number} |\n"
            ));
        }
        write(
            dir.path(),
            "vostok/work/tasks/SPEC-001/TASK-01/registro.md",
            &registro,
        );
        let start = std::time::Instant::now();
        let reply = get(&painel, "/api/atividades?limite=15");
        assert!(start.elapsed() < std::time::Duration::from_millis(500));
        assert_eq!(types_of(&reply).len(), 15);
    }

    #[test]
    fn overview_lists_open_and_archived_specs_activity_and_questions() {
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
        assert!(text.contains("\"id\":\"P-1\""));
        assert!(!text.contains("\"id\":\"P-2\""), "só perguntas abertas");
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
    fn settings_lists_config_files_with_full_text() {
        let (_dir, painel) = project();
        let text = body(&get(&painel, "/api/configuracoes"));
        assert!(text.contains("\"name\":\"main.yml\""));
        assert!(text.contains("\"path\":\"vostok/config/specs.yml\""));
        assert!(text.contains("draft_dir: draft"));
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
}
