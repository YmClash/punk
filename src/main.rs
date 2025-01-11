#![allow(dead_code)]
#![allow(unused)]
//use pyrust::parser::parser::Parser;

//use ymcrust::lexxer;
use pyrust::lexer::lex::{Lexer, Token};
use pyrust::lexer::lex::SyntaxMode;
use pyrust::parser::parser::Parser;
use pyrust::parser::ast::{ASTNode, Declaration, VariableDeclaration, FunctionDeclaration, ConstDeclaration,Expression,Literal};


fn mode(syntax_mode: SyntaxMode){
    match syntax_mode {
        SyntaxMode::Braces => println!("Braces"),
        SyntaxMode::Indentation => println!("Indentation"),
    }
}




fn main() {

    println!("Pyrust Compiler Test");
    println!("===================\n");
    println!("Mode de syntaxe :\n");



    let code_source = r#"let x = 5; const v = 100;"#;

    let code_binary = "array[0][1]";


    let code_number = "a && b || c";

    let code_decl_braces = "let x = 10;let mut y:int = 3;const numb = 5;pub const x:int = 5;pub struct Point {x: int,y: int};pub struct Point {height: int,width: int};enum Color {x:int,y:float,z:str};pub enum Color {pub x:int,y:float,z:str};pub fn add(x: int, y: int) -> int {return x + y};pub fn add(x: int, y: int) -> int {\
    let mut result = x + y;\
    return result};";
    let code_decl_indentation = "let x = 10\nlet mut y:int = 3\nconst numb = 5\npub const x:int = 5\nstruct Point {x: int,y: int}\npub struct Point {height: int,width: int}\nenum Color {x:int,y:float,z:str}\npub enum Color {pub x:int,y:float,z:str}\n";

    let solo_decl = "let x = 10\nlet mut y:int = 3\nconst numb = 5\npub const x:int = 5\nstruct Point {x: int,y: int}}\n";


    let code_struct = "struct Point {pub x: int,pub y: int};";

    let code_struct_indent = "pub struct Point {x: int,y: int}\nstruct Point {height: int,width: int}";

    //\npub struct Point {height: int,width: int}


    let code_enum_brace = "pub enum Color {pub x:int,y:float, z:str};";
    let code_enum_indent = "enum Color {x:int,y:float,z:str}\n";
    //

    let code_func_braces = "pub fn add(x: int, y: int) -> int {\
    let mut result = x + y;\
    return result}";

    let code_func_indent =
        r#"pub fn add(x: int, y: int) -> int:
        return x + y"#;


    let code_func_indent2 =
        r#"pub fn add(x: int, y: int) -> int:
        let mut result = x + y
        let z = result + 5
        return z"#;
/////////////////////////////
    let code_func_braces2 = r#"match x {1 => print("one"),2 => print("two"),_ => print("other")} let sum:int = add(5, 10);fn add(x: int, y: int) -> int {return x + y} pub fn add() ->int{return 5} obj.method1().field.method2(1+2);"#;
////////////////////////////
    let code_func_braces3 = "pub fn add() ->int{return 5};";


    let code_func_call_braces = "let sum:int = add(5, 10);";
    let code_func_call_indent = "let sum:int = add(5, 10)";

    let code_func_call_braces2 = "print(numb);";
    let code_func_call_indent2 = "print(numb)";

    let code_func_call_methode_braces = "let x = chat.danse(x,y);";
    let code_func_call_methode_indent = "let x = chat.danse(x,y)";

    let code_func_call_methode_braces2 = "chat.danse(x,y);";
    let code_func_call_methode_indent2 = "chat.danse(x,y)";

    let code_func_call_methode_braces3 = "obj.method1().field.method2(1+2);";
    let code_func_call_methode_indent3 = "obj.method1().field.method2(1+2)";

    let code_indice_acces_braces = "let x = tab[5];";
    let code_indice_acces_indent = "let x = tab[5]";

    let code_indice_acces_braces2 = "array[0];";
    let code_indice_acces_indent2 = "array[0]";

    let code_indice_acces_braces3 = "tab[i+3];";
    let code_indice_acces_indent3 = "tab[i+3]";

    let code_indice_acces_braces4 = "vector[calculate_index().index];";
    let code_indice_acces_indent4 = "vector[calculate_index().index]";

    let code_indice_acces_braces5 = "obj.array[i].method();";
    let code_indice_acces_indent5 = "obj.array[i].method()";

    let code_indice_acces_braces6 = "obj.array[i].method().field;";
    let code_indice_acces_indent6 = "obj.array[i].method().field";

    let code_indice_acces_braces7 = "array[i][j];";
    let code_indice_acces_indent7 = "array[i][j]";

    let code_indice_acces_braces8 = "vector[obj.get_index()];";
    let code_indice_acces_indent8 = "vector[obj.get_index()]";

    let code_indice_acces_braces9 = "matrix[i][j] = array[get_index()] + offset;";
    let code_indice_acces_indent9 = "matrix[i][j] = array[get_index()] + offset";

    let code_indice_acces_braces10 = "obj.data[start + offset].process()[index];";
    let code_indice_acces_indent10 = "obj.data[start + offset].process()[index]";

    let code_indice_acces_braces11 = "obj.method1().method2()[index];";
    let code_indice_acces_indent11 = "obj.method1().method2()[index]";

    let code_assign_multi_braces = "a = b = c = 0;";
    let code_assign_multi_indent = "a = b = c = 0";

    let code_assign_compound_braces = "a += 5;";
    let code_assign_compound_indent = "counter += offset * 5";


    let code_assign_desctructuring_braces = "[x,y,z] = point3d;";
    let code_assign_desctructuring_indent = "[x, y, z] = point3d";

    let code_lambda_braces = "let add = lambda (x: int, y: int) -> int {x + y};";
    let code_lambda_indent = "add = lambda (x: int, y: int) -> int: x + y";

    // let code_test = r#"if x > 0 { print(x);} elif x > 0 {hallo.chante;}elif x==0 {momo.position(x,y);}else{print(hallo.danse);}"#;
    let code_test = r#"if x > 0 { print(x);}if x < 0 {print()}else{print("0");}"#;


    let code_test2 = r#"if x > 0 { print(IF); } elif { print(ELIF ou ELSE IF };} else{print(ELSE) ;}"#;
    let code_test3 = r#"while x > 0 { print(x);}"#;
    let code_test4 = r#"for i in range(10) { print(i);}"#;

    let code_test0 = r#"match x {1 => print("one"),2 => print("two"),_ => print("other")}"#;
    let code_test1 = r#"match x {1 => print(1),2 => print(2),_ => print("other")}"#;
    let code_test5 = r#"match x {n if n > 0 => print("positive"),n if n<0 =>{print("negative");print(n);},_ => print("zero")}"#;


    let code_test6 = r#"match x:
    1 => print("One")
    2 => print("Two")
    _ => print("Other")
"#;

    let code_test7 = r#"match x :
    n if n > 0:
        print("positive")
    n if n < 0:
        print("negative")
        print(n)
    _:
        print("zero")
"#;

    let code_test8 = r#"match x :
    n if n > 0 =>print("positive")
    n if n < 0 =>print("negative")
    _ =>print("zero")
"#;

    let code_test9 = r#"match x :
    n if n > 0 =>print("positive")
    n if n < 0:
        print("negative")
        print(n)
    _:
        print("zero")
"#;

    let code_test10 = r#"match x :
    (0, 0) => print("Origin")
    (x, 0):
        print("X-axis")
        print(x)
    (0, y) if y > 0 => print("Positive Y-axis")
    (x, y) => print("MOMO")
    _ => print("Other")
"#;
    let code_test11 = r#"match x :
    [0, 0] => print("Origin")
    [x, 0]:
        print("X-axis")
        print(x)
    [0, y] if y > 0 => print("Positive Y-axis")
    _ => print("Other")
"#;

    let code_test12 = r#"match x :1..5 => println!("entre 1 et 4"),10.. => println!("10 ou plus"),..10 => println!("moins de 10")"#;

    let code_test13 = r#"match x :
    n if n > 0 => print("positive")
    (x, y) => print("tuple simple")
    [1, 2] => print("array simple")
    _ => print("default")
"#;

    let code_test14 = r#"match x {n if n > 0 => print("positive"),(x, y) => print("tuple simple"),[1, 2] => print("array simple"),_ => print("default")}"#;


    let code_test15 = r#"if x > 0 {print("hello");}else{print("Nothing");}"#;
    let code_test16 = r#"if x > 0 :
    print("hello")
elif x < 0:
    print("negative")
else:
    print("Nothing")
"#;

    let code_test17 = r#"counter:loop:
    print("infini")
    x += 1
    if x > 10:
        break
"#;
    let code_test18 = r#"counter: loop {print("infini");x += 1;if x > 10 {break;}}"#;

    let code_test19 = r#"1..5"#;
    let code_test20 = r#"use std.io::{Read as R, Write as W};"#;

    let code_test21 = r#"pub class MyClass:
    let x: int
    let y: str
    fn do_something() -> int:
        return self.x + 1 "#;


    let code_test22 = r#"pub class Myclass(classe){let x:int;pubfn do_something() ->int{return self.x + 1}}"#;

    let code_test23 = r#"fn add(x:int)->int{return x+1}"#;

    let code_test24 = r#"pub class Myclass(parent){def init(x: int, y: int) {self.x = x,self.y = y }fn do_something() -> int {return self.x + 1}}"#;
    let code_test25 = r#"pub class Myclass(parent):
    def init(x: int, y: int):
        self.x = x
        self.y = y
    fn do_something() -> int:
        return self.x + 1"#;

    let code_test26 = r#"pub trait Drawable  {fn do_something(x: T) -> int;fn area(a:float)->float;fn do_something_else(x: char) -> int;type Color;}"#;
    let code_test27 = r#"pub trait Drawable:
    fn do_something(x: int) -> int
    fn do_something_else(x: int) -> int
    fn area() -> float
    type Color"#;
    let code_test28 = r#"trait Drawable where T: Copy + Display {fn draw(x: T);fn get_color() -> T;type AssociatedType where Self: Clone;}"#;

    let code_test29 = r#"trait Drawable where T:Copy + Display :
    fn draw(x: T)
    fn get_color() -> T
    type Color"#;

    let code_test30 = r#"where T: Copy"#;

    let code_test31 = r#"impl<T> Drawable for MyType<T>{
    fn draw(x:int) {
        return self.x+1}
    fn get_color() -> int {
        return color.code()
        }
    }"#;



    let code_test32 = r#"impl Color {def init(value: T) -> Self {MyType { value }}fn consume(self)-> T {&self.value} fn get_value(&self) -> &T {&self.value}fn set_value(&mut self, value: T) {self.value = value }}"#;

    let code_test33 = r#"mpl<T> Drawable for MyType<S>:
    fn draw(x: float):
        return self.x+1}
    fn get_color() -> int:
        return color.code()"#;

    let code_test34 = r#"impl Color:
    def init(value: T) -> Self:
        MyType { value }

    fn consume(self) -> T:
        self.value

    fn get_value(&self) -> &T:
        &self.value

    fn set_value(&mut self, value: T):
        self.value = value"#;



    let code_test35 = r#"impl<T> Drawable for MyType<T> where D: Display:
    fn draw(x: T) -> bool:
        return self.x + 1"#;


    let code_test36 = r#"let x = 10
