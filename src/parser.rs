
use crate::blocks::{into_blocks, parse_block, Block};

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blocks::*;
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
            parse(String::from("fn say_hello(name: string): void {}")),
            Ok(Program {
                blocks: vec![Block::FunctionBlock(Function {
                    name: String::from("say_hello"),
                    expressions: vec![],
                    params: vec![Param {
                        name: String::from("name"),
                        type_name: String::from("string")
                    }],
                    return_type: String::from("void"),
                })]
            })
        )
    }

    #[test]
    fn a_function_with_return_passes() {
        assert_eq!(
            parse(String::from(
                "fn say_hello(name: string): string { return name; }"
            )),
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
                    return_type: String::from("string"),
                })]
            })
        )
    }

    #[test]
    fn a_function_with_const_passes() {
        assert_eq!(
            parse(String::from(
                "
fn say_hello(name: string): string {
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
                    return_type: String::from("string"),
                })]
            })
        )
    }

    #[test]
    fn a_function_with_global_const_passes() {
        assert_eq!(
            parse(String::from(
                "
fn say_hello(name: string): string {
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
                    return_type: String::from("string"),
                })]
            })
        )
    }

    #[test]
    fn a_function_with_local_addition_passes() {
        assert_eq!(
            parse(String::from(
                "
fn say_hello(name: string): string {
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
                    return_type: String::from("string"),
                })]
            })
        )
    }

    #[test]
    fn a_function_with_local_numeric_addition_passes() {
        assert_eq!(
            parse(String::from(
                "
fn say_hello(): void {
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
                    return_type: String::from("void"),
                })]
            })
        )
    }

    #[test]
    fn a_function_with_nothing_errors() {
        assert_eq!(
            parse(String::from("fn")),
            Err(String::from(
                "Expected a function name but got nothing at line 1, index 2"
            ))
        )
    }

    #[test]
    fn a_function_without_a_name_errors() {
        assert_eq!(
            parse(String::from("fn () {}")),
            Err(String::from(
                "Expected a function name but got ( at line 1, index 2"
            ))
        )
    }

    #[test]
    fn a_function_without_a_parens_errors() {
        assert_eq!(
            parse(String::from("fn {}")),
            Err(String::from(
                "Expected a function name but got { at line 1, index 2"
            ))
        )
    }

    #[test]
    fn a_function_without_a_parens_after_a_name_errors() {
        assert_eq!(
            parse(String::from("fn say_hello {}")),
            Err(String::from(
                "Expected parens but got { at line 1, index 13"
            ))
        )
    }

    #[test]
    fn a_function_with_a_param_without_type_errors() {
        assert_eq!(
            parse(String::from("fn say_hello (name) {}")),
            Err(String::from(
                "Failed to find type for param name at line 1, index 13"
            ))
        )
    }

    #[test]
    fn a_function_without_return_type_with_colon_errors() {
        assert_eq!(
            parse(String::from("fn say_hello (name: string): {}")),
            Err(String::from(
                "Expected return type name, but got { at line 1, index 29"
            ))
        )
    }

    #[test]
    fn a_function_without_return_type_errors() {
        assert_eq!(
            parse(String::from("fn say_hello (name: string) {}")),
            Err(String::from(
                "Failed parsing function signature - expected return type, got { at line 1, index 28"
            ))
        )
    }

    #[test]
    fn a_function_with_return_type_but_missing_open_bracket_errors() {
        assert_eq!(
            parse(String::from("fn say_hello (name: string): string }")),
            Err(String::from("Expected { but got } at line 1, index 36"))
        )
    }

    #[test]
    fn a_function_with_return_type_but_missing_everything_errors() {
        assert_eq!(
            parse(String::from("fn say_hello (name: string): string")),
            Err(String::from("Expected { but got nothing"))
        )
    }

    #[test]
    fn an_export_without_an_external_name_errors() {
        assert_eq!(
            parse(String::from("export")),
            Err(String::from("Expected external name in export"))
        )
    }

    #[test]
    fn an_export_without_an_external_name_but_a_bracket_errors() {
        assert_eq!(
            parse(String::from("export {")),
            Err(String::from(
                "Expected external name in export, got { at line 1, index 7"
            ))
        )
    }

    #[test]
    fn an_export_without_an_internal_name_errors() {
        assert_eq!(
            parse(String::from("export sayHello")),
            Err(String::from("Expected function name in export"))
        )
    }

    #[test]
    fn an_export_without_an_internal_name_but_a_bracket_errors() {
        assert_eq!(
            parse(String::from("export sayHello {")),
            Err(String::from(
                "Expected function name in export, got { at line 1, index 16"
            ))
        )
    }

    #[test]
    fn a_local_without_a_type_errors() {
        assert_eq!(
            parse(String::from(
                "fn sayHello(): string {
    local var = 5;
}"
            )),
            Err(String::from("Expected : but got = at line 2, index 14"))
        )
    }

    #[test]
    fn a_local_without_a_assign_errors() {
        assert_eq!(
            parse(String::from(
                "fn sayHello(): string {
    local var: i32;
}"
            )),
            Err(String::from("Expected = but got nothing"))
        )
    }
}
