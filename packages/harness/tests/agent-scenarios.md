# Cenários com agente

Roteiro dos cenários que exercitam as skills com um agente de verdade, e os resultados da última execução. Os scripts são cobertos por `tests/run-tests.sh`; aqui fica o que depende do agente: identificar a spec, decompor o pedido, perguntar ou aplicar.

## Como executar

1. Monte as cópias descartáveis:

   ```bash
   packages/harness/tests/fixtures/agent-scenarios.sh <pasta>
   ```

   Cada cenário recebe uma cópia do projeto com as specs de exemplo:

   | Spec | Título | Status |
   |---|---|---|
   | SPEC-001 | Exportar memória em JSON | refinada, com RF-01 a RF-03, RNF-01, RNF-02 (`jq`), RN-01 (obsoletas não saem), CB-01 e CB-02 |
   | SPEC-002 | Listar tarefas | refinada |
   | SPEC-003 | Painel de métricas | rascunho |

   Os cenários `ct04` e `ct09` não têm a SPEC-002, e o `ct05` só tem a SPEC-003.

2. Para cada cenário, inicie um agente numa sessão nova com a pasta do cenário como raiz do projeto. Se houver, passe o contexto da conversa e depois a mensagem do usuário. O agente não recebe resposta do usuário: quando o harness manda parar e aguardar, a mensagem dele é o resultado.

3. Confira a mensagem final e os arquivos: quais specs mudaram (compare com `cksum`), o "Histórico de ajustes" e se `adjust-spec.sh --validate` passa.

## Cenários

| Cenário | Caso de teste | Contexto e mensagem do usuário | Resultado esperado |
|---|---|---|---|
| ct01 | CT-01 | Contexto: o agente acabou de refinar a SPEC-002 nesta conversa. Mensagem: "Ficou bom. Só acrescenta que o script também deve aceitar um filtro opcional por status." | Ajusta a SPEC-002 sem perguntar qual spec. |
| ct02 | CT-02 | "Na SPEC-001, o caminho do arquivo de saída deve vir de config/memory.yml em vez de ser fixo no código." | Ajusta a SPEC-001. |
| ct03 | CT-03 | "Acrescenta na spec um caso de borda para quando o arquivo de saída já existir." | Pergunta qual spec, com as duas candidatas, sem editar. |
| ct04 | CT-04 | "Remove o RNF-02 da spec, não quero depender de nenhuma ferramenta específica para montar o JSON." | Ajusta a única refinada sem perguntar. |
| ct05 | CT-05 | "Ajusta a spec refinada: coloca um requisito de log das operações." | Informa que não há spec refinada e para. |
| ct06 | CT-06 | "Na SPEC-003, ajusta para ter um requisito de exportar o painel em PDF." | Informa que só specs refinadas são ajustadas. |
| ct09 | CT-09 | "Na spec do export, acrescenta que o JSON precisa trazer todas as entradas da memória, inclusive as obsoletas, pro painel mostrar o histórico completo." | 1 rodada de perguntas sobre o conflito com a RN-01, sem editar. |
| ct13 | CT-13 | "Revisei a SPEC-001. Quero que o tipo all deixe de existir, porque o painel vai pedir um tipo por vez, e o arquivo de saída passa a se chamar memory-export.json, na mesma pasta. Ah, e acho que falta um critério de aceite garantindo que o JSON gerado é válido." | Três ajustes aplicados e listados; uma linha no histórico. |
| ct14 | CT-14 | "Ajustes na SPEC-001: 1. Remova o CB-02. 2. O RF-01 deve exportar também a data de criação da entrada. 3. Acrescente um risco: arquivo grande demais para o painel carregar." | Três ajustes aplicados e listados; uma linha no histórico. |
| ct15 | CT-15 | "SPEC-001, na seção Requisitos funcionais, RF-03: onde está \"`memory/export.json`\", coloque \"`work/exports/memory.json`\"." | Só o trecho citado muda. |
| ct16 | CT-16 | "Na SPEC-001: 1. O RF-02 deve aceitar só o tipo decision. 2. O RF-02 deve aceitar todos os tipos menos issue. 3. Remova o RF-07." | 1 rodada de perguntas com a contradição e o RF-07 inexistente, sem editar. |

## Resultados

Execução de 2026-09-27, com Claude Code (Opus 5.5) como agente.

| Cenário | Resultado | Arquivos alterados | Observação |
|---|---|---|---|
| ct01 | passou | refinada da SPEC-002 | Novo RF-03 com `--status`, CB-02, CT-02, CT-03 e CA-02. Duas inferências registradas. |
| ct02 | passou | refinada da SPEC-001 | RF-03, fluxo, FA-02, CB-03, contrato, componentes, CT-03, CT-04 e CA-03 atualizados. |
| ct03 | passou | nenhum | Perguntou entre SPEC-001 e SPEC-002 com o template `question`. |
| ct04 | passou | refinada da SPEC-001 | Removeu RNF-02 e, por inferência, a premissa e o risco ligados ao `jq`. |
| ct05 | passou | nenhum | Informou que não há spec refinada. |
| ct06 | passou | nenhum | Informou que a SPEC-003 está em `draft`. |
| ct09 | passou | nenhum | Perguntou sobre o conflito com a RN-01 e notou que o CA-01 também seria afetado. |
| ct13 | passou | refinada da SPEC-001 | Três ajustes listados; RF-02, CT-02 e CA-02 removidos; novo CA-03. Uma linha no histórico. |
| ct14 | passou | refinada da SPEC-001 | Três ajustes listados; CT-01 atualizado junto com o RF-01. Uma linha no histórico. |
| ct15 | passou, com ressalva | refinada da SPEC-001 | Trocou o RF-03 e também o passo 3 do fluxo, que citava o mesmo caminho, registrando isso como inferência ([adjust-spec-06]). |
| ct16 | passou | nenhum | Uma rodada com as duas perguntas: RF-02 contraditório e RF-07 inexistente. |

Em todos os cenários que ajustaram a spec, `--record-manual` rodou antes da edição, `--validate` passou, o histórico ganhou uma única linha `agente` e nenhum rascunho foi alterado.
