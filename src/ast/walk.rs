use crate::ast::SyntaxNode;

/// Generic walk function that traverses the AST using the SyntaxNode interface.
/// This demonstrates how the visitor pattern can be implemented using just the
/// SyntaxNode trait without needing match statements for each node type.
pub fn walk_ast<F>(node: &dyn SyntaxNode, visitor: &mut F)
where
    F: FnMut(&dyn SyntaxNode),
{
    visitor(node);

    for child in node.children() {
        walk_ast(child, visitor);
    }
}

/// Walk function with pre- and post-order callbacks
pub fn walk_ast_with_callbacks<Pre, Post>(
    node: &dyn SyntaxNode,
    pre_visitor: &mut Pre,
    post_visitor: &mut Post,
) where
    Pre: FnMut(&dyn SyntaxNode),
    Post: FnMut(&dyn SyntaxNode),
{
    pre_visitor(node);

    for child in node.children() {
        walk_ast_with_callbacks(child, pre_visitor, post_visitor);
    }

    post_visitor(node);
}

/// Walk function that can be terminated early
pub fn walk_ast_until<F>(node: &dyn SyntaxNode, visitor: &mut F) -> bool
where
    F: FnMut(&dyn SyntaxNode) -> bool,
{
    if !visitor(node) {
        return false;
    }

    for child in node.children() {
        if !walk_ast_until(child, visitor) {
            return false;
        }
    }

    true
}

/// Walk function that collects results
pub fn walk_ast_map<'a, T, F>(node: &'a dyn SyntaxNode, visitor: &mut F) -> Vec<T>
where
    F: FnMut(&'a dyn SyntaxNode) -> Option<T>,
{
    let mut results = Vec::new();

    if let Some(result) = visitor(node) {
        results.push(result);
    }

    for child in node.children() {
        results.extend(walk_ast_map(child, visitor));
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Position, Root};

    #[test]
    fn test_walk_ast() {
        let root = Root {
            statements: Vec::new(),
            pos: Position::default(),
        };

        let mut visited = Vec::new();
        walk_ast(&root, &mut |node| {
            visited.push(node.position().clone());
        });

        assert_eq!(visited.len(), 1);
        assert_eq!(visited[0].line, 1);
        assert_eq!(visited[0].column, 1);
    }

    #[test]
    fn test_walk_ast_count() {
        let root = Root {
            statements: Vec::new(),
            pos: Position::default(),
        };

        let mut count = 0;
        walk_ast(&root, &mut |_| count += 1);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_walk_until() {
        let root = Root {
            statements: Vec::new(),
            pos: Position::default(),
        };

        let mut count = 0;
        let completed = walk_ast_until(&root, &mut |_| {
            count += 1;
            true // Continue walking
        });

        assert!(completed);
        assert_eq!(count, 1);
    }
}
