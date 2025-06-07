use patina::parser::parse_file;
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

    let input = match fs::read_to_string(filename) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Error reading file '{filename}': {err}");
            process::exit(1);
        }
    };

    let ast = match parse_file(&input) {
        Ok(ast) => ast,
        Err(err) => {
            eprintln!("Parse error: {err}");
            process::exit(1);
        }
    };

    println!("=== AST Structure ===");
    println!("AST created with {} entities", ast.world.len());

    println!("=== Parse Success ===");
    println!("File '{filename}' parsed successfully!");
    println!(
        "AST contains {} top-level statements",
        ast.root.statements.len()
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
        write!(temp_file, "static {{\n    let x = 42\n}}").unwrap();

        let path = temp_file.path().to_str().unwrap();

        let content = fs::read_to_string(path).unwrap();
        println!("Input content: {:?}", content);
        let result = parse_file(&content);
        if let Err(ref e) = result {
            println!("Parse error: {:?}", e);
        }
        assert!(result.is_ok());
    }

    #[test]
    fn test_main_with_invalid_input() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "invalid syntax here {{{{").unwrap();

        let path = temp_file.path().to_str().unwrap();

        let content = fs::read_to_string(path).unwrap();
        let _result = parse_file(&content);
    }
}
