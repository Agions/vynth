//! Slash commands — `/help`, `/clear`, `/model`, `/reset`, `/exit`, `/workflow`
//!
//! Intercepts user input starting with `/` and executes built-in commands
//! instead of sending the message to the AI model.

use crate::app::{App, ChatMessage, GoalState, MessageRole};
use crate::coding_modes::CodingMode;
use crate::config::{McpServerConfig, McpTransport, Provider, SkillSourceConfig};

const DEFAULT_MODEL: &str = "deepseek-v4-flash";

// ── Command framework ──────────────────────────────────────────────────────────

/// Command category for organizing `/help` output
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CmdCategory {
    Help,
    Session,
    Model,
    Config,
    Goal,
    Mode,
    Workflow,
}

impl CmdCategory {
    fn label(self) -> &'static str {
        match self {
            CmdCategory::Help => "💡 帮助",
            CmdCategory::Session => "📋 会话管理",
            CmdCategory::Model => "🤖 模型配置",
            CmdCategory::Config => "⚙️ 配置管理",
            CmdCategory::Goal => "🎯 目标模式",
            CmdCategory::Workflow => "📦 工作流",
            CmdCategory::Mode => "🔀 编码模式",
        }
    }
}

/// Handler type for a slash command
type CmdHandler = fn(&mut App, Option<&str>) -> bool;

/// Structured definition of a slash command
pub struct CmdDef {
    pub name: &'static str,
    pub aliases: &'static [&'static str],
    pub desc: &'static str,
    pub usage: &'static str,
    pub category: CmdCategory,
    pub handler: CmdHandler,
}

/// Registry of all available slash commands
pub const COMMANDS: &[CmdDef] = &[
    CmdDef {
        name: "/help",
        aliases: &["/h", "/?"],
        desc: "显示所有斜杠命令或其详细用法",
        usage: "/help [命令名]",
        category: CmdCategory::Help,
        handler: cmd_help,
    },
    CmdDef {
        name: "/clear",
        aliases: &["/c", "/cls"],
        desc: "清空当前对话",
        usage: "/clear",
        category: CmdCategory::Session,
        handler: cmd_clear,
    },
    CmdDef {
        name: "/model",
        aliases: &["/m"],
        desc: "切换 LLM 模型 / 配置自定义提供商",
        usage: "/model [name] | /model custom <name> <base-url>",
        category: CmdCategory::Model,
        handler: cmd_model,
    },
    CmdDef {
        name: "/reset",
        aliases: &["/re"],
        desc: "重置对话状态（清空 + 恢复默认模型）",
        usage: "/reset",
        category: CmdCategory::Session,
        handler: cmd_reset,
    },
    CmdDef {
        name: "/exit",
        aliases: &["/quit", "/q"],
        desc: "退出 Synerix",
        usage: "/exit",
        category: CmdCategory::Session,
        handler: cmd_exit,
    },
    CmdDef {
        name: "/workflow",
        aliases: &["/wf"],
        desc: "运行工作流",
        usage: "/workflow <name>",
        category: CmdCategory::Workflow,
        handler: cmd_workflow,
    },
    CmdDef {
        name: "/mcp",
        aliases: &[],
        desc: "管理 MCP 服务器（list/show/add/remove）",
        usage: "/mcp [list] | /mcp show <name> | /mcp add <name> stdio|http <args> | /mcp remove <name>",
        category: CmdCategory::Config,
        handler: cmd_mcp,
    },
    CmdDef {
        name: "/skill",
        aliases: &["/skills"],
        desc: "管理技能（dir/source list/add/remove）",
        usage: "/skill [dir <path>] | /skill source [list|add|remove]",
        category: CmdCategory::Config,
        handler: cmd_skill,
    },
    CmdDef {
        name: "/config",
        aliases: &["/cfg"],
        desc: "管理配置（show/save）",
        usage: "/config [show|save]",
        category: CmdCategory::Config,
        handler: cmd_config,
    },
    CmdDef {
        name: "/goal",
        aliases: &["/g"],
        desc: "设置/查看/清除完成目标，Agent 自动循环直到条件满足",
        usage: "/goal [条件] | /goal | /goal clear",
        category: CmdCategory::Goal,
        handler: cmd_goal,
    },
    CmdDef {
        name: "/mode",
        aliases: &["/md"],
        desc: "切换编码模式（Plan/Act/Chat/Architect）",
        usage: "/mode [plan|act|chat|architect] | /mode",
        category: CmdCategory::Mode,
        handler: cmd_mode,
    },
];

/// Find a command definition by name or alias
fn find_cmd(input: &str) -> Option<&'static CmdDef> {
    COMMANDS
        .iter()
        .find(|c| c.name == input || c.aliases.contains(&input))
}

/// Try to handle a slash command. Returns `true` if the input was a slash command.
pub fn try_handle(app: &mut App, input: &str) -> bool {
    let trimmed = input.trim();
    if !trimmed.starts_with('/') {
        return false;
    }

    let (cmd_name, args) = trimmed
        .split_once(' ')
        .map(|(c, a)| (c, Some(a.trim())))
        .unwrap_or((trimmed, None));

    match find_cmd(cmd_name) {
        Some(cmd) => (cmd.handler)(app, args),
        None => {
            sys_msg(
                app,
                &format!("❌ 未知命令 `{}`。输入 `/help` 查看可用命令。", cmd_name),
            );
            true
        }
    }
}

fn sys_msg(app: &mut App, text: &str) {
    app.chat_state.messages.push(ChatMessage {
        role: MessageRole::System,
        content: text.to_string(),
        tool_calls: Vec::new(),
    });
}

