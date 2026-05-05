use std::path::PathBuf;
use std::process::Command;
use anyhow::{Result, Context, anyhow};
use crate::process::execute_with_timeout;
use std::time::Duration;

#[allow(dead_code)]
pub struct GitWorktreeManager {
    repo_root: PathBuf,
}

#[allow(dead_code)]
impl GitWorktreeManager {
    /// Resolves the git repository root.
    pub fn find_root() -> Result<PathBuf> {
        let mut cmd = Command::new("git");
        cmd.arg("rev-parse").arg("--show-toplevel");
        
        let result = execute_with_timeout(cmd, Duration::from_secs(5), None)
            .context("Failed to run git rev-parse")?;

        if !result.success() {
            return Err(anyhow!("Not in a git repository: {}", result.stderr));
        }

        Ok(PathBuf::from(result.stdout.trim()))
    }

    pub fn new(repo_root: PathBuf) -> Self {
        Self { repo_root }
    }

    pub fn get_worktree_path(&self, task_id: &str, worktree_root_config: &str) -> PathBuf {
        self.repo_root.join(worktree_root_config).join(task_id)
    }

    /// Gets the current branch name.
    pub fn get_current_branch(&self) -> Result<String> {
        let mut cmd = Command::new("git");
        cmd.current_dir(&self.repo_root)
            .arg("rev-parse")
            .arg("--abbrev-ref")
            .arg("HEAD");

        let result = execute_with_timeout(cmd, Duration::from_secs(5), None)
            .context("Failed to get current branch")?;

        if !result.success() {
            return Err(anyhow!("Failed to get current branch: {}", result.stderr));
        }

        Ok(result.stdout.trim().to_string())
    }

    /// Creates a new worktree for a task.
    /// Returns the absolute path to the worktree.
    pub fn create_worktree(&self, task_id: &str, worktree_root_config: &str) -> Result<PathBuf> {
        let branch_name = format!("taskflow/{}", task_id);
        let worktree_path = self.get_worktree_path(task_id, worktree_root_config);
        let worktree_root = worktree_path.parent().unwrap();

        if worktree_path.exists() {
             return Err(anyhow!("Worktree path already exists: {:?}", worktree_path));
        }

        // Ensure worktree_root exists
        std::fs::create_dir_all(worktree_root).context("Failed to create worktree root directory")?;

        // 1. Create branch if it doesn't exist
        // We use 'git branch' to create it from HEAD if it doesn't exist.
        let mut cmd = Command::new("git");
        cmd.current_dir(&self.repo_root)
            .arg("branch")
            .arg(&branch_name);
        
        // We ignore failure here (e.g. branch already exists).
        let _ = execute_with_timeout(cmd, Duration::from_secs(5), None);

        // 2. Add worktree
        let mut cmd = Command::new("git");
        cmd.current_dir(&self.repo_root)
            .arg("worktree")
            .arg("add")
            .arg(&worktree_path)
            .arg(&branch_name);

        let result = execute_with_timeout(cmd, Duration::from_secs(10), None)
            .context("Failed to add git worktree")?;

        if !result.success() {
            return Err(anyhow!("Failed to add git worktree: {}", result.stderr));
        }

        Ok(worktree_path)
    }

    /// Removes a worktree.
    pub fn remove_worktree(&self, task_id: &str, worktree_root_config: &str) -> Result<()> {
        let worktree_path = self.get_worktree_path(task_id, worktree_root_config);
        
        if !worktree_path.exists() {
            return Ok(());
        }

        let mut cmd = Command::new("git");
        cmd.current_dir(&self.repo_root)
            .arg("worktree")
            .arg("remove")
            .arg("--force")
            .arg(&worktree_path);

        let result = execute_with_timeout(cmd, Duration::from_secs(10), None)
            .context("Failed to remove git worktree")?;

        if !result.success() {
             return Err(anyhow!("Failed to remove git worktree: {}", result.stderr));
        }

        // Also try to remove the branch if requested? 
        // The design says to leave it intact for review, so we probably shouldn't remove the branch here.
        // But removing the worktree is sometimes needed if we want to restart.

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;
    use std::path::Path;

    fn setup_git_repo(path: &Path) {
        let run = |args: &[&str]| {
            let status = Command::new("git")
                .current_dir(path)
                .args(args)
                .status()
                .unwrap();
            assert!(status.success());
        };

        run(&["init"]);
        run(&["config", "user.email", "you@example.com"]);
        run(&["config", "user.name", "Your Name"]);
        fs::write(path.join("file.txt"), "hello").unwrap();
        run(&["add", "."]);
        run(&["commit", "-m", "initial commit"]);
    }

    #[test]
    fn test_git_worktree_lifecycle() {
        let dir = tempdir().unwrap();
        let repo_path = dir.path().join("repo");
        fs::create_dir(&repo_path).unwrap();
        
        setup_git_repo(&repo_path);
        
        let manager = GitWorktreeManager::new(repo_path.clone());
        
        let worktree_root = ".taskflow/worktrees";
        let task_id = "TF-TEST";
        
        let wt_path = manager.create_worktree(task_id, worktree_root).expect("Should create worktree");
        assert!(wt_path.exists());
        assert!(wt_path.to_string_lossy().contains(task_id));
        
        // Verify it's a git worktree
        assert!(wt_path.join(".git").exists());
        
        manager.remove_worktree(task_id, worktree_root).expect("Should remove worktree");
        assert!(!wt_path.exists());
    }

    #[test]
    fn test_worktree_path_derivation() {
        let repo_path = PathBuf::from("/tmp/repo");
        let manager = GitWorktreeManager::new(repo_path.clone());
        
        let task_id = "TF-1";
        let worktree_root = "custom/wt";
        
        let derived_path = manager.get_worktree_path(task_id, worktree_root);
        let expected_path = repo_path.join(worktree_root).join(task_id);
        
        assert_eq!(derived_path, expected_path);
        assert_eq!(derived_path.to_str().unwrap(), "/tmp/repo/custom/wt/TF-1");
    }
}
