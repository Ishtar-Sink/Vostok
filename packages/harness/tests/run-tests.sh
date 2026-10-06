#!/usr/bin/env bash
# Suíte de testes dos scripts do harness.
# Roda sobre uma cópia descartável do projeto (AGENTS.md, CLAUDE.md e packages/harness), com specs,
# memória e skills customizadas zeradas. O projeto original não é alterado.
#
# Uso: tests/run-tests.sh [--keep]
#        --keep  mantém a cópia no fim, para inspeção. Sem ele, a cópia é apagada.
# Requer bash 3.2+, git e python3 (só para preencher specs de teste).
set -uo pipefail

SOURCE_HARNESS="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SOURCE_ROOT="$(cd "$SOURCE_HARNESS/../.." && pwd)"
KEEP=0
[ "${1:-}" = --keep ] && KEEP=1

R="$(mktemp -d "${TMPDIR:-/tmp}/harness-test.XXXXXX")"
mkdir -p "$R/packages"
cp "$SOURCE_ROOT/AGENTS.md" "$SOURCE_ROOT/CLAUDE.md" "$R/"
cp -R "$SOURCE_HARNESS" "$R/packages/"
cleanup() { if [ "$KEEP" -eq 1 ]; then echo "Cópia mantida em $R"; else rm -rf "$R"; fi; }
trap cleanup EXIT

H="$R/packages/harness"
rm -rf "$H/work" "$H/.state" "$H"/memory/*.md
find "$H/skills/custom" -mindepth 1 ! -name .gitkeep -exec rm -rf {} + 2>/dev/null
for key in specs decisions issues knowledge; do
  sed -i.bak "s/^$key: .*/$key: 1/" "$H/config/sequences.yml"
done
rm -f "$H/config/sequences.yml.bak"

M="$H/scripts/default/memory"
SP="$H/scripts/default/specs"
CX="$H/scripts/default/context"
C="$H/skills/default/create-skill/scripts/create-skill.sh"
A="$H/skills/default/adjust-spec/scripts/adjust-spec.sh"
FILL="python3 $SOURCE_HARNESS/tests/fixtures/fill-refined.py"
OUT="$R/.out"
PASS=0 FAIL=0

check() { # descrição, comando que deve ter sucesso
  local desc="$1"; shift
  if "$@" > "$OUT" 2>&1; then PASS=$((PASS + 1)); else FAIL=$((FAIL + 1)); echo "FALHOU: $desc"; sed 's/^/    /' "$OUT" | head -8; fi
}
fails() { # descrição, comando que deve falhar
  local desc="$1"; shift
  if "$@" > "$OUT" 2>&1; then FAIL=$((FAIL + 1)); echo "FALHOU (deveria falhar): $desc"; sed 's/^/    /' "$OUT" | head -8; else PASS=$((PASS + 1)); fi
}
fill() { sed -i.bak "s/#{[A-Z_]*}/${2:-x}/g" "$1" && rm -f "$1.bak"; }
fm() { sed -n '2,/^---/p' "$1" | grep "^$2:" | sed "s/^$2: //"; } # valor de uma chave do front-matter
sum() { cksum < "$1"; }

cd /
rm -f "$H"/memory/*.md

# --- Contexto ---
check "load-context carrega regras e templates de resposta" \
  bash -c "'$CX/load-context.sh' | grep -q 'templates/responses/question.md'"
check "load-context de outro diretório" bash -c "cd /tmp && '$CX/load-context.sh' | grep -q 'rules/agent/memory.md'"
fails "load-extra-context sem argumentos" "$CX/load-extra-context.sh"
fails "load-extra-context com caminho inexistente" "$CX/load-extra-context.sh" nao/existe.md
mkdir -p "$H/ctx teste/sub" && echo oi > "$H/ctx teste/sub/a b.md"
check "load-extra-context com espaço e pasta aninhada" bash -c "'$CX/load-extra-context.sh' 'ctx teste' | grep -q 'ctx teste/sub/a b.md'"

# --- Memória ---
check "busca vazia" bash -c "'$M/search-memory.sh' all | grep -q 'Nenhuma entrada'"
check "init cria os 3 arquivos" "$M/add-memory.sh" --init
check "init repetido não sobrescreve" bash -c "'$M/add-memory.sh' --init | grep -vq criado; true"
fails "tipo inválido no add" "$M/add-memory.sh" foo "x"
fails "tipo inválido na busca" "$M/search-memory.sh" foo
fails "título vazio" "$M/add-memory.sh" issue "   "
fails "título com #{" "$M/add-memory.sh" issue 'Tem #{FOO}'
fails "título com quebra de linha" "$M/add-memory.sh" issue $'a\nb'
check "título com caracteres especiais" "$M/add-memory.sh" issue 'a|b & c\d "x" /tmp/á'
fails "segunda entrada com a anterior sem preencher" "$M/add-memory.sh" issue "Outra"
f="$H/memory/issues.md"
sed -i.bak 's/#{ENTRY_TAGS}/wsl, build/; s/#{ISSUE_CATEGORY}/ambiente/; s/#{ISSUE_SIGNATURE}/`EACCES: permission denied`/' "$f" && rm -f "$f.bak"; fill "$f" "-"
check "segunda entrada após preencher" "$M/add-memory.sh" issue "Porta ocupada"
sed -i.bak 's/#{ENTRY_TAGS}/rede/; s/#{ISSUE_SIGNATURE}/`EADDRINUSE :::3000`/' "$f" && rm -f "$f.bak"; fill "$f" "-"
check "índice lista as duas issues" bash -c "[ \$('$M/search-memory.sh' issue | grep -c '^ISS-') -eq 2 ]"
check "busca por trecho de erro, sem diferenciar maiúsculas" bash -c "'$M/search-memory.sh' issue 'eacces: PERMISSION' | grep -q '^## ISS-001'"
check "busca por qualquer termo retorna entradas diferentes" bash -c "[ \$('$M/search-memory.sh' all eacces 3000 | grep -c '^## ISS-') -eq 2 ]"
check "entrada com mais termos vem primeiro" bash -c "'$M/search-memory.sh' issue 3000 rede eacces | grep '^## ' | head -1 | grep -q ISS-002"
check "mostra quantos termos bateram" bash -c "'$M/search-memory.sh' issue 3000 rede eacces | head -1 | grep -qF '> Termos encontrados: 2 de 3'"
check "termo sem correspondência não impede o resultado" bash -c "'$M/search-memory.sh' issue eacces inexistentexyz | grep -q '^## ISS-001'"
check "termo vazio é ignorado" bash -c "'$M/search-memory.sh' issue '' eacces | head -1 | grep -qF '1 de 1'"
check "termos com caracteres de regex são literais" bash -c "'$M/search-memory.sh' all '.*' | grep -q Nenhuma"
check "busca por ID" bash -c "'$M/search-memory.sh' --id ISS-002 | grep -q 'Porta ocupada'"
check "--id devolve só a entrada" bash -c "[ \$('$M/search-memory.sh' --id ISS-002 | grep -c '^## ') -eq 1 ]"
fails "--id com prefixo desconhecido" "$M/search-memory.sh" --id XYZ-001
check "update de status válido" "$M/update-memory.sh" ISS-001 Status resolvido
check "update refletido no índice" bash -c "'$M/search-memory.sh' issue | grep -q 'ISS-001.*resolvido'"
fails "update de status inválido" "$M/update-memory.sh" ISS-001 Status feito
fails "update de categoria inválida" "$M/update-memory.sh" ISS-001 Categoria nada
fails "update de entrada inexistente" "$M/update-memory.sh" ISS-099 Status aberto
fails "update de campo inexistente" "$M/update-memory.sh" ISS-001 Foo bar
check "update com \\ e aspas no valor" "$M/update-memory.sh" ISS-002 Causa 'a\b "x" & \1'
check "valor gravado literalmente" grep -qF -- '- **Causa:** a\b "x" & \1' "$f"
check "update não afeta outra entrada" bash -c "grep -A9 '^## ISS-001' '$f' | grep -q 'Causa:\*\* -'"
check "status com acento" bash -c "'$M/add-memory.sh' knowledge 'K' >/dev/null && sed -i.bak 's/#{[A-Z_]*}/k/g' '$H/memory/knowledge.md' && '$M/update-memory.sh' KNW-001 Status obsoleto"
check "busca sem acento acha texto acentuado" bash -c "'$M/add-memory.sh' knowledge 'Decisão de Índice' >/dev/null && sed -i.bak 's/#{[A-Z_]*}/k/g' '$H/memory/knowledge.md' && '$M/search-memory.sh' knowledge decisao indice | grep -q 'KNW-002'"
check "busca com maiúscula acentuada em locale C" bash -c "LC_ALL=C '$M/search-memory.sh' knowledge 'DECISÃO' | grep -q 'KNW-002'"
check "slug remove acentos" bash -c "source '$H/scripts/lib/text.sh' && [ \"\$(slugify 'Ação Rápida: Índice Ñandú')\" = acao-rapida-indice-nandu ]"
for i in 1 2 3 4 5 6; do "$M/add-memory.sh" knowledge "Comum $i" >/dev/null; sed -i.bak "s/#{[A-Z_]*}/comum/g" "$H/memory/knowledge.md"; done; rm -f "$H/memory/knowledge.md.bak"
check "busca limitada a search_max_results com aviso" bash -c "out=\$('$M/search-memory.sh' knowledge comum); [ \$(grep -c '^## KNW' <<< \"\$out\") -eq 5 ] && grep -q 'Entradas omitidas: 1\.' <<< \"\$out\""
check "--id não mostra linha de termos" bash -c "! '$M/search-memory.sh' --id KNW-003 | grep -q 'Termos encontrados'"
d="$H/memory/decisions.md"; printf '# Decisões\n' > "$d"
fails "arquivo sem front-matter é recusado" "$M/add-memory.sh" decision "X"
check "arquivo sem front-matter não é alterado" bash -c "[ \$(grep -c '^## DEC' '$d') -eq 0 ]"
rm -f "$d" && "$M/add-memory.sh" --init > /dev/null
sed -i.bak 's/^decisions: .*/decisions: 1/' "$H/config/sequences.yml" && rm -f "$H/config/sequences.yml.bak"
"$M/add-memory.sh" decision "Primeira" > /dev/null; fill "$d"
sed -i.bak 's/^decisions: .*/decisions: 1/' "$H/config/sequences.yml" && rm -f "$H/config/sequences.yml.bak"
fails "ID duplicado é recusado" "$M/add-memory.sh" decision "Repetida"
sed -i.bak 's/^## DEC-001: Primeira/## DEC-001: Servidor HTTP é o único escritor do MongoDB/; s/- \*\*Tags:\*\* x/- **Tags:** dados, mongodb/' "$d" && rm -f "$d.bak"
check "cenário B com termo extra acha a decisão" bash -c "'$M/search-memory.sh' decision mongo api web servidor arquitetura preferencias | grep -q '^## DEC-001'"
printf -- '---\r\ntype: issue\r\nupdated_at: x\r\n---\r\n\r\n## ISS-001: Crlf\r\n\r\n- **Status:** aberto\r\n- **Tags:** win\r\n' > "$f"
check "CRLF: índice sem \\r" bash -c "'$M/search-memory.sh' issue | grep -q 'aberto | win\$'"
check "CRLF: update funciona" "$M/update-memory.sh" ISS-001 Status resolvido

# --- Specs ---
fails "spec com título com #{" "$SP/create-new-spec.sh" 'X #{Y}'
check "cria spec" "$SP/create-new-spec.sh" 'Painel "A" | B'
check "lista spec em draft" bash -c "'$SP/list-specs.sh' draft | grep -q 'SPEC-001'"
echo '# solta' > "$H/work/specs/draft/solta.md"
check "spec sem front-matter é ignorada na lista" bash -c "! '$SP/list-specs.sh' | grep -q solta"
fails "refinada sem tipo" "$SP/create-refined-spec.sh" SPEC-001
fails "refinada com tipo inválido" "$SP/create-refined-spec.sh" SPEC-001 epic
check "cria refinada feature" "$SP/create-refined-spec.sh" SPEC-001 feature
ref="$(ls "$H"/work/specs/refined/SPEC-001-*.md)"
check "feature tem as 14 seções e as 4 fixas" bash -c "[ \$(grep -c '^## ' '$ref') -eq 18 ]"
check "front-matter da refinada tem type" grep -qx 'type: feature' "$ref"
fails "valida com variáveis pendentes" "$SP/create-refined-spec.sh" --validate SPEC-001
cp "$ref" "$ref.orig"
$FILL "$ref" 'TEST_CASES=| CT-01 | C. | P. | R. | RF-01 |'
check "RF sem caso de teste é apontado" bash -c "'$SP/create-refined-spec.sh' --validate SPEC-001 2>&1 | grep -q 'RF-02 não aparece em .Casos de teste.'"
cp "$ref.orig" "$ref"; $FILL "$ref" 'ACCEPTANCE_CRITERIA=- [ ] CA-01: Critério (RF-01).'
check "RF sem critério de aceite é apontado" bash -c "'$SP/create-refined-spec.sh' --validate SPEC-001 2>&1 | grep -q 'RF-02 não aparece em .Critérios de aceite.'"
cp "$ref.orig" "$ref"; $FILL "$ref" 'FUNCTIONAL_REQUIREMENTS=| RF-01 | A. |\n| RF-010 | B. |' 'TEST_CASES=| CT-01 | C. | P. | R. | RF-01 |' 'ACCEPTANCE_CRITERIA=- [ ] CA-01: (RF-01)'
check "RF-01 não conta como RF-010" bash -c "'$SP/create-refined-spec.sh' --validate SPEC-001 2>&1 | grep -q 'RF-010 não aparece'"
cp "$ref.orig" "$ref"; $FILL "$ref" 'FUNCTIONAL_REQUIREMENTS=| R1 | A. |'
check "ID de requisito inválido é apontado" bash -c "'$SP/create-refined-spec.sh' --validate SPEC-001 2>&1 | grep -q 'ID de requisito funcional inválido'"
cp "$ref.orig" "$ref"; $FILL "$ref" 'AFFECTED_COMPONENTS=| `nao/existe.sh` | alterar | M. |\n| `AGENTS.md` | criar | M. |\n| `x.sh` | mover | M. |'
check "componentes inconsistentes são apontados" bash -c "out=\$('$SP/create-refined-spec.sh' --validate SPEC-001 2>&1); grep -q \"'nao/existe.sh' não existe\" <<< \"\$out\" && grep -q \"'AGENTS.md' já existe\" <<< \"\$out\" && grep -q 'ação inválida' <<< \"\$out\""
cp "$ref.orig" "$ref"; $FILL "$ref" 'RISKS='
check "tabela vazia é apontada" bash -c "'$SP/create-refined-spec.sh' --validate SPEC-001 2>&1 | grep -q 'tabela da seção .Riscos. está vazia'"
cp "$ref.orig" "$ref"; $FILL "$ref"; sed -i.bak '/^## Riscos$/,/^## Casos de teste$/{/^## Casos de teste$/!d;}' "$ref"
check "seção removida é apontada" bash -c "'$SP/create-refined-spec.sh' --validate SPEC-001 2>&1 | grep -q \"seção ausente: '## Riscos'\""
check "rascunho continua draft depois de falhas" grep -qx 'status: draft' "$H"/work/specs/draft/SPEC-001-*.md
cp "$ref.orig" "$ref"; $FILL "$ref" 'CONTRACTS=```md\n## Riscos\n```'; python3 -c "import sys; p=sys.argv[1]; s=open(p).read(); i=s.rindex('\n## Riscos\n'); open(p,'w').write(s[:i]+'\n## Riscoz\n'+s[i+11:])" "$ref"
check "título dentro de bloco de código não conta como seção" bash -c "'$SP/create-refined-spec.sh' --validate SPEC-001 2>&1 | grep -q \"seção ausente: '## Riscos'\""
cp "$ref.orig" "$ref"; $FILL "$ref"
check "valida refinada válida (crases ignoradas)" "$SP/create-refined-spec.sh" --validate SPEC-001
check "marca refinada válida" "$SP/create-refined-spec.sh" --mark-refined SPEC-001
rm -f "$ref.orig" "$ref.bak"
fails "não refina spec já refinada" "$SP/create-refined-spec.sh" SPEC-001 feature
"$SP/create-new-spec.sh" 'Análise' > /dev/null
check "cria refinada analysis" "$SP/create-refined-spec.sh" SPEC-002 analysis
ana="$(ls "$H"/work/specs/refined/SPEC-002-*.md)"
check "analysis tem tese, antítese e síntese" bash -c "grep -q '^## Tese$' '$ana' && grep -q '^## Antítese$' '$ana' && grep -q '^## Síntese$' '$ana' && ! grep -q '^## Requisitos funcionais$' '$ana'"
cp "$ana" "$ana.orig"; $FILL "$ana" 'SYNTHESIS_CONCLUSIONS=| S-01 | C. | T-01, A-01 |'
check "argumento fora da síntese é apontado" bash -c "'$SP/create-refined-spec.sh' --validate SPEC-002 2>&1 | grep -q 'T-02 (.Tese.) não é considerado'"
cp "$ana.orig" "$ana"; $FILL "$ana"; rm -f "$ana.orig"
check "marca analysis válida" "$SP/create-refined-spec.sh" --mark-refined SPEC-002
"$SP/create-new-spec.sh" 'Estudo' > /dev/null
check "cria refinada study sem requisitos" bash -c "'$SP/create-refined-spec.sh' SPEC-003 study >/dev/null && grep -q '^## Perguntas da investigação$' \$(ls '$H'/work/specs/refined/SPEC-003-*.md) && ! grep -q '^## Casos de teste$' \$(ls '$H'/work/specs/refined/SPEC-003-*.md)"
st="$(ls "$H"/work/specs/refined/SPEC-003-*.md)"; $FILL "$st"
check "marca study válida" "$SP/create-refined-spec.sh" --mark-refined SPEC-003


