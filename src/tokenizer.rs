use std::fmt::Display;
use std::fmt::Formatter;

#[derive(PartialEq, Debug, Clone)]
pub struct TokenInfo {
    pub line: i32,
    pub index: i32,
}

#[derive(PartialEq, Debug, Clone)]
pub enum Token {
    LeftParen,
    RightParen,
    Identifier { body: String },
    Number { body: String },
    Fn,
    Colon,
    LeftBracket,
    RightBracket,
    Comma,
    Return,
    Semicolon,
    Local,
    Global,
    Assign,
    Text { body: String },
    Plus,
    Export,
    Import,
    Dot,
}

#[derive(PartialEq, Debug, Clone)]
pub struct FullyQualifiedToken {
    pub token: Token,
    pub info: TokenInfo,
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Token::LeftParen => "(",
                Token::RightParen => "(",
                Token::Identifier { body } => body,
                Token::Fn => "fn",
                Token::Colon => ":",
                Token::LeftBracket => "{",
                Token::RightBracket => "}",
                Token::Comma => ",",
                Token::Return => "return",
                Token::Semicolon => ";",
                Token::Local => "local",
                Token::Global => "global",
                Token::Assign => "=",
                Token::Text { body } => body,
                Token::Plus => "+",
                Token::Number { body } => body,
                Token::Export => "export",
                Token::Import => "import",
                Token::Dot => ".",
            }
        )
    }
}

pub fn error_with_info<A>(error: String, token: &FullyQualifiedToken) -> Result<A, String> {
    Err(format!(
        "{} at line {}, index {}",
        error,
        token.info.line + 1,
        token.info.index
    ))
}

fn is_identifier_char(char: char) -> bool {
    char.is_alphanumeric() || char == '_'
}

fn is_number_string(str: &str) -> bool {
    str.chars().all(|char| char.is_numeric() || char == '.')
}

fn possibly_push_current_buffer(
    tokens: &mut Vec<FullyQualifiedToken>,
    current_buffer: &mut Vec<char>,
    line_number: i32,
    char_index: i32,
) {
    if current_buffer.len() > 0 {
        let chars: String = current_buffer.as_slice().into_iter().collect();

        let token = match chars.as_ref() {
            "fn" => Token::Fn,
            "return" => Token::Return,
            "local" => Token::Local,
            "global" => Token::Global,
            "export" => Token::Export,
            "import" => Token::Import,
            x if is_number_string(x) => Token::Number { body: chars },
            _ => Token::Identifier { body: chars },
        };

        tokens.push(FullyQualifiedToken {
            token: token,
            info: TokenInfo {
                line: line_number,
                index: char_index,
            },
        });

        current_buffer.clear();
    }
}

fn push_text(
    tokens: &mut Vec<FullyQualifiedToken>,
    current_buffer: &mut Vec<char>,
    line_number: i32,
    char_index: i32,
) {
    tokens.push(FullyQualifiedToken {
        token: Token::Text {
            body: current_buffer.as_slice().into_iter().collect(),
        },
        info: TokenInfo {
            line: line_number,
            index: char_index,
        },
    });
    current_buffer.clear();
}

