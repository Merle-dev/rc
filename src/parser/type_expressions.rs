use crate::{
    Token,
    parser::{Parser, TypeTuple, eof},
    pattern,
};

#[derive(Debug)]
pub enum TypeExpr {
    Empty,
    Name(String),
    Array(Box<TypeExpr>),
    SizedArray {
        size: usize,
        item_type: Box<TypeExpr>,
    },
    FnSignature {
        arguments: TypeTuple,
        return_type: Box<TypeExpr>,
    },
}

impl<'a> Parser<'a> {
    pub(super) fn parse_type_expr(self) -> Result<(TypeExpr, Self), String> {
        match self.tokens.first().ok_or_else(eof)? {
            Token::Ident(_) => self.parse_type_ident(),
            Token::Fn => self.parse_function_signiture(),
            // _ => Ok((
            //     TypeExpr::Empty,
            //     Self {
            //         tokens: &self.tokens[1..],
            //         ..self
            //     },
            // )),
            t => panic!("{t:?} not implemented as Type Expression"),
        }
    }
    pub(super) fn parse_type_ident(self) -> Result<(TypeExpr, Self), String> {
        let (tokens, name) = pattern!(self.tokens, Token::Ident(name))?;
        fn parse<'a>(parser: Parser<'a>, name: String) -> Result<(TypeExpr, Parser<'a>), String> {
            if let [Token::LBracket, Token::RBracket, tokens @ ..] = parser.tokens {
                let (type_expr, parser) = parse(Parser { tokens, ..parser }, name)?;
                Ok((TypeExpr::Array(Box::new(type_expr)), parser))
            } else if let [
                Token::LBracket,
                Token::Integer(size),
                Token::RBracket,
                tokens @ ..,
            ] = parser.tokens
            {
                let (type_expr, parser) = parse(Parser { tokens, ..parser }, name)?;
                Ok((
                    TypeExpr::SizedArray {
                        item_type: Box::new(type_expr),
                        size: *size as usize,
                    },
                    parser,
                ))
            } else {
                Ok((TypeExpr::Name(name), parser))
            }
        }

        parse(Self { tokens, ..self }, name.to_owned())
    }

    pub(super) fn parse_function_signiture(self) -> Result<(TypeExpr, Self), String> {
        let (tokens, _) = pattern!(self.tokens, Token::Fn)?;
        let (arguments, parser) =
            Self { tokens, ..self }.parse_typed_delimitered((Token::LParen, Token::RParen))?;

        let (return_type, parser) = if parser.tokens.first().ok_or_else(eof)? == &Token::Arrow {
            Self {
                tokens: &parser.tokens[1..],
                ..parser
            }
            .parse_type_expr()?
        } else {
            (TypeExpr::Empty, parser)
        };

        Ok((
            TypeExpr::FnSignature {
                arguments,
                return_type: Box::new(return_type),
            },
            parser,
        ))
    }
}
