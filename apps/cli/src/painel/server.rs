//! Servidor HTTP do painel: escuta só em `127.0.0.1`, serve o bundle de `apps/painel` com o token da sessão, despacha os GET para a leitura e o aviso ao vivo e aceita POST só nas três rotas de conferência de evidências (DEC-006); os demais métodos e rotas respondem 405.

use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;

use anyhow::{anyhow, Result};

use super::{json, json_string, live, not_found, read, safe_segment, text, Painel, Reply, Session};
use crate::commands::task::{
    review_evidence, ReviewAction, ReviewError, ReviewMark, ReviewOrigin, ReviewOutcome,
    ReviewRequest,
};

/// Se o bundle de `apps/painel` foi embutido neste binário.
pub const HAS_BUNDLE: bool = cfg!(painel_bundle);

/// Resposta da página quando o binário não tem o bundle (FA-02).
pub const BUNDLE_MISSING: &str = "Este vostok foi compilado sem o bundle do painel. Para obtê-lo, \
gere o bundle com `pnpm --dir apps/painel install --frozen-lockfile && pnpm --dir apps/painel build` \
(Node e pnpm no PATH) e compile de novo com `cargo install --path apps/cli --features painel`.";

const LIVE_PATH: &str = "/api/ao-vivo";

/// Maior corpo aceito num POST de conferência.
const MAX_BODY: usize = 16 * 1024;

/// Uma conferência por vez, para duas abas não gravarem o mesmo arquivo juntas.
static REVIEW_LOCK: Mutex<()> = Mutex::new(());

#[cfg(painel_bundle)]
static DIST: include_dir::Dir<'static> =
    include_dir::include_dir!("$CARGO_MANIFEST_DIR/../painel/dist");

/// De onde veio a porta, para a mensagem de porta ocupada apontar o que mudar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortOrigin {
    Argument,
    Config,
}

/// Escuta em `127.0.0.1:<port>` (0 escolhe uma porta livre), anuncia a URL e atende cada
/// requisição numa thread: as leituras são curtas e o painel é usado por uma pessoa.
pub fn serve(
    painel: Painel,
    port: u16,
    origin: PortOrigin,
    announce: impl FnOnce(String),
) -> Result<()> {
    let server = tiny_http::Server::http(("127.0.0.1", port))
        .map_err(|error| listen_error(port, origin, error))?;
    let address = server
        .server_addr()
        .to_ip()
        .ok_or_else(|| anyhow!("endereço de escuta inválido"))?;
    announce(format!("http://{address}/"));
    let painel = Arc::new(painel);
    let session = Arc::new(Session::new(address.port()));
    for request in server.incoming_requests() {
        let painel = Arc::clone(&painel);
        let session = Arc::clone(&session);
        thread::spawn(move || handle(&painel, &session, request));
    }
    Ok(())
}

fn listen_error(
    port: u16,
    origin: PortOrigin,
    error: Box<dyn std::error::Error + Send + Sync>,
) -> anyhow::Error {
    match error.downcast_ref::<io::Error>() {
        Some(io_error) if io_error.kind() == io::ErrorKind::AddrInUse => match origin {
            PortOrigin::Argument => anyhow!(
                "a porta {port} está ocupada; escolha outra com --porta ou use --porta 0 para uma livre"
            ),
            PortOrigin::Config => anyhow!(
                "a porta {port} de panel_port (config/main.yml) está ocupada; mude panel_port ou use --porta 0 para uma livre"
            ),
        },
        _ => anyhow!("não foi possível escutar em 127.0.0.1:{port}: {error}"),
    }
}

fn handle(painel: &Painel, session: &Session, mut request: tiny_http::Request) {
    let method = request.method().as_str().to_string();
    let url = request.url().to_string();
    if method == "POST" {
        let headers = PostHeaders::of(&request);
        let mut body = Vec::new();
        let read = request
            .as_reader()
            .take(MAX_BODY as u64 + 1)
            .read_to_end(&mut body);
        let reply = if read.is_err() || body.len() > MAX_BODY {
            text(400, "corpo grande demais ou ilegível; nada foi registrado")
        } else {
            review_post(painel, session, &url, &headers, &body)
        };
        let _ = request.respond(to_response(reply));
        return;
    }
    if method == "GET" {
        if url.split('?').next() == Some(LIVE_PATH) {
            live::handle(painel, request);
            return;
        }
        let (path, query) = url.split_once('?').unwrap_or((url.as_str(), ""));
        let parts: Vec<&str> = path.trim_matches('/').split('/').collect();
        if let Some(found) = read::file_delivery(painel, &parts, query) {
            respond_file(request, found);
            return;
        }
    }
    let _ = request.respond(to_response(dispatch(painel, session, &method, &url)));
}

