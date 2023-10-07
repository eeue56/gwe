pub mod blocks {
    use crate::{
        expressions::{parse_expression, Expression},
        tokenizer::tokenizer::{tokenize, Token},
    };

    #[derive(PartialEq, Debug)]
    pub struct Param {
        pub name: String,
        pub type_name: String,
    }

    #[derive(PartialEq, Debug)]
    pub struct Function {
        pub name: String,
        pub expressions: Vec<Expression>,
        pub params: Vec<Param>,
    }

    #[derive(PartialEq, Debug)]
    pub enum Block {
        FunctionBlock(Function),
    }

    pub fn into_blocks(body: String) -> Vec<String> {
        body.split("\n\n")
            .map(str::to_string)
            .filter(|block| block.len() > 0)
            .collect()
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

    pub fn parse_block(body: String) -> Result<Block, String> {
        let tokens = tokenize(body);

        match tokens.first() {
            Some(Token::Fn) => parse_function(tokens).map(|f| Block::FunctionBlock(f)),
            _ => Err(String::from("Unrecoginzed block")),
        }
    }
}
