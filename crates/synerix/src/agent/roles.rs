//! Agent role definitions
// TODO: Agent role types — not yet wired
#![allow(dead_code)]

use serde::{Deserialize, Serialize};

/// Pre-defined agent roles
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentRole {
    Coder,
    Reviewer,
    Tester,
    Architect,
    Planner,
    Custom(String),
}

impl std::fmt::Display for AgentRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Coder => write!(f, "coder"),
            Self::Reviewer => write!(f, "reviewer"),
            Self::Tester => write!(f, "tester"),
            Self::Architect => write!(f, "architect"),
            Self::Planner => write!(f, "planner"),
            Self::Custom(name) => write!(f, "{}", name),
        }
    }
}

/// Agent capabilities
#[derive(Debug, Clone)]
pub struct AgentCapabilities {
    pub max_turns: usize,
    pub can_write_code: bool,
    pub can_run_tests: bool,
    pub can_review: bool,
    pub can_access_network: bool,
}

impl Default for AgentCapabilities {
    fn default() -> Self {
        Self {
            max_turns: 10,
            can_write_code: false,
            can_run_tests: false,
            can_review: false,
            can_access_network: false,
        }
    }
}

impl AgentRole {
    pub fn system_prompt(&self) -> String {
        match self {
            Self::Coder => "You are a senior software engineer. Write clean, efficient, well-documented code. Follow existing project conventions. Make minimal, focused changes.".to_string(),
            Self::Reviewer => "You are a senior code reviewer. Review code for bugs, security issues, performance problems, style violations. Be constructive and specific.".to_string(),
            Self::Tester => "You are a QA engineer. Write comprehensive tests covering happy paths, edge cases, error conditions. Use the project's existing test framework.".to_string(),
            Self::Architect => "You are a software architect. Design clean, scalable architectures. Consider separation of concerns, dependency injection, error handling.".to_string(),
            Self::Planner => "You are a project planner. Break complex tasks into clear, ordered subtasks. Estimate effort, identify dependencies, flag risks.".to_string(),
            Self::Custom(prompt) => prompt.clone(),
        }
    }

    pub fn default_tools(&self) -> Vec<&'static str> {
        match self {
            Self::Coder => vec!["file_read", "file_write", "shell_exec", "search", "patch"],
            Self::Reviewer => vec!["file_read", "search"],
            Self::Tester => vec!["file_read", "shell_exec", "search"],
            Self::Architect => vec!["file_read", "search"],
            Self::Planner => vec![],
            Self::Custom(_) => vec!["file_read", "search"],
        }
    }

    pub fn default_capabilities(&self) -> AgentCapabilities {
        match self {
            Self::Coder => AgentCapabilities {
                max_turns: 15,
                can_write_code: true,
                can_run_tests: true,
                ..Default::default()
            },
            Self::Reviewer => AgentCapabilities {
                max_turns: 5,
                can_review: true,
                ..Default::default()
            },
            Self::Tester => AgentCapabilities {
                max_turns: 10,
                can_write_code: true,
                can_run_tests: true,
                ..Default::default()
            },
            Self::Architect => AgentCapabilities {
                max_turns: 8,
                can_review: true,
                ..Default::default()
            },
            Self::Planner => AgentCapabilities {
                max_turns: 5,
                ..Default::default()
            },
            Self::Custom(_) => AgentCapabilities::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_display() {
        assert_eq!(AgentRole::Coder.to_string(), "coder");
        assert_eq!(AgentRole::Reviewer.to_string(), "reviewer");
        assert_eq!(AgentRole::Custom("writer".into()).to_string(), "writer");
    }

    #[test]
    fn test_role_system_prompt() {
        let prompt = AgentRole::Coder.system_prompt();
        assert!(prompt.contains("software engineer"));
        assert!(!prompt.is_empty());
    }

    #[test]
    fn test_role_capabilities() {
        let caps = AgentRole::Coder.default_capabilities();
        assert!(caps.can_write_code);
        assert!(!caps.can_review);

        let caps = AgentRole::Reviewer.default_capabilities();
        assert!(caps.can_review);
        assert!(!caps.can_write_code);
    }

    #[test]
    fn test_role_tools() {
        let tools = AgentRole::Coder.default_tools();
        assert!(tools.contains(&"file_write"));

        let tools = AgentRole::Reviewer.default_tools();
        assert!(!tools.contains(&"file_write"));
    }
}
