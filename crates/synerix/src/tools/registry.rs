//! Tool registry — HashMap<name, Arc<dyn Tool>>

use std::collections::HashMap;
use std::sync::Arc;

use crate::llm::types::ToolSchema;
use crate::tools::traits::Tool;

/// Tool registry with thread-local tools
pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn Tool>>,
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
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

    /// Find tool by name (O(1) lookup)
    pub fn get(&self, name: &str) -> Option<Arc<dyn Tool>> {
        self.tools.get(name).cloned()
    }

    /// Export all tool schemas for LLM (built fresh each call)
    pub fn all_schemas(&self) -> Vec<ToolSchema> {
        self.tools
            .values()
            .map(|tool| {
                let schema = tool.schema();
                ToolSchema {
                    schema_type: "function".to_string(),
                    function: crate::llm::types::FunctionSchema {
                        name: schema["name"].as_str().unwrap_or(tool.name()).to_string(),
                        description: schema["description"].as_str().unwrap_or("").to_string(),
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

    /// Number of registered tools
    pub fn len(&self) -> usize {
        self.tools.len()
    }

    /// Check if registry is empty
    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::traits::{ToolContext, ToolResult};
    use async_trait::async_trait;

    struct MockTool {
        name: String,
    }

    #[async_trait]
    impl Tool for MockTool {
        fn name(&self) -> &str {
            &self.name
        }

        fn schema(&self) -> serde_json::Value {
            serde_json::json!({
                "name": self.name,
                "description": "A mock tool",
                "parameters": {}
            })
        }

        async fn execute(
            &self,
            _args: serde_json::Value,
            _ctx: &ToolContext,
        ) -> Result<ToolResult, crate::error::AppError> {
            Ok(ToolResult {
                output: "ok".to_string(),
                is_error: false,
                preview: None,
            })
        }
    }

    #[test]
    fn test_schema_caching() {
        let mut registry = ToolRegistry::new();
        registry.register(Arc::new(MockTool {
            name: "test_tool".to_string(),
        }));

        let schemas1 = registry.all_schemas();
        assert_eq!(schemas1.len(), 1);

        // Second call returns identical results
        let schemas2 = registry.all_schemas();
        assert_eq!(schemas1, schemas2);
    }

    #[test]
    fn test_register_updates_schemas() {
        let mut registry = ToolRegistry::new();
        registry.register(Arc::new(MockTool {
            name: "tool1".to_string(),
        }));

        let schemas1 = registry.all_schemas();
        assert_eq!(schemas1.len(), 1);

        // Register another tool: schemas should be up-to-date
        registry.register(Arc::new(MockTool {
            name: "tool2".to_string(),
        }));

        let schemas2 = registry.all_schemas();
        assert_eq!(schemas2.len(), 2);
    }

    #[test]
    fn test_immutable_access() {
        let mut registry = ToolRegistry::new();
        registry.register(Arc::new(MockTool {
            name: "test".to_string(),
        }));

        // all_schemas works with &self (no mut needed)
        let schemas = registry.all_schemas();
        assert_eq!(schemas.len(), 1);

        // Can still use &self methods concurrently
        let names = registry.list_names();
        assert_eq!(names, vec!["test"]);
    }
}
