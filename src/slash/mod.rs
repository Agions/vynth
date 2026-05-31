//! Slash commands — `/help`, `/clear`, `/model`, `/reset`, `/exit`, `/workflow`
//!
//! Intercepts user input starting with `/` and executes built-in commands
//! instead of sending the message to the AI model.

use crate::app::{App, ChatMessage, MessageRole};
use crate::config::Provider;

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
                app.settings.llm.provider =
                    Provider::Custom { base_url: base_url.to_string() };
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
}
