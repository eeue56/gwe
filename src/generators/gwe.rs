use crate::{
    blocks::{Block, Export, Function, ImportFunction, ImportMemory, Param},
    expressions::Expression,
};

pub fn indent(body: String) -> String {
    body.split('\n')
        .map(|line| {
            if line.is_empty() {
                String::from("")
            } else {
                format!("    {}\n", line)
            }
        })
        .collect()
}

pub fn generate(program: crate::parser::Program) -> String {
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
        Expression::Number {
            value,
            type_name: _,
        } => value,
        Expression::Return { expression } => {
            format!("return {}", generate_expression(*expression))
        }
        Expression::Variable { body, type_name: _ } => body,
        Expression::String { body } => format!("\"{}\"", body),
        Expression::FunctionCall { name, args } => {
            let params = args
                .iter()
                .map(|e| generate_expression(e.clone()))
                .collect::<Vec<String>>()
                .join(", ");

            format!("{}({})", name, params)
        }
        Expression::MemoryReference {
            offset: _,
            length: _,
        } => String::from(""),
        Expression::IfStatement {
            predicate,
            success,
            fail,
        } => {
            let success_expressions = success
                .iter()
                .map(|expression| format!("{};", generate_expression(expression.clone())))
                .collect::<Vec<String>>()
                .join("\n");

            let fail_expressions = fail
                .iter()
                .map(|expression| format!("{};", generate_expression(expression.clone())))
                .collect::<Vec<String>>()
                .join("\n");

            format!(
                "if ({}) {{
{}
}} else {{
{}
}}",
                generate_expression(*predicate),
                indent(success_expressions),
                indent(fail_expressions)
            )
        }
        Expression::Boolean { value } => format!("{}", value),
        Expression::ForStatement {
            initial_value,
            break_condition,
            incrementor,
            body,
        } => {
            let body_expressions = body
                .iter()
                .map(|expression| format!("{};", generate_expression(expression.clone())))
                .collect::<Vec<String>>()
                .join("\n");
            format!(
                "for ({}, {}, {}) {{
{}
}}",
                generate_expression(*initial_value),
                generate_expression(*break_condition),
                generate_expression(*incrementor),
                indent(body_expressions)
            )
        }
    }
}

fn generate_function(function: Function) -> String {
    let params: Vec<String> = function.params.into_iter().map(generate_param).collect();
    if function.expressions.is_empty() {
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

fn generate_export(export: Export) -> String {
    format!("export {} {}", export.external_name, export.function_name)
}

fn generate_import_function(import: ImportFunction) -> String {
    let params: Vec<String> = import.params.into_iter().map(generate_param).collect();
    let external_name = import.external_name.join(".");
    format!(
        "import fn {}({}) {}",
        import.name,
        params.join(", "),
        external_name
    )
}

fn generate_import_memory(import: ImportMemory) -> String {
    let external_name = import.external_name.join(".");
    format!("import memory {} {}", import.size, external_name)
}

fn generate_block(block: Block) -> String {
    match block {
        Block::Function(function) => generate_function(function),
        Block::Export(export) => generate_export(export),
        Block::ImportFunction(import) => generate_import_function(import),
        Block::ImportMemory(import) => generate_import_memory(import),
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::parse;

    use super::*;

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
            }
        }
    }

    #[test]
    fn export_function() {
        let input = String::from(
            "fn hello_world(): f32 {
    return 3.14;
}

export helloWorld hello_world",
        );

        match parse(input.clone()) {
            Err(err) => panic!("{}", err),
            Ok(program) => {
                assert_eq!(generate(program), input);
            }
        }
    }

    #[test]
    fn import_function() {
        let input = String::from("import fn log(number: i32) console.log");

        match parse(input.clone()) {
            Err(err) => panic!("{}", err),
            Ok(program) => {
                assert_eq!(generate(program), input);
            }
        }
    }

    #[test]
    fn call_function() {
        let input = String::from(
            "import fn log(number: i32) console.log

fn main(): void {
    log(3.14);
}

export main main",
        );

        match parse(input.clone()) {
            Err(err) => panic!("{}", err),
            Ok(program) => {
                assert_eq!(generate(program), input);
            }
        }
    }

    #[test]
    fn import_memory() {
        let input = String::from(
            "import memory 1 js.mem

fn main(): void {
    log(3.14);
}

export main main",
        );

        match parse(input.clone()) {
            Err(err) => panic!("{}", err),
            Ok(program) => {
                assert_eq!(generate(program), input);
            }
        }
    }

    #[test]
    fn if_statement() {
        let input = String::from(
            "import memory 1 js.mem

fn main(): void {
    if (0) {
        log(3.14);
    } else {
        log(42);
    };
}

export main main",
        );

        match parse(input.clone()) {
            Err(err) => panic!("{}", err),
            Ok(program) => {
                assert_eq!(generate(program), input);
            }
        }
    }

    #[test]
    fn boolean() {
        let input = String::from(
            "import memory 1 js.mem

fn main(): void {
    if (true) {
        log(true);
    } else {
        log(false);
    };
}

export main main",
        );

        match parse(input.clone()) {
            Err(err) => panic!("{}", err),
            Ok(program) => {
                assert_eq!(generate(program), input);
            }
        }
    }

    #[test]
    fn for_loop() {
        let input = String::from(
            "import fn log(number: i32) console.log

fn main(): void {
    for (local x: i32 = 0, 10, 1) {
        log(x);
    };
}

export main main",
        );

        match parse(input.clone()) {
            Err(err) => panic!("{}", err),
            Ok(program) => {
                assert_eq!(generate(program), input);
            }
        }
    }
}
