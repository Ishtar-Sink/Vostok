---
name: suggest-harness-feature
description: "Sugere um recurso novo que o próprio harness Vostok ainda não tem (CLI vostok, skills, regras, templates e configuração distribuídos), nunca do projeto do usuário: monta a sugestão a partir do template em work/suggestions/, troca dados do projeto por exemplos neutros, mascara segredos, valida com vostok suggest check, mostra a sugestão inteira e, só com a autorização do usuário, cria a issue [harness-feature] no repositório do harness com gh ou entrega o link do vostok suggest link. Use quando o usuário pedir /suggest-harness-feature ou para sugerir um recurso novo do harness ou do vostok. Não use para melhorias de algo que já existe (suggest-harness-improvement) nem para defeitos (report-harness-bug)."
allowed-tools: Read Write Edit Bash(vostok *) Bash(gh *) Bash(curl *) Bash(uname *) Bash(date *)
metadata:
  version: "1.0.0"
  author: "Victor Batalha"
  scripts: "vostok context load, vostok --version, vostok suggest check, vostok suggest link, gh auth status, gh issue create, curl"
  templates: "templates/suggestion/feature.md templates/responses/question.md templates/responses/blocker.md templates/responses/completion.md"
---

# Sugerir recurso do harness

Transforma um recurso novo que o próprio harness Vostok ainda não tem numa sugestão estruturada e neutra e, com a autorização do usuário, numa issue do repositório do harness.

## Termos

- **Harness:** o que o Vostok distribui: a CLI `vostok`, as skills padrão (`skills/default/`), as regras do agente (`rules/agent/`), os templates e a configuração. As regras em `rules/project/`, as skills em `skills/custom/` e o código do projeto são do usuário, não do harness.
- **Sugestão:** o arquivo `vostok/work/suggestions/<AAAA-MM-DD>-<slug>.md`, criado a partir de `templates/suggestion/feature.md`, com front-matter (`type: feature`, `issue`, `title`, `created_at`) e as seções Versão, Problema ou necessidade, Recurso proposto, Exemplo de uso e Alternativas consideradas.
- **Repositório do harness:** o valor de `harness.repository` em `vostok/config/main.yml` (por exemplo, `Ishtar-Sink/Vostok`).
- **Neutralizar:** trocar dados do projeto do usuário (caminhos, nomes, trechos de código, saídas, IDs e títulos de specs) por exemplos neutros que ilustram o recurso, e mascarar segredos com `***`.
- **Autorização:** a resposta do usuário, numa mensagem dele, liberando o envio da versão da sugestão que acabou de ver.
- **Corpo:** a sugestão sem o front-matter e sem os comentários `<!-- -->`. É o texto que vai para a issue.

## Regras

[suggest-harness-feature-01]: Sugira só recursos do harness. Pedido sobre o projeto do usuário (código, regras de `rules/project/`, skills de `skills/custom/`, ferramentas de terceiros): **PARE**, explique que a skill trata só do harness Vostok e não crie sugestão.
[suggest-harness-feature-02]: A sugestão leva só dados do harness. Antes de mostrá-la, troque caminhos, nomes, trechos de código e saídas do projeto por exemplos neutros (`/caminho/do/projeto`, `src/exemplo.rs`, `SPEC-001`, `minha-feature`), troque o diretório pessoal por `~` e mascare segredos, tokens, senhas, e-mails, IPs e nomes de máquinas com `***`. Nunca leia `.env`, chaves ou credenciais para montar a sugestão.
[suggest-harness-feature-03]: Nada vai para o GitHub sem autorização: mostre a sugestão inteira e **PARE** até o usuário autorizar. A autorização vale só para a versão mostrada; qualquer mudança na sugestão exige mostrar de novo e pedir de novo (RN-01).
[suggest-harness-feature-04]: Esta skill só cria a issue da sugestão. Nunca comente, edite, rotule nem feche issues por ela.
[suggest-harness-feature-05]: A sugestão fica em `vostok/work/suggestions/` mesmo se o usuário recusar o envio; nesse caso nada é publicado.
[suggest-harness-feature-06]: O `vostok suggest check` confere a forma da sugestão, não o conteúdo. Ele não substitui a neutralização nem a revisão do usuário.
[suggest-harness-feature-07]: Use o `gh` só quando `gh auth status` terminar com sucesso. `gh` ausente ou sem autenticação segue pelo link do `vostok suggest link`.
[suggest-harness-feature-08]: Não faça commit nem stage da sugestão ([agent/boundaries-01], [agent/boundaries-02]).
[suggest-harness-feature-09]: Toda pergunta ao usuário usa o template `question`: opções identificadas por letras a partir de A, a recomendada em primeiro lugar e marcada com "(recomendada)" no fim da descrição, e resposta livre aceita. Mesmo um pedido de dado (como o número de uma issue) usa o template, com as opções do que fazer a seguir.