# --- Refinamento: comandos e hash ---
fails "--finalize antigo termina com erro" "$SP/create-refined-spec.sh" --finalize SPEC-001
check "--finalize antigo mostra o uso novo" bash -c "'$SP/create-refined-spec.sh' --finalize SPEC-001 2>&1 | grep -q -- '--mark-refined'"
check "--mark-refined grava o content_hash" bash -c "[ -n \"\$(sed -n '2,/^---/p' '$ref' | grep '^content_hash: [0-9a-f]\{64\}\$')\" ]"
check "refinada tem a seção de histórico" grep -qx '## Histórico de ajustes' "$ref"
check "rascunho marcado como refined" grep -qx 'status: refined' "$H"/work/specs/draft/SPEC-001-*.md
fails "--mark-refined repetido é recusado" "$SP/create-refined-spec.sh" --mark-refined SPEC-001
"$SP/create-new-spec.sh" 'Rascunho' > /dev/null
fails "--validate sem refinada" "$SP/create-refined-spec.sh" --validate SPEC-004
mv "$H/templates/spec/sections/risks.md" "$H/risks.md"
check "template de seção ausente é apontado" bash -c "'$SP/create-refined-spec.sh' SPEC-004 fix 2>&1 | grep -q 'template de seção não encontrado: .*/risks.md'"
mv "$H/risks.md" "$H/templates/spec/sections/risks.md"
"$SP/create-refined-spec.sh" SPEC-004 fix > /dev/null
r4="$(ls "$H"/work/specs/refined/SPEC-004-*.md)"; before="$(sum "$r4")"
fails "--validate com problemas" "$SP/create-refined-spec.sh" --validate SPEC-004
check "--validate não altera a refinada inválida" [ "$(sum "$r4")" = "$before" ]
fails "--mark-refined com problemas" "$SP/create-refined-spec.sh" --mark-refined SPEC-004
check "--mark-refined com problemas não altera a refinada" [ "$(sum "$r4")" = "$before" ]
check "--mark-refined com problemas não altera o rascunho" grep -qx 'status: draft' "$(ls "$H"/work/specs/draft/SPEC-004-*.md)"

# --- Ajuste ---
"$SP/create-new-spec.sh" 'Só rascunho' > /dev/null
check "ajuste de spec em draft informa que só refinadas são ajustadas" bash -c "'$A' --record-manual SPEC-005 2>&1 | grep -q 'ainda não foi refinada'"
fails "ajuste de spec inexistente" "$A" --validate SPEC-099
fails "ajuste sem argumentos" "$A"
fails "--record sem resumo" "$A" --record SPEC-001
check "sem edição manual" bash -c "'$A' --record-manual SPEC-001 | grep -qx 'Nenhuma edição manual.'"
before="$(sum "$ref")"
check "--validate de spec válida" "$A" --validate SPEC-001
check "--validate não altera a spec válida" [ "$(sum "$ref")" = "$before" ]

cp "$ref" "$ref.orig"
sed -i.bak '/^content_hash:/d' "$ref" && rm -f "$ref.bak"
check "sem content_hash: conta como edição manual" bash -c "'$A' --record-manual SPEC-001 | grep -q 'Ajuste manual 1 registrado'"
check "sem content_hash: hash gravado" bash -c "'$A' --record-manual SPEC-001 | grep -qx 'Nenhuma edição manual.'"
mv "$ref.orig" "$ref"

sed -i.bak 's/$/\r/' "$ref" && rm -f "$ref.bak"
check "troca de fim de linha para CRLF não conta como edição" bash -c "'$A' --record-manual SPEC-001 | grep -qx 'Nenhuma edição manual.'"
sed -i.bak 's/\r$//' "$ref" && rm -f "$ref.bak"

sed -i.bak 's/^Objetivo\.$/Objetivo alterado à mão./' "$ref" && rm -f "$ref.bak"
check "edição manual registrada" bash -c "'$A' --record-manual SPEC-001 | grep -q 'Ajuste manual 1 registrado'"
check "edição manual: adjustments 1" [ "$(fm "$ref" adjustments)" = 1 ]
check "edição manual: adjusted_at preenchido" [ "$(fm "$ref" adjusted_at)" != null ]
check "edição manual: linha manual no histórico (fora do git)" grep -qE '^\| 1 \| [0-9-]{10} \| manual \| Edição manual \|$' "$ref"
check "edição manual: hash atualizado" bash -c "'$A' --record-manual SPEC-001 | grep -qx 'Nenhuma edição manual.'"

sed -i.bak '/^## Riscos$/,/^## Casos de teste$/{/^## Casos de teste$/!d;}' "$ref" && rm -f "$ref.bak"
check "edição manual inválida: registra e lista os problemas" bash -c "out=\$('$A' --record-manual SPEC-001); grep -q 'Ajuste manual 2 registrado' <<< \"\$out\" && grep -q \"seção ausente: '## Riscos'\" <<< \"\$out\""
before="$(sum "$ref")"
fails "--validate com problemas" "$A" --validate SPEC-001
fails "--record com problemas" "$A" --record SPEC-001 "Tentativa"
check "--record recusado não altera nada" [ "$(sum "$ref")" = "$before" ]
python3 - "$ref" <<'PY'
import sys; p = sys.argv[1]; s = open(p, encoding='utf-8').read()
s = s.replace('## Casos de teste', '## Riscos\n\n| Risco | Impacto | Mitigação |\n|---|---|---|\n| Risco. | baixo | Mitigação. |\n\n## Casos de teste', 1)
open(p, 'w', encoding='utf-8').write(s)
PY
fails "resumo vazio é recusado" "$A" --record SPEC-001 "   "
fails "resumo com quebra de linha é recusado" "$A" --record SPEC-001 $'a\nb'
check "--record válido" "$A" --record SPEC-001 'RF-01 | ajustado; CB-01 novo'
check "--record: adjustments 3" [ "$(fm "$ref" adjustments)" = 3 ]
check "--record: linha agente com barra vertical escapada" grep -qF '| agente | RF-01 \| ajustado; CB-01 novo |' "$ref"
check "--record: histórico com manual, manual e agente, em ordem" bash -c "[ \"\$(grep -E '^\| [0-9]+ \| [0-9-]{10} \|' '$ref' | cut -d'|' -f2,4 | tr -d ' ')\" = \"\$(printf '1|manual\n2|manual\n3|agente')\" ]"
check "--record: hash atualizado" bash -c "'$A' --record-manual SPEC-001 | grep -qx 'Nenhuma edição manual.'"
check "--record: status continua refined" [ "$(fm "$ref" status)" = refined ]
check "--record: updated_at igual a adjusted_at" [ "$(fm "$ref" updated_at)" = "$(fm "$ref" adjusted_at)" ]

cp "$ref" "$ref.orig"
python3 - "$ref" <<'PY'
import sys; p = sys.argv[1]; s = open(p, encoding='utf-8').read()
s = s[:s.index('\n## Histórico de ajustes')] + '\n'
open(p, 'w', encoding='utf-8').write(s)
PY
before="$(sum "$ref")"
check "spec sem a seção de histórico: --record-manual recusa" bash -c "'$A' --record-manual SPEC-001 2>&1 | grep -q \"seção ausente: '## Histórico de ajustes'\""
check "spec sem a seção de histórico: nada é alterado" [ "$(sum "$ref")" = "$before" ]
mv "$ref.orig" "$ref"

sed -i.bak 's/^status: refined$/status: draft/' "$ref" && rm -f "$ref.bak"
fails "refinada com status diferente de refined" "$A" --validate SPEC-001
sed -i.bak 's/^status: draft$/status: refined/' "$ref" && rm -f "$ref.bak"

if command -v git > /dev/null 2>&1; then
  git -C "$R" init -q && git -C "$R" add -A && git -C "$R" -c user.name=t -c user.email=t@t commit -qm base
  sed -i.bak 's/^| Risco\. | baixo | Mitigação\. |$/| Outro risco. | alto | Outra. |/ ' "$ref" && rm -f "$ref.bak"
  check "edição manual versionada: resumo com as seções alteradas" bash -c "'$A' --record-manual SPEC-001 | grep -q 'Edição manual: Riscos\$'"
  git -C "$R" add -A && git -C "$R" -c user.name=t -c user.email=t@t commit -qm ajuste
  sed -i.bak 's/^Objetivo alterado à mão\.$/De novo./' "$ref" && rm -f "$ref.bak"
  check "edição em duas seções" bash -c "'$A' --record-manual SPEC-001 | grep -q 'Edição manual: .*Objetivo'"
fi

# --- Aprovação ---
AP="$H/skills/default/approve-spec/scripts/approve-spec.sh"
"$SP/create-new-spec.sh" 'Aprovar' > /dev/null
"$SP/create-refined-spec.sh" SPEC-006 feature > /dev/null
r6="$(ls "$H"/work/specs/refined/SPEC-006-*.md)"; d6="$(ls "$H"/work/specs/draft/SPEC-006-*.md)"
$FILL "$r6" && "$SP/create-refined-spec.sh" --mark-refined SPEC-006 > /dev/null
cp "$r6" "$r6.ok"
fails "aprovação sem argumentos" "$AP"
fails "aprovação de spec inexistente" "$AP" --validate SPEC-099
check "aprovação de spec em draft informa que precisa ser refinada" bash -c "'$AP' --validate SPEC-005 2>&1 | grep -q 'ainda não foi refinada'"

sed -i.bak 's/^Objetivo\.$/Objetivo mudado à mão./' "$r6" && rm -f "$r6.bak"
check "edição manual sem registro impede a aprovação" bash -c "'$AP' --validate SPEC-006 2>&1 | grep -q 'edição manual sem registro'"
cp "$r6.ok" "$r6"
sed -i.bak '/^content_hash:/d' "$r6" && rm -f "$r6.bak"
check "spec sem content_hash impede a aprovação" bash -c "'$AP' --validate SPEC-006 2>&1 | grep -q 'edição manual sem registro'"
cp "$r6.ok" "$r6"
sed -i.bak 's/^| - | Nenhuma | - |$/| 1 | Formato do log | JSON ou texto |/' "$r6" && rm -f "$r6.bak"
"$A" --record-manual SPEC-006 > /dev/null
before="$(sum "$r6")"
check "pendência aberta é listada" bash -c "'$AP' --validate SPEC-006 2>&1 | grep -qF 'pendência aberta: Formato do log: JSON ou texto'"
fails "--approve com pendência aberta" "$AP" --approve SPEC-006
check "--approve recusado não altera a refinada" [ "$(sum "$r6")" = "$before" ]
check "--approve recusado não remove o rascunho" [ -f "$d6" ]
cp "$r6.ok" "$r6"
sed -i.bak '/^## Riscos$/,/^## Casos de teste$/{/^## Casos de teste$/!d;}' "$r6" && rm -f "$r6.bak"
"$A" --record-manual SPEC-006 > /dev/null
check "spec inválida impede a aprovação" bash -c "'$AP' --validate SPEC-006 2>&1 | grep -q \"validação: seção ausente: '## Riscos'\""
cp "$r6.ok" "$r6"; rm -f "$r6.ok"

before="$(sum "$r6")"
check "--validate sem impedimentos (pendência só com '-')" "$AP" --validate SPEC-006
check "--validate não altera a refinada" [ "$(sum "$r6")" = "$before" ]
check "--approve válido remove o rascunho" bash -c "'$AP' --approve SPEC-006 | grep -q 'Rascunho removido'"
check "aprovada: status approved" [ "$(fm "$r6" status)" = approved ]
check "aprovada: approved_hash igual ao content_hash" [ "$(fm "$r6" approved_hash)" = "$(fm "$r6" content_hash)" ]
check "aprovada: approved_at igual a updated_at" [ "$(fm "$r6" approved_at)" = "$(fm "$r6" updated_at)" ]
check "aprovada: rascunho não existe mais" [ ! -e "$d6" ]
check "aprovada: continua em refined/" [ -f "$r6" ]
check "aprovação não conta como ajuste" bash -c "[ '$(fm "$r6" adjustments)' = 0 ] && ! grep -qE '^\| [0-9]+ \| 20' '$r6'"
check "aprovada: hash do conteúdo continua em dia" bash -c "'$A' --record-manual SPEC-006 | grep -qx 'Nenhuma edição manual.'"
check "list-specs approved lista a spec uma vez" bash -c "[ \$('$SP/list-specs.sh' approved | grep -c 'SPEC-006') -eq 1 ]"
check "aprovar de novo informa que já está aprovada" bash -c "'$AP' --approve SPEC-006 2>&1 | grep -q 'já está aprovada'"
fails "aprovar de novo termina com erro" "$AP" --approve SPEC-006

check "--validate da adjust-spec aceita spec aprovada" "$A" --validate SPEC-006
check "ajuste do agente em spec aprovada derruba a aprovação" bash -c "'$A' --record SPEC-006 'RF-01 ajustado' | grep -q 'Aprovação de SPEC-006 derrubada'"
check "ajuste do agente: status refined" [ "$(fm "$r6" status)" = refined ]
check "ajuste do agente: campos da aprovação null" bash -c "[ '$(fm "$r6" approved_at)' = null ] && [ '$(fm "$r6" approved_hash)' = null ]"
check "reaprovação sem rascunho avisa e aprova" bash -c "'$AP' --approve SPEC-006 | grep -q 'nada a remover' && [ \"\$(sed -n '2,/^---/p' '$r6' | grep '^status:')\" = 'status: approved' ]"
sed -i.bak 's/^Objetivo\.$/Objetivo mudado à mão./' "$r6" && rm -f "$r6.bak"
check "edição manual em spec aprovada derruba a aprovação" bash -c "'$A' --record-manual SPEC-006 | grep -q 'Aprovação de SPEC-006 derrubada'"
check "edição manual: status refined" [ "$(fm "$r6" status)" = refined ]
check "edição manual: approved_hash null" [ "$(fm "$r6" approved_hash)" = null ]

"$SP/create-new-spec.sh" 'Rascunho protegido' > /dev/null
"$SP/create-refined-spec.sh" SPEC-007 feature > /dev/null
r7="$(ls "$H"/work/specs/refined/SPEC-007-*.md)"
$FILL "$r7" && "$SP/create-refined-spec.sh" --mark-refined SPEC-007 > /dev/null
chmod a-w "$H/work/specs/draft"
if [ -w "$H/work/specs/draft" ]; then
  echo "AVISO: pasta ainda gravável (root?); caso de falha na remoção do rascunho não testado"
else
  check "falha ao remover o rascunho: erro com o caminho" bash -c "'$AP' --approve SPEC-007 2>&1 | grep -q 'rascunho não pôde ser removido'"
  check "falha ao remover o rascunho: aprovação gravada" [ "$(fm "$r7" status)" = approved ]
fi
chmod u+w "$H/work/specs/draft"

# --- Planejamento ---
PT="$H/skills/default/plan-tasks/scripts/plan-tasks.sh"
AJP="$H/skills/default/adjust-plan/scripts/adjust-plan.sh"
APP="$H/skills/default/approve-plan/scripts/approve-plan.sh"
LT="$H/scripts/default/tasks/load-task.sh"
PLANS="$H/work/plans"
TASKS="$H/work/tasks"
levels() { sed -n '/^## Tarefas$/,/^## /p' "$1" | grep '^| TASK-' | cut -d'|' -f6 | tr -d ' ' | paste -sd ' ' -; }
# Monta linhas de tabela a partir de "ID|col|col..." (uma por argumento).
rows() { local r; for r in "$@"; do printf '| %s |\\n' "$(printf '%s' "$r" | sed 's/|/ | /g')"; done | sed 's/\\n$//'; }

"$SP/create-new-spec.sh" 'Planejar' > /dev/null
"$SP/create-refined-spec.sh" SPEC-008 feature > /dev/null
r8="$(ls "$H"/work/specs/refined/SPEC-008-*.md)"
$FILL "$r8" && "$SP/create-refined-spec.sh" --mark-refined SPEC-008 > /dev/null
p8="$PLANS/SPEC-008.md"

fails "planejamento sem argumentos" "$PT"
fails "planejamento de spec inexistente" "$PT" --create SPEC-099
check "spec refined não é planejada" bash -c "'$PT' --create SPEC-008 2>&1 | grep -q \"status 'refined'; só specs com status 'approved'\""
"$AP" --approve SPEC-008 > /dev/null
cp "$r8" "$r8.ok"
sed -i.bak 's/^Objetivo\.$/Objetivo mudado à mão./' "$r8" && rm -f "$r8.bak"
check "spec aprovada alterada à mão não é planejada" bash -c "'$PT' --create SPEC-008 2>&1 | grep -q 'mudou depois da aprovação'"
cp "$r8.ok" "$r8"
sed -i.bak 's/^approved_hash: .*/approved_hash: outro/' "$r8" && rm -f "$r8.bak"
check "approved_hash diferente do content_hash não é planejado" bash -c "'$PT' --create SPEC-008 2>&1 | grep -q 'mudou depois da aprovação'"
cp "$r8.ok" "$r8"
check "nenhum plano criado nas recusas" [ ! -e "$p8" ]

