//! Ações do usuário no painel (POST e PUT), já com o token de sessão conferido.

use std::fs;
use std::path::{Component, Path, PathBuf};

use anyhow::Result;

use super::{json, json_string, not_found, query_value, text, Painel, Reply};
use crate::aval;
use crate::commands::pedidos::{self, Questions, Requests, ANSWERED, OPEN};
use crate::commands::{plan, rules, spec, task};
use crate::harness::Harness;
use crate::support::{config_get, frontmatter_get, frontmatter_set, sha256_hex, utc_datetime};

/// Ações que o painel grava como pedido ao agente. Aprovar spec e tarefa não está aqui: o painel
/// aprova direto, para que o agente não fique entre o clique e a aprovação (RN-04).
const REQUEST_ACTIONS: &[&str] = &[
    "aprovar-plano",
    "planejar-spec",
    "ajustar-spec",
    "ajustar-plano",
    "ajustar-tarefa",
    "executar-tarefa",
    "finalizar-tarefa",
    "desbloquear-tarefa",
    "finalizar-spec",
    "responder-pergunta",
    "fazer-commit",
    "criar-spec",
    "refinar-spec",
    "executar-onda",
    "liberar-commits",
    "desfazer-commits",
];

/// Pedidos que o agente só cumpre com aval: o painel grava o aval junto do pedido (RF-10).
const LOCKED_ACTIONS: &[&str] = &[
    "finalizar-tarefa",
    "aprovar-plano",
    "finalizar-spec",
    "fazer-commit",
    "liberar-commits",
    "desfazer-commits",
];

/// Arquivos da tarefa que só o `vostok` grava (RN-03).
const VOSTOK_ONLY: &[&str] = &["registro.md", "README.md", "estado.yml"];

/// Maior corpo aceito numa ação; o servidor já corta antes, e a ação não confia nisso.
const MAX_BODY: usize = 2 * 1024 * 1024;

/// `parts` são os segmentos do caminho, a partir de `api`; `body` já vem limitado pelo servidor.
pub fn act(painel: &Painel, method: &str, parts: &[&str], query: &str, body: &[u8]) -> Reply {
    if body.len() > MAX_BODY {
        return text(413, "corpo grande demais");
    }
    let value = |key| query_value(query, key);
    match (method, parts) {
        ("PUT", ["api", "arquivo"]) => save(painel, &value("caminho"), &value("hash"), body),
        ("PUT", ["api", "configuracao"]) => {
            save_config(painel, &value("caminho"), &value("hash"), body)
        }
        ("POST", ["api", "aprovacoes"]) => approve(painel, &value("spec"), &value("tarefas")),
        ("POST", ["api", "pedidos"]) => request(painel, query, body),
        ("POST", ["api", "conferencias"]) => review_print(painel, query),
        ("POST", ["api", "limpeza"]) => prune(painel, value("confirmar") == "sim"),
        _ => not_found(),
    }
}

// --- edição ---

/// Qual `adjust record-manual` registra a edição de um arquivo.
#[derive(Debug, PartialEq)]
enum Edited {
    Task(String, String),
    Plan(String),
    Spec(PathBuf),
    Free,
}

/// Grava a edição de um `.md` de `vostok/work/` e registra o ajuste manual, como o agente faria.
fn save(painel: &Painel, relative: &str, hash: &str, body: &[u8]) -> Reply {
    let file = match editable(painel, relative, &painel.work, "md") {
        Ok(file) => file,
        Err(reply) => return reply,
    };
    if let Err(reply) = write_with_hash(&file, hash, body) {
        return reply;
    }
    let (message, problems) = match record_manual(painel, &file) {
        Ok(problems) => ("ajuste manual registrado".to_string(), problems),
        Err(error) => (
            format!("gravado, mas o registro do ajuste falhou: {error:#}"),
            Vec::new(),
        ),
    };
    saved(body, &message, &problems)
}

/// Grava uma configuração de `vostok/config/` com o mesmo controle por hash da edição.
fn save_config(painel: &Painel, relative: &str, hash: &str, body: &[u8]) -> Reply {
    let file = match editable(painel, relative, &painel.root.join("config"), "yml") {
        Ok(file) => file,
        Err(reply) => return reply,
    };
    if let Err(reply) = write_with_hash(&file, hash, body) {
        return reply;
    }
    match rules::validate_at(&painel.project) {
        Ok(problems) => saved(body, "configuração gravada", &problems),
        Err(error) => saved(
            body,
            &format!("gravada, mas a validação das regras falhou: {error:#}"),
            &[],
        ),
    }
}

fn saved(body: &[u8], message: &str, problems: &[String]) -> Reply {
    let problems: Vec<String> = problems
        .iter()
        .map(|problem| json_string(problem))
        .collect();
    json(format!(
        "{{\"hash\":{},\"mensagem\":{},\"problemas\":[{}]}}",
        json_string(&sha256_hex(body)),
        json_string(message),
        problems.join(",")
    ))
}

/// Arquivo existente com a extensão pedida dentro de `dir`, a partir de um caminho relativo ao
/// projeto sem `..`; os arquivos que só o `vostok` grava e os de fora dão 403.
fn editable(
    painel: &Painel,
    relative: &str,
    dir: &Path,
    extension: &str,
) -> Result<PathBuf, Reply> {
    let forbidden = || text(403, "arquivo fora do que o painel pode gravar");
    let path = Path::new(relative);
    let plain = !relative.is_empty()
        && path
            .components()
            .all(|part| matches!(part, Component::Normal(_)));
    let file = painel.project.join(path);
    let kept_by_vostok = path
        .file_name()
        .is_some_and(|name| VOSTOK_ONLY.iter().any(|only| name == *only));
    if !plain
        || kept_by_vostok
        || !file.starts_with(dir)
        || file.extension().is_none_or(|ext| ext != extension)
    {
        return Err(forbidden());
    }
    if !file.is_file() {
        return Err(not_found());
    }
    // Um link simbólico dentro da pasta não pode levar a gravação para fora dela.
    match fs::canonicalize(&file) {
        Ok(real) if real.starts_with(dir) => Ok(file),
        _ => Err(forbidden()),
    }
}

/// Grava só se `expected_hash` for o SHA-256 do arquivo no disco; senão, 409 com o conteúdo e o
/// hash atuais, sem tocar no arquivo. Gravar por cima exige um novo pedido com o hash atual (RN-06).
fn write_with_hash(file: &Path, expected_hash: &str, content: &[u8]) -> Result<(), Reply> {
    let current = fs::read(file).map_err(|_| not_found())?;
    let current_hash = sha256_hex(&current);
    if current_hash != expected_hash {
        let body = format!(
            "{{\"hash\":{},\"conteudo\":{}}}",
            json_string(&current_hash),
            json_string(&String::from_utf8_lossy(&current))
        );
        return Err(Reply {
            status: 409,
            content_type: "application/json; charset=utf-8",
            body: body.into_bytes(),
        });
    }
    let mut temporary = file.as_os_str().to_owned();
    temporary.push(".painel");
    fs::write(&temporary, content)
        .and_then(|()| fs::rename(&temporary, file))
        .map_err(|error| text(500, &error.to_string()))
}

