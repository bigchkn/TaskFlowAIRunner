use std::path::Path;
use std::process::Command;
use std::time::Duration;
use anyhow::{Result, Context};
use crate::config::{ProviderConfig, PromptArgMode};
use crate::process::{execute_with_timeout, ProcessResult};

#[allow(dead_code)]
pub struct ProviderDriver {
    config: ProviderConfig,
    default_timeout: Duration,
}

#[allow(dead_code)]
impl ProviderDriver {
    pub fn new(config: ProviderConfig, default_timeout: Duration) -> Self {
        Self {
            config,
            default_timeout,
        }
    }

    pub fn execute(&self, prompt: &str, workdir: &Path) -> Result<ProcessResult> {
        let mut command = Command::new(&self.config.command);
        
        // Set working directory
        command.current_dir(workdir);
        
        // Set environment variables
        for (key, value) in &self.config.env {
            command.env(key, value);
        }
        
        // Add static arguments
        command.args(&self.config.args_before_prompt);
        
        let stdin_input = match self.config.prompt_arg_mode {
            PromptArgMode::Positional => {
                command.arg(prompt);
                None
            }
            PromptArgMode::Stdin => {
                Some(prompt.to_string())
            }
        };

        let timeout = self.config.timeout_seconds
            .map(Duration::from_secs)
            .unwrap_or(self.default_timeout);

        execute_with_timeout(command, timeout, stdin_input)
            .context("Failed to execute provider driver")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tempfile::tempdir;

    #[test]
    fn test_driver_positional() {
        let workdir = tempdir().unwrap();
        let config = ProviderConfig {
            command: "echo".to_string(),
            args_before_prompt: vec!["-n".to_string()],
            prompt_arg_mode: PromptArgMode::Positional,
            env: HashMap::new(),
            timeout_seconds: None,
        };
        
        let driver = ProviderDriver::new(config, Duration::from_secs(5));
        let result = driver.execute("hello world", workdir.path()).unwrap();
        
        assert!(result.success());
        assert_eq!(result.stdout, "hello world");
    }

    #[test]
    fn test_driver_stdin() {
        let workdir = tempdir().unwrap();
        let config = ProviderConfig {
            command: "cat".to_string(),
            args_before_prompt: vec![],
            prompt_arg_mode: PromptArgMode::Stdin,
            env: HashMap::new(),
            timeout_seconds: Some(10),
        };
        
        let driver = ProviderDriver::new(config, Duration::from_secs(5));
        let result = driver.execute("hello stdin", workdir.path()).unwrap();
        
        assert!(result.success());
        assert_eq!(result.stdout, "hello stdin");
    }

    #[test]
    fn test_driver_env() {
        let workdir = tempdir().unwrap();
        let mut env = HashMap::new();
        env.insert("TEST_VAR".to_string(), "test_value".to_string());
        
        let config = ProviderConfig {
            command: "sh".to_string(),
            args_before_prompt: vec!["-c".to_string(), "echo $TEST_VAR".to_string()],
            prompt_arg_mode: PromptArgMode::Stdin, // Not used but required
            env,
            timeout_seconds: None,
        };
        
        let driver = ProviderDriver::new(config, Duration::from_secs(5));
        let result = driver.execute("", workdir.path()).unwrap();
        
        assert!(result.success());
        assert_eq!(result.stdout.trim(), "test_value");
    }

    #[test]
    fn test_driver_multiple_args() {
        let workdir = tempdir().unwrap();
        let config = ProviderConfig {
            command: "printf".to_string(),
            args_before_prompt: vec!["%s %s".to_string(), "hello".to_string()],
            prompt_arg_mode: PromptArgMode::Positional,
            env: HashMap::new(),
            timeout_seconds: None,
        };
        
        let driver = ProviderDriver::new(config, Duration::from_secs(5));
        let result = driver.execute("world", workdir.path()).unwrap();
        
        assert!(result.success());
        assert_eq!(result.stdout, "hello world");
    }
}
