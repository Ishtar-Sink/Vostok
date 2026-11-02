<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Abordagem.** Implementa `read::route`, o esqueleto deixado pela TASK-02, portando do spike as leituras e o JSON. Cobre RF-02, RF-04, CT-03 e CB-05.

**Decisões.** O JSON mantém os nomes de campos do spike, porque as telas da onda 2 são escritas contra eles. Mudar os nomes agora quebraria as tarefas de frontend que rodam em paralelo. Sem cache: os arquivos são a fonte de verdade (RN-02), e o custo medido no estudo é baixo. `versao` só chama `live::version`, da TASK-05.

**Riscos.** Travessia de caminho em `arquivo` e `fonte`: o `safe_segment` e a checagem de prefixo têm teste.

**Arquivos em comum na onda.** Nenhum.
