use std::{slice::Iter, vec};

use crate::{
    expressions::{parse_expression, Expression},
    tokenizer::{
        error_with_info, split_by_semicolon_within_brackets, tokenize, FullyQualifiedToken, Token,
    },
};

#[derive(PartialEq, Debug, Clone)]
pub struct Param {
    pub name: String,
    pub type_name: String,
}

#[derive(PartialEq, Debug, Clone)]
pub struct Function {
    pub name: String,
    pub expressions: Vec<Expression>,
    pub params: Vec<Param>,
    pub return_type: String,
}

#[derive(PartialEq, Debug, Clone)]
pub struct Export {
    pub external_name: String,
    pub function_name: String,
}

#[derive(PartialEq, Debug, Clone)]
pub struct ImportFunction {
    pub name: String,
    pub params: Vec<Param>,
    pub external_name: Vec<String>,
}

#[derive(PartialEq, Debug, Clone)]
pub struct ImportMemory {
    pub size: i32,
    pub external_name: Vec<String>,
}

#[derive(PartialEq, Debug, Clone)]
pub enum Block {
    Function(Function),
    Export(Export),
    ImportFunction(ImportFunction),
    ImportMemory(ImportMemory),
}

pub fn into_blocks(body: String) -> Vec<String> {
    let mut current_block: Vec<String> = Vec::new();
    let mut blocks: Vec<Vec<String>> = vec![];

    for line in body.split('\n') {
        if !line.trim().is_empty() {
            current_block.push(line.to_string());
            if line.starts_with("export") || line.starts_with("import") || line == "}" {
                blocks.push(current_block.clone());
                current_block.clear();
            }
        }
    }

    if !current_block.is_empty() {
        blocks.push(current_block.clone());
    }

    blocks.iter().map(|b| b.join("\n")).collect::<Vec<String>>()
}

