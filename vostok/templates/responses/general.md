---
name: general
description: Resposta a perguntas e pedidos que não se encaixam em nenhum outro template de resposta.
usage: |
  DIRECT_ANSWER: a resposta em prosa, em 1 a 3 frases, sem preâmbulo.
  DETAILS: opcional. Tabela para itens com três ou mais atributos; lista para itens com um ou dois; seções `##` curtas só quando houver mais de um assunto ([agent/responses-06]).
  NEXT_STEP: opcional. Uma linha, só se houver uma ação que dependa do usuário.
---

#{DIRECT_ANSWER}

#{DETAILS}

#{NEXT_STEP}
