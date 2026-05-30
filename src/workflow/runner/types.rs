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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn step_status_equality() {
        assert_eq!(StepStatus::Success, StepStatus::Success);
        assert_eq!(StepStatus::Skipped, StepStatus::Skipped);
        assert_eq!(StepStatus::TimedOut, StepStatus::TimedOut);
        assert_eq!(
            StepStatus::Failed("x".into()),
            StepStatus::Failed("x".into())
        );
        assert_ne!(StepStatus::Success, StepStatus::Failed("x".into()));
        assert_ne!(StepStatus::Skipped, StepStatus::TimedOut);
    }

    #[test]
    fn step_status_debug_format() {
        assert_eq!(format!("{:?}", StepStatus::Success), "Success");
        assert_eq!(
            format!("{:?}", StepStatus::Failed("err".into())),
            "Failed(\"err\")"
        );
        assert_eq!(format!("{:?}", StepStatus::Skipped), "Skipped");
        assert_eq!(format!("{:?}", StepStatus::TimedOut), "TimedOut");
    }

    #[test]
    fn step_result_construction() {
        let result = StepResult {
            step_id: "step1".to_string(),
            output: "ok".to_string(),
            status: StepStatus::Success,
            duration_ms: 150,
            attempts: 1,
        };
        assert_eq!(result.step_id, "step1");
        assert_eq!(result.output, "ok");
        assert_eq!(result.duration_ms, 150);
        assert_eq!(result.attempts, 1);
        assert!(matches!(result.status, StepStatus::Success));
    }

    #[test]
    fn step_result_clone() {
        let result = StepResult {
            step_id: "s1".to_string(),
            output: "out".to_string(),
            status: StepStatus::Failed("oops".into()),
            duration_ms: 50,
            attempts: 3,
        };
        let cloned = result.clone();
        assert_eq!(cloned.step_id, result.step_id);
        assert_eq!(cloned.attempts, 3);
    }

    #[test]
    fn workflow_status_construction() {
        let ws = WorkflowStatus {
            total_steps: 5,
            completed: 3,
            failed: 1,
            skipped: 0,
            running: 1,
            timed_out: 0,
        };
        assert_eq!(ws.total_steps, 5);
        assert_eq!(ws.completed, 3);
        assert_eq!(ws.failed, 1);
        assert_eq!(ws.running, 1);
    }

    #[test]
    fn workflow_status_debug() {
        let ws = WorkflowStatus {
            total_steps: 2,
            completed: 2,
            failed: 0,
            skipped: 0,
            running: 0,
            timed_out: 0,
        };
        let dbg = format!("{:?}", ws);
        assert!(dbg.contains("total_steps: 2"));
        assert!(dbg.contains("completed: 2"));
    }
}