pub fn tokenize(body: String) -> Vec<FullyQualifiedToken> {
    let chars = body.chars();
    let mut tokens: Vec<FullyQualifiedToken> = vec![];
    let mut current_buffer: Vec<char> = vec![];
    let mut is_in_quotes = false;
    let mut line_number = 0;
    let mut char_index = 0;

    for char in chars {
        match char {
            '"' => {
                if is_in_quotes {
                    push_text(&mut tokens, &mut current_buffer, line_number, char_index);
                    is_in_quotes = false
                } else {
                    possibly_push_current_buffer(
                        &mut tokens,
                        &mut current_buffer,
                        line_number,
                        char_index,
                    );
                    is_in_quotes = true
                }
            }
            char if is_in_quotes => current_buffer.push(char),
            '(' => {
                possibly_push_current_buffer(
                    &mut tokens,
                    &mut current_buffer,
                    line_number,
                    char_index,
                );
                tokens.push(FullyQualifiedToken {
                    token: Token::LeftParen,
                    info: TokenInfo {
                        line: line_number,
                        index: char_index,
                    },
                })
            }
            ')' => {
                possibly_push_current_buffer(
                    &mut tokens,
                    &mut current_buffer,
                    line_number,
                    char_index,
                );
                tokens.push(FullyQualifiedToken {
                    token: Token::RightParen,
                    info: TokenInfo {
                        line: line_number,
                        index: char_index,
                    },
                })
            }
            ':' => {
                possibly_push_current_buffer(
                    &mut tokens,
                    &mut current_buffer,
                    line_number,
                    char_index,
                );
                tokens.push(FullyQualifiedToken {
                    token: Token::Colon,
                    info: TokenInfo {
                        line: line_number,
                        index: char_index,
                    },
                })
            }
            ' ' | '\n' => {
                possibly_push_current_buffer(
                    &mut tokens,
                    &mut current_buffer,
                    line_number,
                    char_index,
                );
            }
            '{' => {
                possibly_push_current_buffer(
                    &mut tokens,
                    &mut current_buffer,
                    line_number,
                    char_index,
                );
                tokens.push(FullyQualifiedToken {
                    token: Token::LeftBracket,
                    info: TokenInfo {
                        line: line_number,
                        index: char_index,
                    },
                })
            }
            '}' => {
                possibly_push_current_buffer(
                    &mut tokens,
                    &mut current_buffer,
                    line_number,
                    char_index,
                );
                tokens.push(FullyQualifiedToken {
                    token: Token::RightBracket,
                    info: TokenInfo {
                        line: line_number,
                        index: char_index,
                    },
                })
            }
            ',' => {
                possibly_push_current_buffer(
                    &mut tokens,
                    &mut current_buffer,
                    line_number,
                    char_index,
                );
                tokens.push(FullyQualifiedToken {
                    token: Token::Comma,
                    info: TokenInfo {
                        line: line_number,
                        index: char_index,
                    },
                })
            }
            ';' => {
                possibly_push_current_buffer(
                    &mut tokens,
                    &mut current_buffer,
                    line_number,
                    char_index,
                );
                tokens.push(FullyQualifiedToken {
                    token: Token::Semicolon,
                    info: TokenInfo {
                        line: line_number,
                        index: char_index,
                    },
                })
            }
            '=' => {
                possibly_push_current_buffer(
                    &mut tokens,
                    &mut current_buffer,
                    line_number,
                    char_index,
                );
                tokens.push(FullyQualifiedToken {
                    token: Token::Assign,
                    info: TokenInfo {
                        line: line_number,
                        index: char_index,
                    },
                })
            }
            '+' => {
                possibly_push_current_buffer(
                    &mut tokens,
                    &mut current_buffer,
                    line_number,
                    char_index,
                );
                tokens.push(FullyQualifiedToken {
                    token: Token::Plus,
                    info: TokenInfo {
                        line: line_number,
                        index: char_index,
                    },
                })
            }
            '.' if is_number_string(
                current_buffer
                    .as_slice()
                    .into_iter()
                    .collect::<String>()
                    .as_str(),
            ) =>
            {
                current_buffer.push(char)
            }
            '.' => {
                possibly_push_current_buffer(
                    &mut tokens,
                    &mut current_buffer,
                    line_number,
                    char_index,
                );
                tokens.push(FullyQualifiedToken {
                    token: Token::Dot,
                    info: TokenInfo {
                        line: line_number,
                        index: char_index,
                    },
                })
            }
            char if is_identifier_char(char) => current_buffer.push(char),
            _ => (),
        }
        char_index += 1;
        if char == '\n' {
            line_number += 1;
            char_index = 0;
        }
    }

    possibly_push_current_buffer(&mut tokens, &mut current_buffer, line_number, char_index);

    tokens
}

