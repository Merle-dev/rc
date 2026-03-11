use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{tag, take_until, take_while1},
    character::complete::{char, digit1, multispace0},
    combinator::{map, recognize},
    number::complete::double,
    sequence::{delimited, preceded, tuple},
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

fn parse_float(input: &str) -> IResult<&str, Token> {
    let (input, s) = preceded(
        multispace0,
        recognize((
            take_while1(|c: char| c.is_ascii_digit()),
            char('.'),
            take_while1(|c: char| c.is_ascii_digit()),
        )),
    )
    .parse(input)?;

    let n = s.parse::<f64>().map_err(|_| {
        nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Float))
    })?;

    Ok((input, Token::Float(n)))
}

fn parse_integer(input: &str) -> IResult<&str, Token> {
    let (input, s) =
        preceded(multispace0, take_while1(|c: char| c.is_ascii_digit())).parse(input)?;

    let n = s.parse::<i128>().map_err(|_| {
        nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Digit))
    })?;

    Ok((input, Token::Integer(n)))
}

fn parse_fn_def(input: &str) -> IResult<&str, Vec<Token>> {
    let (input, _) = preceded(multispace0, tag("fn")).parse(input)?;
    let (input, name) = ident(input)?;
    Ok((input, vec![Token::Fn, Token::Ident(name.to_string())]))
}

fn parse_fn(input: &str) -> IResult<&str, Vec<Token>> {
    let (input, _) = preceded(multispace0, tag("fn")).parse(input)?;
    Ok((input, vec![Token::Fn]))
}

fn parse_enum_def(input: &str) -> IResult<&str, Vec<Token>> {
    let (input, _) = preceded(multispace0, tag("enum")).parse(input)?;
    let (input, name) = ident(input)?;
    Ok((
        input,
        vec![Token::EnumDefinition, Token::Ident(name.to_owned())],
    ))
}

fn parse_struct_def(input: &str) -> IResult<&str, Vec<Token>> {
    let (input, _) = preceded(multispace0, tag("struct")).parse(input)?;
    let (input, name) = ident(input)?;
    Ok((
        input,
        vec![Token::StructDefinition, Token::Ident(name.to_owned())],
    ))
}

fn parse_let(input: &str) -> IResult<&str, Vec<Token>> {
    let (input, _) = preceded(multispace0, tag("let")).parse(input)?;
    let (input, name) = ident(input)?;
    let (input, _) = preceded(multispace0, char('=')).parse(input)?;
    Ok((
        input,
        vec![Token::Let, Token::Ident(name.to_string()), Token::Equals],
    ))
}

fn parse_return(input: &str) -> IResult<&str, Token> {
    let (input, _) = preceded(multispace0, tag("return")).parse(input)?;
    Ok((input, Token::Return))
}

fn parse_fn_call(input: &str) -> IResult<&str, Token> {
    let (input, name) = ident(input)?;
    // peek: confirm `(` follows without consuming it
    preceded(multispace0, char('(')).parse(input)?;
    Ok((input, Token::Ident(name.to_string())))
}

fn parse_variable(input: &str) -> IResult<&str, Token> {
    map(ident, |name: &str| Token::Ident(name.to_string())).parse(input)
}

fn parse_string(input: &str) -> IResult<&str, Token> {
    let (input, s) = preceded(
        multispace0,
        delimited(char('"'), take_until("\""), char('"')),
    )
    .parse(input)?;
    Ok((input, Token::StringIndex(intern(s))))
}

fn parse_delimiters(input: &str) -> IResult<&str, Token> {
    preceded(
        multispace0,
        alt((
            map(char('('), |_| Token::LParen),
            map(char(')'), |_| Token::RParen),
            map(char('{'), |_| Token::LBrace),
            map(char('}'), |_| Token::RBrace),
            map(char('['), |_| Token::LBracket),
            map(char(']'), |_| Token::RBracket),
        )),
    )
    .parse(input)
}
fn parse_operators(input: &str) -> IResult<&str, Token> {
    preceded(
        multispace0,
        alt((
            map(char('!'), |_| Token::Not),
            map(tag("&&"), |_| Token::AndCmp),
            map(tag("||"), |_| Token::OrCmp),
            map(char('+'), |_| Token::Plus),
            map(char('-'), |_| Token::Minus),
            map(char('*'), |_| Token::Multiply),
            map(char('/'), |_| Token::Divide),
            map(char('%'), |_| Token::Modulo),
            map(char('<'), |_| Token::Less),
            map(char('>'), |_| Token::Greater),
            map(char('?'), |_| Token::QuestionMark),
        )),
    )
    .parse(input)
}

fn parse_punctuation(input: &str) -> IResult<&str, Token> {
    preceded(
        multispace0,
        alt((
            map(char(','), |_| Token::Comma),
            map(char(':'), |_| Token::Colon),
            map(char(';'), |_| Token::SemiColon),
            map(char('\''), |_| Token::Apostrophe),
            map(tag("::"), |_| Token::Namespace),
            map(tag("->"), |_| Token::Arrow),
            map(tag("=>"), |_| Token::FatArrow),
        )),
    )
    .parse(input)
}

fn parse_token(input: &str) -> IResult<&str, Vec<Token>> {
    let double_arg_functions = [
        parse_fn_def,
        parse_fn,
        parse_enum_def,
        parse_struct_def,
        parse_let,
    ];
    for f in double_arg_functions.into_iter() {
        if let Ok((rest, tokens)) = f(input) {
            return Ok((rest, tokens));
        }
    }

    let (input, tok) = alt((
        parse_float,
        parse_integer,
        parse_string,
        parse_punctuation,
        parse_delimiters,
        parse_operators,
        parse_fn_call,
        parse_variable,
        parse_return,
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
