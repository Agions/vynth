//! `/mcp` 命令 — MCP 服务器管理
//!
//! # 优化说明
//! - 将原 cmd_mcp 的 148 行 match 拆分为 5 个独立函数：
//!   `mcp_handle_list`, `mcp_handle_show`, `mcp_handle_add`,
//!   `mcp_handle_remove`, `mcp_handle_unknown`
//! - /mcp add 的两个分支（stdio / http）进一步提取为
//!   `mcp_add_stdio` 和 `mcp_add_http`，消除 McpServerConfig
//!   构造代码的重复（原文件 L517-528 和 L537-548 几乎完全一样）。

use crate::app::App;
use crate::config::McpServerConfig;
use crate::slash::common::{
    format_mcp_server_detail, format_mcp_usage, mcp_add_duplicate_check, nth_arg, subcmd, sys_msg,
};

/// 处理 `/mcp` 命令
pub fn cmd_mcp(app: &mut App, args: Option<&str>) -> bool {
    let (sub, rest) = subcmd(args);

    match sub {
        "" | "list" => mcp_handle_list(app),
        "show" => mcp_handle_show(app, rest),
        "add" => mcp_handle_add(app, rest),
        "remove" => mcp_handle_remove(app, rest),
        other => mcp_handle_unknown(app, other),
    }
    true
}

/// 列出所有 MCP 服务器
fn mcp_handle_list(app: &mut App) {
    if app.settings.mcp.is_empty() {
        sys_msg(
            app,
            &format!("📋 未配置 MCP 服务器。\n\n{}", format_mcp_usage()),
        );
    } else {
        let mut lines = vec![format!(
            "📋 已配置 {} 个 MCP 服务器：",
            app.settings.mcp.len()
        )];
        for (i, server) in app.settings.mcp.iter().enumerate() {
            lines.push(String::new());
            lines.push(format_mcp_server_detail(i, server));
        }
        sys_msg(app, &lines.join("\n"));
    }
}

/// 查看单个 MCP 服务器详情
fn mcp_handle_show(app: &mut App, rest: Option<&str>) {
    let name = match nth_arg(rest, 0) {
        Some(n) => n,
        None => {
            sys_msg(app, "❌ 用法：`/mcp show <name>`");
            return;
        }
    };
    if let Some(server) = app.settings.mcp.iter().find(|s| s.name == name) {
        let idx = app
            .settings
            .mcp
            .iter()
            .position(|s| s.name == name)
            .unwrap_or(0);
        sys_msg(app, &format_mcp_server_detail(idx, server));
    } else {
        sys_msg(app, &format!("❌ 未找到 MCP 服务器：`{}`", name));
    }
}

/// 添加 MCP 服务器（入口分派）
fn mcp_handle_add(app: &mut App, rest: Option<&str>) {
    let name = match nth_arg(rest, 0) {
        Some(n) => n,
        None => {
            sys_msg(app, &format!("❌ 参数不足。\n\n{}", format_mcp_usage()));
            return;
        }
    };
    let transport_type = match nth_arg(rest, 1) {
        Some(t) => t,
        None => {
            sys_msg(app, &format!("❌ 缺少传输类型。\n\n{}", format_mcp_usage()));
            return;
        }
    };

    if mcp_add_duplicate_check(app, name) {
        return;
    }

    match transport_type {
        "stdio" => mcp_add_stdio(app, name, rest),
        "http" => mcp_add_http(app, name, rest),
        other => {
            sys_msg(
                app,
                &format!("❌ 不支持的传输类型：`{}`。支持：`stdio`、`http`", other),
            );
        }
    }
}

/// 添加 stdio 类型的 MCP 服务器
fn mcp_add_stdio(app: &mut App, name: &str, rest: Option<&str>) {
    let cmd_parts: Vec<&str> = match rest {
        Some(r) => r.split_whitespace().skip(2).collect(),
        None => vec![],
    };
    if cmd_parts.is_empty() {
        sys_msg(app, "❌ 用法：`/mcp add <name> stdio <command> [args...]`");
        return;
    }
    let command = cmd_parts[0].to_string();
    let args_vec: Vec<String> = cmd_parts[1..].iter().map(|s| s.to_string()).collect();

    app.settings.mcp.push(McpServerConfig::stdio(
        name,
        command,
        args_vec,
    ));
    sys_msg(app, &format!("✅ 已添加 MCP 服务器：`{}`（stdio）", name));
}

/// 添加 HTTP 类型的 MCP 服务器
fn mcp_add_http(app: &mut App, name: &str, rest: Option<&str>) {
    let url = nth_arg(rest, 2).unwrap_or("");
    if url.is_empty() {
        sys_msg(app, "❌ 用法：`/mcp add <name> http <url>`");
        return;
    }

    app.settings.mcp.push(McpServerConfig::http(name, url));
    sys_msg(app, &format!("✅ 已添加 MCP 服务器：`{}`（HTTP）", name));
}

/// 移除 MCP 服务器
fn mcp_handle_remove(app: &mut App, rest: Option<&str>) {
    let name = match nth_arg(rest, 0) {
        Some(n) => n,
        None => {
            sys_msg(app, "❌ 用法：`/mcp remove <name>`");
            return;
        }
    };
    let len_before = app.settings.mcp.len();
    app.settings.mcp.retain(|s| s.name != name);
    if app.settings.mcp.len() < len_before {
        sys_msg(app, &format!("🗑️ 已移除 MCP 服务器：`{}`", name));
    } else {
        sys_msg(app, &format!("❌ 未找到 MCP 服务器：`{}`", name));
    }
}

/// 处理未知的 /mcp 子命令
fn mcp_handle_unknown(app: &mut App, other: &str) {
    sys_msg(
        app,
        &format!("❌ 未知子命令 `{}`。\n\n{}", other, format_mcp_usage()),
    );
}
