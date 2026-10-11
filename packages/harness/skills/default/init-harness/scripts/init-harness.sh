#!/usr/bin/env bash
# Operações determinísticas da skill init-harness: detectar sinais do projeto, gravar uma categoria de
# regras em `rules/project/<categoria>.md` e registrar onde a categoria é carregada (`config/context.yml`).
#
# Uso: init-harness.sh --detect
#        Lista os sinais encontrados na raiz do projeto, um por linha, no formato
#        `categoria: arquivo: sinal`. Projeto sem sinais não imprime nada.
#      init-harness.sh --write <categoria> <arquivo-de-regras>
#        Cria ou atualiza `rules/project/<categoria>.md` pelo template `templates/rules/category.md`,
#        numerando os IDs em sequência.
#      init-harness.sh --preview <categoria> <arquivo-de-regras>
#        Mostra o que o --write mudaria, sem gravar. Sem mudança, imprime "Nenhuma mudança."
#      init-harness.sh --load <categoria> always|<skill>[,<skill>...]
#        Registra a categoria em `required` (always) ou no mapa `skills` do `config/context.yml`.
#
# Arquivo de regras: uma regra por linha (um "- " inicial é ignorado). Linhas opcionais:
#   titulo: <título>        título da categoria nova (padrão: o nome da categoria)
#   proposito: <frase>      frase de propósito da categoria nova
#   remover: NN             marca a regra NN existente como removida (o ID não é reaproveitado)
# Linhas vazias e iniciadas por "#" são ignoradas. Regra já existente no arquivo não é repetida.
set -euo pipefail

# Raiz do harness: primeira pasta acima do script que contém `config/main.yml`.
HARNESS_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && until [ -f config/main.yml ] || [ "$PWD" = / ]; do cd ..; done && pwd)"
source "$HARNESS_DIR/scripts/lib/config.sh"

USAGE='Uso: init-harness.sh --detect | --write <categoria> <arquivo-de-regras> | --preview <categoria> <arquivo-de-regras> | --load <categoria> always|<skill>[,<skill>...]'

CONTEXT_CONFIG="$HARNESS_DIR/config/context.yml"
SKILLS_CONFIG="$HARNESS_DIR/config/skills.yml"
CATEGORY_TEMPLATE="$HARNESS_DIR/templates/rules/category.md"
RULES_DIR="$HARNESS_DIR/rules/project"

die() { echo "Erro: $*" >&2; exit 1; }

TMP_FILES=()
cleanup() { [ "${#TMP_FILES[@]}" -eq 0 ] || rm -f "${TMP_FILES[@]}"; }
trap cleanup EXIT
new_tmp() { local t; t="$(mktemp)"; TMP_FILES+=("$t"); printf '%s' "$t"; }

# Confere o nome da categoria: kebab-case.
check_category() {
  printf '%s' "$1" | grep -Eq '^[a-z0-9]+(-[a-z0-9]+)*$' ||
    die "categoria inválida: '$1' (use kebab-case, ex.: code-style)"
}

# --- detect ---

