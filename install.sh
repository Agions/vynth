#!/usr/bin/env bash
set -euo pipefail

# Synerix installer — installs the latest release binary
# Usage (GitHub):
#   curl -fsSL https://raw.githubusercontent.com/Agions/synerix/main/install.sh | bash
# Usage (Gitee):
#   curl -fsSL https://gitee.com/Agions/synerix/raw/main/install.sh | bash

REPO="Agions/synerix"
BINARY="synerix"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"

# Auto-detect source: prefer GitHub, fallback to Gitee
GITHUB_API="https://api.github.com/repos"
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

# Try GitHub first, fallback to Gitee
get_latest_tag() {
    local tag=""

    # Try GitHub
    tag=$(curl -fsSL "${GITHUB_API}/${REPO}/releases/latest" 2>/dev/null \
        | grep -o '"tag_name":"[^"]*"' | head -1 | cut -d'"' -f4) || true

    # Fallback: Gitee
    if [ -z "$tag" ]; then
        tag=$(curl -fsSL "${GITEE_API}/${REPO}/releases?page=1&per_page=1" 2>/dev/null \
            | grep -o '"tag_name":"[^"]*"' | head -1 | cut -d'"' -f4) || true
    fi

    if [ -z "$tag" ]; then
        echo "0.0.1"
    else
        echo "$tag"
    fi
}

# Try GitHub release download, fallback to Gitee
download_release() {
    local tag="$1"
    local archive_name="$2"
    local dest="$3"

    local github_url="https://github.com/${REPO}/releases/download/${tag}/${archive_name}"
    local gitee_url="https://gitee.com/${REPO}/releases/download/${tag}/${archive_name}"

    # Try GitHub first (faster globally)
    if curl -fsSL "$github_url" -o "$dest" 2>/dev/null; then
        return 0
    fi

    # Fallback: Gitee
    if curl -fsSL "$gitee_url" -o "$dest" 2>/dev/null; then
        return 0
    fi

    return 1
}

build_from_source() {
    local tag="$1"
    local tmp_dir="$2"

    if ! command -v cargo >/dev/null 2>&1; then
        warn "cargo not found, installing Rust via rustup..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y 2>&1
        # shellcheck source=/dev/null
        [ -f "$HOME/.cargo/env" ] && source "$HOME/.cargo/env"
        export PATH="$HOME/.cargo/bin:$PATH"
        if ! command -v cargo >/dev/null 2>&1; then
            error "Rust installation failed. Please install manually: https://rustup.rs"
        fi
        ok "Rust installed successfully"
    fi

    info "Cloning source at ${tag}..."
    git clone --depth 1 --branch "$tag" "https://github.com/${REPO}.git" "${tmp_dir}/${BINARY}" 2>&1 \
        || git clone --depth 1 "https://github.com/${REPO}.git" "${tmp_dir}/${BINARY}" 2>&1 \
        || git clone --depth 1 --branch "$tag" "https://gitee.com/${REPO}.git" "${tmp_dir}/${BINARY}" 2>&1 \
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
    local tmp_dir
    tmp_dir=$(mktemp -d)
    trap 'rm -rf "${tmp_dir:-}"' EXIT

    info "Downloading ${BINARY} ${tag} for ${OS}/${ARCH}..."

    if ! download_release "$tag" "$archive_name" "${tmp_dir}/${archive_name}"; then
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
    echo "  ║     Synerix — AI Coding Terminal       ║"
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
    ok "安装完成！运行 'synerix' 开始使用。"
    echo ""
    echo "  配置文件: ~/.config/synerix/config.toml"
    echo "  文档: https://github.com/Agions/synerix"
    echo ""
}

main "$@"
