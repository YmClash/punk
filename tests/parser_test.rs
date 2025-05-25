#[cfg(test)]
mod tests {
    use punk::parser::parser::Parser;
    use punk::{Lexer, SyntaxMode};
    use punk::parser::ast::Expression;


    // Fonction d'aide pour créer un parser
    fn create_parser(source: &str, mode: SyntaxMode) -> Parser {
        let mut lexer = Lexer::new(source, mode);
        let tokens = lexer.tokenize();
        Parser::new(tokens, mode)
    }

    // fn assert_ast_eq(actual: Expression, expected: Expression) {
    //     assert_eq!(format!("{:#?}", actual), format!("{:#?}", expected));
    // }

    mod expression_tests {

        use super::*;


        #[test]
        fn test_literal_expressions_braces() {
            let input = "42";
            let mut parser = create_parser(input, SyntaxMode::Braces);
            let result = parser.parse_expression(0);
            assert!(result.is_ok());
        }

        #[test]
        fn test_literal_expressions_indent() {
            let input = "42+2";
            let mut parser = create_parser(input, SyntaxMode::Indentation);
            let result = parser.parse_expression(0);
            assert!(result.is_ok());
        }


        #[test]
        fn test_complex_expressions() {
            let test_cases = vec![
                "foo.bar()",
                "array[index]",
                "obj.method().field[i]",
                "(a + b) * (c - d)",
            ];

            for input in test_cases {
                let mut parser = create_parser(input, SyntaxMode::Braces);
                assert!(parser.parse_expression(0).is_ok());
            }
        }


    }

    mod try_except_statement_tests{
        use super::*;

        #[test]
        fn test_try_except_braces() {
            let input = r#"
        try {
            risky_function();
        } except Error as e {
            handle_error(e);
        }
        "#;
            let mut parser = create_parser(input, SyntaxMode::Braces);
            let result = parser.parse_try_statement();
            assert!(result.is_ok());
        }

        #[test]
        fn test_try_except_indent() {

            let input = r#"try:
    risky_function()
except Error as e:
    handle_error(e)
"#;
            let mut parser = create_parser(input, SyntaxMode::Indentation);
            let result = parser.parse_try_statement();
            assert!(result.is_ok());
        }

        #[test]
        fn test_try_except_finally() {
            let input = r#"
        try {
            risky_function();
        } except Error {
            handle_error();
        } finally {
            cleanup();
        }
        "#;
            let mut parser = create_parser(input, SyntaxMode::Braces);
            let result = parser.parse_try_statement();
            assert!(result.is_ok());
        }
    }
    mod if_elif_else_statement_tests{
        use super::*;

        #[test]
        fn test_if_elif_else_brace() {
            let input = r#"if x > 0 { print("if"); } elif x < 0 {print("elif");}else{print("else");}"#;
            let mut parser = create_parser(input, SyntaxMode::Braces);
            let result = parser.parse_if_statement();
            assert!(result.is_ok());
        }
        #[test]
        fn test_if_elif_else_indent() {
            let input = r#"if x > 0 :
    print("if")
elif x < 0:
    print("elif")
else :
    print("else")
"#;
            let mut parser = create_parser(input, SyntaxMode::Indentation);
            let result = parser.parse_if_statement();
            assert!(result.is_ok());
        }

        #[test]
        fn test_multiple_elif_braces() {
            let input = r#"if x > 0 { a(); } elif x < 0 { b(); } elif x == 0 { c(); } else { d(); };"#;
            let mut parser = create_parser(input, SyntaxMode::Braces);
            let result = parser.parse_if_statement();
            assert!(result.is_ok());
        }
        #[test]
        fn test_multiple_elif_indent() {
            let input = r#"if x > 0:
    a()
elif x < 0:
    b()
elif x == 0:
    c()
else :
    d()
"#;
            let mut parser = create_parser(input, SyntaxMode::Indentation);
            let result = parser.parse_if_statement();
            assert!(result.is_ok());
        }

    }


