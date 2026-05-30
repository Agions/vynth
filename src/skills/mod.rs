//! Skills — YAML/MD skill tree

pub mod builtin;
pub mod external;
pub mod loader;
pub mod registry;
pub mod trait_def;

pub use external::{load_external_skills, SkillSource, SkillSourceType};
pub use registry::SkillRegistry;
pub use trait_def::{SkillDef, SkillTrigger};
