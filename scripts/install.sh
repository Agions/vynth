#!/usr/bin/env bash
set -euo pipefail

REPO_URL="https://github.com/Agions/vynth.git"
BIN_NAME="vynth"
PREFIX=""
NO_BUILD=0
UNINSTALL=0

info()  { printf '\033[36m>\033[0m %s\n' "$*"; }
ok()    { printf '\033[32m v\033[0m %s\n' "$*"; }
warn()  { printf '\033[33m !\033[0m %s\n' "$*"; }
fail()  { printf '\033[31m x\033[0m %s\n' "$*" >&2; exit 1; }

while [ $# -gt 0 ]; do
  case "$1" in
    --prefix)    PREFIX="${2:-}"; [ -n "$PREFIX" ] || fail "--prefix needs a directory argument"; shift 2 ;;
    --no-build)  NO_BUILD=1; shift ;;
    --uninstall) UNINSTALL=1; shift ;;
    -h|--help)
      sed -n '3,11p' "$0" | sed 's/^# \{0,1\}//'
      exit 0 ;;
    *) fail "unknown argument: $1 (use --help for usage)" ;;
  esac
done

resolve_prefix() {
  if [ -n "$PREFIX" ]; then echo "$PREFIX"; return; fi
  # prefer a dir that is already on PATH and writable
  for try in /usr/local/bin "$HOME/.local/bin" "$HOME/bin" /opt/homebrew/bin; do
    if [ -d "$try" ] && [ -w "$try" ]; then echo "$try"; return; fi
  done
  # fallback: create ~/.local/bin
  if mkdir -p "$HOME/.local/bin" 2>/dev/null; then
    echo "$HOME/.local/bin"; return
  fi
  echo "/usr/local/bin"
}
INSTALL_DIR="$(resolve_prefix)"

if [ "$UNINSTALL" = 1 ]; then
  removed=0
  for d in "$INSTALL_DIR" "$HOME/.local/bin" /usr/local/bin; do
    if [ -f "$d/$BIN_NAME" ]; then
      rm -f "$d/$BIN_NAME" && ok "removed $d/$BIN_NAME" && removed=1
    fi
  done
  [ "$removed" = 1 ] || warn "$BIN_NAME not found"
  info "config dir ~/.vynth not deleted; remove manually if needed"
  exit 0
fi

echo ""
echo "  vynth -- terminal-first AI coding agent"
echo "  ----------------------------------------"
echo ""

# -- download prebuilt binary for fastest install --
DL_ARCHIVE=""
if [ "$NO_BUILD" != 1 ]; then
  case "$(uname -s)" in
    Darwin) DL_ARCHIVE="vynth"      ;;  # macOS universal binary
    Linux)  DL_ARCHIVE="vynth-linux" ;;
  esac
  if [ -n "$DL_ARCHIVE" ]; then
    LATEST_TAG="$(curl -fsS "https://api.github.com/repos/Agions/vynth/releases/latest" 2>/dev/null \
      | grep '"tag_name"' | head -1 | sed 's/.*"tag_name": *"\([^"]*\)".*/\1/' || true)"
    if [ -n "$LATEST_TAG" ]; then
      DL_URL="https://github.com/Agions/vynth/releases/download/${LATEST_TAG}/${DL_ARCHIVE}"
      info "downloading prebuilt $BIN_NAME ($LATEST_TAG)..."
      TMP_BIN="$(mktemp -d)/$BIN_NAME"
      if curl -fsSL "$DL_URL" -o "$TMP_BIN" 2>/dev/null; then
        chmod +x "$TMP_BIN"
        # verify sha256 if provided
        SHA_URL="https://github.com/Agions/vynth/releases/download/${LATEST_TAG}/vynth.sha256"
        if curl -fsSL "$SHA_URL" -o /tmp/vynth.sha256 2>/dev/null; then
          EXPECT="$(grep "$DL_ARCHIVE" /tmp/vynth.sha256 | awk '{print $1}')"
          ACTUAL="$(shasum -a 256 "$TMP_BIN" | awk '{print $1}')"
          if [ "$EXPECT" = "$ACTUAL" ]; then
            ok "sha256 verified"
          else
            warn "sha256 mismatch (expected $EXPECT, got $ACTUAL); falling back to build from source"
            TMP_BIN=""
          fi
          rm -f /tmp/vynth.sha256
        fi
        if [ -n "$TMP_BIN" ] && [ -s "$TMP_BIN" ]; then
          if [ -w "$INSTALL_DIR" ] || mkdir -p "$INSTALL_DIR" 2>/dev/null; then
            install -m 755 "$TMP_BIN" "$INSTALL_DIR/$BIN_NAME"
          else
            warn "$INSTALL_DIR not writable, trying sudo..."
            sudo install -m 755 "$TMP_BIN" "$INSTALL_DIR/$BIN_NAME" || fail "install failed"
          fi
          ok "installed: $INSTALL_DIR/$BIN_NAME ($(du -h "$INSTALL_DIR/$BIN_NAME" | cut -f1 | tr -d ' '))"
          rm -rf "$(dirname "$TMP_BIN")"
          # ensure PATH
          ensure_path
          verify_install
          exit 0
        fi
        rm -rf "$(dirname "$TMP_BIN")" 2>/dev/null || true
      fi
      warn "download failed; building from source..."
    fi
  fi
