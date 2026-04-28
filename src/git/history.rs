use anyhow::Result;
use git2::{DiffOptions, Oid};
use std::collections::HashMap;
use std::path::Path;

use super::GitClient;

impl GitClient {
    /// Get the number of commits that modified a specific file
    pub fn get_file_change_count(&self, file_path: &Path) -> Result<usize> {
        let relative_path = match self.relative_path(file_path) {
            Some(p) => p,
            None => return Ok(0),
        };

        let mut revwalk = self.repo.revwalk()?;
        revwalk.push_head()?;
        revwalk.set_sorting(git2::Sort::TIME)?;

        let mut count = 0;
        let path_str = relative_path.to_string_lossy();

        for oid in revwalk {
            let oid = oid?;
            if self.commit_modified_file(oid, &path_str)? {
                count += 1;
            }
        }

        Ok(count)
    }

    /// Check if a commit modified a specific file
    fn commit_modified_file(&self, commit_oid: Oid, file_path: &str) -> Result<bool> {
        let commit = self.repo.find_commit(commit_oid)?;
        let tree = commit.tree()?;

        // For the first commit (no parent), check if file exists in tree
        if commit.parent_count() == 0 {
            return Ok(tree.get_path(Path::new(file_path)).is_ok());
        }

        // Compare with parent
        let parent = commit.parent(0)?;
        let parent_tree = parent.tree()?;

        let mut diff_opts = DiffOptions::new();
        diff_opts.pathspec(file_path);

        let diff = self.repo.diff_tree_to_tree(
            Some(&parent_tree),
            Some(&tree),
            Some(&mut diff_opts),
        )?;

        Ok(diff.stats()?.files_changed() > 0)
    }

    /// Get change counts for all files in a directory
    pub fn get_directory_change_counts(&self, dir_path: &Path) -> Result<HashMap<String, usize>> {
        let relative_dir = self.relative_path(dir_path).unwrap_or_default();
        let mut counts: HashMap<String, usize> = HashMap::new();

        let mut revwalk = self.repo.revwalk()?;
        revwalk.push_head()?;
        revwalk.set_sorting(git2::Sort::TIME)?;

        for oid in revwalk {
            let oid = oid?;
            let commit = self.repo.find_commit(oid)?;
            let tree = commit.tree()?;

            if commit.parent_count() == 0 {
                // First commit - count all files
                self.count_files_in_tree(&tree, &relative_dir, &mut counts)?;
                continue;
            }

            let parent = commit.parent(0)?;
            let parent_tree = parent.tree()?;

            let diff = self.repo.diff_tree_to_tree(
                Some(&parent_tree),
                Some(&tree),
                None,
            )?;

            diff.foreach(
                &mut |delta, _| {
                    if let Some(path) = delta.new_file().path() {
                        let path_str = path.to_string_lossy().to_string();
                        if path_str.starts_with(&relative_dir.to_string_lossy().to_string()) {
                            *counts.entry(path_str).or_insert(0) += 1;
                        }
                    }
                    true
                },
                None,
                None,
                None,
            )?;
        }

        Ok(counts)
    }

    /// Count files in a tree under a specific directory
    fn count_files_in_tree(
        &self,
        tree: &git2::Tree,
        prefix: &Path,
        counts: &mut HashMap<String, usize>,
    ) -> Result<()> {
        let prefix_str = prefix.to_string_lossy().to_string();

        tree.walk(git2::TreeWalkMode::PreOrder, |root, entry| {
            if entry.kind() == Some(git2::ObjectType::Blob) {
                let path = format!("{}{}", root, entry.name().unwrap_or(""));
                if prefix_str.is_empty() || path.starts_with(&prefix_str) {
                    *counts.entry(path).or_insert(0) += 1;
                }
            }
            git2::TreeWalkResult::Ok
        })?;

        Ok(())
    }

    /// Get the most recent commit that modified a file
    pub fn get_last_modified_commit(&self, file_path: &Path) -> Result<Option<git2::Commit<'_>>> {
        let relative_path = match self.relative_path(file_path) {
            Some(p) => p,
            None => return Ok(None),
        };

