---
name: update-vostok-projects
description: "Lista e atualiza os projetos que usam o vostok, cadastrados em projetos-vostok.yml na raiz deste repositório: mostra a versão do harness instalado em cada um, compara com a CLI no PATH e roda vostok update em todos ou nos projetos citados, sem commit. Use quando o usuário pedir para listar, conferir ou atualizar os projetos que usam o vostok, \"atualiza os projetos\", \"instala em todos os projetos\", \"quais projetos estão desatualizados\" ou para cadastrar um projeto novo na lista."
allowed-tools: Read Edit Bash(scripts/projetos-vostok/*) Bash(vostok *)
metadata:
  version: "1.0.0"
  author: "Victor Batalha"
  scripts: "vostok context load, vostok --version, vostok commit check"
  templates: "templates/responses/question.md templates/responses/blocker.md templates/responses/completion.md"
---

# Atualizar projetos do vostok

Mostra em que versão do harness está cada projeto que usa o vostok e leva os escolhidos à versão da CLI instalada, sem commit nesses projetos.

## Termos

- **Lista de projetos:** o arquivo `projetos-vostok.yml` na raiz deste repositório, fora do git, com um item por projeto (`nome`, `tipo` e `caminho`). O modelo versionado é o `projetos-vostok.example.yml`, só com exemplos neutros.
- **Projeto:** item de `tipo: projeto`, uma pasta com `vostok/`.
- **Pasta de projetos:** item de `tipo: pasta`; cada subpasta com `vostok/.manifest` vira um projeto com o nome `<nome>/<subpasta>`.
- **Variável de caminho:** a variável de ambiente citada em `caminho` como `${VARIAVEL}`, definida só no `.env` (não versionado); o `.env.example` não a lista, para não revelar os projetos pessoais.
- **Scripts:** `scripts/projetos-vostok/listar.sh` e `scripts/projetos-vostok/atualizar.sh`, que leem a lista, carregam o `.env` e aceitam nomes de projetos como filtro.

## Regras

[update-vostok-projects-01]: Caminhos absolutos ficam só no `.env`. Na lista de projetos, `caminho` é uma variável de caminho ou um caminho relativo a este repositório ([project/security-03]).
[update-vostok-projects-02]: O agente não lê o `.env` ([project/security-01]): quem o carrega são os scripts. Para cadastrar uma variável, acrescente a linha ao fim do `.env` sem abrir o arquivo.
[update-vostok-projects-03]: Na resposta, cite os projetos pelo nome, nunca pelo caminho absoluto ([project/security-02]). Nunca copie nomes da lista de projetos para arquivos versionados: o modelo só tem exemplos neutros.
[update-vostok-projects-04]: Não faça commit, stage nem push nos projetos atualizados ([agent/boundaries-01], [agent/boundaries-02]).
[update-vostok-projects-05]: Só rode o `atualizar.sh` com o pedido do usuário para atualizar; listar e conferir usam só o `listar.sh`.

## Checklist

Copie e marque o progresso:

```
- [ ] 1. Carregar o contexto
- [ ] 2. Cadastrar o projeto novo, se pedido
- [ ] 3. Listar os projetos
- [ ] 4. Atualizar os projetos
- [ ] 5. Responder ao usuário
```

## Passos

### 1. Carregar o contexto

Execute:

```bash
vostok context load --skill update-vostok-projects
```

### 2. Cadastrar o projeto novo, se pedido

Só quando o usuário pedir para acrescentar um projeto ou uma pasta de projetos. Sem o nome ou o caminho, **PARE** e pergunte com o template `question`.

1. Escolha o nome da variável: `VOSTOK_PROJETO_<NOME>` para um projeto e `VOSTOK_PASTA_<NOME>` para uma pasta, em `SCREAMING_SNAKE` ([project/naming-convention-03]).
2. Acrescente `<VARIAVEL>=<caminho>` ao fim do `.env` com `printf '%s\n' '<VARIAVEL>=<caminho>' >> .env`, sem ler o arquivo.
3. Não acrescente a variável ao `.env.example`: o nome dela revela o projeto.
4. Acrescente o item à lista de projetos, com `caminho: ${<VARIAVEL>}`.

### 3. Listar os projetos

Execute:

```bash
scripts/projetos-vostok/listar.sh
```

Com nomes citados pelo usuário, passe-os como argumentos; o nome de uma pasta de projetos vale para todos os projetos dela. A saída traz a versão da CLI no PATH e a tabela `| Projeto | Harness | Situação | Pendências no git |`.

- Linha com `erro:` (variável não definida, pasta não encontrada, sem `vostok/`): cite-a na resposta com o que corrigir no `.env` ou na lista de projetos, e siga com os demais.
- `lista de projetos não encontrada`: **PARE** e responda com o template `blocker`, pedindo ao usuário que copie o modelo e preencha a lista.
- `vostok não encontrado no PATH`: **PARE** e responda com o template `blocker`.
- A CLI no PATH mais antiga que a versão em `apps/cli/Cargo.toml`: avise que é preciso instalá-la ([project/release-local-01]) antes de atualizar.

Sem pedido de atualização, vá para o passo 5.

### 4. Atualizar os projetos

Com o pedido do usuário ([update-vostok-projects-05]), execute, com os nomes citados ou sem nenhum para todos:

```bash
scripts/projetos-vostok/atualizar.sh [nome...]
```

O script roda `vostok update` em cada projeto e imprime `| Projeto | Antes | Depois | Resultado | Pendências no git |` e, em "## Detalhes", as configurações acrescentadas e o fim da saída dos projetos com erro. Código de saída 1 quando algum projeto falhou: cite a falha na resposta, sem corrigir o projeto alheio.

### 5. Responder ao usuário

Execute `vostok commit check` e responda com o template `completion`:

- **O que foi feito:** a tabela do script, sem alteração, e as configurações acrescentadas.
- **Pendências:** os projetos com erro, as pendências no git dos atualizados (o harness novo fica sem commit em cada projeto) e o aviso de reiniciar os painéis abertos, que seguem no binário antigo.
