//! Skill trait definition

use serde::{Deserialize, Serialize};

/// Skill trigger condition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SkillTrigger {
    /// User explicit: /skill <name>
    Explicit,
    /// Auto-match based on keywords
    #[serde(rename = "auto_match")]
    AutoMatch {
        keywords: Vec<String>,
        threshold: f32,
    },
}

/// Skill definition (parsed from YAML frontmatter + Markdown body)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDef {
    /// Skill name (unique identifier)
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// Trigger condition
    pub trigger: SkillTrigger,
    /// Instructions injected into system prompt
    pub instructions: String,
    /// Required tools
    #[serde(default)]
    pub required_tools: Vec<String>,
    /// Required MCP servers
    #[serde(default)]
    pub required_mcp: Vec<String>,
    /// File path (for hot-reload)
    #[serde(skip)]
    pub source_path: Option<std::path::PathBuf>,
}
