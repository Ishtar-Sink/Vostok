//! Aviso ao vivo das mudanças de `work/` pelo WebSocket `/api/ao-vivo`.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, SystemTime};

use sha1::{Digest, Sha1};

use crate::harness::Harness;

const WEBSOCKET_GUID: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
const WATCH: Duration = Duration::from_millis(500);

/// Versão de `work/`: muda sempre que um arquivo de lá muda, menos os da pasta de anexos dos
/// pedidos, que têm o próprio envio e podem ser grandes.
pub fn version(work: &Path) -> u128 {
    newest(work, &attachments_dir(work))
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|time| time.as_nanos())
        .unwrap_or(0)
}

fn attachments_dir(work: &Path) -> PathBuf {
    let harness = Harness {
        root: work.parent().map(Path::to_path_buf).unwrap_or_default(),
    };
    let requests = harness.requests_dir();
    match harness.folders() {
        Ok(folders) => requests.join(folders.requests.attachments),
        Err(_) => work.join("pedidos/anexos"),
    }
}

fn newest(dir: &Path, skipped: &Path) -> SystemTime {
    let mut latest = fs::metadata(dir)
        .and_then(|meta| meta.modified())
        .unwrap_or(SystemTime::UNIX_EPOCH);
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if entry.file_name() == crate::commands::task::ATTACH_TEMP_DIR || path == skipped {
                continue;
            }
            let modified = if path.is_dir() {
                newest(&path, skipped)
            } else {
                entry
                    .metadata()
                    .and_then(|meta| meta.modified())
                    .unwrap_or(SystemTime::UNIX_EPOCH)
            };
            latest = latest.max(modified);
        }
    }
    latest
}

/// Atende o `GET /api/ao-vivo`: cada aba fica na sua thread até a escrita falhar.
pub fn handle(painel: &super::Painel, request: tiny_http::Request) {
    let key = request
        .headers()
        .iter()
        .find(|header| header.field.equiv("Sec-WebSocket-Key"))
        .map(|header| header.value.as_str().to_string());
    let Some(key) = key else {
        let response = tiny_http::Response::from_string("esperado WebSocket").with_status_code(400);
        let _ = request.respond(response);
        return;
    };
    let accept = accept_key(&key);
    let response = tiny_http::Response::new_empty(tiny_http::StatusCode(101))
        .with_header(header(b"Upgrade", b"websocket"))
        .with_header(header(b"Sec-WebSocket-Accept", accept.as_bytes()));
    let mut socket = request.upgrade("websocket", response);
    let mut last = version(&painel.work);
    if send(&mut socket, "conectado").is_err() {
        return;
    }
    loop {
        thread::sleep(WATCH);
        let now = version(&painel.work);
        if now != last {
            last = now;
            if send(&mut socket, "mudou").is_err() {
                return;
            }
        }
    }
}

fn header(field: &[u8], value: &[u8]) -> tiny_http::Header {
    tiny_http::Header::from_bytes(field, value).expect("cabeçalho válido")
}

fn send(socket: &mut impl Write, text: &str) -> std::io::Result<()> {
    socket.write_all(&frame(text))?;
    socket.flush()
}

/// Valor de `Sec-WebSocket-Accept` da RFC 6455 para a chave enviada pelo navegador.
fn accept_key(key: &str) -> String {
    base64(&Sha1::digest(format!("{key}{WEBSOCKET_GUID}").as_bytes()))
}

