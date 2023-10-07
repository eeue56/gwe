pub mod parser {
    use crate::blocks::blocks::{into_blocks, parse_block, Block};

    #[derive(PartialEq, Debug, Clone)]
    pub struct Program {
        pub blocks: Vec<Block>,
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
}

#[cfg(test)]
mod tests {
    use super::parser::*;
    use crate::blocks::blocks::*;
    use crate::expressions::*;

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

    #[test]
    fn a_function_with_const_passes() {
        assert_eq!(
            parse(String::from(
                "
fn say_hello(name: string) {
    local x: string = name;
    return name;
}"
            )),
            Ok(Program {
                blocks: vec![Block::FunctionBlock(Function {
                    name: String::from("say_hello"),
                    expressions: vec![
                        Expression::LocalAssign {
                            name: String::from("x"),
                            type_name: String::from("string"),
                            expression: Box::new(Expression::Variable {
                                body: String::from("name")
                            })
                        },
                        Expression::Return {
                            expression: Box::new(Expression::Variable {
                                body: String::from("name")
                            })
                        }
                    ],
                    params: vec![Param {
                        name: String::from("name"),
                        type_name: String::from("string")
                    }],
                })]
            })
        )
    }

    #[test]
    fn a_function_with_global_const_passes() {
        assert_eq!(
            parse(String::from(
                "
fn say_hello(name: string) {
    global x: string = name;
    return name;
}"
            )),
            Ok(Program {
                blocks: vec![Block::FunctionBlock(Function {
                    name: String::from("say_hello"),
                    expressions: vec![
                        Expression::GlobalAssign {
                            name: String::from("x"),
                            type_name: String::from("string"),
                            expression: Box::new(Expression::Variable {
                                body: String::from("name")
                            })
                        },
                        Expression::Return {
                            expression: Box::new(Expression::Variable {
                                body: String::from("name")
                            })
                        }
                    ],
                    params: vec![Param {
                        name: String::from("name"),
                        type_name: String::from("string")
                    }],
                })]
            })
        )
    }

    #[test]
    fn a_function_with_local_addition_passes() {
        assert_eq!(
            parse(String::from(
                "
fn say_hello(name: string) {
    local x: string = \"Hello \" + name;
    return name;
}"
            )),
            Ok(Program {
                blocks: vec![Block::FunctionBlock(Function {
                    name: String::from("say_hello"),
                    expressions: vec![
                        Expression::LocalAssign {
                            name: String::from("x"),
                            type_name: String::from("string"),
                            expression: Box::new(Expression::Addition {
                                left: Box::new(Expression::String {
                                    body: String::from("Hello ")
                                }),
                                right: Box::new(Expression::Variable {
                                    body: String::from("name")
                                })
                            })
                        },
                        Expression::Return {
                            expression: Box::new(Expression::Variable {
                                body: String::from("name")
                            })
                        }
                    ],
                    params: vec![Param {
                        name: String::from("name"),
                        type_name: String::from("string")
                    }],
                })]
            })
        )
    }

    #[test]
    fn a_function_with_local_numeric_addition_passes() {
        assert_eq!(
            parse(String::from(
                "
fn say_hello() {
    local x: number = 123 + 3.14;
    return x;
}"
            )),
            Ok(Program {
                blocks: vec![Block::FunctionBlock(Function {
                    name: String::from("say_hello"),
                    expressions: vec![
                        Expression::LocalAssign {
                            name: String::from("x"),
                            type_name: String::from("number"),
                            expression: Box::new(Expression::Addition {
                                left: Box::new(Expression::Number {
                                    value: String::from("123")
                                }),
                                right: Box::new(Expression::Number {
                                    value: String::from("3.14")
                                })
                            })
                        },
                        Expression::Return {
                            expression: Box::new(Expression::Variable {
                                body: String::from("x")
                            })
                        }
                    ],
                    params: vec![],
                })]
            })
        )
    }
}
