#![allow(irrefutable_let_patterns)]
use std::slice::Iter;

use tokenizer::{tokenize, Token};

mod tokenizer;

#[derive(PartialEq, Debug)]
pub enum Expression {
    Number { value: String },
    Variable { body: String },
    Return { expression: Box<Expression> },
}

#[derive(PartialEq, Debug)]
pub struct Param {
    name: String,
    type_name: String,
}

#[derive(PartialEq, Debug)]
pub struct Function {
    name: String,
    expressions: Vec<Expression>,
    params: Vec<Param>,
}

#[derive(PartialEq, Debug)]
pub enum Block {
    FunctionBlock(Function),
}

#[derive(PartialEq, Debug)]
pub struct Program {
    blocks: Vec<Block>,
}

fn into_blocks(body: String) -> Vec<String> {
    body.split("\n\n")
        .map(str::to_string)
        .filter(|block| block.len() > 0)
        .collect()
}

fn parse_expression<'a>(tokens: &mut Iter<'a, Token>) -> Result<Expression, String> {
    while let token = tokens.next() {
        match token {
            Some(Token::Return) => {
                return parse_expression(tokens).map(|exp| Expression::Return {
                    expression: Box::new(exp),
                })
            }
            Some(Token::Identifier { body }) => {
                return Ok(Expression::Variable {
                    body: body.to_string(),
                })
            }
            Some(Token::RightBracket) => {}
            Some(value) => {
                return Err(format!(
                    "Failed parsing params, got unexpected token {}",
                    value
                ))
            }
            None => return Err(String::from("Failed parsing params")),
        }
    }

    Err(String::from(""))
}

fn parse_function(tokens: Vec<Token>) -> Result<Function, String> {
    let mut tokens = tokens.iter();

    // fn
    tokens.next();

    let function_name = match tokens.next() {
        Some(Token::Identifier { body }) => body,
        None => return Err(String::from("Expected function name but got nothing")),
        _ => return Err(String::from("Expected function name but got ")),
    };

    // parens
    tokens.next();

    let param_name: &mut Option<String> = &mut None;

    let mut params: Vec<Param> = vec![];

    while let token = tokens.next() {
        match token {
            Some(Token::RightParen) => break,
            Some(Token::Identifier { body }) => match param_name {
                Some(n) => {
                    params.push(Param {
                        name: n.to_string(),
                        type_name: body.to_string(),
                    });

                    param_name.take();
                    ()
                }
                None => {
                    param_name.replace(body.to_string());
                    ()
                }
            },
            Some(Token::Comma) => {
                param_name.take();
                ()
            }
            Some(Token::Colon) => (),
            Some(value) => {
                return Err(format!(
                    "Failed parsing params, got unexpected token {}",
                    value
                ))
            }
            None => return Err(String::from("Failed parsing params")),
        }
    }

    // {
    tokens.next();

    let mut expressions: Vec<Expression> = vec![];
    let mut original_tokens: Vec<Token> = vec![];

    for token in tokens.clone() {
        original_tokens.push(token.clone());
    }

    // cut off }
    original_tokens.truncate(original_tokens.len() - 1);

    let tokens_split_by_semicolon: Vec<&[Token]> = original_tokens
        .split(|token| match token {
            Token::Semicolon => true,
            _ => false,
        })
        .collect::<Vec<&[Token]>>();

    for expression_tokens in tokens_split_by_semicolon.iter() {
        if expression_tokens.len() < 1 {
            continue;
        }
        match parse_expression(&mut expression_tokens.iter()) {
            Ok(exp) => expressions.push(exp),
            Err(error) => return Err(error),
        }
    }

    Ok(Function {
        name: function_name.to_string(),
        expressions: expressions,
        params,
    })
}

fn parse_block(body: String) -> Result<Block, String> {
    let tokens = tokenize(body);

    match tokens.first() {
        Some(Token::Fn) => parse_function(tokens).map(|f| Block::FunctionBlock(f)),
        _ => Err(String::from("Unrecoginzed block")),
    }
}

pub fn parse(body: String) -> Result<Program, String> {
    let unparsed_blocks = into_blocks(body);

    if unparsed_blocks.len() == 0 {
        return Ok(Program { blocks: vec![] });
    }

    let parsed_blocks = unparsed_blocks.into_iter().map(parse_block);

    let mut blocks: Vec<Block> = vec![];
    let mut errors: Vec<String> = vec![];

    for parsed_block in parsed_blocks {
        match parsed_block {
            Ok(block) => blocks.push(block),
            Err(error) => errors.push(error),
        }
    }

    if errors.len() > 0 {
        Err(errors.join("\n"))
    } else {
        Ok(Program { blocks })
    }
}

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_empty_file_passes() {
        assert_eq!(parse(String::from("")), Ok(Program { blocks: vec![] }))
    }

    #[test]
    fn a_gibberish_file_fails_to_parse() {
        assert_eq!(
            parse(String::from("qwertyuio")),
            Err(String::from("Unrecoginzed block"))
        )
    }

    #[test]
    fn an_empty_function_passes() {
        assert_eq!(
            parse(String::from("fn say_hello(name: string) {}")),
            Ok(Program {
                blocks: vec![Block::FunctionBlock(Function {
                    name: String::from("say_hello"),
                    expressions: vec![],
                    params: vec![Param {
                        name: String::from("name"),
                        type_name: String::from("string")
                    }],
                })]
            })
        )
    }

    #[test]
    fn a_function_with_return_passes() {
        assert_eq!(
            parse(String::from("fn say_hello(name: string) { return name; }")),
            Ok(Program {
                blocks: vec![Block::FunctionBlock(Function {
                    name: String::from("say_hello"),
                    expressions: vec![Expression::Return {
                        expression: Box::new(Expression::Variable {
                            body: String::from("name")
                        })
                    }],
                    params: vec![Param {
                        name: String::from("name"),
                        type_name: String::from("string")
                    }],
                })]
            })
        )
    }
}