/// Quadro de texto sem máscara (servidor para navegador), para mensagens de até 125 bytes.
fn frame(text: &str) -> Vec<u8> {
    let mut frame = vec![0x81, text.len() as u8];
    frame.extend_from_slice(text.as_bytes());
    frame
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
    use std::io::Read;
    use std::net::TcpStream;
    use std::sync::mpsc;

    #[test]
    fn version_changes_after_writing_in_a_subfolder() {
        let dir = tempfile::tempdir().unwrap();
        let sub = dir.path().join("tasks/SPEC-001");
        fs::create_dir_all(&sub).unwrap();
        let before = version(dir.path());
        thread::sleep(Duration::from_millis(20));
        fs::write(sub.join("a.md"), "x").unwrap();
        assert_ne!(version(dir.path()), before);
    }

    #[test]
    fn version_ignores_writes_in_the_upload_folder() {
        let dir = tempfile::tempdir().unwrap();
        let upload = dir.path().join("tasks/SPEC-001/TASK-01/evidencias");
        let temp = upload.join(crate::commands::task::ATTACH_TEMP_DIR);
        fs::create_dir_all(&temp).unwrap();
        let before = version(dir.path());
        thread::sleep(Duration::from_millis(20));
        fs::write(temp.join("parcial"), "x").unwrap();
        assert_eq!(version(dir.path()), before);
        fs::write(upload.join("video.mov"), "x").unwrap();
        assert_ne!(version(dir.path()), before);
    }

    fn work_with_requests() -> (tempfile::TempDir, std::path::PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let work = dir.path().join("vostok/work");
        fs::create_dir_all(work.join("pedidos/anexos")).unwrap();
        fs::create_dir_all(work.join("pedidos/pendente")).unwrap();
        (dir, work)
    }

    #[test]
    fn ct66_escrita_na_pasta_de_anexos_nao_muda_a_versao() {
        let (_dir, work) = work_with_requests();
        let draft = work.join("pedidos/anexos/.rascunho/r1");
        fs::create_dir_all(&draft).unwrap();
        let before = version(&work);
        thread::sleep(Duration::from_millis(20));
        fs::write(draft.join(".parte"), "x").unwrap();
        fs::write(draft.join("01-a.txt"), "x").unwrap();
        fs::create_dir_all(work.join("pedidos/anexos/PED-001")).unwrap();
        assert_eq!(version(&work), before);
    }

    #[test]
    fn ct66_escrita_em_pendente_muda_a_versao() {
        let (_dir, work) = work_with_requests();
        let before = version(&work);
        thread::sleep(Duration::from_millis(20));
        fs::write(work.join("pedidos/pendente/PED-001.md"), "x").unwrap();
        assert_ne!(version(&work), before);
    }

    #[test]
    fn version_of_a_missing_folder_is_zero() {
        assert_eq!(version(Path::new("/nao/existe/work")), 0);
    }

    #[test]
    fn accept_key_matches_the_rfc_6455_example() {
        assert_eq!(
            accept_key("dGhlIHNhbXBsZSBub25jZQ=="),
            "s3pPLMBiTxaQ9kYGzzhZRbK+xOo="
        );
    }

    #[test]
    fn frame_has_the_right_length_below_126_bytes() {
        assert_eq!(frame("mudou"), [&[0x81, 5][..], b"mudou"].concat());
        let longest = "a".repeat(125);
        let built = frame(&longest);
        assert_eq!(built[1], 125);
        assert_eq!(built.len(), 127);
        assert_eq!(frame("")[..], [0x81, 0]);
    }

    fn open_socket(address: &str) -> TcpStream {
        let mut stream = TcpStream::connect(address).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let request = format!(
            "GET /api/ao-vivo HTTP/1.1\r\nHost: {address}\r\nUpgrade: websocket\r\n\
             Connection: Upgrade\r\nSec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\n\
             Sec-WebSocket-Version: 13\r\n\r\n"
        );
        stream.write_all(request.as_bytes()).unwrap();
        stream
    }

    /// Lê até o buffer conter `needle` ou o tempo esgotar.
    fn read_until(stream: &mut TcpStream, needle: &[u8]) -> Vec<u8> {
        let mut collected = Vec::new();
        let mut buffer = [0u8; 512];
        while !collected.windows(needle.len()).any(|part| part == needle) {
            match stream.read(&mut buffer) {
                Ok(0) | Err(_) => break,
                Ok(count) => collected.extend_from_slice(&buffer[..count]),
            }
        }
        collected
    }

    #[test]
    fn two_sockets_receive_change() {
        let dir = tempfile::tempdir().unwrap();
        let work = dir.path().join("work");
        fs::create_dir_all(work.join("specs")).unwrap();
        let painel = super::super::Painel {
            project: dir.path().to_path_buf(),
            root: dir.path().to_path_buf(),
            work: work.clone(),
            token: "t".to_string(),
        };
        let (sender, receiver) = mpsc::channel();
        thread::spawn(move || {
            let _ = super::super::server::serve(painel, 0, |url| {
                sender.send(url).unwrap();
            });
        });
        let url = receiver.recv_timeout(Duration::from_secs(5)).unwrap();
        let address = url.trim_start_matches("http://").trim_end_matches('/');

        let mut first = open_socket(address);
        let mut second = open_socket(address);
        let handshake = read_until(&mut first, b"conectado");
        assert!(String::from_utf8_lossy(&handshake).contains("s3pPLMBiTxaQ9kYGzzhZRbK+xOo="));
        read_until(&mut second, b"conectado");

        thread::sleep(Duration::from_millis(50));
        fs::write(work.join("specs/nova.md"), "mudança").unwrap();
        let started = std::time::Instant::now();
        assert!(read_until(&mut first, b"mudou").ends_with(b"mudou"));
        assert!(read_until(&mut second, b"mudou").ends_with(b"mudou"));
        assert!(started.elapsed() < Duration::from_secs(1));
    }
}