/// Cabeçalhos que decidem se um POST vem da página do próprio painel.
pub struct PostHeaders {
    pub host: Option<String>,
    pub origin: Option<String>,
    pub token: Option<String>,
}

impl PostHeaders {
    fn of(request: &tiny_http::Request) -> Self {
        let find = |name: &'static str| {
            request
                .headers()
                .iter()
                .find(|header| header.field.equiv(name))
                .map(|header| header.value.as_str().to_string())
        };
        Self {
            host: find("Host"),
            origin: find("Origin"),
            token: find("X-Vostok-Token"),
        }
    }
}

/// POST de conferência: só `/api/tarefas/<SPEC>/<TASK>/evidencias/<EV-NN>/aceitar|recusar|retirar`,
/// com o token da sessão e `Host` e `Origin` do próprio painel; grava pela mesma função do
/// `vostok task evidence`, com a origem `painel`.
pub fn review_post(
    painel: &Painel,
    session: &Session,
    url: &str,
    headers: &PostHeaders,
    body: &[u8],
) -> Reply {
    let path = url.split('?').next().unwrap_or_default();
    let parts: Vec<&str> = path.trim_matches('/').split('/').collect();
    let (spec, task, evidence, action) = match parts.as_slice() {
        ["api", "tarefas", spec, task, "evidencias", evidence, action] => {
            let action = match *action {
                "aceitar" => ReviewAction::Accept,
                "recusar" => ReviewAction::Reject,
                "retirar" => ReviewAction::Withdraw,
                _ => return text(405, "método não permitido"),
            };
            (*spec, *task, *evidence, action)
        }
        _ => return text(405, "método não permitido"),
    };
    if !trusted_request(session, headers) {
        return text(403, "token ou origem recusados; nada foi registrado");
    }
    let evidence_id = evidence
        .strip_prefix("EV-")
        .is_some_and(|digits| !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit()));
    if safe_segment(spec).is_err() || safe_segment(task).is_err() || !evidence_id {
        return not_found();
    }
    let Some((review_text, expected_hash)) = review_body(body) else {
        return text(
            400,
            "corpo inválido; envie {\"texto\": \"...\"}; nada foi registrado",
        );
    };
    let request = ReviewRequest {
        spec,
        task,
        evidence,
        action,
        text: &review_text,
        origin: ReviewOrigin::Painel,
        expected_hash: expected_hash.as_deref(),
    };
    let result = {
        let _guard = REVIEW_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        review_evidence(&painel.root, &request)
    };
    match result {
        Ok(outcome) => json(review_json(&outcome)),
        Err(ReviewError::MissingText(_)) => text(400, "informe o texto; nada foi registrado"),
        Err(error @ (ReviewError::TaskNotFound(_) | ReviewError::UnknownEvidence(_))) => {
            text(404, &error.to_string())
        }
        Err(
            error @ (ReviewError::NotReviewable(_)
            | ReviewError::Changed(_)
            | ReviewError::Closed(_)
            | ReviewError::NothingToWithdraw(_)),
        ) => text(409, &error.to_string()),
        Err(error @ ReviewError::Other(_)) => text(500, &error.to_string()),
    }
}

/// `texto` e `sha256` do corpo JSON; corpo vazio vale `{}`. `None` para corpo que não é um objeto
/// ou com um dos dois campos fora de texto.
fn review_body(body: &[u8]) -> Option<(String, Option<String>)> {
    if body.iter().all(u8::is_ascii_whitespace) {
        return Some((String::new(), None));
    }
    let value: serde_json::Value = serde_json::from_slice(body).ok()?;
    let object = value.as_object()?;
    let field = |name: &str| match object.get(name) {
        None => Some(None),
        Some(serde_json::Value::String(value)) => Some(Some(value.clone())),
        Some(_) => None,
    };
    let review_text = field("texto")?.unwrap_or_default();
    let expected_hash = field("sha256")?;
    Some((review_text, expected_hash))
}

fn trusted_request(session: &Session, headers: &PostHeaders) -> bool {
    let port = session.port;
    let token = headers
        .token
        .as_deref()
        .is_some_and(|given| same_token(given, &session.token));
    let host = headers.host.as_deref().is_some_and(|host| {
        host == format!("127.0.0.1:{port}") || host == format!("localhost:{port}")
    });
    let origin = headers.origin.as_deref().is_some_and(|origin| {
        origin == format!("http://127.0.0.1:{port}") || origin == format!("http://localhost:{port}")
    });
    token && host && origin
}

