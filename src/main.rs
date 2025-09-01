//src/main.rs
#![allow(dead_code)]
#![allow(unused)]

use punk::lexer::lex::{Lexer, Token};
use punk::lexer::lex::SyntaxMode;
use punk::parser::parser::Parser;
use punk::parser::ast::{ASTNode, Declaration, VariableDeclaration, FunctionDeclaration, ConstDeclaration,Expression,Literal};
use punk::semantic::analyser::SemanticAnalyzer;

fn mode(syntax_mode: SyntaxMode){
    match syntax_mode {
        SyntaxMode::Braces => println!("Mode Braces"),
        SyntaxMode::Indentation => println!("Mode Indentation"),
    }
}

fn print_separator(title: &str) {
    println!("\n{:-^60}", format!(" {} ", title));
}

// Nouvelle fonction pour tester les deux modes avec le même code
fn test_dual_mode_parsing() {
    print_separator("TEST DUAL-MODE PARSING");
    
    // Définir le code pour les deux modes
    let code_samples = vec![
        // Test 1: Variable Declaration
        ("Variable Declaration", 
         "let x: int = 5;",
         "let x: int = 5"),
        
        // Test 2: Function Declaration
        ("Function Declaration",
         r#"fn add(a: int, b: int) -> int {
    return a + b;
}"#,
         r#"fn add(a: int, b: int) -> int:
    return a + b"#),
        
        // Test 3: If-Else Statement
        ("If-Else Statement",
         r#"if x > 0 {
    print("positive");
} else {
    print("negative");
}"#,
         r#"if x > 0:
    print("positive")
else:
    print("negative")"#),
        
        // Test 4: Match Expression
        ("Match Expression",
         r#"match x {
    1 => print("one"),
    2 => print("two"),
    _ => print("other")
}"#,
         r#"match x:
    1 => print("one")
    2 => print("two")
    _ => print("other")"#),
    ];
    
    for (name, brace_code, indent_code) in code_samples {
        println!("\n🔍 Testing: {}", name);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        
        // Test Brace Mode
        println!("▶ Brace Mode:");
        test_single_mode(brace_code, SyntaxMode::Braces);
        
        // Test Indentation Mode
        println!("\n▶ Indentation Mode:");
        test_single_mode(indent_code, SyntaxMode::Indentation);
        
        println!();
    }
}

fn test_single_mode(code: &str, mode: SyntaxMode) {
    // Tokenization
    let mut lexer = Lexer::new(code, mode);
    let tokens = lexer.tokenize();
    println!("  ✅ Tokenization successful: {} tokens", tokens.len());
    
    // Parsing
    let mut parser = Parser::new(tokens, mode);
    match parser.parse_program() {
        Ok(ast) => {
            println!("  ✅ Parsing successful");
            // Afficher un résumé de l'AST
            match ast {
                ASTNode::Program(nodes) => {
                    println!("  📊 AST contains {} top-level nodes", nodes.len());
                }
                _ => {
                    println!("  📊 Single AST node generated");
                }
            }
        }
        Err(e) => {
            println!("  ❌ Parsing failed: {:?}", e);
        }
    }
}

// Nouvelle fonction pour tester des cas complexes
fn test_complex_program() {
    print_separator("TEST COMPLEX PROGRAM");
    
    let complex_brace = r#"
// Fibonacci en mode Braces
fn fibonacci(n: int) -> int {
    if n <= 1 {
        return n;
    }
    
    let mut a = 0;
    let mut b = 1;
    let mut i = 2;
    
    while i <= n {
        let temp = a + b;
        a = b;
        b = temp;
        i = i + 1;
    }
    
    return b;
}

let result = fibonacci(10);"#;

    let complex_indent = r#"
# Fibonacci en mode Indentation
fn fibonacci(n: int) -> int:
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

    println!("\n🔬 Testing Complex Program: Fibonacci");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    println!("\n▶ Brace Mode:");
    test_with_semantic_analysis(complex_brace, SyntaxMode::Braces);
    
    println!("\n▶ Indentation Mode:");
    test_with_semantic_analysis(complex_indent, SyntaxMode::Indentation);
}

