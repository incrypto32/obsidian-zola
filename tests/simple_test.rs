use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;
use obsidian_export::{Exporter, FrontmatterStrategy};
use obsidian_zola::postprocessors::create_zola_link_postprocessor;

/// Copy the test vault to a temporary directory for testing
fn copy_test_vault_to_temp(temp_dir: &Path) {
    let test_vault_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/test_vault");
    copy_dir_recursive(&test_vault_path, temp_dir).expect("Failed to copy test vault");
}

/// Recursively copy a directory
fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }
    
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    
    Ok(())
}

#[test]
fn test_export_matches_expected() {
    // Setup
    let temp_vault = TempDir::new().unwrap();
    let temp_output = TempDir::new().unwrap();
    
    copy_test_vault_to_temp(temp_vault.path());
    
    // Export
    let mut exporter = Exporter::new(
        temp_vault.path().to_path_buf(),
        temp_output.path().to_path_buf(),
    );
    
    // Configure frontmatter strategy to match CLI behavior
    exporter.frontmatter_strategy(FrontmatterStrategy::Always);
    
    let zola_postprocessor = create_zola_link_postprocessor(temp_vault.path().to_path_buf());
    exporter.add_postprocessor(&zola_postprocessor);
    
    exporter.run().expect("Export should succeed");
    
    // Compare with expected output using system diff
    let expected_output = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/expected_output");
    
    let diff_result = Command::new("diff")
        .arg("-r")
        .arg(temp_output.path())
        .arg(&expected_output)
        .output()
        .expect("Failed to run diff command");
    
    if !diff_result.status.success() {
        println!("Diff output:");
        println!("{}", String::from_utf8_lossy(&diff_result.stdout));
        if !diff_result.stderr.is_empty() {
            println!("Diff errors:");
            println!("{}", String::from_utf8_lossy(&diff_result.stderr));
        }
        panic!("Generated output does not match expected output");
    }
} 