use std::fmt::Debug;

#[derive(Clone, Debug, PartialEq)]
enum Token {
    Fn,
    FnCall(String),
    FnDefinition(String),
    LParen,
    RParen,
    LBrace,
    RBrace,
    VariableName(String),
    Comma,
    SemiColon,
    StringIndex(usize),
}

#[derive(Clone, Debug, PartialEq)]
enum IntermediateToken {
    Token(Token),
    String(String),
}

#[derive(Clone, Debug)]
struct Tokenizer {
    tokens: Vec<IntermediateToken>,
    string_interning_store: Vec<String>,
    complete: bool,
}

impl IntermediateToken {
    fn as_str(&self) -> Option<&String> {
        match self {
            Self::Token(_) => None,
            Self::String(s) => Some(s),
        }
    }
    fn as_token(&self) -> Option<&Token> {
        match self {
            Self::Token(t) => Some(t),
            Self::String(_) => None,
        }
    }
}

impl Tokenizer {
    fn string_interning(self) -> Self {
        let (tokens, string_interning_store) = self
            .tokens
            .iter()
            .filter_map(|itoken| itoken.as_str())
            .fold(
                (Vec::new(), Vec::new()),
                |(mut acc_tokens, mut acc_store), string| {
                    string.split('"').enumerate().for_each(|(i, segment)| {
                        if (i & 1) == 0 {
                            acc_tokens.push(IntermediateToken::String(segment.to_owned()));
                        } else {
                            acc_tokens.push(IntermediateToken::Token(Token::StringIndex(
                                acc_store.len(),
                            )));
                            acc_store.push(segment.to_owned());
                        }
                    });
                    (acc_tokens, acc_store)
                },
            );

        Self {
            string_interning_store,
            tokens,
            ..self
        }
    }
    fn split_up(self) -> Self {
        let tokens = self
            .tokens
            .into_iter()
            .map(|itoken| match itoken {
                IntermediateToken::String(string) => {
                    string
                        .split_whitespace()
                        .fold(
                            (Vec::<IntermediateToken>::new(), String::new()),
                            |(mut preidents, mut current_ident), segment| {
                                segment.chars().for_each(|char| {
                                    if char.is_ascii_alphanumeric() || char == ' ' {
                                        current_ident.push(char);
                                    } else {
                                        preidents.push(IntermediateToken::String(std::mem::take(
                                            &mut current_ident,
                                        )));
                                        preidents.push(IntermediateToken::String(char.to_string()));
                                    }
                                });

                                preidents.push(IntermediateToken::String(std::mem::take(
                                    &mut current_ident,
                                )));
                                (preidents, current_ident)
                            },
                        )
                        .0
                }
                _ => vec![itoken],
            })
            .map(|tokens| {
                tokens
                    .into_iter()
                    .filter(|itoken| !itoken.as_str().is_some_and(|str| str.is_empty()))
                    .map(|itoken| match itoken.as_str().map(|str| str.as_str()) {
                        Some("fn") => IntermediateToken::Token(Token::Fn),
                        Some("(") => IntermediateToken::Token(Token::LParen),
                        Some(")") => IntermediateToken::Token(Token::RParen),
                        Some("{") => IntermediateToken::Token(Token::LBrace),
                        Some("}") => IntermediateToken::Token(Token::RBrace),
                        Some(",") => IntermediateToken::Token(Token::Comma),
                        Some(";") => IntermediateToken::Token(Token::SemiColon),
                        _ => itoken,
                    })
                    .collect::<Vec<IntermediateToken>>()
            })
            .flatten()
            .collect::<Vec<IntermediateToken>>();
        Self { tokens, ..self }
    }
    fn funcion_determination(self) -> Self {
        let tokens = self
            .tokens
            .iter()
            .enumerate()
            .map(|(i, itoken)| match itoken {
                IntermediateToken::Token(_) => itoken.clone(),
                IntermediateToken::String(name) => {
                    if self
                        .tokens
                        .get(i + 1)
                        .is_some_and(|token| *token == IntermediateToken::Token(Token::LParen))
                    {
                        if self
                            .tokens
                            .get(i - 1)
                            .is_some_and(|token| *token == IntermediateToken::Token(Token::Fn))
                        {
                            IntermediateToken::Token(Token::FnDefinition(name.clone()))
                        } else {
                            IntermediateToken::Token(Token::FnCall(name.clone()))
                        }
                    } else {
                        itoken.clone()
                    }
                }
            })
            .collect::<Vec<IntermediateToken>>();
        Self { tokens, ..self }
    }
}

#[derive(Debug)]
struct Parser {
    tokens: Vec<Token>,
    string_interning_store: Vec<String>,
    abstract_syntax_tree: Vec<Statement>,
}