/// Compara os tokens sem sair no primeiro byte diferente, para o tempo não revelar o prefixo.
fn same_token(given: &str, expected: &str) -> bool {
    given.len() == expected.len()
        && given
            .bytes()
            .zip(expected.bytes())
            .fold(0u8, |diff, (x, y)| diff | (x ^ y))
            == 0
}

fn review_json(outcome: &ReviewOutcome) -> String {
    let record = &outcome.record;
    let status = match record.mark {
        ReviewMark::Accepted => "conferida",
        ReviewMark::Rejected => "recusada",
        ReviewMark::Withdrawn => "a conferir",
    };
    format!(
        "{{\"id\":{},\"name\":{},\"status\":{},\"reason\":{},\"reviewed_at\":{},\"origin\":{},\"task_status\":{}}}",
        json_string(&record.id),
        json_string(&record.name),
        json_string(status),
        json_string(&record.text),
        json_string(&record.date),
        json_string(&record.origin),
        json_string(&outcome.task_status),
    )
}

/// Interpreta `Range: bytes=...` de um só intervalo (início-fim, início- ou -final) sobre um
/// arquivo de `size` bytes; o fim é limitado ao último byte. `None` para o que não é atendível.
fn byte_range(value: &str, size: u64) -> Option<(u64, u64)> {
    let spec = value.strip_prefix("bytes=")?;
    if size == 0 || spec.contains(',') {
        return None;
    }
    let (first, last) = spec.split_once('-')?;
    let (start, end) = match (first, last) {
        ("", "") => return None,
        ("", suffix) => {
            let length: u64 = suffix.parse().ok().filter(|length| *length > 0)?;
            (size.saturating_sub(length), size - 1)
        }
        (start, "") => (start.parse().ok()?, size - 1),
        (start, end) => (start.parse().ok()?, end.parse::<u64>().ok()?.min(size - 1)),
    };
    (start <= end && start < size).then_some((start, end))
}

fn range_header(request: &tiny_http::Request) -> Option<String> {
    request
        .headers()
        .iter()
        .find(|header| header.field.equiv("Range"))
        .map(|header| header.value.as_str().to_string())
}

fn header(name: &str, value: &str) -> tiny_http::Header {
    tiny_http::Header::from_bytes(name.as_bytes(), value.as_bytes()).expect("cabeçalho válido")
}

fn unreadable_reply() -> Reply {
    let mut reply = text(500, "arquivo ilegível");
    reply.headers = vec![("X-Content-Type-Options", "nosniff".to_string())];
    reply
}

/// Entrega um arquivo por partes, sem lê-lo na memória: sem `Range` o corpo inteiro (200), com
/// `Range` válido o trecho (206) e com `Range` inválido ou além do fim o 416 com o tamanho.
fn respond_file(request: tiny_http::Request, found: Result<read::FileDelivery, Reply>) {
    let delivery = match found {
        Ok(delivery) => delivery,
        Err(reply) => {
            let _ = request.respond(to_response(reply));
            return;
        }
    };
    let opened = File::open(&delivery.path).and_then(|file| Ok((file.metadata()?.len(), file)));
    let Ok((size, mut file)) = opened else {
        let mut reply = not_found();
        reply.headers.extend(delivery.headers);
        let _ = request.respond(to_response(reply));
        return;
    };
    let mut headers = vec![
        header("Content-Type", delivery.content_type),
        header("Cache-Control", "no-store"),
        header("Accept-Ranges", "bytes"),
    ];
    for (name, value) in &delivery.headers {
        headers.push(header(name, value));
    }
    let (status, start, length) = match range_header(&request) {
        None => (200, 0, size),
        Some(value) => match byte_range(&value, size) {
            Some((start, end)) => {
                headers.push(header(
                    "Content-Range",
                    &format!("bytes {start}-{end}/{size}"),
                ));
                (206, start, end - start + 1)
            }
            None => {
                let mut reply = text(416, "intervalo inválido");
                reply.headers = vec![
                    ("Content-Range", format!("bytes */{size}")),
                    ("Accept-Ranges", "bytes".to_string()),
                    ("X-Content-Type-Options", "nosniff".to_string()),
                ];
                let _ = request.respond(to_response(reply));
                return;
            }
        },
    };
    if file.seek(SeekFrom::Start(start)).is_err() {
        let _ = request.respond(to_response(unreadable_reply()));
        return;
    }
    let response = tiny_http::Response::new(
        tiny_http::StatusCode(status),
        headers,
        file.take(length),
        Some(length as usize),
        None,
    )
    .with_chunked_threshold(usize::MAX);
    let _ = request.respond(response);
}

