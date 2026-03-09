use crate::{Token, pattern};

struct Parser<'a> {
    tokens: &'a [Token],
}

enum TypeExpr {}

enum Expr {
    Type(TypeExpr),
    Tuple(Box<Vec<Expr>>),
}

fn eof() -> String {
    "Tokenstream ended too early".to_owned()
}

impl<'a> Parser<'a> {
    fn parse(self) -> Result<(Expr, Self), String> {
        unimplemented!()
    }
    fn parse_delimiters(self, delimiters: (Token, Token)) -> Result<(Expr, Self), String> {
        fn parse<'a>(
            accumulant: Vec<Expr>,
            parser: Parser<'a>,
            end: Token,
        ) -> Result<(Vec<Expr>, Parser<'a>), String> {
            let first = parser.tokens.first().ok_or_else(eof)?;
            if *first == end {
                return Ok((accumulant, parser));
            } else if matches!(first, Token::Comma) {
                let (expr, parser) = parser.parse()?;
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
            let (exprs, parser) = parse(Vec::with_capacity(5), Self { tokens }, delimiters.1)?;
            Ok((Expr::Tuple(Box::new(exprs)), parser))
        } else {
            Err(format!("expected {:?} found {first:?}", delimiters.0))
        }
    }
}
