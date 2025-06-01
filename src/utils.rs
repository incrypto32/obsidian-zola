//! Utility functions for obsidian-zola operations.

use std::path::Path;
use eyre::Result;

/// Validates that a path exists and is a directory.
/// 
/// # Arguments
/// 
/// * `path` - The path to validate
/// * `description` - A description of what this path represents (for error messages)
/// 
/// # Returns
/// 
/// `Ok(())` if the path exists and is a directory, otherwise an error.
pub fn validate_directory<P: AsRef<Path>>(path: P, description: &str) -> Result<()> {
    let path = path.as_ref();
    
    if !path.exists() {
        return Err(eyre::eyre!("{} does not exist: {}", description, path.display()));
    }
    
    if !path.is_dir() {
        return Err(eyre::eyre!("{} is not a directory: {}", description, path.display()));
    }
    
    Ok(())
}

/// Checks if a file has a markdown extension (.md or .markdown).
/// 
/// # Arguments
/// 
/// * `path` - The path to check
/// 
/// # Returns
/// 
/// `true` if the file has a markdown extension, `false` otherwise.
pub fn is_markdown_file<P: AsRef<Path>>(path: P) -> bool {
    match path.as_ref().extension() {
        Some(ext) => {
            let ext_str = ext.to_string_lossy().to_lowercase();
            ext_str == "md" || ext_str == "markdown"
        }
        None => false,
    }
}

/// Normalizes a path by removing redundant components and converting to forward slashes.
/// This ensures consistent path formatting for Zola links.
/// 
/// # Arguments
/// 
/// * `path` - The path to normalize
/// 
/// # Returns
/// 
/// A normalized path string using forward slashes.
pub fn normalize_path<P: AsRef<Path>>(path: P) -> String {
    path.as_ref()
        .components()
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;
    
    #[test]
    fn test_validate_directory_exists() {
        let temp_dir = TempDir::new().unwrap();
        let result = validate_directory(temp_dir.path(), "Test directory");
        assert!(result.is_ok());
    }
    
    #[test] 
    fn test_validate_directory_not_exists() {
        let result = validate_directory("/nonexistent/path", "Test directory");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("does not exist"));
    }
    
    #[test]
    fn test_validate_directory_is_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "test").unwrap();
        
        let result = validate_directory(&file_path, "Test directory");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("is not a directory"));
    }
    
    #[test]
    fn test_is_markdown_file() {
        assert!(is_markdown_file("test.md"));
        assert!(is_markdown_file("test.markdown"));
        assert!(is_markdown_file("TEST.MD"));
        assert!(is_markdown_file("path/to/file.md"));
        
        assert!(!is_markdown_file("test.txt"));
        assert!(!is_markdown_file("test.html"));
        assert!(!is_markdown_file("test"));
        assert!(!is_markdown_file("test.md.backup"));
    }
    
    #[test]
    fn test_normalize_path() {
        assert_eq!(normalize_path("folder/file.md"), "folder/file.md");
        #[cfg(windows)]
        assert_eq!(normalize_path("folder\\file.md"), "folder/file.md");
        #[cfg(not(windows))]
        assert_eq!(normalize_path("folder\\file.md"), "folder\\file.md");
        assert_eq!(normalize_path("./folder/../other/file.md"), "./folder/../other/file.md");
    }
} 