check "--create cria o plano" "$PT" --create SPEC-008
check "plano: spec_hash igual ao approved_hash" [ "$(fm "$p8" spec_hash)" = "$(fm "$r8" approved_hash)" ]
check "plano: adjustments 0 e content_hash gravado" bash -c "[ '$(fm "$p8" adjustments)' = 0 ] && [ '$(fm "$p8" content_hash)' != null ]"
check "plano: histórico e níveis do config" bash -c "grep -q '^## Histórico de ajustes\$' '$p8' && grep -qF '| critica | Contrato público' '$p8'"
before="$(sum "$p8")"
check "--create com plano atualizado informa que existe" bash -c "'$PT' --create SPEC-008 2>&1 | grep -q 'já tem um plano atualizado'"
check "--create recusado não altera o plano" [ "$(sum "$p8")" = "$before" ]
fails "--validate com variáveis sem preencher" "$PT" --validate SPEC-008

# Níveis: pontuação só por arquivos a alterar nos limites (4, 5, 9, 10, 15, 16), cada peso isolado
# e as somas 3, 7, 12 e 20.
$FILL "$p8" "PLAN_TASKS=$(rows 'TASK-01|a|a|tarefa|-|-|RF-01, RF-02, CT-01' 'TASK-02|a|a|tarefa|-|-|-' 'TASK-03|a|a|tarefa|-|-|-' 'TASK-04|a|a|tarefa|-|-|-' 'TASK-05|a|a|tarefa|-|-|-' 'TASK-06|a|a|tarefa|-|-|-' 'TASK-07|a|a|tarefa|-|-|-' 'TASK-08|a|a|tarefa|-|-|-' 'TASK-09|a|a|tarefa|-|-|-' 'TASK-10|a|a|tarefa|-|-|-' 'TASK-11|a|a|tarefa|-|-|-' 'TASK-12|a|a|tarefa|-|-|-' 'TASK-13|a|a|tarefa|-|-|-' 'TASK-14|a|a|tarefa|-|-|-')" \
  "PLAN_FACTS=$(rows 'TASK-01|0|0|não|não|não' 'TASK-02|0|4|não|não|não' 'TASK-03|0|5|não|não|não' 'TASK-04|0|9|não|não|não' 'TASK-05|0|10|não|não|não' 'TASK-06|0|15|não|não|não' 'TASK-07|0|16|não|não|não' 'TASK-08|1|3|não|sim|não' 'TASK-09|0|4|sim|não|sim' 'TASK-10|4|4|sim|não|sim' 'TASK-11|2|0|não|não|não' 'TASK-12|0|2|não|sim|não' 'TASK-13|0|0|sim|não|não' 'TASK-14|0|0|não|não|sim')"
expected='simples simples moderada moderada complexa complexa critica moderada complexa critica simples simples simples simples'
check "--score grava os níveis" "$PT" --score SPEC-008
check "níveis calculados pelos pesos e limites" [ "$(levels "$p8")" = "$expected" ]
check "--score de novo resulta nos mesmos níveis" bash -c "'$PT' --score SPEC-008 >/dev/null && [ \"\$(sed -n '/^## Tarefas\$/,/^## Fatos/p' '$p8')\" = \"$(sed -n '/^## Tarefas$/,/^## Fatos/p' "$p8")\" ]"
check "--score atualiza o hash do conteúdo" bash -c "'$AJP' --record-manual SPEC-008 | grep -qx 'Nenhuma edição manual.'"
check "--validate do plano válido" "$PT" --validate SPEC-008

# Problemas do plano.
cp "$p8" "$p8.ok"
sed -i.bak 's/^| TASK-03 | a | a | tarefa | moderada | - | - |$/| TASK-03 | a | a | tarefa | simples | - | - |/' "$p8" && rm -f "$p8.bak"
check "nível gravado à mão diferente do calculado" bash -c "'$PT' --validate SPEC-008 2>&1 | grep -qF 'TASK-03: nível \"simples\" diferente do calculado \"moderada\"'"
cp "$p8.ok" "$p8"
python3 - "$p8" <<'PY'
import sys; p = sys.argv[1]; s = open(p, encoding='utf-8').read()
s = s.replace('| TASK-01 | a | a | tarefa | simples | - | RF-01, RF-02, CT-01 |', '| TASK-01 | a | a | tarefa | simples | TASK-02 | RF-01, CT-01 |')
s = s.replace('| TASK-02 | a | a | tarefa | simples | - | - |', '| TASK-02 | a | a | tarefa | simples | TASK-01, TASK-40 | - |')
s = s.replace('| TASK-13 | 0 | 0 | sim | não | não |', '| TASK-13 | 0 | x | talvez | não | não |')
open(p, 'w', encoding='utf-8').write(s)
PY
before="$(sum "$p8")"
fails "--validate com problemas" "$PT" --validate SPEC-008
"$PT" --validate SPEC-008 > "$R/.plan-errors" 2>&1
check "problema: ID da spec sem tarefa" grep -qF 'RF-02 não é coberto por nenhuma tarefa' "$R/.plan-errors"
check "problema: dependência em ciclo" grep -q 'dependência em ciclo envolvendo TASK-0[12]' "$R/.plan-errors"
check "problema: dependência inexistente" grep -qF 'TASK-02: dependência inexistente TASK-40' "$R/.plan-errors"
check "problema: fato numérico inválido" grep -qF 'TASK-13: fato "Arquivos a alterar" inválido: "x"' "$R/.plan-errors"
check "problema: fato sim/não inválido" grep -qF 'TASK-13: fato "Contrato público" inválido: "talvez"' "$R/.plan-errors"
check "--validate não altera o plano com problemas" [ "$(sum "$p8")" = "$before" ]
fails "--score com fato inválido" "$PT" --score SPEC-008
check "--score recusado não altera o plano" [ "$(sum "$p8")" = "$before" ]
cp "$p8.ok" "$p8"
python3 - "$p8" <<'PY'
import sys; p = sys.argv[1]; s = open(p, encoding='utf-8').read()
s = s.replace('| TASK-02 | a | a | tarefa | simples | - | - |', '| TASK-02 | a | a | épico | simples | - | RF-09 |\n| TASK-02 | b | b | tarefa | - | TASK-02 | - |\n| TASK-1 | c | c | bug | - | - |')
s = s.replace('| TASK-14 | 0 | 0 | não | não | sim |', '| TASK-14 | 0 | 0 | não | não | sim |\n| TASK-99 | 0 | 0 | não | não | não |')
s = s.replace('| TASK-12 | 0 | 2 | não | sim | não |\n', '')
open(p, 'w', encoding='utf-8').write(s)
PY
"$PT" --validate SPEC-008 > "$R/.plan-errors" 2>&1
check "problema: tipo inválido" grep -qF 'TASK-02: tipo inválido "épico"' "$R/.plan-errors"
check "problema: ID repetido" grep -qF 'ID de tarefa repetido: TASK-02' "$R/.plan-errors"
check "problema: dependência de si mesma" grep -qF 'TASK-02: depende de si mesma' "$R/.plan-errors"
check "problema: linha com colunas faltando" grep -qF 'linha da tabela de tarefas com 6 colunas' "$R/.plan-errors"
check "problema: tarefa sem fatos" grep -qF 'TASK-12: sem fatos de esforço' "$R/.plan-errors"
check "problema: fatos de tarefa inexistente" grep -qF 'fatos de esforço de uma tarefa inexistente: TASK-99' "$R/.plan-errors"
check "problema: cobre ID que não existe na spec" grep -qF 'a tarefa cobre RF-09, que não existe na spec' "$R/.plan-errors"
cp "$p8.ok" "$p8"; rm -f "$p8.ok"

# Plano padrão da fixture: TASK-01 (simples) e TASK-02 (moderada), cobrindo RF-01, RF-02 e CT-01.
rm -f "$p8"; "$PT" --create SPEC-008 > /dev/null; $FILL "$p8"; "$PT" --score SPEC-008 > /dev/null

# Ajuste do plano.
fails "ajuste de plano sem argumentos" "$AJP"
check "ajuste de spec sem plano" bash -c "'$AJP' --validate SPEC-006 2>&1 | grep -q 'não tem plano de tarefas'"
fails "--record sem resumo" "$AJP" --record SPEC-008
check "adjust-plan --validate de plano válido" "$AJP" --validate SPEC-008
sed -i.bak 's/^Visão geral do plano\.$/Visão geral mudada à mão./' "$p8" && rm -f "$p8.bak"
before="$(sum "$p8")"
check "edição manual do plano impede a criação das tarefas" bash -c "'$APP' --create-tasks SPEC-008 2>&1 | grep -q 'edição manual no plano sem registro; registre-a com: adjust-plan.sh --record-manual SPEC-008'"
check "--create-tasks recusado não altera nada" bash -c "[ '$(sum "$p8")' = \"\$(cksum < '$p8')\" ] && [ ! -e '$TASKS/SPEC-008' ]"
check "edição manual do plano registrada" bash -c "'$AJP' --record-manual SPEC-008 | grep -q 'Ajuste manual 1 registrado'"
check "edição manual: linha manual no histórico do plano" grep -qE '^\| 1 \| [0-9-]{10} \| manual \| Edição manual' "$p8"
sed -i.bak 's/^| TASK-02 | Documentar | Documenta o uso\. | historia | moderada | TASK-01 | RF-02 |$/| TASK-02 | Documentar | Documenta o uso. | historia | moderada | TASK-01 | - |/' "$p8" && rm -f "$p8.bak"
before="$(sum "$p8")"
check "adjust-plan --validate aponta RF sem tarefa" bash -c "'$AJP' --validate SPEC-008 2>&1 | grep -q 'RF-02 não é coberto'"
fails "--record com RF sem tarefa" "$AJP" --record SPEC-008 "Remove cobertura"
check "--record recusado não altera o plano" [ "$(sum "$p8")" = "$before" ]
sed -i.bak 's/^| TASK-02 | Documentar | Documenta o uso\. | historia | moderada | TASK-01 | - |$/| TASK-02 | Documentar | Documenta o uso. | historia | - | TASK-01 | RF-02 |/; s/^| TASK-02 | 0 | 2 | não | sim | não |$/| TASK-02 | 0 | 8 | não | sim | não |/' "$p8" && rm -f "$p8.bak"
check "--record recalcula, valida e registra" bash -c "'$AJP' --record SPEC-008 'TASK-02 com mais arquivos' | grep -q 'Ajuste 2 registrado'"
check "--record: nível recalculado" [ "$(levels "$p8")" = "simples complexa" ]
check "--record: adjustments 2 e linha agente" bash -c "[ '$(fm "$p8" adjustments)' = 2 ] && grep -qF '| agente | TASK-02 com mais arquivos |' '$p8'"
check "--record: hash atualizado" bash -c "'$AJP' --record-manual SPEC-008 | grep -qx 'Nenhuma edição manual.'"

# Criação das tarefas.
fails "aprovação de plano sem argumentos" "$APP"
check "aprovação de spec sem plano" bash -c "'$APP' --create-tasks SPEC-006 2>&1 | grep -q 'não tem plano de tarefas'"
check "--validate antes de criar as tarefas" bash -c "'$APP' --validate SPEC-008 2>&1 | grep -q 'ainda não foram criadas'"
mkdir -p "$TASKS/SPEC-008"
before="$(sum "$p8")"
check "pasta de tarefas existente é recusada" bash -c "'$APP' --create-tasks SPEC-008 2>&1 | grep -q 'pasta de tarefas de SPEC-008 já existe'"
check "recusa com pasta existente não cria tarefas" bash -c "[ -z \"\$(ls -A '$TASKS/SPEC-008')\" ] && [ '$before' = \"\$(cksum < '$p8')\" ]"
rmdir "$TASKS/SPEC-008"
check "--create-tasks cria uma tarefa por linha do plano" bash -c "'$APP' --create-tasks SPEC-008 >/dev/null && [ \"\$(ls '$TASKS/SPEC-008' | paste -sd ' ' -)\" = 'TASK-01.md TASK-02.md' ]"
t1="$TASKS/SPEC-008/TASK-01.md"; t2="$TASKS/SPEC-008/TASK-02.md"
check "tarefa: front-matter do plano" bash -c "[ '$(fm "$t2" id)|$(fm "$t2" spec)|$(fm "$t2" title)|$(fm "$t2" effort)|$(fm "$t2" depends_on)|$(fm "$t2" covers)|$(fm "$t2" type)|$(fm "$t2" labels)|$(fm "$t2" status)' = 'TASK-02|SPEC-008|\"Documentar\"|complexa|[TASK-01]|[RF-02]|historia|[SPEC-008, complexa]|pending' ]"
check "tarefa sem dependências: depends_on vazio e card sem bloqueio" bash -c "[ '$(fm "$t1" depends_on)' = '[]' ] && grep -qF -- '- **Bloqueada por:** -' '$t1' && grep -qF -- '- **Bloqueada por:** TASK-01' '$t2'"
check "tarefa: content_hash gravado ao criar" bash -c "[ -n \"\$(sed -n '2,/^---/p' '$t1' | grep '^content_hash: [0-9a-f]\{64\}\$')\" ]"
check "tarefa: arquivos com as permissões padrão" bash -c "ls -l '$t1' | grep -q '^-rw-r--r--'"
check "--validate aponta variáveis das tarefas" bash -c "'$APP' --validate SPEC-008 2>&1 | grep -qF 'TASK-01: variável sem preencher'"
fails "--mark-planned com tarefas sem preencher" "$APP" --mark-planned SPEC-008
$FILL "$t1"; $FILL "$t2"
check "--validate das tarefas preenchidas" "$APP" --validate SPEC-008

