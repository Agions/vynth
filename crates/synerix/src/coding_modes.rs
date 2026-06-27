//! Coding mode system — controls agent behavior, tool access, and approval policy.
//!
//! Five modes:
//! - **Plan**   — analyse first, propose, then execute with explicit approval
//! - **Act**    — direct execution; auto-approves low/medium risk, asks for high/critical
//! - **Chat**   — read-only Q&A; files may be read but never written
//! - **Architect** — design-first; focuses on documentation and architecture decisions
//! - **Vibe**   — vibe-coding: immersive iteration, auto-compile → test → fix → loop

use std::fmt;

/// The active coding mode, which governs agent autonomy and permission levels.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum CodingMode {
    /// 🧠 Plan first — analyse, propose a plan, then execute step-by-step with approval.
    /// The agent should outline what it intends to do before making any changes.
    Plan,
    /// ⚡ Act — direct execution. Low/medium-risk operations are auto-approved;
    /// high-risk and critical operations still require confirmation.
    #[default]
    Act,
    /// 💬 Chat — read-only Q&A. The agent can read files and answer questions,
    /// but **must not** write files, execute commands, or modify state.
    Chat,
    /// 🔧 Architect — design-first. Focuses on writing ADRs, architecture docs,
    /// and design decisions in the `docs/` directory only.
    Architect,
    /// 🎵 Vibe — vibe-coding mode: immersive iteration.
    /// Describe → generate → auto-compile → test → fix → loop.
    /// Minimal approval friction; error-driven auto-repair.
    /// Best for prototyping, CRUD, scripts, and rapid iteration.
    Vibe,
}

