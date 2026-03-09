use std::fmt::Debug;

mod lexer;
mod parser;
mod parsing;
mod pattern_macro;

use parser::ParserMonad;

use crate::lexer::{STRING_TABLE, tokenize};

#[derive(Clone, Debug, PartialEq)]
enum Token {
    Fn,
    StringIndex(usize),
    Ident(String),
    Let,
    EnumDefinition,
    StructDefinition,

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
        struct Parser {
            tokens: Tokens,
            sit: String,
        }

        fn main() {
            let a = E::A;
            println("Hello");
        }
    "#;
    let tokens = tokenize(text);
    dbg!(&tokens);
    dbg!(
        ParserMonad {
            tokens: &tokens[..],
            string_interning_store: STRING_TABLE.take()
        }
        .parse_file()
        .unwrap()
        .0
    );

    // let result = ParserMonad {
    //     tokens,
    //     string_interning_store: lexer::STRING_TABLE.take(),
    // }
    // .parse_function_definition();
    // dbg!(result.unwrap());
}