/// Roteia uma requisição já lida: `GET /api/...` e `GET /arquivos/...` para a leitura, os demais
/// `GET` para o bundle, com o token da sessão no `index.html`, e qualquer outro método para o 405.
/// O POST das três rotas de conferência é tratado antes, por `review_post`.
pub fn dispatch(painel: &Painel, session: &Session, method: &str, url: &str) -> Reply {
    let (path, query) = url.split_once('?').unwrap_or((url, ""));
    let parts: Vec<&str> = path.trim_matches('/').split('/').collect();
    match method {
        "GET" if matches!(parts.first(), Some(&"api" | &"arquivos")) => {
            read::route(painel, &parts, query).unwrap_or_else(not_found)
        }
        "GET" => asset(path, &session.token),
        _ => text(405, "método não permitido"),
    }
}

/// Arquivo do bundle; rotas do app (sem extensão) recebem o `index.html`, com o token da sessão.
fn asset(path: &str, token: &str) -> Reply {
    let name = bundle_name(path);
    let mut reply = page(name, bundle_file(name));
    if name == "index.html" && reply.status == 200 {
        reply.body = with_token(&reply.body, token);
    }
    reply
}

/// Põe `<meta name="vostok-token">` antes do primeiro `</head>`; sem `</head>`, nada muda.
fn with_token(html: &[u8], token: &str) -> Vec<u8> {
    let Some(position) = html.windows(7).position(|window| window == b"</head>") else {
        return html.to_vec();
    };
    let meta = format!("<meta name=\"vostok-token\" content=\"{token}\">");
    let mut out = Vec::with_capacity(html.len() + meta.len());
    out.extend_from_slice(&html[..position]);
    out.extend_from_slice(meta.as_bytes());
    out.extend_from_slice(&html[position..]);
    out
}

fn bundle_name(path: &str) -> &str {
    let name = path.trim_start_matches('/');
    if name.is_empty() || Path::new(name).extension().is_none() {
        "index.html"
    } else {
        name
    }
}

#[cfg(painel_bundle)]
fn bundle_file(name: &str) -> Option<&'static [u8]> {
    DIST.get_file(name).map(|file| file.contents())
}

#[cfg(not(painel_bundle))]
fn bundle_file(_name: &str) -> Option<&'static [u8]> {
    None
}

fn page(name: &str, file: Option<&[u8]>) -> Reply {
    match file {
        Some(contents) => Reply {
            status: 200,
            content_type: content_type(name),
            body: contents.to_vec(),
            headers: Vec::new(),
        },
        None if !HAS_BUNDLE => text(503, BUNDLE_MISSING),
        None => not_found(),
    }
}

fn content_type(name: &str) -> &'static str {
    match Path::new(name).extension().and_then(|ext| ext.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("js") => "text/javascript; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("json") => "application/json; charset=utf-8",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("webp") => "image/webp",
        Some("gif") => "image/gif",
        Some("ico") => "image/x-icon",
        Some("woff2") => "font/woff2",
        Some("ttf") => "font/ttf",
        _ => "application/octet-stream",
    }
}