        let mut revwalk = self.repo.revwalk()?;
        revwalk.push_head()?;
        revwalk.set_sorting(git2::Sort::TIME)?;

        let path_str = relative_path.to_string_lossy();

        for oid in revwalk {
            let oid = oid?;
            if self.commit_modified_file(oid, &path_str)? {
                let commit = self.repo.find_commit(oid)?;
                return Ok(Some(commit));
            }
        }

        Ok(None)
    }

    /// Get the total number of commits in the repository
    pub fn get_total_commits(&self) -> Result<usize> {
        let mut revwalk = self.repo.revwalk()?;
        revwalk.push_head()?;

        let count = revwalk.count();
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::process::Command;
    use tempfile::TempDir;

    fn create_git_repo_with_history() -> TempDir {
        let temp = TempDir::new().unwrap();
        let root = temp.path();

        // Initialize git repo
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

        // Create src directory
        fs::create_dir_all(root.join("src")).unwrap();

        // First commit - create main.rs
        fs::write(root.join("src/main.rs"), "fn main() {}").unwrap();
        Command::new("git")
            .args(["add", "."])
            .current_dir(root)
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "Add main.rs"])
            .current_dir(root)
            .output()
            .unwrap();

        // Second commit - modify main.rs
        fs::write(root.join("src/main.rs"), "fn main() { println!(\"hello\"); }").unwrap();
        Command::new("git")
            .args(["add", "."])
            .current_dir(root)
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "Update main.rs"])
            .current_dir(root)
            .output()
            .unwrap();

        // Third commit - add lib.rs
        fs::write(root.join("src/lib.rs"), "pub fn add() {}").unwrap();
        Command::new("git")
            .args(["add", "."])
            .current_dir(root)
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "Add lib.rs"])
            .current_dir(root)
            .output()
            .unwrap();

        // Fourth commit - modify main.rs again
        fs::write(root.join("src/main.rs"), "fn main() { println!(\"world\"); }").unwrap();
        Command::new("git")
            .args(["add", "."])
            .current_dir(root)
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "Update main.rs again"])
            .current_dir(root)
            .output()
            .unwrap();

        temp
    }

    #[test]
    fn test_get_file_change_count() {
        let temp = create_git_repo_with_history();
        let client = GitClient::open(temp.path()).unwrap();

        // main.rs was modified 3 times (created + 2 updates)
        let count = client
            .get_file_change_count(&temp.path().join("src/main.rs"))
            .unwrap();
        assert_eq!(count, 3);

        // lib.rs was only created once
        let count = client
            .get_file_change_count(&temp.path().join("src/lib.rs"))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_get_file_change_count_nonexistent() {
        let temp = create_git_repo_with_history();
        let client = GitClient::open(temp.path()).unwrap();

        let count = client
            .get_file_change_count(&temp.path().join("nonexistent.rs"))
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_get_file_change_count_outside_repo() {
        let temp = create_git_repo_with_history();
        let client = GitClient::open(temp.path()).unwrap();

        let count = client
            .get_file_change_count(Path::new("/some/other/path.rs"))
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_get_total_commits() {
        let temp = create_git_repo_with_history();
        let client = GitClient::open(temp.path()).unwrap();

        let count = client.get_total_commits().unwrap();
        assert_eq!(count, 4);
    }

    #[test]
    fn test_get_last_modified_commit() {
        let temp = create_git_repo_with_history();
        let client = GitClient::open(temp.path()).unwrap();

        let commit = client
            .get_last_modified_commit(&temp.path().join("src/main.rs"))
            .unwrap();
        assert!(commit.is_some());

        let commit = commit.unwrap();
        assert!(commit.message().unwrap().contains("Update main.rs again"));
    }

    #[test]
    fn test_get_directory_change_counts() {
        let temp = create_git_repo_with_history();
        let client = GitClient::open(temp.path()).unwrap();

        let counts = client
            .get_directory_change_counts(&temp.path().join("src"))
            .unwrap();

        assert!(counts.contains_key("src/main.rs"));
        assert!(counts.contains_key("src/lib.rs"));
        assert_eq!(counts.get("src/main.rs"), Some(&3));
        assert_eq!(counts.get("src/lib.rs"), Some(&1));
    }
}
