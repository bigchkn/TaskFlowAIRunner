use std::time::Duration;
use std::thread;
use anyhow::{Result, anyhow};
use crate::config;
use crate::taskflow::TaskFlowAdapter;
use crate::git::GitWorktreeManager;
use crate::driver::ProviderDriver;

pub fn execute(once: bool) -> Result<()> {
    let config = config::load_config()?;
    let repo_root = GitWorktreeManager::find_root()?;
    let git_manager = GitWorktreeManager::new(repo_root);
    let taskflow = TaskFlowAdapter::new(config.clone());

    let provider_name = &config.enabled_provider;
    let provider_config = config.providers.get(provider_name)
        .ok_or_else(|| anyhow!("Enabled provider '{}' not found in config", provider_name))?;

    let driver = ProviderDriver::new(
        provider_config.clone(),
        Duration::from_secs(config.default_timeout_seconds),
    );

    println!("Starting dispatch loop (once: {}, provider: {})...", once, provider_name);

    loop {
        match taskflow.next()? {
            Some(task) => {
                println!(">>> Next Task: {} - {}", task.id, task.title);

                // 1. Create worktree
                println!("Creating worktree for task {}...", task.id);
                let worktree_path = git_manager.create_worktree(&task.id, &config.worktree_root)?;

                // 2. Start execution in TaskFlow
                println!("Marking task {} as started...", task.id);
                taskflow.execute_start(&task.id, provider_name)?;

                // 3. Run provider
                println!("Invoking provider {}...", provider_name);
                let prompt = "Run /taskflow and execute the next available task.";
                let result = driver.execute(prompt, &worktree_path);

                let (outcome, log) = match result {
                    Ok(process_result) => {
                        if process_result.success() {
                            println!("Provider completed successfully.");
                            
                            // 4. Validate
                            println!("Running TaskFlow validation...");
                            match taskflow.validate(&task.id) {
                                Ok(val_result) => {
                                    if val_result.success() {
                                        ("success", "Provider completed and validation passed.")
                                    } else {
                                        ("failure", "Provider completed but validation failed.")
                                    }
                                }
                                Err(e) => {
                                    println!("Validation error: {}", e);
                                    ("failure", "Provider completed but validation crashed.")
                                }
                            }
                        } else {
                            println!("Provider failed with status: {:?}", process_result.status);
                            ("failure", "Provider process failed.")
                        }
                    }
                    Err(e) => {
                        println!("Provider execution error: {}", e);
                        ("failure", "Failed to invoke provider.")
                    }
                };

                // 5. Complete task
                println!("Completing task {} with outcome: {}...", task.id, outcome);
                taskflow.execute_complete(&task.id, outcome, log)?;

                // 6. Sync
                taskflow.sync()?;
                
                println!("Task {} processing finished.", task.id);
            }
            None => {
                if once {
                    println!("No more tasks. Exiting.");
                    break;
                }
                println!("No tasks available. Sleeping for 30 seconds...");
                thread::sleep(Duration::from_secs(30));
            }
        }

        if once {
            break;
        }
    }

    Ok(())
}
