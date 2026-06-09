//! 斜杠命令系统 — 注册/调度/CmdDef 定义
//!
//! # 架构说明
//! 此文件只做三件事：
//!   1. 声明子模块（每个命令或命令族独立一个文件）
//!   2. 定义命令结构（CmdCategory / CmdDef / COMMANDS 注册表）
//!   3. 提供入口函数 try_handle() 供外部调用
//!
//! 具体的命令处理逻辑分散在各子模块中，通过 handler 指针注入注册表。
//! 添加新命令只需：①在子模块中实现 handler；②在 COMMANDS 数组中注册。
//! 无需修改此文件以外的调度逻辑。

pub mod common;
pub mod config;
pub mod goal;
pub mod help;
pub mod mcp;
pub mod mode;
pub mod model;
pub mod session;
pub mod skill;
pub mod workflow;

use crate::app::App;

use self::common::sys_msg;
use self::config::cmd_config;
use self::goal::cmd_goal;
use self::help::cmd_help;
use self::mcp::cmd_mcp;
use self::mode::cmd_mode;
use self::model::cmd_model;
use self::session::{cmd_clear, cmd_exit, cmd_reset};
use self::skill::cmd_skill;
use self::workflow::cmd_workflow;

// ── 命令分类 ──────────────────────────────────────────────────────────────────────

/// 命令分类（用于 /help 的分组展示）
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
    /// 分类的中文展示标签
    pub fn label(self) -> &'static str {
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

// ── 命令定义 ──────────────────────────────────────────────────────────────────────

/// Handler 函数类型: `fn(&mut App, Option<&str>) -> bool`
type CmdHandler = fn(&mut App, Option<&str>) -> bool;

/// 一条命令的结构化定义
pub struct CmdDef {
    pub name: &'static str,
    pub aliases: &'static [&'static str],
    pub desc: &'static str,
    pub usage: &'static str,
    pub category: CmdCategory,
    pub handler: CmdHandler,
}

// ── 命令注册表 ────────────────────────────────────────────────────────────────────

/// 所有可用斜杠命令的注册表
///
/// # 可扩展性
/// 添加新命令只需在此数组中追加一条 CmdDef 条目，
/// 无需修改调度逻辑或注册机制。
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

// ── 调度逻辑 ──────────────────────────────────────────────────────────────────────

/// 按名称或别名查找命令定义
///
/// # 优化说明
/// 与 try_handle 解耦：查找与执行分离，
/// 使 find_cmd 可被 help 模块复用（展示详细用法时）。
pub fn find_cmd(input: &str) -> Option<&'static CmdDef> {
    COMMANDS
        .iter()
        .find(|c| c.name == input || c.aliases.contains(&input))
}

/// 尝试处理斜杠命令入口
///
/// 返回 `true` 表示输入是一个已处理的斜杠命令，
/// 返回 `false` 表示输入不是斜杠命令（应继续走 AI 对话流程）。
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

// ── 测试套件 ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::CustomAgentRegistry;
    use crate::app::{
        AgentState, ChatState, DiffState, FocusedPanel, GoalState, InputMode, LayoutState,
        SidebarState, SidebarTab, StatusBarState,
    };
    use crate::config::{KeyBindings, KeymapProfile, Provider, Settings};
    use crate::skills::SkillRegistry;

    fn make_app() -> App {
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
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
            agent_rx: _rx,
            agent_tx: tx,
            config_reload_rx: crx,
            config_version: 0,
            skill_registry: SkillRegistry::new(),
            agent_registry: CustomAgentRegistry::new(),
            goal_state: GoalState::inactive(),
            coding_mode: crate::coding_modes::CodingMode::Act,
        }
    }

    // ── Dispatch tests ─────────────────────────────────────────────────────

    #[test]
    fn test_ignore_non_slash() {
        let mut app = make_app();
        assert!(!try_handle(&mut app, "hello"));
        assert!(!try_handle(&mut app, ""));
        assert!(!try_handle(&mut app, "  normal text"));
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

    // ── Command handler tests ──────────────────────────────────────────────

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
        app.chat_state.messages.push(crate::app::ChatMessage {
            role: crate::app::MessageRole::User,
            content: "hello".into(),
            tool_calls: Vec::new(),
        });
        assert!(try_handle(&mut app, "/clear"));
        assert_eq!(app.chat_state.messages.len(), 1);
        assert_eq!(
            app.chat_state.messages[0].role,
            crate::app::MessageRole::System
        );
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
        app.chat_state.messages.push(crate::app::ChatMessage {
            role: crate::app::MessageRole::User,
            content: "test".into(),
            tool_calls: Vec::new(),
        });
        assert!(try_handle(&mut app, "/reset"));
        assert_eq!(app.chat_state.messages.len(), 1);
        assert_eq!(
            app.chat_state.messages[0].role,
            crate::app::MessageRole::System
        );
        assert_eq!(app.settings.llm.model, crate::slash::common::DEFAULT_MODEL);
    }

    #[test]
    fn test_exit() {
        let mut app = make_app();
        assert!(!app.should_quit);
        assert!(try_handle(&mut app, "/exit"));
        assert!(app.should_quit);
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
            crate::config::McpTransport::Stdio { .. }
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
        assert!(try_handle(&mut app, "/mcp add my-srv stdio other cmd"));
        assert_eq!(app.settings.mcp.len(), 1);
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
            crate::config::McpTransport::Http { .. }
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

    // ── Alias tests ───────────────────────────────────────────────────────

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
        app.chat_state.messages.push(crate::app::ChatMessage {
            role: crate::app::MessageRole::User,
            content: "hello".into(),
            tool_calls: Vec::new(),
        });
        assert!(try_handle(&mut app, "/c"));
        assert_eq!(app.chat_state.messages.len(), 1);

        let mut app2 = make_app();
        app2.chat_state.messages.push(crate::app::ChatMessage {
            role: crate::app::MessageRole::User,
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
        app.chat_state.messages.push(crate::app::ChatMessage {
            role: crate::app::MessageRole::User,
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
        assert!(content.contains("当前没有活跃"));
    }

    // ── Help system tests ─────────────────────────────────────────────────

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

    // ── Registry integrity tests ──────────────────────────────────────────

    #[test]
    fn test_cmd_category_labels() {
        assert!(!CmdCategory::Help.label().is_empty());
        assert!(!CmdCategory::Session.label().is_empty());
        assert!(!CmdCategory::Model.label().is_empty());
        assert!(!CmdCategory::Config.label().is_empty());
        assert!(!CmdCategory::Goal.label().is_empty());
        assert!(!CmdCategory::Workflow.label().is_empty());
    }

    #[test]
    fn test_every_command_has_handler() {
        for cmd in COMMANDS {
            let mut app = make_app();
            let result = try_handle(&mut app, cmd.name);
            assert!(result, "Command {} should be handled", cmd.name);
        }
    }
}
