<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

Monta o protótipo funcional do painel: as telas centrais, lidas direto dos arquivos do harness, com os componentes do Atria e a marca de cada pasta, abertas em duas pastas ao mesmo tempo e atualizadas sozinhas quando um arquivo muda.
O usuário usa o protótipo e aprova o visual antes de a tarefa terminar. Perfil padrao. Risco: as cores da marca não casarem com os tokens do Atria.
Depende da medição (TASK-02) e desbloqueia os testes de aprovação e de loop (TASK-04).

## O que foi feito

- O painel ganhou um protótipo funcional que roda no computador do usuário com um comando. Ele lê direto os arquivos do harness da pasta, sem servidor nem banco.
- Telas:
  - **Painel geral:** o que espera o usuário e as specs em andamento.
  - **Pendências e specs:** a página de pendências e a lista de specs.
  - **Spec:** cada spec com índice, quadro kanban das tarefas e plano.
  - **Tarefa:** cada tarefa com o conteúdo para pessoas primeiro.
  - **Documentos e configurações:** os documentos finais e as configurações do harness da pasta, com controles próprios.
- As referências internas (requisitos, casos de borda, perguntas) abrem numa janela flutuante, sem sair da página.
- Os botões de aprovar, responder e pedir execução já aparecem conforme o status. A gravação dessas ações fica para a TASK-04.
- O painel segue o visual do Atria e usa as cores e o ícone de cada pasta. Abre em várias pastas ao mesmo tempo, cada uma na sua porta, e só aceita acesso do próprio computador.
- A tela se atualiza sozinha em cerca de um terço de segundo quando um arquivo muda.
  - O painel usa cerca de 5 MB de memória e quase nada de processador.
  - Com uma aba aberta, a verificação de mudanças custa cerca de 1,7% do processador.
- O programa passa de 4,6 MB para 5,5 MB, acima do limite de 5,3 MB proposto na TASK-02. A TASK-05 revê o limite com o painel completo.
- O usuário usou o protótipo em sete rodadas de ajustes, validou com uma massa de dados fictícia que cobre todas as etapas e aprovou o visual.
