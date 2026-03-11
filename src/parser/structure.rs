use crate::{
    Token,
    parser::{Expr, NamedTypeTuple, Parser, TypeExpr, TypeTuple, ValueTuple, eof},
    pattern,
};

impl<'a> Parser<'a> {
    pub(super) fn parse_delimitered(
        self,
        delimiters: (Token, Token),
    ) -> Result<(ValueTuple, Self), String> {
        fn parse<'a>(
            accumulant: Vec<Expr>,
            parser: Parser<'a>,
            end: Token,
        ) -> Result<(Vec<Expr>, Parser<'a>), String> {
            let (expr, parser) = parser.parse()?;
            let first = parser.tokens.first().ok_or_else(eof)?;
            if *first == end {
                return Ok((
                    accumulant.into_iter().chain(Some(expr)).collect(),
                    Parser {
                        tokens: &parser.tokens[1..],
                        ..parser
                    },
                ));
            } else if *first == Token::Comma {
                let (exprs, parser) = parse(
                    accumulant.into_iter().chain(Some(expr)).collect(),
                    Parser {
                        tokens: &parser.tokens[1..],
                        ..parser
                    },
                    end,
                )?;
                Ok((exprs, parser))
            } else {
                Err(format!("expected {end:?} or , found {first:?}"))
            }
        }
        let (first, tokens) = self.tokens.split_first().ok_or_else(eof)?;
        if *first == delimiters.0 {
            if tokens.first().ok_or_else(eof)? == &Token::RParen {
                Ok((
                    ValueTuple(vec![]),
                    Self {
                        tokens: &tokens[1..],
                        ..self
                    },
                ))
            } else {
                let (exprs, parser) =
                    parse(Vec::with_capacity(5), Self { tokens, ..self }, delimiters.1)?;
                Ok((ValueTuple(exprs), parser))
            }
        } else {
            Err(format!("expected {:?} found {first:?}", delimiters.0))
        }
    }
    pub(super) fn parse_named_typed_delimitered(
        self,
        delimiters: (Token, Token),
    ) -> Result<(NamedTypeTuple, Self), String> {
        fn parse<'a>(
            accumulant: Vec<(String, TypeExpr)>,
            parser: Parser<'a>,
            end: Token,
        ) -> Result<(Vec<(String, TypeExpr)>, Parser<'a>), String> {
            let (tokens, name) = pattern!(parser.tokens, Token::Ident(name))?;
            let (tokens, _) = pattern!(tokens, Token::Colon)?;
            let (type_expr, parser) = Parser { tokens, ..parser }.parse_type_expr()?;

            let first = parser.tokens.first().ok_or_else(eof)?;
            if *first == end {
                return Ok((
                    accumulant
                        .into_iter()
                        .chain(Some((name.to_owned(), type_expr)))
                        .collect(),
                    Parser {
                        tokens: &parser.tokens[1..],
                        ..parser
                    },
                ));
            } else if *first == Token::Comma {
                let (exprs, parser) = parse(
                    accumulant
                        .into_iter()
                        .chain(Some((name.to_owned(), type_expr)))
                        .collect(),
                    Parser {
                        tokens: &parser.tokens[1..],
                        ..parser
                    },
                    end,
                )?;
                Ok((exprs, parser))
            } else {
                dbg!(tokens);
                Err(format!("expected {end:?} or , found {first:?}"))
            }
        }
        let (first, tokens) = self.tokens.split_first().ok_or_else(eof)?;
        if *first == delimiters.0 {
            if tokens.first().ok_or_else(eof)? == &Token::RParen {
                Ok((
                    NamedTypeTuple(vec![]),
                    Self {
                        tokens: &tokens[1..],
                        ..self
                    },
                ))
            } else {
                let (named_types, parser) =
                    parse(Vec::with_capacity(5), Self { tokens, ..self }, delimiters.1)?;
                Ok((NamedTypeTuple(named_types), parser))
            }
        } else {
            Err(format!("expected {:?} found {first:?}", delimiters.0))
        }
    }

    pub(super) fn parse_typed_delimitered(
        self,
        delimiters: (Token, Token),
    ) -> Result<(TypeTuple, Self), String> {
        fn parse<'a>(
            accumulant: Vec<TypeExpr>,
            parser: Parser<'a>,
            end: Token,
        ) -> Result<(Vec<TypeExpr>, Parser<'a>), String> {
            let (type_expr, parser) = parser.parse_type_expr()?;

            let first = parser.tokens.first().ok_or_else(eof)?;
            if *first == end {
                return Ok((
                    accumulant.into_iter().chain(Some(type_expr)).collect(),
                    Parser {
                        tokens: &parser.tokens[1..],
                        ..parser
                    },
                ));
            } else if *first == Token::Comma {
                let (exprs, parser) = parse(
                    accumulant.into_iter().chain(Some(type_expr)).collect(),
                    Parser {
                        tokens: &parser.tokens[1..],
                        ..parser
                    },
                    end,
                )?;
                Ok((exprs, parser))
            } else {
                Err(format!("expected {end:?} or , found {first:?}"))
            }
        }
        let (first, tokens) = self.tokens.split_first().ok_or_else(eof)?;
        if *first == delimiters.0 {
            if tokens.first().ok_or_else(eof)? == &Token::RParen {
                Ok((
                    TypeTuple(vec![]),
                    Self {
                        tokens: &tokens[1..],
                        ..self
                    },
                ))
            } else {
                let (types, parser) =
                    parse(Vec::with_capacity(5), Self { tokens, ..self }, delimiters.1)?;
                Ok((TypeTuple(types), parser))
            }
        } else {
            Err(format!("expected {:?} found {first:?}", delimiters.0))
        }
    }
}