# Imprime os sinais de um arquivo de manifesto ou de configuração, relativos à raiz do projeto.
# detect_file <raiz> <caminho-relativo>
detect_file() {
  local root="$1" rel="$2" base file
  base="$(basename "$rel")"; file="$root/$rel"
  case "$base" in
    package.json)
      echo "stack: $rel: Node.js"
      ! grep -Eq '"typescript"' "$file" || echo "stack: $rel: TypeScript"
      ! grep -Eq '"(eslint|prettier|@biomejs/biome)"' "$file" || echo "code-style: $rel: eslint/prettier/biome nas dependências"
      ! grep -Eq '"(vitest|jest|mocha|playwright|cypress)"' "$file" || echo "testing: $rel: framework de teste nas dependências"
      ! grep -Eq '"(prisma|@prisma/client|typeorm|sequelize|mongoose|knex|drizzle-orm)"' "$file" || echo "database: $rel: ORM ou driver de banco nas dependências"
      ! grep -Eq '"(react|vue|svelte|@angular/core|next|nuxt)"' "$file" || echo "frontend: $rel: framework de interface nas dependências"
      ! grep -Eq '"(express|fastify|koa|hono|@nestjs/core)"' "$file" || echo "api: $rel: framework HTTP nas dependências"
      ;;
    Cargo.toml) echo "stack: $rel: Rust" ;;
    go.mod) echo "stack: $rel: Go" ;;
    pyproject.toml)
      echo "stack: $rel: Python"
      ! grep -Eq 'ruff|black|flake8' "$file" || echo "code-style: $rel: ruff/black/flake8"
      ! grep -Eq 'pytest' "$file" || echo "testing: $rel: pytest"
      ! grep -Eqi 'sqlalchemy|django' "$file" || echo "database: $rel: ORM"
      ! grep -Eqi 'fastapi|flask|django' "$file" || echo "api: $rel: framework HTTP"
      ;;
    requirements.txt) echo "stack: $rel: Python" ;;
    pom.xml|build.gradle|build.gradle.kts) echo "stack: $rel: Java/Kotlin" ;;
    Gemfile) echo "stack: $rel: Ruby" ;;
    composer.json) echo "stack: $rel: PHP" ;;
    Package.swift) echo "stack: $rel: Swift" ;;
    pubspec.yaml) echo "stack: $rel: Dart/Flutter" ;;
    *.csproj) echo "stack: $rel: .NET" ;;
    Dockerfile|docker-compose.yml|docker-compose.yaml) echo "stack: $rel: Docker" ;;
    .eslintrc|.eslintrc.*|eslint.config.*|.prettierrc|.prettierrc.*|prettier.config.*|.editorconfig|rustfmt.toml|.rustfmt.toml|ruff.toml|.flake8|.golangci.yml|biome.json|.swiftlint.yml)
      echo "code-style: $rel: configuração de lint ou formatação" ;;
    vitest.config.*|jest.config.*|pytest.ini|playwright.config.*|cypress.config.*|phpunit.xml|karma.conf.*)
      echo "testing: $rel: configuração de teste" ;;
    schema.prisma) echo "database: $rel: esquema do Prisma" ;;
    openapi.*|swagger.*) echo "api: $rel: contrato da API" ;;
    CONTRIBUTING.md|.pre-commit-config.yaml|.gitlab-ci.yml|Jenkinsfile)
      echo "development-methodology: $rel: processo de contribuição ou integração" ;;
    SECURITY.md|.env.example|.gitleaks.toml|.snyk|dependabot.yml)
      echo "security: $rel: política ou verificação de segurança" ;;
  esac
  return 0
}

# Imprime os sinais de uma pasta, relativos à raiz do projeto.
# detect_dir <caminho-relativo>
detect_dir() {
  local rel="$1"
  case "$(basename "$rel")" in
    src|apps|packages|services|libs|lib|cmd|internal) echo "architecture: $rel: pasta de código" ;;
    tests|test|__tests__) echo "testing: $rel: pasta de testes" ;;
    migrations|prisma) echo "database: $rel: pasta de banco de dados" ;;
    routes|controllers) echo "api: $rel: pasta de rotas" ;;
    components|pages) echo "frontend: $rel: pasta de interface" ;;
    workflows) case "$rel" in *.github/workflows) echo "development-methodology: $rel: integração contínua" ;; esac ;;
    .husky) echo "development-methodology: $rel: hooks de commit" ;;
  esac
  return 0
}

detect() {
  local root rel
  root="$(cd "$HARNESS_DIR/../.." && pwd)"
  cd "$root"
  {
    while IFS= read -r rel; do
      rel="${rel#./}"
      [ -n "$rel" ] && [ "$rel" != . ] || continue
      if [ -d "$rel" ]; then detect_dir "$rel"; else detect_file "$root" "$rel"; fi
    done < <(find . -maxdepth 3 \
      \( -name node_modules -o -name .git -o -name .claude -o -name .agents -o -name .venv \
         -o -name target -o -name dist -o -name build -o -name work -o -path ./packages/harness \) -prune \
      -o \( -type f -o -type d \) -print)
  } | LC_ALL=C sort -u
}

# --- write ---

