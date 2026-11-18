# Leitura de diff

## Problema

O usuário tem dificuldade com os dois formatos que a aba "Mudanças" da SPEC-063 vai oferecer:

- **Lado a lado:** os olhos saltam entre duas colunas para achar o que mudou. A linha antiga fica à esquerda, a nova à direita, e quem lê precisa guardar uma enquanto procura a outra.
- **Unificada:** as linhas `-` e `+` se intercalam numa coluna só, e a palavra trocada não se destaca. Numa troca de `0.7.0` por `0.7.1`, as duas linhas inteiras aparecem quase iguais, e o leitor precisa compará-las caractere a caractere.

O objetivo deste estudo é escolher um terceiro formato para a aba "Mudanças", ao lado do lado a lado e da unificada (RF-01, RF-08). O formato precisa ser montado só com o texto da diff unificada de cada arquivo que a rota devolve (`hunks`), e a escolha vira mais um valor da chave `vostok.diff.formato` no navegador.

## Base de pesquisa

### Carga cognitiva

- A memória de trabalho é limitada, e o esforço gasto em atividades que não levam à compreensão deixa menos capacidade para entender o material [Sweller 1988].
- Quando duas fontes de informação só fazem sentido juntas e aparecem separadas no espaço, o leitor precisa integrá-las de cabeça, o que aumenta a carga; integrar as duas num só lugar reduz esse custo. É o efeito da atenção dividida [Chandler e Sweller 1991]. No lado a lado, a linha antiga e a nova são essas duas fontes separadas.
- O mesmo estudo mostra que acrescentar material que parece útil, mas não é essencial, pode piorar o aprendizado mesmo quando o formato é integrado [Chandler e Sweller 1991]. Na diff, isso vale para linhas de contexto que não ajudam a entender a mudança.
- A capacidade da memória de curto prazo fica em torno de quatro blocos de informação, e não sete, como se costumava dizer [Cowan 2001].
- Em revisão de código, a capacidade de memória de trabalho do revisor se associa à eficácia em achar defeitos espalhados por várias partes da mudança, e mudanças pequenas são revisadas com mais eficácia [Baum e outros 2019].

### Dislexia

- Num estudo com 48 pessoas com dislexia, com rastreamento ocular, fontes sem serifa, monoespaçadas e romanas melhoraram a leitura, e o itálico a piorou [Rello e Baeza-Yates 2013].
- O guia de estilo da British Dyslexia Association reúne princípios para tornar o texto mais fácil de ler para pessoas com dislexia, como fonte sem serifa, bom espaçamento e texto organizado em partes curtas [BDA 2023].
- A nota do W3C sobre pessoas com deficiências cognitivas e de aprendizagem recomenda conteúdo fácil de entender, com a informação importante em destaque e sem obrigar o leitor a guardar informação de um ponto para outro [W3C 2021].
- A WCAG pede que a cor não seja o único meio de transmitir informação (critério 1.4.1) e que o texto continue legível quando o leitor aumenta o espaçamento entre linhas, letras e palavras (critério 1.4.12) [WCAG 2023].

### TDAH

- O modelo de Barkley põe a inibição comportamental no centro do TDAH e liga a ela a memória de trabalho e outras funções executivas; a revisão de evidências que ele faz encontra os déficits mais fortes em inibição e memória de trabalho [Barkley 1997].
- Uma meta-análise de 26 estudos encontrou déficits de memória de trabalho em crianças com TDAH, tanto verbal quanto espacial, que se mantêm depois de descontados os transtornos de linguagem e a capacidade intelectual geral [Martinussen e outros 2005].
- Somado ao limite de cerca de quatro blocos [Cowan 2001], isso quer dizer que um formato que obriga a guardar uma linha enquanto se procura outra custa ainda mais para quem tem TDAH.

### O que a pesquisa sugere para a diff

Ferramentas que já fogem das duas formas mostram que o caminho existe na prática, não na pesquisa: o `git diff --word-diff` marca só as palavras trocadas dentro da linha, e o difftastic compara a estrutura do código em vez das linhas e marca só o que mudou.

1. **P1, uma coluna só:** a mudança e o que ela substitui ficam no mesmo lugar, sem atenção dividida entre colunas [Chandler e Sweller 1991].
2. **P2, a mudança marcada no nível da palavra:** o leitor vê o que trocou sem comparar duas linhas quase iguais [Sweller 1988].
3. **P3, marcação que não dependa só de cor:** a troca aparece também por sinal ou forma (riscado, negrito, seta), sem itálico [WCAG 2023] [Rello e Baeza-Yates 2013].
4. **P4, o resumo antes do detalhe:** o leitor sabe o que esperar antes de entrar nas linhas, e guarda menos de cada vez [Cowan 2001] [W3C 2021].
5. **P5, blocos curtos com o contexto recolhido:** só o contexto necessário fica aberto, porque o material não essencial também custa [Chandler e Sweller 1991] [BDA 2023].