#[cfg(test)]
mod tests {
    use super::Token::*;
    use super::*;

    #[test]
    fn tokenize_parens_passes() {
        assert_eq!(
            tokenize(String::from("())("))
                .iter()
                .map(|fqt| fqt.clone().token)
                .collect::<Vec<Token>>(),
            vec![LeftParen, RightParen, RightParen, LeftParen]
        )
    }

    #[test]
    fn tokenize_identifier_passes() {
        assert_eq!(
            tokenize(String::from("say_hi"))
                .iter()
                .map(|fqt| fqt.clone().token)
                .collect::<Vec<Token>>(),
            vec![Identifier {
                body: String::from("say_hi")
            }]
        )
    }

    #[test]
    fn tokenize_fn_passes() {
        assert_eq!(
            tokenize(String::from("fn say_hi()"))
                .iter()
                .map(|fqt| fqt.clone().token)
                .collect::<Vec<Token>>(),
            vec![
                Fn,
                Identifier {
                    body: String::from("say_hi")
                },
                LeftParen,
                RightParen
            ]
        )
    }

    #[test]
    fn tokenize_fn_with_args_passes() {
        assert_eq!(
            tokenize(String::from("fn say_hi(name: string) {\n}"))
                .iter()
                .map(|fqt| fqt.clone().token)
                .collect::<Vec<Token>>(),
            vec![
                Fn,
                Identifier {
                    body: String::from("say_hi")
                },
                LeftParen,
                Identifier {
                    body: String::from("name"),
                },
                Colon,
                Identifier {
                    body: String::from("string"),
                },
                RightParen,
                LeftBracket,
                RightBracket
            ]
        )
    }

    #[test]
    fn tokenize_empty_string_passes() {
        assert_eq!(
            tokenize(String::from("\"\""))
                .iter()
                .map(|fqt| fqt.clone().token)
                .collect::<Vec<Token>>(),
            vec![Token::Text {
                body: String::from("")
            }]
        )
    }

    #[test]
    fn tokenize_filled_string_passes() {
        assert_eq!(
            tokenize(String::from("\"Hello world this is a = test.\""))
                .iter()
                .map(|fqt| fqt.clone().token)
                .collect::<Vec<Token>>(),
            vec![Token::Text {
                body: String::from("Hello world this is a = test.")
            }]
        )
    }
    #[test]
    fn tokenize_addition_passes() {
        assert_eq!(
            tokenize(String::from("name + \"world\""))
                .iter()
                .map(|fqt| fqt.clone().token)
                .collect::<Vec<Token>>(),
            vec![
                Token::Identifier {
                    body: String::from("name")
                },
                Token::Plus,
                Token::Text {
                    body: String::from("world")
                }
            ]
        )
    }

    #[test]
    fn tokenize_number_addition_passes() {
        assert_eq!(
            tokenize(String::from("123 + 3.14"))
                .iter()
                .map(|fqt| fqt.clone().token)
                .collect::<Vec<Token>>(),
            vec![
                Token::Number {
                    body: String::from("123")
                },
                Token::Plus,
                Token::Number {
                    body: String::from("3.14")
                }
            ]
        )
    }

    #[test]
    fn import_passes() {
        assert_eq!(
            tokenize(String::from("import fn log(number: i32) console.log"))
                .iter()
                .map(|fqt| fqt.clone().token)
                .collect::<Vec<Token>>(),
            vec![
                Token::Import,
                Token::Fn,
                Token::Identifier {
                    body: String::from("log")
                },
                Token::LeftParen,
                Token::Identifier {
                    body: String::from("number")
                },
                Token::Colon,
                Token::Identifier {
                    body: String::from("i32")
                },
                Token::RightParen,
                Token::Identifier {
                    body: String::from("console")
                },
                Token::Dot,
                Token::Identifier {
                    body: String::from("log")
                },
            ]
        )
    }
}
