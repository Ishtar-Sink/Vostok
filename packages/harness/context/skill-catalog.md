# Catálogo de Habilidades

## Padrão

Habilidades incluídas junto ao harness.

| Habilidade | Descrição | Caminho |
|    ---     |    ---    |   ---   |
| refine-spec | Refina uma especificação (spec) em rascunho. Investiga o projeto e a memória, identifica ambiguidades, conflitos e lacunas, resolve com o usuário em rodadas de perguntas e gera em `refined/` uma spec detalhada, com seções conforme o tipo (feature, improvement, fix, study, analysis). Use quando o usuário pedir para refinar, detalhar ou esclarecer uma spec em rascunho (status draft), ou perguntar quais specs em draft estão prontas para refinamento. Não use para specs já refinadas: ajustes nelas são feitos pela skill adjust-spec. | `../skills/default/refine-spec` |
| create-skill | Cria uma nova skill do harness no formato padrão: gera o SKILL.md a partir do template, os scripts da skill, os redirecionamentos para Claude Code, Codex e Cursor e a entrada no catálogo. Use quando o usuário pedir para criar, adicionar ou gerar uma skill ou habilidade, ou transformar um fluxo de trabalho em skill. Use sempre esta skill em vez dos criadores de skill nativos das ferramentas. | `../skills/default/create-skill` |
| adjust-spec | Ajusta uma especificação (spec) já refinada, com status refined ou approved. Aplica um ou vários ajustes pedidos numa mesma mensagem, em prosa, em lista ou citando o trecho exato, registra edições feitas à mão e mantém o histórico de ajustes da spec, validando-a com as mesmas regras do refinamento. Use quando o usuário pedir para ajustar, corrigir, mudar, alterar, acrescentar ou remover algo de uma spec refinada ou aprovada, ou comentar a spec logo depois de ela ser refinada ou ajustada. | `../skills/default/adjust-spec` |
| approve-spec | Aprova uma especificação (spec) refinada, com status refined. Confere se a spec está válida, sem pendências abertas e sem edições manuais sem registro, muda o status para approved, grava a versão aprovada para a granularização e remove o rascunho. Use quando o usuário pedir para aprovar uma spec, disser que aprova ou que a spec está aprovada, ou citar um ID como SPEC-001 para aprovação. | `../skills/default/approve-spec` |

## Customizadas

Habilidades criadas pelo usuário. Toda inclusão e remoção de habiliades deve ser atualizado nesta tabela.

| Habilidade | Descrição | Caminho |
|    ---     |    ---    |   ---   |