## Diff de exemplo

Os três exemplos usam o mesmo commit deste repositório: `de9321f`, "chore(release): bump vostok to 0.7.1". Ele junta um trecho de texto acrescentado e trocas de uma palavra no código.

```text
 CHANGELOG.md           | 6 ++++++
 apps/cli/Cargo.lock    | 2 +-
 apps/cli/Cargo.toml    | 2 +-
 apps/cli/tests/base.rs | 2 +-
 4 files changed, 9 insertions(+), 3 deletions(-)
```

A diff unificada original, sem as linhas `index`, para comparação:

```diff
diff --git a/CHANGELOG.md b/CHANGELOG.md
--- a/CHANGELOG.md
+++ b/CHANGELOG.md
@@ -2,6 +2,12 @@
 
 Mudanças de cada versão publicada do vostok. A versão mais recente fica no topo.
 
+## [0.7.1] - 2026-11-18
+
+### Correções
+
+- Uma spec com tarefas já executadas volta a aceitar ajustes, o registro de edições manuais e uma nova aprovação. A aprovação passa a guardar, no campo `approved_disk` da spec, quais componentes afetados existiam naquele momento, e a validação confere as ações contra esse registro, e não contra os arquivos que as tarefas criaram ou apagaram. Componentes acrescentados depois continuam conferidos contra o disco, e specs aprovadas antes desta versão que já têm tarefas aceitam o ajuste e ganham o registro na próxima aprovação. (SPEC-068, issue #22)
+
 ## [0.7.0] - 2026-11-18
 
 ### Recursos
diff --git a/apps/cli/Cargo.lock b/apps/cli/Cargo.lock
--- a/apps/cli/Cargo.lock
+++ b/apps/cli/Cargo.lock
@@ -552,7 +552,7 @@ checksum = "0b928f33d975fc6ad9f86c8f283853ad26bdd5b10b7f1542aa2fa15e2289105a"
 
 [[package]]
 name = "vostok"
-version = "0.7.0"
+version = "0.7.1"
 dependencies = [
  "anyhow",
  "assert_cmd",
diff --git a/apps/cli/Cargo.toml b/apps/cli/Cargo.toml
--- a/apps/cli/Cargo.toml
+++ b/apps/cli/Cargo.toml
@@ -1,6 +1,6 @@
 [package]
 name = "vostok"
-version = "0.7.0"
+version = "0.7.1"
 edition = "2021"
 description = "CLI do Vostok: instala o harness nos projetos e executa os comandos dele."
 license = "AGPL-3.0-only"
diff --git a/apps/cli/tests/base.rs b/apps/cli/tests/base.rs
--- a/apps/cli/tests/base.rs
+++ b/apps/cli/tests/base.rs
@@ -17,7 +17,7 @@ fn ct01_version_is_0_1_0() {
         .arg("--version")
         .assert()
         .success()
-        .stdout("vostok 0.7.0\n");
+        .stdout("vostok 0.7.1\n");
 }
 
 #[test]
```

## Proposta A: antes e depois empilhados

Uma coluna. Para cada trecho, o bloco "Antes" e logo abaixo o bloco "Depois", com as linhas inteiras, sem `+` e `-`, e a palavra trocada em negrito. Texto só acrescentado aparece só em "Depois".

**`CHANGELOG.md`, a partir da linha 5**

Depois:

