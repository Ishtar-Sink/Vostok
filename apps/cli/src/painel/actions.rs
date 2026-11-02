//! Ações do usuário no painel (POST e PUT), já com o token de sessão conferido.

use super::{text, Painel, Reply};

/// `parts` são os segmentos do caminho, a partir de `api`; `body` já vem limitado pelo servidor.
pub fn act(_painel: &Painel, _method: &str, _parts: &[&str], _query: &str, _body: &[u8]) -> Reply {
    text(501, "ação ainda não implementada")
}
