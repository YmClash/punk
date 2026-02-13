//src/main.rs
#![allow(dead_code)]
#![allow(unused)]

use punk::interpreter;
use punk::interpreter::interpreter::Interpreter;
use punk::lexer::lex::{Lexer, Token};
use punk::lexer::lex::SyntaxMode;
use punk::parser::parser::Parser;
use punk::parser::ast::{ASTNode, Declaration, VariableDeclaration, FunctionDeclaration, ConstDeclaration,Expression,Literal};
use punk::semantic::analyser::SemanticAnalyzer;
use punk::SyntaxMode::{Braces, Indentation};

fn mode(syntax_mode: SyntaxMode){
    match syntax_mode {
        SyntaxMode::Braces => println!("Mode Braces"),
        SyntaxMode::Indentation => println!("Mode Indentation"),
    }
}

fn print_separator(title: &str) {
    println!("\n{:-^60}", format!(" {} ", title));
}

// Nouvelle fonction main avec menu de tests
fn main() {

    env_logger::init();

    println!("╔════════════════════════════════════════╗");
    println!("║     PunkLang Compiler Test Suite       ║");
    println!("║           Dual-Mode Parser             ║");
    println!("║              By YmC                    ║");
    println!("╚════════════════════════════════════════╝");

    // // let syntaxe_mode = SyntaxMode::Braces;
    // let syntax_mode = SyntaxMode::Indentation;


    fn mode(syntax_mode: SyntaxMode){
        match syntax_mode {
            SyntaxMode::Braces => println!("Mode Braces"),
            SyntaxMode::Indentation => println!("Mode Indentation"),
        }
    }




    // Test comparaison des AST avec et sans point-virgules
    //println!("\n=== Test: Comparing ASTs with/without semicolons ===\n");

    // Test 1: Simple let statement

    let code_source = r#"fn fibonacci(n: int) -> int:
    if n <= 1:
        return n
    let mut a = 0
    let mut b = 1
    let mut i = 2
    while i <= n:
        let temp = a + b
        a = b
        b = temp
        i = i + 1
        return b
    let result = fibonacci(10)"#;

    let complex_brace = r#"
    //Exemple
    # Fibonacci en mode Braces
    let x: int = 5;
    let y = 10;
    if x < y {
        print("x is less than y");
    } else {
        print("x is not less than y");
    }
    fn fibonnaci(n:int) -> int {
        while n > 0 {
            n = n - 1;
        }
        return fibonnaci(n - 1) + fibonnaci(n - 2)
    }"#;

    let code = r#"fn add(a, b):
    return a + b

let result = add(5, 7)
println("Result:", result)"#;


    let code_2 = r#"
let nom = "momo"
println("Hello from PunkLang!")
println("Je suis:", nom)
let a = 10
let b = 10
if a > b:
    println("a is greater")
elif b > a:
    print("b is greater")
else:
    println("a is equal to b")
    println("resultat:", a * b)
    "#;

    let code_3 = r#"
    let nom = "momo";
    println("Hello from PunkLang!");
    println("Je suis:", nom);
    let a = 10;
    let b = 10 ;
    if a > b {
        println("a is greater");
    }
    elif b > a {
        println("b is greater");
    }
    else {
        println("a is equal to b");
        println("resultat:", a * b);
    }
    "#;

    let code_4 = r#"
    fn add(a, b) {
    return a + b;
    }
    fn multiply(a, b) {
    return a * b;
    }
    let a = 5;
    let b = 7;

    let use_addition = false;
    if use_addition {
        println("Condition is true, using addition");
        println("Resultat ", add(a, b));
    } else {
        println("Condition is false, using multiplication");
        println("Resultat ", multiply(a, b));
    }
    "#;

    let code_5 = r#"
    let mut i = 0;
    while i < 5 {
        println("i:", i);
        i = i + 1;
    }
    println("Loop finished, final i:", i);

    "#;

    let code_6 = r#"
    let my_array = [10, 20, 30];
    for item in my_array{
        println("Item:", item);
    }
    println("Loop finished, final array:", my_array);


    "#;

    let code_7 = r#"
    let my_array = [10, 20, 30];
    let first_item = my_array[0];
    let second_item = my_array[1];
    let third_item = my_array[2];
    println("First item:", first_item);
    println("Second item:", second_item);
    println("Third item:", third_item);
    "#;




    print_separator("PunkLang Source Code Mode Braces");
    println!("{}", code_7);


    // --- 1. Lexical Analysis ---
    print_separator("Lexical Analysis");
    let mut lexer = Lexer::new(code_7,Braces);
    let tokens = lexer.tokenize();
    println!("Tokenization completed. Total tokens: {}", tokens.len());
    // dbg!(&tokens);

    // Affichage des tokens pour vérification
    for (i, tok) in tokens.iter().enumerate() {
        println!("{}:{:?}", i, tok);
    }
    println!("\n");
    // --- 2. Syntax Analysis ---
    print_separator("Syntax Analysis and AST Generation");

    let mut parser = Parser::new(tokens,Braces);

    match parser.parse_program() {
        Ok(ast) => {
            println!("\n ✅ AST généré avec succès!");
            println!("{:#?}", ast);
            // dbg!(ast);

            // Semantic analysis
            print_separator("Analyse sémantique");

            let mut analyser = SemanticAnalyzer::new();

            // Activer le système de récupération d'erreurs
            use std::rc::Rc;
            use std::cell::RefCell;
            use punk::semantic::context::CompilationContext;

            let context = Rc::new(RefCell::new(CompilationContext::new()));
            analyser.enable_error_recovery(context.clone());

            let ast_nodes = match ast {
                ASTNode::Program(nodes) => nodes,
                single => vec![single],
            };

            match analyser.analyze(&ast_nodes) {
                Ok(()) => {
                    println!("✅ Analyse sémantique réussie!");
                    let stats = analyser.get_analysis_stats();
                    println!("Statistiques: {} symboles, {} types, {} scopes, {} warnings",
                             stats.total_symbols, stats.total_types, stats.total_scopes,stats.warning_count);

                    if stats.warning_count > 0 {
                        println!("⚠️  {} warnings détectés durant l'analyse.", stats.warning_count);
                        for warning in analyser.get_warnings(){
                            println!("  - Warning: {:?}", warning.message);
                        }
                    }

                    // Appliquer les optimisations
                    let mut optimized_ast = ast_nodes.clone();
                    analyser.optimize_ast(&mut optimized_ast, context.clone());
                }
                Err(errors) => {
                    println!("❌ Échec de l'analyse sémantique avec {} erreurs:", errors.len());
                    for (i, e) in errors.iter().enumerate() {
                        println!("Erreur {}: {:?}", i+1, e);
                    }
                }
            }
            print_separator("Interpretation PunkLang");
            // if analyser.is_successful() {
            //     print_separator("Interpretation");
            //     let mut interpreter = Interpreter::new();
            //     match interpreter.interpret(&ast_nodes) {
            //         Ok(final_value) => {
            //             println!("✅ Interpretation successful!");
            //             println!("🏁 Final Result: {:?}", final_value);
            //         }
            //         Err(e) => {
            //             println!("❌ Runtime Error: {}", e);
            //         }
            //     }
            // } else {
            //     print_separator("Interpretation Skipped");
            //     println!("Execution halted due to semantic errors.");
            // }
            let mut interpreter = Interpreter::new();
            match interpreter.interpret(&ast_nodes) {
                Ok(final_value) => {
                    println!("");
                    println!("✅ Interpretation successful!");
                    println!("🏁 Final Result: {:?}", final_value);
                }
                Err(e) => {
                    println!("❌ Runtime Error: {}", e);
                }
            }



        }
        Err(e) => {
            println!("❌ Erreur lors du parsing : {}", e);
        }
    }

    // --- 4. Interpretation ---
    // print_separator("Interpretation");
    // let mut interpreter = Interpreter::new();
    // match interpreter.interpret(&ast_nodes) {
    //     Ok(final_value) => {
    //         println!("✅ Interpretation successful!");
    //         println!("🏁 Final Result: {:?}", final_value);
    //     }
    //     Err(e) => {
    //         println!("❌ Runtime Error: {}", e);
    //     }
    // }

    print_separator("Execution Finished");


    print_separator("TEST SUITE COMPLETED");
    println!("PunkLang Compiler by YmC");
    println!();


}




