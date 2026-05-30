#!/usr/bin/env bash
set -euo pipefail

# Syncode installer — installs the latest release binary
# Usage: curl -fsSL https://gitee.com/Agions/syncode/raw/main/install.sh | bash

REPO="Agions/syncode"
BINARY="syncode"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"
GITEE_API="https://gitee.com/api/v5/repos"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
CYAN='\033[0;36m'
NC='\033[0m'

info()  { echo -e "${CYAN}[info]${NC} $*"; }
ok()    { echo -e "${GREEN}[ok]${NC} $*"; }
warn()  { echo -e "${YELLOW}[warn]${NC} $*"; }
error() { echo -e "${RED}[error]${NC} $*" >&2; exit 1; }

detect_os() {
    case "$(uname -s)" in
        Linux*)   OS="linux" ;;
        Darwin*)  OS="macos" ;;
        MINGW*|MSYS*|CYGWIN*) OS="windows" ;;
        *)        error "Unsupported OS: $(uname -s)" ;;
    esac
}

detect_arch() {
    case "$(uname -m)" in
        x86_64|amd64)  ARCH="x86_64" ;;
        aarch64|arm64) ARCH="aarch64" ;;
        *)             error "Unsupported architecture: $(uname -m)" ;;
    esac
}

check_deps() {
    for cmd in curl tar; do
        command -v "$cmd" >/dev/null 2>&1 || error "Required command not found: $cmd"
    done
}
get_latest_tag() {
    local tag
    tag=$(curl -fsSL "${GITEE_API}/${REPO}/releases?page=1&per_page=1" 2>/dev/null \
        | grep -o '"tag_name":"[^"]*"' | head -1 | cut -d'"' -f4) || true
    if [ -z "$tag" ]; then
        echo "v1.0.0"
    else
        echo "$tag"
    fi
}

build_from_source() {
    local tag="$1"
    local tmp_dir="$2"

    if ! command -v cargo >/dev/null 2>&1; then
        error "cargo not found. Install Rust first: https://rustup.rs"
    fi

    info "Cloning source at ${tag}..."
    git clone --depth 1 --branch "$tag" "https://gitee.com/${REPO}.git" "${tmp_dir}/${BINARY}" 2>&1 \
        || git clone --depth 1 "https://gitee.com/${REPO}.git" "${tmp_dir}/${BINARY}" 2>&1

    info "Building (this may take a few minutes)..."
    cd "${tmp_dir}/${BINARY}"
    cargo build --release 2>&1

    local binary_path="target/release/${BINARY}"
    if [ ! -f "$binary_path" ]; then
        error "Build failed"
    fi

    info "Installing to ${INSTALL_DIR}/${BINARY}..."
    if [ -w "$INSTALL_DIR" ]; then
        cp "$binary_path" "${INSTALL_DIR}/${BINARY}"
        chmod +x "${INSTALL_DIR}/${BINARY}"
    else
        sudo cp "$binary_path" "${INSTALL_DIR}/${BINARY}"
        sudo chmod +x "${INSTALL_DIR}/${BINARY}"
    fi

    ok "Built and installed ${BINARY} ${tag} to ${INSTALL_DIR}/${BINARY}"
}

install_binary() {
    local tag="$1"
    local archive_name="${BINARY}-${tag}-${OS}-${ARCH}.tar.gz"
    local download_url="https://gitee.com/${REPO}/releases/download/${tag}/${archive_name}"
    local tmp_dir
    tmp_dir=$(mktemp -d)
    trap 'rm -rf "${tmp_dir:-}"' EXIT

    info "Downloading ${BINARY} ${tag} for ${OS}/${ARCH}..."

    if ! curl -fsSL "$download_url" -o "${tmp_dir}/${archive_name}" 2>/dev/null; then
        warn "Pre-built binary not available, attempting to build from source..."
        build_from_source "$tag" "$tmp_dir"
        return
    fi

    info "Extracting..."
    tar xzf "${tmp_dir}/${archive_name}" -C "$tmp_dir"

    local binary_path
    binary_path=$(find "$tmp_dir" -name "$BINARY" -type f | head -1)
    if [ -z "$binary_path" ]; then
        binary_path="${tmp_dir}/${BINARY}"
    fi
    if [ ! -f "$binary_path" ]; then
        error "Binary not found in archive"
    fi

    info "Installing to ${INSTALL_DIR}/${BINARY}..."
    if [ -w "$INSTALL_DIR" ]; then
        cp "$binary_path" "${INSTALL_DIR}/${BINARY}"
        chmod +x "${INSTALL_DIR}/${BINARY}"
    else
        sudo cp "$binary_path" "${INSTALL_DIR}/${BINARY}"
        sudo chmod +x "${INSTALL_DIR}/${BINARY}"
    fi

    ok "Installed ${BINARY} ${tag} to ${INSTALL_DIR}/${BINARY}"
}

verify() {
    if command -v "$BINARY" >/dev/null 2>&1; then
        local version
        version=$("$BINARY" --version 2>/dev/null || echo "unknown")
        ok "Installation verified: ${BINARY} ${version}"
    else
        warn "${BINARY} installed but not in PATH. Add ${INSTALL_DIR} to your PATH:"
        echo "  export PATH=\"${INSTALL_DIR}:\$PATH\""
    fi
}

main() {
    echo ""
    echo "  ╔═══════════════════════════════════════╗"
    echo "  ║     Syncode — AI 配对编程终端安装器     ║"
    echo "  ╚═══════════════════════════════════════╝"
    echo ""

    check_deps
    detect_os
    detect_arch
    info "Platform: ${OS}/${ARCH}"

    local tag
    tag=$(get_latest_tag)
    info "Latest release: ${tag}"

    install_binary "$tag"
    verify

    echo ""
    ok "安装完成！运行 'syncode' 开始使用。"
    echo ""
    echo "  配置文件: ~/.config/syncode/config.toml"
    echo "  文档: https://gitee.com/Agions/syncode"
    echo ""
}

main "$@"