# Tarefas mal detalhadas.
cp "$t1" "$t1.ok"; cp "$t2" "$t2.ok"
python3 - "$t1" "$t2" <<'PY'
import sys, re
a, b = sys.argv[1], sys.argv[2]
s = open(a, encoding='utf-8').read()
s = s.replace('```bash\ntrue\n```', '')
s = s.replace('### Baseline\n\n- `true`\n', '### Baseline\n\nrodar tudo\n')
s = s.replace('- `saida.txt` (automática): `echo ok`', '- `saida.txt` (automatica): `echo ok`\n- `nota.png` (manual):')
s = s.replace('- [ ] O script existe (RF-01).\n\n  > Como comprovar: o arquivo `nao/existe/novo.sh` existe.', '- O script existe.\n\n- [ ] Sem comprovação.\n\n- [x] Comprovação vazia.\n\n  > Como comprovar:   \n\n- [ ] Comprovação na mesma linha. Como comprovar: abrir o arquivo.')
s = s.replace('- `AGENTS.md` (alterar): cita o script.', '- `nao/existe.md` (alterar): outro.\n- `AGENTS.md` (criar): já existe.\n- `AGENTS.md` (mover): ação.')
open(a, 'w', encoding='utf-8').write(s)
s = open(b, encoding='utf-8').read()
s = re.sub(r'(### Card\n)[\s\S]*$', r'\1\n', s)
s = s.replace('Entrega o script que permite exportar dados.', '\n'.join('Linha %d.' % i for i in range(1, 8)))
s = s.replace('covers: [RF-02]', 'covers: [RF-01]')
open(b, 'w', encoding='utf-8').write(s)
PY
before="$(sum "$r8")"
"$APP" --validate SPEC-008 > "$R/.task-errors" 2>&1
check "tarefa: 'Verificação' vazia" grep -qF "TASK-01: a subseção 'Verificação' está vazia" "$R/.task-errors"
check "tarefa: caminho a alterar inexistente" grep -qF "TASK-01: 'nao/existe.md' não existe" "$R/.task-errors"
check "tarefa: criar caminho existente" grep -qF "TASK-01: 'AGENTS.md' já existe" "$R/.task-errors"
check "tarefa: ação inválida" grep -qF "TASK-01: ação inválida para 'AGENTS.md': 'mover'" "$R/.task-errors"
check "tarefa: critério de conclusão sem checkbox" grep -qF "TASK-01: 'Concluída quando' com linha fora do formato: - O script existe." "$R/.task-errors"
check "tarefa: comprovação fora da citação recuada" grep -qF "TASK-01: 'Concluída quando' com critério sem 'Como comprovar': - [ ] Comprovação na mesma linha." "$R/.task-errors"
check "tarefa: critério de conclusão sem comprovação" grep -qF "TASK-01: 'Concluída quando' com critério sem 'Como comprovar': - [ ] Sem comprovação." "$R/.task-errors"
check "tarefa: critério de conclusão com comprovação vazia" grep -qF "TASK-01: 'Concluída quando' com critério sem 'Como comprovar': - [x] Comprovação vazia." "$R/.task-errors"
check "tarefa: 'Baseline' fora do formato" grep -qF "TASK-01: 'Baseline' com item fora do formato: rodar tudo" "$R/.task-errors"
check "tarefa: 'Baseline' sem nenhum item" grep -qF "TASK-01: 'Baseline' sem nenhum item no formato" "$R/.task-errors"
check "tarefa: 'Evidências' com rótulo inválido" grep -qF "TASK-01: 'Evidências' com item fora do formato: - \`saida.txt\` (automatica)" "$R/.task-errors"
check "tarefa: 'Evidências' manual sem descrição" grep -qF "TASK-01: 'Evidências' com item fora do formato: - \`nota.png\` (manual):" "$R/.task-errors"
check "tarefa: 'Card' vazio" grep -qF "TASK-02: a subseção 'Card' está vazia" "$R/.task-errors"
check "tarefa: campo do card ausente" grep -qF "TASK-02: campo do card ausente: 'Critérios de aceite'" "$R/.task-errors"
check "tarefa: resumo executivo longo" grep -qF 'TASK-02: o resumo executivo tem 7 linhas (máximo: 5)' "$R/.task-errors"
check "tarefa: cobertura diferente da do plano" grep -qF "TASK-02: cobertura 'RF-01' diferente da do plano 'RF-02'" "$R/.task-errors"
fails "--mark-planned com tarefas inválidas" "$APP" --mark-planned SPEC-008
check "--mark-planned recusado não altera a spec nem remove o plano" bash -c "[ '$before' = \"\$(cksum < '$r8')\" ] && [ -f '$p8' ]"
cp "$t1.ok" "$t1"; cp "$t2.ok" "$t2"
python3 - "$t1" <<'PY'
import sys; p = sys.argv[1]; s = open(p, encoding='utf-8').read()
s = s.replace('### Passos\n\n1. Criar o script.\n\n', '')
s = s.replace('\n## Registro de execução\n', '\n').replace('| # | Data | Origem | Resumo |\n|---|---|---|---|\n', '').replace('- `nao/existe/novo.sh` (criar): script novo.\n- `AGENTS.md` (alterar): cita o script.', 'Arquivos: AGENTS.md')
open(p, 'w', encoding='utf-8').write(s)
PY
cp "$t2" "$TASKS/SPEC-008/TASK-03.md"; rm -f "$t2"
"$APP" --validate SPEC-008 > "$R/.task-errors" 2>&1
check "tarefa: subseção ausente" grep -qF "TASK-01: subseção ausente: '### Passos'" "$R/.task-errors"
check "tarefa: seção fixa ausente" grep -qF "TASK-01: seção ausente: '## Registro de execução'" "$R/.task-errors"
check "tarefa: seção fixa sem a tabela" grep -qF "TASK-01: a seção '## Histórico de ajustes' está sem a tabela" "$R/.task-errors"
check "tarefa: 'Mudar' sem item no formato" grep -qF "TASK-01: 'Mudar' sem nenhum item no formato" "$R/.task-errors"
check "tarefa do plano sem arquivo" grep -qF 'TASK-02: arquivo da tarefa ausente' "$R/.task-errors"
check "arquivo de tarefa fora do plano" grep -qF 'TASK-03: tarefa que não existe no plano' "$R/.task-errors"
rm -f "$TASKS/SPEC-008/TASK-03.md"; mv "$t1.ok" "$t1"; mv "$t2.ok" "$t2"
check "--validate das tarefas corrigidas" "$APP" --validate SPEC-008
cp "$t1" "$t1.ok"
sed -i.bak 's|^- `saida.txt` (automática): `echo ok`$|Não se aplica.|' "$t1" && rm -f "$t1.bak"
check "tarefa: 'Evidências' com 'Não se aplica.' é válida" "$APP" --validate SPEC-008
sed -i.bak 's|^Não se aplica\.$|Não se aplica. Talvez.|' "$t1" && rm -f "$t1.bak"
check "tarefa: 'Não se aplica.' com texto extra é recusado" bash -c "'$APP' --validate SPEC-008 2>&1 | grep -qF \"TASK-01: 'Evidências' com item fora do formato: Não se aplica. Talvez.\""
mv "$t1.ok" "$t1"
cp "$t1" "$t1.ok"; cp "$t2" "$t2.ok"
sed -i.bak 's|^- `nao/existe/novo.sh` (criar): script novo\.$|- `nao/existe/novo.sh` (alterar): muda o script.|' "$t2" && rm -f "$t2.bak"
check "tarefa altera caminho criado pela tarefa de que depende" "$APP" --validate SPEC-008
sed -i.bak 's|^- `AGENTS.md` (alterar): cita o script\.$|- `nao/existe/outro.sh` (criar): outro.|' "$t2" && rm -f "$t2.bak"
sed -i.bak 's|^- `AGENTS.md` (alterar): cita o script\.$|- `nao/existe/outro.sh` (alterar): usa o outro.|' "$t1" && rm -f "$t1.bak"
check "tarefa não altera caminho criado por tarefa de que não depende" bash -c "'$APP' --validate SPEC-008 2>&1 | grep -qF \"TASK-01: 'nao/existe/outro.sh' não existe nem é criado por uma tarefa de que ela depende\""
mv "$t1.ok" "$t1"; mv "$t2.ok" "$t2"

# Carregar tarefa.
fails "load-task sem argumentos" "$LT"
fails "load-task de tarefa inexistente" "$LT" SPEC-008 TASK-09
"$LT" SPEC-008 TASK-02 > "$R/.task-load" 2>&1
check "load-task imprime o front-matter" bash -c "head -1 '$R/.task-load' | grep -qx -- '---' && grep -qx 'id: TASK-02' '$R/.task-load'"
check "load-task imprime a Execução" bash -c "grep -qx '## Execução' '$R/.task-load' && grep -qx '### Concluída quando' '$R/.task-load'"
check "load-task não imprime a Descrição" bash -c "! grep -q -e '^## Descrição' -e 'Resumo executivo' -e '^### Card' '$R/.task-load'"

# Critérios de conclusão.
CD="$H/scripts/default/tasks/check-done.sh"
fails "check-done sem argumentos" "$CD"
fails "check-done de tarefa inexistente" "$CD" SPEC-008 TASK-09
before="$(sum "$t2")"
check "check-done recusa critério sem marcar e lista o critério" bash -c "'$CD' SPEC-008 TASK-02 2>&1 | grep -qF -- '- [ ] O script existe (RF-01).'"
check "check-done não altera a tarefa" [ "$(sum "$t2")" = "$before" ]
cp "$t2" "$t2.ok"
sed -i.bak 's/^- \[ \] O script existe/- [x] O script existe/' "$t2" && rm -f "$t2.bak"
check "check-done com todos os critérios marcados" bash -c "'$CD' SPEC-008 TASK-02 | grep -qF 'Todos os 1 critérios de conclusão de TASK-02 estão marcados.'"
printf '\n%s\n\n%s\n' '- [ ] Outro critério.' '  > Como comprovar: abrir o arquivo.' > "$R/.done-extra"
python3 - "$t2" "$R/.done-extra" <<'PY'
import sys; p = sys.argv[1]; s = open(p, encoding='utf-8').read()
s = s.replace('existe.\n', 'existe.\n' + open(sys.argv[2], encoding='utf-8').read(), 1)
open(p, 'w', encoding='utf-8').write(s)
PY
check "check-done com um de dois critérios sem marcar" bash -c "! '$CD' SPEC-008 TASK-02 >/dev/null 2>&1 && '$CD' SPEC-008 TASK-02 2>&1 | grep -qF -- '- [ ] Outro critério.'"
python3 - "$t2" <<'PY'
import sys, re; p = sys.argv[1]; s = open(p, encoding='utf-8').read()
s = re.sub(r'(### Concluída quando\n)[\s\S]*?(\n## )', r'\1\nSem checklist.\n\2', s, count=1)
open(p, 'w', encoding='utf-8').write(s)
PY
check "check-done sem critérios de conclusão" bash -c "'$CD' SPEC-008 TASK-02 2>&1 | grep -q 'não tem critérios de conclusão'"
mv "$t2.ok" "$t2"

# Plano desatualizado.
cp "$r8" "$r8.ok"; cp "$p8" "$p8.ok"
"$A" --record SPEC-008 'RF-01 ajustado' > /dev/null && "$AP" --approve SPEC-008 > /dev/null
rm -rf "$TASKS/SPEC-008.bak"; mv "$TASKS/SPEC-008" "$TASKS/SPEC-008.bak"
check "plano desatualizado impede a criação das tarefas" bash -c "'$APP' --create-tasks SPEC-008 2>&1 | grep -q 'feito para outra versão da spec SPEC-008'"
check "plano desatualizado: adjust-plan --validate aponta" bash -c "'$AJP' --validate SPEC-008 2>&1 | head -2 | grep -q 'feito para outra versão da spec SPEC-008'"
check "plano desatualizado: --create recria e avisa" bash -c "'$PT' --create SPEC-008 | grep -q 'será recriado'"
check "plano recriado: spec_hash da nova aprovação e sem ajustes" bash -c "[ '$(fm "$r8" approved_hash)' = \"\$(sed -n '2,/^---/p' '$p8' | grep '^spec_hash:' | sed 's/^spec_hash: //')\" ] && grep -q '^adjustments: 0\$' '$p8'"
cp "$r8.ok" "$r8"; cp "$p8.ok" "$p8"; rm -f "$r8.ok" "$p8.ok"
rm -rf "$TASKS/SPEC-008"; mv "$TASKS/SPEC-008.bak" "$TASKS/SPEC-008"

# Conclusão do planejamento.
check "--mark-planned conclui" bash -c "'$APP' --mark-planned SPEC-008 | grep -q 'Plano removido'"
check "planejada: status planned e plano removido" bash -c "[ '$(fm "$r8" status)' = planned ] && [ ! -e '$p8' ]"
check "planejada: tarefas mantidas" bash -c "[ \"\$(ls '$TASKS/SPEC-008' | wc -l | tr -d ' ')\" = 2 ]"
check "planejada: adjust-spec recusa e cita o novo ciclo" bash -c "'$A' --validate SPEC-008 2>&1 | grep -q 'exige um novo ciclo'"
check "planejada: plan-tasks recusa" bash -c "'$PT' --create SPEC-008 2>&1 | grep -q \"já está com status 'planned'\""
check "planejada: approve-spec recusa" bash -c "! '$AP' --approve SPEC-008 >/dev/null 2>&1"

# Aprovação de tarefas.
AT="$H/skills/default/approve-task/scripts/approve-task.sh"
store_hash() { HARNESS_DIR="$H" bash -c 'for l in config text frontmatter markdown spec-validation adjustments; do source "$HARNESS_DIR/scripts/lib/$l.sh"; done; adjustment_store_hash "$1"' _ "$1"; }
set_status() { sed -i.bak "s/^status: .*/status: $2/" "$1" && rm -f "$1.bak"; }
cp -R "$TASKS/SPEC-008" "$R/.tasks-ok"
t3="$TASKS/SPEC-008/TASK-03.md"
check "--mark-planned grava o hash das tarefas detalhadas" "$AT" --validate SPEC-008 TASK-01 TASK-02
fails "approve-task sem argumentos" "$AT"
fails "approve-task sem spec" "$AT" --approve
check "approve-task sem tarefas nem --all" bash -c "'$AT' --approve SPEC-008 2>&1 | grep -qF 'informe as tarefas a aprovar ou --all'"
fails "approve-task com --all e IDs" "$AT" --approve SPEC-008 --all TASK-01
fails "approve-task com ID inválido" "$AT" --approve SPEC-008 ../TASK-01
fails "approve-task com spec inválida" "$AT" --approve ../SPEC-008 TASK-01
check "approve-task de tarefa inexistente" bash -c "'$AT' --approve SPEC-008 TASK-09 2>&1 | grep -qF 'tarefa TASK-09 não encontrada em SPEC-008'"
check "approve-task de spec sem tarefas" bash -c "'$AT' --approve SPEC-006 TASK-01 2>&1 | grep -qF 'a spec SPEC-006 não tem tarefas'"
before="$(sum "$t1")"
check "--validate não altera a tarefa" bash -c "'$AT' --validate SPEC-008 TASK-01 >/dev/null && [ '$before' = \"\$(cksum < '$t1')\" ]"

# Impedimentos: edição manual, tarefa inválida e status fora de pending, todos listados.
cp "$t2" "$t3"; sed -i.bak 's/^id: TASK-02$/id: TASK-03/' "$t3" && rm -f "$t3.bak"; store_hash "$t3"
sed -i.bak 's/^1\. Criar o script\.$/1. Criar o script à mão./' "$t2" && rm -f "$t2.bak"
python3 - "$t3" <<'PY'
import sys, re; p = sys.argv[1]; s = open(p, encoding='utf-8').read()
s = re.sub(r'(### Passos\n\n)[\s\S]*?(\n### )', r'\1\2', s, count=1); open(p, 'w', encoding='utf-8').write(s)
PY
store_hash "$t3"; set_status "$t1" approved
before="$(cat "$t1" "$t2" "$t3" | cksum)"
"$AT" --validate SPEC-008 TASK-01 TASK-02 TASK-03 > "$R/.at" 2>&1
check "impedimento: status fora de pending" grep -qF "TASK-01: status 'approved'; só tarefas com status 'pending' são aprovadas" "$R/.at"
check "impedimento: edição manual sem registro" grep -qF "TASK-02: edição manual sem registro; registre-a com: adjust-task.sh --record-manual SPEC-008 TASK-02" "$R/.at"
check "impedimento: tarefa inválida" grep -qF "TASK-03: a subseção 'Passos' está vazia" "$R/.at"
fails "--approve com impedimentos" "$AT" --approve SPEC-008 TASK-01 TASK-02 TASK-03
check "--approve recusado não altera nenhuma tarefa" [ "$before" = "$(cat "$t1" "$t2" "$t3" | cksum)" ]
rm -rf "$TASKS/SPEC-008"; cp -R "$R/.tasks-ok" "$TASKS/SPEC-008"

# Aprovar uma, várias e todas.
check "--approve de uma tarefa" bash -c "'$AT' --approve SPEC-008 TASK-01 | grep -qx -- '- TASK-01'"
check "aprovada: status, approved_at e approved_hash" bash -c "[ '$(fm "$t1" status 2>/dev/null)' != x ] && [ \"\$(sed -n '2,/^---/p' '$t1' | grep '^status:')\" = 'status: approved' ] && sed -n '2,/^---/p' '$t1' | grep -q '^approved_at: [0-9T:-]*Z\$' && [ \"\$(sed -n '2,/^---/p' '$t1' | grep '^approved_hash:' | cut -d' ' -f2)\" = \"\$(sed -n '2,/^---/p' '$t1' | grep '^content_hash:' | cut -d' ' -f2)\" ]"
check "só a tarefa citada é aprovada" bash -c "sed -n '2,/^---/p' '$t2' | grep -qx 'status: pending'"
check "aprovada continua sem edição manual" bash -c "! '$AT' --validate SPEC-008 TASK-01 2>&1 | grep -q 'edição manual'"
before="$(sum "$t2")"
fails "recusa atômica: uma das citadas já aprovada" "$AT" --approve SPEC-008 TASK-01 TASK-02
check "recusa atômica não aprova a outra" [ "$before" = "$(sum "$t2")" ]
cp "$t2" "$t3"; sed -i.bak 's/^id: TASK-02$/id: TASK-03/' "$t3" && rm -f "$t3.bak"; store_hash "$t3"
check "--approve de duas tarefas, com ID repetido" bash -c "[ \"\$('$AT' --approve SPEC-008 TASK-02 TASK-03 TASK-02 | grep -c '^- TASK-')\" = 2 ] && sed -n '2,/^---/p' '$t3' | grep -qx 'status: approved'"
check "--all sem tarefas pending" bash -c "'$AT' --approve SPEC-008 --all 2>&1 | grep -qF \"não tem tarefas com status 'pending'\""
set_status "$t2" pending; set_status "$t3" pending
check "--all aprova só as pending e ignora as aprovadas" bash -c "out=\$('$AT' --approve SPEC-008 --all) && grep -qx -- '- TASK-02' <<< \"\$out\" && grep -qx -- '- TASK-03' <<< \"\$out\" && ! grep -q TASK-01 <<< \"\$out\""
rm -rf "$TASKS/SPEC-008"; cp -R "$R/.tasks-ok" "$TASKS/SPEC-008"

# Dependências lidas do front-matter, sem o plano.
sed -i.bak 's|^- `AGENTS.md` (alterar): cita o script\.$|- `nao/existe/novo.sh` (alterar): muda o script.|' "$t2" && rm -f "$t2.bak"; store_hash "$t2"
check "altera caminho criado pela dependência (depends_on)" "$AT" --validate SPEC-008 TASK-02
sed -i.bak 's/^depends_on: \[TASK-01\]$/depends_on: []/' "$t2" && rm -f "$t2.bak"
check "sem a dependência, o caminho não existe" bash -c "'$AT' --validate SPEC-008 TASK-02 2>&1 | grep -qF \"TASK-02: 'nao/existe/novo.sh' não existe nem é criado por uma tarefa de que ela depende\""
rm -rf "$TASKS/SPEC-008"; mv "$R/.tasks-ok" "$TASKS/SPEC-008"

# Ajuste de tarefas.
AT2="$H/skills/default/adjust-task/scripts/adjust-task.sh"
cp -R "$TASKS/SPEC-008" "$R/.tasks-ok"
hist() { sed -n '/^## Histórico de ajustes$/,$p' "$1" | grep -c '^| [0-9]'; }
fails "adjust-task sem argumentos" "$AT2"
fails "adjust-task --record sem resumo" "$AT2" --record SPEC-008 TASK-01
fails "adjust-task --record com resumo vazio" "$AT2" --record SPEC-008 TASK-01 "   "
fails "adjust-task com ID inválido" "$AT2" --validate SPEC-008 ../TASK-01
check "adjust-task de tarefa inexistente" bash -c "'$AT2' --validate SPEC-008 TASK-09 2>&1 | grep -qF 'tarefa TASK-09 não encontrada em SPEC-008'"
check "adjust-task --validate de tarefa válida" "$AT2" --validate SPEC-008 TASK-01
check "--record-manual sem edição" bash -c "'$AT2' --record-manual SPEC-008 TASK-01 | grep -qx 'Nenhuma edição manual.'"

