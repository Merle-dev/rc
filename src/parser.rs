use crate::{Token, pattern};

#[derive(Debug)]
pub enum Expression {
    Block {
        content: Box<Vec<Expression>>,
    },
    String(String),
    VariableDeclaration {
        name: String,
        expression: Box<Expression>,
    },
    FunctionDeclaration {
        name: String,
        arguments: Vec<(String, Expression)>,
        body: Box<Expression>,
    },
    FunctionCall {
        name: String,
        arguments: Box<Vec<Expression>>,
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
    pub fn parse(self) -> ParserResult<'a> {
        match self
            .tokens
            .first()
            .ok_or("Tokenstream ended early".to_owned())?
        {
            Token::Fn => self.parse_function_def(),
            Token::LBrace => self.parse_block(),
            Token::FnCall(_) => self.parse_function_call(),
            Token::Let => self.parse_let(),
            Token::StringIndex(_) => self.parse_string(),
            Token::SemiColon => self.parse_semicolon(),
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
            match parser
                .tokens
                .first()
                .ok_or("Tokenstream ended early".to_owned())?
            {
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
    fn parse_let(self) -> ParserResult<'a> {
        let (tokens, _) = pattern!(self.tokens, Token::Let)?;
        let (tokens, name) = pattern!(tokens, Token::VariableName(name))?;
        let (tokens, _) = pattern!(tokens, Token::Equals)?;
        let (expression, parser) = Self { tokens, ..self }.parse()?;
        if matches!(expression, Expression::SemiColon) {
            return Err("Variable declaration requires a value, found ';'".to_owned());
        }
        Ok((
            Expression::VariableDeclaration {
                name: name.to_owned(),
                expression: Box::new(expression),
            },
            parser,
        ))
    }
    fn parse_function_call(self) -> ParserResult<'a> {
        let (tokens, name) = pattern!(self.tokens, Token::FnCall(name))?;
        let (tokens, _) = pattern!(tokens, Token::LParen)?;
        let mut parser = Self { tokens, ..self };
        let mut arguments = vec![];
        loop {
            match parser
                .tokens
                .first()
                .ok_or("Tokenstream ended early".to_owned())?
            {
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
        let (tokens, name) = pattern!(tokens, Token::FnDefinition(name))?;
        let (tokens, _) = pattern!(tokens, Token::LParen)?;
        let mut parser = Self { tokens, ..self };
        let mut arguments = vec![];
        let mut current_argument_name = None::<String>;
        loop {
            match parser
                .tokens
                .first()
                .ok_or("Tokenstream ended early".to_owned())?
            {
                Token::RParen => {
                    parser.tokens = &parser.tokens[1..];
                    break;
                }
                Token::Comma | Token::Colon => parser.tokens = &parser.tokens[1..],
                Token::VariableName(name) => {
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
}
