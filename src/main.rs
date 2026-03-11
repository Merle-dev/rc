use std::fmt::Debug;

mod lexer;
mod parser;
mod pattern_macro;

// use parser::ParserMonad;

use crate::{
    lexer::{STRING_TABLE, tokenize},
    parser::Parser,
};

#[derive(Clone, Debug, PartialEq)]
enum Token {
    Fn,
    StringIndex(usize),
    Integer(i128),
    // Float { whole: i128, decimal: u128 },
    Float(f64),
    Ident(String),
    Let,
    EnumDefinition,
    StructDefinition,
    Return,

    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,

    Equals,
    Comma,
    Colon,
    SemiColon,
    FatArrow,
    Arrow,
    Apostrophe,
    Namespace,

    Not,
    Plus,
    Minus,
    Multiply,
    Divide,
    Modulo,
    Less,
    Greater,
    AndCmp,
    OrCmp,
    QuestionMark,
}

fn main() {
    // let a = fn f() { let a = "inside_function"; };
    let text = r#"
        fn main(args: String[]) -> u8 {
            println("Hello");
            fn adder(base: i32) -> fn(i32) -> i32 {
                fn a (a: i32) -> i32 {
                    add("base", "a")
                }
            }
        }
    "#;
    let tokens = tokenize(text);
    dbg!(&tokens);
    // dbg!(
    //     ParserMonad {
    //         tokens: &tokens[..],
    //         string_interning_store: STRING_TABLE.take()
    //     }
    //     .parse_file()
    //     .unwrap()
    //     .0
    // );

    let parser = Parser {
        tokens: &tokens[..],
        string_interning_store: STRING_TABLE.take(),
    };
    match parser.parse() {
        Ok(ast) => println!("{:#?}", ast.0),
        Err(error_msg) => println!("\n{error_msg}\n"),
    }
}
