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
