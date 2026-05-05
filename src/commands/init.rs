use crate::config::{Config, PromptArgMode, ProviderConfig, save_config};
use anyhow::{Result, anyhow};
use dialoguer::Select;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use which::which;

pub fn execute() -> Result<()> {
    // 1. Verify presence of .git/ directory
    if !Path::new(".git").exists() {
        return Err(anyhow!(
            "This command must be run from the root of a Git repository."
        ));
    }

    println!("Initializing taskflow-runner...");

    // 2. Update or create .gitignore
    update_gitignore()?;

    // 3. Configure providers
    let mut config = Config::default();

    let known_provider_ids = vec!["gemini", "claude", "dirac", "opencode"];

    println!("Detecting installed providers...");
    let mut detected_providers = Vec::new();
    for id in known_provider_ids {
        if which(id).is_ok() {
            detected_providers.push(id);
        }
    }

    if detected_providers.is_empty() {
        println!("No known providers detected in PATH (gemini, claude, dirac, opencode).");
        println!("You can manually edit ~/.taskflow/taskflow-runner.json to add providers.");
    } else {
        println!("Detected providers: {}", detected_providers.join(", "));

        let mut providers = HashMap::new();
        for id in &detected_providers {
            providers.insert(
                id.to_string(),
                ProviderConfig {
                    command: id.to_string(),
                    args_before_prompt: vec![],
                    prompt_arg_mode: PromptArgMode::Stdin,
                    env: HashMap::new(),
                    timeout_seconds: None,
                },
            );
        }

        detected_providers.sort();
        let default_provider = detected_providers[0].to_string();

        // Optional: Let user override the selection if they want, but default to detected
        let use_detected = if detected_providers.len() > 1 {
            let selection = Select::new()
                .with_prompt("Select default enabled provider")
                .items(&detected_providers)
                .default(0)
                .interact()?;
            detected_providers[selection].to_string()
        } else {
            default_provider
        };

        config.providers = providers;
        config.enabled_provider = use_detected;
    }

    save_config(&config)?;
    println!("Configuration saved successfully.");

    Ok(())
}

fn update_gitignore() -> Result<()> {
    let gitignore_path = Path::new(".gitignore");
    let entry = ".taskflow/worktrees";

    let mut content = if gitignore_path.exists() {
        fs::read_to_string(gitignore_path)?
    } else {
        String::new()
    };

    if !content.lines().any(|l| l.trim() == entry) {
        if !content.is_empty() && !content.ends_with('\n') {
            content.push('\n');
        }
        content.push_str(entry);
        content.push('\n');
        fs::write(gitignore_path, content)?;
        println!("Added {} to .gitignore", entry);
    } else {
        println!("{} already in .gitignore", entry);
    }

    Ok(())
}
