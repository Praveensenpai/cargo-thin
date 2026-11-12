#!/usr/bin/env bash
# 🧹 Cargo Thin (カーゴ・シン) installer
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/Praveensenpai/cargo-thin/main/install.sh | bash
#   curl -fsSL https://raw.githubusercontent.com/Praveensenpai/cargo-thin/main/install.sh | bash -s -- --version v0.1.0
#
# No Rust toolchain is required: the script downloads a prebuilt
# x86_64-unknown-linux-gnu binary and verifies its SHA-256 checksum.
set -euo pipefail
IFS=$'\n\t'

REPO="Praveensenpai/cargo-thin"
BIN_NAME="cargo-thin"
ASSET_NAME="cargo-thin-x86_64-linux.tar.gz"
BIN_DIR="${XDG_BIN_HOME:-${HOME}/.local/bin}"
VERSION=""

TMP_DIR="$(mktemp -d -t cargo_thin_install.XXXXXXXXXX)"
cleanup() {
    local exit_code=$?
    rm -rf "${TMP_DIR}"
    exit "${exit_code}"
}
trap cleanup EXIT INT TERM HUP

log_info()  { printf "\033[1;34mℹ\033[0m %s\n" "$*" >&2; }
log_ok()    { printf "\033[1;32m✔\033[0m %s\n" "$*" >&2; }
log_warn()  { printf "\033[1;33m⚠\033[0m %s\n" "$*" >&2; }
log_error() { printf "\033[1;31m✖\033[0m %s\n" "$*" >&2; }

usage() {
    cat << EOF
Cargo Thin installer

Usage:
  install.sh [--version <tag>] [--dir <path>]

Options:
  --version <tag>  Install a specific release (for example: v0.1.0)
  --dir <path>     Install directory (default: ${BIN_DIR})
  -h, --help       Show this help

Examples:
  curl -fsSL https://raw.githubusercontent.com/${REPO}/main/install.sh | bash
  curl -fsSL https://raw.githubusercontent.com/${REPO}/main/install.sh | bash -s -- --version v0.1.0
EOF
}

parse_args() {
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --version)
                [[ $# -ge 2 ]] || { log_error "--version requires a release tag"; exit 1; }
                VERSION="$2"
                shift 2
                ;;
            --dir)
                [[ $# -ge 2 ]] || { log_error "--dir requires a path"; exit 1; }
                BIN_DIR="$2"
                shift 2
                ;;
            -h|--help)
                usage
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                usage
                exit 1
                ;;
        esac
    done
    if [[ -n "${VERSION}" && "${VERSION}" != v* ]]; then
        VERSION="v${VERSION}"
    fi
}

detect_target() {
    local os arch
    os="$(uname -s)"
    arch="$(uname -m)"

    if [[ "${os}" != "Linux" ]]; then
        log_error "Unsupported OS: ${os}. Prebuilt binaries target Linux only."
        exit 1
    fi

    case "${arch}" in
        x86_64|amd64) ;;
        *)
            log_error "Unsupported architecture: ${arch}. Published binaries target x86_64 only."
            exit 1
            ;;
    esac
}

resolve_version() {
    if [[ -n "${VERSION}" ]]; then
        return 0
    fi

    log_info "Resolving the latest release..."
    local latest_url
    latest_url="$(curl -fsSL --connect-timeout 20 --retry 5 --retry-delay 2 --retry-all-errors \
        -o /dev/null -w '%{url_effective}' "https://github.com/${REPO}/releases/latest")"
    VERSION="${latest_url##*/}"

    if [[ "${VERSION}" != v* ]]; then
        log_error "Could not resolve a published release for ${REPO}."
        log_error "Pass --version <tag> manually, or build from source instead."
        exit 1
    fi
}

sha256_of() {
    if command -v sha256sum > /dev/null 2>&1; then
        sha256sum "$1" | awk '{print $1}'
    elif command -v shasum > /dev/null 2>&1; then
        shasum -a 256 "$1" | awk '{print $1}'
    else
        log_error "No SHA-256 tool found (need 'sha256sum' or 'shasum')."
        exit 1
    fi
}

verify_checksum() {
    local archive="$1" sums_file="$2" expected actual
    expected="$(awk 'NR == 1 {print $1}' "${sums_file}")"

    if [[ ! "${expected}" =~ ^[a-fA-F0-9]{64}$ ]]; then
        log_error "Malformed checksum file: ${sums_file}"
        exit 1
    fi

    actual="$(sha256_of "${archive}")"
    if [[ "${actual}" != "${expected}" ]]; then
        log_error "Checksum mismatch for ${ASSET_NAME}."
        log_error "expected: ${expected}"
        log_error "actual:   ${actual}"
        log_error "Aborting: the download may be corrupted or tampered with."
        exit 1
    fi
    log_ok "Checksum verified."
}

download_release() {
    local base_url="https://github.com/${REPO}/releases/download/${VERSION}"
    local archive="${TMP_DIR}/${ASSET_NAME}"
    local sums_file="${TMP_DIR}/${ASSET_NAME}.sha256"

    log_info "Downloading ${ASSET_NAME} (${VERSION})..."
    curl -fSL --connect-timeout 20 --retry 5 --retry-delay 2 --retry-all-errors \
        "${base_url}/${ASSET_NAME}" -o "${archive}"
    curl -fsSL --connect-timeout 20 --retry 5 --retry-delay 2 --retry-all-errors \
        "${base_url}/${ASSET_NAME}.sha256" -o "${sums_file}"

    verify_checksum "${archive}" "${sums_file}"

    log_info "Extracting archive..."
    tar -xzf "${archive}" -C "${TMP_DIR}"
}

install_binary() {
    local extracted="${TMP_DIR}/${BIN_NAME}"

    if [[ ! -f "${extracted}" ]]; then
        extracted="$(find "${TMP_DIR}" -maxdepth 2 -type f -name "${BIN_NAME}" | head -n 1)"
    fi
    if [[ -z "${extracted}" || ! -f "${extracted}" ]]; then
        log_error "Failed to locate the ${BIN_NAME} binary inside the archive."
        exit 1
    fi

    mkdir -p "${BIN_DIR}"
    install -Dm 755 "${extracted}" "${BIN_DIR}/${BIN_NAME}"
    log_ok "Installed ${BIN_DIR}/${BIN_NAME} (${VERSION})"
}

ensure_path() {
    case ":${PATH:-}:" in
        *":${BIN_DIR}:"*) ;;
        *)
            log_warn "${BIN_DIR} is not on your PATH."
            log_info "Add it to your shell profile:"
            printf "%s\n" "    export PATH=\"${BIN_DIR}:\$PATH\"" >&2
            ;;
    esac
}

verify_binary() {
    local output
    if output="$("${BIN_DIR}/${BIN_NAME}" --version 2> /dev/null)"; then
        log_ok "Verified: ${output}"
    else
        log_warn "The binary was installed but could not be executed."
        log_warn "It requires a glibc-based Linux distribution on x86_64."
    fi
}

main() {
    parse_args "$@"
    detect_target
    resolve_version
    download_release
    install_binary
    ensure_path
    verify_binary

    printf '\n  \033[1;32m✔\033[0m \033[1mCargo Thin is ready.\033[0m\n'
    printf '  Run: \033[1;36mcargo thin --path ~/Projects\033[0m\n\n'
}

main "$@"
