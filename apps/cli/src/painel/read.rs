//! Leitura dos arquivos de `vostok/` para o painel: visão, specs, planos, tarefas e documentos.

use super::{Painel, Reply};

/// Responde um `GET /api/...`; `parts` são os segmentos do caminho, a partir de `api`.
/// `None` quando a rota não existe, e o servidor responde 404.
pub fn route(_painel: &Painel, _parts: &[&str], _query: &str) -> Option<Reply> {
    None
}
