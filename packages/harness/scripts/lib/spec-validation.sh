# Validação da spec refinada, compartilhada entre o refinamento (create-refined-spec.sh)
# e o ajuste (adjust-spec.sh). As seções de cada tipo vêm de config/specs.yml.
# Uso: source "$HARNESS_DIR/scripts/lib/spec-validation.sh"
#      (requer HARNESS_DIR e as libs config.sh, text.sh, frontmatter.sh e markdown.sh)

# Ações aceitas na tabela de componentes afetados. `criar` exige que o caminho ainda não exista;
# `alterar` e `remover` exigem que exista. Caminhos são relativos a `paths.root` (main.yml).
SPEC_COMPONENT_ACTIONS="criar alterar remover"

_spec_config() {
  printf '%s/config/specs.yml' "$HARNESS_DIR"
}

# Caminho do template de uma seção (`<seção>.md` em `sections_dir`).
# spec_section_template <seção>
spec_section_template() {
  local templates_dir
  templates_dir="$(harness_path "$(config_get "$HARNESS_DIR/config/main.yml" paths.templates)")"
  printf '%s/%s/%s.md' "$templates_dir" "$(config_get "$(_spec_config)" sections_dir)" "$1"
}

# Título da seção, lido da primeira linha do template (`## Título`).
# spec_section_heading <seção>
spec_section_heading() {
  head -n 1 "$(spec_section_template "$1")" | sed 's/^## //'
}

# Imprime um erro por linha para cada problema da refinada. Não imprime nada se ela estiver válida.
refined_errors() {
  local file="$1" type sections section heading text requirement row path action root_dir

  root_dir="$(cd "$(harness_path "$(config_get "$HARNESS_DIR/config/main.yml" paths.root)")" && pwd)"

  template_leftovers "$file" | sed 's/^/variável sem preencher: /' || true

  type="$(frontmatter_get "$file" type 2>/dev/null || true)"
  if ! config_list "$(_spec_config)" types | grep -qxF "$type"; then
    echo "o campo 'type' do front-matter é inválido: '$type'"
    return
  fi
  sections="$(config_list "$(_spec_config)" "sections_$type")"

  # Toda seção do tipo existe e tem conteúdo; seções com tabela têm ao menos uma linha.
  while IFS= read -r section; do
    heading="$(spec_section_heading "$section")"
    if ! text="$(markdown_section "$file" "$heading")"; then
      echo "seção ausente: '## $heading'"
    elif grep -q '^|' "$(spec_section_template "$section")"; then
      [ -n "$(markdown_table_rows <<< "$text")" ] ||
        echo "a tabela da seção '$heading' está vazia (use uma linha com '-' se não se aplica)"
    elif ! grep -q '[^[:space:]]' <<< "$text"; then
      echo "a seção '$heading' está vazia"
    fi
  done <<< "$sections"

  # Todo requisito funcional aparece em algum caso de teste e em algum critério de aceite.
  if grep -qxF functional-requirements <<< "$sections"; then
    while IFS= read -r row; do
      requirement="$(markdown_cell "$row" 1)"
      [ "$requirement" != "-" ] || continue
      if [[ ! "$requirement" =~ ^RF-[0-9]+$ ]]; then
        echo "ID de requisito funcional inválido: '$requirement' (use RF-01, RF-02...)"
        continue
      fi
      for section in test-cases acceptance-criteria; do
        grep -qxF "$section" <<< "$sections" || continue
        heading="$(spec_section_heading "$section")"
        markdown_section "$file" "$heading" 2>/dev/null | grep -qE "(^|[^A-Z0-9-])$requirement([^0-9]|\$)" ||
          echo "$requirement não aparece em '$heading'"
      done
    done <<< "$(markdown_section "$file" "$(spec_section_heading functional-requirements)" 2>/dev/null | markdown_table_rows)"
  fi

  # Análise: todo argumento da tese e da antítese é considerado em alguma conclusão da síntese.
  if grep -qxF synthesis <<< "$sections"; then
    text="$(markdown_section "$file" "$(spec_section_heading synthesis)" 2>/dev/null | markdown_table_rows)"
    for section in thesis antithesis; do
      grep -qxF "$section" <<< "$sections" || continue
      while IFS= read -r row; do
        [ -n "$row" ] || continue
        requirement="$(markdown_cell "$row" 1)"
        [ "$requirement" != "-" ] || continue
        grep -qE "(^|[^A-Z0-9-])$requirement([^0-9]|\$)" <<< "$text" ||
          echo "$requirement ('$(spec_section_heading "$section")') não é considerado em nenhuma conclusão da síntese"
      done <<< "$(markdown_section "$file" "$(spec_section_heading "$section")" 2>/dev/null | markdown_table_rows)"
    done
  fi

  # Componentes afetados: ação conhecida e caminho coerente com ela.
  if grep -qxF affected-components <<< "$sections"; then
    while IFS= read -r row; do
      [ -n "$row" ] || continue
      path="$(markdown_cell "$row" 1 | tr -d '`')"
      action="$(markdown_cell "$row" 2)"
      [ "$path" != "-" ] || continue
      case " $SPEC_COMPONENT_ACTIONS " in
        *" $action "*) ;;
        *) echo "ação inválida para '$path': '$action' (use: $SPEC_COMPONENT_ACTIONS)"; continue ;;
      esac
      if [ "$action" = criar ] && [ -e "$root_dir/$path" ]; then
        echo "'$path' já existe; use a ação 'alterar'"
      elif [ "$action" != criar ] && [ ! -e "$root_dir/$path" ]; then
        echo "'$path' não existe em $root_dir; use a ação 'criar' ou corrija o caminho"
      fi
    done <<< "$(markdown_section "$file" "$(spec_section_heading affected-components)" 2>/dev/null | markdown_table_rows)"
  fi
}
