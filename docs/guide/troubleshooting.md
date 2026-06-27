# Troubleshooting

Solutions to common problems when using Synerix.

## Installation Issues

### Script fails to download

```bash
# Check network connectivity
curl -I https://github.com

# Try with verbose output
curl -v https://raw.githubusercontent.com/Agions/synerix/main/install.sh

# Fallback: download binary manually
wget https://github.com/Agions/synerix/releases/latest/download/synerix-linux-x86_64.tar.gz
tar -xzf synerix-linux-x86_64.tar.gz
sudo mv synerix /usr/local/bin/
```

### Permission denied

```bash
# Add to PATH
export PATH="$HOME/.local/bin:$PATH"

# Or make executable
chmod +x ~/.local/bin/synerix
```

## Runtime Issues

### App crashes on startup

```bash
# Run with debug logging
synerix --log-level debug

# Reset to default config
synerix config reset

# Check for version conflicts
cargo update
```

### AI connection fails

```bash
# Verify API key is set
synerix config show

# Test connection manually
curl https://api.deepseek.com/v1/models \
  -H "Authorization: Bearer $SYNERIX_API_KEY"

# Check proxy settings if behind a firewall
```

### Slow responses

```bash
# Switch to a faster model
synerix config set llm.model deepseek-v4-flash

# Reduce context window
synerix config set llm.max_tokens 4096
```

## Terminal Issues

### Display corruption

```bash
# Ensure terminal supports 256 colors
export TERM=xterm-256color

# Reset terminal state
reset

# Try a different terminal font
synerix config set ui.font_family "JetBrains Mono"
```

### Input lag

- Disable other shell plugins (zsh-syntax-highlighting, etc.)
- Try a GPU-accelerated terminal (Alacritty, Kitty)

## Getting Help

| Channel | Link |
|---|---|
| GitHub Issues | [github.com/Agions/synerix/issues](https://github.com/Agions/synerix/issues) |
| Discussions | [github.com/Agions/synerix/discussions](https://github.com/Agions/synerix/discussions) |
| Gitee Mirror | [gitee.com/Agions/synerix](https://gitee.com/Agions/synerix) |

## Reporting Bugs

Include the following when filing an issue:

1. `synerix --version`
2. OS and version
3. Terminal emulator and version
4. Error output or log
5. Steps to reproduce
