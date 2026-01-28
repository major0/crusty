// Copyright (c) 2026 Mark Ferrell
// Licensed under the MIT License. See LICENSE.txt in the project root.

//! Property-based tests for CLI and file I/O operations.

#[cfg(test)]
mod tests {
    use crate::cli::{read_source_file, write_output_file};
    use proptest::prelude::*;
    use std::fs;
    use std::path::PathBuf;

    /// Property 29: Valid file paths are read successfully
    /// Validates: Requirements 11.1
    ///
    /// This property verifies that when we write content to a file and then read it back,
    /// the content is preserved exactly. This ensures file I/O operations work correctly.
    #[test]
    fn property_29_valid_file_paths_read_successfully() {
        proptest!(|(content in ".*")| {
            // Generate a unique temporary file path
            let test_path = PathBuf::from(format!("test_prop_29_{}.tmp", 
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()));

            // Write content to file
            let write_result = write_output_file(&test_path, &content);
            prop_assert!(write_result.is_ok(), "Failed to write file: {:?}", write_result.err());

            // Read content back
            let read_result = read_source_file(&test_path);
            prop_assert!(read_result.is_ok(), "Failed to read file: {:?}", read_result.err());

            // Verify content matches
            let read_content = read_result.unwrap();
            prop_assert_eq!(read_content, content, "File content mismatch");

            // Clean up
            let _ = fs::remove_file(&test_path);
        });
    }

    /// Test that reading non-existent files produces appropriate errors
    #[test]
    fn property_nonexistent_files_produce_errors() {
        proptest!(|(filename in "[a-z]{10,20}\\.crst")| {
            // Ensure file doesn't exist
            let test_path = PathBuf::from(format!("nonexistent_{}", filename));
            let _ = fs::remove_file(&test_path); // Ensure it doesn't exist

            // Try to read non-existent file
            let read_result = read_source_file(&test_path);
            prop_assert!(read_result.is_err(), "Should fail to read non-existent file");
            prop_assert_eq!(
                read_result.unwrap_err().kind(),
                std::io::ErrorKind::NotFound,
                "Should return NotFound error"
            );
        });
    }

    /// Test that file I/O handles various content types correctly
    #[test]
    fn property_file_io_handles_various_content() {
        proptest!(|(
            content in prop::collection::vec(any::<u8>(), 0..1000)
        )| {
            let content_str = String::from_utf8_lossy(&content).to_string();
            let test_path = PathBuf::from(format!("test_content_{}.tmp",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()));

            // Write and read back
            let write_result = write_output_file(&test_path, &content_str);
            if write_result.is_ok() {
                let read_result = read_source_file(&test_path);
                if let Ok(read_content) = read_result {
                    prop_assert_eq!(read_content, content_str);
                }
            }

            // Clean up
            let _ = fs::remove_file(&test_path);
        });
    }
}