## Checklist

Copie e marque o progresso:

```
- [ ] 1. Carregar o contexto
- [ ] 2. Confirmar que é um recurso do harness
- [ ] 3. Montar a sugestão
- [ ] 4. Validar a sugestão
- [ ] 5. Pedir a autorização
- [ ] 6. Criar a issue
- [ ] 7. Gravar o número da issue
- [ ] 8. Responder ao usuário
```

## Passos

### 1. Carregar o contexto

Execute:

```bash
vostok context load --skill suggest-harness-feature
```

### 2. Confirmar que é um recurso do harness

Reúna o que se sabe do pedido: a necessidade, o que o harness faz hoje a respeito e o que faltaria. Confira a que ele se refere:

| Pedido | Skill |
|---|---|
| Recurso que o harness não tem | Esta skill (`suggest-harness-feature`) |
| Melhoria de algo que o harness já faz | `suggest-harness-improvement` |
| Defeito: o harness faz algo diferente do esperado | `report-harness-bug` |
| Algo do projeto do usuário (código, regras de `rules/project/`, skills de `skills/custom/`) | Nenhuma: **PARE** e explique que a skill trata só do harness Vostok ([suggest-harness-feature-01]) |

Na dúvida, pergunte com o template `question` antes de montar a sugestão ([suggest-harness-feature-09]), com a recomendada marcada:

| Opção | Descrição |
|---|---|
| A | Seguir com esta skill, como recurso novo do harness (recomendada) |
| B | Usar `suggest-harness-improvement`, para uma melhoria |
| C | Usar `report-harness-bug`, para um defeito |
| D | Encerrar sem sugestão |

### 3. Montar a sugestão

1. Leia `harness.repository` em `vostok/config/main.yml`.
2. Colete a versão e o sistema operacional:

   ```bash
   vostok --version
   uname -sr
   date -u +%Y-%m-%dT%H:%M:%SZ
   ```

3. Copie `vostok/templates/suggestion/feature.md` para `vostok/work/suggestions/<AAAA-MM-DD>-<slug>.md`, com a data de hoje e um slug curto em kebab-case do resumo (por exemplo, `2026-10-17-task-export-em-json.md`). Crie a pasta se não existir.
4. Preencha a sugestão, sem deixar variável de template:

   | Campo | Conteúdo |
   |---|---|
   | `title` | `[harness-feature] ` seguido de um resumo curto do recurso. |
   | `created_at` | A data e hora UTC coletadas. |
   | `issue` | Fica `null` até a issue existir. |
   | Versão | Saída de `vostok --version` e de `uname -sr`, e a ferramenta do agente quando importar. |
   | Problema ou necessidade | O que o usuário não consegue fazer hoje e por que importa. |
   | Recurso proposto | O que o harness passaria a fazer. |
   | Exemplo de uso | Um exemplo neutro do recurso em uso, como o comando ou o fluxo esperado. |
   | Alternativas consideradas | Outras formas avaliadas e contornos usados hoje, ou "Nenhuma". |

