// // src/repl/mod.rs
//
// use crate::lexer::lex;
// use crate::parser::parser;
//
//
// use rustyline::Editor;
// use rustyline::error::ReadlineError;
// use colored::*;
// use crate::interpreter::error::RuntimeError;
// use crate::interpreter::evaluator::Evaluator;
// use crate::Lexer;
// use crate::parser::parser::Parser;
//
// pub struct REPL {
//     evaluator: Evaluator,
//     history_file: String,
// }
//
// impl REPL {
//     pub fn new() -> Self {
//         REPL {
//             evaluator: Evaluator::new(),
//             history_file: ".punklang_history".to_string(),
//         }
//     }
//
//     // pub fn
//
//     pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
//         println!("{}", "PunkLang REPL v0.1.0".bright_cyan().bold());
//         println!("{}", "Type 'help' for commands, 'exit' to quit".bright_black());
//         println!();
//
//         let mut rl = Editor::<()>::new()?;
//
//         // Charger l'historique
//         let _ = rl.load_history(&self.history_file);
//
//         loop {
//             let prompt = "punk> ".bright_green().to_string();
//
//             match rl.readline(&prompt) {
//                 Ok(line) => {
//                     // Ajouter à l'historique
//                     rl.add_history_entry(&line);
//
//                     // Commandes spéciales
//                     match line.trim() {
//                         "exit" | "quit" |"bye" => {
//                             println!("{}", "Goodbye!".bright_yellow());
//                             break;
//                         },
//                         "help" => {
//                             self.print_help();
//                             continue;
//                         },
//                         "clear" => {
//                             print!("\x1B[2J\x1B[1;1H");
//                             continue;
//                         },
//                         "env" => {
//                             self.print_environment();
//                             continue;
//                         },
//                         "" => continue,
//                         _ => {}
//                     }
//
//                     // Évaluer le code
//                     match self.eval_line(&line) {
//                         Ok(value) => {
//                             println!("{} {}", "=>".bright_blue(), value);
//                         },
//                         Err(e) => {
//                             eprintln!("{} {}", "Error:".bright_red().bold(), e);
//                         }
//                     }
//                 },
//
//                 Err(ReadlineError::Interrupted) => {
//                     println!("{}", "Use 'exit' to quit".bright_yellow());
//                 },
//
//                 Err(ReadlineError::Eof) => {
//                     println!("{}", "\nGoodbye!".bright_yellow());
//                     break;
//                 },
//
//                 Err(err) => {
//                     eprintln!("Error: {:?}", err);
//                     break;
//                 }
//             }
//         }
//
//         // Sauvegarder l'historique
//         rl.save_history(&self.history_file)?;
//
//         Ok(())
//     }
//
//     fn eval_line(&mut self, line: &str) -> Result<String, RuntimeError> {
//         // Mode multi-ligne pour les fonctions
//         let code = if line.ends_with(':') || line.contains("fn ") || line.contains("def ") {
//             self.read_multiline(line)?
//         } else {
//             line.to_string()
//         };
//
//         // Lexer
//         let mut lexer = Lexer::new(&code,self.evaluator.syntax_mode);
//         let tokens = lexer.tokenize();
//
//         // Parser
//         let mut parser = Parser::new(tokens,self.evaluator.syntax_mode);
//         let ast = parser.parse_program()
//             .map_err(|e| RuntimeError::TypeError(format!("Parse error: {:?}", e)))?;
//
//         // Évaluer
//         let result = self.evaluator.eval(&ast)?;
//
//         // Formater le résultat
//         Ok(format!("{}", result))
//     }
//
//     // fn read_multiline(&self, first_line: &str) -> Result<String, RuntimeError> {
//     //     let mut lines = vec![first_line.to_string()];
//     //     let mut rl = Editor::<(), I>::new().unwrap();
//     //
//     //     loop {
//     //         let prompt = "... ".bright_green().to_string();
//     //
//     //         match rl.readline(&prompt) {
//     //             Ok(line) => {
//     //                 if line.trim().is_empty() {
//     //                     break;
//     //                 }
//     //                 lines.push(line);
//     //             },
//     //             Err(_) => break,
//     //         }
//     //     }
//     //
//     //     Ok(lines.join("\n"))
//     // }
//
//     fn print_help(&self) {
//         println!("{}", "PunkLang REPL Commands:".bright_cyan().bold());
//         println!("  {}  - Exit the REPL", "exit/quit".bright_yellow());
//         println!("  {}      - Show this help message", "help".bright_yellow());
//         println!("  {}     - Clear the screen", "clear".bright_yellow());
//         println!("  {}       - Show environment variables", "env".bright_yellow());
//         println!();
//         println!("{}", "Examples:".bright_cyan().bold());
//         println!("  let x = 42");
//         println!("  fn add(a, b) {{ return a + b }}");
//         println!("  print(\"Hello, World!\")");
//         println!("  [1, 2, 3].map(lambda x => x * 2)");
//     }
//
//     fn print_environment(&self) {
//         println!("{}", "Environment:".bright_cyan().bold());
//         // a faire : Imprimer les variables de l'environnement
//         println!("  {} variables defined",
//                  self.evaluator.env.borrow().vars.len());
//     }
// }