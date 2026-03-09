use crate::{Token, pattern};

#[derive(Debug)]
pub struct Type(String);

#[derive(Debug)]
pub enum EnumDeclarationMembers {
    Simple(String),
    Variable(String, Type),
    Complex {
        name: String,
        content: Vec<(String, Type)>,
    },
}

#[derive(Debug)]
pub enum Expression {
    Block {
        content: Box<Vec<Expression>>,
    },
    String(String),
    VariableDeclaration {
        name: String,
        value: Box<Expression>,
    },
    FunctionDeclaration {
        name: String,
        arguments: Vec<(String, Expression)>,
        body: Box<Expression>,
    },
    EnumDeclaration {
        name: String,
        members: Box<Vec<EnumDeclarationMembers>>,
    },
    StructDeclaration {
        name: String,
        content: Vec<(String, Type)>,
    },
    FunctionCall {
        name: String,
        arguments: Box<Vec<Expression>>,
    },
    VariableAssignment {
        name: String,
        value: Box<Expression>,
    },
    SemiColon,
}

#[derive(Debug)]
pub struct ParserMonad<'a> {
    pub tokens: &'a [Token],
    pub string_interning_store: Vec<String>,
}

type ParserResult<'a> = Result<(Expression, ParserMonad<'a>), String>;
impl<'a> ParserMonad<'a> {
    pub fn consume(&mut self) -> Result<&Token, String> {
        let (first, tokens) = self
            .tokens
            .split_first()
            .ok_or("Tokenstream ended early".to_owned())?;
        self.tokens = tokens;
        Ok(first)
    }
    pub fn first(&self) -> Result<&Token, String> {
        self.tokens
            .first()
            .ok_or("Tokenstream ended early".to_owned())
    }
    pub fn parse_file(mut self) -> ParserResult<'a> {
        let mut block_content = vec![];
        loop {
            match self.tokens.first() {
                None => {
                    break;
                }
                _ => {
                    let (expression, new_parser) = self.parse()?;
                    block_content.push(expression);
                    self = new_parser;
                }
            };
        }
        Ok((
            Expression::Block {
                content: Box::new(block_content),
            },
            self,
        ))
    }
    pub fn parse(self) -> ParserResult<'a> {
        match self.first()? {
            Token::Fn => self.parse_function_def(),
            Token::Let => self.parse_let(),
            Token::LBrace => self.parse_block(),
            Token::StringIndex(_) => self.parse_string(),
            Token::EnumDefinition => self.parse_enum_def(),
            Token::SemiColon => self.parse_semicolon(),
            Token::Ident(_) => self.parse_ident(),
            Token::StructDefinition => Self {
                tokens: &self.tokens[1..],
                ..self
            }
            .parse_struct_def(),
            t => panic!("Token {t:?} is not yet implemented"),
        }
    }
    fn parse_string(mut self) -> ParserResult<'a> {
        let (tokens, id) = pattern!(self.tokens, Token::StringIndex(id))?;
        Ok((
            Expression::String(std::mem::take(&mut self.string_interning_store[*id])),
            Self { tokens, ..self },
        ))
    }
    fn parse_semicolon(self) -> ParserResult<'a> {
        let (tokens, _) = pattern!(self.tokens, Token::SemiColon)?;
        Ok((Expression::SemiColon, Self { tokens, ..self }))
    }
    fn parse_block(self) -> ParserResult<'a> {
        let (tokens, _) = pattern!(self.tokens, Token::LBrace)?;
        let mut parser = Self {
            tokens: tokens,
            ..self
        };
        let mut block_content = vec![];
        loop {
            match parser.first()? {
                Token::RBrace => {
                    parser.tokens = &parser.tokens[1..];
                    break;
                }
                _ => {
                    let (expression, new_parser) = parser.parse()?;
                    block_content.push(expression);
                    parser = new_parser;
                }
            };
        }
        Ok((
            Expression::Block {
                content: Box::new(block_content),
            },
            parser,
        ))
    }
    fn parse_ident(self) -> ParserResult<'a> {
        match self.tokens.get(1).ok_or("Tokenstream ended early")? {
            Token::LParen => self.parse_function_call(),
            Token::Equals => self.parse_assignment(),
            Token::Namespace => self.parse_ns(),
            t => panic!("Token {t:?} is not yet implemented {:#?}", self.tokens),
        }
    }
    fn parse_ns(self) -> ParserResult<'a> {
        unimplemented!()
    }
    fn parse_struct(self) -> ParserResult<'a> {
        unimplemented!()
    }
    fn parse_assignment(self) -> ParserResult<'a> {
        let (tokens, name) = pattern!(self.tokens, Token::Ident(name))?;
        let (tokens, _) = pattern!(tokens, Token::Equals)?;
        let (expression, parser) = Self { tokens, ..self }.parse()?;
        if matches!(expression, Expression::SemiColon) {
            return Err("Variable declaration requires a value, found ';'".to_owned());
        }
        Ok((
            Expression::VariableAssignment {
                name: name.to_owned(),
                value: Box::new(expression),
            },
            parser,
        ))
    }
    fn parse_let(self) -> ParserResult<'a> {
        let (tokens, _) = pattern!(self.tokens, Token::Let)?;
        let (tokens, name) = pattern!(tokens, Token::Ident(name))?;
        let (tokens, _) = pattern!(tokens, Token::Equals)?;
        let (expression, parser) = Self { tokens, ..self }.parse()?;
        if matches!(expression, Expression::SemiColon) {
            return Err("Variable declaration requires a value, found ';'".to_owned());
        }
        Ok((
            Expression::VariableDeclaration {
                name: name.to_owned(),
                value: Box::new(expression),
            },
            parser,
        ))
    }
    fn parse_function_call(self) -> ParserResult<'a> {
        let (tokens, name) = pattern!(self.tokens, Token::Ident(name))?;
        let (tokens, _) = pattern!(tokens, Token::LParen)?;
        let mut parser = Self { tokens, ..self };
        let mut arguments = vec![];
        loop {
            match parser.first()? {
                Token::RParen => {
                    parser.tokens = &parser.tokens[1..];
                    break;
                }
                Token::Comma => parser.tokens = &parser.tokens[1..],
                _ => {
                    let (expression, new_parser) = parser.parse()?;
                    arguments.push(expression);
                    parser = new_parser;
                }
            };
        }
        Ok((
            Expression::FunctionCall {
                name: name.to_owned(),
                arguments: Box::new(arguments),
            },
            parser,
        ))
    }
    fn parse_function_def(self) -> ParserResult<'a> {
        let (tokens, _) = pattern!(self.tokens, Token::Fn)?;
        let (tokens, name) = pattern!(tokens, Token::Ident(name))?;
        let (tokens, _) = pattern!(tokens, Token::LParen)?;
        let mut parser = Self { tokens, ..self };
        let mut arguments = vec![];
        let mut current_argument_name = None::<String>;
        loop {
            match parser.first()? {
                Token::RParen => {
                    parser.tokens = &parser.tokens[1..];
                    break;
                }
                Token::Comma | Token::Colon => parser.tokens = &parser.tokens[1..],
                Token::Ident(name) => {
                    current_argument_name = Some(name.to_owned());
                    parser.tokens = &parser.tokens[1..];
                }
                _ => {
                    let (expression, new_parser) = parser.parse()?;
                    arguments.push((
                        std::mem::take(&mut current_argument_name).ok_or(
                            "Function arguments must have a name then the type".to_owned(),
                        )?,
                        expression,
                    ));
                    parser = new_parser;
                }
            };
        }
        let (body, parser) = parser.parse()?;

        Ok((
            Expression::FunctionDeclaration {
                name: name.to_owned(),
                arguments,
                body: Box::new(body),
            },
            parser,
        ))
    }
    fn parse_enum_def(self) -> ParserResult<'a> {
        let (tokens, _) = pattern!(self.tokens, Token::EnumDefinition)?;
        let (tokens, name) = pattern!(tokens, Token::Ident(name))?;
        let (tokens, _) = pattern!(tokens, Token::LBrace)?;
        let mut members = vec![];
        let mut parser = Self { tokens, ..self };
        loop {
            match parser.first()? {
                Token::RBrace => {
                    parser.tokens = &parser.tokens[1..];
                    break;
                }
                Token::Comma => {
                    parser.tokens = &parser.tokens[1..];
                }
                _ => {
                    let (member, new_parser) = parser.parse_enum_member()?;
                    members.push(member);
                    parser = new_parser;
                }
            };
        }
        Ok((
            Expression::EnumDeclaration {
                name: name.to_owned(),
                members: Box::new(members),
            },
            parser,
        ))
    }
    fn parse_enum_member(self) -> Result<(EnumDeclarationMembers, Self), String> {
        let (tokens, name) = pattern!(self.tokens, Token::Ident(name))?;

        match tokens.first().ok_or("Tokenstream ended early")? {
            Token::RBrace | Token::Comma => Ok((
                EnumDeclarationMembers::Simple(name.to_owned()),
                Self { tokens, ..self },
            )),
            Token::LParen => {
                let (tokens, type_name) = pattern!(tokens[1..], Token::Ident(x))?;
                let (tokens, _) = pattern!(tokens, Token::RParen)?;
                if !tokens
                    .first()
                    .map(|token| *token == Token::Comma || *token == Token::RBrace)
                    .unwrap_or(false)
                {
                    return Err("Expected a , or } after enum".to_owned());
                }
                Ok((
                    EnumDeclarationMembers::Variable(name.to_owned(), Type(type_name.to_owned())),
                    Self { tokens, ..self },
                ))
            }
            Token::LBrace => {
                let (expression, parser) = self.parse_struct()?;
                let Expression::StructDeclaration { name, content } = expression else {
                    unreachable!()
                };
                Ok((EnumDeclarationMembers::Complex { name, content }, parser))
            }
            _ => Err("Enum Member expected a , ( or {".to_owned()),
        }
    }

    fn parse_struct_def(self) -> ParserResult<'a> {
        let (tokens, name) = pattern!(self.tokens, Token::Ident(name))?;
        let (tokens, _) = pattern!(tokens, Token::LBrace)?;
        let mut mtokens = tokens;
        let mut content = vec![];
        loop {
            let tokens = if mtokens.first() == Some(&Token::RBrace) {
                mtokens = &mtokens[1..];
                break;
            } else if mtokens.first() == Some(&Token::Comma) {
                if mtokens.get(1) == Some(&Token::RBrace) {
                    mtokens = &mtokens[2..];
                    break;
                }
                &mtokens[1..]
            } else {
                mtokens
            };
            let (tokens, name) = pattern!(tokens, Token::Ident(name))?;
            let (tokens, _) = pattern!(tokens, Token::Colon)?;
            let (tokens, type_name) = pattern!(tokens, Token::Ident(name))?;
            mtokens = tokens;
            content.push((name.to_owned(), Type(type_name.to_owned())));
        }
        Ok((
            Expression::StructDeclaration {
                name: name.to_owned(),
                content,
            },
            Self {
                tokens: mtokens,
                ..self
            },
        ))
    }
}