# Gera o arquivo de categoria em <saida> a partir do arquivo de regras, sem tocar em rules/project.
# build_category <categoria> <arquivo-de-regras> <saida>
build_category() {
  local cat="$1" input="$2" out="$3" target="$RULES_DIR/$1.md"
  local title="" purpose="" removals=" " line text n next rules_file count=0
  [ -f "$input" ] || die "arquivo de regras não encontrado: $input"
  rules_file="$(new_tmp)"

  # Lê o arquivo de regras: cabeçalhos opcionais, remoções e regras novas (uma por linha).
  while IFS= read -r line || [ -n "$line" ]; do
    case "$line" in
      ''|'#'*) continue ;;
      titulo:*) title="$(printf '%s' "${line#titulo:}" | sed 's/^[[:space:]]*//;s/[[:space:]]*$//')"; continue ;;
      proposito:*) purpose="$(printf '%s' "${line#proposito:}" | sed 's/^[[:space:]]*//;s/[[:space:]]*$//')"; continue ;;
      remover:*)
        n="$(printf '%s' "${line#remover:}" | sed 's/[[:space:]]//g')"
        printf '%s' "$n" | grep -Eq '^[0-9]+$' || die "remover: espera o número da regra: '$line'"
        removals="$removals$((10#$n)) "; continue ;;
    esac
    text="$(printf '%s' "$line" | sed 's/^[[:space:]]*-[[:space:]]*//;s/^[[:space:]]*//;s/[[:space:]]*$//')"
    [ -n "$text" ] || continue
    printf '%s\n' "$text" >> "$rules_file"
    count=$((count + 1))
  done < "$input"
  [ "$count" -gt 0 ] || [ "$removals" != " " ] || die "o arquivo de regras não tem nenhuma regra: $input"

  if [ -f "$target" ]; then
    # Categoria existente: aplica as remoções e acrescenta só as regras novas, com os próximos IDs.
    next="$(awk -v pre="[project/$cat-" '
      index($0, pre) == 1 { n = substr($0, length(pre) + 1, 2) + 0; if (n > max) max = n }
      END { print max + 1 }
    ' "$target")"
    awk -v pre="[project/$cat-" -v removals="$removals" -v today="$(date +%F)" '
      index($0, pre) == 1 {
        n = substr($0, length(pre) + 1, 2) + 0
        if (index(removals, " " n " ") && $0 !~ /\]: Removida em /) {
          printf "%s%02d]: Removida em %s.\n", pre, n, today; next
        }
      }
      { print }
    ' "$target" > "$out"
    while IFS= read -r text; do
      # Repetir uma regra existente não cria um ID novo.
      if awk -v pre="[project/$cat-" -v text="$text" 'index($0, pre) == 1 && substr($0, index($0, "]: ") + 3) == text { found = 1 } END { exit !found }' "$out"; then
        continue
      fi
      [ -z "$(tail -c 1 "$out")" ] || echo >> "$out"
      printf '[project/%s-%02d]: %s\n' "$cat" "$next" "$text" >> "$out"
      next=$((next + 1))
    done < "$rules_file"
  else
    [ "$count" -gt 0 ] || die "a categoria '$cat' não existe: não há o que remover"
    [ -n "$title" ] || title="$cat"
    [ -n "$purpose" ] || purpose="Define as regras de $cat para o agente."
    # Numera as regras e preenche o template, sem o comentário de formato do início.
    local numbered; numbered="$(new_tmp)"
    awk -v cat="$cat" '{ printf "[project/%s-%02d]: %s\n", cat, NR, $0 }' "$rules_file" > "$numbered"
    awk -v title="$title" -v purpose="$purpose" -v rules="$numbered" '
      NR == 1 && /^<!--/ { skip = 1 }
      skip { if ($0 ~ /-->/) skip = 0; next }
      /^# #\{CATEGORY_TITLE\}$/ { print "# " title; next }
      /^#\{CATEGORY_PURPOSE\}$/ { print purpose; next }
      /^#\{CATEGORY_RULES\}$/ { while ((getline r < rules) > 0) print r; close(rules); next }
      { print }
    ' "$CATEGORY_TEMPLATE" > "$out"
  fi
}

write_category() {
  local cat="$1" input="$2" out
  check_category "$cat"
  out="$(new_tmp)"
  build_category "$cat" "$input" "$out"
  mkdir -p "$RULES_DIR"
  if [ -f "$RULES_DIR/$cat.md" ] && cmp -s "$out" "$RULES_DIR/$cat.md"; then
    echo "Nenhuma mudança em rules/project/$cat.md."
    return
  fi
  cat "$out" > "$RULES_DIR/$cat.md"
  echo "Categoria gravada: rules/project/$cat.md"
}

preview_category() {
  local cat="$1" input="$2" out
  check_category "$cat"
  out="$(new_tmp)"
  build_category "$cat" "$input" "$out"
  if [ ! -f "$RULES_DIR/$cat.md" ]; then
    echo "Categoria nova: rules/project/$cat.md"
    cat "$out"
  elif cmp -s "$out" "$RULES_DIR/$cat.md"; then
    echo "Nenhuma mudança."
  else
    diff -u "$RULES_DIR/$cat.md" "$out" || true
  fi
}

# --- load ---

