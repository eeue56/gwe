pub mod gwe {
    use crate::{
        blocks::blocks::{Block, Function, Param},
        expressions::Expression,
    };

    pub fn indent(body: String) -> String {
        body.split("\n")
            .map(|line| {
                if line.len() == 0 {
                    String::from("")
                } else {
                    format!("    {}\n", line)
                }
            })
            .collect()
    }

    pub fn generate(program: crate::parser::parser::Program) -> String {
        let blocks: Vec<String> = program.blocks.into_iter().map(generate_block).collect();

        blocks.join("\n\n")
    }

    fn generate_param(param: Param) -> String {
        format!("{}: {}", param.name, param.type_name)
    }

    fn generate_expression(expression: Expression) -> String {
        match expression {
            Expression::Addition { left, right } => {
                let generated_left = generate_expression(*left);
                let generated_right = generate_expression(*right);

                format!("{} + {}", generated_left, generated_right)
            }
            Expression::GlobalAssign {
                name,
                type_name,
                expression,
            } => {
                format!(
                    "global {}: {} = {}",
                    name,
                    type_name,
                    generate_expression(*expression)
                )
            }
            Expression::LocalAssign {
                name,
                type_name,
                expression,
            } => {
                format!(
                    "local {}: {} = {}",
                    name,
                    type_name,
                    generate_expression(*expression)
                )
            }
            Expression::Number { value } => value,
            Expression::Return { expression } => {
                format!("return {}", generate_expression(*expression))
            }
            Expression::Variable { body } => body,
            Expression::String { body } => format!("\"{}\"", body),
        }
    }

    fn generate_function(function: Function) -> String {
        let params: Vec<String> = function.params.into_iter().map(generate_param).collect();
        if function.expressions.len() == 0 {
            format!(
                "fn {}({}): {} {{\n}}",
                function.name,
                params.join(", "),
                function.return_type
            )
        } else {
            let body = indent(
                function
                    .expressions
                    .into_iter()
                    .map(generate_expression)
                    .map(|line| format!("{};\n", line))
                    .collect::<Vec<String>>()
                    .join(""),
            );

            format!(
                "fn {}({}): {} {{\n{}}}",
                function.name,
                params.join(", "),
                function.return_type,
                body
            )
        }
    }

    fn generate_block(block: Block) -> String {
        match block {
            Block::FunctionBlock(function) => generate_function(function),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::parser::parse;

    use super::gwe::*;

    #[test]
    fn empty_function() {
        let input = String::from(
            "fn hello_world(): void {
}",
        );

        match parse(input.clone()) {
            Err(err) => panic!("{}", err),
            Ok(program) => {
                assert_eq!(generate(program), input);
                ()
            }
        }
    }

    #[test]
    fn empty_with_an_arg_function() {
        let input = String::from(
            "fn hello_world(name: string): void {
}",
        );

        match parse(input.clone()) {
            Err(err) => panic!("{}", err),
            Ok(program) => {
                assert_eq!(generate(program), input);
                ()
            }
        }
    }

    #[test]
    fn empty_with_several_args_function() {
        let input = String::from(
            "fn hello_world(name: string, age: i32): void {
}",
        );

        match parse(input.clone()) {
            Err(err) => panic!("{}", err),
            Ok(program) => {
                assert_eq!(generate(program), input);
                ()
            }
        }
    }

    #[test]
    fn return_function() {
        let input = String::from(
            "fn hello_world(name: string): string {
    return name;
}",
        );

        match parse(input.clone()) {
            Err(err) => panic!("{}", err),
            Ok(program) => {
                assert_eq!(generate(program), input);
                ()
            }
        }
    }

    #[test]
    fn local_var_and_addition_function() {
        let input = String::from(
            "fn hello_world(name: string): string {
    local message: string = \"Hello \" + name;
    return message;
}",
        );

        match parse(input.clone()) {
            Err(err) => panic!("{}", err),
            Ok(program) => {
                assert_eq!(generate(program), input);
                ()
            }
        }
    }

    #[test]
    fn global_var_and_addition_function() {
        let input = String::from(
            "fn hello_world(): void {
    global num: f32 = 123 + 3.14;
}",
        );

        match parse(input.clone()) {
            Err(err) => panic!("{}", err),
            Ok(program) => {
                assert_eq!(generate(program), input);
                ()
            }
        }
    }
}