fn parse_params(
    tokens: &mut Iter<'_, FullyQualifiedToken>,
    entry_fqt: FullyQualifiedToken,
) -> Result<Vec<Param>, String> {
    let param_name: &mut Option<String> = &mut None;

    let mut params: Vec<Param> = vec![];

    while let token = tokens.next().map(|fqt| &fqt.token) {
        match token {
            Some(Token::RightParen) => break,
            Some(Token::Identifier { body }) => match param_name {
                Some(n) => {
                    params.push(Param {
                        name: n.to_string(),
                        type_name: body.to_string(),
                    });

                    param_name.take();
                }
                None => {
                    param_name.replace(body.to_string());
                }
            },
            Some(Token::Comma) => {
                param_name.take();
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
    if let Some(name) = param_name {
        return error_with_info(
            format!("Failed to find type for param {}", name),
            &entry_fqt,
        );
    }

    Ok(params)
}

fn parse_function(tokens: Vec<FullyQualifiedToken>) -> Result<Function, String> {
    let mut tokens = tokens.iter();

    // fn
    let fn_token = tokens.next().unwrap();

    let function_name = match tokens.next().map(|fqt| &fqt.token) {
        Some(Token::Identifier { body }) => body,
        None => {
            return error_with_info(
                String::from("Expected a function name but got nothing"),
                fn_token,
            )
        }
        Some(token) => {
            return error_with_info(
                format!("Expected a function name but got {}", token),
                fn_token,
            )
        }
    };

    let open_parens = tokens.next();

    match open_parens.map(|fqt| &fqt.token) {
        Some(Token::LeftParen) => (),
        Some(token) => {
            return error_with_info(
                format!("Expected parens but got {}", token),
                open_parens.unwrap(),
            )
        }
        None => return Err("Expected parens but got nothing".to_string()),
    }

    let params = match parse_params(&mut tokens, open_parens.unwrap().clone()) {
        Err(error) => return Err(error),
        Ok(params) => params,
    };

    match tokens.next() {
        Some(fqt) => match &fqt.token {
            Token::Colon => (),
            token => {
                return error_with_info(
                    format!(
                        "Failed parsing function signature - expected return type, got {}",
                        token
                    ),
                    fqt,
                )
            }
        },
        None => return Err(String::from("Expected colon but got nothing")),
    }

    let return_type = match tokens.next() {
        Some(fqt) => match &fqt.token {
            Token::Identifier { body } => body.to_string(),
            token => {
                return error_with_info(
                    format!("Expected return type name, but got {}", token),
                    fqt,
                )
            }
        },
        None => return Err(String::from("Expected return type name, but got nothing")),
    };

    // {
    match tokens.next() {
        Some(fqt) => match &fqt.token {
            Token::LeftBracket => (),
            token => return error_with_info(format!("Expected {{ but got {}", token), fqt),
        },
        None => return Err(String::from("Expected { but got nothing")),
    }

    let mut expressions: Vec<Expression> = vec![];
    let mut original_tokens: Vec<FullyQualifiedToken> = vec![];

    for token in tokens.clone() {
        original_tokens.push(token.clone());
    }

    // cut off }
    original_tokens.truncate(original_tokens.len() - 1);

    let tokens_split_by_semicolon: Vec<Vec<FullyQualifiedToken>> =
        split_by_semicolon_within_brackets(original_tokens);

    for expression_tokens in tokens_split_by_semicolon.iter() {
        if expression_tokens.is_empty() {
            continue;
        }
        match parse_expression(
            &mut expression_tokens.iter(),
            expressions.clone(),
            params.clone(),
        ) {
            Ok(exp) => expressions.push(exp),
            Err(error) => return Err(error),
        }
    }

    Ok(Function {
        name: function_name.to_string(),
        expressions,
        params,
        return_type,
    })
}

fn parse_export(tokens: Vec<FullyQualifiedToken>) -> Result<Export, String> {
    let mut tokens = tokens.iter();
    tokens.next();

    let external_name = match tokens.next() {
        Some(fqt) => match &fqt.token {
            Token::Identifier { body } => body,
            token => {
                return error_with_info(
                    format!("Expected external name in export, got {}", token),
                    fqt,
                )
            }
        },
        None => return Err(String::from("Expected external name in export")),
    };

    let function_name = match tokens.next() {
        None => return Err(String::from("Expected function name in export")),
        Some(fqt) => match &fqt.token {
            Token::Identifier { body } => body,
            token => {
                return error_with_info(
                    format!("Expected function name in export, got {}", token),
                    fqt,
                )
            }
        },
    };

    Ok(Export {
        external_name: external_name.to_string(),
        function_name: function_name.to_string(),
    })
}

fn parse_import_function(tokens: Vec<FullyQualifiedToken>) -> Result<ImportFunction, String> {
    let mut tokens = tokens.iter();

    // import
    tokens.next();
    // fn
    tokens.next();

    let name = match tokens.next() {
        Some(fqt) => match &fqt.token {
            Token::Identifier { body } => body,
            token => {
                return error_with_info(
                    format!("Expected function name in import, got {}", token),
                    fqt,
                )
            }
        },
        None => return Err(String::from("Expected function name in export")),
    };

    let open_parens = tokens.next();

    match open_parens.map(|fqt| &fqt.token) {
        Some(Token::LeftParen) => (),
        Some(token) => {
            return error_with_info(
                format!("Expected parens but got {}", token),
                open_parens.unwrap(),
            )
        }
        None => return Err("Expected parens but got nothing".to_string()),
    }

    let params = match parse_params(&mut tokens, open_parens.unwrap().clone()) {
        Err(error) => return Err(error),
        Ok(params) => params,
    };

    let mut external_name: Vec<String> = vec![];

    while let fqt = tokens.next() {
        match fqt {
            Some(token) => match &token.token {
                Token::Identifier { body } => external_name.push(body.to_string()),
                Token::Dot => (),
                other => {
                    return error_with_info(
                        format!("Expected dot or identifier, got {}", other),
                        token,
                    )
                }
            },
            None => break,
        }
    }

    Ok(ImportFunction {
        name: name.to_string(),
        params,
        external_name,
    })
}

fn parse_import_memory(tokens: Vec<FullyQualifiedToken>) -> Result<ImportMemory, String> {
    let mut tokens = tokens.iter();

    // import
    tokens.next();
    // memory
    tokens.next();

    let size = match tokens.next() {
        Some(fqt) => match &fqt.token {
            Token::Number { body } => match body.parse::<i32>() {
                Ok(v) => v,
                Err(err) => return Err(err.to_string()),
            },
            token => return error_with_info(format!("Unexpected token {} in import", token), fqt),
        },
        None => return Err(String::from("Expected memory size but got nothing")),
    };

    let mut external_name: Vec<String> = vec![];

    while let fqt = tokens.next() {
        match fqt {
            Some(token) => match &token.token {
                Token::Identifier { body } => external_name.push(body.to_string()),
                Token::Dot => (),
                other => {
                    return error_with_info(
                        format!("Expected dot or identifier, got {}", other),
                        token,
                    )
                }
            },
            None => break,
        }
    }

    Ok(ImportMemory {
        size,
        external_name,
    })
}

pub fn parse_block(body: String) -> Result<Block, String> {
    let tokens = tokenize(body);

    match tokens.first().map(|fqt| &fqt.token) {
        Some(Token::Fn) => parse_function(tokens).map(Block::Function),
        Some(Token::Export) => parse_export(tokens).map(Block::Export),
        Some(Token::Import) => match tokens.get(1).map(|fqt| &fqt.token) {
            Some(Token::Fn) => parse_import_function(tokens).map(Block::ImportFunction),
            Some(Token::Memory) => parse_import_memory(tokens).map(Block::ImportMemory),
            _ => Err(String::from("Unexpected token in import statement")),
        },
        _ => Err(String::from("Unrecoginzed block")),
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn export_block() {
        assert_eq!(
            parse_block(String::from("export sayHello say_hello")),
            Ok(Block::Export(Export {
                external_name: String::from("sayHello"),
                function_name: String::from("say_hello")
            }))
        )
    }

    #[test]
    fn multiple_blocks() {
        let blocks = into_blocks(String::from(
            "import fn log(number: i32) console.log
import memory 1 js.mem

fn main(): void {
    log(3.14);
}",
        ));

        assert_eq!(
            blocks,
            vec![
                "import fn log(number: i32) console.log",
                "import memory 1 js.mem",
                "fn main(): void {
    log(3.14);
}"
            ]
        )
    }

    #[test]
    fn single_block() {
        let blocks = into_blocks(String::from(
            "fn main(): void {
    log(3.14);
}",
        ));

        assert_eq!(
            blocks,
            vec![
                "fn main(): void {
    log(3.14);
}"
            ]
        )
    }
}
