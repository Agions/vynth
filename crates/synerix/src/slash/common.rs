//! 共享工具函数 — 所有斜杠命令的公共基础设施
//!
//! # 职责
//! - 系统消息推送（sys_msg）
//! - 参数解析辅助（subcmd, nth_arg, rest_from）
//! - 格式化函数（provider_display, format_mcp_*）
//! - 常量定义（DEFAULT_MODEL）

use crate::app::{App, ChatMessage, MessageRole};
use crate::config::{McpServerConfig, McpTransport, Provider};

/// 默认 LLM 模型名
pub const DEFAULT_MODEL: &str = "deepseek-v4-flash";

/// 向 chat_state 追加一条系统消息
///
/// # 优化说明
/// 提取为独立函数：将「构造 ChatMessage → push 到 messages」的统一模式
/// 集中到一处，所有 handler 只需调用 `sys_msg(app, text)`，消除重复内联构造。
pub fn sys_msg(app: &mut App, text: &str) {
    app.chat_state.messages.push(ChatMessage {
        role: MessageRole::System,
        content: text.to_string(),
        tool_calls: Vec::new(),
    });
}

/// 分割参数为 (子命令, 剩余参数)
///
/// 示例：`"source add git url"` → `("source", Some("add git url"))`
pub fn subcmd(args: Option<&str>) -> (&str, Option<&str>) {
    // 需要返回 &str 而非 &'static str，但实际我们只跟字面量比较
    // 所以返回 &str 即可，这里为了兼容风格用 static 声明
    match args {
        None | Some("") => ("", None),
        Some(a) => match a.trim().split_once(' ') {
            Some((s, r)) => (s, Some(r.trim())),
            None => (a.trim(), None),
        },
    }
}

/// 获取第 N 个位置参数（0-indexed）
///
/// # 优化说明
/// 与 subcmd 配合使用，消除 handler 中反复出现的
/// `let parts: Vec<&str> = ...; parts.get(n)` 重复模式。
pub fn nth_arg(args: Option<&str>, n: usize) -> Option<&str> {
    args?.split_whitespace().nth(n)
}

/// 格式化提供商信息显示
pub fn provider_display(provider: &Provider) -> String {
    match provider {
        Provider::DeepSeek => "DeepSeek（默认）".to_string(),
        Provider::MiMo => "MiMo".to_string(),
        Provider::Custom { base_url } => format!("自定义 (`{}`)", base_url),
    }
}

/// 格式化单个 MCP 服务器详情
pub fn format_mcp_server_detail(idx: usize, server: &McpServerConfig) -> String {
    let transport_desc = match &server.transport {
        McpTransport::Stdio { command, args } => {
            let args_str = if args.is_empty() {
                String::new()
            } else {
                format!(" {}", args.join(" "))
            };
            format!("stdio: `{}{}`", command, args_str)
        }
        McpTransport::Http { url } => format!("http: `{}`", url),
    };
    let tools = if server.allowed_tools.is_empty() {
        "（全部）".to_string()
    } else {
        server.allowed_tools.join(", ")
    };
    format!(
        "{}. `{}` — {}\n   工具: {} | 超时: {}s | 重连: {}",
        idx + 1,
        server.name,
        transport_desc,
        tools,
        server.timeout_secs,
        if server.auto_reconnect { "开" } else { "关" }
    )
}

/// MCP 命令用法说明（静态文本）
pub fn format_mcp_usage() -> &'static str {
    "用法：\n  `/mcp` — 列出所有\n  `/mcp list` — 列出所有\n  `/mcp show <name>` — 查看详情\n  `/mcp add <name> stdio <command> [args...]` — 添加 stdio 服务器\n  `/mcp add <name> http <url>` — 添加 HTTP 服务器\n  `/mcp remove <name>` — 删除服务器"
}

/// 检查 MCP 服务器名是否已存在
///
/// # 优化说明
/// 提取重复的「名称冲突检测」逻辑，消除 /mcp add 中
/// 的内联 `find` + `sys_msg` 重复模式。
pub fn mcp_add_duplicate_check(app: &mut App, name: &str) -> bool {
    if app.settings.mcp.iter().any(|s| s.name == name) {
        sys_msg(
            app,
            &format!(
                "❌ MCP 服务器 `{}` 已存在。请先 `/mcp remove {}`",
                name, name
            ),
        );
        true
    } else {
        false
    }
}
