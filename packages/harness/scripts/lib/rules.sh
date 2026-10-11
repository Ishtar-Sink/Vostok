# Validação das regras do projeto e do mapa de contexto das skills.
# Uso: source "$HARNESS_DIR/scripts/lib/rules.sh"   (requer HARNESS_DIR e config.sh carregados)
# Cada função imprime uma linha por problema e retorna 1 se houver algum.

# Imprime as entradas do `config/context.yml`, uma por linha, separadas por TAB:
#   required<TAB><caminho>
#   skill<TAB><nome><TAB><caminho>   (caminho vazio para skill sem itens)
_rules_context_entries() {
  awk '
    function item(s) {
      sub(/^[ \t]*-[ \t]+/, "", s); sub(/[ \t]+#.*$/, "", s); gsub(/[ \t]+$/, "", s)
      if (s ~ /^".*"$/ || s ~ /^\047.*\047$/) s = substr(s, 2, length(s) - 2)
      return s
    }
    /^[ \t]*(#|$)/ { next }
    /^[^ \t]/ {
      block = ""; skill = ""
      if ($0 ~ /^required:/) block = "required"
      else if ($0 ~ /^skills:/) block = "skills"
      next
    }
    block == "required" && /^[ \t]*-[ \t]/ { print "required\t" item($0); next }
    block == "skills" && /^  [^ \t-][^:]*:/ {
      skill = $0; sub(/^  /, "", skill); sub(/[ \t]*:.*$/, "", skill)
      print "skill\t" skill "\t"
      next
    }
    block == "skills" && skill != "" && /^[ \t]+-[ \t]/ { print "skill\t" skill "\t" item($0); next }
  ' "$HARNESS_DIR/config/context.yml"
}

# Confere o mapa de contexto: caminhos de `required` e das skills existem (RF-13) e
# cada skill do mapa existe em `dirs.default` ou `dirs.custom` (RF-12).
rules_validate_map() {
  local skills_cfg="$HARNESS_DIR/config/skills.yml" ctx="config/context.yml"
  local rel_default rel_custom dir_default dir_custom kind a b
  local bad=0 seen=" "
  rel_default="$(config_get "$skills_cfg" dirs.default)"
  rel_custom="$(config_get "$skills_cfg" dirs.custom)"
  dir_default="$(harness_path "$rel_default")"
  dir_custom="$(harness_path "$rel_custom")"
  while IFS=$'\t' read -r kind a b; do
    if [ "$kind" = required ]; then
      [ -e "$(harness_path "$a")" ] || { echo "$ctx: required: caminho inexistente: $a"; bad=1; }
      continue
    fi
    case "$seen" in
      *" $a "*) ;;
      *)
        seen="$seen$a "
        if [ ! -d "$dir_default/$a" ] && [ ! -d "$dir_custom/$a" ]; then
          echo "$ctx: skills.$a: skill inexistente em $rel_default e $rel_custom"
          bad=1
        fi
        ;;
    esac
    if [ -n "$b" ] && [ ! -e "$(harness_path "$b")" ]; then
      echo "$ctx: skills.$a: caminho inexistente: $b"; bad=1
    fi
  done < <(_rules_context_entries)
  return $bad
}

# Confere um arquivo de categoria de `rules/project/`: nome em kebab-case (RF-14), título na
# primeira linha, frase de propósito antes da primeira regra e IDs `[project/<arquivo>-NN]:`
# sem repetição e em sequência a partir de 01 (RF-15). Regra removida conta na sequência.
# Saída: `<arquivo>:<linha>: <problema>`.
# rules_validate_category <arquivo>
rules_validate_category() {
  local file="$1" base out bad=0
  base="$(basename "$file")"
  if ! printf '%s' "$base" | grep -Eq '^[a-z0-9]+(-[a-z0-9]+)*\.md$'; then
    echo "$file:1: nome fora de kebab-case: $base"; bad=1
  fi
  out="$(awk -v file="$file" -v name="${base%.md}" '
    function err(n, msg) { print file ":" n ": " msg }
    NR == 1 && $0 !~ /^# [^ \t]/ { err(1, "primeira linha deve ser o título \"# <título>\"") }
    /^\[[^]]*\]:/ {
      if (!purpose && !warned) { err(NR, "falta a frase de propósito antes da primeira regra"); warned = 1 }
      id = $0; sub(/\].*$/, "", id); sub(/^\[/, "", id)
      pre = "project/" name "-"
      if ($0 !~ /^\[[^]\/]+\/[a-z0-9-]+-[0-9][0-9]\]: ./ && $0 !~ /^\[project\/[^]]*\]: ./) {
        err(NR, "ID fora do formato [project/" name "-NN]: " id); next
      }
      if (substr(id, 1, length(pre)) != pre || id !~ /-[0-9][0-9]$/ || length(id) != length(pre) + 2) {
        err(NR, "ID fora do formato [project/" name "-NN]: " id); next
      }
      n = substr(id, length(pre) + 1) + 0
      if (id in seen) { err(NR, "ID repetido: " id); next }
      seen[id] = 1
      if (n != expected + 1) err(NR, sprintf("ID fora de sequência: esperado %02d, encontrado %02d", expected + 1, n))
      expected = n
      seenrule = 1
      next
    }
    NR > 1 && !seenrule && $0 !~ /^[ \t]*$/ && $0 !~ /^#/ { purpose = 1 }
    END { if (NR > 0 && !purpose && !warned) err(NR, "falta a frase de propósito antes da primeira regra") }
  ' "$file")"
  if [ -n "$out" ]; then printf '%s\n' "$out"; bad=1; fi
  return $bad
}