    mod test_array_declaration_tests{
        use super::*;

        #[test]
        fn test_array_declaration_braces() {
            let input = r#"let arr = [1, 2, 3];"#;
            let mut parser = create_parser(input, SyntaxMode::Braces);
            let result = parser.parse_variable_declaration();
            assert!(result.is_ok());
        }

        #[test]
        fn test_array_declaration_indent() {
            let input = r#"let arr = [1, 2, 3]"#;
            let mut parser = create_parser(input, SyntaxMode::Indentation);
            let result = parser.parse_variable_declaration();
            assert!(result.is_ok());
        }

        #[test]
        fn test_nested_array_declaration_braces() {
            let input = r#"let arr = [[1, 2], [3, 4]];"#;
            let mut parser = create_parser(input, SyntaxMode::Braces);
            let result = parser.parse_variable_declaration();
            assert!(result.is_ok());
        }

        #[test]
        fn test_nested_array_declaration_indent() {
            let input = r#"let arr = [[1, 2], [3, 4]]"#;
            let mut parser = create_parser(input, SyntaxMode::Indentation);
            let result = parser.parse_variable_declaration();
            assert!(result.is_ok());
        }

        #[test]
        fn test_empty_array_declaration_braces() {
            let input = r#"let arr = [];"#;
            let mut parser = create_parser(input, SyntaxMode::Braces);
            let result = parser.parse_variable_declaration();
            assert!(result.is_ok());
        }

        #[test]
        fn test_empty_array_declaration_indent() {
            let input = r#"let arr = []"#;
            let mut parser = create_parser(input, SyntaxMode::Indentation);
            let result = parser.parse_variable_declaration();
            assert!(result.is_ok());
        }

    }

    mod test_list_comprehesion {
        use super::*;
        #[test]
        fn test_list_comprehension_braces() {

            let tests = r#"[x * 2 for x in range(10)]"#;

            let mut parser = create_parser(tests, SyntaxMode::Braces);
            let result = parser.parse_list_comprehension();
            assert!(result.is_ok());
        }

        #[test]
        fn test_list_comprehension_indent() {


            let tests = r#"[x * 2 for x in range(10)]"#;

            let mut parser = create_parser(tests, SyntaxMode::Indentation);
            let result = parser.parse_list_comprehension();
            assert!(result.is_ok());
        }

    }

    mod test_dictionary_declaration_tests{
        use super::*;

        #[test]
        fn test_dictionary_declaration_braces() {
            let input = r#"let dict = {2 + 2: "four", "array": [1, 2, 3]};"#;
            let mut parser = create_parser(input, SyntaxMode::Braces);
            let result = parser.parse_variable_declaration();
            assert!(result.is_ok());
        }

        #[test]
        fn test_dictionary_declaration_indent() {
            let input = r#"let dict = {2 + 2: "four", "array": [1, 2, 3]}"#;
            let mut parser = create_parser(input, SyntaxMode::Indentation);
            let result = parser.parse_variable_declaration();
            assert!(result.is_ok());
        }

        #[test]
        fn test_nested_dictionary_declaration_braces() {
            let input = r#"let dict = { "key": { "nested": "value" } };"#;
            let mut parser = create_parser(input, SyntaxMode::Braces);
            let result = parser.parse_variable_declaration();
            assert!(result.is_ok());
        }

        #[test]
        fn test_nested_dictionary_declaration_indent() {
            let input = r#"let dict = { "key": { "nested": "value" } }"#;
            let mut parser = create_parser(input, SyntaxMode::Indentation);
            let result = parser.parse_variable_declaration();
            assert!(result.is_ok());
        }

        #[test]
        fn test_empty_dictionary_declaration_braces() {
            let input = r#"let dict = {};"#;
            let mut parser = create_parser(input, SyntaxMode::Braces);
            let result = parser.parse_variable_declaration();
            assert!(result.is_ok());
        }

