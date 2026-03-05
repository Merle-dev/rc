use std::fmt::Debug;

mod lexer;
mod parser;
mod pattern_macro;

use parser::ParserMonad;

use crate::lexer::{STRING_TABLE, tokenize};

#[derive(Clone, Debug, PartialEq)]
enum Token {
    Fn,
    FnCall(String),
    FnDefinition(String),
    LParen,
    RParen,
    LBrace,
    RBrace,
    Let,
    VariableName(String),
    Equals,
    Comma,
    Colon,
    SemiColon,
    StringIndex(usize),
}

fn main() {
    // let a = fn f() { let a = "inside_function"; };
    let text = r#"
        fn main(b: get_type()) {
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
        .parse()
    );

    // let result = ParserMonad {
    //     tokens,
    //     string_interning_store: lexer::STRING_TABLE.take(),
    // }
    // .parse_function_definition();
    // dbg!(result.unwrap());
}
