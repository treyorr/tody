#!/usr/bin/env sh
set -eu

REPO_OWNER="${REPO_OWNER:-treyorr}"
REPO_NAME="${REPO_NAME:-tody}"
BINDIR="${BINDIR:-$HOME/.local/bin}"

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "error: missing required command: $1" >&2
    exit 1
  fi
}

detect_target() {
  os="$(uname -s)"
  arch="$(uname -m)"

  case "${os}:${arch}" in
    Linux:x86_64) echo "x86_64-unknown-linux-gnu" ;;
    Linux:aarch64 | Linux:arm64) echo "aarch64-unknown-linux-gnu" ;;
    Darwin:x86_64) echo "x86_64-apple-darwin" ;;
    Darwin:arm64 | Darwin:aarch64) echo "aarch64-apple-darwin" ;;
    *)
      echo "error: unsupported platform: os=${os} arch=${arch}" >&2
      echo "supported targets: x86_64/aarch64 for macOS and Linux" >&2
      exit 1
      ;;
  esac
}

resolve_tag() {
  if [ -n "${TODY_VERSION:-}" ]; then
    case "${TODY_VERSION}" in
      v*) echo "${TODY_VERSION}" ;;
      *) echo "v${TODY_VERSION}" ;;
    esac
    return
  fi

  latest_json="$(curl -fsSL "https://api.github.com/repos/${REPO_OWNER}/${REPO_NAME}/releases/latest")"
  tag="$(printf '%s\n' "${latest_json}" | sed -n 's/.*"tag_name":[[:space:]]*"\([^"]*\)".*/\1/p' | head -n1)"

  if [ -z "${tag}" ]; then
    echo "error: unable to resolve latest release tag from GitHub API" >&2
    exit 1
  fi

  echo "${tag}"
}

require_cmd curl
require_cmd tar
require_cmd mktemp
require_cmd uname

target="$(detect_target)"
tag="$(resolve_tag)"
asset="tody-${tag}-${target}.tar.gz"
url="https://github.com/${REPO_OWNER}/${REPO_NAME}/releases/download/${tag}/${asset}"

tmpdir="$(mktemp -d)"
trap 'rm -rf "${tmpdir}"' EXIT INT TERM

echo "-> installing tody ${tag} for ${target}"
echo "-> download: ${url}"

mkdir -p "${BINDIR}"
curl -fsSL "${url}" -o "${tmpdir}/${asset}"
tar -xzf "${tmpdir}/${asset}" -C "${tmpdir}"

if [ ! -f "${tmpdir}/tody" ]; then
  echo "error: release archive does not contain a top-level 'tody' binary" >&2
  exit 1
fi

if command -v install >/dev/null 2>&1; then
  install -m 0755 "${tmpdir}/tody" "${BINDIR}/tody"
else
  cp "${tmpdir}/tody" "${BINDIR}/tody"
  chmod 0755 "${BINDIR}/tody"
fi

echo "-> installed: ${BINDIR}/tody"

case ":${PATH}:" in
  *:"${BINDIR}":*)
    echo "ready: run 'tody --version'"
    ;;
  *)
    echo "warning: ${BINDIR} is not on PATH."
    echo "  add this to your shell config:"
    echo "  export PATH=\"${BINDIR}:\$PATH\""
    ;;
esac