# Edição manual.
sed -i.bak 's/^1\. Criar o script\.$/1. Criar o script à mão./' "$t1" && rm -f "$t1.bak"
check "edição manual impede a aprovação" bash -c "'$AT' --validate SPEC-008 TASK-01 2>&1 | grep -qF 'edição manual sem registro'"
check "--record-manual registra a edição" bash -c "'$AT2' --record-manual SPEC-008 TASK-01 | grep -qF 'Ajuste manual 1 registrado em TASK-01: Edição manual'"
check "edição manual: linha manual no histórico e adjustments 1" bash -c "grep -qE '^\| 1 \| [0-9-]{10} \| manual \| Edição manual' '$t1' && sed -n '2,/^---/p' '$t1' | grep -qx 'adjustments: 1'"
check "edição manual registrada libera a aprovação" "$AT" --validate SPEC-008 TASK-01
python3 - "$t1" <<'PY'
import sys, re; p = sys.argv[1]; s = open(p, encoding='utf-8').read()
s = re.sub(r'(### Passos\n\n)[\s\S]*?(\n### )', r'\1\2', s, count=1); open(p, 'w', encoding='utf-8').write(s)
PY
check "edição manual inválida: registra e aponta os problemas" bash -c "out=\$('$AT2' --record-manual SPEC-008 TASK-01) && grep -qF 'Ajuste manual 2 registrado' <<< \"\$out\" && grep -qF \"TASK-01: a subseção 'Passos' está vazia\" <<< \"\$out\""
check "--validate lista os problemas" bash -c "'$AT2' --validate SPEC-008 TASK-01 2>&1 | grep -qF \"TASK-01: a subseção 'Passos' está vazia\""
before="$(sum "$t1")"
fails "--record com tarefa inválida" "$AT2" --record SPEC-008 TASK-01 "Passos removidos"
check "--record recusado não altera a tarefa" [ "$before" = "$(sum "$t1")" ]

# Ajuste do agente.
python3 - "$t1" <<'PY'
import sys; p = sys.argv[1]; s = open(p, encoding='utf-8').read()
s = s.replace('### Passos\n\n', '### Passos\n\n1. Criar o script com testes.\n', 1); open(p, 'w', encoding='utf-8').write(s)
PY
check "--record registra o ajuste do agente" bash -c "'$AT2' --record SPEC-008 TASK-01 'Passo com testes; resumo com | barra' | grep -qF 'Ajuste 3 registrado em TASK-01'"
check "ajuste do agente: linha agente com barra escapada" grep -qF '| agente | Passo com testes; resumo com \| barra |' "$t1"
check "ajuste do agente: hash atualizado" bash -c "'$AT2' --record-manual SPEC-008 TASK-01 | grep -qx 'Nenhuma edição manual.'"
check "ajuste do agente: uma linha por registro" [ "$(hist "$t1")" = 3 ]

# Ajuste de tarefa aprovada.
"$AT" --approve SPEC-008 TASK-01 > /dev/null
sed -i.bak 's/^1\. Criar o script com testes\.$/1. Criar o script com testes e docs./' "$t1" && rm -f "$t1.bak"
check "--record em tarefa aprovada avisa que a aprovação caiu" bash -c "'$AT2' --record SPEC-008 TASK-01 'Docs no passo 1' | grep -qF \"Aprovação de TASK-01 derrubada: o status voltou para 'pending'\""
check "aprovação derrubada: status pending e campos limpos" bash -c "fmx=\$(sed -n '2,/^---/p' '$t1'); grep -qx 'status: pending' <<< \"\$fmx\" && grep -qx 'approved_at: null' <<< \"\$fmx\" && grep -qx 'approved_hash: null' <<< \"\$fmx\""
check "tarefa ajustada pode ser aprovada de novo" "$AT" --approve SPEC-008 TASK-01
sed -i.bak 's/^1\. Criar o script com testes e docs\.$/1. Criar o script./' "$t1" && rm -f "$t1.bak"
check "edição manual em tarefa aprovada também derruba a aprovação" bash -c "'$AT2' --record-manual SPEC-008 TASK-01 | grep -qF 'Aprovação de TASK-01 derrubada' && sed -n '2,/^---/p' '$t1' | grep -qx 'status: pending'"

# Status fora de pending e approved.
for st in in_progress validating blocked done; do
  set_status "$t2" "$st"; before="$(sum "$t2")"
  check "adjust-task recusa tarefa $st e informa o status" bash -c "'$AT2' --record SPEC-008 TASK-02 'x' 2>&1 | grep -qF \"a tarefa TASK-02 está com status '$st'\" && [ '$before' = \"\$(cksum < '$t2')\" ]"
done
fails "--record-manual recusa tarefa fora de pending e approved" "$AT2" --record-manual SPEC-008 TASK-02
rm -rf "$TASKS/SPEC-008"; mv "$R/.tasks-ok" "$TASKS/SPEC-008"

# Execução de tarefas.
ET="$H/skills/default/execute-task/scripts/execute-task.sh"
cp -R "$TASKS/SPEC-008" "$R/.tasks-ok"
EV="$TASKS/SPEC-008/evidence"
rec() { sed -n '/^## Registro de execução$/,/^## /p' "$1" | grep '^| [0-9]' | cut -d'|' -f2,4,5 | tr -d ' ' | paste -sd ';' -; }
hash_ok() { HARNESS_DIR="$H" bash -c 'for l in config text frontmatter markdown spec-validation adjustments; do source "$HARNESS_DIR/scripts/lib/$l.sh"; done; [ "$(adjustment_hash_state "$1")" = same ]' _ "$1"; }
python3 - "$t1" <<'PY'
import sys; p = sys.argv[1]; s = open(p, encoding='utf-8').read()
s = s.replace('### Baseline\n\n- `true`\n', '### Baseline\n\n- `ls AGENTS.md`\n- `echo "a|b"; echo erro >&2`\n- `false`\n', 1)
open(p, 'w', encoding='utf-8').write(s)
PY
store_hash "$t1"
fails "execute-task sem argumentos" "$ET"
fails "execute-task com ID inválido" "$ET" --start SPEC-008 ../TASK-01
check "execute-task de tarefa inexistente" bash -c "'$ET' --start SPEC-008 TASK-09 2>&1 | grep -qF 'tarefa TASK-09 não encontrada em SPEC-008'"

# Recusas do início.
before="$(sum "$t1")"
check "--start recusa tarefa pending" bash -c "'$ET' --start SPEC-008 TASK-01 2>&1 | grep -qF \"a tarefa TASK-01 está com status 'pending'; só tarefas com status 'approved' são executadas\""
check "--start recusado não altera a tarefa" [ "$before" = "$(sum "$t1")" ]
"$AT" --approve SPEC-008 TASK-01 TASK-02 > /dev/null
sed -i.bak 's/^1\. Criar o script\.$/1. Criar o script à mão./' "$t1" && rm -f "$t1.bak"; before="$(sum "$t1")"
check "--start recusa tarefa editada depois da aprovação" bash -c "'$ET' --start SPEC-008 TASK-01 2>&1 | grep -qF 'a tarefa TASK-01 mudou depois da aprovação'"
check "--start recusado por edição não altera a tarefa" [ "$before" = "$(sum "$t1")" ]
store_hash "$t1"
check "--start recusa edição mesmo com o hash regravado" bash -c "'$ET' --start SPEC-008 TASK-01 2>&1 | grep -qF 'mudou depois da aprovação'"
sed -i.bak 's/^1\. Criar o script à mão\.$/1. Criar o script./' "$t1" && rm -f "$t1.bak"; store_hash "$t1"
before="$(sum "$t2")"
check "--start recusa dependência aprovada e cita a dependência" bash -c "'$ET' --start SPEC-008 TASK-02 2>&1 | grep -qF \"a dependência TASK-01 está com status 'approved'; ela precisa estar 'done' antes\""
set_status "$t1" blocked
check "--start recusa dependência blocked" bash -c "'$ET' --start SPEC-008 TASK-02 2>&1 | grep -qF \"a dependência TASK-01 está com status 'blocked'\""
check "--start recusado por dependência não altera a tarefa" [ "$before" = "$(sum "$t2")" ]
set_status "$t1" approved
fails "--baseline antes do início" "$ET" --baseline SPEC-008 TASK-01
check "--baseline antes do início não cria evidências" [ ! -e "$EV" ]

# Início e baseline.
check "--start inicia a tarefa" bash -c "'$ET' --start SPEC-008 TASK-01 | grep -qF \"Tarefa TASK-01 iniciada: status 'in_progress'\""
check "iniciada: status in_progress" bash -c "sed -n '2,/^---/p' '$t1' | grep -qx 'status: in_progress'"
check "iniciada: linha no registro" [ "$(rec "$t1")" = '1|início|in_progress' ]
check "iniciada: hash do conteúdo atualizado" hash_ok "$t1"
fails "--start de tarefa já iniciada" "$ET" --start SPEC-008 TASK-01
check "--baseline coleta um arquivo por comando" bash -c "'$ET' --baseline SPEC-008 TASK-01 | grep -qF '3 comando(s); códigos de saída: 0, 0, 1' && [ \"\$(ls '$EV/TASK-01/baseline' | paste -sd ' ' -)\" = '01.out 02.out 03.out' ]"
check "baseline: comando, código e saída, a partir da raiz" bash -c "[ \"\$(cat '$EV/TASK-01/baseline/01.out')\" = \"\$(printf '\$ ls AGENTS.md\nexit: 0\nAGENTS.md')\" ]"
check "baseline: saída de erro e barra vertical gravadas" bash -c "grep -qx 'a|b' '$EV/TASK-01/baseline/02.out' && grep -qx 'erro' '$EV/TASK-01/baseline/02.out' && grep -qx 'exit: 1' '$EV/TASK-01/baseline/03.out'"
check "baseline: pasta com as permissões padrão" bash -c "ls -ld '$EV/TASK-01/baseline' | grep -q '^drwxr-xr-x'"
check "baseline: linha no registro" [ "$(rec "$t1")" = '1|início|in_progress;2|baseline|coletado' ]
check "baseline: detalhe no registro" grep -qF '3 comando(s); códigos de saída: 0, 0, 1 |' "$t1"
check "baseline: hash do conteúdo atualizado" hash_ok "$t1"
before="$(cat "$EV"/TASK-01/baseline/* | cksum)"
check "--baseline repetido é recusado" bash -c "'$ET' --baseline SPEC-008 TASK-01 2>&1 | grep -qF 'o baseline de TASK-01 já foi coletado'"
check "--baseline repetido não sobrescreve" [ "$before" = "$(cat "$EV"/TASK-01/baseline/* | cksum)" ]
check "--baseline repetido não registra" [ "$(rec "$t1")" = '1|início|in_progress;2|baseline|coletado' ]
set_status "$t1" done
check "--start aceita dependência done" "$ET" --start SPEC-008 TASK-02
rm -rf "$TASKS/SPEC-008"; mv "$R/.tasks-ok" "$TASKS/SPEC-008"

# Validação de tarefas: testes, regressão e retomada.
VT="$H/skills/default/validate-task/scripts/validate-task.sh"
cp -R "$TASKS/SPEC-008" "$R/.tasks-ok"
fmv() { sed -n '2,/^---/p' "$1" | grep "^$2:" | sed "s/^$2: //"; }
python3 - "$t1" <<'PY'
import sys; p = sys.argv[1]; s = open(p, encoding='utf-8').read()
s = s.replace('```bash\ntrue\n```', '```bash\ntest ! -e .falha   # falha enquanto .falha existir\ntrue\n```', 1)
s = s.replace('### Baseline\n\n- `true`\n', '### Baseline\n\n- `cat estado.txt`\n', 1)
open(p, 'w', encoding='utf-8').write(s)
PY
store_hash "$t1"; "$AT" --approve SPEC-008 TASK-01 > /dev/null
echo um > "$R/estado.txt"; rm -f "$R/.falha"
fails "validate-task sem argumentos" "$VT"
fails "validate-task com ID inválido" "$VT" --tests SPEC-008 ../TASK-01
check "--tests recusa tarefa aprovada, ainda não iniciada" bash -c "'$VT' --tests SPEC-008 TASK-01 2>&1 | grep -qF \"a tarefa TASK-01 está com status 'approved'; só tarefas com status 'in_progress' ou 'validating' são validadas\""
"$ET" --start SPEC-008 TASK-01 > /dev/null
mv "$TASKS/SPEC-008" "$R/.sem-baseline"; cp -R "$R/.sem-baseline" "$TASKS/SPEC-008"
check "--regression recusa sem baseline" bash -c "'$VT' --regression SPEC-008 TASK-01 2>&1 | grep -qF 'o baseline de TASK-01 não foi coletado'"
rm -rf "$TASKS/SPEC-008"; mv "$R/.sem-baseline" "$TASKS/SPEC-008"
"$ET" --baseline SPEC-008 TASK-01 > /dev/null

# Testes que passam e regressão sem diferença.
check "--tests que passam" bash -c "'$VT' --tests SPEC-008 TASK-01 | grep -qF 'Testes de TASK-01 passaram'"
check "testes: validating, passed e saída gravada" bash -c "[ \"\$(sed -n '2,/^---/p' '$t1' | grep -E '^(status|tests_result):' | paste -sd ' ' -)\" = 'status: validating tests_result: passed' ] && [ -f '$EV/TASK-01/validation/tests.out' ]"
check "--regression sem diferença" bash -c "'$VT' --regression SPEC-008 TASK-01 | grep -qF 'Sem regressão em TASK-01: 1 comando(s)'"
check "regressão: passed" [ "$(fmv "$t1" regression_result)" = passed ]

# Testes que falham até o máximo; o primeiro comando com falha basta, mesmo com o último passando.
touch "$R/.falha"
"$VT" --tests SPEC-008 TASK-01 > "$R/.vt" 2>&1; code=$?
check "--tests com falha termina com erro" [ "$code" -ne 0 ]
check "--tests com falha mostra a tentativa e onde está a saída" bash -c "grep -qF '(código de saída 1), tentativa 1 de 2. Últimas linhas da saída' '$R/.vt' && grep -qF 'os testes de TASK-01 falharam; corrija' '$R/.vt'"
check "--tests com falha aponta o comando que falhou" grep -qF 'comando com falha (código 1): test ! -e .falha' "$R/.vt"
check "falha: attempts 1, failed, regressão e evidências null, validating" bash -c "[ \"\$(sed -n '2,/^---/p' '$t1' | grep -E '^(status|attempts|tests_result|regression_result|evidence_result):' | paste -sd ' ' -)\" = 'status: validating attempts: 1 tests_result: failed regression_result: null evidence_result: null' ]"
check "segunda falha bloqueia e manda parar" bash -c "'$VT' --tests SPEC-008 TASK-01 2>&1 | grep -qF 'a tarefa TASK-01 ficou bloqueada depois de 2 tentativas. PARE'"
check "bloqueada: blocked e attempts 2" bash -c "sed -n '2,/^---/p' '$t1' | grep -qx 'status: blocked' && sed -n '2,/^---/p' '$t1' | grep -qx 'attempts: 2'"
before="$(sum "$t1")"
check "terceira execução de --tests recusada" bash -c "'$VT' --tests SPEC-008 TASK-01 2>&1 | grep -qF 'a tarefa TASK-01 está bloqueada'"
fails "--regression recusada em tarefa bloqueada" "$VT" --regression SPEC-008 TASK-01
check "recusas em tarefa bloqueada não alteram a tarefa" [ "$before" = "$(sum "$t1")" ]

# Retomada.
fails "--resume sem decisão" "$VT" --resume SPEC-008 TASK-01 "  "
fails "--resume com quebra de linha" "$VT" --resume SPEC-008 TASK-01 $'a\nb'
check "--resume retoma a tarefa" bash -c "'$VT' --resume SPEC-008 TASK-01 'remover o arquivo | de falha' | grep -qF \"Tarefa TASK-01 retomada: status 'validating'\""
check "retomada: validating, attempts 0 e resultados null" bash -c "[ \"\$(sed -n '2,/^---/p' '$t1' | grep -E '^(status|attempts|tests_result|regression_result|evidence_result):' | paste -sd ' ' -)\" = 'status: validating attempts: 0 tests_result: null regression_result: null evidence_result: null' ]"
check "retomada: decisão no registro" grep -qF '| retomada | validating | decisão do usuário: remover o arquivo \| de falha |' "$t1"
fails "--resume em tarefa não bloqueada" "$VT" --resume SPEC-008 TASK-01 "outra"
rm -f "$R/.falha"
check "depois da retomada, --tests passa" "$VT" --tests SPEC-008 TASK-01

# Regressão por saída e por código de saída.
echo dois > "$R/estado.txt"
"$VT" --regression SPEC-008 TASK-01 > "$R/.vt" 2>&1; code=$?
check "regressão por saída: erro, comando listado e diferença" bash -c "[ $code -ne 0 ] && grep -qx -- '- cat estado.txt' '$R/.vt' && grep -q '^    > dois' '$R/.vt' && grep -qF 'bloqueada por regressão. PARE' '$R/.vt'"
check "regressão: failed e blocked" bash -c "sed -n '2,/^---/p' '$t1' | grep -qx 'regression_result: failed' && sed -n '2,/^---/p' '$t1' | grep -qx 'status: blocked'"
"$VT" --resume SPEC-008 TASK-01 "corrigir o estado" > /dev/null
echo um > "$R/estado.txt"
check "regressão corrigida passa" bash -c "'$VT' --tests SPEC-008 TASK-01 >/dev/null && '$VT' --regression SPEC-008 TASK-01 >/dev/null"
rm -f "$R/estado.txt"
check "regressão por código de saída" bash -c "'$VT' --regression SPEC-008 TASK-01 2>&1 | grep -q '^    < exit: 0'"
check "registro: uma linha por etapa" bash -c "[ \"\$(sed -n '/^## Registro de execução\$/,/^## /p' '$t1' | grep -c '^| [0-9]')\" = 13 ]"
check "registro: hash do conteúdo atualizado" hash_ok "$t1"
rm -rf "$TASKS/SPEC-008"; mv "$R/.tasks-ok" "$TASKS/SPEC-008"; rm -f "$R/estado.txt" "$R/.falha"

