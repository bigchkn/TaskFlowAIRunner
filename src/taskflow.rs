use crate::config::Config;
use crate::process::{ProcessResult, execute_with_timeout};
use anyhow::{Context, Result, anyhow};
use std::process::Command;
use std::time::Duration;

#[derive(Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub struct TaskInfo {
    pub id: String,
    pub title: String,
    pub milestone: String,
    pub status: String,
    pub priority: String,
}

#[allow(dead_code)]
pub struct TaskFlowAdapter {
    config: Config,
}

#[allow(dead_code)]
impl TaskFlowAdapter {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    fn taskflow_cmd(&self) -> Command {
        Command::new(&self.config.taskflow_command)
    }

    pub fn next(&self) -> Result<Option<TaskInfo>> {
        let mut cmd = self.taskflow_cmd();
        cmd.arg("next");

        let result = execute_with_timeout(cmd, Duration::from_secs(30), None)?;

        if !result.success() {
            return Err(anyhow!("taskflow-ai next failed: {}", result.stderr));
        }

        parse_next_output(&result.stdout)
    }

    pub fn execute_start(&self, task_id: &str, agent: &str) -> Result<()> {
        let mut cmd = self.taskflow_cmd();
        cmd.arg("execute")
            .arg("start")
            .arg(task_id)
            .arg("--agent")
            .arg(agent);

        let result = execute_with_timeout(cmd, Duration::from_secs(30), None)?;

        if !result.success() {
            return Err(anyhow!(
                "taskflow-ai execute start failed: {}",
                result.stderr
            ));
        }

        Ok(())
    }

    pub fn validate(&self, task_id: &str) -> Result<ProcessResult> {
        let mut cmd = self.taskflow_cmd();
        cmd.arg("validate").arg(task_id);

        execute_with_timeout(cmd, Duration::from_secs(60), None)
            .context("Failed to run taskflow-ai validate")
    }

    pub fn execute_complete(&self, task_id: &str, outcome: &str, log: &str) -> Result<()> {
        let mut cmd = self.taskflow_cmd();
        cmd.arg("execute")
            .arg("complete")
            .arg(task_id)
            .arg("--outcome")
            .arg(outcome)
            .arg("--log")
            .arg(log);

        let result = execute_with_timeout(cmd, Duration::from_secs(30), None)?;

        if !result.success() {
            return Err(anyhow!(
                "taskflow-ai execute complete failed: {}",
                result.stderr
            ));
        }

        Ok(())
    }

    pub fn sync(&self) -> Result<()> {
        let mut cmd = self.taskflow_cmd();
        cmd.arg("sync");

        let result = execute_with_timeout(cmd, Duration::from_secs(30), None)?;

        if !result.success() {
            return Err(anyhow!("taskflow-ai sync failed: {}", result.stderr));
        }

        Ok(())
    }
}

fn parse_next_output(stdout: &str) -> Result<Option<TaskInfo>> {
    if stdout.contains("No pending tasks found") || stdout.trim().is_empty() {
        return Ok(None);
    }

    let mut id = String::new();
    let mut title = String::new();
    let mut milestone = String::new();
    let mut status = String::new();
    let mut priority = String::new();

    for line in stdout.lines() {
        let line = line.trim();
        if let Some(content) = line.strip_prefix(">>> Next Task:") {
            let content = content.trim();
            // Look for the first space after the ID or just split by '-'
            if let Some(dash_idx) = content.find(" - ") {
                id = content[..dash_idx].trim().to_string();
                title = content[dash_idx + 3..].trim().to_string();
            } else {
                // Fallback for unexpected format
                id = content.to_string();
            }
        } else if let Some(content) = line.strip_prefix("Milestone:") {
            milestone = content.trim().to_string();
        } else if let Some(content) = line.strip_prefix("Status:") {
            status = content.trim().to_string();
        } else if let Some(content) = line.strip_prefix("Priority:") {
            priority = content.trim().to_string();
        }
    }

    if id.is_empty() {
        Ok(None)
    } else {
        Ok(Some(TaskInfo {
            id,
            title,
            milestone,
            status,
            priority,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_next_output() {
        let output = r#">>> Next Task: TF-8 - Implement the TaskFlow adapter for next/start/validate/complete/sync
Milestone: Implement initial version of runner
Status:    Backlog
Priority:  0

Relevant Designs:
  - [Parent] [Hld] docs/designs/M1/TF-1/hld-taskflow-runner-initial-architecture.md (`Approved`)

Suggested Action:
  taskflow-ai execute start TF-8"#;

        let task = parse_next_output(output)
            .unwrap()
            .expect("Should find a task");
        assert_eq!(task.id, "TF-8");
        assert_eq!(
            task.title,
            "Implement the TaskFlow adapter for next/start/validate/complete/sync"
        );
        assert_eq!(task.milestone, "Implement initial version of runner");
        assert_eq!(task.status, "Backlog");
        assert_eq!(task.priority, "0");
    }

    #[test]
    fn test_parse_next_output_none() {
        let output = "No pending tasks found.";
        let task = parse_next_output(output).unwrap();
        assert!(task.is_none());
    }

    #[test]
    fn test_parse_next_output_missing_fields() {
        let output = r#">>> Next Task: TF-12 - Add tests
Priority:  Low"#;

        let task = parse_next_output(output)
            .unwrap()
            .expect("Should find a task");
        assert_eq!(task.id, "TF-12");
        assert_eq!(task.title, "Add tests");
        assert_eq!(task.priority, "Low");
        assert_eq!(task.milestone, "");
        assert_eq!(task.status, "");
    }

    #[test]
    fn test_parse_next_output_unexpected_format() {
        let output = "Just some random text that doesn't match.";
        let task = parse_next_output(output).unwrap();
        assert!(task.is_none());
    }
}