        #[test]
        fn test_empty_dictionary_declaration_indent() {
            let input = r#"let dict = {}"#;
            let mut parser = create_parser(input, SyntaxMode::Indentation);
            let result = parser.parse_variable_declaration();
            assert!(result.is_ok());
        }

    }





    mod declaration_tests {
        use punk::parser::ast::Visibility;
        use super::*;

        #[test]
        fn test_variable_declarations() {
            let test_cases = vec![
                "let x = 42;",
                "let mut y: int = 10;",
                "let z: float = 3.14;",
                "let s = \"hello\";",
            ];

            for input in test_cases {
                let mut parser = create_parser(input, SyntaxMode::Braces);
                assert!(parser.parse_variable_declaration().is_ok());
            }
        }

        #[test]
        fn test_function_declarations_braces() {
            let test_cases = vec![
                "fn foo() { }",
                "fn add(x: int, y: int) -> int { x + y }",

            ];

            for input in test_cases {
                let mut parser = create_parser(input, SyntaxMode::Braces);
                assert!(parser.parse_function_declaration(Visibility::Public).is_ok());
            }
        }

        #[test]
        fn test_function_declarations_indent() {
            let test_cases = r#"fn add(x: int, y: int) -> int:
    return x + y"#;

            let mut parser = create_parser(test_cases, SyntaxMode::Indentation);
            assert!(parser.parse_function_declaration(Visibility::Public).is_ok());
        }

        #[test]
        fn test_divers_variable_declarations_braces(){
            let input = r#"let x = 10;let mut y:int = 3;const numb = 5;pub const x:int = 5;pub struct Point {x: int,y: int}pub struct Point {height: int,width: int}enum Color {x:int,y:float,z:str}pub enum Color {pub x:int,y:float,z:str}pub fn add(x: int, y: int) -> int {return x + y}pub fn add(x: int, y: int) -> int {let mut result = x + y;}"#;

            let mut parser = create_parser(input, SyntaxMode::Braces);
            let result = parser.parse_program();
            assert!(result.is_ok());
        }


        #[test]
        fn test_divers_variable_declarations_indent(){
            let input = "let x = 10\nlet mut y:int = 3\nconst numb = 5\npub const x:int = 5\nstruct Point {x: int,y: int}pub struct Point {height: int,width: int} enum Color {x:int,y:float,z:str}pub enum Color {pub x:int,y:float,z:str}";
            let mut parser = create_parser(input, SyntaxMode::Indentation);
            let result = parser.parse_program();
            assert!(result.is_ok());
        }
    }



    mod pattern_matching_tests {
        use super::*;

        #[test]
        fn test_simple_patterns_braces() {
            let input = r#"match x {
                n if n > 0 => print("positive"),
                n if n < 0 => print("negative"),
                _ => print("zero")
            }
            "#;
            let mut parser = create_parser(input, SyntaxMode::Braces);
            assert!(parser.parse_match_statement().is_ok());
        }

        #[test]
        fn test_simple_patterns_indent() {
            let input = r#"match x :
    n if n > 0 => print("positive")
    (x, y) => print("tuple simple")
    [1, 2] => print("array simple")
    _ => print("default")
"#;
            let mut parser = create_parser(input, SyntaxMode::Indentation);
            assert!(parser.parse_match_statement().is_ok());
        }

        #[test]
        fn test_complex_patterns_braces() {
            let input = r#"match x {
            1 => print(1),2 => print(2),
            [0, 0] => print("Origin"),
            (0, y) if y > 0 => print("Positive Y-axis"),
            (x, y) => print("MOMO"),

            _ => print("other")}"#;