fn record_manual(painel: &Painel, file: &Path) -> Result<Vec<String>> {
    match edited(&Harness::locate(&painel.project)?, file)? {
        Edited::Task(spec_id, task_id) => {
            task::record_manual_at(&painel.project, &spec_id, &task_id)
        }
        Edited::Plan(spec_id) => plan::record_manual_at(&painel.project, &spec_id),
        Edited::Spec(spec_file) => {
            spec::record_manual_at(&painel.project, &frontmatter_get(&spec_file, "id")?)
        }
        Edited::Free => Ok(Vec::new()),
    }
}

/// Classifica pelo lugar do arquivo nas pastas configuradas: tarefa (pasta ou arquivo único),
/// plano ou spec refinada; o resto não tem registro de ajuste.
fn edited(harness: &Harness, file: &Path) -> Result<Edited> {
    let refined = harness.path("specs")?.join(config_get(
        &harness.config_file("specs.yml"),
        "refined_dir",
    )?);
    let parent = file.parent().unwrap_or(file);
    let stem = || file_stem(file);
    if let Ok(inside) = file.strip_prefix(harness.path("tasks")?) {
        let parts: Vec<String> = inside
            .iter()
            .map(|part| part.to_string_lossy().into_owned())
            .collect();
        return Ok(match parts.as_slice() {
            [spec_id, task_id, _] => Edited::Task(spec_id.clone(), task_id.clone()),
            [spec_id, _] => Edited::Task(spec_id.clone(), stem()),
            _ => Edited::Free,
        });
    }
    if parent == harness.path("plans")? {
        return Ok(Edited::Plan(stem()));
    }
    if parent == refined {
        return Ok(Edited::Spec(file.to_path_buf()));
    }
    Ok(Edited::Free)
}

fn file_stem(file: &Path) -> String {
    file.file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_default()
}

// --- aprovação ---

/// Aprova a spec (sem `tarefas`) ou as tarefas citadas pelas conferências do `vostok` e acrescenta o
/// aval de cada item aprovado; uma aprovação recusada não grava aval (FA-04).
fn approve(painel: &Painel, spec_id: &str, tasks: &str) -> Reply {
    if !is_id(spec_id) {
        return text(400, "spec inválida");
    }
    let approved = if tasks.is_empty() {
        spec::approve_from_panel(&painel.project, spec_id)
            .map(|hash| vec![(spec_id.to_string(), hash)])
    } else {
        let ids: Vec<String> = tasks.split(',').map(str::trim).map(String::from).collect();
        if !ids.iter().all(|id| is_id(id)) {
            return text(400, "tarefa inválida");
        }
        task::approve_from_panel(&painel.project, spec_id, &ids).map(|list| {
            list.into_iter()
                .map(|(id, hash)| (format!("{spec_id}/{id}"), hash))
                .collect()
        })
    };
    let approved = match approved {
        Ok(approved) => approved,
        Err(error) => return text(422, &format!("{error:#}")),
    };
    for (id, hash) in &approved {
        if let Err(error) = record_aval(painel, id, hash, "aprovar") {
            return text(
                500,
                &format!("aprovado, mas o aval não foi gravado: {error}"),
            );
        }
    }
    let ids: Vec<String> = approved.iter().map(|(id, _)| json_string(id)).collect();
    json(format!("{{\"aprovadas\":[{}]}}", ids.join(",")))
}

/// O log de avais da spec só recebe acréscimos (RN-05): data, ID, hash aprovado, origem e ação.
fn record_aval(painel: &Painel, id: &str, hash: &str, action: &str) -> Result<()> {
    let harness = Harness {
        root: painel.root.clone(),
    };
    aval::append(&harness, id, hash, "painel", action)
}

// --- pedidos ---

/// Grava um pedido ao agente na subpasta de pendentes, para o chat em loop processar.
fn request(painel: &Painel, query: &str, body: &[u8]) -> Reply {
    let action = query_value(query, "acao");
    let spec_id = query_value(query, "spec");
    let task_id = query_value(query, "tarefa");
    if action.starts_with("aprovar-") && !REQUEST_ACTIONS.contains(&action.as_str()) {
        return text(400, "aprovar spec e tarefa é direto no painel, não pedido");
    }
    if !REQUEST_ACTIONS.contains(&action.as_str()) {
        return text(400, "ação desconhecida");
    }
    let optional_id = |value: &str| value.is_empty() || is_id(value);
    if !optional_id(&spec_id) || !optional_id(&task_id) {
        return text(400, "spec ou tarefa inválida");
    }
    let text_of_user = String::from_utf8_lossy(body);
    let text_of_user = text_of_user.trim();
    let (spec_id, task_id, request_text) =
        match request_fields(painel, &action, query, spec_id, task_id, text_of_user) {
            Ok(fields) => fields,
            Err(reply) => return reply,
        };
    let locked = LOCKED_ACTIONS.contains(&action.as_str());
    let target = if locked {
        match locked_target(painel, &action, &spec_id, &task_id) {
            Ok(target) => Some(target),
            Err(reply) => return reply,
        }
    } else {
        None
    };
    let requests = match Requests::open(&Harness {
        root: painel.root.clone(),
    }) {
        Ok(requests) => requests,
        Err(error) => return text(500, &error.to_string()),
    };
    let created = requests.create(|id| {
        request_markdown(
            id,
            &action,
            &spec_id,
            &task_id,
            &request_text,
            &utc_datetime(),
        )
    });
    let (id, path) = match created {
        Ok(created) => created,
        Err(error) => return text(500, &error.to_string()),
    };
    if let Some(target) = target {
        let (aval_id, hash) = match target {
            AvalTarget::Hashed(aval_id, hash) => (aval_id, hash),
            AvalTarget::Request => (id.clone(), sha256_hex(text_of_user.as_bytes())),
        };
        if let Err(error) = record_aval(painel, &aval_id, &hash, &action) {
            let _ = fs::remove_file(&path);
            return text(500, &format!("o aval não foi gravado: {error}"));
        }
    }
    json(format!("{{\"id\":{}}}", json_string(&id)))
}

/// A spec, a tarefa e o texto do pedido, conferidos para as ações que exigem campos; a resposta
/// a uma pergunta já move a pergunta para as respondidas e usa a spec e a tarefa dela.
fn request_fields(
    painel: &Painel,
    action: &str,
    query: &str,
    spec_id: String,
    task_id: String,
    text_of_user: &str,
) -> Result<(String, String, String), Reply> {
    match action {
        "criar-spec" if text_of_user.is_empty() => {
            Err(text(400, "o assunto da spec é obrigatório"))
        }
        "refinar-spec" | "executar-onda" if spec_id.is_empty() => {
            Err(text(400, "spec obrigatória"))
        }
        "executar-onda" => {
            let wave = query_value(query, "onda")
                .parse::<u32>()
                .ok()
                .filter(|wave| *wave > 0)
                .ok_or_else(|| text(400, "onda inválida"))?;
            let request_text = if text_of_user.is_empty() {
                format!("Onda {wave}.")
            } else {
                format!("Onda {wave}.\n{text_of_user}")
            };
            Ok((spec_id, task_id, request_text))
        }
        "responder-pergunta" => {
            answer_question(painel, &query_value(query, "pergunta"), text_of_user)
        }
        _ => Ok((spec_id, task_id, text_of_user.to_string())),
    }
}

