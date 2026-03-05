use std::fs;
use std::path::Path;

/// Helper: create a temporary git repository for testing.
fn create_test_repo() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("Failed to create temp dir");

    // Initialize a git repo
    let repo = git2::Repository::init(dir.path()).expect("Failed to init repo");

    // Configure user for commits
    let mut config = repo.config().expect("Failed to get config");
    config
        .set_str("user.name", "Test User")
        .expect("Failed to set user.name");
    config
        .set_str("user.email", "test@example.com")
        .expect("Failed to set user.email");

    // Create an initial file and commit
    let file_path = dir.path().join("README.md");
    fs::write(&file_path, "# Test Project\n").expect("Failed to write README");

    let mut index = repo.index().expect("Failed to get index");
    index
        .add_path(Path::new("README.md"))
        .expect("Failed to add");
    index.write().expect("Failed to write index");

    let tree_id = index.write_tree().expect("Failed to write tree");
    let tree = repo.find_tree(tree_id).expect("Failed to find tree");
    let sig = repo.signature().expect("Failed to get signature");

    repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])
        .expect("Failed to initial commit");

    dir
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_git_repo() {
        let dir = create_test_repo();
        let repo = devos::core::git::open_repo(dir.path());
        assert!(repo.is_some());
    }

    #[test]
    fn test_detect_no_repo() {
        let dir = tempfile::tempdir().expect("Failed to create temp dir");
        let repo = devos::core::git::open_repo(dir.path());
        // discover walks up, so it may find a parent repo.
        // For a truly isolated test, we check the repo workdir matches.
        if let Some(repo) = repo {
            let workdir = repo.workdir().unwrap();
            // If workdir isn't our temp dir, treat as "not our repo"
            assert_ne!(workdir, dir.path());
        }
    }

    #[test]
    fn test_get_branch() {
        let dir = create_test_repo();
        let repo = devos::core::git::open_repo(dir.path()).unwrap();
        let branch = devos::core::git::get_branch(&repo);
        // Could be "main" or "master" depending on git config
        assert!(!branch.is_empty());
    }

    #[test]
    fn test_stage_and_commit() {
        let dir = create_test_repo();

        // Create a new file
        let new_file = dir.path().join("new.txt");
        fs::write(&new_file, "Hello world\n").expect("Failed to write");

        let repo = devos::core::git::open_repo(dir.path()).unwrap();

        // Stage
        devos::core::git::stage_file(&repo, "new.txt").expect("Failed to stage");

        // Commit
        devos::core::git::commit(&repo, "Add new.txt").expect("Failed to commit");

        // Verify — status should be clean
        let status = devos::core::git::get_status(&repo);
        let new_txt_status = status.iter().find(|s| s.path == "new.txt");
        assert!(
            new_txt_status.is_none(),
            "new.txt should not appear in status after commit"
        );
    }

    #[test]
    fn test_unstage_file() {
        let dir = create_test_repo();

        let new_file = dir.path().join("staged.txt");
        fs::write(&new_file, "Staged content\n").expect("Failed to write");

        let repo = devos::core::git::open_repo(dir.path()).unwrap();

        // Stage
        devos::core::git::stage_file(&repo, "staged.txt").expect("Failed to stage");

        // Verify staged
        let status = devos::core::git::get_status(&repo);
        let staged = status.iter().find(|s| s.path == "staged.txt");
        assert!(staged.is_some());
        assert!(staged.unwrap().staged);

        // Unstage
        devos::core::git::unstage_file(&repo, "staged.txt").expect("Failed to unstage");

        // Verify unstaged — file should still be in status but not staged
        let status = devos::core::git::get_status(&repo);
        let unstaged = status.iter().find(|s| s.path == "staged.txt");
        assert!(unstaged.is_some());
        assert!(!unstaged.unwrap().staged);
    }

    #[test]
    fn test_status_shows_modified() {
        let dir = create_test_repo();

        // Modify the README
        let readme = dir.path().join("README.md");
        fs::write(&readme, "# Modified Project\n").expect("Failed to modify");

        let repo = devos::core::git::open_repo(dir.path()).unwrap();
        let status = devos::core::git::get_status(&repo);

        let readme_status = status.iter().find(|s| s.path == "README.md");
        assert!(readme_status.is_some(), "README.md should appear as modified");
        assert_eq!(
            readme_status.unwrap().status,
            devos::models::GitStatus::Modified
        );
    }
}
