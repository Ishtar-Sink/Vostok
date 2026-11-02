//! Aviso ao vivo das mudanças de `work/` pelo WebSocket `/api/ao-vivo`.

use std::path::Path;

/// Versão de `work/`: muda sempre que um arquivo de lá muda.
pub fn version(_work: &Path) -> u128 {
    0
}

/// Atende o `GET /api/ao-vivo`.
pub fn handle(_painel: &super::Painel, request: tiny_http::Request) {
    let response =
        tiny_http::Response::from_string("ao vivo ainda não implementado").with_status_code(501);
    let _ = request.respond(response);
}
