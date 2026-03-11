use crate::{Token, parser::type_expressions::TypeExpr, pattern};

// mod expressions;
mod expressions;
mod structure;
mod type_expressions;

#[derive(Debug)]
pub struct Parser<'a> {
    pub tokens: &'a [Token],
    pub string_interning_store: Vec<String>,
}

#[derive(Debug)]
pub struct NamedTypeTuple(Vec<(String, TypeExpr)>);
#[derive(Debug)]
pub struct TypeTuple(Vec<TypeExpr>);
#[derive(Debug)]
pub struct ValueTuple(Vec<Expr>);

#[derive(Debug)]
pub enum Expr {
    Block(Box<Vec<Expr>>),
    FnDef {
        name: String,
        arguments: NamedTypeTuple,
        return_type: TypeExpr,
        body: Box<Expr>,
    },
    FnCall {
        name: String,
        arguments: Box<ValueTuple>,
    },
    String(String),
    Execute,
}

pub(super) fn eof() -> String {
    "Tokenstream ended too early".to_owned()
}

impl<'a> Parser<'a> {
    pub fn parse(self) -> Result<(Expr, Self), String> {
        match self.tokens.first().ok_or_else(eof)? {
            Token::Fn => self.parse_function_def(),
            Token::LBrace => self.parse_block(),
            Token::Ident(_) => self.parse_ident(),
            Token::StringIndex(_) => self.parse_string(),
            Token::SemiColon => self.parse_semicolon(),
            t => panic!("{t:?} not implemented"),
        }
    }
}
