//! Slash commands — `/help`, `/clear`, `/model`, `/reset`, `/exit`, `/workflow`
//!
//! Intercepts user input starting with `/` and executes built-in commands
//! instead of sending the message to the AI model.

use crate::app::{App, ChatMessage, MessageRole};
use crate::config::{McpServerConfig, McpTransport, Provider, SkillSourceConfig};

const DEFAULT_MODEL: &str = "deepseek-v4-flash";

/// List of all available slash commands with descriptions
const COMMANDS: &[CommandDef] = &[
    CommandDef {
        name: "/help",
        desc: "显示所有斜杠命令",
        usage: "/help",
    },
    CommandDef {
        name: "/clear",
        desc: "清空当前对话",
        usage: "/clear",
    },
    CommandDef {
        name: "/model",
        desc: "切换 LLM 模型 / 配置自定义提供商",
        usage: "/model [name] | /model custom <name> <base-url>",
    },
    CommandDef {
        name: "/reset",
        desc: "重置对话状态",
        usage: "/reset",
    },
    CommandDef {
        name: "/exit",
        desc: "退出 Synerix",
        usage: "/exit",
    },
    CommandDef {
        name: "/workflow",
        desc: "运行工作流",
        usage: "/workflow <name>",
    },
    CommandDef {
        name: "/mcp",
        desc: "管理 MCP 服务器",
        usage: "/mcp [list|show|add|remove]",
    },
    CommandDef {
        name: "/skill",
        desc: "管理技能（Skills）",
        usage: "/skill [dir|source [list|add|remove]]",
    },
    CommandDef {
        name: "/config",
        desc: "管理配置",
        usage: "/config [save|show]",
    },
];

struct CommandDef {
    name: &'static str,
    desc: &'static str,
    usage: &'static str,
}