# Validação de tarefas: evidências e conclusão.
cp -R "$TASKS/SPEC-008" "$R/.tasks-ok"; cp "$r8" "$R/.r8-ok"
python3 - "$t1" <<'PY'
import sys; p = sys.argv[1]; s = open(p, encoding='utf-8').read()
s = s.replace('- `saida.txt` (automática): `echo ok`', '- `saida.txt` (automática): `echo ok; ls AGENTS.md`\n- `nota.txt` (manual): anotação do usuário', 1)
open(p, 'w', encoding='utf-8').write(s)
PY
store_hash "$t1"; "$AT" --approve SPEC-008 TASK-01 > /dev/null
"$ET" --start SPEC-008 TASK-01 > /dev/null; "$ET" --baseline SPEC-008 TASK-01 > /dev/null
"$VT" --tests SPEC-008 TASK-01 > /dev/null; "$VT" --regression SPEC-008 TASK-01 > /dev/null
check "--finish recusa evidências não feitas" bash -c "'$VT' --finish SPEC-008 TASK-01 2>&1 | grep -qF \"evidence_result: 'null' (precisa ser 'passed')\""
"$VT" --evidence SPEC-008 TASK-01 > "$R/.vt" 2>&1; code=$?
check "evidência gravada com as permissões padrão" bash -c "ls -l '$EV/TASK-01/saida.txt' | grep -q '^-rw-r--r--'"
check "--evidence grava a automática, a partir da raiz" bash -c "[ $code -eq 0 ] && [ \"\$(cat '$EV/TASK-01/saida.txt')\" = \"\$(printf 'ok\nAGENTS.md')\" ]"
check "--evidence aponta a manual sem arquivo" grep -qF "/work/tasks/SPEC-008/evidence/TASK-01/nota.txt: anotação do usuário" "$R/.vt"
check "evidência manual pendente: pending e registro" bash -c "sed -n '2,/^---/p' '$t1' | grep -qx 'evidence_result: pending' && grep -qF '| evidências | pending | evidência manual sem arquivo |' '$t1'"
check "--finish recusa evidência pending" bash -c "'$VT' --finish SPEC-008 TASK-01 2>&1 | grep -qF \"evidence_result: 'pending'\""
: > "$EV/TASK-01/nota.txt"
check "arquivo manual vazio continua pendente" bash -c "'$VT' --evidence SPEC-008 TASK-01 | grep -qF 'nota.txt: anotação do usuário'"
echo 'anexo' > "$EV/TASK-01/nota.txt"
check "--evidence com a manual anexada passa" bash -c "'$VT' --evidence SPEC-008 TASK-01 | grep -qF 'Resultado: passed' && sed -n '2,/^---/p' '$t1' | grep -qx 'evidence_result: passed'"
check "--evidence não sobrescreve o anexo manual" grep -qx 'anexo' "$EV/TASK-01/nota.txt"
before="$(sum "$t1")"
check "--finish recusa critério de conclusão sem marcar" bash -c "'$VT' --finish SPEC-008 TASK-01 2>&1 | grep -qF 'critérios de conclusão sem marcar' && '$VT' --finish SPEC-008 TASK-01 2>&1 | grep -qF -- '- [ ] O script existe (RF-01).'"
check "--finish recusado não altera a tarefa" [ "$before" = "$(sum "$t1")" ]
sed -i.bak 's/^- \[ \] O script existe/- [x] O script existe/' "$t1" && rm -f "$t1.bak"
check "--finish conclui a tarefa" bash -c "'$VT' --finish SPEC-008 TASK-01 | grep -qF \"Tarefa TASK-01 concluída: status 'done'.\""
check "concluída: done, registro e hash" bash -c "sed -n '2,/^---/p' '$t1' | grep -qx 'status: done' && grep -qF '| conclusão | done | testes, regressão e evidências passed' '$t1'" 
check "concluída: hash do conteúdo atualizado" hash_ok "$t1"
check "com outra tarefa pendente, a spec não muda" bash -c "sed -n '2,/^---/p' '$r8' | grep -qx 'status: planned'"
fails "--finish de tarefa já concluída" "$VT" --finish SPEC-008 TASK-01
fails "--evidence de tarefa já concluída" "$VT" --evidence SPEC-008 TASK-01

# Segunda tarefa: evidência automática com erro, nomes inválidos, etapas puladas e "Não se aplica.".
store_hash "$t2"; "$AT" --approve SPEC-008 TASK-02 > /dev/null
"$ET" --start SPEC-008 TASK-02 > /dev/null; "$ET" --baseline SPEC-008 TASK-02 > /dev/null; "$VT" --tests SPEC-008 TASK-02 > /dev/null
check "--finish recusa regressão não feita" bash -c "'$VT' --finish SPEC-008 TASK-02 2>&1 | grep -qF \"regression_result: 'null' (precisa ser 'passed')\""
"$VT" --regression SPEC-008 TASK-02 > /dev/null
sed -i.bak 's/^- `saida.txt` (automática): `echo ok`$/- `erro.txt` (automática): `false`/' "$t2" && rm -f "$t2.bak"
check "--evidence com automática que falha" bash -c "'$VT' --evidence SPEC-008 TASK-02 2>&1 | grep -qF 'erro.txt: \`false\` terminou com erro'"
check "evidência com erro não grava resultado" bash -c "sed -n '2,/^---/p' '$t2' | grep -qx 'evidence_result: null'"
check "evidência com erro não deixa arquivo" [ ! -e "$EV/TASK-02/erro.txt" ]
for bad in '../fora.txt' 'baseline' 'sub/x.txt' '.oculto'; do
  sed -i.bak "s|^- \`[^\`]*\` (automática): .*|- \`$bad\` (automática): \`echo x\`|" "$t2" && rm -f "$t2.bak"
  check "--evidence recusa o nome '$bad'" bash -c "'$VT' --evidence SPEC-008 TASK-02 2>&1 | grep -qF \"nome de evidência inválido: '$bad'\""
done
check "nome inválido não grava fora da pasta" [ ! -e "$TASKS/SPEC-008/evidence/fora.txt" ]
sed -i.bak "s|^- \`[^\`]*\` (automática): .*|Não se aplica.|" "$t2" && rm -f "$t2.bak"
check "--evidence com 'Não se aplica.' passa sem arquivos" bash -c "'$VT' --evidence SPEC-008 TASK-02 | grep -qF 'Não se aplica. Resultado: passed' && [ \"\$(ls '$EV/TASK-02' | paste -sd ' ' -)\" = 'baseline validation' ]"
"$VT" --tests SPEC-008 TASK-02 > /dev/null
check "--tests que passam de novo mantêm as evidências" bash -c "sed -n '2,/^---/p' '$t2' | grep -qx 'evidence_result: passed'"
sed -i.bak 's/^- \[ \] O script existe/- [x] O script existe/' "$t2" && rm -f "$t2.bak"
check "última tarefa concluída leva a spec a implemented" bash -c "'$VT' --finish SPEC-008 TASK-02 | grep -qF \"Todas as tarefas de SPEC-008 concluídas: spec marcada como 'implemented'.\""
check "spec implemented com updated_at" bash -c "sed -n '2,/^---/p' '$r8' | grep -qx 'status: implemented' && sed -n '2,/^---/p' '$r8' | grep -q '^updated_at: [0-9T:-]*Z\$'"
check "implemented: adjust-spec recusa" bash -c "'$A' --validate SPEC-008 2>&1 | grep -qF \"a spec SPEC-008 está com status 'implemented'\""
check "implemented: plan-tasks recusa" bash -c "! '$PT' --create SPEC-008 >/dev/null 2>&1"
rm -rf "$TASKS/SPEC-008"; mv "$R/.tasks-ok" "$TASKS/SPEC-008"; mv "$R/.r8-ok" "$r8"

# Cobertura de spec do tipo study: PQ.
"$SP/create-new-spec.sh" 'Estudo' > /dev/null
"$SP/create-refined-spec.sh" SPEC-009 study > /dev/null
r9="$(ls "$H"/work/specs/refined/SPEC-009-*.md)"
$FILL "$r9" && "$SP/create-refined-spec.sh" --mark-refined SPEC-009 > /dev/null && "$AP" --approve SPEC-009 > /dev/null
"$PT" --create SPEC-009 > /dev/null
$FILL "$PLANS/SPEC-009.md" "PLAN_TASKS=$(rows 'TASK-01|a|a|tarefa|-|-|PQ-01')" "PLAN_FACTS=$(rows 'TASK-01|0|1|não|não|não')"
"$PT" --score SPEC-009 > /dev/null
check "study: plano cobrindo PQ é válido" "$PT" --validate SPEC-009
sed -i.bak 's/| PQ-01 |$/| RF-01 |/' "$PLANS/SPEC-009.md" && rm -f "$PLANS/SPEC-009.md.bak"
check "study: RF não existe e PQ fica sem tarefa" bash -c "out=\$('$PT' --validate SPEC-009 2>&1); grep -q 'PQ-01 não é coberto' <<< \"\$out\" && grep -q 'cobre RF-01, que não existe' <<< \"\$out\""
sed -i.bak 's/| RF-01 |$/| PQ-01 |/' "$PLANS/SPEC-009.md" && rm -f "$PLANS/SPEC-009.md.bak"
$FILL "$PLANS/SPEC-009.md" > /dev/null
python3 - "$PLANS/SPEC-009.md" <<'PY'
import sys; p = sys.argv[1]; s = open(p, encoding='utf-8').read()
s = s.replace('| TASK-01 | a | a | tarefa | simples | - | PQ-01 |', '| TASK-01 | a | a | tarefa | simples | - | PQ-01 |\n| TASK-02 | b | b | tarefa | simples | TASK-01 | - |\n| TASK-03 | c | c | tarefa | simples | TASK-02 | - |')
s = s.replace('| TASK-01 | 0 | 1 | não | não | não |', '| TASK-01 | 0 | 1 | não | não | não |\n| TASK-02 | 0 | 1 | não | não | não |\n| TASK-03 | 0 | 1 | não | não | não |')
open(p, 'w', encoding='utf-8').write(s)
PY
"$AJP" --record-manual SPEC-009 > /dev/null; "$APP" --create-tasks SPEC-009 > /dev/null
$FILL "$TASKS/SPEC-009/TASK-01.md"
$FILL "$TASKS/SPEC-009/TASK-02.md" 'TASK_CHANGE=- `AGENTS.md` (alterar): cita o script.'
$FILL "$TASKS/SPEC-009/TASK-03.md" 'TASK_CHANGE=- `nao/existe/novo.sh` (remover): substituído.'
check "tarefa remove caminho criado por dependência indireta" "$APP" --validate SPEC-009
check "o plano de uma spec não interfere no de outra" bash -c "[ ! -e '$p8' ] && [ -d '$TASKS/SPEC-008' ] && [ '$(fm "$r8" status)' = planned ]"

# --- Finalização ---
FS="$H/skills/default/finalize-spec/scripts/finalize-spec.sh"
fails "finalize sem argumentos" "$FS"
fails "finalize com opção inválida" "$FS" --publicar SPEC-008
check "finalize com spec inexistente" bash -c "'$FS' --sources SPEC-999 2>&1 | grep -qF 'spec SPEC-999 não encontrada'"
for opt in --sources --create --validate --build --archive; do
  check "finalize $opt recusa spec planned" bash -c "! '$FS' $opt SPEC-008 >/dev/null 2>&1 && '$FS' $opt SPEC-008 2>&1 | grep -qF \"a spec está com status 'planned'; só specs com status 'implemented' são finalizadas\""
done
check "recusa lista as tarefas fora de done" bash -c "'$FS' --sources SPEC-008 2>&1 | grep -qF \"TASK-01: status 'pending'\""
cp "$r8" "$R/.fin-r8"; cp -R "$TASKS/SPEC-008" "$R/.fin-tasks"
sed -i.bak 's/^status: .*/status: implemented/' "$r8" && rm -f "$r8.bak"
for t in "$TASKS"/SPEC-008/TASK-*.md; do sed -i.bak 's/^status: .*/status: done/' "$t" && rm -f "$t.bak"; done
sed -i.bak 's/^status: .*/status: validating/' "$TASKS/SPEC-008/TASK-02.md" && rm -f "$TASKS/SPEC-008/TASK-02.md.bak"
check "spec implemented com tarefa fora de done é recusada" bash -c "out=\$('$FS' --archive SPEC-008 2>&1); grep -qF \"TASK-02: status 'validating', não 'done'\" <<< \"\$out\" && ! grep -q 'status da spec' <<< \"\$out\" && ! grep -q TASK-01 <<< \"\$out\""
sed -i.bak 's/^status: .*/status: done/' "$TASKS/SPEC-008/TASK-02.md" && rm -f "$TASKS/SPEC-008/TASK-02.md.bak"
check "spec implemented com tarefas done passa da conferência" bash -c "! '$FS' --sources SPEC-008 2>&1 | grep -q 'não pode ser finalizada'"
check "a conferência não altera a spec nem as tarefas" bash -c "grep -qx 'status: implemented' '$r8' && grep -qx 'status: done' '$TASKS/SPEC-008/TASK-02.md'"
rm -rf "$TASKS/SPEC-008"; mv "$R/.fin-tasks" "$TASKS/SPEC-008"; mv "$R/.fin-r8" "$r8"

# Fontes da síntese (--sources).
cp "$r8" "$R/.fin-r8"; cp -R "$TASKS/SPEC-008" "$R/.fin-tasks"; cp -R "$H/memory" "$R/.fin-mem"
sed -i.bak 's/^status: .*/status: implemented/; s/^approved_at: .*/approved_at: 2026-01-10T00:00:00Z/' "$r8" && rm -f "$r8.bak"
for t in "$TASKS"/SPEC-008/TASK-*.md; do sed -i.bak 's/^status: .*/status: done/' "$t" && rm -f "$t.bak"; done
gf() { git -C "$R" -c user.name=Teste -c user.email=t@t -c commit.gpgsign=false -c core.hooksPath=/dev/null "$@"; }
commit_at() { # data, arquivo, mensagem
  printf '%s\n' "$3" > "$R/$2" && gf add -- "$2" && GIT_AUTHOR_DATE="$1" GIT_COMMITTER_DATE="$1" gf commit -qm "$3" -- "$2"
}
for f in "$H"/memory/*.md; do awk '/^## [A-Z]+-[0-9]+: /{exit} {print}' "$f" > "$f.tmp" && mv "$f.tmp" "$f"; done
entry() { printf '\n## %s: %s\n\n- **Data:** %s\n- **Status:** %s\n' "$2" "$3" "$4" "$5" >> "$H/memory/$1"; }
entry decisions.md DEC-101 'Decisão antes da spec' 2026-01-05 ativa
entry decisions.md DEC-102 'Decisão durante a spec' 2026-01-12 ativa
entry decisions.md DEC-103 'Decisão depois de hoje' 2999-01-01 ativa
entry issues.md ISS-101 'Issue durante a spec' 2026-01-20 resolvido
entry knowledge.md KNW-101 'Conhecimento durante a spec' 2026-02-01 válido
rm -rf "$R/.git"; gf init -q -b main
mkdir -p "$R/nao/existe"
commit_at 2026-01-05T12:00:00Z nao/existe/novo.sh 'antes da aprovação'
commit_at 2026-01-15T12:00:00Z nao/existe/novo.sh 'depois da aprovação, caminho da tarefa'
commit_at 2026-01-16T12:00:00Z fin-outro.txt 'depois da aprovação, outro caminho'
sed -i.bak 's|^- `AGENTS.md` (alterar): .*|&\n- `nao/existe/novo.sh` (criar): repetido em outra tarefa.|' "$TASKS/SPEC-008/TASK-02.md" && rm -f "$TASKS/SPEC-008/TASK-02.md.bak"
before="$(cat "$r8" "$TASKS"/SPEC-008/TASK-*.md | cksum)"
out="$("$FS" --sources SPEC-008 2>&1)"
for h in '## Spec' '## Tarefas' '## Caminhos alterados' '## Memória do período' '## Commits'; do
  check "fontes têm o bloco '$h'" grep -qx "$h" <<< "$out"
done
check "fontes: dados da spec" bash -c "grep -qF -- '- **ID:** SPEC-008' <<< '$out' && grep -qF -- '- **Aprovada em:** 2026-01-10T00:00:00Z' <<< '$out'"
check "fontes: cada tarefa com status, registro e histórico" bash -c "[ \$(grep -c '^### TASK-0' <<< '$out') -eq 2 ] && [ \$(grep -c -- '- \*\*Status:\*\* done' <<< '$out') -eq 2 ] && [ \$(grep -c '^\*\*Registro de execução\*\*' <<< '$out') -eq 2 ] && [ \$(grep -c '^\*\*Histórico de ajustes\*\*' <<< '$out') -eq 2 ]"
check "fontes: evidência ausente" bash -c "grep -qF 'evidence/TASK-01/saida.txt\` (automática): ausente' <<< '$out'"
mkdir -p "$TASKS/SPEC-008/evidence/TASK-01" && echo ok > "$TASKS/SPEC-008/evidence/TASK-01/saida.txt"
check "fontes: evidência existente" bash -c "'$FS' --sources SPEC-008 | grep -qF 'evidence/TASK-01/saida.txt\` (automática): existe'"
check "fontes: evidências com caminho relativo à raiz do projeto" bash -c "grep -qF -- '- \`packages/harness/work/tasks/SPEC-008/evidence/TASK-01/saida.txt\`' <<< \"\$('$FS' --sources SPEC-008)\""
check "fontes: caminho repetido em duas tarefas aparece uma vez" bash -c "[ \$(grep -cF -- '- \`nao/existe/novo.sh\` (criar)' <<< '$out') -eq 1 ]"
check "fontes: caminhos alterados com a ação" bash -c "grep -qF -- '- \`nao/existe/novo.sh\` (criar)' <<< '$out' && grep -qF -- '- \`AGENTS.md\` (alterar)' <<< '$out'"
check "fontes: memória só do período" bash -c "grep -q 'DEC-102' <<< '$out' && grep -q 'ISS-101' <<< '$out' && grep -q 'KNW-101' <<< '$out' && ! grep -q 'DEC-101' <<< '$out' && ! grep -q 'DEC-103' <<< '$out'"
check "fontes: memória com status e data" bash -c "grep -qF -- '- \`DEC-102\` Decisão durante a spec (ativa, 2026-01-12)' <<< '$out'"
check "fontes: commits só do período e dos caminhos das tarefas" bash -c "grep -qF 'depois da aprovação, caminho da tarefa' <<< '$out' && ! grep -qF 'antes da aprovação' <<< '$out' && ! grep -qF 'outro caminho' <<< '$out'"
check "fontes: commit com hash e data" bash -c "grep -Eq -- '^- \`[0-9a-f]+\` 2026-01-15 depois da aprovação, caminho da tarefa\$' <<< '$out'"
check "fontes não alteram a spec nem as tarefas" bash -c "[ '$before' = \"\$(cat '$r8' '$TASKS'/SPEC-008/TASK-*.md | cksum)\" ]"
sed -i.bak 's/^approved_at: .*/approved_at: 2035-01-01T00:00:00Z/' "$r8" && rm -f "$r8.bak"
out="$("$FS" --sources SPEC-008 2>&1)"
check "fontes sem commits no período" bash -c "grep -qF 'Nenhum commit encontrado.' <<< '$out'"
check "fontes sem memória no período" bash -c "grep -qF 'Nenhuma entrada de memória no período.' <<< '$out'"
sed -i.bak 's/^approved_at: .*/approved_at: null/' "$r8" && rm -f "$r8.bak"
check "fontes de spec sem data de aprovação" bash -c "out=\$('$FS' --sources SPEC-008 2>&1); [ \$(grep -c 'Spec sem data de aprovação.' <<< \"\$out\") -eq 2 ] && grep -qF 'sem data de aprovação' <<< \"\$out\""
rm -rf "$R/.git"
check "fontes fora de um repositório git" bash -c "'$FS' --sources SPEC-008 | grep -qF 'Fora de um repositório git.'"
rm -rf "$R/nao" "$R/fin-outro.txt" "$TASKS/SPEC-008"
mv "$R/.fin-tasks" "$TASKS/SPEC-008"; mv "$R/.fin-r8" "$r8"; rm -rf "$H/memory"; mv "$R/.fin-mem" "$H/memory"

