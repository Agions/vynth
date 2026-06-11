# 插件

Synerix 支持通过插件系统扩展功能。

## 插件类型

### 命令插件

添加新的命令到 Synerix。

### 主题插件

自定义 Synerix 的外观。

### 集成插件

连接外部服务和工具。

## 创建插件

### 基本结构

```rust
use synerix::plugin::{Plugin, PluginContext, PluginResult};
use async_trait::async_trait;

pub struct MyPlugin;

#[async_trait]
impl Plugin for MyPlugin {
    fn name(&self) -> &str {
        "my-plugin"
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    fn description(&self) -> &str {
        "My awesome plugin"
    }
    
    async fn initialize(&self, ctx: PluginContext) -> PluginResult {
        println!("Plugin initialized!");
        PluginResult::Success
    }
    
    async fn shutdown(&self) -> PluginResult {
        println!("Plugin shutdown!");
        PluginResult::Success
    }
}
```

### 注册插件

```rust
use synerix::Synerix;

let app = Synerix::new()
    .register_plugin(Box::new(MyPlugin))
    .build()
    .await?;
```

## 插件 API

### PluginContext

提供插件运行时的上下文信息：

```rust
pub struct PluginContext {
    pub config: Config,
    pub event_bus: EventBus,
    pub command_registry: CommandRegistry,
}
```

### 事件系统

插件可以监听和发布事件：

```rust
use synerix::event::{Event, EventHandler};

struct MyEventHandler;

#[async_trait]
impl EventHandler for MyEventHandler {
    async fn handle(&self, event: Event) -> Result<()> {
        match event {
            Event::CommandExecuted(cmd) => {
                println!("Command executed: {}", cmd.name);
            }
            Event::AiResponse(response) => {
                println!("AI response received");
            }
            _ => {}
        }
        Ok(())
    }
}
```

## 插件示例

### Git 集成插件

```rust
pub struct GitPlugin;

#[async_trait]
impl Plugin for GitPlugin {
    fn name(&self) -> &str {
        "git"
    }
    
    async fn initialize(&self, ctx: PluginContext) -> PluginResult {
        // 注册 Git 命令
        ctx.command_registry.register(Box::new(GitCommand));
        PluginResult::Success
    }
}

struct GitCommand;

#[async_trait]
impl Command for GitCommand {
    fn name(&self) -> &str {
        "git"
    }
    
    async fn execute(&self, ctx: CommandContext) -> CommandResult {
        let subcommand = ctx.args().first()
            .ok_or("Please provide a git subcommand")?;
        
        match subcommand.as_str() {
            "status" => {
                // 执行 git status
                CommandResult::Success
            }
            "commit" => {
                // 执行 git commit
                CommandResult::Success
            }
            _ => {
                CommandResult::Error(format!("Unknown git subcommand: {}", subcommand))
            }
        }
    }
}
```

## 插件配置

插件可以有自己的配置：

```toml
# ~/.config/synerix/plugins/my-plugin.toml

[settings]
enabled = true
verbose = false

[options]
timeout = 30
retries = 3
```

## 插件管理

```bash
# 列出已安装插件
synerix plugins list

# 安装插件
synerix plugins install <plugin-name>

# 卸载插件
synerix plugins uninstall <plugin-name>

# 更新插件
synerix plugins update <plugin-name>
```

## 下一步

- [故障排除](/guide/troubleshooting) - 常见问题解决方案
- [贡献指南](/guide/contributing) - 如何贡献代码
