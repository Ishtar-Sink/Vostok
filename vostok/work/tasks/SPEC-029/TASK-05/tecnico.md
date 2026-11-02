<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Abordagem.** Implementa `live::version` e `live::handle`, os esqueletos da TASK-02. A versão é a maior data de modificação de `vostok/work/`, conferida a cada 500 ms, e o aviso vai por WebSocket só do servidor para o navegador. Cobre RF-05, CT-13 e RNF-03.

**Decisões.**
- **WebSocket, não SSE:** o estudo viu que o `tiny_http` só envia a resposta de SSE no fim da conexão.
- **Consulta periódica, não `notify`:** o `notify` soma 245 KB ao binário e quase não reduz o atraso enquanto o laço de 500 ms existir. Fica para pastas grandes.
- **Handshake próprio:** o handshake e o quadro de texto são escritos à mão, com `sha1`, para não trazer um crate de WebSocket.

**Riscos.** Thread presa numa aba fechada: o laço termina quando a escrita falha.

**Arquivos em comum na onda.** Nenhum.


## O que foi feito

**O que mudou:** `live::version` e `live::handle` em `apps/cli/src/painel/live.rs`.
- **Versão:** é a maior data de modificação de `vostok/work/`, conferida a cada 500 ms.
- **WebSocket:**
  - handshake com `sha1` e base64;
  - mensagens `conectado` e `mudou`;
  - uma thread por aba, que termina quando a escrita falha.

**Testes:** 5, entre eles o da chave da RFC 6455 e o `two_sockets_receive_change`.

**Pontos de atenção:**
- `base64` e `newest` foram copiados como funções privadas. Juntar os dois `base64` (`read.rs` e `live.rs`) fica como limpeza.
- Os testes foram escritos antes, mas a saída da falha não foi guardada.

**Verificação:**
- Os testes passaram, e o clippy ficou sem avisos.
- A regressão só registrou acréscimo.
- A evidência é `ao-vivo-testes.txt`.
