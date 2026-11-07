//! Servidor HTTP do painel: escuta só em `127.0.0.1`, serve o bundle de `apps/painel` com o token
//! de sessão na página e despacha a API para a leitura, o aviso ao vivo e as ações.

use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;
use std::sync::Arc;
use std::thread;

use anyhow::{anyhow, Result};

use crate::commands::task::{self, AttachRefusal};

use super::{actions, json, json_string, live, not_found, query_value, read, text, Painel, Reply};

/// Se o bundle de `apps/painel` foi embutido neste binário.
pub const HAS_BUNDLE: bool = cfg!(painel_bundle);

/// Resposta da página quando o binário não tem o bundle (FA-02).
pub const BUNDLE_MISSING: &str = "Este vostok foi compilado sem o bundle do painel. Para obtê-lo, \
gere o bundle com `pnpm --dir apps/painel install --frozen-lockfile && pnpm --dir apps/painel build` \
(Node e pnpm no PATH) e compile de novo com `cargo install --path apps/cli --features painel`.";

/// Maior corpo aceito numa ação (um `.md` editado ou o texto de um pedido).
const MAX_BODY: u64 = 2 * 1024 * 1024;

const TOKEN_HEADER: &str = "X-Painel-Token";
const LIVE_PATH: &str = "/api/ao-vivo";
const EVIDENCE_UPLOAD_PATH: &str = "/api/evidencias";
const ATTACHMENT_UPLOAD_PATH: &str = "/api/pedidos/anexos";

#[cfg(painel_bundle)]
static DIST: include_dir::Dir<'static> =
    include_dir::include_dir!("$CARGO_MANIFEST_DIR/../painel/dist");

/// Escuta em `127.0.0.1:<port>` (0 escolhe uma porta livre), anuncia a URL e atende cada
/// requisição numa thread: as leituras são curtas e o painel é usado por uma pessoa.
pub fn serve(painel: Painel, port: u16, announce: impl FnOnce(String)) -> Result<()> {
    let server =
        tiny_http::Server::http(("127.0.0.1", port)).map_err(|error| listen_error(port, error))?;
    let address = server
        .server_addr()
        .to_ip()
        .ok_or_else(|| anyhow!("endereço de escuta inválido"))?;
    announce(format!("http://{address}/"));
    let painel = Arc::new(painel);
    for request in server.incoming_requests() {
        let painel = Arc::clone(&painel);
        thread::spawn(move || handle(&painel, request));
    }
    Ok(())
}

fn listen_error(port: u16, error: Box<dyn std::error::Error + Send + Sync>) -> anyhow::Error {
    match error.downcast_ref::<io::Error>() {
        Some(io_error) if io_error.kind() == io::ErrorKind::AddrInUse => anyhow!(
            "a porta {port} está ocupada; escolha outra com --porta ou use --porta 0 para uma livre"
        ),
        _ => anyhow!("não foi possível escutar em 127.0.0.1:{port}: {error}"),
    }
}

