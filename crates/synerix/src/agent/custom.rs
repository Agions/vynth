//! Custom agent definitions — load from YAML/TOML files
//!
//! Users can define custom agent roles in `~/.config/synerix/agents/*.yaml`:
//!
//! ```yaml
//! name: security-auditor
//! description: Security-focused code auditor
//! system_prompt: |
//!   You are a security auditor. Focus on:
//!   - SQL injection, XSS, CSRF vulnerabilities
//!   - Authentication/authorization flaws
//!   - Secrets in code
//!   - Dependency vulnerabilities
//! tools:
//!   - file_read
//!   - search
//! max_turns: 10
//! capabilities:
//!   can_review: true
//!   can_write_code: false
//! ```
// TODO: Custom agent defs — not yet wired
#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::agent::roles::{AgentCapabilities, AgentRole};
use crate::error::AppError;

/// Custom agent definition loaded from file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomAgentDef {
    /// Agent name (unique identifier)
    pub name: String,
    /// Human-readable description
    #[serde(default)]
    pub description: String,
    /// System prompt for this agent
    pub system_prompt: String,
    /// Allowed tools (empty = all tools)
    #[serde(default)]
    pub tools: Vec<String>,
    /// Max reasoning turns
    #[serde(default = "default_max_turns")]
    pub max_turns: usize,
    /// Capability flags
    #[serde(default)]
    pub capabilities: CustomCapabilities,
    /// Environment variables to set when this agent runs
    #[serde(default)]
    pub env: HashMap<String, String>,
    /// Tags for matching/filtering
    #[serde(default)]
    pub tags: Vec<String>,
}

fn default_max_turns() -> usize {
    10
}

/// Capability flags for custom agents
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CustomCapabilities {
    #[serde(default)]
    pub can_write_code: bool,
    #[serde(default)]
    pub can_run_tests: bool,
    #[serde(default)]
    pub can_review: bool,
    #[serde(default)]
    pub can_access_network: bool,
}

impl CustomAgentDef {
    /// Convert to AgentRole::Custom with the system prompt
    pub fn to_role(&self) -> AgentRole {
        AgentRole::Custom(self.name.clone())
    }

    /// Convert capabilities to AgentCapabilities
    pub fn to_agent_capabilities(&self) -> AgentCapabilities {
        AgentCapabilities {
            max_turns: self.max_turns,
            can_write_code: self.capabilities.can_write_code,
            can_run_tests: self.capabilities.can_run_tests,
            can_review: self.capabilities.can_review,
            can_access_network: self.capabilities.can_access_network,
        }
    }
}

/// Registry of custom agent definitions
pub struct CustomAgentRegistry {
    agents: HashMap<String, CustomAgentDef>,
}

impl Default for CustomAgentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl CustomAgentRegistry {
    /// Create an empty registry
    pub fn new() -> Self {
        Self {
            agents: HashMap::new(),
        }
    }

    /// Load all custom agents from a directory
    pub async fn load_from_dir(path: &Path) -> Result<Self, AppError> {
        let mut registry = Self::new();

        if !path.exists() {
            tracing::info!("Agents directory not found: {}", path.display());
            return Ok(registry);
        }

        for entry in crate::util::walk_dir(path).await {
            let ext = entry.extension().and_then(|e| e.to_str()).unwrap_or("");

            if ext == "yaml" || ext == "yml" || ext == "toml" {
                match load_agent_file(&entry).await {
                    Ok(agent) => {
                        tracing::info!("Loaded custom agent: {}", agent.name);
                        registry.agents.insert(agent.name.clone(), agent);
                    }
                    Err(e) => {
                        tracing::warn!("Failed to load agent from {}: {}", entry.display(), e);
                    }
                }
            }
        }

        Ok(registry)
    }

    /// Get a custom agent by name
    pub fn get(&self, name: &str) -> Option<&CustomAgentDef> {
        self.agents.get(name)
    }

