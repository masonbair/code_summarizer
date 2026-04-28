mod history;
mod staleness;


use anyhow::{Context, Result};
use git2::Repository;
use std::path::{Path, PathBuf};

/// Client for Git operations
pub struct GitClient {
    repo: Repository,
    repo_root: PathBuf,
}

impl GitClient {
    /// Open a git repository at the given path
    pub fn open(path: &Path) -> Result<Self> {
        let repo = Repository::discover(path)
            .with_context(|| format!("Git repository not found at: {:?}", path))?;

        let repo_root = repo
            .workdir()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| path.to_path_buf());

        Ok(Self { repo, repo_root })
    }

    /// Get the repository root path
    pub fn repo_root(&self) -> &Path {
        &self.repo_root
    }

    /// Get reference to the underlying git2 Repository
    pub fn repo(&self) -> &Repository {
        &self.repo
    }

    /// Check if a path is inside the repository
    pub fn is_in_repo(&self, path: &Path) -> bool {
        path.starts_with(&self.repo_root)
    }

    /// Get the relative path within the repository
    pub fn relative_path(&self, path: &Path) -> Option<PathBuf> {
        path.strip_prefix(&self.repo_root)
            .ok()
            .map(|p| p.to_path_buf())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::process::Command;
    use tempfile::TempDir;

    fn create_git_repo() -> TempDir {
        let temp = TempDir::new().unwrap();
        let root = temp.path();

        // Initialize git repo
        Command::new("git")
            .args(["init"])
            .current_dir(root)
            .output()
            .unwrap();

        // Configure git user for commits
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

        // Create a file and commit
        fs::write(root.join("test.txt"), "hello").unwrap();

        Command::new("git")
            .args(["add", "."])
            .current_dir(root)
            .output()
            .unwrap();

        Command::new("git")
            .args(["commit", "-m", "Initial commit"])
            .current_dir(root)
            .output()
            .unwrap();

        temp
    }

    #[test]
    fn test_open_git_repo() {
        let temp = create_git_repo();
        let client = GitClient::open(temp.path());
        assert!(client.is_ok());
    }

    #[test]
    fn test_open_non_git_directory() {
        let temp = TempDir::new().unwrap();
        let client = GitClient::open(temp.path());
        assert!(client.is_err());
    }

    #[test]
    fn test_repo_root() {
        let temp = create_git_repo();
        let client = GitClient::open(temp.path()).unwrap();
        assert_eq!(client.repo_root(), temp.path());
    }

    #[test]
    fn test_is_in_repo() {
        let temp = create_git_repo();
        let client = GitClient::open(temp.path()).unwrap();

        assert!(client.is_in_repo(&temp.path().join("test.txt")));
        assert!(client.is_in_repo(&temp.path().join("subdir/file.rs")));
        assert!(!client.is_in_repo(Path::new("/some/other/path")));
    }

    #[test]
    fn test_relative_path() {
        let temp = create_git_repo();
        let client = GitClient::open(temp.path()).unwrap();

        let rel = client.relative_path(&temp.path().join("src/main.rs"));
        assert_eq!(rel, Some(PathBuf::from("src/main.rs")));

        let outside = client.relative_path(Path::new("/some/other/path"));
        assert_eq!(outside, None);
    }
}
