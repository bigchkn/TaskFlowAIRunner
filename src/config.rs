use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::fs;
use anyhow::{Result, Context, anyhow};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum PromptArgMode {
    #[serde(rename = "positional")]
    Positional,
    #[serde(rename = "stdin")]
    Stdin,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProviderConfig {
    pub command: String,
    pub args_before_prompt: Vec<String>,
    pub prompt_arg_mode: PromptArgMode,
    pub env: HashMap<String, String>,
    pub timeout_seconds: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub providers: HashMap<String, ProviderConfig>,
    pub enabled_provider: String,
    pub default_timeout_seconds: u64,
    pub taskflow_command: String,
    pub worktree_root: String,
}

impl Default for Config {
    fn default() -> Self {
        let mut providers = HashMap::new();
        
        // Add some default providers
        providers.insert("gemini".to_string(), ProviderConfig {
            command: "gemini".to_string(),
            args_before_prompt: vec![],
            prompt_arg_mode: PromptArgMode::Stdin,
            env: HashMap::new(),
            timeout_seconds: None,
        });

        Config {
            providers,
            enabled_provider: "gemini".to_string(),
            default_timeout_seconds: 3600,
            taskflow_command: "taskflow-ai".to_string(),
            worktree_root: ".taskflow/worktrees".to_string(),
        }
    }
}

pub fn get_config_path() -> Result<PathBuf> {
    let home = directories::UserDirs::new()
        .context("Could not find home directory")?;
    let config_dir = home.home_dir().join(".taskflow");
    Ok(config_dir.join("taskflow-runner.json"))
}

pub fn load_config() -> Result<Config> {
    let path = get_config_path()?;
    if !path.exists() {
        return Err(anyhow!("Config file not found at {:?}. Run 'init' first.", path));
    }
    let content = fs::read_to_string(path)?;
    let config: Config = serde_json::from_str(&content)?;
    Ok(config)
}

pub fn save_config(config: &Config) -> Result<()> {
    let path = get_config_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(config)?;
    fs::write(path, content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.enabled_provider, "gemini");
        assert_eq!(config.default_timeout_seconds, 3600);
        assert!(config.providers.contains_key("gemini"));
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: Config = serde_json::from_str(&json).unwrap();
        
        assert_eq!(config.enabled_provider, deserialized.enabled_provider);
        assert_eq!(config.default_timeout_seconds, deserialized.default_timeout_seconds);
        assert_eq!(config.taskflow_command, deserialized.taskflow_command);
        assert_eq!(config.worktree_root, deserialized.worktree_root);
        assert_eq!(config.providers.len(), deserialized.providers.len());
    }

    #[test]
    fn test_provider_config_serialization() {
        let json = r#"{
            "command": "test-cmd",
            "args_before_prompt": ["--flag"],
            "prompt_arg_mode": "positional",
            "env": { "KEY": "VALUE" },
            "timeout_seconds": 120
        }"#;
        
        let config: ProviderConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.command, "test-cmd");
        assert_eq!(config.args_before_prompt, vec!["--flag"]);
        match config.prompt_arg_mode {
            PromptArgMode::Positional => (),
            _ => panic!("Wrong prompt_arg_mode"),
        }
        assert_eq!(config.env.get("KEY").unwrap(), "VALUE");
        assert_eq!(config.timeout_seconds, Some(120));
    }
}
