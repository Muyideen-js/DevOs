use std::path::Path;

use crate::models::{GitFileStatus, GitStatus};

/// Open a git repository at the given path. Returns None if not a git repo.
pub fn open_repo(path: &Path) -> Option<git2::Repository> {
    git2::Repository::discover(path).ok()
}

/// Get the current branch name.
pub fn get_branch(repo: &git2::Repository) -> String {
    match repo.head() {
        Ok(head) => {
            if let Some(name) = head.shorthand() {
                name.to_string()
            } else {
                "HEAD (detached)".to_string()
            }
        }
        Err(_) => "No commits yet".to_string(),
    }
}

/// Get the status of all files in the repository.
pub fn get_status(repo: &git2::Repository) -> Vec<GitFileStatus> {
    let mut result = Vec::new();

    let statuses = match repo.statuses(None) {
        Ok(s) => s,
        Err(_) => return result,
    };

    for entry in statuses.iter() {
        let path = entry.path().unwrap_or("").to_string();
        let s = entry.status();

        let (status, staged) = if s.contains(git2::Status::INDEX_NEW) {
            (GitStatus::New, true)
        } else if s.contains(git2::Status::INDEX_MODIFIED) {
            (GitStatus::Modified, true)
        } else if s.contains(git2::Status::INDEX_DELETED) {
            (GitStatus::Deleted, true)
        } else if s.contains(git2::Status::INDEX_RENAMED) {
            (GitStatus::Renamed, true)
        } else if s.contains(git2::Status::WT_NEW) {
            (GitStatus::Untracked, false)
        } else if s.contains(git2::Status::WT_MODIFIED) {
            (GitStatus::Modified, false)
        } else if s.contains(git2::Status::WT_DELETED) {
            (GitStatus::Deleted, false)
        } else if s.contains(git2::Status::WT_RENAMED) {
            (GitStatus::Renamed, false)
        } else {
            continue;
        };

        result.push(GitFileStatus {
            path,
            status,
            staged,
        });
    }

    result
}

/// Stage a file (add to index).
pub fn stage_file(repo: &git2::Repository, path: &str) -> Result<(), String> {
    let mut index = repo.index().map_err(|e| format!("Index error: {}", e))?;
    let file_path = std::path::Path::new(path);

    // Check if the file exists on disk (not deleted)
    let workdir = repo.workdir().ok_or("No workdir")?;
    if workdir.join(file_path).exists() {
        index
            .add_path(file_path)
            .map_err(|e| format!("Stage error: {}", e))?;
    } else {
        index
            .remove_path(file_path)
            .map_err(|e| format!("Stage delete error: {}", e))?;
    }

    index.write().map_err(|e| format!("Index write error: {}", e))
}

/// Unstage a file (reset to HEAD).
pub fn unstage_file(repo: &git2::Repository, path: &str) -> Result<(), String> {
    let head = repo.head().map_err(|e| format!("HEAD error: {}", e))?;
    let head_commit = head
        .peel_to_commit()
        .map_err(|e| format!("Commit error: {}", e))?;
    let _tree = head_commit
        .tree()
        .map_err(|e| format!("Tree error: {}", e))?;

    repo.reset_default(Some(head_commit.as_object()), [path])
        .map_err(|e| format!("Unstage error: {}", e))
}

/// Create a commit with the given message using files currently in the index.
pub fn commit(repo: &git2::Repository, message: &str) -> Result<(), String> {
    let mut index = repo.index().map_err(|e| format!("Index error: {}", e))?;
    let tree_oid = index
        .write_tree()
        .map_err(|e| format!("Write tree error: {}", e))?;
    let tree = repo
        .find_tree(tree_oid)
        .map_err(|e| format!("Find tree error: {}", e))?;

    let sig = repo
        .signature()
        .map_err(|e| format!("Signature error (set git user.name and user.email): {}", e))?;

    let parent = match repo.head() {
        Ok(head) => {
            let commit = head
                .peel_to_commit()
                .map_err(|e| format!("Parent commit error: {}", e))?;
            Some(commit)
        }
        Err(_) => None, // Initial commit
    };

    let parents: Vec<&git2::Commit> = parent.iter().collect();

    repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &parents)
        .map_err(|e| format!("Commit error: {}", e))?;

    Ok(())
}
