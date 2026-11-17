# Ajuste que preserva testes e conferências

Estudo da SPEC-057, feito antes do código.

## Decisões

O estudo fecha cinco decisões:

1. Os testes continuam valendo enquanto os arquivos que a tarefa muda não mudarem.
2. A Verificação e o Baseline ganham uma impressão própria, separada das Evidências.
3. O ajuste guarda uma cópia da Verificação anterior para saber o que mudou em cada evidência.
4. A conferência cai quando o comando ou o tipo da evidência muda. Quando só o texto esperado muda, ela fica com uma marca.
5. A CLI e o painel rodam a mesma lista de exemplos de busca.

As tarefas TASK-02 a TASK-06 já foram detalhadas com essas escolhas.

## Impressão do código

**O problema.** Hoje o ajuste zera os testes sempre, porque nada diz se o código testado continua o mesmo. Para manter os testes com segurança, o `vostok task tests` precisa gravar uma impressão do código no momento em que passa. O ajuste e o `finish` comparam essa impressão com o código atual (RF-01, RF-02, RF-03, RNF-02).

**Alternativas comparadas.**

| Opção | Como calcula | Falso "código igual" | Falso "código mudou" |
|---|---|---|---|
| (a) Só os arquivos de "Mudar" | SHA-256 de `caminho`, byte zero, conteúdo, byte zero de cada caminho de "Mudar", em ordem alfabética; arquivo ausente entra como `<ausente>`; pasta entra com os arquivos dela, em ordem | Não, para o que a tarefa declara mudar | Não |
| (b) "Mudar" mais `git diff HEAD` | A impressão de (a) seguida do diff do repositório inteiro | Não | Sim: qualquer arquivo mexido por outra tarefa da mesma onda, ou por um commit, muda a impressão |

**Escolha: (a).** A opção (b) zeraria os testes de uma tarefa sempre que outra tarefa da mesma onda mexesse no repositório, que é o caso normal de uma onda em paralelo. Com isso, o benefício pedido na issue #17 deixaria de existir justamente nas specs grandes.

**Risco que fica.** Um código mudado fora de "Mudar" não altera a impressão. Esse risco já existia antes desta spec e continua coberto: o `vostok task regression` compara o Baseline antes e depois. Uma mudança fora de "Mudar" que quebre algo aparece como regressão. Uma mudança fora de "Mudar" sem quebra é um desvio de escopo, que a revisão do diff pega.

**Campos no estado.** O `vostok task tests` grava dois campos quando passa e os grava como `null` quando falha:

- `tests_code_hash`: a impressão (a).
- `tests_checks_hash`: o SHA-256 do texto de `### Baseline` seguido de uma quebra de linha e do texto de `### Verificação`.

O `verification_hash` atual não serve para isso, porque cobre o `verificacao.md` inteiro, inclusive as Evidências. Um ajuste só no `Trecho:` mudaria esse hash e zeraria os testes, que é o que a spec quer evitar (RN-01).

**Tarefa antiga.** Sem os dois campos, o ajuste zera tudo como hoje (RNF-03, CB-01). Não há como saber o que foi testado.

## Versão anterior das evidências

**O problema.** O `vostok task adjust record` roda depois da edição. Nesse momento, o texto anterior do `verificacao.md` já foi perdido. Sem ele, não há como dizer se o comando, o tipo, o `Trecho:` ou a `Falha:` de cada evidência mudaram (RF-04, RF-05).

**Alternativas comparadas.**

| Opção | Onde fica | Prós | Contras |
|---|---|---|---|
| (a) Cópia do `verificacao.md` | `evidencias/baseline/verificacao.md`, gravada no `vostok task start` e regravada no fim de cada `adjust record` e `record-manual` com a tarefa em execução | Guarda tudo, inclusive os subitens; acompanha a recoleta do baseline, que move a pasta inteira | Um arquivo a mais na pasta do baseline |
| (b) Impressões por evidência no `estado.yml` | Um campo com `nome=hash do comando:hash do Trecho` por evidência | Nada novo em disco | Muda o formato do estado; não diz qual subitem mudou sem um hash por subitem |

**Escolha: (a).** A cópia é simples, não muda o formato do estado e vive junto do baseline. O baseline já é o retrato da tarefa no início da execução.

**Sem a cópia.** Isso vale para uma tarefa iniciada antes desta versão. O ajuste não imprime a linha `Conferências:` e segue como hoje: as conferências continuam valendo pelo hash da saída (SPEC-046).

## Conferência derrubada e marca

**Hoje.** Depois de um ajuste, a evidência é gerada de novo. A conferência continua valendo se o hash da saída for igual. Por isso, uma evidência cujo comando mudou e que, por acaso, produz a mesma saída manteria a conferência sem que o usuário tenha visto o comando novo.

**Derrubada.** Quando o comando ou o tipo da evidência mudou, ou quando ela foi removida, o ajuste faz duas coisas:

