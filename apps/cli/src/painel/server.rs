//! Servidor HTTP do painel: escuta só em `127.0.0.1`, serve o bundle de `apps/painel` e despacha os GET para a leitura e o aviso ao vivo; os demais métodos respondem 405.

use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;
use std::sync::Arc;
use std::thread;

use anyhow::{anyhow, Result};

use super::{live, not_found, read, text, Painel, Reply};

/// Se o bundle de `apps/painel` foi embutido neste binário.
pub const HAS_BUNDLE: bool = cfg!(painel_bundle);

/// Resposta da página quando o binário não tem o bundle (FA-02).
pub const BUNDLE_MISSING: &str = "Este vostok foi compilado sem o bundle do painel. Para obtê-lo, \
gere o bundle com `pnpm --dir apps/painel install --frozen-lockfile && pnpm --dir apps/painel build` \
(Node e pnpm no PATH) e compile de novo com `cargo install --path apps/cli --features painel`.";

const LIVE_PATH: &str = "/api/ao-vivo";

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
    for request in server.incoming_requests() {
        let painel = Arc::clone(&painel);
        thread::spawn(move || handle(&painel, request));
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

fn handle(painel: &Painel, request: tiny_http::Request) {
    let method = request.method().as_str().to_string();
    let url = request.url().to_string();
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
    let _ = request.respond(to_response(dispatch(painel, &method, &url)));
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
/// `GET` para o bundle e qualquer outro método para o 405.
pub fn dispatch(painel: &Painel, method: &str, url: &str) -> Reply {
    let (path, query) = url.split_once('?').unwrap_or((url, ""));
    let parts: Vec<&str> = path.trim_matches('/').split('/').collect();
    match method {
        "GET" if matches!(parts.first(), Some(&"api" | &"arquivos")) => {
            read::route(painel, &parts, query).unwrap_or_else(not_found)
        }
        "GET" => asset(path),
        _ => text(405, "método não permitido"),
    }
}

/// Arquivo do bundle; rotas do app (sem extensão) recebem o `index.html`, sem alteração.
fn asset(path: &str) -> Reply {
    let name = bundle_name(path);
    page(name, bundle_file(name))
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
                let reply = dispatch(&painel(), method, path);
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
        let reply = dispatch(&painel, "GET", "/arquivos/vostok/work/docs/executivo.html");
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
        let reply = dispatch(&painel(), "GET", "/api/atividades?limite=5");
        assert_eq!(reply.status, 404);
    }

    #[test]
    fn unknown_api_route_is_404() {
        for url in ["/api/nada", "/api/rota-desconhecida/x?y=1"] {
            assert_eq!(dispatch(&painel(), "GET", url).status, 404, "{url}");
        }
    }

    #[test]
    fn other_methods_are_405() {
        for method in ["DELETE", "PATCH", "OPTIONS"] {
            let reply = dispatch(&painel(), method, "/api/arquivo");
            assert_eq!(reply.status, 405, "{method}");
        }
    }

    #[test]
    fn index_html_vai_sem_token() {
        let html = b"<html><head><title>Painel</title></head><body></body></html>";
        let reply = page("index.html", Some(html));
        assert_eq!(reply.status, 200);
        assert_eq!(reply.content_type, "text/html; charset=utf-8");
        assert_eq!(reply.body, html);
        assert!(!String::from_utf8(reply.body)
            .unwrap()
            .contains("painel-token"));
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
        let reply = dispatch(&painel(), "GET", "/");
        assert_eq!(reply.status, 503);
        assert!(String::from_utf8(reply.body)
            .unwrap()
            .contains("pnpm --dir apps/painel"));
    }

    #[cfg(painel_bundle)]
    #[test]
    fn served_index_vai_sem_token() {
        let reply = dispatch(&painel(), "GET", "/");
        assert_eq!(reply.status, 200);
        assert!(!String::from_utf8(reply.body)
            .unwrap()
            .contains("painel-token"));
    }
}