    /// List all custom agent names
    pub fn list_names(&self) -> Vec<&str> {
        self.agents.keys().map(|s| s.as_str()).collect()
    }

    /// List all custom agents
    pub fn all(&self) -> Vec<&CustomAgentDef> {
        self.agents.values().collect()
    }

    /// Find agents by tag
    pub fn find_by_tag(&self, tag: &str) -> Vec<&CustomAgentDef> {
        self.agents
            .values()
            .filter(|a| a.tags.iter().any(|t| t == tag))
            .collect()
    }

    /// Number of loaded agents
    pub fn len(&self) -> usize {
        self.agents.len()
    }

    /// Check if registry is empty
    pub fn is_empty(&self) -> bool {
        self.agents.is_empty()
    }
}

/// Load a single agent definition from a YAML or TOML file
async fn load_agent_file(path: &Path) -> Result<CustomAgentDef, AppError> {
    let content = tokio::fs::read_to_string(path)
        .await
        .map_err(|e| AppError::Config(format!("Failed to read agent file: {}", e)))?;

    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

    match ext {
        "yaml" | "yml" => serde_yaml::from_str(&content)
            .map_err(|e| AppError::Config(format!("Invalid agent YAML: {} ", e))),
        "toml" => toml::from_str(&content)
            .map_err(|e| AppError::Config(format!("Invalid agent TOML: {}", e))),
        _ => Err(AppError::Config(format!(
            "Unsupported agent file format: {}",
            ext
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_agent_def_yaml() {
        let yaml = r#"
name: security-auditor
description: Security-focused code auditor
system_prompt: |
  You are a security auditor. Focus on vulnerabilities.
tools:
  - file_read
  - search
max_turns: 8
capabilities:
  can_review: true
  can_write_code: false
tags:
  - security
  - audit
"#;
        let agent: CustomAgentDef = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(agent.name, "security-auditor");
        assert_eq!(agent.tools, vec!["file_read", "search"]);
        assert_eq!(agent.max_turns, 8);
        assert!(agent.capabilities.can_review);
        assert!(!agent.capabilities.can_write_code);
        assert!(agent.tags.contains(&"security".to_string()));
    }

    #[test]
    fn test_custom_agent_def_toml() {
        let toml = r#"
name = "doc-writer"
description = "Documentation writer"
system_prompt = "You write clear, concise documentation."
tools = ["file_read", "file_write"]
max_turns = 5
tags = ["docs"]

[capabilities]
can_write_code = true
"#;
        let agent: CustomAgentDef = toml::from_str(toml).unwrap();
        assert_eq!(agent.name, "doc-writer");
        assert!(agent.capabilities.can_write_code);
    }

    #[test]
    fn test_custom_agent_to_role() {
        let agent = CustomAgentDef {
            name: "test-agent".to_string(),
            description: "Test".to_string(),
            system_prompt: "Test prompt".to_string(),
            tools: vec![],
            max_turns: 5,
            capabilities: CustomCapabilities::default(),
            env: HashMap::new(),
            tags: vec![],
        };

        let role = agent.to_role();
        assert_eq!(role.to_string(), "test-agent");

        let caps = agent.to_agent_capabilities();
        assert_eq!(caps.max_turns, 5);
    }

    #[test]
    fn test_custom_agent_registry() {
        let mut registry = CustomAgentRegistry::new();
        assert!(registry.is_empty());

        let agent = CustomAgentDef {
            name: "test".to_string(),
            description: "Test".to_string(),
            system_prompt: "Test".to_string(),
            tools: vec![],
            max_turns: 5,
            capabilities: CustomCapabilities::default(),
            env: HashMap::new(),
            tags: vec!["security".to_string()],
        };
        registry.agents.insert(agent.name.clone(), agent);

        assert_eq!(registry.len(), 1);
        assert!(registry.get("test").is_some());
        assert_eq!(registry.list_names(), vec!["test"]);
        assert_eq!(registry.find_by_tag("security").len(), 1);
        assert_eq!(registry.find_by_tag("docs").len(), 0);
    }
}