impl CodingMode {
    /// Return all variants in display order.
    pub fn all() -> &'static [CodingMode] {
        &[
            CodingMode::Plan,
            CodingMode::Act,
            CodingMode::Chat,
            CodingMode::Architect,
            CodingMode::Vibe,
        ]
    }

    pub fn next(self) -> CodingMode {
        let modes = Self::all();
        let idx = modes.iter().position(|mode| *mode == self).unwrap_or(0);
        modes[(idx + 1) % modes.len()]
    }

    pub fn previous(self) -> CodingMode {
        let modes = Self::all();
        let idx = modes.iter().position(|mode| *mode == self).unwrap_or(0);
        modes[(idx + modes.len() - 1) % modes.len()]
    }

    /// Human-readable label (short, for status bar).
    pub fn label(&self) -> &'static str {
        match self {
            CodingMode::Plan => "🧠 Plan",
            CodingMode::Act => "⚡ Act",
            CodingMode::Chat => "💬 Chat",
            CodingMode::Architect => "🔧 Architect",
            CodingMode::Vibe => "🎵 Vibe",
        }
    }

    pub fn plain_label(&self) -> &'static str {
        match self {
            CodingMode::Plan => "Plan",
            CodingMode::Act => "Act",
            CodingMode::Chat => "Chat",
            CodingMode::Architect => "Architect",
            CodingMode::Vibe => "Vibe",
        }
    }

    /// One-line description (for `/mode` help text).
    pub fn description(&self) -> &'static str {
        match self {
            CodingMode::Plan => "先规划后执行 — 分析问题，提出方案，逐步骤审批",
            CodingMode::Act => "直接执行 — 低风险自动放行，高风险需确认",
            CodingMode::Chat => "只读问答 — 可读文件，不可写文件或执行命令",
            CodingMode::Architect => "架构设计 — 专注 ADR/设计文档，仅允许写入 docs/ 目录",
            CodingMode::Vibe => "沉浸式迭代 — 描述需求→AI生成→自动编译测试→修复循环",
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
            CodingMode::Vibe => "\n\n## 编码模式：沉浸式模式 (Vibe)\n你是用户的 AI 编程搭档，采用「氛围编程/Vibe Coding」工作流：\n1. **沉浸式迭代**：用户描述需求后，直接进入「生成代码→自动编译→看结果→修复→再编译」循环\n2. **零阻碍执行**：低风险和中风险操作自动放行，不打断心流状态\n3. **错误驱动修复**：编译错误、测试失败直接反馈给 LLM 模型，自动修复直到通过\n4. **少问多干**：除非遇到安全问题或高风险操作（sudo、递归删除），否则先干再说\n5. **即时验证**：完成代码后自动运行 `cargo check` 或 `cargo test`，确保代码正确\n6. **输出精简**：不要输出大量解释，专注于执行和结果展示",
        }
    }

    /// Whether the agent is allowed to write files.
    pub fn allow_file_write(&self) -> bool {
        match self {
            CodingMode::Chat => false,
            CodingMode::Architect => true, // constrained to docs/ by prompt
            CodingMode::Plan | CodingMode::Act | CodingMode::Vibe => true,
        }
    }

    /// Whether the agent is allowed to execute shell commands.
    #[allow(dead_code)]
    pub fn allow_command_exec(&self) -> bool {
        match self {
            CodingMode::Chat => false,
            CodingMode::Plan | CodingMode::Act | CodingMode::Architect | CodingMode::Vibe => true,
        }
    }

    /// Whether the agent should pre-analyse and propose a plan before acting.
    #[allow(dead_code)]
    pub fn require_plan(&self) -> bool {
        match self {
            CodingMode::Plan => true,
            CodingMode::Act | CodingMode::Chat | CodingMode::Architect | CodingMode::Vibe => false,
        }
    }

    /// Whether this mode supports auto-iteration (auto-compile → test → fix).
    /// Vibe mode enables this — the agent loop will auto-feed errors back.
    /// This method is a public API hook for future agent-loop integration;
    /// currently the behavior is driven via `system_prompt_suffix()`.
    #[allow(dead_code)]
    pub fn auto_iterate(&self) -> bool {
        match self {
            CodingMode::Vibe => true,
            CodingMode::Plan | CodingMode::Act | CodingMode::Chat | CodingMode::Architect => false,
        }
    }

    /// Parse from a string argument (case-insensitive).
    pub fn parse(s: &str) -> Option<CodingMode> {
        match s.trim().to_lowercase().as_str() {
            "plan" | "p" | "规划" | "计划" => Some(CodingMode::Plan),
            "act" | "a" | "执行" | "行动" => Some(CodingMode::Act),
            "chat" | "c" | "对话" | "问答" => Some(CodingMode::Chat),
            "architect" | "arc" | "架构" | "设计" => Some(CodingMode::Architect),
            "vibe" | "v" | "沉浸" | "氛围" => Some(CodingMode::Vibe),
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
        assert_eq!(all.len(), 5);
        assert!(all.contains(&CodingMode::Plan));
        assert!(all.contains(&CodingMode::Act));
        assert!(all.contains(&CodingMode::Chat));
        assert!(all.contains(&CodingMode::Architect));
        assert!(all.contains(&CodingMode::Vibe));
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
    fn test_vibe_mode_permissions() {
        assert!(CodingMode::Vibe.allow_file_write());
        assert!(CodingMode::Vibe.allow_command_exec());
        assert!(!CodingMode::Vibe.require_plan());
        assert!(CodingMode::Vibe.auto_iterate());
    }

    #[test]
    fn test_parse_vibe() {
        assert_eq!(CodingMode::parse("vibe"), Some(CodingMode::Vibe));
        assert_eq!(CodingMode::parse("VIBE"), Some(CodingMode::Vibe));
        assert_eq!(CodingMode::parse("V"), Some(CodingMode::Vibe));
        assert_eq!(CodingMode::parse("沉浸"), Some(CodingMode::Vibe));
        assert_eq!(CodingMode::parse("氛围"), Some(CodingMode::Vibe));
    }

    #[test]
    fn test_mode_cycle() {
        assert_eq!(CodingMode::Plan.next(), CodingMode::Act);
        assert_eq!(CodingMode::Vibe.next(), CodingMode::Plan);
        assert_eq!(CodingMode::Plan.previous(), CodingMode::Vibe);
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", CodingMode::Plan), "🧠 Plan");
        assert_eq!(format!("{}", CodingMode::Act), "⚡ Act");
        assert_eq!(format!("{}", CodingMode::Chat), "💬 Chat");
        assert_eq!(format!("{}", CodingMode::Architect), "🔧 Architect");
        assert_eq!(format!("{}", CodingMode::Vibe), "🎵 Vibe");
    }
}
