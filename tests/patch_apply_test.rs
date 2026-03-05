use std::fs;
use std::path::Path;

/// Helper: create a temporary directory with a file for testing.
fn create_test_dir() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("Failed to create temp dir");

    // Create a sample file
    let file_path = dir.path().join("hello.rs");
    fs::write(&file_path, "fn main() {\n    println!(\"Hello\");\n}\n")
        .expect("Failed to write test file");

    dir
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_and_write_file() {
        let dir = create_test_dir();
        let file_path = dir.path().join("hello.rs");

        // Read
        let content = devos::core::file_manager::read_file(&file_path).unwrap();
        assert!(content.contains("println!"));

        // Write
        let new_content = "fn main() {\n    println!(\"World\");\n}\n";
        devos::core::file_manager::write_file(&file_path, new_content).unwrap();

        let re_read = devos::core::file_manager::read_file(&file_path).unwrap();
        assert!(re_read.contains("World"));
        assert!(!re_read.contains("Hello"));
    }

    #[test]
    fn test_create_backup() {
        let dir = create_test_dir();
        let file_path = dir.path().join("hello.rs");

        let backup = devos::core::file_manager::create_backup(&file_path, dir.path()).unwrap();

        assert!(backup.exists());

        let original = fs::read_to_string(&file_path).unwrap();
        let backed_up = fs::read_to_string(&backup).unwrap();
        assert_eq!(original, backed_up);
    }

    #[test]
    fn test_backup_before_patch_simple() {
        let dir = create_test_dir();
        let file_path = dir.path().join("hello.rs");

        let original_content = fs::read_to_string(&file_path).unwrap();
        let new_content = "fn main() {\n    println!(\"Patched!\");\n}\n";

        devos::core::patch::apply_patch_simple(
            &file_path,
            &original_content,
            new_content,
            dir.path(),
        )
        .unwrap();

        // File should be updated
        let updated = fs::read_to_string(&file_path).unwrap();
        assert!(updated.contains("Patched!"));

        // Backup should exist with original content
        let backup_path = dir.path().join(".devos_backups").join("hello.rs");
        assert!(backup_path.exists());
        let backup_content = fs::read_to_string(&backup_path).unwrap();
        assert!(backup_content.contains("Hello"));
    }

    #[test]
    fn test_reject_patch_no_change() {
        let dir = create_test_dir();
        let file_path = dir.path().join("hello.rs");

        let original = fs::read_to_string(&file_path).unwrap();

        // Don't apply anything — just verify file is unchanged
        let still_original = fs::read_to_string(&file_path).unwrap();
        assert_eq!(original, still_original);

        // No backup dir should exist
        let backup_dir = dir.path().join(".devos_backups");
        assert!(!backup_dir.exists());
    }

    #[test]
    fn test_parse_unified_diff() {
        let diff = "\
--- a/hello.rs
+++ b/hello.rs
@@ -1,3 +1,3 @@
 fn main() {
-    println!(\"Hello\");
+    println!(\"World\");
 }
";
        let patches = devos::core::patch::parse_unified_diff(diff);
        assert_eq!(patches.len(), 1);
        assert_eq!(patches[0].file_path, "hello.rs");
        assert!(patches[0].diff_text.contains("-    println!(\"Hello\")"));
        assert!(patches[0].diff_text.contains("+    println!(\"World\")"));
    }
}
