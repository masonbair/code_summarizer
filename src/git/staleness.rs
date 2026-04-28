use anyhow::Result;
use std::fs;
use std::path::Path;
use std::time::SystemTime;

use super::GitClient;

/// Result of staleness check
#[derive(Debug, Clone)]
pub struct StalenessResult {
    pub is_stale: bool,
    pub summary_time: Option<SystemTime>,
    pub latest_commit_time: Option<SystemTime>,
    pub commits_since: usize,
    pub reason: Option<String>,
}

impl GitClient {
    /// Check if a summary file is stale compared to git history
    pub fn is_summary_stale(&self, summary_path: &Path) -> Result<StalenessResult> {
        // Get summary file modification time
        let summary_time = match fs::metadata(summary_path) {
            Ok(meta) => meta.modified().ok(),
            Err(_) => {
                return Ok(StalenessResult {
                    is_stale: true,
                    summary_time: None,
                    latest_commit_time: None,
                    commits_since: 0,
                    reason: Some("Summary file does not exist".to_string()),
                });
            }
        };

        // Get latest commit time
        let head = self.repo.head()?;
        let commit = head.peel_to_commit()?;
        let commit_time = commit.time();

        // Convert git time to SystemTime
        let latest_commit_time = SystemTime::UNIX_EPOCH
            + std::time::Duration::from_secs(commit_time.seconds() as u64);

        // Compare times
        let is_stale = match summary_time {
            Some(st) => latest_commit_time > st,
            None => true,
        };

        // Count commits since summary was generated
        let commits_since = if is_stale {
            self.count_commits_since(summary_time)?
        } else {
            0
        };

        let reason = if is_stale {
            Some(format!(
                "{} commits since summary was generated",
                commits_since
            ))
        } else {
            None
        };

        Ok(StalenessResult {
            is_stale,
            summary_time,
            latest_commit_time: Some(latest_commit_time),
            commits_since,
            reason,
        })
    }

    /// Count commits since a given time
    fn count_commits_since(&self, since: Option<SystemTime>) -> Result<usize> {
        let since_epoch = match since {
            Some(st) => st
                .duration_since(SystemTime::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0),
            None => return self.get_total_commits(),
        };

        let mut revwalk = self.repo.revwalk()?;
        revwalk.push_head()?;
        revwalk.set_sorting(git2::Sort::TIME)?;

        let mut count = 0;
        for oid in revwalk {
            let oid = oid?;
            let commit = self.repo.find_commit(oid)?;
            if commit.time().seconds() > since_epoch {
                count += 1;
            } else {
                break;
            }
        }

        Ok(count)
    }

    /// Check if any tracked file has been modified since a given time
    pub fn has_changes_since(&self, since: SystemTime) -> Result<bool> {
        let since_epoch = since
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        let head = self.repo.head()?;
        let commit = head.peel_to_commit()?;

        Ok(commit.time().seconds() > since_epoch)
    }

    /// Get the timestamp of the most recent commit
    pub fn get_latest_commit_time(&self) -> Result<SystemTime> {
        let head = self.repo.head()?;
        let commit = head.peel_to_commit()?;
        let commit_time = commit.time();

        Ok(SystemTime::UNIX_EPOCH
            + std::time::Duration::from_secs(commit_time.seconds() as u64))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;
    use std::thread;
    use std::time::Duration;
    use tempfile::TempDir;

    fn create_git_repo() -> TempDir {
        let temp = TempDir::new().unwrap();
        let root = temp.path();

        Command::new("git")
            .args(["init"])
            .current_dir(root)
            .output()
            .unwrap();

        Command::new("git")
            .args(["config", "user.email", "test@test.com"])
            .current_dir(root)
            .output()
            .unwrap();

        Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(root)
            .output()
            .unwrap();

        fs::write(root.join("test.txt"), "hello").unwrap();

        Command::new("git")
            .args(["add", "."])
            .current_dir(root)
            .output()
            .unwrap();

        Command::new("git")
            .args(["commit", "-m", "Initial"])
            .current_dir(root)
            .output()
            .unwrap();

        temp
    }

    #[test]
    fn test_is_summary_stale_missing_file() {
        let temp = create_git_repo();
        let client = GitClient::open(temp.path()).unwrap();

        let result = client
            .is_summary_stale(&temp.path().join("nonexistent.md"))
            .unwrap();

        assert!(result.is_stale);
        assert!(result.summary_time.is_none());
        assert!(result.reason.unwrap().contains("does not exist"));
    }

    #[test]
    fn test_is_summary_stale_fresh_summary() {
        let temp = create_git_repo();
        let client = GitClient::open(temp.path()).unwrap();

        // Wait a moment to ensure time difference
        thread::sleep(Duration::from_millis(100));

        // Create summary file after the commit
        let summary_path = temp.path().join("summary.md");
        fs::write(&summary_path, "# Summary").unwrap();

        let result = client.is_summary_stale(&summary_path).unwrap();

        assert!(!result.is_stale);
        assert!(result.summary_time.is_some());
        assert_eq!(result.commits_since, 0);
    }

    #[test]
    fn test_is_summary_stale_old_summary() {
        let temp = create_git_repo();

        // Create summary file before new commits
        let summary_path = temp.path().join("summary.md");
        fs::write(&summary_path, "# Summary").unwrap();

        // Wait to ensure timestamp difference (git uses second-level precision)
        thread::sleep(Duration::from_secs(2));

        // Make a new commit
        fs::write(temp.path().join("new_file.txt"), "new content").unwrap();
        Command::new("git")
            .args(["add", "."])
            .current_dir(temp.path())
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "New commit"])
            .current_dir(temp.path())
            .output()
            .unwrap();

        let client = GitClient::open(temp.path()).unwrap();
        let result = client.is_summary_stale(&summary_path).unwrap();

        assert!(result.is_stale);
        assert!(result.commits_since >= 1);
    }

    #[test]
    fn test_get_latest_commit_time() {
        let temp = create_git_repo();
        let client = GitClient::open(temp.path()).unwrap();

        let time = client.get_latest_commit_time().unwrap();
        let now = SystemTime::now();

        // Commit time should be in the past (or very close to now)
        assert!(time <= now);

        // Should be recent (within last minute for test)
        let age = now.duration_since(time).unwrap();
        assert!(age.as_secs() < 60);
    }

    #[test]
    fn test_has_changes_since_past() {
        let temp = create_git_repo();
        let client = GitClient::open(temp.path()).unwrap();

        // Check from a time in the past
        let past = SystemTime::UNIX_EPOCH;
        assert!(client.has_changes_since(past).unwrap());
    }

    #[test]
    fn test_has_changes_since_future() {
        let temp = create_git_repo();
        let client = GitClient::open(temp.path()).unwrap();

        // Check from future time
        let future = SystemTime::now() + Duration::from_secs(3600);
        assert!(!client.has_changes_since(future).unwrap());
    }

    #[test]
    fn test_staleness_result_fields() {
        let result = StalenessResult {
            is_stale: true,
            summary_time: Some(SystemTime::now()),
            latest_commit_time: Some(SystemTime::now()),
            commits_since: 5,
            reason: Some("Test reason".to_string()),
        };

        assert!(result.is_stale);
        assert_eq!(result.commits_since, 5);
        assert_eq!(result.reason, Some("Test reason".to_string()));
    }
}
