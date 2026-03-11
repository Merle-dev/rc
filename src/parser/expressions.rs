use crate::{
    Token,
    parser::{Expr, Parser, eof, type_expressions::TypeExpr},
    pattern,
};

impl<'a> Parser<'a> {
    pub(super) fn parse_string(self) -> Result<(Expr, Self), String> {
        let mut string_interning_store = self.string_interning_store;
        let (tokens, id) = pattern!(self.tokens, Token::StringIndex(id))?;
        Ok((
            Expr::String(std::mem::take(&mut string_interning_store[*id])),
            Self {
                tokens,
                string_interning_store,
            },
        ))
    }
    pub(super) fn parse_ident(self) -> Result<(Expr, Self), String> {
        match self.tokens.get(1).ok_or_else(eof)? {
            Token::LParen => self.parse_function_call(),
            _ => unimplemented!(),
        }
    }
    pub(super) fn parse_function_call(self) -> Result<(Expr, Self), String> {
        let (tokens, name) = pattern!(self.tokens, Token::Ident(name))?;
        let (value_tuple, parser) =
            Self { tokens, ..self }.parse_delimitered((Token::LParen, Token::RParen))?;
        Ok((
            Expr::FnCall {
                name: name.to_owned(),
                arguments: Box::new(value_tuple),
            },
            parser,
        ))
    }

    pub(super) fn parse_function_def(self) -> Result<(Expr, Self), String> {
        let (tokens, _) = pattern!(self.tokens, Token::Fn)?;
        dbg!(self.tokens);
        let (tokens, name) = pattern!(tokens, Token::Ident(name))?;
        let (arguments, parser) = Self { tokens, ..self }
            .parse_named_typed_delimitered((Token::LParen, Token::RParen))?;

        let (return_type, parser) = if parser.tokens.first().ok_or_else(eof)? == &Token::Arrow {
            Self {
                tokens: &parser.tokens[1..],
                ..parser
            }
            .parse_type_expr()?
        } else {
            (TypeExpr::Empty, parser)
        };

        let (body, parser) = parser.parse_block()?;

        Ok((
            Expr::FnDef {
                name: name.to_owned(),
                arguments,
                return_type,
                body: Box::new(body),
            },
            parser,
        ))
    }
    pub(super) fn parse_semicolon(self) -> Result<(Expr, Self), String> {
        let (tokens, _) = pattern!(self.tokens, Token::SemiColon)?;
        Ok((Expr::Execute, Self { tokens, ..self }))
    }
    pub(super) fn parse_block(self) -> Result<(Expr, Self), String> {
        let (tokens, _) = pattern!(self.tokens, Token::LBrace)?;
        fn parse<'a>(
            accumulant: Vec<Expr>,
            parser: Parser<'a>,
        ) -> Result<(Vec<Expr>, Parser<'a>), String> {
            if parser
                .tokens
                .first()
                .map(|first| *first == Token::RBrace)
                .ok_or_else(eof)?
            {
                return Ok((accumulant, parser));
            }
            let (expr, parser) = parser.parse()?;
            parse(accumulant.into_iter().chain(Some(expr)).collect(), parser)
        }
        let (expressions, parser) = parse(vec![], Self { tokens, ..self })?;
        Ok((Expr::Block(Box::new(expressions)), parser))
    }
}