5. Neutralize a sugestão inteira ([suggest-harness-feature-02]): releia cada seção e troque todo dado do projeto. Mantenha só o que o harness produziu ou recebeu, reduzido ao mínimo que ilustra o recurso.

### 4. Validar a sugestão

Execute:

```bash
vostok suggest check vostok/work/suggestions/<arquivo>.md
```

Com problemas (código 1), corrija cada linha listada e execute de novo, até o comando aceitar a sugestão.

### 5. Pedir a autorização

Mostre a sugestão inteira, como está no arquivo, e o repositório de destino. **PARE** e pergunte com o template `question` ([suggest-harness-feature-03], [suggest-harness-feature-09]):

| Opção | Descrição |
|---|---|
| A | Enviar a sugestão como está para `<harness.repository>` (recomendada) |
| B | Ajustar a sugestão antes (diga o que mudar) |
| C | Não enviar; a sugestão fica só em `vostok/work/suggestions/` |

- **A:** siga para o passo 6.
- **B:** aplique as mudanças, neutralize de novo, volte ao passo 4 e mostre a sugestão outra vez.
- **C:** não publique nada ([suggest-harness-feature-05]) e vá para o passo 8.

### 6. Criar a issue

Execute:

```bash
gh auth status
```

**Com `gh` autenticado:** grave o corpo (a sugestão sem o front-matter e sem os comentários) num arquivo temporário fora do projeto, como `/tmp/harness-feature-<slug>.md`, e execute:

```bash
gh issue create --repo <harness.repository> --title "<title do front-matter>" --body-file /tmp/harness-feature-<slug>.md
```

O comando imprime o endereço da issue (`https://github.com/<repositório>/issues/<N>`); o número é o `<N>` do fim.

**Sem `gh` ou sem autenticação:** execute:

```bash
vostok suggest link vostok/work/suggestions/<arquivo>.md
```

Mostre o link ao usuário para ele abrir no navegador e criar a issue. Se o comando avisar que a sugestão passa do limite do link, o link traz só o título: mostre também o corpo, para o usuário colar na issue. **PARE** e peça o número da issue criada, com o template `question` ([suggest-harness-feature-09]):

| Opção | Descrição |
|---|---|
| A | Informar o número da issue criada (recomendada) |
| B | Deixar o número para depois; a sugestão fica com issue: null |

Com o número, confira a issue pela API pública (veja "Ler uma issue sem gh") e verifique se o título começa com `[harness-feature]`.

### 7. Gravar o número da issue

Troque `issue: null` pelo número no front-matter da sugestão (por exemplo, `issue: 42`). O usuário não informou o número: deixe `null` e registre a pendência na resposta.

### 8. Responder ao usuário

Responda com o template `completion`:

- **O que foi feito:** o caminho da sugestão e, quando houver, o número e o endereço da issue (`https://github.com/<harness.repository>/issues/<N>`). Na recusa, diga que nada foi publicado.
- **Verificação:** o resultado do `vostok suggest check` e do envio (`gh issue create` ou link entregue).
- **Pendências:** o número da issue, quando o usuário ainda não o informou.

## Ler uma issue sem gh

Sem `gh`, leia uma issue e os comentários dela pela API pública do GitHub, sem autenticação (limite de 60 requisições por hora):

```bash
curl -s https://api.github.com/repos/<harness.repository>/issues/<N>
curl -s https://api.github.com/repos/<harness.repository>/issues/<N>/comments
```

O primeiro traz `title`, `state`, `body` e `html_url`; o segundo, a lista de comentários com `body` e `user.login`. Uma resposta com `"message": "Not Found"` indica que a issue não existe. Para comentar sem `gh`, entregue ao usuário o texto e o link `html_url` da issue, para ele comentar à mão.

## Formatos de resposta

Respostas ao usuário usam os templates de `templates/responses/`. A sugestão é mostrada no passo 5 como está no arquivo, dentro de um bloco `markdown`, seguido da pergunta do template `question`.