// fn run_file(filename:&str){
//     use std::fs;
//
//     let code_source = fs::read_to_string(filename)
//         .expect("Failed to read source file");
//
//     let mut lexer = Lexer::new(&code_source, SyntaxMode::Indentation);
//     let tokens = lexer.tokenize();
//
//     let mut parser = Parser::new(tokens, SyntaxMode::Indentation);
//     let ast = parser.parse_program().expect("Failed to parse program");
//
//     let mut analyser = SemanticAnalyzer::new();
//     let ast_nodes = match ast {
//         ASTNode::Program(nodes) => nodes,
//         single => vec![single],
//     };
//     analyser.analyze(&ast_nodes).expect("Semantic analysis failed");
//
//     let mut evaluator = interpreter::evaluator::Evaluator::new();
//     match evaluator.eval(&ast_nodes) {
//         Ok(_) => println!("Program executed successfully."),
//         Err(e) => eprintln!("Runtime error: {}", e),
//     }
//
// }



// Fonction pour garder le test original
fn run_original_test() {
    let syntax_mode = SyntaxMode::Indentation;

    println!("Mode de syntaxe : ");
    mode(syntax_mode);

    let code_source = r#"match x :
    (0, 0) => print("Origin")
    (x, 0):
        print("X-axis")
        print(x)
    (0, y) if y > 0 => print("Positive Y-axis")
    (x, y) => print("MOMO")
    _ => print("Other")
"#;

    print_separator("Analyse lexicale et tokenization");
    
    let mut lexer = Lexer::new(code_source, syntax_mode);
    let tokens = lexer.tokenize();
    
    // Affichage des tokens pour vérification
    for (i, tok) in tokens.iter().enumerate() {
        println!("{}:{:?}", i, tok);
    }
    println!("\n");
    
    print_separator("Analyse syntaxique et génération de l'AST");
    
    let mut parser = Parser::new(tokens, syntax_mode);
    
    match parser.parse_program() {
        Ok(ast) => {
            println!("✅ AST généré avec succès!");
            println!("{:#?}", ast);
            
            // Semantic analysis
            print_separator("Analyse sémantique");
            
            let mut analyser = SemanticAnalyzer::new();
            
            // Activer le système de récupération d'erreurs
            use std::rc::Rc;
            use std::cell::RefCell;
            use punk::semantic::context::CompilationContext;
            
            let context = Rc::new(RefCell::new(CompilationContext::new()));
            analyser.enable_error_recovery(context.clone());
            
            let ast_nodes = match ast {
                ASTNode::Program(nodes) => nodes,
                single => vec![single],
            };
            
            match analyser.analyze(&ast_nodes) {
                Ok(()) => {
                    println!("✅ Analyse sémantique réussie!");
                    let stats = analyser.get_analysis_stats();
                    println!("Statistiques: {} symboles, {} types, {} scopes",
                            stats.total_symbols, stats.total_types, stats.total_scopes);
                    
                    // Appliquer les optimisations
                    let mut optimized_ast = ast_nodes.clone();
                    analyser.optimize_ast(&mut optimized_ast, context.clone());
                }
                Err(errors) => {
                    println!("❌ Échec de l'analyse sémantique avec {} erreurs:", errors.len());
                    for (i, e) in errors.iter().enumerate() {
                        println!("Erreur {}: {:?}", i+1, e);
                    }
                }
            }
        }
        Err(e) => {
            println!("❌ Erreur lors du parsing : {}", e);
        }
    }
}