// ── Argument parsing utilities ─────────────────────────────────────────────────

/// Split args into (subcommand, remaining).
/// Returns ("", None) when no args given.
fn subcmd(args: Option<&str>) -> (&str, Option<&str>) {
    match args {
        None | Some("") => ("", None),
        Some(a) => match a.trim().split_once(' ') {
            Some((s, r)) => (s, Some(r.trim())),
            None => (a.trim(), None),
        },
    }
}

/// Get the Nth positional argument (0-indexed) from args.
fn nth_arg(args: Option<&str>, n: usize) -> Option<&str> {
    args?.split_whitespace().nth(n)
}

/// Join remaining args from position N onward into a single string.
#[allow(dead_code)]
/// Available for handlers needing rest-of-line capture (e.g. /note content)
fn rest_from(args: Option<&str>, n: usize) -> Option<String> {
    let parts: Vec<&str> = args?.split_whitespace().collect();
    if n < parts.len() {
        Some(parts[n..].join(" "))
    } else {
        None
    }
}

// ── Categorized help system ────────────────────────────────────────────────────

fn cmd_help(app: &mut App, args: Option<&str>) -> bool {
    let (target, _) = subcmd(args);

    if !target.is_empty() {
        // ── /help <command> — show detailed usage ────────────────────────
        let lookup = if target.starts_with('/') {
            target
        } else {
            // Allow /help model (with or without leading /)
            return cmd_help(app, Some(&format!("/{}", target)));
        };

        if let Some(cmd) = find_cmd(lookup) {
            let aliases_str = if cmd.aliases.is_empty() {
                String::new()
            } else {
                format!("\n  别名：{}", cmd.aliases.join(", "))
            };
            sys_msg(
                app,
                &format!(
                    "📖 **{}** — {}{}\n\n用法：`{}`",
                    cmd.name, cmd.desc, aliases_str, cmd.usage,
                ),
            );
        } else {
            sys_msg(
                app,
                &format!("❌ 没有 `{}` 命令。输入 `/help` 查看所有命令。", target),
            );
        }
        return true;
    }

    // ── /help — categorized overview ─────────────────────────────────────
    let cat_order = [
        CmdCategory::Help,
        CmdCategory::Session,
        CmdCategory::Model,
        CmdCategory::Config,
        CmdCategory::Goal,
        CmdCategory::Workflow,
    ];

    let mut lines = vec!["**📋 可用斜杠命令：**".to_string(), String::new()];
    for cat in &cat_order {
        let cmds: Vec<&CmdDef> = COMMANDS.iter().filter(|c| c.category == *cat).collect();
        if cmds.is_empty() {
            continue;
        }
        lines.push(format!("**{}**", cat.label()));
        for cmd in &cmds {
            lines.push(format!("  `{}` — {}", cmd.name, cmd.desc));
        }
        lines.push(String::new());
    }
    lines.push("💡 使用 `/help <命令名>` 查看单个命令的详细用法和别名。".to_string());
    sys_msg(app, &lines.join("\n"));
    true
}

fn cmd_clear(app: &mut App, _args: Option<&str>) -> bool {
    app.chat_state.messages.clear();
    app.chat_state.streaming_text.clear();
    app.chat_state.scroll_offset = 0;
    sys_msg(app, "✅ 对话已清空。");
    true
}

fn provider_display(provider: &Provider) -> String {
    match provider {
        Provider::DeepSeek => "DeepSeek（默认）".to_string(),
        Provider::MiMo => "MiMo".to_string(),
        Provider::Custom { base_url } => format!("自定义 (`{}`)", base_url),
    }
}

fn cmd_model(app: &mut App, args: Option<&str>) -> bool {
    match args {
        None => {
            let provider_str = provider_display(&app.settings.llm.provider);
            sys_msg(
                app,
                &format!(
                    "当前模型：`{}`\n提供商：{}\n\n用法：\n  `/model <name>` — 切换模型名称\n  `/model custom <name> <base-url>` — 配置自定义模型",
                    app.settings.llm.model, provider_str
                ),
            );
        }
        Some("") => {
            sys_msg(
                app,
                "❌ 请指定参数。用法：`/model <name>` 或 `/model custom <name> <base-url>`",
            );
        }
        Some(args) => {
            let trimmed = args.trim();
            if trimmed == "custom" {
                sys_msg(
                    app,
                    "❌ 用法：`/model custom <model-name> <base-url>`\n例如：`/model custom gpt-4o https://api.openai.com/v1`",
                );
            } else if let Some(rest) = trimmed.strip_prefix("custom ") {
                let rest = rest.trim();
                let parts: Vec<&str> = rest.splitn(2, ' ').collect();
                if parts.len() < 2 || parts[0].is_empty() || parts[1].is_empty() {
                    sys_msg(
                        app,
                        "❌ 用法：`/model custom <model-name> <base-url>`\n例如：`/model custom gpt-4o https://api.openai.com/v1`",
                    );
                    return true;
                }
                let model_name = parts[0].trim();
                let base_url = parts[1].trim();
                app.settings.llm.provider = Provider::Custom {
                    base_url: base_url.to_string(),
                };
                app.settings.llm.model = model_name.to_string();
                app.status_bar.model_name = model_name.to_string();
                let provider_str = provider_display(&app.settings.llm.provider);
                sys_msg(
                    app,
                    &format!(
                        "✅ 已配置自定义模型：\n  模型：`{}`\n  提供商：{}\n  API Base URL：`{}`",
                        model_name, provider_str, base_url
                    ),
                );
            } else {
                let old = std::mem::replace(&mut app.settings.llm.model, trimmed.to_string());
                app.status_bar.model_name = trimmed.to_string();
                sys_msg(app, &format!("✅ 模型已切换：`{}` → `{}`", old, trimmed));
            }
        }
    }
    true
}