#[derive(Debug)]
enum Expression {
    Block { statements: Vec<Statement> },
    StringIndex(usize),
}

#[derive(Debug)]
enum Statement {
    FunctionDeclaration {
        name: String,
        arguments: Vec<Token>,
        body: Expression,
    },
    FunctionCall {
        name: String,
        arguments: Vec<Expression>,
    },
}

type ParseResult<T> = Result<(T, Vec<Token>), String>;

impl Parser {
    fn from_tokenizer(tokenizer: Tokenizer) -> Self {
        let tokens = tokenizer
            .tokens
            .into_iter()
            .map(|itoken| {
                itoken
                    .as_token()
                    .cloned()
                    .ok_or_else(|| itoken.as_str().cloned().unwrap())
            })
            .collect::<Result<Vec<Token>, String>>()
            .unwrap();
        Self {
            tokens,
            string_interning_store: tokenizer.string_interning_store,
            abstract_syntax_tree: vec![],
        }
    }
    fn parse(self) -> Self {
        let Some(first_element) = self.tokens.first() else {
            return self;
        };
        match first_element {
            Token::Fn => {
                let (statement, tokens) = Self::parse_function_definition(self.tokens).unwrap();
                let mut abstract_syntax_tree = self.abstract_syntax_tree;
                abstract_syntax_tree.push(statement);
                Self {
                    tokens,
                    abstract_syntax_tree,
                    ..self
                }
            }
            _ => self,
        }
    }
    fn parse_block(tokens: Vec<Token>) -> ParseResult<Expression> {
        let (statement, tail) = match tokens
            .first()
            .ok_or("token stream too short while parsing block")?
        {
            Token::FnCall(_) => Self::parse_function_call(tokens)?,
            _ => unreachable!(),
        };
        Ok((
            Expression::Block {
                statements: vec![statement],
            },
            tail,
        ))
    }
    fn parse_function_call(tokens: Vec<Token>) -> ParseResult<Statement> {
        let ([function_call, lparen], tail) = tokens
            .split_first_chunk()
            .ok_or_else(|| "token stream too short while parsing function.".to_owned())?;
        let Token::FnCall(name) = function_call else {
            return Err("fn must be followed by function name.".to_owned());
        };
        if *lparen != Token::LParen {
            return Err("function name must be followed by '('.".to_owned());
        }
        let (arguments, tail) = tail
            .iter()
            .position(|token| *token == Token::RParen)
            .map(|pos| tail.split_at(pos))
            .ok_or("')' expected".to_owned())?;

        let arguments = arguments
            .into_iter()
            .map(|token| match token {
                Token::StringIndex(id) => Ok(Expression::StringIndex(*id)),
                _ => Err("Invalid tokens in function call".to_owned()),
            })
            .collect::<Result<Vec<Expression>, String>>()?;

        Ok((
            Statement::FunctionCall {
                name: name.to_string(),
                arguments,
            },
            tail[1..].to_vec(),
        ))
    }
    fn parse_function_definition(tokens: Vec<Token>) -> ParseResult<Statement> {
        let ([_, function_definition, lparen], tail) = tokens
            .split_first_chunk()
            .ok_or_else(|| "token stream too short while parsing function.".to_owned())?;
        let Token::FnDefinition(name) = function_definition else {
            return Err("fn must be followed by function name.".to_owned());
        };
        if *lparen != Token::LParen {
            return Err("function name must be followed by '('.".to_owned());
        }
        let (arguments, tail) = tail
            .iter()
            .position(|token| *token == Token::RParen)
            .map(|pos| tail.split_at(pos))
            .ok_or("')' expected".to_owned())?;
        let (lbrace, tail) = tail
            .get(1..)
            .ok_or("token stream ended on '('".to_owned())?
            .split_first()
            .ok_or("token stream too short while parsing function body.".to_owned())?;
        if *lbrace != Token::LBrace {
            return Err("function must have body starting with '{'".to_owned());
        }

        let (body, tail) = Self::parse_block(tail.to_vec())?;

        Ok((
            Statement::FunctionDeclaration {
                name: name.to_owned(),
                arguments: arguments.to_vec(),
                body,
            },
            tail,
        ))
    }
}

fn main() {
    let text: String = r#"
        fn main() {
            println("Hello");
        }
    "#
    .into();
    let intertoken = IntermediateToken::String(text.trim().to_string());
    let tokenizer = Tokenizer {
        tokens: vec![intertoken],
        string_interning_store: vec![],
        complete: false,
    }
    .string_interning()
    .split_up()
    .funcion_determination();
    let parser = Parser::from_tokenizer(tokenizer);
    dbg!(&parser);
    dbg!(parser.parse().parse());
}
