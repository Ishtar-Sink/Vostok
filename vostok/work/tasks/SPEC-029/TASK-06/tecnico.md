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
