<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Abordagem.** Cria `apps/cli/tests/painel.rs` com os testes de ponta a ponta e altera a release para gerar o bundle, compilar com a feature `painel` e conferir o tamanho. Cobre RF-01, RF-13, CT-01 e CT-15.

**Decisões.** A conferência do tamanho fica na release, que é onde o binário distribuído é gerado; uma verificação só local foi descartada porque não protege as plataformas da matriz. Os prints finais usam o binário novo, substituindo os da onda 2, que usaram o spike.

**Riscos.** Passar do limite: o estudo prevê embutir o bundle com gzip (cerca de 108 KB a menos) como saída, numa tarefa nova se for preciso. O build de release leva minutos: a evidência do tamanho tem tempo-limite de 600 s.

**Arquivos em comum na onda.** Nenhum: é a única tarefa da onda 3.
