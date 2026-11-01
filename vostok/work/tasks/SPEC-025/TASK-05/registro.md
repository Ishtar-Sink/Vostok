## Registro de execução

| # | Data | Etapa | Resultado | Detalhe |
|---|---|---|---|---|
| 1 | 2026-11-01 | início | in_progress | aprovada em 2026-11-01T23:09:52Z; modelo: claude-opus-5-5 |
| 2 | 2026-11-01 | baseline | coletado | 1 comando(s); códigos de saída: 0 |
| 3 | 2026-11-01 | ajuste | in_progress | os prints executivo-novo.png e evidencias-novo.png passam a gerar os documentos da SPEC-023 arquivada numa cópia temporária, porque o conteúdo da SPEC-025 só existe na finalização |
| 4 | 2026-11-01 | testes | passed | Verificação sem falhas; saída em validation/tests.out |
| 5 | 2026-11-01 | regressão | passed | 0 comando(s) iguais ao baseline; 1 com acréscimo |
| 6 | 2026-11-01 | evidências | pending | print sem conferência humana |
| 7 | 2026-11-01 | conferência | usuário | print executivo-novo.png conferido pelo usuário (sha256 7e06660c) |
| 8 | 2026-11-01 | conferência | usuário | print evidencias-novo.png conferido pelo usuário (sha256 a359a85f) |
| 9 | 2026-11-01 | evidências | passed | 3 evidência(s) em TASK-05/evidencias/ |
| 10 | 2026-11-01 | conclusão | done | testes, regressão e evidências passed; critérios de conclusão marcados |

## Histórico de ajustes

| # | Data | Origem | Resumo |
|---|---|---|---|
| 1 | 2026-11-01 | agente | toda cor dos documentos vem de documents.yml, sem valor fixo nos templates nem no finalize.rs, com o teste visual_colors_come_only_from_documents_yml, para a SPEC-026 só trocar a paleta |
| 2 | 2026-11-01 | agente | detalhes do modelo escolhido na TASK-04 (C com o índice do B): página de referência em modelo-c-indice, subconjunto das fontes com pyftsubset via uvx autorizado pelo usuário, tokens de 2026-10 com os nomes da organização em documents.yml e a estrutura dos templates |
| 3 | 2026-11-01 | agente | a faixa do resumo mostra a coluna de números a partir da seção Em números, criada pela TASK-06, e ocupa a largura toda sem ela |
| 4 | 2026-11-01 | agente | os prints executivo-novo.png e evidencias-novo.png passam a gerar os documentos da SPEC-023 arquivada numa cópia temporária, porque o conteúdo da SPEC-025 só existe na finalização |
