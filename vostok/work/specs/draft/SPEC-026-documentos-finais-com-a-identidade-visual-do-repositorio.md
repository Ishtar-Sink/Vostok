---
id: SPEC-026
title: "Documentos finais com a identidade visual do repositório"
status: draft
author: "Victor Batalha"
created_at: 2026-11-01T22:49:30Z
updated_at: 2026-11-01T22:49:30Z
tags: []
---

# Documentos finais com a identidade visual do repositório

Permite customizar o visual dos documentos finais: o usuário informa a cor principal ou a paleta de cores por uma skill, e os documentos executivo, técnico e de evidências passam a sair com a identidade visual do repositório.

## Contexto

Recurso novo, pedido durante a execução da SPEC-025. A SPEC-025 refaz o visual dos documentos finais com a identidade do Ishtar Sink; esta spec deixa cada repositório que usa o harness aplicar a própria identidade, em vez de todos os documentos saírem com as cores do Ishtar Sink. Como a SPEC-025 já estava planejada, o recurso virou uma spec própria, a ser feita depois dela.

## Recursos

- Uma skill em que o usuário informa a cor principal ou a paleta de cores do repositório.
- Os documentos finais gerados com a identidade visual informada.

## Detalhes Técnicos

- Depende da SPEC-025: a TASK-05 leva o visual escolhido para os modelos de `packages/harness/templates/finalize/`, com as cores em `packages/harness/config/documents.yml`.

## Roteiro de Testes

_Se aplicável e/ou conhecido, insira o roteiro de testes. Não preencher fará o agente inferir com base no restante das informações. Pode ser ajustado posteriormente._

## Critérios de Aceite
- [ ] O usuário consegue informar, por uma skill, a cor principal ou a paleta de cores do repositório.
- [ ] Os documentos finais saem com a identidade visual informada.