fi

# -- build from source --
if ! command -v bun >/dev/null 2>&1; then
  warn "bun not found, installing (https://bun.sh)..."
  curl -fsSL https://bun.sh/install | bash || fail "bun auto-install failed"
  export BUN_INSTALL="${BUN_INSTALL:-$HOME/.bun}"
  export PATH="$BUN_INSTALL/bin:$PATH"
  command -v bun >/dev/null 2>&1 || fail "bun still not available; reopen terminal and retry"
fi
ok "bun $(bun --version)"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]:-$0}")" 2>/dev/null && pwd || true)"
if [ -n "$SCRIPT_DIR" ] && [ -f "$SCRIPT_DIR/../package.json" ] && grep -q '"name": "vynth"' "$SCRIPT_DIR/../package.json" 2>/dev/null; then
  SRC_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
  info "using local repo: $SRC_DIR"
else
  command -v git >/dev/null 2>&1 || fail "git required; install git first"
  SRC_DIR="$(mktemp -d)/vynth"
  info "cloning source..."
  git clone --depth 1 "$REPO_URL" "$SRC_DIR" || fail "clone failed: $REPO_URL"
fi
cd "$SRC_DIR"

if [ "$NO_BUILD" = 1 ] && [ -f "dist/$BIN_NAME" ]; then
  info "skipping build, using existing dist/$BIN_NAME"
else
  info "installing dependencies..."
  bun install --silent || bun install
  info "compiling single binary (bun build --compile)..."
  bun run compile
fi
[ -f "dist/$BIN_NAME" ] || fail "compile output dist/$BIN_NAME missing"

if [ -w "$INSTALL_DIR" ] || mkdir -p "$INSTALL_DIR" 2>/dev/null; then
  install -m 755 "dist/$BIN_NAME" "$INSTALL_DIR/$BIN_NAME"
else
  warn "$INSTALL_DIR not writable, trying sudo..."
  sudo install -m 755 "dist/$BIN_NAME" "$INSTALL_DIR/$BIN_NAME" || fail "install failed: cannot write to $INSTALL_DIR"
fi
ok "installed: $INSTALL_DIR/$BIN_NAME ($(du -h "$INSTALL_DIR/$BIN_NAME" | cut -f1 | tr -d ' '))"

# ensure PATH so vynth is globally available
ensure_path() {
  case ":$PATH:" in
    *":$INSTALL_DIR:"*) return ;;
  esac
  warn "$INSTALL_DIR is not on your PATH; adding to shell config..."
  local rc
  case "$(basename "${SHELL:-/bin/zsh}")" in
    zsh)  rc="$HOME/.zshrc" ;;
    bash) rc="$HOME/.bashrc" ;;
    *)    rc="$HOME/.profile" ;;
  esac
  if ! grep -q "$INSTALL_DIR" "$rc" 2>/dev/null; then
    echo "# vynth" >> "$rc"
    echo "export PATH=\"$INSTALL_DIR:\$PATH\"" >> "$rc"
    ok "added $INSTALL_DIR to $rc (reopen terminal or run: source $rc)"
  fi
  export PATH="$INSTALL_DIR:$PATH"
}
ensure_path

verify_install() {
  if "$INSTALL_DIR/$BIN_NAME" --version >/dev/null 2>&1; then
    local ver
    ver="$("$INSTALL_DIR/$BIN_NAME" --version 2>/dev/null)"
    ok "verified: ${ver:-ok}"
  fi
}
verify_install

echo ""
echo "  install complete! to get started:"
echo ""
echo "    export VYNTH_API_KEY=\"sk-...\"        # 1. set LLM API key"
echo "    $BIN_NAME                              # 2. interactive TUI"
echo "    $BIN_NAME -g 'write unit tests'        # 3. headless mode"
echo ""
echo "  docs: https://github.com/Agions/vynth/tree/main/docs"
echo ""