- tira a linha dela do `conferencias.md`;
- grava no `registro.md` a linha `conferência | ajuste | EV-NN derrubada: comando mudou`, com o motivo `comando mudou`, `tipo mudou` ou `evidência removida`.

Gravar o estado `retirada` foi descartado. A retirada é uma decisão do usuário, e a conferência é sempre dele (RN-04).

**Mantida se a saída não mudar.** Quando só mudam `Título:`, `Prova:`, `Trecho:`, `Falha:` ou outro subitem, a conferência fica. Ela cai sozinha se a saída gerada de novo mudar (RN-02).

**Marca.** Uma conferência `conferida` cujo `Trecho:` ou `Falha:` mudou recebe o sufixo ` · conferida sob o trecho anterior (ajuste N)` no Texto, que é a coluna do aval no `conferencias.md`. A marca de um ajuste anterior é trocada, nunca acumulada. O formato exato é `conferida sob o trecho anterior (ajuste N)`. Uma coluna nova foi descartada, porque quebraria a leitura dos `conferencias.md` existentes. O painel separa a marca do aval pela expressão `/ · conferida sob o trecho anterior \(ajuste (\d+)\)$/` e a mostra numa linha própria.

**Tarefa em arquivo único.** Ela não tem `conferencias.md`, por isso fica fora da marca e da derrubada. Recebe só a previsão na saída do ajuste.

## Paridade das buscas

**Onde fica a tabela.** A tabela fica em `apps/painel/src/features/task/snippet-cases.json`. É uma lista de objetos `{"name", "log", "snippet", "lines", "unsearchable"}`, com `lines` contadas a partir de 0 e `unsearchable` igual ao motivo ou `""`.

**Como os dois lados a leem.**

- **Vitest:** `import cases from './snippet-cases.json'`. Isso exige `"resolveJsonModule": true` no `apps/painel/tsconfig.json`; hoje o painel não tem tipos do Node para ler o arquivo com `fs`.
- **`cargo test`:** `include_str!("../../painel/src/features/task/snippet-cases.json")` no módulo de testes de `apps/cli/src/support.rs`, lido com `serde_json`, que já é dependência.

Gerar os casos de um lado para o outro foi descartado, porque pediria um passo de build.

**Busca em Rust sem regex.** A regra do painel usa `(?<![\p{L}\p{N}])` e `(?![\p{L}\p{N}])`. O `std` não tem regex, e a crate `regex` não tem lookbehind. A busca é escrita à mão:

- acha cada ocorrência da primeira parte;
- confere o caractere anterior com `char::is_alphanumeric`;
- procura as partes do meio em ordem;
- confere o caractere seguinte da última parte.

Acrescentar `fancy-regex` foi descartado: seria uma dependência nova só para duas regras de vizinhança.

**Onde `is_alphanumeric` e `\p{L}\p{N}` podem divergir.** `is_alphanumeric` cobre `Alphabetic` e `Numeric`. `Alphabetic` inclui alguns sinais combinantes, que não são `\p{L}`. A tabela precisa ter ao menos estes três casos:

1. **Acento:** `ação` acha a linha `ação concluída`, e `ção` não acha. O acento pré-composto é letra nos dois lados.
2. **Símbolo:** `×` não acha `1280×720`. O `×` não é letra nem número nos dois lados, mas os números colados nele impedem o limite de palavra.
3. **Número colado:** `exit=` não acha `exit=0`, e `rc` acha `rc=0`. Depois de `=` vem um número no primeiro caso; no segundo, o `=` não é letra nem número.

**Textos que não dá para buscar.** `unsearchable` e `snippetsOf` vão para `snippet-match.ts`, com reexportação em `snippet-checks.ts`. Assim a tabela cobre também `+` solto e a crase escapada, sem mudar quem importa.

## Impacto nas tarefas

| Tarefa | O que segue deste estudo | Situação |
|---|---|---|
| TASK-02 | Tabela em `snippet-cases.json`, `resolveJsonModule`, busca em Rust sem regex, os três casos de divergência e a mudança de `unsearchable` para `snippet-match.ts` | Já escrita assim |
| TASK-03 | `tests_code_hash` (só "Mudar") e `tests_checks_hash` (Baseline e Verificação); tarefa sem os campos zera tudo | Já escrita assim |
| TASK-04 | Cópia em `evidencias/baseline/verificacao.md`; derrubada tira a linha e registra; marca como sufixo do Texto, trocando a anterior; arquivo único só com a previsão | Já escrita assim |
| TASK-05 | Usa `find_lines`, `unsearchable` e `snippets_of` da TASK-02; nas validações, só a Falha contra `evidencias/baseline/evidence/` | Já escrita assim |
| TASK-06 | Descreve no passo 9 da `adjust-task` a manutenção dos testes, a previsão, a derrubada e a marca; descreve na `approve-plan` a regra da busca desta seção | Já escrita assim |
