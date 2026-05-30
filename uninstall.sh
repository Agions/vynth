#!/usr/bin/env bash
set -euo pipefail

# Syncode uninstaller
# Usage: curl -fsSL https://gitee.com/Agions/syncode/raw/main/uninstall.sh | bash
#   or: bash uninstall.sh [--all]

BINARY="syncode"
CONFIG_DIR="${HOME}/.config/syncode"
CACHE_DIR="${HOME}/.cache/syncode"
DATA_DIR="${HOME}/.local/share/syncode"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
CYAN='\033[0;36m'
NC='\033[0m'

info()  { echo -e "${CYAN}[info]${NC} $*"; }
ok()    { echo -e "${GREEN}[ok]${NC} $*"; }
warn()  { echo -e "${YELLOW}[warn]${NC} $*"; }
error() { echo -e "${RED}[error]${NC} $*" >&2; }

remove_binary() {
    local removed=false

    # Check common install locations
    for dir in /usr/local/bin ~/.local/bin ~/.cargo/bin; do
        local path="${dir}/${BINARY}"
        if [ -f "$path" ]; then
            info "Removing ${path}..."
            if [ -w "$dir" ]; then
                rm -f "$path"
            else
                sudo rm -f "$path"
            fi
            removed=true
            ok "Removed ${path}"
        fi
    done

    # Check if still in PATH
    if command -v "$BINARY" >/dev/null 2>&1; then
        local which_path
        which_path=$(command -v "$BINARY")
        info "Removing ${which_path}..."
        rm -f "$which_path"
        removed=true
        ok "Removed ${which_path}"
    fi

    if [ "$removed" = false ]; then
        warn "Binary not found in common locations"
    fi
}

remove_config() {
    if [ -d "$CONFIG_DIR" ]; then
        info "Removing config directory: ${CONFIG_DIR}"
        rm -rf "$CONFIG_DIR"
        ok "Removed config directory"
    else
        info "Config directory not found: ${CONFIG_DIR}"
    fi
}

remove_cache() {
    if [ -d "$CACHE_DIR" ]; then
        info "Removing cache directory: ${CACHE_DIR}"
        rm -rf "$CACHE_DIR"
        ok "Removed cache directory"
    else
        info "Cache directory not found: ${CACHE_DIR}"
    fi
}

remove_data() {
    if [ -d "$DATA_DIR" ]; then
        info "Removing data directory: ${DATA_DIR}"
        rm -rf "$DATA_DIR"
        ok "Removed data directory"
    else
        info "Data directory not found: ${DATA_DIR}"
    fi
}

print_summary() {
    echo ""
    echo "  ╔═══════════════════════════════════════╗"
    echo "  ║     Syncode 卸载完成                   ║"
    echo "  ╚═══════════════════════════════════════╝"
    echo ""
    echo "  已清理："
    echo "    • 二进制文件"
    echo "    • 配置目录 (${CONFIG_DIR})"
    echo "    • 缓存目录 (${CACHE_DIR})"
    echo "    • 数据目录 (${DATA_DIR})"
    echo ""
    echo "  如需重新安装："
    echo "    curl -fsSL https://gitee.com/Agions/syncode/raw/main/install.sh | bash"
    echo ""
}

main() {
    echo ""
    echo "  ╔═══════════════════════════════════════╗"
    echo "  ║     Syncode — AI 配对编程终端卸载器     ║"
    echo "  ╚═══════════════════════════════════════╝"
    echo ""

    # Parse arguments
    local remove_all=false
    local remove_config_only=false

    for arg in "$@"; do
        case "$arg" in
            --all)
                remove_all=true
                ;;
            --config-only)
                remove_config_only=true
                ;;
            --help|-h)
                echo "Usage: uninstall.sh [OPTIONS]"
                echo ""
                echo "Options:"
                echo "  --all          Remove binary + config + cache + data"
                echo "  --config-only  Remove only config directory"
                echo "  --help         Show this help"
                echo ""
                echo "Default: Remove binary only, keep config and data"
                exit 0
                ;;
        esac
    done

    if [ "$remove_config_only" = true ]; then
        info "Removing config only..."
        remove_config
        ok "Config removed. Binary and data preserved."
        exit 0
    fi

    if [ "$remove_all" = true ]; then
        info "Removing everything..."
        remove_binary
        remove_config
        remove_cache
        remove_data
        print_summary
    else
        info "Removing binary only (use --all to remove config and data)..."
        remove_binary
        echo ""
        ok "Binary removed."
        echo ""
        echo "  Config and data preserved at:"
        echo "    • ${CONFIG_DIR}"
        echo "    • ${DATA_DIR}"
        echo ""
        echo "  To remove everything:"
        echo "    curl -fsSL https://gitee.com/Agions/syncode/raw/main/uninstall.sh | bash -s -- --all"
        echo ""
    fi
}

main "$@"
