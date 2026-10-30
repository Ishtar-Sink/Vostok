---
spec: #{SPEC_ID}
spec_hash: #{SPEC_HASH}
content_hash: null
adjustments: 0
adjusted_at: null
created_at: #{DATETIME}
updated_at: #{DATETIME}
---

# Plano de tarefas: #{SPEC_ID} - #{SPEC_TITLE}

Documento para revisão. É removido quando o plano é aprovado.

## Visão geral

#{PLAN_OVERVIEW}

## Tarefas

| ID | Título | Descrição | Tipo | Persona | Perfil | Dependências | Onda | Cobre |
|---|---|---|---|---|---|---|---|---|
#{PLAN_TASKS}

## Componentes por tarefa

Cada tarefa lista os caminhos de "Componentes afetados" da spec que ela assume.

| ID | Componentes |
|---|---|
#{PLAN_COMPONENTS}

## Fatos de esforço

| ID | Arquivos a criar | Arquivos a alterar | Contrato público | Inferência | Risco alto |
|---|---|---|---|---|---|
#{PLAN_FACTS}

## Arquivos em comum por onda

Seção calculada pelo `vostok plan score` a partir dos componentes por tarefa. O Coordenador executa em sequência as tarefas de uma mesma linha.

| Onda | Arquivo | Tarefas |
|---|---|---|
#{PLAN_SHARED_FILES}

## Perfis de execução

| Perfil | Descrição |
|---|---|
#{PROFILE_LEVELS}

#{ADJUSTMENT_HISTORY}
