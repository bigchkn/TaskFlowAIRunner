use anyhow::{Context, Result};
use std::io::{Read, Write};
use std::process::{Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime};
use wait_timeout::ChildExt;

/// The result of a process execution.
#[derive(Debug)]
#[allow(dead_code)]
pub struct ProcessResult {
    /// Captured stdout.
    pub stdout: String,
    /// Captured stderr.
    pub stderr: String,
    /// Exit status of the process. None if it timed out and was killed.
    pub status: Option<ExitStatus>,
    /// When the process was started.
    pub start_time: SystemTime,
    /// When the process finished (or was killed).
    pub end_time: SystemTime,
    /// Total duration of the execution.
    pub duration: Duration,
    /// Whether the process timed out.
    pub timed_out: bool,
}

impl ProcessResult {
    /// Returns true if the process finished successfully (exit code 0).
    pub fn success(&self) -> bool {
        self.status.is_some_and(|s| s.success())
    }

    /// Returns the exit code if available.
    #[allow(dead_code)]
    pub fn code(&self) -> Option<i32> {
        self.status.and_then(|s| s.code())
    }
}

/// Executes a command with a timeout and captures its output.
#[allow(dead_code)]
pub fn execute_with_timeout(
    mut command: Command,
    timeout: Duration,
    stdin_input: Option<String>,
) -> Result<ProcessResult> {
    let start_instant = Instant::now();
    let start_time = SystemTime::now();

    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());
    if stdin_input.is_some() {
        command.stdin(Stdio::piped());
    }

    let mut child = command
        .spawn()
        .with_context(|| format!("Failed to spawn process: {:?}", command))?;

    #[allow(clippy::collapsible_if)]
    if let Some(input) = stdin_input {
        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(input.as_bytes())
                .context("Failed to write to stdin")?;
            // Explicitly drop stdin to signal EOF
            drop(stdin);
        }
    }

    // Capture stdout and stderr in separate threads to avoid deadlocks
    let mut stdout_pipe = child.stdout.take().context("Failed to capture stdout")?;
    let mut stderr_pipe = child.stderr.take().context("Failed to capture stderr")?;

    let stdout_handle = thread::spawn(move || {
        let mut s = String::new();
        stdout_pipe.read_to_string(&mut s).ok();
        s
    });

    let stderr_handle = thread::spawn(move || {
        let mut s = String::new();
        stderr_pipe.read_to_string(&mut s).ok();
        s
    });

    // Wait for the process to finish or timeout
    let status = match child
        .wait_timeout(timeout)
        .context("Error while waiting for process")?
    {
        Some(status) => Some(status),
        None => {
            // Timeout occurred
            child.kill().ok();
            // Wait for it to be cleaned up
            child.wait().ok();
            None
        }
    };

    let duration = start_instant.elapsed();
    let end_time = SystemTime::now();
    let stdout = stdout_handle.join().unwrap_or_default();
    let stderr = stderr_handle.join().unwrap_or_default();

    Ok(ProcessResult {
        stdout,
        stderr,
        status,
        start_time,
        end_time,
        duration,
        timed_out: status.is_none(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    #[test]
    fn test_execute_success() {
        let mut cmd = Command::new("echo");
        cmd.arg("hello world");

        let result = execute_with_timeout(cmd, Duration::from_secs(5), None).unwrap();

        assert!(result.success());
        assert_eq!(result.stdout.trim(), "hello world");
        assert!(!result.timed_out);
    }

    #[test]
    fn test_execute_failure() {
        let mut cmd = Command::new("ls");
        cmd.arg("/non-existent-directory-12345");

        let result = execute_with_timeout(cmd, Duration::from_secs(5), None).unwrap();

        assert!(!result.success());
        assert!(result.code().unwrap() != 0);
        assert!(!result.timed_out);
    }

    #[test]
    fn test_execute_timeout() {
        // Use 'sleep' which should take longer than our timeout
        let mut cmd = Command::new("sleep");
        cmd.arg("10");

        let result = execute_with_timeout(cmd, Duration::from_millis(100), None).unwrap();

        assert!(result.timed_out);
        assert!(result.status.is_none());
        assert!(result.duration >= Duration::from_millis(100));
    }

    #[test]
    fn test_execute_stdin() {
        let cmd = Command::new("cat");

        let result =
            execute_with_timeout(cmd, Duration::from_secs(5), Some("hello stdin".to_string()))
                .unwrap();

        assert!(result.success());
        assert_eq!(result.stdout.trim(), "hello stdin");
    }
}
