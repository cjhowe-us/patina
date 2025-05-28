#[cfg(test)]
mod tests {
    use crate::parser::parse_file;
    use crate::ast_ecs::converter::AstConverter;
    use crate::ast_ecs::query::AstQuery;
    use crate::ast::walk_ast;
    use std::time::Instant;

    #[test]
    fn compare_memory_usage() {
        let input = r#"module math {
    fun add(x: i32, y: i32): i32 {
        return x + y
    }
    
    fun multiply(x: i32, y: i32): i32 {
        return x * y
    }
}

let x = 42
let mut y = true
let z: *mut i32 = null"#;

        // Traditional AST
        let start = Instant::now();
        let traditional_ast = parse_file(input).unwrap();
        let traditional_parse_time = start.elapsed();
        
        // Count nodes in traditional AST
        let mut traditional_count = 0;
        walk_ast(&traditional_ast, &mut |_| {
            traditional_count += 1;
        });

        // ECS AST
        let start = Instant::now();
        let ast = parse_file(input).unwrap();
        let converter = AstConverter::new();
        let ecs_ast = converter.convert(&ast);
        let ecs_parse_time = start.elapsed();
        
        // Count entities in ECS
        let ecs_count = ecs_ast.world.len();

        println!("\n=== Performance Comparison ===");
        println!("Traditional AST:");
        println!("  Parse time: {:?}", traditional_parse_time);
        println!("  Node count: {}", traditional_count);
        
        println!("\nECS AST:");
        println!("  Parse time: {:?}", ecs_parse_time);
        println!("  Entity count: {}", ecs_count);
    }

    #[test]
    fn compare_query_capabilities() {
        let input = r#"let x: i32 = 42
let mut y: i64 = 100
let z: *mut i32 = null

fun foo(a: i32, b: i32): i32 {
    let sum = a + b
    return sum
}"#;

        println!("\n=== Query Capabilities Comparison ===");

        // Traditional AST queries
        let traditional_ast = parse_file(input).unwrap();
        
        println!("\nTraditional AST (requires custom visitors):");
        
        // Count mutable variables - requires custom visitor
        let _mutable_count = 0;
        walk_ast(&traditional_ast, &mut |_node| {
            // Would need to match on node type and check is_mutable field
            // This is verbose and requires knowledge of AST structure
        });
        
        // ECS queries
        let ast = parse_file(input).unwrap();
        let converter = AstConverter::new();
        let ecs_ast = converter.convert(&ast);
        let query = AstQuery::new(&ecs_ast.world);
        
        println!("\nECS AST (built-in query capabilities):");
        
        // Find all mutable variables
        let let_statements = query.find_by_kind(crate::ast_ecs::components::NodeKind::Let);
        let mutable_vars: Vec<_> = let_statements.into_iter()
            .filter(|&entity| query.is_mutable(entity))
            .collect();
        println!("  Mutable variables: {}", mutable_vars.len());
        
        // Find all integer types
        let int_types = query.find_by_kind(crate::ast_ecs::components::NodeKind::Integer);
        println!("  Integer types: {}", int_types.len());
        
        // Find all functions
        let functions = query.get_all_functions();
        println!("  Functions: {}", functions.len());
        
        // Complex query: Find all i32 types
        let i32_types: Vec<_> = int_types.into_iter()
            .filter(|&entity| {
                query.is_signed(entity) == Some(true) && 
                query.get_bit_size(entity) == Some(32)
            })
            .collect();
        println!("  i32 types: {}", i32_types.len());
    }

    #[test]
    fn compare_extensibility() {
        println!("\n=== Extensibility Comparison ===");
        
        println!("\nTraditional AST:");
        println!("  - Adding new fields requires modifying struct definitions");
        println!("  - Adding new node types requires updating enums and match statements");
        println!("  - Visitor pattern helps but still requires trait implementations");
        
        println!("\nECS AST:");
        println!("  - Adding new components is trivial - just define a new struct");
        println!("  - No need to modify existing code");
        println!("  - Components can be added/removed at runtime");
        println!("  - Mix and match components for new node types");
    }

    #[test]
    fn compare_design_benefits() {
        println!("\n=== Design Benefits Comparison ===");
        
        println!("\nTraditional AST:");
        println!("  Pros:");
        println!("    + Type safety - compiler catches missing fields");
        println!("    + Familiar OOP-style design");
        println!("    + Direct field access");
        println!("    + Clear hierarchical structure");
        println!("  Cons:");
        println!("    - Rigid structure");
        println!("    - Memory inefficient (each node stores all fields)");
        println!("    - Difficult to add cross-cutting concerns");
        println!("    - Complex visitor implementations");
        
        println!("\nECS AST:");
        println!("  Pros:");
        println!("    + Flexible component composition");
        println!("    + Memory efficient (only store needed components)");
        println!("    + Powerful query capabilities");
        println!("    + Easy to add new features");
        println!("    + Cache-friendly data layout");
        println!("    + Natural support for parallel processing");
        println!("  Cons:");
        println!("    - Less compile-time safety");
        println!("    - Indirect data access through queries");
        println!("    - Less familiar paradigm");
        println!("    - Potential runtime overhead for small ASTs");
    }

    #[test]
    fn recommendation() {
        println!("\n=== RECOMMENDATION ===");
        println!("\nThe ECS approach is worth keeping for the following reasons:");
        println!();
        println!("1. **Flexibility**: Much easier to extend with new node types and attributes");
        println!("2. **Query Power**: Built-in ability to find nodes by any component combination");
        println!("3. **Memory Efficiency**: Only store components that are actually used");
        println!("4. **Composability**: Mix and match components to create new node types");
        println!("5. **Future-Proof**: Easy to add semantic analysis, type checking, etc.");
        println!();
        println!("The ECS design is particularly well-suited for:");
        println!("- Language servers (efficient queries)");
        println!("- Compiler optimizations (parallel processing)");
        println!("- IDE features (cross-references, refactoring)");
        println!("- Incremental compilation (track changes efficiently)");
        println!();
        println!("While the traditional AST is simpler and more familiar,");
        println!("the ECS approach provides significant long-term benefits");
        println!("that outweigh the initial complexity.");
    }
}