fn test_with_semantic_analysis(code: &str, mode: SyntaxMode) {
    // Tokenization
    let mut lexer = Lexer::new(code, mode);
    let tokens = lexer.tokenize();
    println!("  ✅ Tokenization: {} tokens", tokens.len());
    
    // Parsing
    let mut parser = Parser::new(tokens, mode);
    match parser.parse_program() {
        Ok(ast) => {
            println!("  ✅ Parsing successful");
            
            // Semantic Analysis
            let mut analyzer = SemanticAnalyzer::new();
            let ast_nodes = match ast {
                ASTNode::Program(nodes) => nodes,
                single => vec![single],
            };
            
            match analyzer.analyze(&ast_nodes) {
                Ok(()) => {
                    println!("  ✅ Semantic analysis passed");
                    let stats = analyzer.get_analysis_stats();
                    println!("  📊 Stats: {} symbols, {} types, {} scopes",
                            stats.total_symbols, stats.total_types, stats.total_scopes);
                }
                Err(errors) => {
                    println!("  ⚠️ Semantic analysis: {} errors", errors.len());
                    for (i, e) in errors.iter().take(3).enumerate() {
                        println!("     Error {}: {}", i+1, e.message);
                    }
                }
            }
        }
        Err(e) => {
            println!("  ❌ Parsing failed: {:?}", e);
        }
    }
}

// Fonction pour tester la robustesse avec des erreurs
fn test_error_recovery() {
    print_separator("TEST ERROR RECOVERY");
    
    let error_cases = vec![
        ("Missing Value", 
         "let x = ;",
         "let x ="),
        
        ("Unclosed Block",
         "fn test() { let x = 5",
         "fn test():\n    let x = 5"),
        
        ("Invalid Type",
         "let x: unknowntype = 5;",
         "let x: unknowntype = 5"),
    ];
    
    for (name, brace_code, indent_code) in error_cases {
        println!("\n⚠️ Testing Error Case: {}", name);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        
        println!("▶ Brace Mode:");
        test_error_handling(brace_code, SyntaxMode::Braces);
        
        println!("\n▶ Indentation Mode:");
        test_error_handling(indent_code, SyntaxMode::Indentation);
    }
}

fn test_error_handling(code: &str, mode: SyntaxMode) {
    let mut lexer = Lexer::new(code, mode);
    let tokens = lexer.tokenize();
    
    let mut parser = Parser::new(tokens, mode);
    match parser.parse_program() {
        Ok(_) => {
            println!("  ⚠️ Unexpectedly succeeded (should have failed)");
        }
        Err(e) => {
            println!("  ✅ Error correctly detected: {:?}", e.error);
        }
    }
}

// Nouvelle fonction main avec menu de tests
fn main() {
    println!("╔════════════════════════════════════════╗");
    println!("║     PunkLang Compiler Test Suite       ║");
    println!("║           Dual-Mode Parser             ║");
    println!("╚════════════════════════════════════════╝");
    
    // Obtenir l'argument de ligne de commande pour choisir le test
    let args: Vec<String> = std::env::args().collect();
    let test_mode = if args.len() > 1 { &args[1] } else { "all" };
    
    match &test_mode[..] {
        "dual" => {
            println!("\n🎯 Running Dual-Mode Tests Only");
            test_dual_mode_parsing();
        }
        "complex" => {
            println!("\n🎯 Running Complex Program Tests");
            test_complex_program();
        }
        "error" => {
            println!("\n🎯 Running Error Recovery Tests");
            test_error_recovery();
        }
        "original" => {
            println!("\n🎯 Running Original Test");
            run_original_test();
        }
        "all" | _ => {
            println!("\n🎯 Running All Tests");
            test_dual_mode_parsing();
            test_complex_program();
            test_error_recovery();
        }
    }
    
    print_separator("TEST SUITE COMPLETED");
    println!("PunkLang Compiler by YmC");
    println!();
}

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