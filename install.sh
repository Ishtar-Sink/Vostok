#!/bin/sh
# Instala o binário vostok a partir do GitHub Releases.
#
# Uso: install.sh [--version <versão>] [--dir <pasta>] [<pasta>]
#
# Variáveis de ambiente:
#   VOSTOK_REPO         repositório no GitHub (padrão: Ishtar-Sink/Vostok)
#   VOSTOK_VERSION      versão a instalar, como 0.1.0 (padrão: a última release)
#   VOSTOK_INSTALL_DIR  pasta de destino (padrão: ~/.local/bin)
#   VOSTOK_OS           simula a saída de `uname -s`, para testar outras plataformas
#   VOSTOK_ARCH         simula a saída de `uname -m`, para testar outras plataformas

set -eu

AVAILABLE_TARGETS="x86_64-unknown-linux-gnu
aarch64-unknown-linux-gnu
x86_64-apple-darwin
aarch64-apple-darwin"

repo="${VOSTOK_REPO:-Ishtar-Sink/Vostok}"
version="${VOSTOK_VERSION:-}"
install_dir="${VOSTOK_INSTALL_DIR:-${HOME}/.local/bin}"
work_dir=""

fail() {
  echo "install.sh: $*" >&2
  exit 1
}

usage() {
  cat << 'EOF'
Uso: install.sh [--version <versão>] [--dir <pasta>] [<pasta>]

Variáveis: VOSTOK_REPO, VOSTOK_VERSION, VOSTOK_INSTALL_DIR, VOSTOK_OS e VOSTOK_ARCH.
EOF
}

cleanup() {
  if [ -n "$work_dir" ]; then
    rm -rf "$work_dir"
  fi
}

parse_args() {
  while [ "$#" -gt 0 ]; do
    case "$1" in
      --version)
        [ "$#" -ge 2 ] || fail "--version precisa de um valor."
        version="$2"
        shift 2
        ;;
      --dir)
        [ "$#" -ge 2 ] || fail "--dir precisa de um valor."
        install_dir="$2"
        shift 2
        ;;
      -h | --help)
        usage
        exit 0
        ;;
      -*)
        fail "opção desconhecida: $1"
        ;;
      *)
        install_dir="$1"
        shift
        ;;
    esac
  done
  version="${version#v}"
}

detect_target() {
  os="${VOSTOK_OS:-$(uname -s)}"
  arch="${VOSTOK_ARCH:-$(uname -m)}"

  case "$os" in
    Linux) os_part="unknown-linux-gnu" ;;
    Darwin) os_part="apple-darwin" ;;
    *) os_part="" ;;
  esac

  case "$arch" in
    x86_64 | amd64) arch_part="x86_64" ;;
    aarch64 | arm64) arch_part="aarch64" ;;
    *) arch_part="" ;;
  esac

  candidate="${arch_part}-${os_part}"
  if [ -z "$os_part" ] || [ -z "$arch_part" ] || ! is_available "$candidate"; then
    {
      echo "install.sh: não há binário do vostok para ${os} ${arch}."
      echo "Plataformas disponíveis:"
      echo "$AVAILABLE_TARGETS" | sed 's/^/  /'
    } >&2
    exit 1
  fi
  target="$candidate"
}

is_available() {
  echo "$AVAILABLE_TARGETS" | grep -qx "$1"
}

download() {
  if command -v curl > /dev/null 2>&1; then
    curl -fsSL --proto '=https' --tlsv1.2 -o "$2" "$1"
  elif command -v wget > /dev/null 2>&1; then
    wget -q --https-only -O "$2" "$1"
  else
    fail "é preciso ter curl ou wget."
  fi
}

resolve_version() {
  if [ -n "$version" ]; then
    return
  fi
  download "https://api.github.com/repos/${repo}/releases/latest" "${work_dir}/latest.json" ||
    fail "não foi possível consultar a última release de ${repo}."
  version="$(sed -n 's/.*"tag_name": *"v\{0,1\}\([^"]*\)".*/\1/p' "${work_dir}/latest.json" | head -n 1)"
  [ -n "$version" ] || fail "não foi possível ler a versão da última release de ${repo}."
}

sha256_of() {
  if command -v sha256sum > /dev/null 2>&1; then
    sha256sum "$1" | cut -d ' ' -f 1
  elif command -v shasum > /dev/null 2>&1; then
    shasum -a 256 "$1" | cut -d ' ' -f 1
  else
    fail "é preciso ter sha256sum ou shasum para conferir o download."
  fi
}

download_and_verify() {
  archive="vostok-${version}-${target}.tar.gz"
  base_url="https://github.com/${repo}/releases/download/v${version}"

  echo "Baixando ${archive} de ${repo}..."
  download "${base_url}/${archive}" "${work_dir}/${archive}" ||
    fail "não foi possível baixar ${base_url}/${archive}."
  download "${base_url}/${archive}.sha256" "${work_dir}/${archive}.sha256" ||
    fail "não foi possível baixar o checksum ${base_url}/${archive}.sha256."

  expected="$(cut -d ' ' -f 1 "${work_dir}/${archive}.sha256")"
  actual="$(sha256_of "${work_dir}/${archive}")"
  [ -n "$expected" ] && [ "$expected" = "$actual" ] ||
    fail "checksum diferente para ${archive}: esperado ${expected}, obtido ${actual}."
}

install_binary() {
  tar -xzf "${work_dir}/${archive}" -C "$work_dir" vostok ||
    fail "não foi possível extrair ${archive}."
  mkdir -p "$install_dir" || fail "não foi possível criar ${install_dir}."
  # Copia com outro nome e renomeia, para não deixar um binário pela metade se a cópia falhar.
  cp "${work_dir}/vostok" "${install_dir}/.vostok.tmp" &&
    chmod 755 "${install_dir}/.vostok.tmp" &&
    mv -f "${install_dir}/.vostok.tmp" "${install_dir}/vostok" ||
    fail "não foi possível instalar em ${install_dir}."
  echo "vostok ${version} instalado em ${install_dir}/vostok."
}

warn_if_not_in_path() {
  case ":${PATH}:" in
    *":${install_dir}:"*) ;;
    *) echo "Aviso: ${install_dir} não está no PATH; acrescente-o para usar o vostok." >&2 ;;
  esac
}

main() {
  parse_args "$@"
  detect_target
  work_dir="$(mktemp -d)"
  trap cleanup EXIT
  trap 'exit 1' INT TERM
  resolve_version
  download_and_verify
  install_binary
  warn_if_not_in_path
}

main "$@"