/// Move a pergunta aberta para as respondidas com a resposta; a segunda resposta dá 409 (CB-06).
fn answer_question(
    painel: &Painel,
    question_id: &str,
    answer: &str,
) -> Result<(String, String, String), Reply> {
    if !is_id(question_id) {
        return Err(text(400, "pergunta inválida"));
    }
    if answer.is_empty() {
        return Err(text(400, "a resposta é obrigatória"));
    }
    let questions = Questions::open(&Harness {
        root: painel.root.clone(),
    })
    .map_err(|error| text(500, &error.to_string()))?;
    let folder = |status| questions.folder_of(status).unwrap_or_default();
    let name = format!("{question_id}.md");
    let (open, answered) = (folder(OPEN).join(&name), folder(ANSWERED).join(&name));
    if !open.is_file() {
        return Err(if answered.is_file() {
            text(409, "a pergunta já foi respondida")
        } else {
            text(404, "pergunta não encontrada")
        });
    }
    fs::create_dir_all(folder(ANSWERED)).map_err(|error| text(500, &error.to_string()))?;
    if let Err(error) = fs::rename(&open, &answered) {
        // Outra aba respondeu entre a conferência e o `rename`.
        return Err(if error.kind() == std::io::ErrorKind::NotFound {
            text(409, "a pergunta já foi respondida")
        } else {
            text(500, &error.to_string())
        });
    }
    let recorded = frontmatter_set(&answered, "status", ANSWERED)
        .and_then(|()| spec::frontmatter_put(&answered, "answered_at", &utc_datetime()))
        .and_then(|()| {
            let content = fs::read_to_string(&answered)?;
            fs::write(
                &answered,
                format!("{}\n\n## Resposta\n\n{answer}\n", content.trim_end()),
            )?;
            Ok(())
        });
    if let Err(error) = recorded {
        return Err(text(500, &format!("a resposta não foi gravada: {error:#}")));
    }
    let field = |key| frontmatter_get(&answered, key).unwrap_or_default();
    Ok((
        field("spec"),
        field("task"),
        format!(
            "Resposta à pergunta {question_id} do pedido {}.\n\n{answer}",
            field("pedido")
        ),
    ))
}

// --- conferência de prints e limpeza ---

/// `POST /api/conferencias`: o usuário aceita ou rejeita um print da tarefa (RF-06).
fn review_print(painel: &Painel, query: &str) -> Reply {
    let (spec_id, task_id) = (query_value(query, "spec"), query_value(query, "tarefa"));
    let print = query_value(query, "print");
    if !is_id(&spec_id) || !is_id(&task_id) {
        return text(400, "spec ou tarefa inválida");
    }
    if print.is_empty() || print.contains('/') || print.contains('\\') || print.contains("..") {
        return text(400, "print inválido");
    }
    let result = query_value(query, "resultado");
    let accepted = match result.as_str() {
        "aceito" => true,
        "rejeitado" => false,
        _ => return text(400, "resultado inválido: use aceito ou rejeitado"),
    };
    match task::review_print_from_panel(&painel.project, &spec_id, &task_id, &print, accepted) {
        Ok(()) => json(format!(
            "{{\"conferido\":{},\"resultado\":{}}}",
            json_string(&print),
            json_string(&result)
        )),
        Err(error) => text(422, &format!("{error:#}")),
    }
}

/// `POST /api/limpeza`: lista o que o `vostok pedidos prune` apagaria; com `confirmar=sim`, apaga.
fn prune(painel: &Painel, confirm: bool) -> Reply {
    let harness = Harness {
        root: painel.root.clone(),
    };
    let items = match pedidos::prune_candidates(&harness) {
        Ok(items) => items,
        Err(error) => return text(500, &format!("{error:#}")),
    };
    let paths: Vec<String> = items
        .iter()
        .map(|item| {
            let relative = item
                .path
                .strip_prefix(&painel.project)
                .unwrap_or(&item.path);
            json_string(&relative.to_string_lossy())
        })
        .collect();
    if !confirm {
        return json(format!("{{\"arquivos\":[{}]}}", paths.join(",")));
    }
    match pedidos::prune_delete(&items) {
        Ok(_) => json(format!("{{\"apagados\":[{}]}}", paths.join(","))),
        Err(error) => text(500, &format!("{error:#}")),
    }
}

/// Alvo do aval de um pedido travado: o hash já conhecido, ou o texto do próprio pedido.
enum AvalTarget {
    Hashed(String, String),
    Request,
}

fn locked_target(
    painel: &Painel,
    action: &str,
    spec_id: &str,
    task_id: &str,
) -> Result<AvalTarget, Reply> {
    match action {
        "fazer-commit" | "liberar-commits" | "desfazer-commits" => Ok(AvalTarget::Request),
        "finalizar-tarefa" => {
            let hash = task_approved_hash(painel, spec_id, task_id)
                .ok_or_else(|| text(422, "a tarefa não está aprovada"))?;
            Ok(AvalTarget::Hashed(format!("{spec_id}/{task_id}"), hash))
        }
        "aprovar-plano" => {
            let plan = painel.work.join("plans").join(format!("{spec_id}.md"));
            let hash = frontmatter_get(&plan, "content_hash")
                .ok()
                .filter(|hash| !hash.is_empty() && hash != "null")
                .ok_or_else(|| text(422, "o plano não tem content_hash"))?;
            Ok(AvalTarget::Hashed(spec_id.to_string(), hash))
        }
        _ => {
            let hash = refined_spec(painel, spec_id)
                .and_then(|file| frontmatter_get(&file, "content_hash").ok())
                .filter(|hash| !hash.is_empty() && hash != "null")
                .ok_or_else(|| text(422, "a spec refinada não tem content_hash"))?;
            Ok(AvalTarget::Hashed(spec_id.to_string(), hash))
        }
    }
}

/// `approved_hash` da tarefa, em pasta (`estado.yml`) ou em arquivo único (front-matter).
fn task_approved_hash(painel: &Painel, spec_id: &str, task_id: &str) -> Option<String> {
    let dir = painel.work.join("tasks").join(spec_id);
    let entry = [dir.join(task_id), dir.join(format!("{task_id}.md"))]
        .into_iter()
        .find(|path| path.exists())?;
    task::TaskDoc::at(&entry)
        .get("approved_hash")
        .ok()
        .filter(|hash| !hash.is_empty() && hash != "null")
}

fn refined_spec(painel: &Painel, spec_id: &str) -> Option<PathBuf> {
    fs::read_dir(painel.work.join("specs").join("refined"))
        .ok()?
        .flatten()
        .map(|entry| entry.path())
        .find(|path| {
            path.extension().is_some_and(|extension| extension == "md")
                && path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.starts_with(&format!("{spec_id}-")))
        })
}

/// Pedido no formato do estudo: front-matter com a ação e o estado (`pendente`, `em_andamento`,
/// `feito` ou `recusado`), o texto do usuário e a seção em que o agente registra o resultado.
fn request_markdown(
    id: &str,
    action: &str,
    spec_id: &str,
    task_id: &str,
    text: &str,
    now: &str,
) -> String {
    format!(
        "---\nid: {id}\nacao: {action}\nspec: {spec_id}\ntarefa: {task_id}\nestado: pendente\norigem: painel\ncriado_em: {now}\natualizado_em: {now}\n---\n\n## Pedido\n\n{text}\n\n## Resultado\n\n"
    )
}

