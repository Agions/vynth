# API 概览

Synerix 提供丰富的 API 接口，允许你扩展和集成到自己的工具中。

## 核心概念

### 命令系统

Synerix 使用命令系统来处理用户输入。每个命令都有：

- **名称**: 命令的唯一标识符
- **参数**: 命令接受的参数
- **回调**: 命令执行的逻辑

### 事件系统

Synerix 提供事件系统，允许你监听和响应各种事件：

- **输入事件**: 用户输入
- **输出事件**: AI 响应
- **系统事件**: 启动、关闭等

### 插件系统

通过插件系统，你可以扩展 Synerix 的功能：

- **命令插件**: 添加新命令
- **主题插件**: 自定义外观
- **集成插件**: 连接外部服务

## 快速开始

### 基本用法

```rust
use synerix::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化 Synerix
    let app = Synerix::new()
        .with_config(Config::load()?)
        .build()
        .await?;
    
    // 运行应用
    app.run().await?;
    
    Ok(())
}
```

### 创建自定义命令

```rust
use synerix::command::{Command, CommandContext};

struct MyCommand;

#[async_trait]
impl Command for MyCommand {
    fn name(&self) -> &str {
        "my-command"
    }
    
    fn description(&self) -> &str {
        "My custom command"
    }
    
    async fn execute(&self, ctx: CommandContext) -> Result<()> {
        println!("Hello from my command!");
        Ok(())
    }
}
```

## 下一步

- [命令](/api/commands) - 详细了解命令系统
- [配置](/api/config) - 配置选项详解
- [插件](/api/plugins) - 插件开发指南
