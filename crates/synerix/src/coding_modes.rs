//! Coding mode system — controls agent behavior, tool access, and approval policy.
//!
//! Four modes:
//! - **Plan**   — analyse first, propose, then execute with explicit approval
//! - **Act**    — direct execution; auto-approves low/medium risk, asks for high/critical
//! - **Chat**   — read-only Q&A; files may be read but never written
//! - **Architect** — design-first; focuses on documentation and architecture decisions

use std::fmt;

/// The active coding mode, which governs agent autonomy and permission levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodingMode {
    /// 🧠 Plan first — analyse, propose a plan, then execute step-by-step with approval.
    /// The agent should outline what it intends to do before making any changes.
    Plan,
    /// ⚡ Act — direct execution. Low/medium-risk operations are auto-approved;
    /// high-risk and critical operations still require confirmation.
    Act,
    /// 💬 Chat — read-only Q&A. The agent can read files and answer questions,
    /// but **must not** write files, execute commands, or modify state.
    Chat,
    /// 🔧 Architect — design-first. Focuses on writing ADRs, architecture docs,
    /// and design decisions in the `docs/` directory only.
    Architect,
}

impl CodingMode {
    /// Return all variants in display order.
    pub fn all() -> &'static [CodingMode] {
        &[
            CodingMode::Plan,
            CodingMode::Act,
            CodingMode::Chat,
            CodingMode::Architect,
        ]
    }

    /// Human-readable label (short, for status bar).
    pub fn label(&self) -> &'static str {
        match self {
            CodingMode::Plan => "🧠 Plan",
            CodingMode::Act => "⚡ Act",
            CodingMode::Chat => "💬 Chat",
            CodingMode::Architect => "🔧 Architect",
        }
    }

    /// One-line description (for `/mode` help text).
    pub fn description(&self) -> &'static str {
        match self {
            CodingMode::Plan => "先规划后执行 — 分析问题，提出方案，逐步骤审批",
            CodingMode::Act => "直接执行 — 低风险自动放行，高风险需确认",
            CodingMode::Chat => "只读问答 — 可读文件，不可写文件或执行命令",
            CodingMode::Architect => "架构设计 — 专注 ADR/设计文档，仅允许写入 docs/ 目录",
        }
    }

    /// System prompt fragment injected into the agent's system prompt.
    ///
    /// Returns a system prompt suffix based on the current coding mode.
    #[allow(dead_code)]
    pub fn system_prompt_suffix(&self) -> &'static str {
        match self {
            CodingMode::Plan => "\n\n## 编码模式：规划模式 (Plan)\n在做出任何修改之前，请先分析问题并提出一个清晰的计划。\n列出你将修改的文件和步骤，等待用户确认后再执行。\n每个步骤完成后，确认结果并询问是否继续。",
            CodingMode::Act => "\n\n## 编码模式：执行模式 (Act)\n直接执行任务。低风险和中风险操作自动进行，\n高风险操作（如 sudo、递归删除）会向用户请求批准。\n专注于高效完成任务。",
            CodingMode::Chat => "\n\n## 编码模式：对话模式 (Chat)\n**只读模式**。你可以读取文件、分析代码、回答问题。\n**严禁**写入文件、执行修改命令或进行任何状态变更。\n仅用于代码审查、学习和调试分析。",
            CodingMode::Architect => "\n\n## 编码模式：架构模式 (Architect)\n专注于架构设计和决策记录。\n你可以读取任何文件进行分析，但只允许在 `docs/` 目录下写入。\n输出应为架构决策记录(ADR)、设计文档和架构分析。",
        }
    }

    /// Whether the agent is allowed to write files.
    pub fn allow_file_write(&self) -> bool {
        match self {
            CodingMode::Chat => false,
            CodingMode::Architect => true, // constrained to docs/ by prompt
            CodingMode::Plan | CodingMode::Act => true,
        }
    }

    /// Whether the agent is allowed to execute shell commands.
    #[allow(dead_code)]
    pub fn allow_command_exec(&self) -> bool {
        match self {
            CodingMode::Chat => false,
            CodingMode::Plan | CodingMode::Act | CodingMode::Architect => true,
        }
    }

    /// Whether the agent should pre-analyse and propose a plan before acting.
    #[allow(dead_code)]
    pub fn require_plan(&self) -> bool {
        match self {
            CodingMode::Plan => true,
            CodingMode::Act | CodingMode::Chat | CodingMode::Architect => false,
        }
    }

    /// Parse from a string argument (case-insensitive).
    pub fn parse(s: &str) -> Option<CodingMode> {
        match s.trim().to_lowercase().as_str() {
            "plan" | "p" | "规划" | "计划" => Some(CodingMode::Plan),
            "act" | "a" | "执行" | "行动" => Some(CodingMode::Act),
            "chat" | "c" | "对话" | "问答" => Some(CodingMode::Chat),
            "architect" | "arc" | "架构" | "设计" => Some(CodingMode::Architect),
            _ => None,
        }
    }
}