fn is_id(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '-')
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    const TASK: &str = "vostok/work/tasks/SPEC-001/TASK-01.md";
    const VALID_EVIDENCE: &str = "- `saida.txt` (automática): `echo ok`\n  - Prova: o comando roda.\n  - Trecho: a linha `ok`.\n  - Falha: saída vazia.";
    const PATH_EVIDENCE: &str = "- `docs/x/saida.txt` (automática): `echo ok`";
    const AVAIS_DIR: &str = "avais";
    const SPEC_LOG: &str = "avais/SPEC-001.log";

    fn task_content(evidence: &str) -> String {
        format!(
            "---\nid: TASK-01\nspec: SPEC-001\ntitle: \"Tarefa\"\neffort: simples\ndepends_on: []\ncovers: [RF-01]\ntype: tarefa\nlabels: [SPEC-001, simples]\nstatus: pending\ncontent_hash: null\nadjustments: 0\nadjusted_at: null\napproved_at: null\napproved_hash: null\nattempts: 0\ntests_result: null\nregression_result: null\nevidence_result: null\n---\n\n# TASK-01: Tarefa\n\n## Execução\n\n### Ler\n\n- `README.md`\n\n### Mudar\n\n- `novo.txt` (criar): arquivo novo.\n\n### Passos\n\n1. Crie o arquivo.\n\n### Baseline\n\n- `true`\n\n### Verificação\n\n```bash\ntrue\n```\n\n### Evidências\n\n{evidence}\n\n### Concluída quando\n\n- [ ] Pronto (RF-01).\n\n  > Como comprovar: ver o arquivo.\n\n## Descrição\n\n### Resumo executivo\n\nResumo.\n\n### Detalhes técnicos\n\nDetalhes.\n\n### Card\n\n- **Título:** Tarefa\n- **Tipo:** tarefa\n- **Descrição:** Descrição.\n- **Critérios de aceite:**\n  - [ ] Pronto.\n- **Rótulos:** SPEC-001, simples\n- **Estimativa:** simples\n- **Bloqueada por:** -\n\n## Registro de execução\n\n| # | Data | Etapa | Resultado | Detalhe |\n|---|---|---|---|---|\n\n## Histórico de ajustes\n\n| # | Data | Origem | Resumo |\n|---|---|---|---|\n"
        )
    }

    /// Projeto temporário com o harness embutido e uma tarefa pendente já registrada.
    fn project(evidence: &str) -> (TempDir, Painel) {
        let dir = tempfile::tempdir().expect("pasta temporária");
        crate::embedded::extract_to(&dir.path().join("vostok")).expect("extrair o harness");
        let painel = Painel::new(&Harness::locate(dir.path()).unwrap()).unwrap();
        write(&painel, TASK, &task_content(evidence));
        task::record_manual_at(&painel.project, "SPEC-001", "TASK-01").unwrap();
        (dir, painel)
    }

    fn write(painel: &Painel, relative: &str, content: &str) {
        let file = painel.project.join(relative);
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(file, content).unwrap();
    }

    fn read(painel: &Painel, relative: &str) -> String {
        fs::read_to_string(painel.project.join(relative)).unwrap()
    }

    fn put(painel: &Painel, route: &str, relative: &str, hash: &str, body: &[u8]) -> Reply {
        let query = format!("caminho={relative}&hash={hash}");
        act(painel, "PUT", &["api", route], &query, body)
    }

    fn post(painel: &Painel, route: &str, query: &str, body: &[u8]) -> Reply {
        act(painel, "POST", &["api", route], query, body)
    }

    fn body(reply: &Reply) -> String {
        String::from_utf8(reply.body.clone()).unwrap()
    }

    fn hash_of(painel: &Painel, relative: &str) -> String {
        sha256_hex(read(painel, relative).as_bytes())
    }

    // --- edição (CT-08 a CT-10, CB-02, CB-04) ---

    #[test]
    fn put_with_current_hash_saves_records_and_returns_hash_and_problems() {
        let (_dir, painel) = project(VALID_EVIDENCE);
        let edited = read(&painel, TASK).replace("Resumo.", "Resumo novo.");
        let reply = put(
            &painel,
            "arquivo",
            TASK,
            &hash_of(&painel, TASK),
            edited.as_bytes(),
        );
        assert_eq!(reply.status, 200, "{}", body(&reply));
        let answer = body(&reply);
        assert!(
            answer.contains(&format!("\"hash\":\"{}\"", sha256_hex(edited.as_bytes()))),
            "{answer}"
        );
        assert!(
            answer.contains("\"mensagem\":\"ajuste manual registrado\""),
            "{answer}"
        );
        assert!(answer.contains("\"problemas\":[]"), "{answer}");
        let saved = read(&painel, TASK);
        assert!(saved.contains("Resumo novo."));
        assert!(saved.contains("adjustments: 2\n"), "{saved}");
    }

    #[test]
    fn put_returns_the_validation_problems_after_saving() {
        let (_dir, painel) = project(VALID_EVIDENCE);
        let broken = read(&painel, TASK).replace("### Passos\n\n1. Crie o arquivo.\n\n", "");
        let reply = put(
            &painel,
            "arquivo",
            TASK,
            &hash_of(&painel, TASK),
            broken.as_bytes(),
        );
        assert_eq!(reply.status, 200, "{}", body(&reply));
        assert!(
            !body(&reply).contains("\"problemas\":[]"),
            "{}",
            body(&reply)
        );
    }

    #[test]
    fn put_with_stale_hash_is_409_with_current_content_and_keeps_the_file() {
        let (_dir, painel) = project(VALID_EVIDENCE);
        let before = read(&painel, TASK);
        let stale = sha256_hex(b"versao antiga");
        let reply = put(&painel, "arquivo", TASK, &stale, b"versao do painel");
        assert_eq!(reply.status, 409);
        let answer = body(&reply);
        assert!(answer.contains(&format!("\"hash\":\"{}\"", sha256_hex(before.as_bytes()))));
        assert!(
            answer.contains("\"conteudo\":\"---\\nid: TASK-01"),
            "{answer}"
        );
        assert_eq!(read(&painel, TASK), before);
    }

    #[test]
    fn put_without_hash_is_409() {
        let (_dir, painel) = project(VALID_EVIDENCE);
        assert_eq!(put(&painel, "arquivo", TASK, "", b"x").status, 409);
    }

    #[test]
    fn put_on_files_kept_by_vostok_is_403() {
        let (_dir, painel) = project(VALID_EVIDENCE);
        for name in ["registro.md", "README.md", "estado.yml"] {
            let relative = format!("vostok/work/tasks/SPEC-001/TASK-02/{name}");
            write(&painel, &relative, "x\n");
            let reply = put(&painel, "arquivo", &relative, &sha256_hex(b"x\n"), b"y");
            assert_eq!(reply.status, 403, "{name}");
            assert_eq!(read(&painel, &relative), "x\n", "{name}");
        }
    }

    #[test]
    fn put_outside_work_with_dots_or_not_markdown_is_403() {
        let (_dir, painel) = project(VALID_EVIDENCE);
        write(&painel, "vostok/work/notas.txt", "x\n");
        write(&painel, "fora.md", "x\n");
        let paths = [
            "vostok/config/main.yml",
            "vostok/work/../config/main.yml",
            "vostok/work/../../fora.md",
            "fora.md",
            "/etc/passwd",
            "vostok/work/notas.txt",
            "",
        ];
        for path in paths {
            let reply = put(&painel, "arquivo", path, &sha256_hex(b"x\n"), b"y");
            assert_eq!(reply.status, 403, "{path}");
        }
        assert_eq!(read(&painel, "fora.md"), "x\n");
    }

    #[cfg(unix)]
    #[test]
    fn put_through_a_symlink_leaving_work_is_403() {
        let (_dir, painel) = project(VALID_EVIDENCE);
        write(&painel, "fora.md", "x\n");
        std::os::unix::fs::symlink(painel.project.join("fora.md"), painel.work.join("link.md"))
            .unwrap();
        let reply = put(
            &painel,
            "arquivo",
            "vostok/work/link.md",
            &sha256_hex(b"x\n"),
            b"y",
        );
        assert_eq!(reply.status, 403);
        assert_eq!(read(&painel, "fora.md"), "x\n");
    }

    #[test]
    fn put_on_a_missing_file_is_404() {
        let (_dir, painel) = project(VALID_EVIDENCE);
        let reply = put(&painel, "arquivo", "vostok/work/nada.md", "", b"y");
        assert_eq!(reply.status, 404);
        assert!(!painel.work.join("nada.md").exists());
    }

    #[test]
    fn body_above_2_mb_is_refused_without_writing() {
        let (_dir, painel) = project(VALID_EVIDENCE);
        let before = read(&painel, TASK);
        let big = vec![b'a'; MAX_BODY + 1];
        let reply = put(&painel, "arquivo", TASK, &hash_of(&painel, TASK), &big);
        assert_eq!(reply.status, 413);
        assert_eq!(read(&painel, TASK), before);
        let reply = post(&painel, "pedidos", "acao=ajustar-tarefa", &big);
        assert_eq!(reply.status, 413);
        assert!(!painel.work.join("pedidos").exists());
    }

    #[test]
    fn body_of_exactly_2_mb_is_accepted() {
        let (_dir, painel) = project(VALID_EVIDENCE);
        let free = "vostok/work/notas.md";
        write(&painel, free, "x\n");
        let full = vec![b'a'; MAX_BODY];
        let reply = put(&painel, "arquivo", free, &sha256_hex(b"x\n"), &full);
        assert_eq!(reply.status, 200, "{}", body(&reply));
        assert_eq!(fs::read(painel.project.join(free)).unwrap().len(), MAX_BODY);
    }

    #[test]
    fn edited_files_are_classified_for_the_record_manual() {
        let (_dir, painel) = project(VALID_EVIDENCE);
        let harness = Harness::locate(&painel.project).unwrap();
        let at = |relative: &str| edited(&harness, &painel.work.join(relative)).unwrap();
        assert_eq!(
            at("tasks/SPEC-001/TASK-02/criterios.md"),
            Edited::Task("SPEC-001".into(), "TASK-02".into())
        );
        assert_eq!(
            at("tasks/SPEC-001/TASK-01.md"),
            Edited::Task("SPEC-001".into(), "TASK-01".into())
        );
        assert_eq!(at("plans/SPEC-002.md"), Edited::Plan("SPEC-002".into()));
        assert_eq!(
            at("specs/refined/SPEC-003-x.md"),
            Edited::Spec(painel.work.join("specs/refined/SPEC-003-x.md"))
        );
        assert_eq!(at("specs/draft/SPEC-004-x.md"), Edited::Free);
        assert_eq!(at("notas.md"), Edited::Free);
    }

    // --- configuração (CT-11) ---

    const CONFIG: &str = "vostok/config/tasks.yml";

    #[test]
    fn config_that_breaks_the_context_map_returns_the_problem() {
        let (_dir, painel) = project(VALID_EVIDENCE);
        let map = "vostok/config/context.yml";
        let broken = read(&painel, map).replacen(
            "required:\n",
            "required:\n  - rules/project/inexistente.md\n",
            1,
        );
        let reply = put(
            &painel,
            "configuracao",
            map,
            &hash_of(&painel, map),
            broken.as_bytes(),
        );
        assert_eq!(reply.status, 200, "{}", body(&reply));
        let answer = body(&reply);
        assert!(
            answer.contains("caminho inexistente: rules/project/inexistente.md"),
            "{answer}"
        );
        assert_eq!(read(&painel, map), broken);
    }

    #[test]
    fn config_with_current_hash_is_saved() {
        let (_dir, painel) = project(VALID_EVIDENCE);
        let changed = format!("{}\n# ajuste pelo painel\n", read(&painel, CONFIG));
        let reply = put(
            &painel,
            "configuracao",
            CONFIG,
            &hash_of(&painel, CONFIG),
            changed.as_bytes(),
        );
        assert_eq!(reply.status, 200, "{}", body(&reply));
        assert!(body(&reply).contains(&sha256_hex(changed.as_bytes())));
        assert!(
            body(&reply).contains("\"problemas\":[]"),
            "{}",
            body(&reply)
        );
        assert_eq!(read(&painel, CONFIG), changed);
    }

    #[test]
    fn config_with_stale_hash_is_409_and_keeps_the_file() {
        let (_dir, painel) = project(VALID_EVIDENCE);
        let before = read(&painel, CONFIG);
        let reply = put(
            &painel,
            "configuracao",
            CONFIG,
            &sha256_hex(b"velho"),
            b"novo: 1\n",
        );
        assert_eq!(reply.status, 409);
        assert_eq!(read(&painel, CONFIG), before);
    }

    #[test]
    fn config_outside_the_config_folder_is_403() {
        let (_dir, painel) = project(VALID_EVIDENCE);
        write(&painel, "vostok/work/x.yml", "a: 1\n");
        write(&painel, "vostok/config/notas.md", "x\n");
        for path in [
            "vostok/work/x.yml",
            "vostok/config/../work/x.yml",
            "vostok/config/notas.md",
            TASK,
            "/etc/hosts",
            "",
        ] {
            let reply = put(
                &painel,
                "configuracao",
                path,
                &sha256_hex(b"a: 1\n"),
                b"b: 2\n",
            );
            assert_eq!(reply.status, 403, "{path}");
        }
        assert_eq!(read(&painel, "vostok/work/x.yml"), "a: 1\n");
    }

    // --- aprovação (CT-04, FA-04) ---

    #[test]
    fn task_approval_records_the_panel_origin_and_the_aval() {
        let (_dir, painel) = project(VALID_EVIDENCE);
        let reply = post(&painel, "aprovacoes", "spec=SPEC-001&tarefas=TASK-01", b"");
        assert_eq!(reply.status, 200, "{}", body(&reply));
        assert_eq!(body(&reply), "{\"aprovadas\":[\"SPEC-001/TASK-01\"]}");
        let content = read(&painel, TASK);
        assert!(content.contains("\nstatus: approved\n"), "{content}");
        assert!(content.contains("\napproved_by: painel\n"), "{content}");
        let approved_hash = frontmatter_get(&painel.project.join(TASK), "approved_hash").unwrap();
        let log = fs::read_to_string(painel.work.join(SPEC_LOG)).unwrap();
        let fields: Vec<&str> = log.trim_end().split('\t').collect();
        assert_eq!(fields.len(), 5, "{log}");
        assert!(fields[0].ends_with('Z') && fields[0].contains('T'), "{log}");
        assert_eq!(
            &fields[1..],
            [
                "SPEC-001/TASK-01",
                approved_hash.as_str(),
                "painel",
                "aprovar"
            ]
        );
    }

    #[test]
    fn avais_log_only_receives_appends() {
        let (_dir, painel) = project(VALID_EVIDENCE);
        fs::create_dir_all(painel.work.join(AVAIS_DIR)).unwrap();
        fs::write(painel.work.join(SPEC_LOG), "linha anterior\n").unwrap();
        let reply = post(&painel, "aprovacoes", "spec=SPEC-001&tarefas=TASK-01", b"");
        assert_eq!(reply.status, 200, "{}", body(&reply));
        let log = fs::read_to_string(painel.work.join(SPEC_LOG)).unwrap();
        assert!(log.starts_with("linha anterior\n"), "{log}");
        assert_eq!(log.lines().count(), 2);
    }

    #[test]
    fn refused_task_approval_is_422_without_aval() {
        let (_dir, painel) = project(PATH_EVIDENCE);
        let before = read(&painel, TASK);
        let reply = post(&painel, "aprovacoes", "spec=SPEC-001&tarefas=TASK-01", b"");
        assert_eq!(reply.status, 422);
        assert!(
            body(&reply).contains("docs/x/saida.txt"),
            "{}",
            body(&reply)
        );
        assert_eq!(read(&painel, TASK), before);
        assert!(!painel.work.join(AVAIS_DIR).exists());
    }

    #[test]
    fn approval_of_a_missing_spec_is_422_without_aval() {
        let (_dir, painel) = project(VALID_EVIDENCE);
        let reply = post(&painel, "aprovacoes", "spec=SPEC-404", b"");
        assert_eq!(reply.status, 422);
        assert!(!painel.work.join(AVAIS_DIR).exists());
    }

    #[test]
    fn approval_with_invalid_ids_is_400() {
        let (_dir, painel) = project(VALID_EVIDENCE);
        for query in [
            "",
            "spec=../x",
            "spec=SPEC-001&tarefas=TASK-01,../x",
            "spec=SPEC-001&tarefas=,",
        ] {
            assert_eq!(
                post(&painel, "aprovacoes", query, b"").status,
                400,
                "{query}"
            );
        }
        assert!(!painel.work.join(AVAIS_DIR).exists());
    }

    // --- pedidos (CT-12, CB-03) ---

    #[test]
    fn request_is_written_in_the_study_format() {
        let (_dir, painel) = project(VALID_EVIDENCE);
        let reply = post(
            &painel,
            "pedidos",
            "acao=ajustar-tarefa&spec=SPEC-001&tarefa=TASK-01",
            "  Acrescentar um critério\n".as_bytes(),
        );
        assert_eq!(reply.status, 200, "{}", body(&reply));
        assert_eq!(body(&reply), "{\"id\":\"PED-001\"}");
        let content = read(&painel, "vostok/work/pedidos/pendente/PED-001.md");
        let header = "---\nid: PED-001\nacao: ajustar-tarefa\nspec: SPEC-001\ntarefa: TASK-01\nestado: pendente\norigem: painel\ncriado_em: ";
        assert!(content.starts_with(header), "{content}");
        assert!(content.contains("\natualizado_em: "), "{content}");
        assert!(
            content.ends_with("---\n\n## Pedido\n\nAcrescentar um critério\n\n## Resultado\n\n"),
            "{content}"
        );
    }

    fn approve_task_and_write_targets(painel: &Painel) {
        let reply = post(painel, "aprovacoes", "spec=SPEC-001&tarefas=TASK-01", b"");
        assert_eq!(reply.status, 200, "{}", body(&reply));
        write(
            painel,
            "vostok/work/plans/SPEC-001.md",
            "---\nid: SPEC-001\ncontent_hash: plan-hash\n---\n",
        );
        write(
            painel,
            "vostok/work/specs/refined/SPEC-001-exemplo.md",
            "---\nid: SPEC-001\ncontent_hash: spec-hash\n---\n",
        );
    }

    fn aval_lines(painel: &Painel, log: &str) -> Vec<String> {
        fs::read_to_string(painel.work.join(AVAIS_DIR).join(log))
            .unwrap_or_default()
            .lines()
            .map(String::from)
            .collect()
    }

    fn last_aval_fields(painel: &Painel, log: &str) -> Vec<String> {
        let lines = aval_lines(painel, log);
        lines
            .last()
            .unwrap()
            .split('\t')
            .skip(1)
            .map(String::from)
            .collect()
    }

    #[test]
    fn approval_aval_has_the_action_column() {
        let (_dir, painel) = project(VALID_EVIDENCE);
        approve_task_and_write_targets(&painel);
        assert!(aval_lines(&painel, "SPEC-001.log")[0].ends_with("\tpainel\taprovar"));
    }

    #[test]
    fn finish_request_records_the_aval() {
        let (_dir, painel) = project(VALID_EVIDENCE);
        approve_task_and_write_targets(&painel);
        let query = "acao=finalizar-tarefa&spec=SPEC-001&tarefa=TASK-01";
        let reply = post(&painel, "pedidos", query, b"");
        assert_eq!(reply.status, 200, "{}", body(&reply));
        let approved_hash = frontmatter_get(&painel.project.join(TASK), "approved_hash").unwrap();
        assert_eq!(
            last_aval_fields(&painel, "SPEC-001.log"),
            [
                "SPEC-001/TASK-01",
                approved_hash.as_str(),
                "painel",
                "finalizar-tarefa"
            ]
        );
        assert!(painel.work.join("pedidos/pendente/PED-001.md").is_file());
    }

    #[test]
    fn finish_request_of_an_unapproved_task_is_422_without_request_or_aval() {
        let (_dir, painel) = project(VALID_EVIDENCE);
        let query = "acao=finalizar-tarefa&spec=SPEC-001&tarefa=TASK-01";
        let reply = post(&painel, "pedidos", query, b"");
        assert_eq!(reply.status, 422);
        assert_eq!(body(&reply), "a tarefa não está aprovada");
        assert!(!painel.work.join("pedidos").exists());
        assert!(!painel.work.join(AVAIS_DIR).exists());
    }

    #[test]
    fn plan_and_spec_requests_record_the_aval() {
        let (_dir, painel) = project(VALID_EVIDENCE);
        approve_task_and_write_targets(&painel);
        let reply = post(&painel, "pedidos", "acao=aprovar-plano&spec=SPEC-001", b"");
        assert_eq!(reply.status, 200, "{}", body(&reply));
        assert_eq!(
            last_aval_fields(&painel, "SPEC-001.log"),
            ["SPEC-001", "plan-hash", "painel", "aprovar-plano"]
        );
        let reply = post(&painel, "pedidos", "acao=finalizar-spec&spec=SPEC-001", b"");
        assert_eq!(reply.status, 200, "{}", body(&reply));
        assert_eq!(
            last_aval_fields(&painel, "SPEC-001.log"),
            ["SPEC-001", "spec-hash", "painel", "finalizar-spec"]
        );
    }

    #[test]
    fn commit_request_records_the_aval() {
        let (_dir, painel) = project(VALID_EVIDENCE);
        let reply = post(
            &painel,
            "pedidos",
            "acao=fazer-commit",
            b"  Commit da onda 1.\n",
        );
        assert_eq!(reply.status, 200, "{}", body(&reply));
        assert_eq!(
            last_aval_fields(&painel, "sem-spec.log"),
            [
                "PED-001",
                sha256_hex(b"Commit da onda 1.").as_str(),
                "painel",
                "fazer-commit"
            ]
        );
    }

    #[test]
    fn unlocked_requests_record_no_aval() {
        let (_dir, painel) = project(VALID_EVIDENCE);
        let query = "acao=ajustar-tarefa&spec=SPEC-001&tarefa=TASK-01";
        assert_eq!(post(&painel, "pedidos", query, b"x").status, 200);
        assert!(!painel.work.join(AVAIS_DIR).exists());
    }

    #[test]
    fn request_actions_are_sixteen() {
        assert_eq!(REQUEST_ACTIONS.len(), 16);
    }

    fn pending(painel: &Painel, id: &str) -> String {
        read(painel, &format!("vostok/work/pedidos/pendente/{id}.md"))
    }

    fn pending_count(painel: &Painel) -> usize {
        fs::read_dir(painel.work.join("pedidos/pendente"))
            .map(|entries| entries.count())
            .unwrap_or(0)
    }

    #[test]
    fn spec_requests_are_written_with_their_fields() {
        let (_dir, painel) = project(VALID_EVIDENCE);
        let reply = post(&painel, "pedidos", "acao=criar-spec", b"Painel de custos");
        assert_eq!(reply.status, 200, "{}", body(&reply));
        let content = pending(&painel, "PED-001");
        assert!(content.contains("\nspec: \n"), "{content}");
        assert!(
            content.contains("## Pedido\n\nPainel de custos\n"),
            "{content}"
        );
        let reply = post(&painel, "pedidos", "acao=refinar-spec&spec=SPEC-001", b"");
        assert_eq!(reply.status, 200, "{}", body(&reply));
        assert!(pending(&painel, "PED-002").contains("\nspec: SPEC-001\n"));
        let query = "acao=executar-onda&spec=SPEC-001&onda=2";
        let reply = post(&painel, "pedidos", query, b"Com cuidado");
        assert_eq!(reply.status, 200, "{}", body(&reply));
        let content = pending(&painel, "PED-003");
        assert!(content.contains("\nspec: SPEC-001\n"), "{content}");
        assert!(
            content.contains("## Pedido\n\nOnda 2.\nCom cuidado\n"),
            "{content}"
        );
    }

    #[test]
    fn spec_requests_without_their_fields_are_400() {
        let (_dir, painel) = project(VALID_EVIDENCE);
        for (query, text_of_user) in [
            ("acao=criar-spec", &b""[..]),
            ("acao=criar-spec", &b"  \n "[..]),
            ("acao=refinar-spec", &b"x"[..]),
            ("acao=executar-onda&spec=SPEC-001", &b"x"[..]),
            ("acao=executar-onda&spec=SPEC-001&onda=0", &b"x"[..]),
            ("acao=executar-onda&spec=SPEC-001&onda=x", &b"x"[..]),
            ("acao=executar-onda&onda=1", &b"x"[..]),
        ] {
            let reply = post(&painel, "pedidos", query, text_of_user);
            assert_eq!(reply.status, 400, "{query}: {}", body(&reply));
        }
        assert_eq!(pending_count(&painel), 0);
    }

    #[test]
    fn commit_release_and_undo_requests_record_the_aval() {
        let (_dir, painel) = project(VALID_EVIDENCE);
        for (index, action) in ["liberar-commits", "desfazer-commits"].iter().enumerate() {
            let reply = post(
                &painel,
                "pedidos",
                &format!("acao={action}"),
                b" Liberar.\n",
            );
            assert_eq!(reply.status, 200, "{}", body(&reply));
            assert_eq!(
                last_aval_fields(&painel, "sem-spec.log"),
                [
                    format!("PED-{:03}", index + 1).as_str(),
                    sha256_hex(b"Liberar.").as_str(),
                    "painel",
                    action
                ]
            );
        }
    }

    const PRINT_EVIDENCE: &str = "- `tela.png` (print): `true`";
    const PRINT_FILE: &str = "vostok/work/tasks/SPEC-001/evidence/TASK-01/tela.png";

    fn running_print_task() -> (TempDir, Painel) {
        let (dir, painel) = project(PRINT_EVIDENCE);
        let content = read(&painel, TASK).replace("status: pending", "status: in_progress");
        write(&painel, TASK, &content);
        write(&painel, PRINT_FILE, "PNG");
        (dir, painel)
    }

    fn review(painel: &Painel, print: &str, result: &str) -> Reply {
        post(
            painel,
            "conferencias",
            &format!("spec=SPEC-001&tarefa=TASK-01&print={print}&resultado={result}"),
            b"",
        )
    }

    #[test]
    fn print_review_accepted_records_the_panel_origin() {
        let (_dir, painel) = running_print_task();
        let reply = review(&painel, "tela.png", "aceito");
        assert_eq!(reply.status, 200, "{}", body(&reply));
        let content = read(&painel, TASK);
        assert!(
            content.contains(
                "| conferência | painel | print tela.png conferido pelo usuário (sha256 "
            ),
            "{content}"
        );
    }

    #[test]
    fn print_review_rejected_blocks_the_task() {
        let (_dir, painel) = running_print_task();
        let reply = review(&painel, "tela.png", "rejeitado");
        assert_eq!(reply.status, 200, "{}", body(&reply));
        let content = read(&painel, TASK);
        assert!(content.contains("\nstatus: blocked\n"), "{content}");
        assert!(content.contains("\nevidence_result: failed\n"), "{content}");
        assert!(
            content.contains(
                "| conferência | rejeitado | print tela.png rejeitado no painel (sha256 "
            ),
            "{content}"
        );
    }

    #[test]
    fn print_review_of_an_unknown_print_or_idle_task_is_422() {
        let (_dir, painel) = running_print_task();
        let before = read(&painel, TASK);
        let reply = review(&painel, "outra.png", "aceito");
        assert_eq!(reply.status, 422);
        assert!(
            body(&reply).contains("não é um print desta tarefa"),
            "{}",
            body(&reply)
        );
        assert_eq!(review(&painel, "tela.png", "talvez").status, 400);
        for print in ["../tela.png", "a/tela.png", ""] {
            assert_eq!(review(&painel, print, "aceito").status, 400, "{print}");
        }
        assert_eq!(read(&painel, TASK), before);

        let (_dir, painel) = project(PRINT_EVIDENCE);
        write(&painel, PRINT_FILE, "PNG");
        let before = read(&painel, TASK);
        assert_eq!(review(&painel, "tela.png", "aceito").status, 422);
        assert_eq!(read(&painel, TASK), before);
    }

    #[test]
    fn prune_route_lists_without_deleting_and_deletes_with_confirmation() {
        let (_dir, painel) = project(VALID_EVIDENCE);
        assert_eq!(
            body(&post(&painel, "limpeza", "", b"")),
            "{\"arquivos\":[]}"
        );
        assert_eq!(
            body(&post(&painel, "limpeza", "confirmar=sim", b"")),
            "{\"apagados\":[]}"
        );
        let archived = "vostok/work/.archive/SPEC-009/pedidos/PED-001.md";
        write(&painel, archived, "---\nid: PED-001\nspec: SPEC-009\n---\n");
        write(
            &painel,
            "vostok/work/avais/SPEC-009.log",
            "d\tSPEC-009\th\tpainel\n",
        );
        let listed = post(&painel, "limpeza", "", b"");
        assert_eq!(listed.status, 200);
        assert_eq!(body(&listed), format!("{{\"arquivos\":[\"{archived}\"]}}"));
        assert!(painel.project.join(archived).is_file());
        let deleted = post(&painel, "limpeza", "confirmar=sim", b"");
        assert_eq!(deleted.status, 200);
        assert_eq!(body(&deleted), format!("{{\"apagados\":[\"{archived}\"]}}"));
        assert!(!painel.project.join(archived).exists());
        assert!(painel.work.join("avais/SPEC-009.log").is_file());
    }

    const QUESTION: &str = "vostok/work/perguntas/aberta/P-001.md";
    const ANSWERED_QUESTION: &str = "vostok/work/perguntas/respondida/P-001.md";

    fn with_question() -> (TempDir, Painel) {
        let (dir, painel) = project(VALID_EVIDENCE);
        write(
            &painel,
            QUESTION,
            "---\nid: P-001\npedido: PED-004\nspec: SPEC-001\ntask: TASK-01\nstatus: aberta\ncreated_at: 2026-11-03T10:00:00Z\n---\n\n## Pergunta\n\n# 1. Qual caminho?\n",
        );
        (dir, painel)
    }

    fn answer(painel: &Painel, question: &str, text_of_user: &[u8]) -> Reply {
        post(
            painel,
            "pedidos",
            &format!("acao=responder-pergunta&pergunta={question}&spec=SPEC-009"),
            text_of_user,
        )
    }

    #[test]
    fn answer_moves_the_question_and_creates_the_reply_request() {
        let (_dir, painel) = with_question();
        let reply = answer(&painel, "P-001", b"Op\xc3\xa7\xc3\xa3o A");
        assert_eq!(reply.status, 200, "{}", body(&reply));
        assert!(!painel.project.join(QUESTION).exists());
        let question = read(&painel, ANSWERED_QUESTION);
        assert!(question.contains("\nstatus: respondida\n"), "{question}");
        assert!(question.contains("\nanswered_at: "), "{question}");
        assert!(
            question.ends_with("\n## Resposta\n\nOpção A\n"),
            "{question}"
        );
        let request = pending(&painel, "PED-001");
        for expected in [
            "\nacao: responder-pergunta\n",
            "\nspec: SPEC-001\n",
            "\ntarefa: TASK-01\n",
            "## Pedido\n\nResposta à pergunta P-001 do pedido PED-004.\n\nOpção A\n",
        ] {
            assert!(request.contains(expected), "{expected}: {request}");
        }
    }

    #[test]
    fn second_answer_is_409_and_keeps_the_first() {
        let (_dir, painel) = with_question();
        assert_eq!(answer(&painel, "P-001", "Opção A".as_bytes()).status, 200);
        let reply = answer(&painel, "P-001", b"Outra");
        assert_eq!(reply.status, 409, "{}", body(&reply));
        assert_eq!(pending_count(&painel), 1);
        let question = read(&painel, ANSWERED_QUESTION);
        assert!(
            question.contains("Opção A") && !question.contains("Outra"),
            "{question}"
        );
    }

    #[test]
    fn answer_without_question_or_text_is_400_and_missing_is_404() {
        let (_dir, painel) = with_question();
        let missing = post(&painel, "pedidos", "acao=responder-pergunta", b"x");
        assert_eq!(missing.status, 400);
        assert_eq!(answer(&painel, "../x", b"x").status, 400);
        assert_eq!(answer(&painel, "P-001", b"  ").status, 400);
        assert_eq!(answer(&painel, "P-099", b"x").status, 404);
        assert_eq!(pending_count(&painel), 0);
        assert!(painel.project.join(QUESTION).is_file());
    }

    #[test]
    fn every_accepted_action_is_written_with_the_next_number() {
        let (_dir, painel) = project(VALID_EVIDENCE);
        approve_task_and_write_targets(&painel);
        let actions = REQUEST_ACTIONS
            .iter()
            .filter(|action| **action != "responder-pergunta");
        for (index, action) in actions.enumerate() {
            let reply = post(
                &painel,
                "pedidos",
                &format!("acao={action}&spec=SPEC-001&tarefa=TASK-01&onda=1"),
                b"x",
            );
            assert_eq!(reply.status, 200, "{action}: {}", body(&reply));
            let id = format!("PED-{:03}", index + 1);
            let content = read(&painel, &format!("vostok/work/pedidos/pendente/{id}.md"));
            assert!(
                content.contains(&format!("\nacao: {action}\n")),
                "{content}"
            );
        }
        assert_eq!(REQUEST_ACTIONS.len(), 16);
    }

    #[test]
    fn unknown_or_approval_actions_are_400_without_writing() {
        let (_dir, painel) = project(VALID_EVIDENCE);
        for action in [
            "",
            "apagar-tudo",
            "aprovar-tarefa",
            "aprovar-spec",
            "ajustar-tarefa%0Aestado",
        ] {
            let reply = post(
                &painel,
                "pedidos",
                &format!("acao={action}&spec=SPEC-001"),
                b"x",
            );
            assert_eq!(reply.status, 400, "{action}");
        }
        assert!(!painel.work.join("pedidos").exists());
    }

    #[test]
    fn request_with_invalid_spec_or_task_is_400() {
        let (_dir, painel) = project(VALID_EVIDENCE);
        for query in [
            "acao=ajustar-tarefa&spec=../x",
            "acao=ajustar-tarefa&spec=SPEC-001&tarefa=TASK-01%0Aestado: feito",
        ] {
            assert_eq!(post(&painel, "pedidos", query, b"x").status, 400, "{query}");
        }
    }

    #[test]
    fn next_request_id_follows_the_highest_of_every_folder() {
        let (_dir, painel) = project(VALID_EVIDENCE);
        write(&painel, "vostok/work/pedidos/pendente/PED-002.md", "x");
        write(&painel, "vostok/work/pedidos/feito/PED-010.md", "x");
        write(&painel, "vostok/work/pedidos/notas.md", "x");
        let reply = post(
            &painel,
            "pedidos",
            "acao=ajustar-tarefa&spec=SPEC-001&tarefa=TASK-01",
            b"x",
        );
        assert_eq!(reply.status, 200);
        assert!(String::from_utf8_lossy(&reply.body).contains("PED-011"));
        assert!(painel.work.join("pedidos/pendente/PED-011.md").is_file());
        assert!(painel.work.join("pedidos/notas.md").is_file());
    }

    #[test]
    fn unknown_route_or_method_is_404() {
        let (_dir, painel) = project(VALID_EVIDENCE);
        assert_eq!(
            act(&painel, "POST", &["api", "arquivo"], "", b"").status,
            404
        );
        assert_eq!(
            act(&painel, "PUT", &["api", "pedidos"], "", b"").status,
            404
        );
        assert_eq!(act(&painel, "POST", &["api", "nada"], "", b"").status, 404);
    }
}
