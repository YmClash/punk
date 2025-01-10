// #[cfg(test)]
// mod tests {
//     use nom::Parser;
//     use num_bigint::BigInt;
//     use pyrust::parser::ast::{Declaration, Expression, Literal, Mutability, Statement, Type, BinaryOperation, Operator};
//     use pyrust::parser::parser::Parser;
//     use pyrust::{Lexer, SyntaxMode};
//     use BigInt;
//
//
//     use pyrust::tok::{Delimiters, Keywords, Operators, TokenType};
//     use super::*;
//
//
//
//     #[test]
//     fn test_complex_expressions() {
//         let test_cases = vec![
//             ("5 + 3 * 2", 11),
//             ("(10 - 4) / 2", 3),
//             ("2 * (3 + 4)", 14),
//             ("15 - 5 * 2 + 3", 8),
//             // ("true && (5 > 3)", true),
//             // ("3.14 <= 3.14 || false", true),
//         ];
//
//         for (input, expected_result) in test_cases {
//             let mut lexer = Lexer::new(input, SyntaxMode::Braces);
//             let tokens = lexer.tokenize();
//             let mut parser = Parser::new(tokens, SyntaxMode::Braces);
//
//             match parser.parse_expression() {
//                 Ok(ast) => {
//                     // Ici, vous devriez évaluer l'AST pour obtenir le résultat
//                     // et le comparer avec expected_result
//                     // Par exemple :
//                     // let result = evaluate_ast(ast);
//                     // assert_eq!(result, expected_result, "Failed for input: {}", input);
//                 },
//                 Err(e) => panic!("Error parsing expression '{}': {:?}", input, e),
//             }
//         }
//

    // Vous devrez implémenter cette fonction pour évaluer l'AST
    // fn evaluate_ast(ast: Expression) -> Result<Value, String> {
    //     // Implémentez l'évaluation de l'AST ici
    //     todo!()
    // }


    // fn create_parser(source: &str, syntax_mode: SyntaxMode) -> Parser {
    //     let mut lexer = Lexer::new(source, syntax_mode);
    //     let tokens = lexer.tokenize();
    //     Parser::new(tokens, syntax_mode)
    // }
    //
    // fn assert_type_eq(actual: &Type, expected: &Type) {
    //     match (actual, expected) {
    //         (Type::Int, Type::Int) => {},
    //         (Type::Float, Type::Float) => {},
    //         (Type::String, Type::String) => {},
    //         (Type::Bool, Type::Bool) => {},
    //         (Type::Char, Type::Char) => {},
    //         (Type::Array(a), Type::Array(b)) => assert_type_eq(a, b),
    //         (Type::Tuple(a), Type::Tuple(b)) => {
    //             assert_eq!(a.len(), b.len());
    //             for (a_type, b_type) in a.iter().zip(b.iter()) {
    //                 assert_type_eq(a_type, b_type);
    //             }
    //         },
    //         (Type::Custom(a), Type::Custom(b)) => assert_eq!(a, b),
    //         (Type::Generic(a), Type::Generic(b)) => {
    //             assert_eq!(a.base, b.base);
    //             assert_eq!(a.parameters.len(), b.parameters.len());
    //             for (a_param, b_param) in a.parameters.iter().zip(b.parameters.iter()) {
    //                 assert_type_eq(a_param, b_param);
    //             }
    //         },
    //         (Type::Infer, Type::Infer) => {},
    //         _ => panic!("Types do not match: {:?} != {:?}", actual, expected),
    //     }
    // }

    // #[test]
    // fn test_multiple_declarations() {
    //     let input = r#"
    //         let x: int = 5;
    //         const y: float = 3.14;
    //         let mut z: string = "hello";
    //     "#;
    //
    //     let mut lexer = Lexer::new(input, SyntaxMode::Braces);
    //     let tokens = lexer.tokenize();
    //     let mut parser = Parser::new(tokens, SyntaxMode::Braces);
    //
    //     let declarations = parser.parse();
    //
    //     assert!(declarations.is_ok());
    //     let declarations = declarations.unwrap();
    //
    //     assert_eq!(declarations.len(), 3);
    //
    //     // Vérifiez chaque déclaration individuellement
    //     match &declarations[0] {
    //         Statement::Declaration(Declaration::Variable(var)) => {
    //             assert_eq!(var.name, "x");
    //             assert_eq!(var.variable_type, Some(Type::Int));
    //             assert_eq!(var.mutability, Mutability::Immutable);
    //             // Vérifiez la valeur
    //         },
    //         _ => panic!("Expected variable declaration"),
    //     }
    //
    //     match &declarations[1] {
    //         Statement::Declaration(Declaration::Constante(cons)) => {
    //             assert_eq!(cons.name, "y");
    //             assert_eq!(cons.constant_type, Some(Type::Float));
    //             // Vérifiez la valeur
    //         },
    //         _ => panic!("Expected constant declaration"),
    //     }
    //
    //     match &declarations[2] {
    //         Statement::Declaration(Declaration::Variable(var)) => {
    //             assert_eq!(var.name, "z");
    //             assert_eq!(var.variable_type, Some(Type::String));
    //             assert_eq!(var.mutability, Mutability::Mutable);
    //             // Vérifiez la valeur
    //         },
    //         _ => panic!("Expected variable declaration"),
    //     }
    // }