let mut y = 10
let z:int = 1.5
fn get_color(x:int) -> int:
    return self.x+1"#;


    let code_test37 = r#"let mut c = &mut 10;"#;

    let code_test38 = r#"let x = 10 ;
    match x {
        n if n > 0 => print("positive"),
        n if n < 0 => {
            print("negative");
            print(n);
        },
        _ => print("zero")}"#;

    let code_test39 = r#"let x = 10
match x :
    n if n > 0 => print("positive")
    (x, y) => print("tuple simple")
    [1, 2] => print("array simple")
    _ => print("default")
"#;

    let code_test40 = r#"try {
            risky_function();
        } except Error {
            handle_error();
        } finally {
            cleanup();
        }"#;




    // let mut lexer = Lexer::new(code_test39, SyntaxMode::Indentation);
    let mut lexer = Lexer::new(code_test, SyntaxMode::Braces);
    let tokens = lexer.tokenize();

    // Affichage des tokens pour vérification
    for (i, tok) in tokens.iter().enumerate() {
        println!("{}:{:?}", i, tok);
    }
    println!("\n");

    // let mut parser = Parser::new(tokens, SyntaxMode::Indentation);
    let mut parser = Parser::new(tokens, SyntaxMode::Braces);

    // while !parser.is_at_end() {
    //     match parser.parse_statement() {
    //         Ok(ast) => {
    //             println!("AST OK!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!");
    //             println!("AST généré pour la déclaration,l'expression ou le statement  :");
    //             println!("{:#?}", ast);
    //         }
    //         Err(e) => {
    //             println!("Erreur lors du parsing : {}", e);
    //             break;
    //         }
    //     }
    // }


    //parser  le  programme
    while !parser.is_at_end() {
        match parser.parse_program() {
            Ok(ast) => {
                println!("AST OK!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!");
                println!("AST généré pour la déclaration,l'expression ou le statement  :");
                println!("{:#?}", ast);
            }
            Err(e) => {
                println!("Erreur lors du parsing : {}", e);
                break;
            }
        }
    }




    println!("Parsing terminé\n");
    // println!("Sinon, Parsing des Statement \n");



    println!("\n");
    println!("=========OK==========\n");
    println!("Pyrust Compiler By YmC");
    println!("===================\n");
    println!("\n");



}
