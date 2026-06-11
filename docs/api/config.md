# 配置

Synerix 提供灵活的配置系统。

## 配置文件

### 位置

- **Linux/macOS**: `~/.config/synerix/config.toml`
- **Windows**: `%APPDATA%\synerix\config.toml`

### 基本结构

```toml
# 基本设置
name = "Your Name"
theme = "dark"
language = "zh-CN"

# AI 设置
[ai]
provider = "openai"
model = "gpt-4"
api_key = "sk-..."

# 终端设置
[terminal]
font_size = 14
font_family = "JetBrains Mono"
cursor_style = "block"

# 行为设置
[behavior]
auto_save = true
auto_compile = true
verbose = false
```

## 配置选项

### 基本设置

| 选项 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `name` | string | - | 用户名 |
| `theme` | string | `auto` | 主题 (dark/light/auto) |
| `language` | string | `en` | 语言 |

### AI 设置

| 选项 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `provider` | string | `openai` | AI 提供商 |
| `model` | string | `gpt-4` | AI 模型 |
| `api_key` | string | - | API 密钥 |
| `max_tokens` | int | 4096 | 最大 token 数 |
| `temperature` | float | 0.7 | 温度参数 |

### 终端设置

| 选项 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `font_size` | int | 14 | 字体大小 |
| `font_family` | string | `monospace` | 字体 |
| `cursor_style` | string | `block` | 光标样式 |
| `scrollback` | int | 10000 | 滚动缓冲区 |

## 环境变量

Synerix 支持环境变量配置：

```bash
# AI 设置
export SYNERIX_API_KEY="sk-..."
export SYNERIX_MODEL="gpt-4"

# 终端设置
export SYNERIX_THEME="dark"
export SYNERIX_FONT_SIZE="14"
```

## 配置命令

```bash
# 查看配置
synerix config show

# 设置配置
synerix config set theme dark
synerix config set ai.model gpt-4

# 重置配置
synerix config reset

# 验证配置
synerix config validate
```

## 配置验证

Synerix 会自动验证配置：

```toml
# 无效配置示例
[ai]
model = 123  # 错误：应该是字符串

# 有效配置示例
[ai]
model = "gpt-4"  # 正确
```

## 下一步

- [插件](/api/plugins) - 插件开发指南
- [故障排除](/guide/troubleshooting) - 常见问题解决方案
