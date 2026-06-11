# 配置

Synerix 支持通过配置文件和命令行参数进行自定义。

## 配置文件

配置文件位于 `~/.config/synerix/config.toml`。

### 基本配置

```toml
# ~/.config/synerix/config.toml

# 基本设置
name = "Your Name"
theme = "dark"  # dark, light, auto

# AI 设置
[ai]
provider = "openai"  # openai, anthropic, local
model = "gpt-4"
api_key = "your-api-key"  # 或者使用环境变量 SYNERIX_API_KEY

# 终端设置
[terminal]
font_size = 14
font_family = "JetBrains Mono"
cursor_style = "block"  # block, underline, line

# 快捷键
[keybindings]
# 自定义快捷键
ctrl_shift_r = "reload"
ctrl_shift_p = "palette"
```

### AI 提供商配置

#### OpenAI

```toml
[ai]
provider = "openai"
model = "gpt-4"
api_key = "sk-..."  # 或者使用环境变量 OPENAI_API_KEY
```

#### Anthropic

```toml
[ai]
provider = "anthropic"
model = "claude-3-opus-20240229"
api_key = "sk-ant-..."  # 或者使用环境变量 ANTHROPIC_API_KEY
```

#### 本地模型

```toml
[ai]
provider = "local"
model = "codellama"
endpoint = "http://localhost:11434"
```

## 环境变量

Synerix 支持通过环境变量配置：

| 环境变量 | 说明 | 默认值 |
|----------|------|--------|
| `SYNERIX_API_KEY` | AI API 密钥 | - |
| `SYNERIX_THEME` | 主题 | `auto` |
| `SYNERIX_MODEL` | AI 模型 | `gpt-4` |
| `SYNERIX_PROVIDER` | AI 提供商 | `openai` |

## 配置管理命令

```bash
# 查看当前配置
synerix config show

# 设置配置值
synerix config set theme dark

# 重置配置
synerix config reset

# 验证配置
synerix config validate
```

## 下一步

- [使用模式](/guide/modes) - 了解不同的工作模式
- [故障排除](/guide/troubleshooting) - 常见问题解决方案