fn handle(painel: &Painel, mut request: tiny_http::Request) {
    let method = request.method().as_str().to_string();
    let url = request.url().to_string();
    if method == "GET" && url.split('?').next() == Some(LIVE_PATH) {
        live::handle(painel, request);
        return;
    }
    if method == "GET" {
        let (path, query) = url.split_once('?').unwrap_or((url.as_str(), ""));
        let parts: Vec<&str> = path.trim_matches('/').split('/').collect();
        if let Some(found) = read::file_delivery(painel, &parts, query) {
            respond_file(request, found);
            return;
        }
    }
    let token = token_header(&request);
    let authorized = token.as_deref() == Some(painel.token.as_str());
    if method == "PUT" && authorized && url.split('?').next() == Some(EVIDENCE_UPLOAD_PATH) {
        let query = url.split_once('?').map_or("", |(_, query)| query);
        let reply = upload_evidence(painel, query, &mut request);
        let _ = request.respond(to_response(reply));
        return;
    }
    if method == "PUT" && url.split('?').next() == Some(ATTACHMENT_UPLOAD_PATH) {
        let reply = if authorized {
            upload_attachment(painel, &url, &mut request)
        } else {
            text(403, "token de sessão ausente ou inválido")
        };
        let _ = request.respond(to_response(reply));
        return;
    }
    // O corpo só é lido de quem mostrou o token; os demais recebem o 403 do despacho.
    let body = if is_action(&method) && authorized {
        read_body(&mut request)
    } else {
        Ok(Vec::new())
    };
    let reply = match body {
        Ok(body) => dispatch(painel, &method, &url, token.as_deref(), &body),
        Err(reply) => reply,
    };
    let _ = request.respond(to_response(reply));
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
        let _ = request.respond(to_response(text(500, "arquivo ilegível")));
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

/// Roteia uma requisição já lida: `GET /api/...` e `GET /arquivos/...` para a leitura, POST e PUT
/// com o token para as ações (403 sem ele, RF-08) e os demais `GET` para o bundle.
pub fn dispatch(
    painel: &Painel,
    method: &str,
    url: &str,
    token: Option<&str>,
    body: &[u8],
) -> Reply {
    let (path, query) = url.split_once('?').unwrap_or((url, ""));
    let parts: Vec<&str> = path.trim_matches('/').split('/').collect();
    match method {
        "GET" if matches!(parts.first(), Some(&"api" | &"arquivos")) => {
            read::route(painel, &parts, query).unwrap_or_else(not_found)
        }
        "GET" => asset(painel, path),
        "POST" | "PUT" if token == Some(painel.token.as_str()) => {
            actions::act(painel, method, &parts, query, body)
        }
        "POST" | "PUT" => text(403, "token de sessão ausente ou inválido"),
        _ => text(405, "método não permitido"),
    }
}

/// Recebe o anexo por partes, sem o limite de 2 MB das ações: o tamanho declarado é conferido
/// antes de ler qualquer byte do corpo, e a gravação é da CLI.
fn upload_evidence(painel: &Painel, query: &str, request: &mut tiny_http::Request) -> Reply {
    let Some(declared) = request.body_length().map(|length| length as u64) else {
        return text(411, "envio sem Content-Length");
    };
    let limit_mb = read::attach_limit_mb(painel);
    if declared > limit_mb * 1024 * 1024 {
        return text(413, &format!("o arquivo passa do limite de {limit_mb} MB"));
    }
    let attached = task::attach_evidence_from_panel(
        &painel.project,
        &query_value(query, "spec"),
        &query_value(query, "tarefa"),
        &query_value(query, "evidencia"),
        &query_value(query, "extensao"),
        declared,
        &mut request.as_reader(),
    );
    match attached {
        Ok(done) => json(format!(
            "{{\"evidencia\":{},\"tamanho\":{},\"sha256\":{}}}",
            json_string(&done.evidence),
            done.size,
            json_string(&done.sha256)
        )),
        Err(error) => match error.downcast_ref::<AttachRefusal>() {
            Some(AttachRefusal::WrongStatus(message)) => text(409, message),
            Some(AttachRefusal::Invalid(message)) => text(400, message),
            None => text(500, &format!("não foi possível gravar o anexo: {error}")),
        },
    }
}

/// Recebe o anexo de um pedido por partes, sem o limite de 2 MB das ações; as recusas e a
/// gravação são de `actions::upload_attachment`.
fn upload_attachment(painel: &Painel, url: &str, request: &mut tiny_http::Request) -> Reply {
    let Some(length) = request.body_length().map(|length| length as u64) else {
        return text(411, "envio sem Content-Length");
    };
    let query = url.split_once('?').map_or("", |(_, query)| query);
    actions::upload_attachment(painel, query, length, &mut request.as_reader())
}

fn is_action(method: &str) -> bool {
    matches!(method, "POST" | "PUT")
}

fn token_header(request: &tiny_http::Request) -> Option<String> {
    request
        .headers()
        .iter()
        .find(|header| header.field.equiv(TOKEN_HEADER))
        .map(|header| header.value.as_str().to_string())
}

fn read_body(request: &mut tiny_http::Request) -> Result<Vec<u8>, Reply> {
    let mut body = Vec::new();
    request
        .as_reader()
        .take(MAX_BODY + 1)
        .read_to_end(&mut body)
        .map_err(|_| text(400, "corpo ilegível"))?;
    if body.len() as u64 > MAX_BODY {
        return Err(text(413, "corpo grande demais"));
    }
    Ok(body)
}

/// Arquivo do bundle; rotas do app (sem extensão) recebem o `index.html`, já com o token.
fn asset(painel: &Painel, path: &str) -> Reply {
    let name = bundle_name(path);
    page(name, bundle_file(name), &painel.token)
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

fn page(name: &str, file: Option<&[u8]>, token: &str) -> Reply {
    match file {
        Some(contents) if name == "index.html" => Reply {
            status: 200,
            content_type: content_type(name),
            body: with_token(contents, token),
            headers: Vec::new(),
        },
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

fn with_token(html: &[u8], token: &str) -> Vec<u8> {
    String::from_utf8_lossy(html)
        .replacen(
            "</head>",
            &format!("<meta name=\"painel-token\" content=\"{token}\"></head>"),
            1,
        )
        .into_bytes()
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

    const TOKEN: &str = "0123456789abcdef0123456789abcdef";

    fn painel() -> Painel {
        Painel {
            project: PathBuf::from("/projeto-inexistente"),
            root: PathBuf::from("/projeto-inexistente/vostok"),
            work: PathBuf::from("/projeto-inexistente/vostok/work"),
            token: TOKEN.to_string(),
        }
    }

    #[test]
    fn post_without_token_is_403() {
        let reply = dispatch(
            &painel(),
            "POST",
            "/api/aprovacoes?spec=SPEC-001",
            None,
            b"",
        );
        assert_eq!(reply.status, 403);
    }

    #[test]
    fn put_with_wrong_or_empty_token_is_403() {
        for token in ["errado", "", &TOKEN[..31]] {
            let reply = dispatch(&painel(), "PUT", "/api/arquivo", Some(token), b"x");
            assert_eq!(reply.status, 403, "{token:?}");
        }
    }

    #[test]
    fn put_evidencias_without_token_is_403_in_the_dispatch() {
        let target = "/api/evidencias?spec=SPEC-001&tarefa=TASK-01&evidencia=v.mov&extensao=mov";
        for token in [None, Some("errado")] {
            let reply = dispatch(&painel(), "PUT", target, token, b"x");
            assert_eq!(reply.status, 403, "{token:?}");
        }
    }

    #[test]
    fn action_with_token_reaches_the_actions() {
        // Sem `acao`, o pedido é recusado pela própria ação (400), e não pelo token (403).
        let reply = dispatch(&painel(), "POST", "/api/pedidos", Some(TOKEN), b"texto");
        assert_eq!(reply.status, 400);
        assert_eq!(reply.body, "ação desconhecida".as_bytes());
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
            token: TOKEN.to_string(),
        };
        let reply = dispatch(
            &painel,
            "GET",
            "/arquivos/vostok/work/docs/executivo.html",
            None,
            b"",
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
    fn atividades_sao_servidas_sem_token() {
        let reply = dispatch(&painel(), "GET", "/api/atividades?limite=5", None, b"");
        assert_eq!(reply.status, 200);
    }

    #[test]
    fn unknown_api_route_is_404() {
        for url in ["/api/nada", "/api/rota-desconhecida/x?y=1"] {
            assert_eq!(
                dispatch(&painel(), "GET", url, None, b"").status,
                404,
                "{url}"
            );
        }
    }

    #[test]
    fn ct62_put_de_anexo_sem_token_e_403_pelo_despacho() {
        for token in [None, Some("errado")] {
            let reply = dispatch(
                &painel(),
                "PUT",
                "/api/pedidos/anexos?rascunho=r1&nome=a.txt",
                token,
                b"abc",
            );
            assert_eq!(reply.status, 403, "{token:?}");
        }
    }

    #[test]
    fn other_methods_are_405() {
        for method in ["DELETE", "PATCH", "OPTIONS"] {
            let reply = dispatch(&painel(), method, "/api/arquivo", Some(TOKEN), b"");
            assert_eq!(reply.status, 405, "{method}");
        }
    }

    #[test]
    fn index_html_carries_the_token() {
        let html = b"<html><head><title>Painel</title></head><body></body></html>";
        let reply = page("index.html", Some(html), TOKEN);
        assert_eq!(reply.status, 200);
        assert_eq!(reply.content_type, "text/html; charset=utf-8");
        let body = String::from_utf8(reply.body).unwrap();
        assert!(body.contains(&format!(
            "<meta name=\"painel-token\" content=\"{TOKEN}\"></head>"
        )));
    }

    #[test]
    fn other_bundle_files_go_unchanged() {
        let reply = page("assets/app.js", Some(b"</head>"), TOKEN);
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
        let reply = dispatch(&painel(), "GET", "/", None, b"");
        assert_eq!(reply.status, 503);
        assert!(String::from_utf8(reply.body)
            .unwrap()
            .contains("pnpm --dir apps/painel"));
    }

    #[cfg(painel_bundle)]
    #[test]
    fn served_index_carries_the_token() {
        let reply = dispatch(&painel(), "GET", "/", None, b"");
        assert_eq!(reply.status, 200);
        assert!(String::from_utf8(reply.body).unwrap().contains(TOKEN));
    }
}