fn cmd_reset(app: &mut App, _args: Option<&str>) -> bool {
    app.chat_state.messages.clear();
    app.chat_state.streaming_text.clear();
    app.chat_state.scroll_offset = 0;
    app.settings.llm.model = DEFAULT_MODEL.to_string();
    app.status_bar.model_name = DEFAULT_MODEL.to_string();
    sys_msg(
        app,
        &format!("🔄 对话已重置，模型恢复为 `{}`。", DEFAULT_MODEL),
    );
    true
}

fn cmd_exit(app: &mut App, _args: Option<&str>) -> bool {
    app.should_quit = true;
    true
}

fn cmd_workflow(app: &mut App, args: Option<&str>) -> bool {
    match args {
        None => {
            sys_msg(
                app,
                "可用工作流：`code-review`、`refactor`、`debug`\n\n用法：`/workflow <name>`",
            );
        }
        Some("") => {
            sys_msg(app, "❌ 请指定工作流名称。用法：`/workflow <name>`");
        }
        Some(name) => {
            sys_msg(app, &format!("🚀 启动工作流：`{}`（功能开发中）", name));
        }
    }
    true
}

fn format_mcp_server_detail(idx: usize, server: &McpServerConfig) -> String {
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

fn format_mcp_usage() -> &'static str {
    "用法：\n  `/mcp` — 列出所有\n  `/mcp list` — 列出所有\n  `/mcp show <name>` — 查看详情\n  `/mcp add <name> stdio <command> [args...]` — 添加 stdio 服务器\n  `/mcp add <name> http <url>` — 添加 HTTP 服务器\n  `/mcp remove <name>` — 删除服务器"
}

