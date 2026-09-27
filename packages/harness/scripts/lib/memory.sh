# Acesso à memória do agente, configurada em `config/memory.yml`.
# Uso: source "$HARNESS_DIR/scripts/lib/memory.sh"
#      (requer HARNESS_DIR definido e lib/config.sh carregado)

MEMORY_CONFIG="$HARNESS_DIR/config/memory.yml"

# Linha que abre uma entrada: `## <PREFIXO>-<número>: <título>`.
MEMORY_ENTRY_PATTERN='^## [A-Z]+-[0-9]+: '

memory_dir() {
  harness_path "$(config_get "$HARNESS_DIR/config/main.yml" paths.agent_memory)"
}

# Falha com mensagem se o tipo não estiver em `types`.
memory_require_type() {
  config_list "$MEMORY_CONFIG" types | grep -qxF "$1" ||
    { echo "Tipo de memória inválido '$1'. Tipos: $(config_list "$MEMORY_CONFIG" types | tr '\n' ' ')" >&2; return 1; }
}

# memory_file <tipo>
memory_file() {
  printf '%s/%s' "$(memory_dir)" "$(config_get "$MEMORY_CONFIG" "$1.file")"
}

# Imprime o tipo cujo prefixo corresponde ao do ID (ex.: DEC-001 -> decision).
# memory_type_for_id <ID>
memory_type_for_id() {
  local type
  while IFS= read -r type; do
    if [ "$(config_get "$MEMORY_CONFIG" "$type.prefix")" = "${1%%-*}" ]; then
      printf '%s' "$type"
      return 0
    fi
  done <<< "$(config_list "$MEMORY_CONFIG" types)"
  echo "Nenhum tipo de memória usa o prefixo do ID '$1'" >&2
  return 1
}
