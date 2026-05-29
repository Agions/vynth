//! Tool registry — HashMap<name, Arc<dyn Tool>>

use std::collections::HashMap;
use std::sync::Arc;

use crate::tools::trait_def::Tool;
use crate::llm::types::ToolSchema;

/// Tool registry
pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    /// Register a tool (called once at startup)
    pub fn register(&mut self, tool: Arc<dyn Tool>) {
        self.tools.insert(tool.name().to_string(), tool);
    }

    /// Find tool by name
    pub fn get(&self, name: &str) -> Option<Arc<dyn Tool>> {
        self.tools.get(name).cloned()
    }

    /// Export all tool schemas for LLM
    pub fn all_schemas(&self) -> Vec<ToolSchema> {
        self.tools
            .values()
            .map(|tool| {
                let schema = tool.schema();
                ToolSchema {
                    schema_type: "function".to_string(),
                    function: crate::llm::types::FunctionSchema {
                        name: schema["name"].as_str().unwrap_or(tool.name()).to_string(),
                        description: schema["description"]
                            .as_str()
                            .unwrap_or("")
                            .to_string(),
                        parameters: schema["parameters"].clone(),
                    },
                }
            })
            .collect()
    }

    /// List all registered tool names
    pub fn list_names(&self) -> Vec<&str> {
        self.tools.keys().map(|s| s.as_str()).collect()
    }
}