# Conteúdo dos documentos (--create e --validate).
cp "$r8" "$R/.fin-r8"; cp -R "$TASKS/SPEC-008" "$R/.fin-tasks"
sed -i.bak 's/^status: .*/status: implemented/' "$r8" && rm -f "$r8.bak"
for t in "$TASKS"/SPEC-008/TASK-*.md; do sed -i.bak 's/^status: .*/status: done/' "$t" && rm -f "$t.bak"; done
sed -i.bak 's|^- `[^`]*` (automática): .*|Não se aplica.|' "$TASKS/SPEC-008/TASK-02.md" && rm -f "$TASKS/SPEC-008/TASK-02.md.bak"
FC="$H/work/finalize/SPEC-008/content.md"
check "--validate sem conteúdo pede o --create" bash -c "'$FS' --validate SPEC-008 2>&1 | grep -qF 'conteúdo não encontrado' && '$FS' --validate SPEC-008 2>&1 | grep -qF -- '--create'"
check "--create cria o conteúdo" bash -c "'$FS' --create SPEC-008 | grep -qF 'criado em packages/harness/work/finalize/SPEC-008/content.md: 1 bloco(s) de evidência.'" 
for h in '# Executivo' '## Resumo' '## Problema e resultado' '## O que muda para quem usa' '## Entregas' '## Próximos passos' '# Técnico' '## Visão geral' '## Arquitetura e componentes' '## O que foi feito' '## Decisões' '## Aprendizados' '## Pontos de atenção' '## Referências' '# Evidências'; do
  check "conteúdo tem a seção '$h'" grep -qxF "$h" "$FC"
done
check "conteúdo traz o ID da spec e o Markdown aceito" bash -c "grep -q 'dos documentos de SPEC-008' '$FC' && grep -qF 'Markdown aceito' '$FC'"
check "um bloco por evidência, com como chegar e como ler" bash -c "[ \$(grep -c '^## TASK-[0-9]* / ' '$FC') -eq 1 ] && grep -qxF '## TASK-01 / saida.txt' '$FC' && grep -qxF -- '- **Como chegar:** #{EVIDENCE_01_HOW}' '$FC' && grep -qxF -- '- **Como ler:** #{EVIDENCE_01_READ}' '$FC'"
check "tarefa com 'Não se aplica.' não gera bloco" bash -c "! grep -q 'TASK-02 / ' '$FC'"
sum_fc="$(sum "$FC")"
fails "--create repetido é recusado" "$FS" --create SPEC-008
check "--create repetido não altera o conteúdo" bash -c "[ '$sum_fc' = \"\$(cksum < '$FC')\" ] && '$FS' --create SPEC-008 2>&1 | grep -qF 'o conteúdo de SPEC-008 já existe'"
out="$("$FS" --validate SPEC-008 2>&1)"
check "--validate aponta as variáveis sem preencher" bash -c "[ \$(grep -c 'variável sem preencher: linha [0-9]*: ' <<< '$out') -ge 14 ]"
check "--validate aponta a variável do bloco de evidência" bash -c "grep -qF 'EVIDENCE_01_HOW' <<< '$out'"
check "--validate aponta a evidência ausente" bash -c "grep -qF 'TASK-01: evidência ausente: packages/harness/work/tasks/SPEC-008/evidence/TASK-01/saida.txt' <<< '$out'"
check "--validate sai com erro e não altera o conteúdo" bash -c "! '$FS' --validate SPEC-008 >/dev/null 2>&1 && [ '$sum_fc' = \"\$(cksum < '$FC')\" ]"
sed -i.bak 's/#{[A-Z0-9_]*}/texto/g' "$FC" && rm -f "$FC.bak"
mkdir -p "$TASKS/SPEC-008/evidence/TASK-01" && echo ok > "$TASKS/SPEC-008/evidence/TASK-01/saida.txt"
check "conteúdo preenchido e com a evidência é válido" bash -c "'$FS' --validate SPEC-008 | grep -qF 'Conteúdo de SPEC-008 válido: packages/harness/work/finalize/SPEC-008/content.md'"
cp "$FC" "$R/.fin-content"
sed -i.bak '/^## Resumo$/,/^## Problema e resultado$/{/^texto$/d;}' "$FC" && rm -f "$FC.bak"
check "--validate aponta a seção vazia" bash -c "'$FS' --validate SPEC-008 2>&1 | grep -qxF '  - seção vazia: ## Resumo'"
check "seção só com comentário é vazia" bash -c "[ \$(grep -c '^## Resumo\$' '$FC') -eq 1 ] && sed -n '/^## Resumo\$/,/^## Problema/p' '$FC' | grep -q '^<!--'"
cp "$R/.fin-content" "$FC"
sed -i.bak '/^## Entregas$/d' "$FC" && rm -f "$FC.bak"
check "--validate aponta a seção ausente" bash -c "'$FS' --validate SPEC-008 2>&1 | grep -qxF '  - seção ausente: ## Entregas'"
cp "$R/.fin-content" "$FC"
sed -i.bak '/^- \*\*Como ler:\*\* /d' "$FC" && rm -f "$FC.bak"
check "--validate aponta o bloco sem 'Como ler'" bash -c "'$FS' --validate SPEC-008 2>&1 | grep -qxF '  - bloco de evidência sem \"Como ler\": ## TASK-01 / saida.txt'"
cp "$R/.fin-content" "$FC"
sed -i.bak 's/^- \*\*Como chegar:\*\* texto$/- **Como chegar:** /' "$FC" && rm -f "$FC.bak"
check "--validate aponta o 'Como chegar' sem texto" bash -c "'$FS' --validate SPEC-008 2>&1 | grep -qxF '  - bloco de evidência sem \"Como chegar\": ## TASK-01 / saida.txt'"
cp "$R/.fin-content" "$FC"
sed -i.bak '/^## TASK-01 \/ saida.txt$/d' "$FC" && rm -f "$FC.bak"
check "--validate aponta a evidência sem bloco" bash -c "'$FS' --validate SPEC-008 2>&1 | grep -qxF '  - evidência sem bloco: ## TASK-01 / saida.txt'"
cp "$R/.fin-content" "$FC"
rm -f "$TASKS/SPEC-008/evidence/TASK-01/saida.txt"
check "--validate aponta só a evidência ausente (CB-03)" bash -c "out=\$('$FS' --validate SPEC-008 2>&1); [ \$(grep -c '^  - ' <<< \"\$out\") -eq 1 ] && grep -qF 'evidência ausente' <<< \"\$out\""
sum_fc="$(sum "$FC")"; "$FS" --validate SPEC-008 > /dev/null 2>&1
check "--validate não altera o conteúdo" bash -c "[ '$sum_fc' = \"\$(cksum < '$FC')\" ]"
rm -rf "$H/work/finalize" "$R/.fin-content" "$TASKS/SPEC-008"
mv "$R/.fin-tasks" "$TASKS/SPEC-008"; mv "$R/.fin-r8" "$r8"

# Tokens, Markdown restrito e escape dos documentos.
fl() { ( HARNESS_DIR="$H"; for l in config text frontmatter markdown spec-validation adjustments plan task memory finalize; do source "$H/scripts/lib/$l.sh"; done; "$@" ); }
mdcheck() { # descrição, entrada (formato do printf), HTML esperado (formato do printf)
  local got; got="$(printf -- "$2" | fl finalize_markdown_html)"
  check "$1" [ "$got" = "$(printf -- "$3")" ]
}
DOCS="$H/config/documents.yml"
check "documents.yml: limite em megabytes" grep -qx 'embed_max_mb: 10' "$DOCS"
check "documents.yml: grupos de tema e fonte" bash -c "grep -qx 'dark:' '$DOCS' && grep -qx 'light:' '$DOCS' && grep -qx 'font:' '$DOCS'"
check "documents.yml: extensões de cada tipo" bash -c "source '$H/scripts/lib/config.sh' && config_list '$DOCS' text_extensions | grep -qx log && config_list '$DOCS' image_extensions | grep -qx png && config_list '$DOCS' video_extensions | grep -qx mp4"
check "documents.yml: tokens do diff nos dois temas" bash -c "[ \$(grep -c 'color_diff_add:' '$DOCS') -eq 2 ] && [ \$(grep -c 'color_diff_del:' '$DOCS') -eq 2 ] && [ \$(grep -c 'color_diff_add_soft:\|color_diff_del_soft:' '$DOCS') -eq 4 ]"
css="$(fl finalize_css_tokens)"
check "tokens: tema escuro em :root, na ordem do arquivo" [ "$(sed -n 1,3p <<< "$css" | paste -sd '|' -)" = ':root {|  color-scheme: dark;|  --color-bg: #110e18;' ]
check "tokens: tema claro em data-theme" bash -c "grep -qxF ':root[data-theme=\"light\"] {' <<< '$css' && grep -qxF '  --color-bg: #f7f5fb;' <<< '$css' && grep -qxF '  color-scheme: light;' <<< '$css'"
check "tokens: chave com sublinhado vira hífen" bash -c "grep -qxF '  --color-surface-2: #221d2f;' <<< '$css' && grep -qxF '  --color-primary-soft: #2a2140;' <<< '$css'"
check "tokens: fontes só no tema escuro" bash -c "[ \$(grep -c -- '--font-' <<< '$css') -eq 2 ] && grep -qF -- '--font-body: system-ui' <<< '$css' && grep -qF -- '--font-mono: ui-monospace' <<< '$css' && ! sed -n '/data-theme/,\$p' <<< '$css' | grep -q -- '--font-'"
check "tokens: cores do diff nos dois temas" bash -c "[ \$(grep -c -- '--color-diff-add:' <<< '$css') -eq 2 ] && [ \$(grep -c -- '--color-diff-del-soft:' <<< '$css') -eq 2 ]"
cp "$DOCS" "$R/.documents-ok"
sed -i.bak 's/^  color_primary: "#a78bfa"/  color_primary: "#123456"/' "$DOCS" && rm -f "$DOCS.bak"
css="$(fl finalize_css_tokens)"
check "tokens: trocar a cor principal muda a saída (RN-05)" bash -c "grep -qxF '  --color-primary: #123456;' <<< '$css' && grep -qxF '  --color-primary: #6d28d9;' <<< '$css' && ! grep -q '#a78bfa' <<< '$css'"
sed -i.bak 's/^  color_primary: "#6d28d9"/  color_primary: "#654321"  # roxo do tema claro/' "$DOCS" && rm -f "$DOCS.bak"
css="$(fl finalize_css_tokens)"
check "tokens: comentário no fim da linha é removido" bash -c "grep -qxF '  --color-primary: #654321;' <<< '$css'"
sed -i.bak '/^light:/d' "$DOCS" && rm -f "$DOCS.bak"
fails "tokens sem o grupo light" fl finalize_css_tokens
got="$(fl finalize_css_tokens 2>&1 || true)"
check "tokens sem light: mensagem" grep -qF "sem os grupos 'dark' e 'light'" <<< "$got"
cp "$R/.documents-ok" "$DOCS"; rm -f "$R/.documents-ok"
got="$(printf '%s' "<b>\"a\" & 'x'</b>" | fl finalize_html_escape)"
check "escape: & < > e aspas" [ "$got" = '&lt;b&gt;&quot;a&quot; &amp; &#39;x&#39;&lt;/b&gt;' ]
mdcheck "markdown: parágrafos juntam as linhas" 'linha um\nlinha dois\n\nsegundo\n' '<p>linha um linha dois</p>\n<p>segundo</p>'
mdcheck "markdown: lista com marcadores" '- a\n- b\n' '<ul>\n<li>a</li>\n<li>b</li>\n</ul>'
mdcheck "markdown: lista numerada" '1. a\n2. b\n' '<ol>\n<li>a</li>\n<li>b</li>\n</ol>'
mdcheck "markdown: tabela com cabeçalho" '| A | B |\n|---|---|\n| 1 | 2 |\n' '<table>\n<thead>\n<tr><th>A</th><th>B</th></tr>\n</thead>\n<tbody>\n<tr><td>1</td><td>2</td></tr>\n</tbody>\n</table>'
mdcheck "markdown: tabela sem separador não tem cabeçalho" '| 1 | 2 |\n| 3 | 4 |\n' '<table>\n<tbody>\n<tr><td>1</td><td>2</td></tr>\n<tr><td>3</td><td>4</td></tr>\n</tbody>\n</table>'
mdcheck "markdown: código e negrito" 'usa `cmd` e **negrito**\n' '<p>usa <code>cmd</code> e <strong>negrito</strong></p>'
mdcheck "markdown: negrito dentro de código não é convertido" '`**x**`\n' '<p><code>**x**</code></p>'
mdcheck "markdown: negrito e código em item de lista e célula" '- **a** `b`\n\n| **c** |\n|---|\n| `d` |\n' '<ul>\n<li><strong>a</strong> <code>b</code></li>\n</ul>\n<table>\n<thead>\n<tr><th><strong>c</strong></th></tr>\n</thead>\n<tbody>\n<tr><td><code>d</code></td></tr>\n</tbody>\n</table>'
mdcheck "markdown: texto com < > & é escapado (CB-06)" 'a < b & c > d <script>x</script>\n' '<p>a &lt; b &amp; c &gt; d &lt;script&gt;x&lt;/script&gt;</p>'
mdcheck "markdown: código, lista e tabela escapados" '`<b>`\n\n- <i>\n\n| <u> |\n|---|\n| & |\n' '<p><code>&lt;b&gt;</code></p>\n<ul>\n<li>&lt;i&gt;</li>\n</ul>\n<table>\n<thead>\n<tr><th>&lt;u&gt;</th></tr>\n</thead>\n<tbody>\n<tr><td>&amp;</td></tr>\n</tbody>\n</table>'
mdcheck "markdown: aspas do texto são escapadas" "diz \"oi\" e 'tchau'\n" '<p>diz &quot;oi&quot; e &#39;tchau&#39;</p>'
mdcheck "markdown: crase sem par fica literal" 'a `b **c**\n' '<p>a `b <strong>c</strong></p>'
mdcheck "markdown: comentários são descartados" '<!-- nota -->\ntexto\n<!-- outra -->\n' '<p>texto</p>'
mdcheck "markdown: blocos seguidos sem linha em branco" 'antes\n- a\ndepois\n' '<p>antes</p>\n<ul>\n<li>a</li>\n</ul>\n<p>depois</p>'
mdcheck "markdown: CRLF" 'a\r\n\r\n- b\r\n' '<p>a</p>\n<ul>\n<li>b</li>\n</ul>'
mdcheck "markdown: entrada vazia gera saída vazia" '' ''
got="$(fl finalize_markdown_html < "$H/templates/finalize/content.md")"
check "markdown converte o conteúdo do template, sem os comentários" bash -c "grep -qxF '<p>#{EXEC_SUMMARY}</p>' <<< '$got' && ! grep -q 'Markdown aceito' <<< '$got'"

