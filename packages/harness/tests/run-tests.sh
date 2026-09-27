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
rm -rf "$H/work" "$H"/memory/*.md
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
check "sem content_hash: só grava o hash" bash -c "'$A' --record-manual SPEC-001 | grep -q 'Nenhum ajuste registrado'"
check "sem content_hash: adjustments não muda" [ "$(fm "$ref" adjustments)" = 0 ]
check "sem content_hash: hash gravado igual ao do refinamento" [ "$(fm "$ref" content_hash)" = "$(fm "$ref.orig" content_hash)" ]
rm -f "$ref.orig"

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

python3 - "$ref" <<'PY'
import sys, re; p = sys.argv[1]; s = open(p, encoding='utf-8').read()
s = s[:s.index('\n## Histórico de ajustes')] + '\n'
s = re.sub(r'\ncontent_hash: .*', '', s)
open(p, 'w', encoding='utf-8').write(s)
PY
"$A" --record-manual SPEC-001 > /dev/null
check "spec sem a seção de histórico: --record cria a seção" bash -c "'$A' --record SPEC-001 'Ajuste' >/dev/null && [ \$(grep -c '^## Histórico de ajustes\$' '$ref') -eq 1 ] && tail -1 '$ref' | grep -q '| agente | Ajuste |'"

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
check "finalize das skills padrão" bash -c "'$C' --finalize create-skill >/dev/null && '$C' --finalize refine-spec >/dev/null && '$C' --finalize adjust-spec >/dev/null"

echo
echo "Passou: $PASS  Falhou: $FAIL"
[ "$FAIL" -eq 0 ]
