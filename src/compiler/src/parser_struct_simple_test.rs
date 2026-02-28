#[cfg(test)]
mod simple_struct_tests {
    use crate::lexer::tokenize_with_locations;
    use crate::parser::Parser;
    use crate::ast::{AstNode, Statement, Type, Visibility};

    #[test]
    fn test_basic_struct_definition() {
        let source = "struct Point { x: i32, y: i32 }";
        let tokens = tokenize_with_locations(source, None);
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();
        
        assert_eq!(ast.len(), 1);
        match &ast[0] {
            AstNode::Statement(Statement::Struct { name, generics, fields, is_tuple }) => {
                assert_eq!(name, "Point");
                assert_eq!(generics.len(), 0);
                assert_eq!(fields.len(), 2);
                assert!(!is_tuple);
                
                // Check first field
                assert_eq!(fields[0].name, "x");
                assert_eq!(fields[0].field_type, Type::Named("i32".to_string()));
                assert!(matches!(fields[0].visibility, Visibility::Private));
                
                // Check second field
                assert_eq!(fields[1].name, "y");
                assert_eq!(fields[1].field_type, Type::Named("i32".to_string()));
                assert!(matches!(fields[1].visibility, Visibility::Private));
            }
            _ => panic!("Expected struct statement"),
        }
    }

    #[test]
    fn test_tuple_struct_definition() {
        let source = "struct Point(i32, i32);";
        let tokens = tokenize_with_locations(source, None);
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();
        
        match &ast[0] {
            AstNode::Statement(Statement::Struct { name, generics, fields, is_tuple }) => {
                assert_eq!(name, "Point");
                assert_eq!(generics.len(), 0);
                assert_eq!(fields.len(), 2);
                assert!(*is_tuple);
                
                // Check fields (indexed by position)
                assert_eq!(fields[0].name, "0");
                assert_eq!(fields[0].field_type, Type::Named("i32".to_string()));
                assert!(matches!(fields[0].visibility, Visibility::Public));
                
                assert_eq!(fields[1].name, "1");
                assert_eq!(fields[1].field_type, Type::Named("i32".to_string()));
                assert!(matches!(fields[1].visibility, Visibility::Public));
            }
            _ => panic!("Expected struct statement"),
        }
    }

    #[test]
    fn test_generic_struct_definition() {
        let source = "struct Container<T> { value: T }";
        let tokens = tokenize_with_locations(source, None);
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();
        
        match &ast[0] {
            AstNode::Statement(Statement::Struct { name, generics, fields, .. }) => {
                assert_eq!(name, "Container");
                assert_eq!(generics.len(), 1);
                assert_eq!(generics[0], "T");
                assert_eq!(fields.len(), 1);
                assert_eq!(fields[0].field_type, Type::Named("T".to_string()));
            }
            _ => panic!("Expected struct statement"),
        }
    }
}