/// Try to handle a slash command. Returns `true` if the input was a slash command.
pub fn try_handle(app: &mut App, input: &str) -> bool {
    let trimmed = input.trim();
    if !trimmed.starts_with('/') {
        return false;
    }

    let (cmd, args) = match trimmed.split_once(' ') {
        Some((c, a)) => (c.trim(), Some(a.trim())),
        None => (trimmed, None),
    };

    match cmd {
        "/help" => cmd_help(app),
        "/clear" => cmd_clear(app),
        "/model" => cmd_model(app, args),
        "/reset" => cmd_reset(app),
        "/exit" => cmd_exit(app),
        "/workflow" => cmd_workflow(app, args),
        "/mcp" => cmd_mcp(app, args),
        "/skill" => cmd_skill(app, args),
        "/config" => cmd_config(app, args),
        _ => {
            sys_msg(
                app,
                &format!("未知命令 `{}`。输入 `/help` 查看可用命令。", cmd),
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

fn cmd_help(app: &mut App) -> bool {
    let mut lines = vec!["**可用斜杠命令：**".to_string(), String::new()];
    for cmd in COMMANDS {
        lines.push(format!("  `{}` — {}", cmd.name, cmd.desc));
        lines.push(format!("    用法：`{}`", cmd.usage));
        lines.push(String::new());
    }
    lines.push("提示：斜杠命令仅在输入框第一字符为 `/` 时触发。".to_string());
    sys_msg(app, &lines.join("\n"));
    true
}

fn cmd_clear(app: &mut App) -> bool {
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
        Some(args) if args.is_empty() => {
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

fn cmd_reset(app: &mut App) -> bool {
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

fn cmd_exit(app: &mut App) -> bool {
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
        Some(name) if name.is_empty() => {
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
    let args = args.unwrap_or("");
    let trimmed = args.trim();

    // 无参数或 `list` — 列出所有
    if trimmed.is_empty() || trimmed == "list" {
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
        return true;
    }

    // `show <name>` — 查看详情
    if let Some(name) = trimmed.strip_prefix("show ") {
        let name = name.trim();
        if let Some(server) = app.settings.mcp.iter().find(|s| s.name == name) {
            let detail = format_mcp_server_detail(
                app.settings
                    .mcp
                    .iter()
                    .position(|s| s.name == name)
                    .unwrap(),
                server,
            );
            sys_msg(app, &detail);
        } else {
            sys_msg(app, &format!("❌ 未找到 MCP 服务器：`{}`", name));
        }
        return true;
    }

    // `add <name> stdio <command> [args...]`
    if let Some(rest) = trimmed.strip_prefix("add ") {
        let parts: Vec<&str> = rest.splitn(3, ' ').collect();
        if parts.len() < 3 || parts[0].trim().is_empty() || parts[1].trim().is_empty() {
            sys_msg(app, &format!("❌ 参数不足。\n\n{}", format_mcp_usage()));
            return true;
        }
        let name = parts[0].trim();
        let transport_type = parts[1].trim();

        if mcp_add_duplicate_check(app, name) {
            return true;
        }

        match transport_type {
            "stdio" => {
                let cmd_rest = parts[2].trim();
                let cmd_parts: Vec<&str> = cmd_rest.split_whitespace().collect();
                if cmd_parts.is_empty() {
                    sys_msg(app, "❌ 请指定 stdio 命令。");
                    return true;
                }
                let command = cmd_parts[0].to_string();
                let args_vec: Vec<String> = cmd_parts[1..].iter().map(|s| s.to_string()).collect();

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
                let url = parts[2].trim().to_string();
                if url.is_empty() {
                    sys_msg(app, "❌ 请指定 HTTP URL。");
                    return true;
                }
                app.settings.mcp.push(McpServerConfig {
                    name: name.to_string(),
                    transport: McpTransport::Http { url },
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
        return true;
    }

    // `remove <name>`
    if let Some(name) = trimmed.strip_prefix("remove ") {
        let name = name.trim();
        if name.is_empty() {
            sys_msg(app, "❌ 用法：`/mcp remove <name>`");
            return true;
        }
        let len_before = app.settings.mcp.len();
        app.settings.mcp.retain(|s| s.name != name);
        if app.settings.mcp.len() < len_before {
            sys_msg(app, &format!("🗑️ 已移除 MCP 服务器：`{}`", name));
        } else {
            sys_msg(app, &format!("❌ 未找到 MCP 服务器：`{}`", name));
        }
        return true;
    }

    sys_msg(app, &format!("❌ 未知子命令。\n\n{}", format_mcp_usage()));
    true
}

fn cmd_skill(app: &mut App, args: Option<&str>) -> bool {
    let args = args.unwrap_or("");
    let trimmed = args.trim();

    // 无参数 — 显示状态
    if trimmed.is_empty() {
        let dir_info = match &app.settings.skills_dir {
            Some(path) => format!("`{}`", path.display()),
            None => "未设置（使用默认路径）".to_string(),
        };
        let mut lines = vec![format!("📂 Skills 目录：{}", dir_info), String::new()];

        // source list 作为状态视图的一部分
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
        return true;
    }

    // `dir <path>`
    if let Some(path) = trimmed.strip_prefix("dir ") {
        let path = path.trim();
        if path.is_empty() {
            sys_msg(app, "❌ 用法：`/skill dir <path>`");
            return true;
        }
        app.settings.skills_dir = Some(std::path::PathBuf::from(path));
        sys_msg(app, &format!("📂 Skills 目录已设置为：`{}`", path));
        return true;
    }

    // `source <subcommand> [args...]`
    if let Some(rest) = trimmed.strip_prefix("source ") {
        let rest = rest.trim();
        return cmd_skill_source(app, rest);
    }

    sys_msg(app, "❌ 未知子命令。↙ 输入 `/skill` 查看用法。");
    true
}

fn cmd_skill_source(app: &mut App, args: &str) -> bool {
    let trimmed = args.trim();

    // `source list`
    if trimmed.is_empty() || trimmed == "list" {
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
        return true;
    }

    // `source add <type> <location> [branch]`
    if let Some(rest) = trimmed.strip_prefix("add ") {
        let parts: Vec<&str> = rest.splitn(3, ' ').collect();
        if parts.len() < 2 || parts[0].trim().is_empty() || parts[1].trim().is_empty() {
            sys_msg(app, "❌ 用法：`/skill source add <type> <location> [branch]`\n类型：`git`、`local`、`url`\n例如：`/skill source add git https://github.com/user/skills main`");
            return true;
        }
        let source_type = parts[0].trim();
        let location = parts[1].trim().to_string();
        let branch = parts
            .get(2)
            .map(|s| s.trim().to_string())
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
        return true;
    }

    // `source remove <index>`
    if let Some(idx_str) = trimmed.strip_prefix("remove ") {
        let idx_str = idx_str.trim();
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
        return true;
    }

    sys_msg(app, "❌ 未知子命令。用法：`/skill source list`、`/skill source add <type> <location>`、`/skill source remove <index>`");
    true
}

fn cmd_config(app: &mut App, args: Option<&str>) -> bool {
    let args = args.unwrap_or("");
    let trimmed = args.trim();

    if trimmed.is_empty() || trimmed == "show" {
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
        return true;
    }

    if trimmed == "save" {
        match app.settings.save() {
            Ok(()) => {
                let path = crate::config::settings::Settings::config_path();
                sys_msg(app, &format!("💾 配置已保存到：`{}`", path.display()));
            }
            Err(e) => {
                sys_msg(app, &format!("❌ 保存配置失败：{}", e));
            }
        }
        return true;
    }

    sys_msg(app, "❌ 未知子命令。用法：`/config show`、`/config save`");
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{
        AgentState, ChatState, DiffState, FocusedPanel, InputMode, LayoutState, SidebarState,
        SidebarTab, StatusBarState,
    };
    use crate::config::{KeyBindings, KeymapProfile, Settings};

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
            },
            settings,
            should_quit: false,
            input_buffer: String::new(),
            input_cursor: 0,
            keybindings: KeyBindings::new(KeymapProfile::Default),
            yank_buffer: String::new(),
            layout_state: LayoutState::default(),
            agent_rx: rx,
            agent_tx: tx,
            config_reload_rx: crx,
            config_version: 0,
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
}
