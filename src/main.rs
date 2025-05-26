use patina::parser::parse_file;
use patina::visitor::{walk, DebugPrinter, ValidationVisitor};
use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <input-file>", args[0]);
        process::exit(1);
    }

    let filename = &args[1];

    // Read the input file
    let input = match fs::read_to_string(filename) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Error reading file '{}': {}", filename, err);
            process::exit(1);
        }
    };

    // Parse the input
    let root = match parse_file(&input) {
        Ok(ast) => ast,
        Err(err) => {
            eprintln!("Parse error: {}", err);
            process::exit(1);
        }
    };

    // Validate the AST
    let mut validator = ValidationVisitor::new();
    if let Err(err) = walk(&mut validator, None, &root) {
        eprintln!("Validation error: {}", err);
        process::exit(1);
    }

    if validator.has_errors() {
        eprintln!("Validation failed:");
        for error in &validator.errors {
            eprintln!("  - {}", error);
        }
        process::exit(1);
    }

    // Print debug information
    let mut printer = DebugPrinter::new();
    if let Err(err) = walk(&mut printer, None, &root) {
        eprintln!("Debug printing error: {}", err);
        process::exit(1);
    }

    println!("=== AST Structure ===");
    println!("{}", printer.output);

    println!("=== Parse Success ===");
    println!("File '{}' parsed successfully!", filename);
    println!(
        "AST contains {} top-level statements",
        root.statements.len()
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_main_with_valid_input() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "module test {{ let x = 42 }}").unwrap();

        let path = temp_file.path().to_str().unwrap();

        // Test that parsing succeeds
        let content = fs::read_to_string(path).unwrap();
        let result = parse_file(&content);
        assert!(result.is_ok());
    }

    #[test]
    fn test_main_with_invalid_input() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "invalid syntax here {{{{").unwrap();

        let path = temp_file.path().to_str().unwrap();

        // Test that parsing fails
        let content = fs::read_to_string(path).unwrap();
        let result = parse_file(&content);
        assert!(result.is_err());
    }
}