// Fonction pour benchmark (optionnelle)
#[allow(dead_code)]
fn benchmark_modes() {
    use std::time::Instant;
    
    let code_brace = r#"
fn test(n: int) -> int {
    let mut sum = 0;
    for i in range(n) {
        sum = sum + i;
    }
    return sum;
}"#;
    
    let code_indent = r#"
fn test(n: int) -> int:
    let mut sum = 0
    for i in range(n):
        sum = sum + i
    return sum"#;
    
    // Benchmark Brace Mode
    let start = Instant::now();
    for _ in 0..1000 {
        let mut lexer = Lexer::new(code_brace, SyntaxMode::Braces);
        let _ = lexer.tokenize();
    }
    let brace_time = start.elapsed();
    
    // Benchmark Indentation Mode
    let start = Instant::now();
    for _ in 0..1000 {
        let mut lexer = Lexer::new(code_indent, SyntaxMode::Indentation);
        let _ = lexer.tokenize();
    }
    let indent_time = start.elapsed();
    
    println!("Benchmark Results (1000 iterations):");
    println!("  Brace Mode:       {:?}", brace_time);
    println!("  Indentation Mode: {:?}", indent_time);
    println!("  Difference:       {:?}", 
             if brace_time > indent_time {
                 brace_time - indent_time
             } else {
                 indent_time - brace_time
             });
}