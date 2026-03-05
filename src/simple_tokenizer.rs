#[derive(Clone, Debug, PartialEq)]
pub enum IntermediateToken {
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
