#!/usr/bin/env bash
set -euo pipefail

REPO_URL="https://github.com/Agions/zeno.git"
BIN_NAME="zeno"
PREFIX=""
NO_BUILD=0
UNINSTALL=0

info()  { printf '\033[36m›\033[0m %s\n' "$*"; }
ok()    { printf '\033[32m✓\033[0m %s\n' "$*"; }
warn()  { printf '\033[33m⚠\033[0m %s\n' "$*"; }
fail()  { printf '\033[31m✗\033[0m %s\n' "$*" >&2; exit 1; }

while [ $# -gt 0 ]; do
  case "$1" in
    --prefix)    PREFIX="${2:-}"; [ -n "$PREFIX" ] || fail "--prefix 缺少目录参数"; shift 2 ;;
    --no-build)  NO_BUILD=1; shift ;;
    --uninstall) UNINSTALL=1; shift ;;
    -h|--help)
      sed -n '3,11p' "$0" | sed 's/^# \{0,1\}//'
      exit 0 ;;
    *) fail "未知参数: $1（--help 查看用法）" ;;
  esac
done

resolve_prefix() {
  if [ -n "$PREFIX" ]; then echo "$PREFIX"; return; fi
  if [ -d "$HOME/.local/bin" ] || mkdir -p "$HOME/.local/bin" 2>/dev/null; then
    echo "$HOME/.local/bin"; return
  fi
  echo "/usr/local/bin"
}
INSTALL_DIR="$(resolve_prefix)"

if [ "$UNINSTALL" = 1 ]; then
  removed=0
  for d in "$INSTALL_DIR" "$HOME/.local/bin" /usr/local/bin; do
    if [ -f "$d/$BIN_NAME" ]; then
      rm -f "$d/$BIN_NAME" && ok "已删除 $d/$BIN_NAME" && removed=1
    fi
  done
  [ "$removed" = 1 ] || warn "未找到已安装的 $BIN_NAME"
  info "配置目录 ~/.zeno 未删除（含 config.json/审计日志），如需彻底清理请手动删除"
  exit 0
fi

echo ""
echo "  ⚡ Zeno Installer — Terminal-first AI coding agent"
echo "  ─────────────────────────────────────────────────"
echo ""

if ! command -v bun >/dev/null 2>&1; then
  warn "未检测到 Bun，正在自动安装（https://bun.sh）..."
  curl -fsSL https://bun.sh/install | bash || fail "Bun 自动安装失败，请手动安装后重试"
  export BUN_INSTALL="${BUN_INSTALL:-$HOME/.bun}"
  export PATH="$BUN_INSTALL/bin:$PATH"
  command -v bun >/dev/null 2>&1 || fail "Bun 安装后仍不可用，请重开终端后重试"
fi
ok "Bun $(bun --version)"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]:-$0}")" 2>/dev/null && pwd || true)"
if [ -n "$SCRIPT_DIR" ] && [ -f "$SCRIPT_DIR/../package.json" ] && grep -q '"name": "zeno"' "$SCRIPT_DIR/../package.json"; then
  SRC_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
  info "使用当前仓库: $SRC_DIR"
else
  command -v git >/dev/null 2>&1 || fail "需要 git 来获取源码，请先安装 git"
  SRC_DIR="$(mktemp -d)/zeno"
  info "克隆源码到临时目录..."
  git clone --depth 1 "$REPO_URL" "$SRC_DIR" || fail "克隆失败: $REPO_URL"
fi
cd "$SRC_DIR"

if [ "$NO_BUILD" = 1 ] && [ -f "dist/$BIN_NAME" ]; then
  info "跳过编译，使用现有 dist/$BIN_NAME"
else
  info "安装依赖..."
  bun install --silent || bun install
  info "编译单二进制（bun build --compile）..."
  bun run compile
fi
[ -f "dist/$BIN_NAME" ] || fail "编译产物 dist/$BIN_NAME 不存在"

if [ -w "$INSTALL_DIR" ] || mkdir -p "$INSTALL_DIR" 2>/dev/null; then
  install -m 755 "dist/$BIN_NAME" "$INSTALL_DIR/$BIN_NAME"
else
  warn "$INSTALL_DIR 不可写，尝试 sudo..."
  sudo install -m 755 "dist/$BIN_NAME" "$INSTALL_DIR/$BIN_NAME" || fail "安装失败：无法写入 $INSTALL_DIR"
fi
ok "已安装: $INSTALL_DIR/$BIN_NAME ($(du -h "$INSTALL_DIR/$BIN_NAME" | cut -f1 | tr -d ' '))"

case ":$PATH:" in
  *":$INSTALL_DIR:"*) ;;
  *)
    warn "$INSTALL_DIR 不在 PATH 中，请追加到 shell 配置："
    echo ""
    echo "    echo 'export PATH=\"$INSTALL_DIR:\$PATH\"' >> ~/.$(basename "${SHELL:-zsh}")rc"
    echo ""
    ;;
esac

echo ""
echo "  🎉 安装完成！三步开跑："
echo ""
echo "    export ZENO_API_KEY=\"sk-...\"        # 1. 设置 LLM API Key"
echo "    $BIN_NAME                              # 2. 交互式 TUI"
echo "    $BIN_NAME -g '给 src 写单元测试'       # 3. 或无头模式直接干活"
echo ""
echo "  文档: https://github.com/Agions/zeno/tree/main/docs"
echo ""