//     #[test]
//     fn test_variable_declaration_indentation() {
//         let source = "let x = 5\n";
//         let mut parser = create_parser(source, SyntaxMode::Indentation);
//         let result = parser.parse_variable_declaration();
//         assert!(result.is_ok());
//         if let Ok(Declaration::Variable(var_decl)) = result {
//             assert_eq!(var_decl.name, "x");
//             assert_eq!(var_decl.mutability, mutability);
//             assert!(matches!(var_decl.value, Some(Expression::Literal(Literal::Integer { value })) if value == 5.into()));
//         } else {
//             panic!("Expected variable declaration");
//         }
//     }
//
//     #[test]
//     fn test_variable_declaration_braces() {
//         let source = "let x = 5;";
//         let mut parser = create_parser(source, SyntaxMode::Braces);
//         let result = parser.parse_variable_declaration();
//         assert!(result.is_ok());
//         if let Ok(Declaration::Variable(var_decl)) = result {
//             assert_eq!(var_decl.name, "x");
//             assert_eq!(var_decl.mutable, false);
//             assert!(matches!(var_decl.value, Some(Expression::Literal(Literal::Integer { value })) if value == 5.into()));
//         } else {
//             panic!("Expected variable declaration");
//         }
//     }
//
//     #[test]
//     fn test_function_declaration_indentation() {
//         let source = "fn test_func():\n    return 0\n";
//         let mut parser = create_parser(source, SyntaxMode::Indentation);
//         let result = parser.parse_function_declaration();
//         assert!(result.is_ok());
//         if let Ok(Declaration::Function(func_decl)) = result {
//             assert_eq!(func_decl.name, "test_func");
//             assert!(func_decl.parameters.is_empty());
//             assert!(func_decl.return_type.is_none());
//             assert_eq!(func_decl.body.statements.len(), 1);
//         } else {
//             panic!("Expected function declaration");
//         }
//     }
//
//     #[test]
//     fn test_function_declaration_braces() {
//         let source = "fn test_func() { return 0; }";
//         let mut parser = create_parser(source, SyntaxMode::Braces);
//         let result = parser.parse_function_declaration();
//         assert!(result.is_ok());
//         if let Ok(Declaration::Function(func_decl)) = result {
//             assert_eq!(func_decl.name, "test_func");
//             assert!(func_decl.parameters.is_empty());
//             assert!(func_decl.return_type.is_none());
//             assert_eq!(func_decl.body.statements.len(), 1);
//         } else {
//             panic!("Expected function declaration");
//         }
//     }
//
//     #[test]
//     fn test_variable_declaration_with_type_annotation() {
//         let source = "let x: int = 5;";
//         let mut parser = create_parser(source, SyntaxMode::Braces);
//         let result = parser.parse_variable_declaration();
//         assert!(result.is_ok());
//         if let Ok(Declaration::Variable(var_decl)) = result {
//             assert_eq!(var_decl.name, "x");
//             assert_eq!(var_decl.mutable, false);
//             assert!(matches!(var_decl.variable_type, Some(Type::Int)));
//             assert!(matches!(var_decl.value, Some(Expression::Literal(Literal::Integer { value })) if value == 5.into()));
//         } else {
//             panic!("Expected variable declaration with type annotation");
//         }
//     }
//
//     #[test]
//     fn test_function_declaration_with_parameters_and_return_type() {
//         let source = "fn add(a: int, b: int) -> int:\n    return a + b\n";
//         let mut parser = create_parser(source, SyntaxMode::Indentation);
//         let result = parser.parse_function_declaration();
//         assert!(result.is_ok());
//         if let Ok(Declaration::Function(func_decl)) = result {
//             assert_eq!(func_decl.name, "add");
//             assert_eq!(func_decl.parameters.len(), 2);
//             assert!(matches!(func_decl.return_type, Some(Type::Int)));
//             assert_eq!(func_decl.body.statements.len(), 1);
//         } else {
//             panic!("Expected function declaration with parameters and return type");
//         }
//     }
//
//     #[test]
//     fn test_variable_declaration_error() {
//         let source = "let 5 = x;";
//         let mut parser = create_parser(source, SyntaxMode::Braces);
//         let result = parser.parse_variable_declaration();
//         assert!(result.is_err());
//     }
//
//     #[test]
//     fn test_function_declaration_error() {
//         let source = "fn () { return 0; }";
//         let mut parser = create_parser(source, SyntaxMode::Braces);
//         let result = parser.parse_function_declaration();
//         assert!(result.is_err());
//     }
//     // ///////////////////////////////////////////////////////////
//
//     // // Tests pour les déclarations de constantes
//     #[test]
//     fn test_constant_declaration_indentation() {
//         let source = "const PI: float = 3.14159\n";
//         let mut parser = create_parser(source, SyntaxMode::Indentation);
//         let result = parser.parse_constant_declaration();
//         assert!(result.is_ok());
//         if let Ok(Declaration::Constante(const_decl)) = result {
//             assert_eq!(const_decl.name, "PI");
//
//             // Vérification du type
//             match const_decl.constant_type {
//                 Some(Type::Float) => (), // C'est ce qu'on attend
//                 _ => panic!("Expected Float type for constant"),
//             }
//
//             // Vérification de la valeur
//             match const_decl.value {
//                 Expression::Literal(Literal::Float { value }) => {
//                     assert!((value - 3.14159).abs() < f64::EPSILON);
//                 },
//                 _ => panic!("Expected Float literal for constant value"),
//             }
//         } else {
//             panic!("Expected constant declaration");
//         }
//     }
//     #[test]
//     fn test_constant_declaration_braces() {
//         let source = "const MAX_VALUE = 100;";
//         let mut parser = create_parser(source, SyntaxMode::Braces);
//         let result = parser.parse_constant_declaration();
//         assert!(result.is_ok());
//         if let Ok(Declaration::Constante(const_decl)) = result {
//             assert_eq!(const_decl.name, "MAX_VALUE");
//
//             // Vérification du type
//             assert!(const_decl.constant_type.is_none(), "Expected no type annotation");
//
//             // Vérification de la valeur
//             match const_decl.value {
//                 Expression::Literal(Literal::Integer { value }) => {
//                     assert_eq!(value, 100.into());
//                 },
//                 _ => panic!("Expected Integer literal for constant value"),
//             }
//         } else {
//             panic!("Expected constant declaration");
//         }
//     }
// //////////////////////////////
//     // // Tests pour les déclarations de structures
//
//     #[test]
//     fn test_struct_declaration_indentation() {
//         let source = "struct Point { x: int, y: int }\n";
//         let mut parser = create_parser(source, SyntaxMode::Indentation);
//         let result = parser.parse_struct_declaration();
//         assert!(result.is_ok(), "Failed to parse struct declaration: {:?}", result.err());
//
//         if let Ok(Declaration::Structure(struct_decl)) = result {
//             assert_eq!(struct_decl.name, "Point");
//             assert_eq!(struct_decl.fields.len(), 2);
//             assert_eq!(struct_decl.fields[0].name, "x");
//             assert!(matches!(struct_decl.fields[0].field_type, Type::Int));
//             assert_eq!(struct_decl.fields[1].name, "y");
//             assert!(matches!(struct_decl.fields[1].field_type, Type::Int));
//         } else {
//             panic!("Expected struct declaration");
//         }
//     }
//
//
//     #[test]
//     fn test_struct_declaration_braces() {
//         let source = "struct Person { name: str, age: int }";
//         let mut parser = create_parser(source, SyntaxMode::Braces);
//         let result = parser.parse_struct_declaration();
//         assert!(result.is_ok());
//         if let Ok(Declaration::Structure(struct_decl)) = result {
//             assert_eq!(struct_decl.name, "Person");
//             assert_eq!(struct_decl.fields.len(), 2);
//             assert_eq!(struct_decl.fields[0].name, "name");
//             assert!(matches!(struct_decl.fields[0].field_type, Type::String));
//             assert_eq!(struct_decl.fields[1].name, "age");
//             assert!(matches!(struct_decl.fields[1].field_type, Type::Int));
//         } else {
//             panic!("Expected struct declaration");
//         }
//     }
//
//     #[test]
//     fn test_struct_declaration_empty() {
//         let source = "struct Empty {}";
//         let mut parser = create_parser(source, SyntaxMode::Braces);
//         let result = parser.parse_struct_declaration();
//         assert!(result.is_ok());
//         if let Ok(Declaration::Structure(struct_decl)) = result {
//             assert_eq!(struct_decl.name, "Empty");
//             assert_eq!(struct_decl.fields.len(), 0);
//         } else {
//             panic!("Expected struct declaration");
//         }
//     }
//
//     #[test]
//     fn test_struct_declaration_error() {
//         let source = "struct InvalidStruct {";  // Accolade fermante manquante
//         let mut parser = create_parser(source, SyntaxMode::Braces);
//         let result = parser.parse_struct_declaration();
//         assert!(result.is_err());
//     }
// ///////////////////////////MULTILINE STRUCT DECLARATION////////////////////////////
//     // #[test]
//     // fn test_struct_declaration_indentation_multiline() {
//     //     let source = "struct Complex {\n    real: float,\n    imag: float\n}\n";
//     //     let mut parser = create_parser(source, SyntaxMode::Indentation);
//     //     let result = parser.parse_struct_declaration();
//     //     assert!(result.is_ok(), "Failed to parse multiline struct declaration: {:?}", result.err());
//     //
//     //     if let Ok(Declaration::Structure(struct_decl)) = result {
//     //         assert_eq!(struct_decl.name, "Complex");
//     //         assert_eq!(struct_decl.fields.len(), 2);
//     //         assert_eq!(struct_decl.fields[0].name, "real");
//     //         assert!(matches!(struct_decl.fields[0].field_type, Type::Float));
//     //         assert_eq!(struct_decl.fields[1].name, "imag");
//     //         assert!(matches!(struct_decl.fields[1].field_type, Type::Float));
//     //     } else {
//     //         panic!("Expected multiline struct declaration");
//     //     }
//     // }
// //////////////////////////////A TESTER A LA FIN /////////////////////////////////////
//
//     #[test]
//     fn test_class_declaration_indentation() {
//         let source = "class MyClass(ParentClass):
//         x: int
//         y: float
//         fn method1(self) -> int:
//             return 42
//         fn method2(self, a: int) -> float:
//             return 3.14
//             ";
//         let mut parser = create_parser(source, SyntaxMode::Indentation);
//         let result = parser.parse_class_declaration();
//
//         assert!(result.is_ok());
//         if let Ok(Declaration::Class(class_decl)) = result {
//             assert_eq!(class_decl.name, "MyClass");
//             assert_eq!(class_decl.parent_class, Some("ParentClass".to_string()));
//
//             // Vérifier les champs
//             assert_eq!(class_decl.fields.len(), 2);
//             assert_eq!(class_decl.fields[0].name, "x");
//             //assert_type_eq(&class_decl.fields[0].field_type, &Type::Int);
//             assert_eq!(&class_decl.fields[0].field_type, &Type::Int);
//             assert_eq!(class_decl.fields[1].name, "y");
//             //assert_type_eq(&class_decl.fields[1].field_type, &Type::Float);
//             assert_eq!(&class_decl.fields[1].field_type, &Type::Float);
//
//             // Vérifier les méthodes
//             assert_eq!(class_decl.methods.len(), 2);
//             assert_eq!(class_decl.methods[0].name, "method1");
//             assert_eq!(class_decl.methods[1].name, "method2");
//         } else {
//             panic!("Expected class declaration");
//         }
//     }
//
//     #[test]
//     fn test_class_declaration_braces() {
//         let source = r#"
// class SimpleClass {
//     z: string;
//     fn simple_method(self) {
//         return "Hello";
//     }
// }
// "#;
//         let mut parser = create_parser(source, SyntaxMode::Braces);
//         let result = parser.parse_class_declaration();
//
//         assert!(result.is_ok());
//         if let Ok(Declaration::Class(class_decl)) = result {
//             assert_eq!(class_decl.name, "SimpleClass");
//             assert_eq!(class_decl.parent_class, None);
//
//             assert_eq!(class_decl.fields.len(), 1);
//             assert_eq!(class_decl.fields[0].name, "z");
//             // assert_type_eq(&class_decl.fields[0].field_type, &Type::String);
//             assert_eq!(&class_decl.fields[0].field_type, &Type::String);
//
//             assert_eq!(class_decl.methods.len(), 1);
//             assert_eq!(class_decl.methods[0].name, "simple_method");
//         } else {
//             panic!("Expected class declaration");
//         }
//     }

    // #[test]
    // fn test_class_declaration_indentation() {
    //     let source = "class MyClass(ParentClass):\n\
    //                   \tx: int\n\
    //                   \ty: float\n\
    //                   \tfn method1(self) -> int:\n\
    //                   \t\treturn 42\n\
    //                   \tfn method2(self, a: int) -> float:\n\
    //                   \t\treturn 3.14\n";
    //     let mut parser = create_parser(source, SyntaxMode::Indentation);
    //     let result = parser.parse_class_declaration();
    //
    //     assert!(result.is_ok(), "Failed to parse class declaration: {:?}", result.err());
    //     if let Ok(Declaration::Class(class_decl)) = result {
    //         assert_eq!(class_decl.name, "MyClass");
    //         assert_eq!(class_decl.parent_class, Some("ParentClass".to_string()));
    //
    //         // Vérifier les champs
    //         assert_eq!(class_decl.fields.len(), 2);
    //         assert_eq!(class_decl.fields[0].name, "x");
    //         assert_type_eq(&class_decl.fields[0].field_type, &Type::Int);
    //         assert_eq!(class_decl.fields[1].name, "y");
    //         assert_type_eq(&class_decl.fields[1].field_type, &Type::Float);
    //
    //         // Vérifier les méthodes
    //         assert_eq!(class_decl.methods.len(), 2);
    //         assert_eq!(class_decl.methods[0].name, "method1");
    //         assert_eq!(class_decl.methods[1].name, "method2");
    //     } else {
    //         panic!("Expected class declaration");
    //     }
    // }
    //
    // #[test]
    // fn test_class_declaration_braces() {
    //     let source = "class SimpleClass {\n\
    //                   \tz: string;\n\
    //                   \tfn simple_method(self) {\n\
    //                   \t\treturn \"Hello\";\n\
    //                   \t}\n\
    //                   }\n";
    //     let mut parser = create_parser(source, SyntaxMode::Braces);
    //     let result = parser.parse_class_declaration();
    //
    //     assert!(result.is_ok(), "Failed to parse class declaration: {:?}", result.err());
    //     if let Ok(Declaration::Class(class_decl)) = result {
    //         assert_eq!(class_decl.name, "SimpleClass");
    //         assert_eq!(class_decl.parent_class, None);
    //
    //         assert_eq!(class_decl.fields.len(), 1);
    //         assert_eq!(class_decl.fields[0].name, "z");
    //         assert_type_eq(&class_decl.fields[0].field_type, &Type::String);
    //
    //         assert_eq!(class_decl.methods.len(), 1);
    //         assert_eq!(class_decl.methods[0].name, "simple_method");
    //     } else {
    //         panic!("Expected class declaration");
    //     }
    // }
    //




    // #[test]
    // fn test_constant_declaration_braces() {
    //     let source = "const PI: float = 3.14159;";
    //     let mut parser = create_parser(source, SyntaxMode::Braces);
    //     let result = parser.parse_constant_declaration();
    //     assert!(result.is_ok());
    //     if let Ok(Declaration::Constante(const_decl)) = result {
    //         assert_eq!(const_decl.name, "PI");
    //         assert!(matches!(const_decl.constant_type, Some(Type::Float)));
    //         assert!(matches!(const_decl.value, Expression::Literal(Literal::Float { value }) if (value - 3.14159).abs() < f64::EPSILON));
    //     } else {
    //         panic!("Expected constant declaration");
    //     }
    // }
    //
    // #[test]
    // fn test_constant_declaration_indentation() {
    //     let source = "const E = 2.71828\n";
    //     let mut parser = create_parser(source, SyntaxMode::Indentation);
    //     let result = parser.parse_constant_declaration();
    //     assert!(result.is_ok());
    //     if let Ok(Declaration::Constante(const_decl)) = result {
    //         assert_eq!(const_decl.name, "E");
    //         assert!(const_decl.constant_type.is_none());
    //         assert!(matches!(const_decl.value, Expression::Literal(Literal::Float { value }) if (value - 2.71828).abs() < f64::EPSILON));
    //     } else {
    //         panic!("Expected constant declaration");
    //     }
    // }
    //
    // #[test]
    // fn test_constant_declaration_error() {
    //     let source = "const 123 = 456;";
    //     let mut parser = create_parser(source, SyntaxMode::Braces);
    //     let result = parser.parse_constant_declaration();
    //     assert!(result.is_err());
    // }
    // //////////////////////////////////////////////////////////////////////////

    // // Tests pour les déclarations de structures
    // #[test]
    // fn test_struct_declaration_braces() {
    //     let source = "struct Point { x: int, y: int }";
    //     let mut parser = create_parser(source, SyntaxMode::Braces);
    //     let result = parser.parse_struct_declaration();
    //     assert!(result.is_ok());
    //     if let Ok(Declaration::Structure(struct_decl)) = result {
    //         assert_eq!(struct_decl.name, "Point");
    //         assert_eq!(struct_decl.fields.len(), 2);
    //         assert_eq!(struct_decl.fields[0].name, "x");
    //         assert_eq!(struct_decl.fields[0].parameter_type, Some("int".to_string()));
    //         assert_eq!(struct_decl.fields[1].name, "y");
    //         assert_eq!(struct_decl.fields[1].parameter_type, Some("int".to_string()));
    //     } else {
    //         panic!("Expected struct declaration");
    //     }
    // }
    //
    // #[test]
    // fn test_struct_declaration_indentation() {
    //     let source = "struct Person:\n    name: str\n    age: int\n";
    //     let mut parser = create_parser(source, SyntaxMode::Indentation);
    //     let result = parser.parse_struct_declaration();
    //     assert!(result.is_ok());
    //     if let Ok(Declaration::Structure(struct_decl)) = result {
    //         assert_eq!(struct_decl.name, "Person");
    //         assert_eq!(struct_decl.fields.len(), 2);
    //         assert_eq!(struct_decl.fields[0].name, "name");
    //         assert_eq!(struct_decl.fields[0].parameter_type, Some("str".to_string()));
    //         assert_eq!(struct_decl.fields[1].name, "age");
    //         assert_eq!(struct_decl.fields[1].parameter_type, Some("int".to_string()));
    //     } else {
    //         panic!("Expected struct declaration");
    //     }
    // }
    //
    // // Tests pour les déclarations de classes
    // #[test]
    // fn test_class_declaration_braces() {
    //     let source = "class Rectangle { width: float; height: float; fn area() -> float { return self.width * self.height; } }";
    //     let mut parser = create_parser(source, SyntaxMode::Braces);
    //     let result = parser.parse_class_declaration();
    //     assert!(result.is_ok());
    //     if let Ok(Declaration::Class(class_decl)) = result {
    //         assert_eq!(class_decl.name, "Rectangle");
    //         assert_eq!(class_decl.fields.len(), 2);
    //         assert_eq!(class_decl.methods.len(), 1);
    //         assert_eq!(class_decl.methods[0].name, "area");
    //     } else {
    //         panic!("Expected class declaration");
    //     }
    // }
    //
    // #[test]
    // fn test_class_declaration_indentation() {
    //     let source = "class Circle:\n    radius: float\n    fn circumference() -> float:\n        return 2 * 3.14159 * self.radius\n";
    //     let mut parser = create_parser(source, SyntaxMode::Indentation);
    //     let result = parser.parse_class_declaration();
    //     assert!(result.is_ok());
    //     if let Ok(Declaration::Class(class_decl)) = result {
    //         assert_eq!(class_decl.name, "Circle");
    //         assert_eq!(class_decl.fields.len(), 1);
    //         assert_eq!(class_decl.methods.len(), 1);
    //         assert_eq!(class_decl.methods[0].name, "circumference");
    //     } else {
    //         panic!("Expected class declaration");
    //     }
    // }
    //
    // // Tests pour les déclarations d'énumérations
    // #[test]
    // fn test_enum_declaration_braces() {
    //     let source = "enum Color { Red, Green, Blue }";
    //     let mut parser = create_parser(source, SyntaxMode::Braces);
    //     let result = parser.parse_enum_declaration();
    //     assert!(result.is_ok());
    //     if let Ok(Declaration::Enum(enum_decl)) = result {
    //         assert_eq!(enum_decl.name, "Color");
    //         assert_eq!(enum_decl.variants.len(), 3);
    //         assert_eq!(enum_decl.variants, vec!["Red", "Green", "Blue"]);
    //     } else {
    //         panic!("Expected enum declaration");
    //     }
    // }
    //
    // #[test]
    // fn test_enum_declaration_indentation() {
    //     let source = "enum DaysOfWeek:\n    Monday\n    Tuesday\n    Wednesday\n    Thursday\n    Friday\n";
    //     let mut parser = create_parser(source, SyntaxMode::Indentation);
    //     let result = parser.parse_enum_declaration();
    //     assert!(result.is_ok());
    //     if let Ok(Declaration::Enum(enum_decl)) = result {
    //         assert_eq!(enum_decl.name, "DaysOfWeek");
    //         assert_eq!(enum_decl.variants.len(), 5);
    //         assert_eq!(enum_decl.variants, vec!["Monday", "Tuesday", "Wednesday", "Thursday", "Friday"]);
    //     } else {
    //         panic!("Expected enum declaration");
    //     }
    // }
    //
    // // Tests pour les cas d'erreur
    // #[test]
    // fn test_struct_declaration_error() {
    //     let source = "struct 123 { x: int }";
    //     let mut parser = create_parser(source, SyntaxMode::Braces);
    //     let result = parser.parse_struct_declaration();
    //     assert!(result.is_err());
    // }
    //
    // #[test]
    // fn test_class_declaration_error() {
    //     let source = "class { field: int }";
    //     let mut parser = create_parser(source, SyntaxMode::Braces);
    //     let result = parser.parse_class_declaration();
    //     assert!(result.is_err());
    // }
    //
    // #[test]
    // fn test_enum_declaration_error() {
    //     let source = "enum Color { Red, 123, Blue }";
    //     let mut parser = create_parser(source, SyntaxMode::Braces);
    //     let result = parser.parse_enum_declaration();
    //     assert!(result.is_err());
    // }


// }