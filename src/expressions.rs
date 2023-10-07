use crate::tokenizer::tokenizer::Token;
use std::slice::Iter;

#[derive(PartialEq, Debug)]
pub enum Expression {
    Number {
        value: String,
    },
    Variable {
        body: String,
    },
    Return {
        expression: Box<Expression>,
    },
    LocalAssign {
        name: String,
        type_name: String,
        expression: Box<Expression>,
    },
    GlobalAssign {
        name: String,
        type_name: String,
        expression: Box<Expression>,
    },
    Addition {
        left: Box<Expression>,
        right: Box<Expression>,
    },
    String {
        body: String,
    },
}

pub fn parse_expression<'a>(tokens: &mut Iter<'a, Token>) -> Result<Expression, String> {
    let has_addition = tokens.clone().any(|t| t == &Token::Plus);
    let has_assign = tokens.clone().any(|t| t == &Token::Assign);

    if has_addition && !has_assign {
        let sides: Vec<Vec<Token>> = tokens
            .clone()
            .as_slice()
            .splitn(2, |token| token == &Token::Plus)
            .map(|v| v.to_vec())
            .collect();

        let left_tokens = &mut sides[0].iter();
        let right_tokens = &mut sides[1].iter();

        return match parse_expression(left_tokens) {
            Ok(left) => match parse_expression(right_tokens) {
                Ok(right) => Ok(Expression::Addition {
                    left: Box::new(left),
                    right: Box::new(right),
                }),
                Err(err) => Err(err),
            },
            Err(err) => Err(err),
        };
    }

    while let token = tokens.next() {
        match token {
            Some(Token::Return) => {
                return parse_expression(tokens).map(|exp| Expression::Return {
                    expression: Box::new(exp),
                })
            }
            Some(Token::Local) => match tokens.next() {
                Some(Token::Identifier { body: name }) => {
                    // skip ":"

                    tokens.next();

                    match tokens.next() {
                        Some(Token::Identifier { body: type_name }) => {
                            // skip "="
                            tokens.next();

                            return parse_expression(tokens).map(|exp| Expression::LocalAssign {
                                name: name.to_string(),
                                type_name: type_name.to_string(),
                                expression: Box::new(exp),
                            });
                        }

                        Some(token) => {
                            return Err(format!(
                                "Failed parsing expression, got unexpected token {}",
                                token
                            ))
                        }
                        None => {
                            return Err(format!(
                                "Failed parsing expression, was expecting an identifier token for the type name",
                            ))
                        }
                    }
                }
                Some(token) => {
                    return Err(format!(
                        "Failed parsing expression, got unexpected token {}",
                        token
                    ))
                }
                None => {
                    return Err(format!(
                        "Failed parsing expression, was expecting an identifier token for the variable name",
                    ))
                }
            },
            Some(Token::Global) => match tokens.next() {
                Some(Token::Identifier { body: name }) => {
                    // skip ":"

                    tokens.next();

                    match tokens.next() {
                        Some(Token::Identifier { body: type_name }) => {
                            // skip "="
                            tokens.next();

                            return parse_expression(tokens).map(|exp| Expression::GlobalAssign {
                                name: name.to_string(),
                                type_name: type_name.to_string(),
                                expression: Box::new(exp),
                            });
                        }

                        Some(token) => {
                            return Err(format!(
                                "Failed parsing expression, got unexpected token {}",
                                token
                            ))
                        }
                        None => {
                            return Err(format!(
                                "Failed parsing expression, was expecting an identifier token for the type name",
                            ))
                        }
                    }
                }
                Some(token) => {
                    return Err(format!(
                        "Failed parsing expression, got unexpected token {}",
                        token
                    ))
                }
                None => {
                    return Err(format!(
                        "Failed parsing expression, was expecting an identifier token for the variable name",
                    ))
                }
            },
            Some(Token::Identifier { body }) => {
                return Ok(Expression::Variable {
                    body: body.to_string(),
                })
            }
            Some(Token::RightBracket) => {},
            Some(Token::Text { body }) => return Ok(Expression::String { body: body.to_string() }),
            Some(Token::Number { body }) => return Ok(Expression::Number {value: body.to_string()}),
            Some(value) => {
                return Err(format!(
                    "Failed parsing expression, got unexpected token {}",
                    value
                ))
            }
            None => return Err(String::from("Failed parsing expression, ran out of tokens")),
        }
    }

    Err(String::from(""))
}
