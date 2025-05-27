use serde::{Deserialize, Serialize};
use serde_json;
use std::fs;
use std::path::Path;

/// Represents a test case with input data and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase<T> {
    pub name: String,
    pub input: T,
}

/// Error wrapper for test results
#[derive(Debug, Serialize, Deserialize)]
pub struct TestError {
    pub error: String,
}

impl TestError {
    pub fn new<E: std::fmt::Display>(err: E) -> Self {
        Self {
            error: err.to_string(),
        }
    }
}

/// Result type for test cases - either success with data or error
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TestResult<T> {
    Success(T),
    Error(TestError),
}

impl<T> From<Result<T, Box<dyn std::error::Error>>> for TestResult<T> {
    fn from(result: Result<T, Box<dyn std::error::Error>>) -> Self {
        match result {
            Ok(value) => TestResult::Success(value),
            Err(err) => TestResult::Error(TestError::new(err)),
        }
    }
}

/// Load test cases from a JSON file
pub fn load_test_cases<T>(filename: &str) -> Result<Vec<TestCase<T>>, Box<dyn std::error::Error>>
where
    T: for<'de> Deserialize<'de>,
{
    let path = Path::new("ast/testing/cases").join(filename);
    let content = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read test file {}: {}", path.display(), e))?;

    let test_cases: Vec<TestCase<T>> = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse test file {}: {}", path.display(), e))?;

    Ok(test_cases)
}

/// Run table-driven tests with snapshot testing
pub fn run_tests<I, O>(
    test_name: &str,
    filenames: &[&str],
    test_fn: impl Fn(&I, &str) -> Result<O, Box<dyn std::error::Error>>,
) where
    I: for<'de> Deserialize<'de>,
    O: Serialize,
{
    let mut all_test_cases = Vec::new();

    // Load all test cases from specified files
    for filename in filenames {
        match load_test_cases::<I>(filename) {
            Ok(mut cases) => {
                // Add the filename to each test case's name for uniqueness
                let file_base = filename.split('.').next().unwrap_or("");
                for case in &mut cases {
                    // Only append the filename if there are multiple files and the name might conflict
                    if filenames.len() > 1 {
                        case.name = format!("{}_{}", case.name, file_base);
                    }
                }
                all_test_cases.append(&mut cases);
            }
            Err(e) => panic!("Failed to load test cases from {}: {}", filename, e),
        }
    }

    // Run each test case
    for test_case in all_test_cases {
        let test_case_name = test_case.name.replace(' ', "_");
        let result = test_fn(&test_case.input, &test_case_name);
        let test_result = TestResult::from(result);

        // Create snapshot name
        let snapshot_name = format!("{}_{}", test_name, test_case_name);

        // Use insta for snapshot testing
        insta::with_settings!({sort_maps => true}, {
            insta::assert_json_snapshot!(snapshot_name, test_result);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Serialize, Deserialize)]
    struct SimpleInput {
        value: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct SimpleOutput {
        result: String,
    }

    #[test]
    fn test_test_error() {
        let error = TestError::new("test error message");
        assert_eq!(error.error, "test error message");
    }

    #[test]
    fn test_test_result_success() {
        let result: TestResult<SimpleOutput> = TestResult::Success(SimpleOutput {
            result: "success".to_string(),
        });

        match result {
            TestResult::Success(output) => assert_eq!(output.result, "success"),
            TestResult::Error(_) => panic!("Expected success"),
        }
    }

    #[test]
    fn test_test_result_error() {
        let result: TestResult<SimpleOutput> = TestResult::Error(TestError::new("test error"));

        match result {
            TestResult::Success(_) => panic!("Expected error"),
            TestResult::Error(err) => assert_eq!(err.error, "test error"),
        }
    }

    #[test]
    fn test_test_result_from_result_ok() {
        let ok_result: Result<SimpleOutput, Box<dyn std::error::Error>> = Ok(SimpleOutput {
            result: "success".to_string(),
        });
        let test_result = TestResult::from(ok_result);

        match test_result {
            TestResult::Success(output) => assert_eq!(output.result, "success"),
            TestResult::Error(_) => panic!("Expected success"),
        }
    }

    #[test]
    fn test_test_result_from_result_err() {
        let err_result: Result<SimpleOutput, Box<dyn std::error::Error>> = Err("test error".into());
        let test_result = TestResult::from(err_result);

        match test_result {
            TestResult::Success(_) => panic!("Expected error"),
            TestResult::Error(err) => assert_eq!(err.error, "test error"),
        }
    }
}