# --- Commits sem liberação ---
CC="$H/scripts/default/git/check-commits.sh"
AC="$H/skills/default/approve-commit/scripts/approve-commit.sh"
ACK="$H/.state/commit-ack"
g() { git -C "$R" -c user.name=Teste -c user.email=t@t -c commit.gpgsign=false -c core.hooksPath=/dev/null "$@"; }
token() { "$AC" --show | sed -n 's/^Código de liberação: //p'; }
head_of() { g rev-parse "$1"; }
rm -rf "$R/.git"  # repositório criado na seção de ajustes; aqui o histórico começa do zero

check "check-commits sem repositório git" bash -c "'$CC' | grep -q 'nada a verificar'"
check "check-commits sem git não cria referência" [ ! -e "$ACK" ]
fails "approve-commit sem repositório git" "$AC" --show
g init -q -b main
printf '%s\n' '/.*' '!/.claude/' > "$R/.git/info/exclude"  # rascunhos da suíte ficam fora dos commits
check "check-commits em repositório sem commits" "$CC"
rm -f "$ACK"
echo base > "$R/base.txt"; g add -A; g commit -qm 'base'
fails "approve-commit sem referência" "$AC" --show
check "primeira verificação grava a referência" bash -c "'$CC' | grep -q 'Referência de commits gravada'"
check "referência fora do git" bash -c "[ -z \"\$(git -C '$R' status --porcelain -- packages/harness/.state)\" ]"
check "sem commits novos, passa" bash -c "'$CC' | grep -qx 'Nenhum commit sem liberação.'"
check "check-commits de outro diretório" bash -c "cd /tmp && '$CC'"
fails "check-commits com argumento" "$CC" extra
check "alteração sem commit não bloqueia" bash -c "echo x >> '$R/base.txt' && '$CC' && git -C '$R' checkout -q -- base.txt"
check "stage sem commit não bloqueia" bash -c "echo y > '$R/novo.txt' && git -C '$R' add novo.txt && '$CC'; r=\$?; git -C '$R' rm -q --cached novo.txt; rm -f '$R/novo.txt'; exit \$r"

# Commit comum: bloqueia, lista e não altera nada.
echo um > "$R/um arquivo.txt"; g add -A; g commit -qm 'commit do agente'
before="$(sum "$ACK")"; head="$(head_of HEAD)"
fails "commit comum bloqueia" "$CC"
"$CC" > "$R/.cc" 2>&1 || true
check "bloqueio lista o commit e os arquivos" grep -qF "commit do agente (arquivos: um arquivo.txt)" "$R/.cc"
check "bloqueio manda parar e cita a regra" bash -c "grep -qF 'PARE: responda com o template \`blocker\`' '$R/.cc' && grep -qF 'agent/boundaries-04' '$R/.cc'"
check "check-commits não altera referência nem HEAD" bash -c "[ '$before' = \"\$(cksum < '$ACK')\" ] && [ '$head' = \"\$(git -C '$R' rev-parse HEAD)\" ]"
check "--show lista e dá o código" bash -c "'$AC' --show | grep -qF 'commit do agente' && [ -n \"\$('$AC' --show | sed -n 's/^Código de liberação: //p')\" ]"
check "--show não altera nada" bash -c "'$AC' --show >/dev/null && [ '$before' = \"\$(cksum < '$ACK')\" ]"
fails "--approve sem código" "$AC" --approve
fails "--approve com código errado" "$AC" --approve 000000000000
fails "argumento desconhecido" "$AC" --liberar
check "--approve com código errado não libera" bash -c "! '$CC' >/dev/null 2>&1"
t="$(token)"
echo dois > "$R/dois.txt"; g add -A; g commit -qm 'segundo commit'
fails "--approve com código desatualizado (commit depois do --show)" "$AC" --approve "$t"
check "--approve com o código atual" bash -c "'$AC' --approve '$(token)' | grep -qF 'Liberação gravada: 2 commit(s)'"
check "liberado, passa" "$CC"
check "--approve sem nada a liberar" bash -c "'$AC' --approve '$(token)' | grep -qx 'Nenhum commit sem liberação.'"

# Caminhos que contornam o `git commit`.
fails "commit com --no-verify bloqueia" bash -c "echo a >> '$R/base.txt' && git -C '$R' -c user.name=T -c user.email=t@t commit -qam nv --no-verify && '$CC'"
g reset -q --hard HEAD~1
check "commit não liberado desfeito à mão: nada a liberar" "$CC"
tree="$(g write-tree)"; parent="$(head_of HEAD)"
c="$(printf 'baixo nível\n' | g commit-tree "$tree" -p "$parent")"; g update-ref refs/heads/main "$c"
fails "commit-tree + update-ref bloqueia" "$CC"
"$AC" --approve "$(token)" > /dev/null
g commit -q --amend --allow-empty -m 'amend'
"$CC" > "$R/.cc" 2>&1 || true
check "--amend: commit novo e removido" bash -c "grep -qF 'Commits novos:' '$R/.cc' && grep -qF 'Commits removidos' '$R/.cc' && grep -qF 'baixo nível' '$R/.cc'"
"$AC" --approve "$(token)" > /dev/null
g reset -q --hard HEAD~1
"$CC" > "$R/.cc" 2>&1 || true
check "reset para trás: commit removido bloqueia" bash -c "grep -qF 'Commits removidos' '$R/.cc' && ! grep -qF 'Commits novos:' '$R/.cc'"
fails "--revert recusa commits removidos" "$AC" --revert "$(token)"
"$AC" --approve "$(token)" > /dev/null
g branch outra; c="$(printf 'em outra branch\n' | g commit-tree "$(g write-tree)" -p "$(head_of HEAD)")"; g update-ref refs/heads/outra "$c"
fails "commit em outra branch, sem trocar de branch, bloqueia" "$CC"
fails "--revert recusa commit fora da branch atual" "$AC" --revert "$(token)"
"$AC" --approve "$(token)" > /dev/null
check "trocar de branch sem commit não bloqueia" bash -c "git -C '$R' checkout -q outra && '$CC' && git -C '$R' checkout -q main && '$CC'"
check "criar branch e tag sem commit não bloqueia" bash -c "git -C '$R' branch nova && git -C '$R' tag v1 && '$CC'"
g checkout -q --detach; echo d > "$R/d.txt"; g add -A; g commit -qm 'detached'
fails "commit em HEAD solto (detached) bloqueia" "$CC"
fails "--revert recusa HEAD solto" "$AC" --revert "$(token)"
g checkout -q main
check "commit sem ref, abandonado, não bloqueia" "$CC"
c="$(printf 'só numa tag\n' | g commit-tree "$(g write-tree)" -p "$(head_of HEAD)")"; g tag -a -m t v2 "$c"
fails "commit só alcançável por tag anotada bloqueia" "$CC"
g tag -d v2 > /dev/null
check "tag apagada antes da liberação: nada a liberar" "$CC"
g stash list > /dev/null; echo s >> "$R/base.txt"; g stash -q
check "stash não conta como commit" "$CC"
g stash drop -q

# Referência corrompida.
cp "$ACK" "$R/.ack-ok"
printf 'lixo\n' >> "$ACK"
fails "referência com linha inválida bloqueia" "$CC"
fails "--revert com referência inválida" "$AC" --revert "$(token)"
check "--approve refaz a referência inválida" bash -c "'$AC' --approve '$(token)' | grep -qF 'Referência de commits refeita' && '$CC'"
printf '%s refs/heads/main\n' 0123456789012345678901234567890123456789 > "$ACK"
check "referência com commit inexistente bloqueia" bash -c "'$CC' 2>&1 | grep -qF 'commit liberado não existe mais'"
: > "$ACK"
fails "referência vazia bloqueia o histórico inteiro" "$CC"
cp "$R/.ack-ok" "$ACK"
check "referência restaurada passa" "$CC"
rm -f "$ACK"
check "referência apagada é regravada" bash -c "'$CC' | grep -q 'gravada'"

# Desfazer.
echo r1 > "$R/r1.txt"; echo base2 >> "$R/base.txt"; g add -A; g commit -qm 'r1'
echo r2 > "$R/r2.txt"; g add -A; g commit -qm 'r2'
check "--revert desfaz os commits da branch" bash -c "'$AC' --revert '$(token)' | grep -qF '2 commit(s) desfeito(s) em main'"
check "--revert mantém as alterações fora de stage" bash -c "[ -f '$R/r1.txt' ] && [ -f '$R/r2.txt' ] && grep -q base2 '$R/base.txt' && [ -z \"\$(git -C '$R' diff --cached --name-only)\" ]"
check "depois do --revert, passa" "$CC"
g add -A; g commit -qm 'enviado'
g init -q --bare "$R/.remote.git"; g remote add origin "$R/.remote.git"; g push -q origin main 2> /dev/null
fails "commit já enviado ao remoto ainda bloqueia" "$CC"
check "bloqueio marca o commit enviado" bash -c "'$CC' 2>&1 | grep -q '^- [0-9a-f]* enviado (arquivos: .*) \[já enviado ao remoto\]\$'"
head="$(head_of HEAD)"
fails "--revert recusa commit enviado ao remoto" "$AC" --revert "$(token)"
check "--revert recusado não altera HEAD" [ "$head" = "$(head_of HEAD)" ]
"$AC" --approve "$(token)" > /dev/null
check "fetch e refs remotas não bloqueiam" bash -c "git -C '$R' fetch -q origin && '$CC'"

# Commit pelo harness: o pedido do usuário é a liberação.
echo c1 > "$R/c1.txt"; echo c2 > "$R/c 2.txt"; echo st > "$R/st.txt"; g add st.txt
check "--commit sem mensagem: motivo" bash -c "printf '  \n' | '$AC' --commit c1.txt 2>&1 | grep -q 'mensagem de commit vazia'"
check "--commit sem mensagem não commita" [ "$(head_of HEAD)" = "$(g rev-parse HEAD)" ]
head="$(head_of HEAD)"
check "--commit com caminhos commita e libera" bash -c "printf 'feat: c1 e c 2\n\ncorpo \"com\" aspas\n' | env GIT_AUTHOR_NAME=T GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=T GIT_COMMITTER_EMAIL=t@t '$AC' --commit c1.txt 'c 2.txt' | grep -qF 'Commit feito e liberado:' && '$CC'"
check "--commit com caminhos commita só eles" bash -c "[ \"\$(git -C '$R' show --format= --name-only HEAD | paste -sd '|' -)\" = 'c 2.txt|c1.txt' ] && [ \"\$(git -C '$R' rev-parse HEAD^)\" = '$head' ]"
check "--commit de arquivo removido" bash -c "git -C '$R' rm -q --cached 'c 2.txt' >/dev/null; git -C '$R' reset -q -- 'c 2.txt'; rm -f '$R/c 2.txt' && echo 'chore: remove c 2' | env GIT_AUTHOR_NAME=T GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=T GIT_COMMITTER_EMAIL=t@t '$AC' --commit 'c 2.txt' >/dev/null && [ -z \"\$(git -C '$R' ls-files 'c 2.txt')\" ] && '$CC'"
fails "--commit com caminho inexistente" bash -c "echo 'feat: nada' | env GIT_AUTHOR_NAME=T GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=T GIT_COMMITTER_EMAIL=t@t '$AC' --commit nao-existe.txt"
check "--commit com caminhos mantém o resto em stage" bash -c "[ \"\$(git -C '$R' diff --cached --name-only)\" = st.txt ]"
check "--commit sem caminhos commita o stage" bash -c "echo 'feat: stage' | env GIT_AUTHOR_NAME=T GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=T GIT_COMMITTER_EMAIL=t@t '$AC' --commit >/dev/null && '$CC' && [ \"\$(git -C '$R' log -1 --format=%s)\" = 'feat: stage' ]"
head="$(head_of HEAD)"; before="$(sum "$ACK")"
fails "--commit sem nada a commitar" bash -c "echo 'feat: vazio' | env GIT_AUTHOR_NAME=T GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=T GIT_COMMITTER_EMAIL=t@t '$AC' --commit"
check "--commit recusado não altera HEAD nem referência" bash -c "[ '$head' = \"\$(git -C '$R' rev-parse HEAD)\" ] && [ '$before' = \"\$(cksum < '$ACK')\" ]"
echo x > "$R/x.txt"; g add x.txt; g commit -qm 'por fora'
fails "--commit com commit sem liberação pendente" bash -c "echo 'feat: c3' | env GIT_AUTHOR_NAME=T GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=T GIT_COMMITTER_EMAIL=t@t '$AC' --commit x.txt"
check "--commit recusado mantém o commit por fora apontado" bash -c "'$CC' 2>&1 | grep -qF 'por fora'"
"$AC" --approve "$(token)" > /dev/null
cp "$ACK" "$R/.ack-ok"; printf 'lixo\n' >> "$ACK"; echo y > "$R/y.txt"; head="$(head_of HEAD)"
fails "--commit com referência inválida" bash -c "echo 'feat: y' | env GIT_AUTHOR_NAME=T GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=T GIT_COMMITTER_EMAIL=t@t '$AC' --commit y.txt"
check "--commit com referência inválida não commita" [ "$head" = "$(head_of HEAD)" ]
cp "$R/.ack-ok" "$ACK"
printf '#!/bin/sh\nexit 1\n' > "$R/.git/hooks/pre-commit"; chmod +x "$R/.git/hooks/pre-commit"; before="$(sum "$ACK")"
fails "--commit barrado por hook do git" bash -c "echo 'feat: y' | env GIT_AUTHOR_NAME=T GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=T GIT_COMMITTER_EMAIL=t@t '$AC' --commit y.txt"
check "--commit barrado não libera nada" bash -c "[ '$head' = \"\$(git -C '$R' rev-parse HEAD)\" ] && [ '$before' = \"\$(cksum < '$ACK')\" ]"
rm -f "$R/.git/hooks/pre-commit" "$R/y.txt"
check "depois do --commit, commit por fora continua apontado" bash -c "echo z > '$R/z.txt' && git -C '$R' add z.txt && git -C '$R' -c user.name=T -c user.email=t@t commit -qm 'z por fora' && ! '$CC' >/dev/null 2>&1"
"$AC" --approve "$(token)" > /dev/null

# Volume: muitos commits.
for i in $(seq 1 150); do echo "$i" > "$R/v.txt"; g add v.txt; g commit -qm "v$i"; done
check "150 commits: bloqueia e lista todos" bash -c "[ \$('$CC' 2>&1 | grep -c '^- .* v[0-9]* (arquivos: v.txt)') -eq 150 ]"
check "150 commits: --revert" bash -c "'$AC' --revert '$(token)' | grep -qF '150 commit(s) desfeito(s)' && '$CC'"

# --- Skills ---
fails "nome de skill inválido" "$C" 'Bad_Name'
fails "nome reservado" "$C" claude-x
check "cria skill custom" "$C" review-x
fails "skill duplicada" "$C" review-x
check "cria script da skill" "$C" --script review-x list-files
fails "finalize com template sem preencher" "$C" --finalize review-x
s="$H/skills/custom/review-x/SKILL.md"
python3 - "$s" <<'EOF'
import sys,re; p=sys.argv[1]; s=open(p).read()
s=s.replace('description: "#{SKILL_DESCRIPTION}"','description: >\n  Revisa código: usa | regras.\n  Use quando pedirem review.')
s=re.sub(r'  (requires|scripts|templates): .*\n','',s); s=re.sub(r'#\{[A-Z_]+\}','x',s); open(p,'w').write(s)
EOF
fill "$H/skills/custom/review-x/scripts/list-files.sh"
check "finalize com description em bloco" "$C" --finalize review-x
check "catálogo com a description unida" grep -qF '| review-x | Revisa código: usa \| regras. Use quando pedirem review. |' "$H/context/skill-catalog.md"
check "redirecionamento preserva o bloco" grep -q '^description: >$' "$R/.claude/skills/review-x/SKILL.md"
check "finalize repetido não duplica no catálogo" bash -c "'$C' --finalize review-x >/dev/null && [ \$(grep -c '| review-x |' '$H/context/skill-catalog.md') -eq 1 ]"
sed -i.bak 's/^name: review-x/name: outro/' "$s" && rm -f "$s.bak"
fails "name diferente da pasta" "$C" --finalize review-x
check "finalize das skills padrão" bash -c "'$C' --finalize create-skill >/dev/null && '$C' --finalize refine-spec >/dev/null && '$C' --finalize adjust-spec >/dev/null && '$C' --finalize approve-spec >/dev/null && '$C' --finalize plan-tasks >/dev/null && '$C' --finalize adjust-plan >/dev/null && '$C' --finalize approve-plan >/dev/null && '$C' --finalize approve-commit >/dev/null && '$C' --finalize approve-task >/dev/null && '$C' --finalize adjust-task >/dev/null && '$C' --finalize execute-task >/dev/null && '$C' --finalize validate-task >/dev/null"

echo
echo "Passou: $PASS  Falhou: $FAIL"
[ "$FAIL" -eq 0 ]
