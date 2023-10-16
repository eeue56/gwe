use crate::tokenizer::{error_with_info, FullyQualifiedToken, Token};
use std::slice::Iter;

#[derive(PartialEq, Debug, Clone)]
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
    FunctionCall {
        name: String,
        args: Vec<Box<Expression>>,
    },
    MemoryReference {
        offset: i32,
        length: i32,
    },
}

fn try_to_match<'a>(tokens: &mut Iter<'a, FullyQualifiedToken>, token: Token) -> Option<String> {
    match tokens.next() {
        Some(fqt) => {
            if &token != &fqt.token {
                Some(
                    error_with_info::<()>(format!("Expected : but got {}", &fqt.token), fqt)
                        .unwrap_err(),
                )
            } else {
                None
            }
        }
        None => Some(format!("Expected {} but got nothing", token)),
    }
}

fn parse_params<'a>(tokens: &mut Iter<'a, FullyQualifiedToken>) -> Result<Vec<Expression>, String> {
    let mut tokens_for_current_expression: Vec<FullyQualifiedToken> = vec![];
    let mut arguments: Vec<Expression> = vec![];

    while let maybe_fqt = tokens.next() {
        match maybe_fqt {
            Some(fqt) => match &fqt.token {
                Token::RightParen => break,
                Token::Comma => {
                    match parse_expression(&mut tokens_for_current_expression.iter()) {
                        Ok(exp) => arguments.push(exp),
                        Err(error) => return Err(error),
                    };

                    tokens_for_current_expression.clear();
                }
                _ => {
                    tokens_for_current_expression.push(fqt.clone());
                }
            },
            None => return Err(String::from("Failed parsing params")),
        }
    }

    if tokens_for_current_expression.len() > 0 {
        match parse_expression(&mut tokens_for_current_expression.iter()) {
            Ok(exp) => arguments.push(exp),
            Err(error) => return Err(error),
        };
    }

    Ok(arguments)
}

pub fn parse_expression<'a>(
    tokens: &mut Iter<'a, FullyQualifiedToken>,
) -> Result<Expression, String> {
    let has_addition = tokens.clone().any(|fqt| fqt.token == Token::Plus);
    let has_assign = tokens.clone().any(|fqt| fqt.token == Token::Assign);

    if has_addition && !has_assign {
        let sides: Vec<Vec<FullyQualifiedToken>> = tokens
            .clone()
            .as_slice()
            .splitn(2, |fqt| fqt.token == Token::Plus)
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

    while let maybe_fqt = tokens.next() {
        match maybe_fqt {
            Some(fqt) => {
                match &fqt.token {
                    Token::Return => {
                        return parse_expression(tokens).map(|exp| Expression::Return {
                            expression: Box::new(exp),
                        })
                    }
                    Token::Local => match tokens.next().map(|fqt|  &fqt.token) {
                        Some(Token::Identifier { body: name }) => {
                            // skip ":"
                            if let Some(error) = try_to_match(tokens, Token::Colon) {
                                return Err(error);
                            }

                            match tokens.next() {
                                Some(fqt) => match &fqt.token {
                                    Token::Identifier { body: type_name } => {
                                        // Skip "="
                                        if let Some(error) = try_to_match(tokens, Token::Assign) {
                                            return Err(error);
                                        }

                                        return parse_expression(tokens).map(|exp| Expression::LocalAssign {
                                            name: name.to_string(),
                                            type_name: type_name.to_string(),
                                            expression: Box::new(exp),
                                        });
                                    }

                                    token => {
                                        return error_with_info(format!(
                                            "Failed parsing expression, got unexpected token {}",
                                            token
                                        ), fqt)
                                    }
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
                    Token::Global => match tokens.next() {
                        Some(fqt) => match &fqt.token {
                            Token::Identifier { body: name } => {
                                // skip ":"
                                if let Some(error) = try_to_match(tokens, Token::Colon) {
                                    return Err(error);
                                }

                                match tokens.next().map(|fqt| &fqt.token) {
                                    Some(Token::Identifier { body: type_name }) => {
                                        // skip "="
                                        if let Some(error) = try_to_match(tokens, Token::Assign) {
                                            return Err(error);
                                        }

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
                            token => {
                                return error_with_info(format!(
                                    "Failed parsing expression, got unexpected token {}",
                                    token
                                ), fqt)
                            }

                        }
                        None => {
                            return Err(format!(
                                "Failed parsing expression, was expecting an identifier token for the variable name",
                            ))
                        }
                    },
                    Token::Identifier { body } => {
                        match tokens.next() {
                            Some(fqt) => match &fqt.token {
                                Token::LeftParen => match parse_params(tokens) {
                                    Ok(expressions) => return Ok(Expression::FunctionCall { name: body.to_string(), args: expressions.iter().map(|e| Box::new(e.clone())).collect::<Vec<Box<Expression>>>() }),
                                    Err(error) => return Err(error)
                                },
                                token => return error_with_info(format!("Unexpected token {}", token), fqt)
                            }
                            None => return Ok(Expression::Variable {
                                body: body.to_string(),
                            })
                        }
                    }
                    Token::RightBracket => {},
                    Token::Text { body } => return Ok(Expression::String { body: body.to_string() }),
                    Token::Number { body } => return Ok(Expression::Number {value: body.to_string()}),
                    value => {
                        return error_with_info(format!(
                            "Failed parsing expression, got unexpected token {}",
                            value
                        ), fqt)
                    }
                }
            }
            None => return Err(String::from("Failed parsing expression, ran out of tokens")),
        }
    }

    Err(String::from(""))
}
