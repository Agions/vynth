//! Skills — YAML/MD skill tree

pub mod builtin;
pub mod registry;
pub mod skill_loader;
pub mod traits;

pub use registry::SkillRegistry;
pub use traits::{SkillDef, SkillTrigger};
