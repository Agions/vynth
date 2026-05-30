/// Result of executing a single step
#[derive(Debug, Clone)]
pub struct StepResult {
    pub step_id: String,
    pub output: String,
    pub status: StepStatus,
    pub duration_ms: u64,
    pub attempts: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepStatus {
    Success,
    Failed(String),
    Skipped,
    TimedOut,
}

/// Overall workflow status
#[derive(Debug, Clone)]
pub struct WorkflowStatus {
    pub total_steps: usize,
    pub completed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub running: usize,
    pub timed_out: usize,
}