fn mcp_add_duplicate_check(app: &mut App, name: &str) -> bool {
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

fn cmd_mcp(app: &mut App, args: Option<&str>) -> bool {
    let (sub, rest) = subcmd(args);

    match sub {
        // 无参数或 `list` — 列出所有
        "" | "list" => {
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

        // `show <name>` — 查看详情
        "show" => {
            let name = match nth_arg(rest, 0) {
                Some(n) => n,
                None => {
                    sys_msg(app, "❌ 用法：`/mcp show <name>`");
                    return true;
                }
            };
            if let Some(server) = app.settings.mcp.iter().find(|s| s.name == name) {
                let idx = app
                    .settings
                    .mcp
                    .iter()
                    .position(|s| s.name == name)
                    .unwrap();
                sys_msg(app, &format_mcp_server_detail(idx, server));
            } else {
                sys_msg(app, &format!("❌ 未找到 MCP 服务器：`{}`", name));
            }
        }

        // `add <name> stdio <command> [args...]` 或 `add <name> http <url>`
        "add" => {
            let name = match nth_arg(rest, 0) {
                Some(n) => n,
                None => {
                    sys_msg(app, &format!("❌ 参数不足。\n\n{}", format_mcp_usage()));
                    return true;
                }
            };
            let transport_type = match nth_arg(rest, 1) {
                Some(t) => t,
                None => {
                    sys_msg(app, &format!("❌ 缺少传输类型。\n\n{}", format_mcp_usage()));
                    return true;
                }
            };

            if mcp_add_duplicate_check(app, name) {
                return true;
            }

            match transport_type {
                "stdio" => {
                    let cmd_parts: Vec<&str> = match rest {
                        Some(r) => r.split_whitespace().skip(2).collect(),
                        None => vec![],
                    };
                    if cmd_parts.is_empty() {
                        sys_msg(app, "❌ 用法：`/mcp add <name> stdio <command> [args...]`");
                        return true;
                    }
                    let command = cmd_parts[0].to_string();
                    let args_vec: Vec<String> =
                        cmd_parts[1..].iter().map(|s| s.to_string()).collect();
                    app.settings.mcp.push(McpServerConfig {
                        name: name.to_string(),
                        transport: McpTransport::Stdio {
                            command,
                            args: args_vec,
                        },
                        allowed_tools: Vec::new(),
                        env: std::collections::HashMap::new(),
                        cwd: None,
                        auto_reconnect: true,
                        timeout_secs: 30,
                    });
                    sys_msg(app, &format!("✅ 已添加 MCP 服务器：`{}`（stdio）", name));
                }
                "http" => {
                    let url = nth_arg(rest, 2).unwrap_or("");
                    if url.is_empty() {
                        sys_msg(app, "❌ 用法：`/mcp add <name> http <url>`");
                        return true;
                    }
                    app.settings.mcp.push(McpServerConfig {
                        name: name.to_string(),
                        transport: McpTransport::Http {
                            url: url.to_string(),
                        },
                        allowed_tools: Vec::new(),
                        env: std::collections::HashMap::new(),
                        cwd: None,
                        auto_reconnect: true,
                        timeout_secs: 30,
                    });
                    sys_msg(app, &format!("✅ 已添加 MCP 服务器：`{}`（HTTP）", name));
                }
                other => {
                    sys_msg(
                        app,
                        &format!("❌ 不支持的传输类型：`{}`。支持：`stdio`、`http`", other),
                    );
                }
            }
        }

        // `remove <name>`
        "remove" => {
            let name = match nth_arg(rest, 0) {
                Some(n) => n,
                None => {
                    sys_msg(app, "❌ 用法：`/mcp remove <name>`");
                    return true;
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

        other => {
            sys_msg(
                app,
                &format!("❌ 未知子命令 `{}`。\n\n{}", other, format_mcp_usage()),
            );
        }
    }
    true
}

fn cmd_skill(app: &mut App, args: Option<&str>) -> bool {
    let (sub, rest) = subcmd(args);

    match sub {
        // 无参数 — 显示状态
        "" => {
            let dir_info = match &app.settings.skills_dir {
                Some(path) => format!("`{}`", path.display()),
                None => "未设置（使用默认路径）".to_string(),
            };
            let mut lines = vec![format!("📂 Skills 目录：{}", dir_info), String::new()];

            if app.settings.skill_sources.is_empty() {
                lines.push("技能源：无".to_string());
            } else {
                lines.push(format!(
                    "技能源（{} 个）：",
                    app.settings.skill_sources.len()
                ));
                for (i, src) in app.settings.skill_sources.iter().enumerate() {
                    let branch = src
                        .branch
                        .as_ref()
                        .map(|b| format!(" @{}", b))
                        .unwrap_or_default();
                    lines.push(format!(
                        "  {}. [{}] {}{}",
                        i + 1,
                        src.source_type,
                        src.location,
                        branch
                    ));
                }
            }
            lines.push(String::new());
            lines.push("用法：".to_string());
            lines.push("  `/skill` — 查看当前状态".to_string());
            lines.push("  `/skill dir <path>` — 设置 Skills 目录".to_string());
            lines.push("  `/skill source list` — 列出技能源".to_string());
            lines.push("  `/skill source add <type> <location> [branch]` — 添加技能源".to_string());
            lines.push("  `/skill source remove <index>` — 移除技能源".to_string());
            sys_msg(app, &lines.join("\n"));
        }

        // `dir <path>`
        "dir" => {
            let path = match nth_arg(rest, 0) {
                Some(p) => p,
                None => {
                    sys_msg(app, "❌ 用法：`/skill dir <path>`");
                    return true;
                }
            };
            app.settings.skills_dir = Some(std::path::PathBuf::from(path));
            sys_msg(app, &format!("📂 Skills 目录已设置为：`{}`", path));
        }

        // `source <subcommand> [args...]`
        "source" => {
            return cmd_skill_source(app, rest);
        }

        other => {
            sys_msg(
                app,
                &format!("❌ 未知子命令 `{}`。输入 `/skill` 查看用法。", other),
            );
        }
    }
    true
}

fn cmd_skill_source(app: &mut App, args: Option<&str>) -> bool {
    let (sub, rest) = subcmd(args);

    match sub {
        // `source list`
        "" | "list" => {
            let dir_info = match &app.settings.skills_dir {
                Some(path) => format!("`{}`", path.display()),
                None => "未设置".to_string(),
            };
            let mut lines = vec![format!("📂 Skills 目录：{}", dir_info), String::new()];
            if app.settings.skill_sources.is_empty() {
                lines.push("技能源：无".to_string());
            } else {
                lines.push(format!(
                    "技能源（{} 个）：",
                    app.settings.skill_sources.len()
                ));
                for (i, src) in app.settings.skill_sources.iter().enumerate() {
                    let branch = src
                        .branch
                        .as_ref()
                        .map(|b| format!(" @{}", b))
                        .unwrap_or_default();
                    lines.push(format!(
                        "  {}. [{}] {}{}",
                        i + 1,
                        src.source_type,
                        src.location,
                        branch
                    ));
                }
            }
            sys_msg(app, &lines.join("\n"));
        }

        // `source add <type> <location> [branch]`
        "add" => {
            let source_type = match nth_arg(rest, 0) {
                Some(t) => t,
                None => {
                    sys_msg(app, "❌ 用法：`/skill source add <type> <location> [branch]`\n类型：`git`、`local`、`url`\n例如：`/skill source add git https://github.com/user/skills main`");
                    return true;
                }
            };
            let location = match nth_arg(rest, 1) {
                Some(l) => l.to_string(),
                None => {
                    sys_msg(app, "❌ 用法：`/skill source add <type> <location> [branch]`\n类型：`git`、`local`、`url`\n例如：`/skill source add git https://github.com/user/skills main`");
                    return true;
                }
            };
            let branch = nth_arg(rest, 2)
                .map(|s| s.to_string())
                .filter(|s| !s.is_empty());

            let branch_info = branch
                .as_ref()
                .map(|b| format!(" @{}", b))
                .unwrap_or_default();
            sys_msg(
                app,
                &format!(
                    "✅ 已添加技能源：`[{}] {}{}`",
                    source_type, location, branch_info
                ),
            );

            app.settings.skill_sources.push(SkillSourceConfig {
                source_type: source_type.to_string(),
                location,
                branch,
                include: Vec::new(),
                exclude: Vec::new(),
            });
        }

        // `source remove <index>`
        "remove" => {
            let idx_str = match nth_arg(rest, 0) {
                Some(s) => s,
                None => {
                    sys_msg(app, "❌ 用法：`/skill source remove <index>`");
                    return true;
                }
            };
            match idx_str.parse::<usize>() {
                Ok(idx) if idx > 0 && idx <= app.settings.skill_sources.len() => {
                    let removed = app.settings.skill_sources.remove(idx - 1);
                    sys_msg(
                        app,
                        &format!(
                            "🗑️ 已移除技能源 #{}：`[{}] {}`",
                            idx, removed.source_type, removed.location
                        ),
                    );
                }
                Ok(_) | Err(_) => {
                    sys_msg(
                        app,
                        &format!(
                            "❌ 无效索引。有效范围：1~{}",
                            app.settings.skill_sources.len()
                        ),
                    );
                }
            }
        }

        other => {
            sys_msg(app, &format!("❌ 未知子命令 `{}`。用法：`/skill source list`、`/skill source add <type> <location>`、`/skill source remove <index>`", other));
        }
    }
    true
}

/// Handle `/goal` — set, check, or clear a completion condition
fn cmd_goal(app: &mut App, args: Option<&str>) -> bool {
    let args = args.unwrap_or("").trim();

    // ── /goal clear — clear active goal ──────────────────────
    if args.eq_ignore_ascii_case("clear")
        || args.eq_ignore_ascii_case("stop")
        || args.eq_ignore_ascii_case("off")
        || args.eq_ignore_ascii_case("reset")
        || args.eq_ignore_ascii_case("none")
        || args.eq_ignore_ascii_case("cancel")
    {
        if app.goal_state.is_active() {
            app.goal_state = GoalState::inactive();
            app.status_bar.goal_active = false;
            app.status_bar.goal_duration = String::new();
            app.dirty_flags.status = true;
            sys_msg(app, "◎ /goal 已清除");
        } else {
            sys_msg(app, "当前没有活跃的 /goal");
        }
        return true;
    }

    // ── /goal (no args) — show status ────────────────────────
    if args.is_empty() {
        let gs = &app.goal_state;
        if gs.is_active() {
            sys_msg(
                app,
                &format!(
                    "◎ /goal 活跃中\n  条件: {}\n  已运行: {} | 轮次: {} | 最近评估: {}",
                    gs.condition.as_deref().unwrap_or(""),
                    gs.duration_str(),
                    gs.turns,
                    if gs.last_reason.is_empty() {
                        "等待首次评估…"
                    } else {
                        gs.last_reason.as_str()
                    },
                ),
            );
        } else if gs.achieved {
            sys_msg(
                app,
                &format!(
                    "✅ /goal 已完成\n  条件: {}\n  轮次: {}",
                    gs.condition.as_deref().unwrap_or(""),
                    gs.turns,
                ),
            );
        } else {
            sys_msg(
                app,
                "当前没有活跃的 /goal。使用 `/goal <条件>` 设置一个完成目标。\n  示例: `/goal all tests in test/auth pass`",
            );
        }
        return true;
    }

    // ── /goal <condition> — set new goal ──────────────────────
    let condition = args.to_string();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    app.goal_state = GoalState {
        condition: Some(condition.clone()),
        turns: 0,
        started_at: Some(now),
        last_reason: String::new(),
        achieved: false,
    };
    app.status_bar.goal_active = true;
    app.status_bar.goal_duration = String::new();
    app.dirty_flags.status = true;

    // Inject the goal as a user message to trigger the agent loop
    app.chat_state.messages.push(ChatMessage {
        role: MessageRole::User,
        content: format!(
            "🎯 目标: {}\n\n请持续工作直到上述条件满足。每完成一轮，我会检查条件并决定是否继续。",
            condition,
        ),
        tool_calls: Vec::new(),
    });

    sys_msg(
        app,
        &format!(
            "◎ /goal 已设置: {}\nAgent 将持续工作直到条件满足。使用 `/goal` 查看状态，`/goal clear` 清除。",
            condition,
        ),
    );
    true
}

/// Handle `/mode` — switch or display coding mode
fn cmd_mode(app: &mut App, args: Option<&str>) -> bool {
    let args = args.map(|a| a.trim()).unwrap_or("");

    if args.is_empty() {
        // Show current mode + all available modes
        let mut lines = vec![format!("当前编码模式：{}", app.coding_mode)];
        lines.push("".to_string());
        for mode in CodingMode::all() {
            let mark = if *mode == app.coding_mode {
                " ✅ "
            } else {
                "   "
            };
            lines.push(format!(
                "{} {} — {}",
                mark,
                mode.label(),
                mode.description()
            ));
        }
        sys_msg(app, &lines.join("\n"));
        return true;
    }

    let (subcmd, _) = subcmd(Some(args));
    if let Some(new_mode) = CodingMode::parse(subcmd) {
        app.coding_mode = new_mode;
        app.status_bar.coding_mode = new_mode;
        app.dirty_flags.status = true;
        sys_msg(
            app,
            &format!(
                "✅ 编码模式已切换为：{} — {}",
                new_mode.label(),
                new_mode.description()
            ),
        );
    } else {
        sys_msg(
            app,
            &format!(
                "❌ 未知模式 `{}`。可用模式：plan, act, chat, architect",
                subcmd
            ),
        );
    }
    true
}

fn cmd_config(app: &mut App, args: Option<&str>) -> bool {
    let (sub, _rest) = subcmd(args);

    match sub {
        "" | "show" => {
            let path = crate::config::settings::Settings::config_path();
            sys_msg(
                app,
                &format!(
                    "⚙️ 配置路径：`{}`\nMCP 服务器：{} 个\n技能源：{} 个\nSkills 目录：{}\n\n用法：\n  `/config show` — 显示配置路径\n  `/config save` — 保存当前配置到文件",
                    path.display(),
                    app.settings.mcp.len(),
                    app.settings.skill_sources.len(),
                    app.settings.skills_dir.as_ref().map(|p| p.display().to_string()).unwrap_or_else(|| "未设置".to_string())
                ),
            );
        }

        "save" => match app.settings.save() {
            Ok(()) => {
                let path = crate::config::settings::Settings::config_path();
                sys_msg(app, &format!("💾 配置已保存到：`{}`", path.display()));
            }
            Err(e) => {
                sys_msg(app, &format!("❌ 保存配置失败：{}", e));
            }
        },

        other => {
            sys_msg(
                app,
                &format!(
                    "❌ 未知子命令 `{}`。用法：`/config show`、`/config save`",
                    other
                ),
            );
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::CustomAgentRegistry;
    use crate::app::{
        AgentState, ChatState, DiffState, FocusedPanel, InputMode, LayoutState, SidebarState,
        SidebarTab, StatusBarState,
    };
    use crate::config::{KeyBindings, KeymapProfile, Settings};
    use crate::skills::SkillRegistry;

    fn make_app() -> App {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let (_, crx) = tokio::sync::mpsc::unbounded_channel();
        let settings = Settings::defaults();
        App {
            mode: InputMode::Insert,
            focused_panel: FocusedPanel::Input,
            chat_state: ChatState {
                messages: Vec::new(),
                streaming_text: String::new(),
                is_streaming: false,
                scroll_offset: 0,
            },
            sidebar_state: SidebarState {
                active_tab: SidebarTab::Files,
                file_tree: Vec::new(),
                scroll_offset: 0,
            },
            diff_state: DiffState {
                visible: false,
                content: String::new(),
                hunks: Vec::new(),
                scroll_offset: 0,
            },
            status_bar: StatusBarState {
                agent_state: AgentState::Idle,
                model_name: settings.llm.model.clone(),
                tokens_used: 0,
                tokens_total: 0,
                sandbox_mode: "confirm".to_string(),
                startup_metrics: None,
                goal_active: false,
                goal_duration: String::new(),
                coding_mode: crate::coding_modes::CodingMode::Act,
            },
            settings,
            should_quit: false,
            dirty_flags: crate::app::DirtyFlags::all_dirty(),
            input_buffer: String::new(),
            input_cursor: 0,
            keybindings: KeyBindings::new(KeymapProfile::Default),
            yank_buffer: String::new(),
            layout_state: LayoutState::default(),
            agent_rx: rx,
            agent_tx: tx,
            config_reload_rx: crx,
            config_version: 0,
            skill_registry: SkillRegistry::new(),
            agent_registry: CustomAgentRegistry::new(),
            goal_state: GoalState::inactive(),
            coding_mode: crate::coding_modes::CodingMode::Act,
        }
    }

    #[test]
    fn test_ignore_non_slash() {
        let mut app = make_app();
        assert!(!try_handle(&mut app, "hello"));
        assert!(!try_handle(&mut app, ""));
        assert!(!try_handle(&mut app, "  normal text"));
    }

    #[test]
    fn test_help() {
        let mut app = make_app();
        assert!(try_handle(&mut app, "/help"));
        assert_eq!(app.chat_state.messages.len(), 1);
        assert!(app.chat_state.messages[0].content.contains("/clear"));
    }

    #[test]
    fn test_clear() {
        let mut app = make_app();
        app.chat_state.messages.push(ChatMessage {
            role: MessageRole::User,
            content: "hello".into(),
            tool_calls: Vec::new(),
        });
        assert!(try_handle(&mut app, "/clear"));
        // Only the system confirmation message remains
        assert_eq!(app.chat_state.messages.len(), 1);
        assert_eq!(app.chat_state.messages[0].role, MessageRole::System);
    }

    #[test]
    fn test_model_switch() {
        let mut app = make_app();
        let old = app.settings.llm.model.clone();
        assert!(try_handle(&mut app, "/model mimo-v2.5-pro"));
        assert_eq!(app.settings.llm.model, "mimo-v2.5-pro");
        assert_eq!(app.status_bar.model_name, "mimo-v2.5-pro");
        assert!(app.chat_state.messages[0].content.contains(&old));
    }

    #[test]
    fn test_model_no_args() {
        let mut app = make_app();
        assert!(try_handle(&mut app, "/model"));
        assert!(app
            .chat_state
            .messages
            .last()
            .unwrap()
            .content
            .contains("当前模型"));
    }

    #[test]
    fn test_model_no_args_shows_provider() {
        let mut app = make_app();
        assert!(try_handle(&mut app, "/model"));
        let content = &app.chat_state.messages.last().unwrap().content;
        assert!(content.contains("提供商"));
        assert!(content.contains("DeepSeek"));
    }

    #[test]
    fn test_model_custom() {
        let mut app = make_app();
        assert!(try_handle(
            &mut app,
            "/model custom claude-sonnet-4 https://api.anthropic.com/v1"
        ));
        assert_eq!(app.settings.llm.model, "claude-sonnet-4");
        match &app.settings.llm.provider {
            Provider::Custom { base_url } => {
                assert_eq!(base_url, "https://api.anthropic.com/v1");
            }
            other => panic!("expected Custom provider, got {:?}", other),
        }
        assert_eq!(app.status_bar.model_name, "claude-sonnet-4");
        let content = &app.chat_state.messages.last().unwrap().content;
        assert!(content.contains("自定义模型"));
        assert!(content.contains("claude-sonnet-4"));
        assert!(content.contains("api.anthropic.com"));
    }

    #[test]
    fn test_model_custom_missing_args() {
        let mut app = make_app();
        assert!(try_handle(&mut app, "/model custom"));
        let content = &app.chat_state.messages.last().unwrap().content;
        assert!(content.contains("用法"));
    }

    #[test]
    fn test_reset() {
        let mut app = make_app();
        app.settings.llm.model = "some-custom-model".into();
        app.chat_state.messages.push(ChatMessage {
            role: MessageRole::User,
            content: "test".into(),
            tool_calls: Vec::new(),
        });
        assert!(try_handle(&mut app, "/reset"));
        // Only the system confirmation message remains
        assert_eq!(app.chat_state.messages.len(), 1);
        assert_eq!(app.chat_state.messages[0].role, MessageRole::System);
        assert_eq!(app.settings.llm.model, DEFAULT_MODEL);
    }

    #[test]
    fn test_exit() {
        let mut app = make_app();
        assert!(!app.should_quit);
        assert!(try_handle(&mut app, "/exit"));
        assert!(app.should_quit);
    }

    #[test]
    fn test_unknown() {
        let mut app = make_app();
        assert!(try_handle(&mut app, "/foobar"));
        assert!(app
            .chat_state
            .messages
            .last()
            .unwrap()
            .content
            .contains("未知命令"));
    }

    #[test]
    fn test_workflow() {
        let mut app = make_app();
        assert!(try_handle(&mut app, "/workflow"));
        assert!(app
            .chat_state
            .messages
            .last()
            .unwrap()
            .content
            .contains("code-review"));

        assert!(try_handle(&mut app, "/workflow code-review"));
        assert!(app
            .chat_state
            .messages
            .last()
            .unwrap()
            .content
            .contains("开发中"));
    }

    #[test]
    fn test_mcp_list_empty() {
        let mut app = make_app();
        assert!(try_handle(&mut app, "/mcp"));
        assert!(app
            .chat_state
            .messages
            .last()
            .unwrap()
            .content
            .contains("未配置 MCP"));
    }

    #[test]
    fn test_mcp_add_stdio() {
        let mut app = make_app();
        assert!(try_handle(
            &mut app,
            "/mcp add my-fs stdio npx @modelcontextprotocol/server-filesystem /tmp"
        ));
        assert_eq!(app.settings.mcp.len(), 1);
        assert_eq!(app.settings.mcp[0].name, "my-fs");
        assert!(matches!(
            app.settings.mcp[0].transport,
            McpTransport::Stdio { .. }
        ));
        assert!(app
            .chat_state
            .messages
            .last()
            .unwrap()
            .content
            .contains("my-fs"));
    }

    #[test]
    fn test_mcp_add_duplicate() {
        let mut app = make_app();
        assert!(try_handle(&mut app, "/mcp add my-srv stdio echo hello"));
        assert_eq!(app.settings.mcp.len(), 1);
        // Second add with same name should fail
        assert!(try_handle(&mut app, "/mcp add my-srv stdio other cmd"));
        assert_eq!(app.settings.mcp.len(), 1); // unchanged
        assert!(app
            .chat_state
            .messages
            .last()
            .unwrap()
            .content
            .contains("已存在"));
    }

    #[test]
    fn test_mcp_add_http() {
        let mut app = make_app();
        assert!(try_handle(
            &mut app,
            "/mcp add remote http http://localhost:8080"
        ));
        assert_eq!(app.settings.mcp.len(), 1);
        assert_eq!(app.settings.mcp[0].name, "remote");
        assert!(matches!(
            app.settings.mcp[0].transport,
            McpTransport::Http { .. }
        ));
    }

    #[test]
    fn test_mcp_remove() {
        let mut app = make_app();
        assert!(try_handle(&mut app, "/mcp add demo stdio echo test"));
        assert_eq!(app.settings.mcp.len(), 1);
        assert!(try_handle(&mut app, "/mcp remove demo"));
        assert!(app.settings.mcp.is_empty());
    }

    #[test]
    fn test_mcp_remove_not_found() {
        let mut app = make_app();
        assert!(try_handle(&mut app, "/mcp remove nonexistent"));
        assert!(app
            .chat_state
            .messages
            .last()
            .unwrap()
            .content
            .contains("未找到"));
    }

    #[test]
    fn test_skill_list_empty() {
        let mut app = make_app();
        assert!(try_handle(&mut app, "/skill"));
        let content = &app.chat_state.messages.last().unwrap().content;
        assert!(content.contains("Skills 目录"));
        assert!(content.contains("技能源：无"));
    }

    #[test]
    fn test_skill_set_dir() {
        let mut app = make_app();
        assert!(try_handle(&mut app, "/skill dir /home/user/skills"));
        assert_eq!(
            app.settings.skills_dir,
            Some(std::path::PathBuf::from("/home/user/skills"))
        );
    }

    #[test]
    fn test_skill_add_source() {
        let mut app = make_app();
        assert!(try_handle(
            &mut app,
            "/skill source add git https://github.com/user/skills main"
        ));
        assert_eq!(app.settings.skill_sources.len(), 1);
        assert_eq!(app.settings.skill_sources[0].source_type, "git");
        assert_eq!(app.settings.skill_sources[0].branch, Some("main".into()));
    }

    #[test]
    fn test_skill_remove_source() {
        let mut app = make_app();
        assert!(try_handle(&mut app, "/skill source add local ./skills"));
        assert_eq!(app.settings.skill_sources.len(), 1);
        assert!(try_handle(&mut app, "/skill source remove 1"));
        assert!(app.settings.skill_sources.is_empty());
    }

    #[test]
    fn test_skill_remove_source_invalid_index() {
        let mut app = make_app();
        assert!(try_handle(&mut app, "/skill source remove 1"));
        assert!(app
            .chat_state
            .messages
            .last()
            .unwrap()
            .content
            .contains("无效索引"));
    }

    #[test]
    fn test_config_show() {
        let mut app = make_app();
        assert!(try_handle(&mut app, "/config show"));
        let content = &app.chat_state.messages.last().unwrap().content;
        assert!(content.contains("配置路径"));
        assert!(content.contains("MCP 服务器"));
        assert!(content.contains("技能源"));
    }

    // ── Alias tests ─────────────────────────────────────────────────────

    #[test]
    fn test_alias_help() {
        let mut app = make_app();
        assert!(try_handle(&mut app, "/h"));
        assert!(app
            .chat_state
            .messages
            .last()
            .unwrap()
            .content
            .contains("/clear"));
        let mut app2 = make_app();
        assert!(try_handle(&mut app2, "/?"));
        assert!(app2
            .chat_state
            .messages
            .last()
            .unwrap()
            .content
            .contains("/clear"));
    }

    #[test]
    fn test_alias_clear() {
        let mut app = make_app();
        app.chat_state.messages.push(ChatMessage {
            role: MessageRole::User,
            content: "hello".into(),
            tool_calls: Vec::new(),
        });
        assert!(try_handle(&mut app, "/c"));
        assert_eq!(app.chat_state.messages.len(), 1);
        assert_eq!(app.chat_state.messages[0].role, MessageRole::System);

        let mut app2 = make_app();
        app2.chat_state.messages.push(ChatMessage {
            role: MessageRole::User,
            content: "hello".into(),
            tool_calls: Vec::new(),
        });
        assert!(try_handle(&mut app2, "/cls"));
        assert_eq!(app2.chat_state.messages.len(), 1);
    }

    #[test]
    fn test_alias_model() {
        let mut app = make_app();
        assert!(try_handle(&mut app, "/m mimo-v2.5-pro"));
        assert_eq!(app.settings.llm.model, "mimo-v2.5-pro");
    }

    #[test]
    fn test_alias_reset() {
        let mut app = make_app();
        app.chat_state.messages.push(ChatMessage {
            role: MessageRole::User,
            content: "test".into(),
            tool_calls: Vec::new(),
        });
        assert!(try_handle(&mut app, "/re"));
        assert_eq!(app.chat_state.messages.len(), 1);
    }

    #[test]
    fn test_alias_exit() {
        let mut app = make_app();
        assert!(try_handle(&mut app, "/quit"));
        assert!(app.should_quit);
        let mut app2 = make_app();
        assert!(try_handle(&mut app2, "/q"));
        assert!(app2.should_quit);
    }

    #[test]
    fn test_alias_workflow() {
        let mut app = make_app();
        assert!(try_handle(&mut app, "/wf"));
        assert!(app
            .chat_state
            .messages
            .last()
            .unwrap()
            .content
            .contains("code-review"));
    }

    #[test]
    fn test_alias_skill() {
        let mut app = make_app();
        assert!(try_handle(&mut app, "/skills"));
        let content = &app.chat_state.messages.last().unwrap().content;
        assert!(content.contains("Skills 目录"));
    }

    #[test]
    fn test_alias_config() {
        let mut app = make_app();
        assert!(try_handle(&mut app, "/cfg show"));
        let content = &app.chat_state.messages.last().unwrap().content;
        assert!(content.contains("配置路径"));
    }

    #[test]
    fn test_alias_goal() {
        let mut app = make_app();
        assert!(try_handle(&mut app, "/g"));
        let content = &app.chat_state.messages.last().unwrap().content;
        // /g without args shows goal status (inactive by default)
        assert!(content.contains("当前没有活跃"));
    }

    // ── Help system tests ───────────────────────────────────────────────

    #[test]
    fn test_help_categorized() {
        let mut app = make_app();
        assert!(try_handle(&mut app, "/help"));
        let content = &app.chat_state.messages.last().unwrap().content;
        assert!(content.contains("💡 帮助"));
        assert!(content.contains("📋 会话管理"));
        assert!(content.contains("🤖 模型配置"));
        assert!(content.contains("⚙️ 配置管理"));
        assert!(content.contains("🎯 目标模式"));
        assert!(content.contains("📦 工作流"));
        assert!(content.contains("/help <命令名>"));
    }

    #[test]
    fn test_help_detail_with_slash() {
        let mut app = make_app();
        // /help has aliases [/h, /?] so detail shows aliases section
        assert!(try_handle(&mut app, "/help /help"));
        let content = &app.chat_state.messages.last().unwrap().content;
        assert!(content.contains("/help"));
        assert!(content.contains("别名"));
        assert!(content.contains("用法"));
    }

    #[test]
    fn test_help_detail_without_slash() {
        let mut app = make_app();
        assert!(try_handle(&mut app, "/help clear"));
        let content = &app.chat_state.messages.last().unwrap().content;
        assert!(content.contains("/clear"));
        assert!(content.contains("清空"));
    }

    #[test]
    fn test_help_detail_unknown() {
        let mut app = make_app();
        assert!(try_handle(&mut app, "/help nonexistent"));
        let content = &app.chat_state.messages.last().unwrap().content;
        assert!(content.contains("没有"));
    }

    // ── Command category correctness ────────────────────────────────────

    #[test]
    fn test_cmd_category_labels() {
        // Verify all categories have non-empty labels
        assert!(!CmdCategory::Help.label().is_empty());
        assert!(!CmdCategory::Session.label().is_empty());
        assert!(!CmdCategory::Model.label().is_empty());
        assert!(!CmdCategory::Config.label().is_empty());
        assert!(!CmdCategory::Goal.label().is_empty());
        assert!(!CmdCategory::Workflow.label().is_empty());
    }

    #[test]
    fn test_every_command_has_handler() {
        // Verify every registered command resolves to a valid handler
        // Note: /exit sets should_quit but doesn't produce output
        for cmd in COMMANDS {
            let mut app = make_app();
            let result = try_handle(&mut app, cmd.name);
            assert!(result, "Command {} should be handled", cmd.name);
        }
    }
}
