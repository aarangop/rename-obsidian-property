//! File utility functions for working with the filesystem.
//! 
//! This module provides utilities for finding files that match
//! glob patterns.

use std::path::PathBuf;
use glob::{glob, PatternError};


/// Returns a vector of paths that match the provided glob pattern.
/// 
/// This function searches the filesystem for paths that match the given pattern
/// and returns them as a vector of `PathBuf` objects.
/// 
/// # Arguments
/// 
/// * `pattern` - A glob pattern string to match against file paths
/// 
/// # Returns
/// 
/// * `Result<Vec<PathBuf>, PatternError>` - A vector of matched paths or a pattern error
/// 
/// # Examples
/// 
/// ```
/// let md_files = match_files(&"**/*.md".to_string());
/// if let Ok(files) = md_files {
///     for file in files {
///         println!("Found markdown file: {:?}", file);
///     }
/// }
/// ```
pub fn match_files(pattern: &String) -> Result<Vec<PathBuf>, PatternError> {
  let paths = glob(pattern)?;

  let path_bufs = paths
    .filter_map(|entry| match entry {
      Ok(path) => Some(path),
      Err(_) => None
    })
    .collect();

    Ok(path_bufs)
}


#[cfg(test)]
mod tests {
    use std::fs::{self, File};
    use std::io::Write;

    use tempfile::TempDir;

    use super::*;

    // Helper function to create a temporary directory with test files
    fn setup_test_directory() -> TempDir {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
        
        // Create some test files
        let test_files = [
            "file1.txt",
            "file2.txt",
            "image.png",
            "document.pdf",
            "nested/file3.txt",
        ];
        
        for file_name in &test_files {
            let path = temp_dir.path().join(file_name);
            
            // Create directory if it doesn't exist
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).expect("Failed to create directory");
            }
            
            // Create the file
            let mut file = File::create(&path).expect("Failed to create file");
            writeln!(file, "Test content").expect("Failed to write to file");
        }
        
        temp_dir
    }

    #[test]
    fn test_matching_txt_files() {
        let temp_dir = setup_test_directory();
        let pattern = format!("{}/*.txt", temp_dir.path().display());
        
        let result = match_files(&pattern).unwrap();
        
        // There should be 2 .txt files in the root directory
        assert_eq!(result.len(), 2);
        
        // Check that we found the expected files
        let file_names: Vec<String> = result
            .iter()
            .map(|path| path.file_name().unwrap().to_string_lossy().to_string())
            .collect();
        
        assert!(file_names.contains(&"file1.txt".to_string()));
        assert!(file_names.contains(&"file2.txt".to_string()));
    }

    #[test]
    fn test_matching_all_files_recursively() {
        let temp_dir = setup_test_directory();
        let pattern = format!("{}/**/*", temp_dir.path().display());
        
        let result = match_files(&pattern).unwrap();
        
        // There should be 5 files total + the nested directory
        assert_eq!(result.len(), 6);
    }

    #[test]
    fn test_no_matches() {
        let temp_dir = setup_test_directory();
        let pattern = format!("{}/*.nonexistent", temp_dir.path().display());
        
        let result = match_files(&pattern).unwrap();
        
        // There should be no matches
        assert!(result.is_empty());
    }

    #[test]
    fn test_invalid_pattern() {
        // An invalid glob pattern
        let result = match_files(&']'.to_string());
        
        // Should return an error
        assert!(result.is_err());
    }
}
