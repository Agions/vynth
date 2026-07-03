//! Coding mode system — controls agent behavior, tool access, and approval policy.
//!
//! Two modes:
//! - **Plan**   — analyse first, propose a plan, then execute with explicit approval
//! - **Vibe**   — immersive iteration: describe → generate → compile → test → fix → loop

use std::fmt;

/// The active coding mode, which governs agent autonomy and permission levels.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum CodingMode {
    /// Plan first — analyse, propose a plan, then execute step-by-step with approval.
    #[default]
    Plan,
    /// Vibe-coding: immersive iteration with minimal friction.
    /// Auto-compile → test → fix → loop.
    Vibe,
}

impl CodingMode {
    /// Return all variants in display order.
    pub fn all() -> &'static [CodingMode] {
        &[CodingMode::Plan, CodingMode::Vibe]
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
            CodingMode::Plan => "Plan",
            CodingMode::Vibe => "Vibe",
        }
    }

    pub fn plain_label(&self) -> &'static str {
        match self {
            CodingMode::Plan => "Plan",
            CodingMode::Vibe => "Vibe",
        }
    }

    /// One-line description (for `/mode` help text).
    pub fn description(&self) -> &'static str {
        match self {
            CodingMode::Plan => "先规划后执行 — 分析问题，提出方案，逐步骤审批",
            CodingMode::Vibe => "沉浸式迭代 — 描述需求→AI生成→自动编译测试→修复循环",
        }
    }

    /// System prompt fragment injected into the agent's system prompt.
    pub fn system_prompt_suffix(&self) -> &'static str {
        match self {
            CodingMode::Plan => "\n\n## 编码模式：规划模式 (Plan)\n在做出任何修改之前，请先分析问题并提出一个清晰的计划。\n列出你将修改的文件和步骤，等待用户确认后再执行。\n每个步骤完成后，确认结果并询问是否继续。",
            CodingMode::Vibe => "\n\n## 编码模式：沉浸式模式 (Vibe)\n你是用户的 AI 编程搭档，采用「氛围编程/Vibe Coding」工作流：\n1. **沉浸式迭代**：用户描述需求后，直接进入「生成代码→自动编译→看结果→修复→再编译」循环\n2. **零阻碍执行**：低风险和中风险操作自动放行，不打断心流状态\n3. **错误驱动修复**：编译错误、测试失败直接反馈给 LLM 模型，自动修复直到通过\n4. **少问多干**：除非遇到安全问题或高风险操作（sudo、递归删除），否则先干再说\n5. **即时验证**：完成代码后自动运行 `cargo check` 或 `cargo test`，确保代码正确\n6. **输出精简**：不要输出大量解释，专注于执行和结果展示",
        }
    }

    /// Whether the agent is allowed to write files.
    pub fn allow_file_write(&self) -> bool {
        match self {
            CodingMode::Plan | CodingMode::Vibe => true,
        }
    }

    /// Whether the agent is allowed to execute shell commands.
    pub fn allow_command_exec(&self) -> bool {
        match self {
            CodingMode::Plan | CodingMode::Vibe => true,
        }
    }

    /// Whether the agent should pre-analyse and propose a plan before acting.
    pub fn require_plan(&self) -> bool {
        match self {
            CodingMode::Plan => true,
            CodingMode::Vibe => false,
        }
    }

    /// Whether this mode supports auto-iteration (auto-compile → test → fix).
    pub fn auto_iterate(&self) -> bool {
        match self {
            CodingMode::Vibe => true,
            CodingMode::Plan => false,
        }
    }

    /// Parse from a string argument (case-insensitive).
    pub fn parse(s: &str) -> Option<CodingMode> {
        match s.trim().to_lowercase().as_str() {
            "plan" | "p" | "规划" | "计划" => Some(CodingMode::Plan),
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
    fn test_all_variants() {
        let all = CodingMode::all();
        assert_eq!(all.len(), 2);
        assert_eq!(all[0], CodingMode::Plan);
        assert_eq!(all[1], CodingMode::Vibe);
    }

    #[test]
    fn test_labels() {
        assert_eq!(CodingMode::Plan.label(), "Plan");
        assert_eq!(CodingMode::Vibe.label(), "Vibe");
    }

    #[test]
    fn test_descriptions_non_empty() {
        for mode in CodingMode::all() {
            assert!(!mode.description().is_empty());
            assert!(!mode.system_prompt_suffix().is_empty());
        }
    }

    #[test]
    fn test_permissions() {
        assert!(CodingMode::Plan.allow_file_write());
        assert!(CodingMode::Vibe.allow_file_write());
        assert!(CodingMode::Plan.allow_command_exec());
        assert!(CodingMode::Vibe.allow_command_exec());
    }

    #[test]
    fn test_plan_requires_plan() {
        assert!(CodingMode::Plan.require_plan());
        assert!(!CodingMode::Vibe.require_plan());
    }

    #[test]
    fn test_vibe_auto_iterate() {
        assert!(CodingMode::Vibe.auto_iterate());
        assert!(!CodingMode::Plan.auto_iterate());
    }

    #[test]
    fn test_parse() {
        assert_eq!(CodingMode::parse("plan"), Some(CodingMode::Plan));
        assert_eq!(CodingMode::parse("PLAN"), Some(CodingMode::Plan));
        assert_eq!(CodingMode::parse("p"), Some(CodingMode::Plan));
        assert_eq!(CodingMode::parse("vibe"), Some(CodingMode::Vibe));
        assert_eq!(CodingMode::parse("VIBE"), Some(CodingMode::Vibe));
        assert_eq!(CodingMode::parse("v"), Some(CodingMode::Vibe));
        assert_eq!(CodingMode::parse("foo"), None);
        assert_eq!(CodingMode::parse(""), None);
    }

    #[test]
    fn test_mode_cycle() {
        assert_eq!(CodingMode::Plan.next(), CodingMode::Vibe);
        assert_eq!(CodingMode::Vibe.next(), CodingMode::Plan);
        assert_eq!(CodingMode::Plan.previous(), CodingMode::Vibe);
        assert_eq!(CodingMode::Vibe.previous(), CodingMode::Plan);
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", CodingMode::Plan), "Plan");
        assert_eq!(format!("{}", CodingMode::Vibe), "Vibe");
    }
}