impl fmt::Display for CodingMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_variants_present() {
        let all = CodingMode::all();
        assert_eq!(all.len(), 4);
        assert!(all.contains(&CodingMode::Plan));
        assert!(all.contains(&CodingMode::Act));
        assert!(all.contains(&CodingMode::Chat));
        assert!(all.contains(&CodingMode::Architect));
    }

    #[test]
    fn test_labels_are_non_empty() {
        for mode in CodingMode::all() {
            assert!(!mode.label().is_empty());
            assert!(!mode.description().is_empty());
            assert!(!mode.system_prompt_suffix().is_empty());
        }
    }

    #[test]
    fn test_chat_mode_denies_writes_and_commands() {
        assert!(!CodingMode::Chat.allow_file_write());
        assert!(!CodingMode::Chat.allow_command_exec());
    }

    #[test]
    fn test_act_mode_allows_writes_and_commands() {
        assert!(CodingMode::Act.allow_file_write());
        assert!(CodingMode::Act.allow_command_exec());
    }

    #[test]
    fn test_plan_mode_requires_plan() {
        assert!(CodingMode::Plan.require_plan());
        assert!(!CodingMode::Act.require_plan());
        assert!(!CodingMode::Chat.require_plan());
        assert!(!CodingMode::Architect.require_plan());
    }

    #[test]
    fn test_architect_allows_writes() {
        assert!(CodingMode::Architect.allow_file_write());
        assert!(CodingMode::Architect.allow_command_exec());
    }

    #[test]
    fn test_parse_case_insensitive() {
        assert_eq!(CodingMode::parse("plan"), Some(CodingMode::Plan));
        assert_eq!(CodingMode::parse("PLAN"), Some(CodingMode::Plan));
        assert_eq!(CodingMode::parse("P"), Some(CodingMode::Plan));
        assert_eq!(CodingMode::parse("Act"), Some(CodingMode::Act));
        assert_eq!(CodingMode::parse("CHAT"), Some(CodingMode::Chat));
        assert_eq!(CodingMode::parse("arc"), Some(CodingMode::Architect));
        assert_eq!(CodingMode::parse("架构"), Some(CodingMode::Architect));
        assert_eq!(CodingMode::parse("设计"), Some(CodingMode::Architect));
    }

    #[test]
    fn test_parse_invalid() {
        assert_eq!(CodingMode::parse("foo"), None);
        assert_eq!(CodingMode::parse(""), None);
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", CodingMode::Plan), "🧠 Plan");
        assert_eq!(format!("{}", CodingMode::Act), "⚡ Act");
        assert_eq!(format!("{}", CodingMode::Chat), "💬 Chat");
        assert_eq!(format!("{}", CodingMode::Architect), "🔧 Architect");
    }
}
