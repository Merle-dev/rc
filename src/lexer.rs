use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{tag, take_until, take_while1},
    character::complete::{char, multispace0},
    combinator::map,
    sequence::{delimited, preceded},
};
use std::cell::RefCell;

use crate::Token;

thread_local! {
    pub static STRING_TABLE: RefCell<Vec<String>> = RefCell::new(Vec::new());
}

fn intern(s: &str) -> usize {
    STRING_TABLE.with(|t| {
        let mut table = t.borrow_mut();
        if let Some(i) = table.iter().position(|x| x == s) {
            i
        } else {
            table.push(s.to_string());
            table.len() - 1
        }
    })
}

// nom 8: ws() must return a concrete type, not `impl Fn` wrapping combinators.
// Easiest fix: just call preceded() inline everywhere instead of a ws() wrapper.

fn is_ident_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

fn ident(input: &str) -> IResult<&str, &str> {
    preceded(multispace0, take_while1(is_ident_char)).parse(input)
}

fn parse_fn_def(input: &str) -> IResult<&str, Vec<Token>> {
    let (input, _) = preceded(multispace0, tag("fn")).parse(input)?;
    let (input, name) = ident(input)?;
    Ok((
        input,
        vec![Token::Fn, Token::FnDefinition(name.to_string())],
    ))
}

fn parse_let(input: &str) -> IResult<&str, Vec<Token>> {
    let (input, _) = preceded(multispace0, tag("let")).parse(input)?;
    let (input, name) = ident(input)?;
    let (input, _) = preceded(multispace0, char('=')).parse(input)?;
    // let (input, value) = parse_string(input)?;
    Ok((
        input,
        vec![
            Token::Let,
            Token::VariableName(name.to_string()),
            Token::Equals,
            // value,
        ],
    ))
}

fn parse_fn_call(input: &str) -> IResult<&str, Token> {
    let (input, name) = ident(input)?;
    // peek: confirm `(` follows without consuming it
    preceded(multispace0, char('(')).parse(input)?;
    Ok((input, Token::FnCall(name.to_string())))
}

fn parse_variable(input: &str) -> IResult<&str, Token> {
    map(ident, |name: &str| Token::VariableName(name.to_string())).parse(input)
}

fn parse_string(input: &str) -> IResult<&str, Token> {
    let (input, s) = preceded(
        multispace0,
        delimited(char('"'), take_until("\""), char('"')),
    )
    .parse(input)?;
    Ok((input, Token::StringIndex(intern(s))))
}

fn parse_punct(input: &str) -> IResult<&str, Token> {
    preceded(
        multispace0,
        alt((
            map(char('('), |_| Token::LParen),
            map(char(')'), |_| Token::RParen),
            map(char('{'), |_| Token::LBrace),
            map(char('}'), |_| Token::RBrace),
            map(char(','), |_| Token::Comma),
            map(char(':'), |_| Token::Colon),
            map(char(';'), |_| Token::SemiColon),
        )),
    )
    .parse(input)
}

fn parse_token(input: &str) -> IResult<&str, Vec<Token>> {
    if let Ok((rest, tokens)) = parse_fn_def(input) {
        return Ok((rest, tokens));
    }
    if let Ok((rest, tokens)) = parse_let(input) {
        return Ok((rest, tokens));
    }
    let (input, tok) = alt((
        parse_string,
        parse_punct,
        parse_fn_call, // before parse_variable — both read an ident
        parse_variable,
    ))
    .parse(input)?;
    Ok((input, vec![tok]))
}

pub fn tokenize(mut input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    loop {
        let (rest, _) = multispace0::<&str, nom::error::Error<&str>>(input).unwrap();
        if rest.is_empty() {
            break;
        }
        match parse_token(rest) {
            Ok((rest, mut batch)) => {
                tokens.append(&mut batch);
                input = rest;
            }
            Err(e) => panic!("Parse error: {e:?}"),
        }
    }
    tokens
}
