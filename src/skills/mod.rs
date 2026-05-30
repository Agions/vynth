//! Skills — YAML/MD skill tree

pub mod registry;
pub mod trait_def;
pub mod loader;
pub mod builtin;

pub use registry::SkillRegistry;
pub use trait_def::{SkillDef, SkillTrigger};