- \## [0.7.1] - 2026-11-18
- \### Correções
- \- Uma spec com tarefas já executadas volta a aceitar ajustes, o registro de edições manuais e uma nova aprovação. A aprovação passa a guardar, no campo `approved_disk` da spec, quais componentes afetados existiam naquele momento, e a validação confere as ações contra esse registro, e não contra os arquivos que as tarefas criaram ou apagaram. Componentes acrescentados depois continuam conferidos contra o disco, e specs aprovadas antes desta versão que já têm tarefas aceitam o ajuste e ganham o registro na próxima aprovação. (SPEC-068, issue #22)

**`apps/cli/Cargo.lock`, linha 555**

Antes:

- name = "vostok"
- version = "**0.7.0**"
- dependencies = [

Depois:

- name = "vostok"
- version = "**0.7.1**"
- dependencies = [

**`apps/cli/Cargo.toml`, linha 3**

Antes:

- name = "vostok"
- version = "**0.7.0**"
- edition = "2021"

Depois:

- name = "vostok"
- version = "**0.7.1**"
- edition = "2021"

**`apps/cli/tests/base.rs`, linha 20**

Antes:

- .success()
- .stdout("vostok **0.7.0**\n");
- }

Depois:

- .success()
- .stdout("vostok **0.7.1**\n");
- }

**Princípios atendidos:** P1, P2, P3

**Bom para:** quem quer ver o trecho inteiro como era e como ficou, com a mesma forma de leitura em cima e embaixo.

**Custa:** cada troca de uma palavra repete o trecho duas vezes; o leitor ainda compara dois blocos (agora empilhados), e o contexto não fica recolhido.

## Proposta B: trocas em linha

Uma linha por mudança, com o valor antigo e o novo separados por uma seta, e o contexto recolhido. Trecho só acrescentado aparece como um bloco "Acrescentado" com as linhas novas.

- `CHANGELOG.md`, linhas 5 a 10, acrescentado:
  - \## [0.7.1] - 2026-11-18
  - \### Correções
  - \- Uma spec com tarefas já executadas volta a aceitar ajustes, o registro de edições manuais e uma nova aprovação. A aprovação passa a guardar, no campo `approved_disk` da spec, quais componentes afetados existiam naquele momento, e a validação confere as ações contra esse registro, e não contra os arquivos que as tarefas criaram ou apagaram. Componentes acrescentados depois continuam conferidos contra o disco, e specs aprovadas antes desta versão que já têm tarefas aceitam o ajuste e ganham o registro na próxima aprovação. (SPEC-068, issue #22)
- `apps/cli/Cargo.lock`, linha 555: `0.7.0` → `0.7.1`
- `apps/cli/Cargo.toml`, linha 3: `0.7.0` → `0.7.1`
- `apps/cli/tests/base.rs`, linha 20: `0.7.0` → `0.7.1`

**Princípios atendidos:** P1, P2, P3, P5

**Bom para:** trocas pequenas e repetidas, como um aumento de versão, que viram uma lista curta lida de cima a baixo sem comparar linhas.

**Custa:** sem o contexto, o leitor não sabe a que campo cada troca pertence (qual `version`, qual `stdout`) e precisa lembrar ou abrir o arquivo; trocas de várias palavras na mesma linha ficam difíceis de escrever como "antigo → novo".

## Proposta C: resumo em camadas

Primeiro, uma frase por arquivo em linguagem simples. Depois, recolhido sob cada frase, o detalhe numa coluna com as palavras removidas riscadas e as acrescentadas em negrito, sem depender de cor. No painel, o detalhe fica fechado até o leitor abrir a frase.

- **`CHANGELOG.md`:** 6 linhas novas com a entrada da 0.7.1.
  - Detalhe, a partir da linha 5:
    - **\## [0.7.1] - 2026-11-18**
    - **\### Correções**
    - **\- Uma spec com tarefas já executadas volta a aceitar ajustes, o registro de edições manuais e uma nova aprovação. A aprovação passa a guardar, no campo `approved_disk` da spec, quais componentes afetados existiam naquele momento, e a validação confere as ações contra esse registro, e não contra os arquivos que as tarefas criaram ou apagaram. Componentes acrescentados depois continuam conferidos contra o disco, e specs aprovadas antes desta versão que já têm tarefas aceitam o ajuste e ganham o registro na próxima aprovação. (SPEC-068, issue #22)**
- **`apps/cli/Cargo.lock`:** a versão passa de 0.7.0 para 0.7.1.
  - Detalhe, linha 555:
    - name = "vostok"
    - version = "~~0.7.0~~ **0.7.1**"
    - dependencies = [
- **`apps/cli/Cargo.toml`:** a versão passa de 0.7.0 para 0.7.1.
  - Detalhe, linha 3:
    - name = "vostok"
    - version = "~~0.7.0~~ **0.7.1**"
    - edition = "2021"
- **`apps/cli/tests/base.rs`:** a versão esperada no teste passa de 0.7.0 para 0.7.1.
  - Detalhe, linha 20:
    - .success()
    - .stdout("vostok ~~0.7.0~~ **0.7.1**\n");
    - }

**Princípios atendidos:** P1, P2, P3, P4, P5

**Bom para:** quem quer saber primeiro o que mudou em cada arquivo e só depois abrir as linhas, com a troca marcada dentro da própria linha.

**Custa:** a frase do resumo, montada só com os `hunks`, é mecânica ("1 troca: 0.7.0 → 0.7.1", "6 linhas acrescentadas"), e não a frase com sentido dos exemplos ("a versão passa de..."); o componente precisa parear remoções e acréscimos e achar as palavras trocadas, como o `diff_rows` do técnico já faz com as linhas.

## Comparação

| Proposta | Carga de memória | Depende de cor | Funciona a 400 px | Cabe nos `hunks` da rota | Custo no componente da aba |
|---|---|---|---|---|---|
| A: antes e depois empilhados | Média: guarda o bloco "Antes" enquanto lê o "Depois" | Não (negrito) | Sim, mas o trecho dobra de altura | Sim: parear `-` e `+` e achar a palavra | Médio: pareamento e diff de palavras |
| B: trocas em linha | Baixa por troca, mas alta para lembrar onde cada troca fica, sem contexto | Não (seta) | Sim, uma linha que quebra | Sim, mas trocas de várias palavras não cabem em "antigo → novo" | Médio: pareamento, diff de palavras e regra para quando a troca é longa |
| C: resumo em camadas | Baixa: uma frase por arquivo, e uma linha marcada por vez no detalhe | Não (riscado e negrito) | Sim, uma coluna com recolhimento | Sim, com a frase do resumo mecânica | Médio a alto: pareamento, diff de palavras, frase do resumo e recolhimento por arquivo |

## Recomendação

A recomendação é a **Proposta C: resumo em camadas**. Pelo critério de maior peso, a menor carga de memória, ela é a única que dá o resumo antes do detalhe (P4) e mostra a troca dentro da própria linha, com o contexto ao lado, de modo que o leitor guarda uma coisa por vez. Ela também não depende de cor (riscado e negrito), funciona a 400 px por ser uma coluna só com recolhimento e cabe nos `hunks` da rota, desde que a frase do resumo seja a mecânica ("1 troca: 0.7.0 → 0.7.1").

- A Proposta A ficou atrás porque, no critério de maior peso, o leitor ainda guarda o bloco "Antes" enquanto lê o "Depois", só que agora em cima e embaixo, e o trecho dobra de altura a 400 px.
- A Proposta B ficou atrás porque, sem o contexto, o leitor precisa lembrar a que campo cada troca pertence, o que devolve a carga de memória, e as trocas de várias palavras não cabem em "antigo → novo" montado só com os `hunks`.

## Pergunta de escolha

Qual destas propostas entra como terceiro formato da aba Mudanças: A, B ou C?

A escolha entra na SPEC-063 pela skill `adjust-spec`, antes de a tarefa da interface ser detalhada (RF-02).

## Fontes

- [Sweller 1988] Sweller, J. "Cognitive load during problem solving: effects on learning". Cognitive Science, 12(2), 257-285, 1988. https://doi.org/10.1207/s15516709cog1202_4
- [Chandler e Sweller 1991] Chandler, P.; Sweller, J. "Cognitive load theory and the format of instruction". Cognition and Instruction, 8(4), 293-332, 1991. https://doi.org/10.1207/s1532690xci0804_2
- [Cowan 2001] Cowan, N. "The magical number 4 in short-term memory: a reconsideration of mental storage capacity". Behavioral and Brain Sciences, 24(1), 87-114, 2001. https://pubmed.ncbi.nlm.nih.gov/11515286/
- [Baum e outros 2019] Baum, T.; Schneider, K.; Bacchelli, A. "Associating working memory capacity and code change ordering with code review performance". Empirical Software Engineering, 24(4), 1762-1798, 2019. https://doi.org/10.1007/s10664-018-9676-8
- [Rello e Baeza-Yates 2013] Rello, L.; Baeza-Yates, R. "Good fonts for dyslexia". ASSETS '13, ACM, 2013. https://doi.org/10.1145/2513383.2513447
- [BDA 2023] British Dyslexia Association. "Dyslexia Style Guide", 2023. https://cdn.bdadyslexia.org.uk/uploads/documents/Advice/style-guide/BDA-Style-Guide-2023.pdf
- [W3C 2021] W3C. "Making Content Usable for People with Cognitive and Learning Disabilities". W3C Working Group Note, 29 de abril de 2021. https://www.w3.org/TR/2021/NOTE-coga-usable-20210429/
- [WCAG 2023] W3C. "Web Content Accessibility Guidelines (WCAG) 2.2", critérios 1.4.1 (Uso de cor) e 1.4.12 (Espaçamento de texto). W3C Recommendation, 2023. https://www.w3.org/TR/WCAG22/
- [Barkley 1997] Barkley, R. A. "Behavioral inhibition, sustained attention, and executive functions: constructing a unifying theory of ADHD". Psychological Bulletin, 121(1), 65-94, 1997. https://doi.org/10.1037/0033-2909.121.1.65
- [Martinussen e outros 2005] Martinussen, R.; Hayden, J.; Hogg-Johnson, S.; Tannock, R. "A meta-analysis of working memory impairments in children with attention-deficit/hyperactivity disorder". Journal of the American Academy of Child and Adolescent Psychiatry, 44(4), 377-384, 2005. https://doi.org/10.1097/01.chi.0000153228.72591.73
