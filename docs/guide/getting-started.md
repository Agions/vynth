# Getting Started

Welcome to **Synerix** — an AI-native coding terminal designed to keep you in flow.

## What is Synerix?

Synerix is a high-performance terminal that combines the power of modern AI with the speed of a native CLI. It thinks, writes, reviews, and fixes code — without pulling you out of the command line.

## Installation

### Linux / macOS

```bash
curl -fsSL https://raw.githubusercontent.com/Agions/synerix/main/install.sh | bash
```

### Windows (PowerShell 5.1+)

```powershell
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
irm https://raw.githubusercontent.com/Agions/synerix/main/install.ps1 | iex
```

### From Source

```bash
git clone https://github.com/Agions/synerix.git
cd synerix
cargo build --release
cargo install --path .
```

## First Launch

After installation, configure your LLM provider:

```bash
# Create config directory
mkdir -p ~/.config/synerix

# Create config file
cat > ~/.config/synerix/config.toml << 'EOF'
[llm]
provider = "deepseek"
api_key = "sk-..."
model = "deepseek-v4-flash"
EOF
```

Then launch:

```bash
synerix
```

## Your First Session

```
❯ Refactor main.rs to reduce nesting
   ✓ Code generated
   ✓ cargo check passes
   ✓ Tests pass
   ✓ Done in 1.8s

❯ /mode vibe
   Switched to Vibe mode — auto-iterate enabled

❯ Add pagination to the API endpoint
   ✓ Generated pagination module
   ✓ Compiled successfully
   ✓ All tests green
```

## Common Commands

| Command | Description |
|---|---|
| `synerix` | Launch the interactive terminal |
| `synerix --help` | Show help information |
| `synerix --version` | Show version |
| `/mode <name>` | Switch coding mode (act, vibe, chat, architect, plan) |
| `/help` | List all slash commands |
| `/clear` | Clear conversation |
| `/exit` | Quit Synerix |

## Keybindings

| Key | Action |
|---|---|
| `Tab` | Auto-complete / switch mode |
| `Ctrl+C` | Cancel current operation |
| `Ctrl+D` | Exit |
| `Ctrl+L` | Clear screen |
| `↑ / ↓` | Navigate history |

## Next Steps

- [Installation](/guide/installation) — Detailed installation guide
- [Configuration](/guide/configuration) — Customize Synerix to your workflow
- [Coding Modes](/guide/modes) — Learn about Act, Vibe, Chat, Architect, and Plan modes
- [Troubleshooting](/guide/troubleshooting) — Common issues and solutions