# Registra <caminho> no `config/context.yml`: em `required` (modo required) ou na lista de <skill>
# do mapa `skills` (modo skill). Cria a entrada da skill se não existir. Não repete um item.
# context_add <required|skill> <skill|-> <caminho>
context_add() {
  local mode="$1" skill="$2" path="$3" tmp label status=0
  tmp="$(new_tmp)"
  awk -v mode="$mode" -v skill="$skill" -v path="$path" '
    # Comentários e linhas em branco ficam em espera e saem depois do item novo.
    function flush_pend(   i) { for (i = 1; i <= np; i++) print pend[i]; np = 0 }
    function leave_required() { if (inreq && !has) { print "  - " path; has = 1; done = 1 } }
    function leave_skill() { if (cur && !has) { print "    - " path; done = 1 } cur = 0 }
    function leave_skills() {
      leave_skill()
      if (inskills && mode == "skill" && !seen) { print "  " skill ":"; print "    - " path; done = 1; seen = 1 }
    }
    /^[ \t]*(#|$)/ && (inreq || inskills) { pend[++np] = $0; next }
    /^[^ \t#]/ {
      if (inreq) leave_required()
      if (inskills) leave_skills()
      flush_pend()
      inreq = ($0 ~ /^required:/ && mode == "required"); inskills = ($0 ~ /^skills:/ && mode == "skill")
      print; next
    }
    inreq {
      item = $0; sub(/^[ \t]*-[ \t]+/, "", item); sub(/[ \t]+#.*$/, "", item); gsub(/[ \t]+$/, "", item)
      if (item == path) has = 1
      flush_pend(); print; next
    }
    inskills && /^  [^ \t-][^:]*:/ {
      leave_skill()
      name = $0; sub(/^  /, "", name); sub(/[ \t]*:.*$/, "", name)
      flush_pend()
      if (name == skill) {
        seen = 1
        if ($0 ~ /:[ \t]*\[\][ \t]*$/) { print "  " name ":"; print "    - " path; done = 1; next }
        cur = 1; has = 0
      }
      print; next
    }
    inskills && cur && /^[ \t]+-[ \t]/ {
      item = $0; sub(/^[ \t]*-[ \t]+/, "", item); sub(/[ \t]+#.*$/, "", item); gsub(/[ \t]+$/, "", item)
      if (item == path) has = 1
      flush_pend(); print; next
    }
    { flush_pend(); print }
    END {
      if (inreq) leave_required()
      if (inskills) leave_skills()
      flush_pend()
      if (mode == "skill" && !seen && !inskills) { print "skills:"; print "  " skill ":"; print "    - " path; done = 1 }
      if (!done && !has) exit 3
    }
  ' "$CONTEXT_CONFIG" > "$tmp" || status=$?
  [ "$status" -eq 0 ] || die "não consegui registrar '$path' em config/context.yml"
  if [ "$mode" = required ]; then label="sempre"; else label="$skill"; fi
  if cmp -s "$tmp" "$CONTEXT_CONFIG"; then
    echo "Já registrado: $path ($label)."
  else
    cat "$tmp" > "$CONTEXT_CONFIG"
    echo "Registrado: $path ($label)."
  fi
}

load_category() {
  local cat="$1" where="$2" rel skill rel_default rel_custom
  check_category "$cat"
  rel="rules/project/$cat.md"
  [ -f "$HARNESS_DIR/$rel" ] || die "a categoria '$cat' não existe em $rel: grave-a antes de registrar onde ela é carregada"
  [ -n "$where" ] || die "informe 'always' ou as skills separadas por vírgula"
  if [ "$where" = always ]; then
    context_add required - "$rel"
    return
  fi
  rel_default="$(config_get "$SKILLS_CONFIG" dirs.default)"
  rel_custom="$(config_get "$SKILLS_CONFIG" dirs.custom)"
  # Valida todas as skills antes de gravar qualquer uma.
  for skill in $(printf '%s' "$where" | tr ',' ' '); do
    printf '%s' "$skill" | grep -Eq '^[a-z0-9]+(-[a-z0-9]+)*$' || die "nome de skill inválido: '$skill'"
    [ -d "$(harness_path "$rel_default")/$skill" ] || [ -d "$(harness_path "$rel_custom")/$skill" ] ||
      die "skill inexistente: '$skill'"
  done
  for skill in $(printf '%s' "$where" | tr ',' ' '); do
    context_add skill "$skill" "$rel"
  done
}

case "${1:-}" in
  --detect) [ "$#" -eq 1 ] || die "$USAGE"; detect ;;
  --write) [ "$#" -eq 3 ] || die "$USAGE"; write_category "$2" "$3" ;;
  --preview) [ "$#" -eq 3 ] || die "$USAGE"; preview_category "$2" "$3" ;;
  --load) [ "$#" -eq 3 ] || die "$USAGE"; load_category "$2" "$3" ;;
  *) die "$USAGE" ;;
esac
