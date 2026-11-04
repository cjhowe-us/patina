//! Shared test utilities for directory-based RON tests

use ron;
use serde::Deserialize;
use std::fs;
use std::fs::DirEntry;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
pub struct TestCase {
    pub name: String,
    pub input: String,
    #[serde(default)]
    pub should_fail: bool,
    pub description: Option<String>,
}

/// Run tests by reading all RON files from a directory
pub fn run_directory_tests<T, F>(test_name: &str, directory: &str, test_fn: F)
where
    T: std::fmt::Debug,
    F: Fn(&str, &str) -> Result<T, Box<dyn std::error::Error>>,
{
    let dir_path = Path::new("test_cases").join(directory);

    let mut entries: Vec<_> = ron_files_in_dir(&dir_path)
        .unwrap_or_else(|e| panic!("Failed to read directory {}: {}", directory, e));

    // Sort entries for consistent test order
    entries.sort_by_key(|e| e.path());

    if entries.is_empty() {
        panic!("No test files found in directory: {}", directory);
    }

    for entry in entries {
        let path = entry.path();

        let content = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("Failed to read test file {:?}: {}", path, e));

        let test_case: TestCase = ron::from_str(&content)
            .unwrap_or_else(|e| panic!("Failed to parse RON file {:?}: {}", path, e));

        // Trim the input to remove leading/trailing newlines from raw strings
        let input = test_case.input.trim();

        match test_fn(input, &test_case.name) {
            Ok(_) => {
                if test_case.should_fail {
                    panic!(
                        "{}: {} - Expected test to fail but it succeeded",
                        test_name, test_case.name
                    );
                }
            }
            Err(e) => {
                if !test_case.should_fail {
                    panic!(
                        "{}: {} - Test failed unexpectedly: {:?}",
                        test_name, test_case.name, e
                    );
                }
            }
        }
    }
}

/// Run parser tests with AST verification
pub fn run_parser_directory_tests<F, V>(test_name: &str, directory: &str, parse_fn: F, verify_fn: V)
where
    F: Fn(&str, &str) -> Result<crate::ast::Ast, Box<dyn std::error::Error>>,
    V: Fn(&str, &crate::ast::Ast),
{
    let dir_path = Path::new("test_cases").join(directory);

    let mut entries: Vec<_> = ron_files_in_dir(&dir_path)
        .unwrap_or_else(|e| panic!("Failed to read directory {}: {}", directory, e));

    // Sort entries for consistent test order
    entries.sort_by_key(|e| e.path());

    if entries.is_empty() {
        panic!("No test files found in directory: {}", directory);
    }

    for entry in entries {
        let path = entry.path();

        let content = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("Failed to read test file {:?}: {}", path, e));

        let test_case: TestCase = ron::from_str(&content)
            .unwrap_or_else(|e| panic!("Failed to parse RON file {:?}: {}", path, e));

        // Trim the input to remove leading/trailing newlines from raw strings
        let input = test_case.input.trim();

        if input.is_empty() {
            eprintln!(
                "Warning: Test '{}' has empty input, skipping",
                test_case.name
            );
            continue;
        }

        match parse_fn(input, &test_case.name) {
            Ok(ast) => {
                if test_case.should_fail {
                    panic!(
                        "{}: {} - Expected test to fail but it succeeded",
                        test_name, test_case.name
                    );
                }
                verify_fn(&test_case.name, &ast);
            }
            Err(e) => {
                if !test_case.should_fail {
                    panic!(
                        "{}: {} - Test failed unexpectedly: {:?}",
                        test_name, test_case.name, e
                    );
                }
            }
        }
    }
}

fn ron_files_in_dir(dir_path: &PathBuf) -> std::io::Result<Vec<DirEntry>> {
    fs::read_dir(dir_path).map(|directory| {
        directory
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .and_then(|s| s.to_str())
                    .map(|s| s == "ron")
                    .unwrap_or(false)
            })
            .collect()
    })
}