            let mut parser = create_parser(input, SyntaxMode::Braces);
            assert!(parser.parse_match_statement().is_ok());

        }

        #[test]
        fn test_complex_patterns_indent() {
            let input = r#"match x :
    [0, 0] => print("Origin")
    [x, 0]:
        print("X-axis")
        print(x)
    [0, y] if y > 0 => print("Positive Y-axis")
    _ => print("Other")
"#;

            let mut parser = create_parser(input, SyntaxMode::Indentation);
            assert!(parser.parse_match_statement().is_ok());
        }


    }


    mod lambda_tests{
        use super::*;

        #[test]
        fn test_lambda_braces() {
            // let input = r#"let add = |x, y| x + y;"#;
            let input =r#"lambda (x: int, y: int) -> int {x + y};"#;
            let mut parser = create_parser(input, SyntaxMode::Braces);
            // let result = parser.parse_variable_declaration();
            let result = parser.parse_lambda_expression();
            assert!(result.is_ok());
        }


        // Test unitaire pour la fonction lambda n'est pas encore bien  implemente.
        // Je vais y remedier plus tard

        #[test]
        fn test_lambda_indent() {
            let input = r#"lambda (x: int, y: int) -> int:{x + y}"#;
            let mut parser = create_parser(input, SyntaxMode::Indentation);
            // let result = parser.parse_lambda_expression();
            let result = parser.parse_program();
            assert!(result.is_ok());
    }



}
    mod control_flow_tests {
        use super::*;

        #[test]
        fn test_if_statements_braces() {
            let test_cases =  r#"if x > 0 {print("hello");}else{print("Nothing");}"#;

            let mut parser = create_parser(test_cases, SyntaxMode::Braces);
            assert!(parser.parse_if_statement().is_ok());
        }

        #[test]
        fn test_loops_indent_with_label() {
            let test_cases = r#"counter:loop:
    print("infini")
    x += 1
if x > 10:
    break
"#;

            let mut parser = create_parser(test_cases, SyntaxMode::Indentation);
            assert!(parser.parse_loop_statement().is_ok());


        }

        #[test]
        fn test_loops_braces_with_label() {
            let test_cases = r#"counter: loop {print("infini"),x += 1,if x > 10 {break;}}"#;

            let mut parser = create_parser(test_cases, SyntaxMode::Braces);
            assert!(parser.parse_loop_statement().is_ok());
        }
    }


    mod error_tests {
        use super::*;

        #[test]
        fn test_syntax_errors() {
            let test_cases = vec![
                "let;", // Missing identifier
                "let x;", // Missing initializer
                "fn {}", // Missing function name
                "match {}", // Missing match expression
            ];

            for input in test_cases {
                let mut parser = create_parser(input, SyntaxMode::Braces);
                assert!(parser.parse_statement().is_err());
            }
        }

        #[test]
        fn test_recovery() {
            let input = r#"
            let x = ; // Error
            let y = 42; // Should parse correctly
            "#;
            let mut parser = create_parser(input, SyntaxMode::Braces);
            assert!(parser.parse_program().is_ok());
            // Test error recovery
        }
    }

    mod integration_tests {
        use punk::SyntaxMode;
        use super::*;

        #[test]
        fn test_complete_file_braces() {
            let input = r#"
            fn main() {
                let x = 42;
                if x > 0 {
                    print("positive");
                }

                match x {
                    n if n > 0 => print("positive"),
                    _ => print("other")
                }
            }
            "#;
            let mut parser = create_parser(input, SyntaxMode::Braces);
            assert!(parser.parse_program().is_ok());
        }

        #[test]
        fn test_complete_file_indent() {
            let input = r#"let x = 42
if x > 0:
    print("positive")
match x:
    n if n > 0 => print("positive")
    _ => print("other")
"#;


            //             r#"fn main():
            //     let x = 42
            //     if x > 0:
            //         print("positive")
            //
            // match x:
            //                     n if n > 0 => print("positive")
            //                     _ => print("other")
            //             "#;


            let mut parser = create_parser(input, SyntaxMode::Indentation);
            assert!(parser.parse_program().is_ok());
        }
    }

    mod fonction_declaration_tests{
        use punk::parser::ast::Visibility;
        use super::*;

        #[test]
        fn test_function_declaration_braces() {
            let input = r#"fn add(x: int, y: int) -> int {let mut result = x + y;return result}"#;
            let mut parser = create_parser(input, SyntaxMode::Braces);
            let result = parser.parse_function_declaration(Visibility::Public);
            assert!(result.is_ok());
        }

        #[test]
        fn test_function_declaration_indent(){
            let input = r#"fn add(x: int, y: int) -> int:
    let mut result = x + y
    return result"#;
            let mut parser = create_parser(input, SyntaxMode::Indentation);
            let result = parser.parse_function_declaration(Visibility::Public);
            assert!(result.is_ok());
        }
    }

    mod access_call_tests{
        use super::*;

        #[test]
        fn test_function_call_method_braces(){
            let input = r#"chat.danse(x,y);"#;
            let mut parser = create_parser(input, SyntaxMode::Braces);
            let result = parser.parse_expression(0);
            assert!(result.is_ok());
        }

        #[test]
        fn test_function_call_method_indent(){
            let input = r#"chat.danse(x,y)"#;
            let mut parser = create_parser(input, SyntaxMode::Indentation);
            let result = parser.parse_expression(0);
            assert!(result.is_ok());
        }

        #[test]
        fn test_complex_function_call_method_braces(){
            // let input = r#"chat.danse(x,y).parle(z).mange(a,b);"#;
            let input = r#"obj.method1().field.method2(1+2);"#;
            let mut parser = create_parser(input, SyntaxMode::Braces);
            let result = parser.parse_expression(0);
            assert!(result.is_ok());
        }

        #[test]

        fn test_complex_function_call_method_indent(){
            let input = r#"obj.method1().field.method2(1+2)"#;
            let mut parser = create_parser(input, SyntaxMode::Indentation);
            let result = parser.parse_expression(0);
            assert!(result.is_ok());
        }

        #[test]
        fn test_array_access_braces(){
            let input = r#"array[index];"#;
            let mut parser = create_parser(input, SyntaxMode::Braces);
            let result = parser.parse_expression(0);
            assert!(result.is_ok());
        }

        #[test]
        fn test_array_access_indent(){
            let input = r#"array[index]"#;
            let mut parser = create_parser(input, SyntaxMode::Indentation);
            let result = parser.parse_expression(0);
            assert!(result.is_ok());
        }

        #[test]
        fn test_complex_array_access_braces(){
            let input = r#"array[index].field[index];"#;
            let mut parser = create_parser(input, SyntaxMode::Braces);
            let result = parser.parse_expression(0);
            assert!(result.is_ok());
        }

        #[test]
        fn test_complex_array_access_indent(){
            let input = r#"array[index].field[index]"#;
            let mut parser = create_parser(input, SyntaxMode::Indentation);
            let result = parser.parse_expression(0);
            assert!(result.is_ok());
        }

    }

    mod destructuring_and_compound_tests{
        use super::*;


        #[test]
        fn test_destructuring_braces(){
            let input = r#"[x,y,z] = point3d;"#;
            let mut parser = create_parser(input, SyntaxMode::Braces);
            // let result = parser.parse_variable_declaration();
            let result = parser.parse_expression(0);
            assert!(result.is_ok());
        }

        #[test]
        fn test_destructuring_indent(){
            let input = r#"[x,y,z] = point3d"#;
            let mut parser = create_parser(input, SyntaxMode::Indentation);
            // let result = parser.parse_variable_declaration();
            let result = parser.parse_expression(0);
            assert!(result.is_ok());
        }

        #[test]
        fn test_compound_braces(){
            let input = r#"counter += offset * 5"#;
            let mut parser = create_parser(input, SyntaxMode::Braces);
            // let result = parser.parse_variable_declaration();
            let result = parser.parse_expression(0);
            assert!(result.is_ok());
        }

        #[test]
        fn test_compound_indent(){
            let input = r#"counter += offset * 5"#;
            let mut parser = create_parser(input, SyntaxMode::Indentation);
            // let result = parser.parse_variable_declaration();
            let result = parser.parse_expression(0);
            assert!(result.is_ok());
        }

        #[test]
        fn test_assign_compound_braces(){
            let input = r#"a = b = c = 0;"#;
            let mut parser = create_parser(input, SyntaxMode::Braces);
            // let result = parser.parse_variable_declaration();
            let result = parser.parse_expression(0);
            assert!(result.is_ok());
        }

        #[test]
        fn test_assign_compound_indent(){
            let input = r#"a = b = c = 0"#;
            let mut parser = create_parser(input, SyntaxMode::Indentation);
            // let result = parser.parse_variable_declaration();
            let result = parser.parse_expression(0);
            assert!(result.is_ok());
        }

    }

    mod trait_tests {
        use punk::parser::ast::Visibility;
        use super::*;

        #[test]
        fn test_trait_declaration_braces() {
            let input = r#"trait Drawable  {fn do_something(x: T) -> int;fn area(a:float)->float;fn do_something_else(x: char) -> int;type Color;}"#;

            let mut parser = create_parser(input, SyntaxMode::Braces);
            let result = parser.parse_trait_declaration(Visibility::Public);
            assert!(result.is_ok());
        }

        #[test]
        fn test_trait_declaration_indent() {
            let input = r#"trait Drawable:
    fn do_something(x: int) -> int
    fn do_something_else(x: int) -> int
    fn area() -> float
    type Color"#;

            let mut parser = create_parser(input, SyntaxMode::Indentation);
            let result = parser.parse_trait_declaration(Visibility::Public);
            assert!(result.is_ok());
        }
    }
    mod impl_tests {
        use punk::parser::ast::Visibility;
        use super::*;

        #[test]
        fn test_trait_impl_braces() {
            let input = r#"impl<T> Drawable for MyType<T>{
    fn draw(x:int) {
        return self.x+1}
    fn get_color() -> int {
        return color.code()
        }
    }"#;

            let mut parser = create_parser(input, SyntaxMode::Braces);
            let result = parser.parse_impl_declaration(Visibility::Private);
            assert!(result.is_ok());
        }

        #[test]
        fn test_trait_impl_indent() {
            let input = r#"impl<T> Drawable for MyType<T> where D: Display:
    fn draw(x: T) -> bool:
        return self.x + 1"#;

            let mut parser = create_parser(input, SyntaxMode::Indentation);
            let result = parser.parse_impl_declaration(Visibility::Private);
            assert!(result.is_ok());
        }

    }

    mod struct_tests {
        use punk::parser::ast::Visibility;
        use super::*;

        #[test]
        fn test_struct_declaration_braces() {
            let input = r#"struct Point {x: int,y: int};"#;

            let mut parser = create_parser(input, SyntaxMode::Braces);
            let result = parser.parse_struct_declaration(Visibility::Public);
            assert!(result.is_ok());
        }

        #[test]
        fn test_struct_declaration_indent() {
            let input = r#"struct Point {x: int,y: int}"#;

            let mut parser = create_parser(input, SyntaxMode::Indentation);
            let result = parser.parse_struct_declaration(Visibility::Public);
            assert!(result.is_ok());
        }
    }

    mod enum_tests {
        use punk::parser::ast::Visibility;
        use super::*;

        #[test]
        fn test_enum_declaration_braces() {
            let input = r#"enum Color {x:int,y:float,z:str}"#;

            let mut parser = create_parser(input, SyntaxMode::Braces);
            let result = parser.parse_enum_declaration(Visibility::Public);
            assert!(result.is_ok());
        }

        #[test]
        fn test_enum_declaration_indent() {
            let input = r#"enum Color {x:int,y:float,z:str}"#;

            let mut parser = create_parser(input, SyntaxMode::Indentation);
            let result = parser.parse_enum_declaration(Visibility::Public);
            assert!(result.is_ok());
        }
    }
}