fn to_response(reply: Reply) -> tiny_http::Response<io::Cursor<Vec<u8>>> {
    let content_type = tiny_http::Header::from_bytes(&b"Content-Type"[..], reply.content_type)
        .expect("content type válido");
    let cache = tiny_http::Header::from_bytes(&b"Cache-Control"[..], &b"no-store"[..])
        .expect("cabeçalho válido");
    let mut response = tiny_http::Response::from_data(reply.body)
        .with_status_code(reply.status)
        .with_header(content_type)
        .with_header(cache);
    for (name, value) in reply.headers {
        response.add_header(
            tiny_http::Header::from_bytes(name.as_bytes(), value.as_bytes())
                .expect("cabeçalho válido"),
        );
    }
    response
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use super::*;

    fn session() -> Session {
        Session {
            token: "t".to_string(),
            port: 1,
        }
    }

    fn painel() -> Painel {
        Painel {
            project: PathBuf::from("/projeto-inexistente"),
            root: PathBuf::from("/projeto-inexistente/vostok"),
            work: PathBuf::from("/projeto-inexistente/vostok/work"),
        }
    }

    #[test]
    fn ct04_falha_de_seek_responde_500_com_nosniff() {
        let reply = unreadable_reply();
        assert_eq!(reply.status, 500);
        assert!(reply
            .headers
            .contains(&("X-Content-Type-Options", "nosniff".to_string())));
    }

    #[test]
    fn post_put_e_delete_sao_405() {
        for path in ["/api/arquivo", "/api/evidencias", "/api/pedidos/anexos"] {
            for method in ["POST", "PUT", "DELETE", "PATCH"] {
                let reply = dispatch(&painel(), &session(), method, path);
                assert_eq!(reply.status, 405, "{method} {path}");
            }
        }
    }

    #[test]
    fn arquivos_get_goes_to_the_reading_not_to_the_bundle() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("vostok");
        fs::create_dir_all(root.join("work/docs")).unwrap();
        fs::write(root.join("work/docs/executivo.html"), "<p>executivo</p>").unwrap();
        let painel = Painel {
            project: dir.path().to_path_buf(),
            work: root.join("work"),
            root,
        };
        let reply = dispatch(
            &painel,
            &session(),
            "GET",
            "/arquivos/vostok/work/docs/executivo.html",
        );
        assert_eq!(reply.status, 200);
        assert_eq!(reply.body, b"<p>executivo</p>");
    }

    #[test]
    fn byte_range_aceita_inicio_fim_aberto_e_final() {
        assert_eq!(byte_range("bytes=0-99", 1000), Some((0, 99)));
        assert_eq!(byte_range("bytes=900-", 1000), Some((900, 999)));
        assert_eq!(byte_range("bytes=-10", 1000), Some((990, 999)));
        assert_eq!(byte_range("bytes=0-5000", 1000), Some((0, 999)));
    }

    #[test]
    fn byte_range_recusa_invalido_ou_alem_do_fim() {
        for value in [
            "abc",
            "bytes=",
            "bytes=-",
            "bytes=-0",
            "bytes=x-y",
            "bytes=5-2",
            "bytes=1000-",
            "bytes=0-1,3-4",
            "items=0-1",
        ] {
            assert_eq!(byte_range(value, 1000), None, "{value:?}");
        }
        assert_eq!(byte_range("bytes=0-0", 0), None);
    }

    #[test]
    fn atividades_sao_404() {
        let reply = dispatch(&painel(), &session(), "GET", "/api/atividades?limite=5");
        assert_eq!(reply.status, 404);
    }

    #[test]
    fn unknown_api_route_is_404() {
        for url in ["/api/nada", "/api/rota-desconhecida/x?y=1"] {
            assert_eq!(
                dispatch(&painel(), &session(), "GET", url).status,
                404,
                "{url}"
            );
        }
    }

    #[test]
    fn other_methods_are_405() {
        for method in ["DELETE", "PATCH", "OPTIONS"] {
            let reply = dispatch(&painel(), &session(), method, "/api/arquivo");
            assert_eq!(reply.status, 405, "{method}");
        }
    }

    #[test]
    fn index_html_leva_o_token_da_sessao() {
        assert_eq!(
            with_token(
                b"<html><head><title>Painel</title></head><body></body></html>",
                "abc"
            ),
            b"<html><head><title>Painel</title><meta name=\"vostok-token\" content=\"abc\"></head><body></body></html>"
        );
        assert_eq!(with_token(b"sem head", "abc"), b"sem head");
    }

    #[test]
    fn other_bundle_files_go_unchanged() {
        let reply = page("assets/app.js", Some(b"</head>"));
        assert_eq!(reply.body, b"</head>");
        assert_eq!(reply.content_type, "text/javascript; charset=utf-8");
    }

    #[test]
    fn app_routes_fall_back_to_index_html() {
        assert_eq!(bundle_name("/"), "index.html");
        assert_eq!(bundle_name("/specs/SPEC-001"), "index.html");
        assert_eq!(bundle_name("/assets/app.js"), "assets/app.js");
    }

    #[cfg(not(painel_bundle))]
    #[test]
    fn page_without_bundle_explains_how_to_build_it() {
        let reply = dispatch(&painel(), &session(), "GET", "/");
        assert_eq!(reply.status, 503);
        assert!(String::from_utf8(reply.body)
            .unwrap()
            .contains("pnpm --dir apps/painel"));
    }

    #[cfg(painel_bundle)]
    #[test]
    fn served_index_leva_o_token_da_sessao() {
        let reply = dispatch(&painel(), &session(), "GET", "/");
        assert_eq!(reply.status, 200);
        assert!(String::from_utf8(reply.body)
            .unwrap()
            .contains("<meta name=\"vostok-token\" content=\"t\">"));
    }
}
