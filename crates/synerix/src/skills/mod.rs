//! Skills — YAML/MD skill tree

pub mod builtin;
pub mod external;
pub mod registry;
pub mod skill_loader;
pub mod traits;

pub use external::{load_external_skills, SkillSource, SkillSourceType};
pub use registry::SkillRegistry;
pub use traits::{SkillDef, SkillTrigger};
