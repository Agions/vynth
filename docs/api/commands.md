# 命令

Synerix 的命令系统允许你扩展终端功能。

## 内置命令

### 基本命令

| 命令 | 说明 | 用法 |
|------|------|------|
| `help` | 显示帮助信息 | `help [command]` |
| `version` | 显示版本 | `version` |
| `config` | 配置管理 | `config [show\|set\|reset]` |
| `exit` | 退出程序 | `exit` |

### AI 命令

| 命令 | 说明 | 用法 |
|------|------|------|
| `ask` | 询问 AI | `ask <question>` |
| `review` | 代码审查 | `review [file]` |
| `explain` | 代码解释 | `explain [code]` |
| `generate` | 代码生成 | `generate <description>` |

### 工具命令

| 命令 | 说明 | 用法 |
|------|------|------|
| `build` | 构建项目 | `build [--release]` |
| `test` | 运行测试 | `test [pattern]` |
| `run` | 运行程序 | `run [args]` |
| `git` | Git 操作 | `git <command>` |

## 自定义命令

### 创建命令

```rust
use synerix::command::{Command, CommandContext, CommandResult};
use async_trait::async_trait;

pub struct GreetCommand;

#[async_trait]
impl Command for GreetCommand {
    fn name(&self) -> &str {
        "greet"
    }
    
    fn description(&self) -> &str {
        "Greet someone"
    }
    
    fn usage(&self) -> &str {
        "greet <name>"
    }
    
    async fn execute(&self, ctx: CommandContext) -> CommandResult {
        let name = ctx.args().first()
            .ok_or("Please provide a name")?;
        
        println!("Hello, {}!", name);
        CommandResult::Success
    }
}
```

### 注册命令

```rust
use synerix::Synerix;

let app = Synerix::new()
    .register_command(Box::new(GreetCommand))
    .build()
    .await?;
```

## 命令参数

### 位置参数

```rust
async fn execute(&self, ctx: CommandContext) -> CommandResult {
    let first_arg = ctx.args().get(0);
    let second_arg = ctx.args().get(1);
    // ...
}
```

### 可选参数

```rust
async fn execute(&self, ctx: CommandContext) -> CommandResult {
    let verbose = ctx.has_flag("--verbose");
    let output = ctx.get_option("--output");
    // ...
}
```

## 命令历史

Synerix 自动保存命令历史：

```bash
# 查看历史
history

# 搜索历史
history | grep "cargo"

# 清空历史
history --clear
```

## 下一步

- [配置](/api/config) - 配置选项详解
- [插件](/api/plugins) - 插件开发指南
