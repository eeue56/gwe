pub mod tokenizer {
    use std::fmt::Display;
    use std::fmt::Formatter;

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
    }

    impl Display for Token {
        fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
            // Use `self.number` to refer to each positional data point.

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
                }
            )
        }
    }

    fn is_identifier_char(char: char) -> bool {
        char.is_alphanumeric() || char == '_'
    }

    fn is_number_string(str: &str) -> bool {
        str.chars().all(|char| char.is_numeric() || char == '.')
    }

    fn possibly_push_current_buffer(tokens: &mut Vec<Token>, current_buffer: &mut Vec<char>) {
        if current_buffer.len() > 0 {
            let chars: String = current_buffer.as_slice().into_iter().collect();

            match chars.as_ref() {
                "fn" => tokens.push(Token::Fn),
                "return" => tokens.push(Token::Return),
                "local" => tokens.push(Token::Local),
                "global" => tokens.push(Token::Global),
                x if is_number_string(x) => tokens.push(Token::Number { body: chars }),
                _ => tokens.push(Token::Identifier { body: chars }),
            }

            current_buffer.clear();
        }
    }

    fn push_text(tokens: &mut Vec<Token>, current_buffer: &mut Vec<char>) {
        tokens.push(Token::Text {
            body: current_buffer.as_slice().into_iter().collect(),
        });
        current_buffer.clear();
    }

    pub fn tokenize(body: String) -> Vec<Token> {
        let chars = body.chars();
        let mut tokens: Vec<Token> = vec![];
        let mut current_buffer: Vec<char> = vec![];
        let mut is_in_quotes = false;

        for char in chars {
            match char {
                '"' => {
                    if is_in_quotes {
                        push_text(&mut tokens, &mut current_buffer);
                        is_in_quotes = false
                    } else {
                        possibly_push_current_buffer(&mut tokens, &mut current_buffer);
                        is_in_quotes = true
                    }
                }
                char if is_in_quotes => current_buffer.push(char),
                '(' => {
                    possibly_push_current_buffer(&mut tokens, &mut current_buffer);
                    tokens.push(Token::LeftParen)
                }
                ')' => {
                    possibly_push_current_buffer(&mut tokens, &mut current_buffer);
                    tokens.push(Token::RightParen)
                }
                ':' => {
                    possibly_push_current_buffer(&mut tokens, &mut current_buffer);
                    tokens.push(Token::Colon)
                }
                ' ' | '\n' => {
                    possibly_push_current_buffer(&mut tokens, &mut current_buffer);
                }
                '{' => {
                    possibly_push_current_buffer(&mut tokens, &mut current_buffer);
                    tokens.push(Token::LeftBracket)
                }
                '}' => {
                    possibly_push_current_buffer(&mut tokens, &mut current_buffer);
                    tokens.push(Token::RightBracket)
                }
                ',' => {
                    possibly_push_current_buffer(&mut tokens, &mut current_buffer);
                    tokens.push(Token::Comma)
                }
                ';' => {
                    possibly_push_current_buffer(&mut tokens, &mut current_buffer);
                    tokens.push(Token::Semicolon)
                }
                '=' => {
                    possibly_push_current_buffer(&mut tokens, &mut current_buffer);
                    tokens.push(Token::Assign)
                }
                '+' => {
                    possibly_push_current_buffer(&mut tokens, &mut current_buffer);
                    tokens.push(Token::Plus)
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
                char if is_identifier_char(char) => current_buffer.push(char),
                _ => (),
            }
        }

        possibly_push_current_buffer(&mut tokens, &mut current_buffer);

        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::tokenizer::Token::*;
    use super::tokenizer::*;

    #[test]
    fn tokenize_parens_passes() {
        assert_eq!(
            tokenize(String::from("())(")),
            vec![LeftParen, RightParen, RightParen, LeftParen]
        )
    }

    #[test]
    fn tokenize_identifier_passes() {
        assert_eq!(
            tokenize(String::from("say_hi")),
            vec![Identifier {
                body: String::from("say_hi")
            }]
        )
    }

    #[test]
    fn tokenize_fn_passes() {
        assert_eq!(
            tokenize(String::from("fn say_hi()")),
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
            tokenize(String::from("fn say_hi(name: string) {\n}")),
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
            tokenize(String::from("\"\"")),
            vec![Token::Text {
                body: String::from("")
            }]
        )
    }

    #[test]
    fn tokenize_filled_string_passes() {
        assert_eq!(
            tokenize(String::from("\"Hello world this is a = test.\"")),
            vec![Token::Text {
                body: String::from("Hello world this is a = test.")
            }]
        )
    }
    #[test]
    fn tokenize_addition_passes() {
        assert_eq!(
            tokenize(String::from("name + \"world\"")),
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
            tokenize(String::from("123 + 3.14")),
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
}
