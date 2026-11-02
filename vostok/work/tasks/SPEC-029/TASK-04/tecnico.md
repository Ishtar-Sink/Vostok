<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Abordagem.** Implementa `read::route`, o esqueleto deixado pela TASK-02, portando do spike as leituras e o JSON. Cobre RF-02, RF-04, CT-03 e CB-05.

**Decisões.** O JSON mantém os nomes de campos do spike, porque as telas da onda 2 são escritas contra eles. Mudar os nomes agora quebraria as tarefas de frontend que rodam em paralelo. Sem cache: os arquivos são a fonte de verdade (RN-02), e o custo medido no estudo é baixo. `versao` só chama `live::version`, da TASK-05.

**Riscos.** Travessia de caminho em `arquivo` e `fonte`: o `safe_segment` e a checagem de prefixo têm teste.

**Arquivos em comum na onda.** Nenhum.


## O que foi feito

**O que mudou:** `read::route` em `apps/cli/src/painel/read.rs`.
- **Rotas:** `visao`, `versao`, `marca`, `documentos`, `configuracoes`, `specs/<ID>`, `planos/<ID>`, `tarefas/<SPEC>/<TASK>`, `fonte`, `arquivo` e `/arquivos/...`.
- **Leitura:** tudo é lido do disco a cada requisição, com os nomes de campos do JSON do spike.
- **Arquivadas:** spec e tarefa arquivadas trazem `archived: true`.
- **Marca:** sem `brand`, `light` e `dark`, a rota devolve os tokens padrão.

**Decisões fora do spike:**
- **Prefixo permitido:** vem de `painel.root`, e não de `vostok` fixo.
- **Recusas:** caminho com `..`, fora do harness ou por link simbólico é recusado com 400.

**Pontos de atenção:**
- `content_type` ficou duplicado aqui, porque o de `server.rs` é privado.
- O encaminhamento de `/arquivos/...` no `server.rs` ficou pronto pelo ajuste da TASK-06.
- Os testes foram escritos junto com o código, sem a etapa de vê-los falhar.

**Verificação:**
- Os 21 testes de `painel::read` passaram, e o clippy ficou sem avisos.
- A regressão só registrou acréscimo: os testes novos do módulo.
- A evidência é `leitura-testes.txt`.
