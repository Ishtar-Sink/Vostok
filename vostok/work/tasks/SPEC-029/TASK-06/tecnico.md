<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Abordagem.** Implementa `actions::act`, o esqueleto da TASK-02, portando do spike a edição, a aprovação, o aval e os pedidos, e acrescenta a gravação de configuração. O token já foi conferido pelo `server.rs` antes da chamada. Cobre RF-06, RF-09, RF-10, RF-11, CT-04, CT-08 a CT-12 e CB-02 a CB-04.

**Decisões.**
- **Aprovação direta:** a aprovação chama as funções da CLI no mesmo processo (decisão do estudo). O pedido ao agente foi descartado, porque coloca o agente entre o clique e a aprovação.
- **Controle otimista por hash:** a trava de arquivo foi descartada, porque fica presa e o agente não a respeita.
- **Configuração com a mesma rota de hash:** a gravação de configuração usa uma rota própria, mas o mesmo controle. Reaproveitar `/api/arquivo` foi descartado, porque essa rota só aceita `vostok/work/` e registra ajuste manual, o que não vale para `config/`.
- **Aprovação não é pedido:** aprovar tarefa e spec nunca vira pedido (RN-04).

**Escolhas que ficam com quem executa.** Mensagens de erro em português, iguais às do `vostok`. Em caso de dúvida entre recusar e aceitar um caminho, recuse.

**Riscos.** Gravar fora de `vostok/` por travessia de caminho: há teste para cada recusa.

**Arquivos em comum na onda.** Nenhum.


## O que foi feito

**O que mudou em `apps/cli/src/painel/actions.rs`:**
- **Edição (`PUT /api/arquivo`):** grava com hash, responde 409 com o conteúdo atual, recusa `registro.md`, `README.md`, `estado.yml` e caminhos de fora (403), e registra o ajuste manual com os problemas da validação.
- **Configuração (`PUT /api/configuracao`):** grava com hash e devolve os problemas de `rules::validate_at`.
- **Aprovação (`POST /api/aprovacoes`):**
  - grava `approved_by: painel`;
  - acrescenta o aval em `avais.log`, só por acréscimo;
  - responde 422 sem gravar aval quando o `vostok` recusa.
- **Pedidos (`POST /api/pedidos`):**
  - 10 ações aceitas, com 400 para ação desconhecida e para as de aprovação;
  - numeração que não deixa dois pedidos pegarem o mesmo `PED-NNN`.
- **Limites:** corpo acima de 2 MB recebe 413, e link simbólico para fora da pasta é recusado.

**Ajuste 1, com a sua aprovação:**
- **`rules.rs`:** expõe `validate_at(project) -> Result<Vec<String>>`, que o `validate` passou a usar.
- **`server.rs`:**
  - encaminha `GET /arquivos/...` para `read::route`;
  - o teste `action_with_token_reaches_the_actions` agora espera o 400 da ação real;
  - há teste novo de que `/arquivos` não cai no bundle.

**Pontos de atenção:**
- Falta o teste da aprovação de spec que dá certo; só o 422 de spec inexistente está coberto.
- Os testes foram escritos junto com o código.

**Verificação:**
- Os testes passaram: 26 de `painel::actions`, 10 de `painel::server` e 26 de `context_memory_rules`.
- A regressão só registrou acréscimo, e os testes de aprovação ficaram iguais.
- A evidência é `acoes-testes.txt`.
