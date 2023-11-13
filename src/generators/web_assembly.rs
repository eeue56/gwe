use std::vec;

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
                format!("  {}\n", line)
            }
        })
        .collect()
}

pub fn generate(program: crate::parser::Program) -> String {
    let blocks: Vec<String> = program
        .blocks
        .clone()
        .into_iter()
        .map(generate_block)
        .collect();
    let globals = program
        .blocks
        .clone()
        .iter()
        .filter_map(|block| match block {
            Block::Function(function) => match define_globals(function.expressions.clone()) {
                str if str.is_empty() => None,
                str if !str.is_empty() => Some(str),
                _ => None,
            },
            _ => None,
        })
        .collect::<Vec<String>>();

    let globals_and_blocks = [globals, blocks].concat();

    format!(
        "(module
{})",
        indent(globals_and_blocks.join("\n\n"))
    )
}

fn define_globals(expressions: Vec<Expression>) -> String {
    expressions
        .into_iter()
        .filter_map(|expression| match expression {
            Expression::GlobalAssign {
                name,
                type_name,
                expression: _,
            } => Some((name, type_name)),
            _ => None,
        })
        .map(|(name, type_name)| format!("(global ${} (mut {}))", name, type_name))
        .collect::<Vec<String>>()
        .join("\n")
}

fn define_locals(expressions: Vec<Expression>) -> String {
    expressions
        .into_iter()
        .filter_map(|expression| match expression {
            Expression::LocalAssign {
                name,
                type_name,
                expression: _,
            } => {
                if type_name == *"string" {
                    None
                } else {
                    Some((name, type_name))
                }
            }
            Expression::ForStatement {
                initial_value,
                incrementor: _,
                break_condition: _,
                body: _,
            } => match *initial_value {
                Expression::LocalAssign {
                    name,
                    type_name,
                    expression: _,
                } => {
                    if type_name == *"string" {
                        None
                    } else {
                        Some((name, type_name))
                    }
                }
                _ => None,
            },
            _ => None,
        })
        .map(|(name, type_name)| format!("(local ${} {})", name, type_name))
        .collect::<Vec<String>>()
        .join("\n")
}

fn generate_param(param: Param) -> String {
    format!("(param ${} {})", param.name, param.type_name)
}

fn extract_strings(expressions: Vec<Expression>) -> (Option<String>, Vec<Expression>) {
    let mut strings: Vec<(i32, String)> = vec![];
    let mut offset: i32 = 0;

    let new_expressions = expressions
        .iter()
        .map(|exp| match exp {
            Expression::LocalAssign {
                name: _,
                type_name,
                expression,
            } => {
                if type_name == &String::from("string") {
                    let length: i32 = match *expression.clone() {
                        Expression::String { body } => {
                            strings.push((offset, body.clone()));
                            body.len().try_into().unwrap()
                        }
                        _ => 0,
                    };

                    offset += length;

                    Expression::MemoryReference {
                        offset: offset - length,
                        length,
                    }
                } else {
                    exp.clone()
                }
            }
            _ => exp.clone(),
        })
        .collect::<Vec<Expression>>();

    let output = if strings.is_empty() {
        None
    } else {
        let datas: String = strings
            .iter()
            .map(|(offset, string)| format!("(data (i32.const {}) \"{}\")", offset, string))
            .collect::<Vec<String>>()
            .join("\n");
        Some(format!("{}\n", datas))
    };

    (output, new_expressions)
}

fn generate_expression(expression: Expression) -> String {
    match expression {
        Expression::Addition { left, right } => {
            let generated_left = generate_expression(*left);
            let generated_right = generate_expression(*right);

            format!("(f32.add {} {})", generated_left, generated_right)
        }
        Expression::GlobalAssign {
            name,
            type_name: _,
            expression,
        } => {
            format!(
                "(global.set ${} {})",
                name,
                generate_expression(*expression)
            )
        }
        Expression::LocalAssign {
            name,
            type_name: _,
            expression,
        } => {
            format!("(local.set ${} {})", name, generate_expression(*expression))
        }
        Expression::Number { value, type_name } => format!("({}.const {})", type_name, value),
        Expression::Return { expression } => generate_expression(*expression),
        Expression::Variable { body, type_name: _ } => format!("(local.get ${})", body),
        Expression::String { body } => format!("\"{}\"", body),
        Expression::FunctionCall { name, args } => {
            let params = args
                .iter()
                .map(|e| generate_expression(e.clone()))
                .collect::<Vec<String>>()
                .join("\n");
            format!("{}\n(call ${})", params, name)
        }
        Expression::MemoryReference { offset, length } => {
            format!("(i32.const {})\n(i32.const {})", offset, length)
        }
        Expression::IfStatement {
            predicate,
            success,
            fail,
        } => {
            format!(
                "(if
  {}
  (then
{}
  )
  (else
{}
  )
)",
                generate_expression(*predicate),
                indent(indent(generate_expression(*success))),
                indent(indent(generate_expression(*fail)))
            )
        }
        Expression::Boolean { value } => {
            if value {
                "(i32.const 0)".to_string()
            } else {
                "(i32.const 1)".to_string()
            }
        }
        Expression::ForStatement {
            initial_value,
            break_condition,
            incrementor,
            body,
        } => {
            let body_expressions = body
                .iter()
                .map(|expression| generate_expression(expression.clone()))
                .collect::<Vec<String>>()
                .join("\n");

            let variable_name = match *initial_value.clone() {
                Expression::LocalAssign {
                    name,
                    type_name: _,
                    expression: _,
                } => name,
                _ => return String::from(""),
            };

            let type_name = match *initial_value.clone() {
                Expression::LocalAssign {
                    name: _,
                    type_name,
                    expression: _,
                } => type_name,
                _ => return String::from("i32"),
            };

            format!(
                "{}
(loop $loop
{}
  (local.get ${variable_name})
  {incrementor}
  ({type_name}.add)
  (local.set ${variable_name})

  (local.get ${variable_name})
  {break_condition}
  ({type_name}.lt_s)
  (br_if $loop)
)",
                generate_expression(*initial_value),
                indent(body_expressions),
                incrementor = generate_expression(*incrementor),
                variable_name = variable_name,
                break_condition = generate_expression(*break_condition),
                type_name = type_name
            )
        }
    }
}

fn generate_function(function: Function) -> String {
    let params: String = if function.params.is_empty() {
        String::from("")
    } else {
        String::from(" ")
            + &function
                .params
                .clone()
                .into_iter()
                .map(generate_param)
                .collect::<Vec<String>>()
                .join(" ")
    };

    let return_value: String = if function.return_type == *"void" {
        String::from("")
    } else {
        format!(" (result {})", function.return_type)
    };

    let locals = define_locals(function.expressions.clone());

    let (memory, extracted_expressions) = extract_strings(function.expressions);

    let expressions = extracted_expressions
        .into_iter()
        .map(generate_expression)
        .map(|line| format!("{}\n", line))
        .collect::<Vec<String>>()
        .join("");

    let definitions = if locals.is_empty() {
        indent(expressions)
    } else {
        indent(format!("{}\n{}", locals, expressions))
    };

    let maybe_memory = match memory {
        Some(inner) => inner,
        None => String::from(""),
    };

    format!(
        "{}(func ${}{}{}
{})",
        maybe_memory, function.name, params, return_value, definitions
    )
}

fn generate_export(export: Export) -> String {
    format!(
        "(export \"{}\" (func ${}))",
        export.external_name, export.function_name
    )
}

fn generate_import_function(import: ImportFunction) -> String {
    let params: Vec<String> = import
        .params
        .into_iter()
        .map(|param| param.type_name)
        .collect();
    let external_name = import
        .external_name
        .iter()
        .map(|n| format!("\"{}\"", n))
        .collect::<Vec<String>>()
        .join(" ");
    format!(
        "(import {} (func ${} (param {})))",
        external_name,
        import.name,
        params.join(" "),
    )
}

fn generate_import_memory(import: ImportMemory) -> String {
    let external_name = import
        .external_name
        .iter()
        .map(|n| format!("\"{}\"", n))
        .collect::<Vec<String>>()
        .join(" ");
    format!("(import {} (memory {}))", external_name, import.size)
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

        let output = String::from(
            "(module
  (func $hello_world
  )
)",
        );

        match parse(input.clone()) {
            Err(err) => panic!("{}", err),
            Ok(program) => {
                assert_eq!(generate(program), output);
            }
        }
    }

    #[test]
    fn empty_with_an_arg_function() {
        let input = String::from(
            "fn hello_world(name: string): void {
}",
        );

        let output = String::from(
            "(module
  (func $hello_world (param $name string)
  )
)",
        );

        match parse(input.clone()) {
            Err(err) => panic!("{}", err),
            Ok(program) => {
                assert_eq!(generate(program), output);
            }
        }
    }

    #[test]
    fn empty_with_several_args_function() {
        let input = String::from(
            "fn hello_world(name: string, age: i32): void {
}",
        );
        let output = String::from(
            "(module
  (func $hello_world (param $name string) (param $age i32)
  )
)",
        );

        match parse(input.clone()) {
            Err(err) => panic!("{}", err),
            Ok(program) => {
                assert_eq!(generate(program), output);
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
        let output = String::from(
            "(module
  (func $hello_world (param $name string) (result string)
    (local.get $name)
  )
)",
        );

        match parse(input.clone()) {
            Err(err) => panic!("{}", err),
            Ok(program) => {
                assert_eq!(generate(program), output);
            }
        }
    }

    #[test]
    fn local_var_and_addition_function() {
        let input = String::from(
            "fn hello_world(name: i32): i32 {
    local message: i32 = name;
    return message;
}",
        );
        let output = String::from(
            "(module
  (func $hello_world (param $name i32) (result i32)
    (local $message i32)
    (local.set $message (local.get $name))
    (local.get $message)
  )
)",
        );

        match parse(input.clone()) {
            Err(err) => panic!("{}", err),
            Ok(program) => {
                assert_eq!(generate(program), output);
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
        let output = String::from(
            "(module
  (global $num (mut f32))
  (func $hello_world
    (global.set $num (f32.add (f32.const 123) (f32.const 3.14)))
  )
)",
        );

        match parse(input.clone()) {
            Err(err) => panic!("{}", err),
            Ok(program) => {
                assert_eq!(generate(program), output);
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
        let output = String::from(
            "(module
  (func $hello_world (result f32)
    (f32.const 3.14)
  )
  (export \"helloWorld\" (func $hello_world))
)",
        );

        match parse(input.clone()) {
            Err(err) => panic!("{}", err),
            Ok(program) => {
                assert_eq!(generate(program), output);
            }
        }
    }

    #[test]
    fn import_function() {
        let input = String::from("import fn log(number: i32) console.log");
        let output = String::from(
            "(module
  (import \"console\" \"log\" (func $log (param i32)))
)",
        );

        match parse(input.clone()) {
            Err(err) => panic!("{}", err),
            Ok(program) => {
                assert_eq!(generate(program), output);
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
        let output = String::from(
            "(module
  (import \"console\" \"log\" (func $log (param i32)))
  (func $main
    (f32.const 3.14)
    (call $log)
  )
  (export \"main\" (func $main))
)",
        );

        match parse(input.clone()) {
            Err(err) => panic!("{}", err),
            Ok(program) => {
                assert_eq!(generate(program), output);
            }
        }
    }

    #[test]
    fn string_function() {
        let input = String::from(
            "import fn log(offset: i32, length: i32) console.log
import memory 1 js.mem

fn main(): void {
    local message: string = \"Hello world\";
    log();
}

export main main",
        );
        let output = String::from(
            "(module
  (import \"console\" \"log\" (func $log (param i32 i32)))
  (import \"js\" \"mem\" (memory 1))
  (data (i32.const 0) \"Hello world\")
  (func $main
    (i32.const 0)
    (i32.const 11)
    (call $log)
  )
  (export \"main\" (func $main))
)",
        );

        match parse(input.clone()) {
            Err(err) => panic!("{}", err),
            Ok(program) => {
                assert_eq!(
                    generate(program.clone()),
                    output,
                    "Generated:\n{}\n\n\n========\nExpected:\n{}",
                    generate(program.clone()),
                    output
                );
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
        let output = String::from(
            "(module
  (import \"js\" \"mem\" (memory 1))
  (func $main
    (f32.const 3.14)
    (call $log)
  )
  (export \"main\" (func $main))
)",
        );

        match parse(input.clone()) {
            Err(err) => panic!("{}", err),
            Ok(program) => {
                assert_eq!(
                    generate(program.clone()),
                    output,
                    "Generated:\n{}\n\n\n========\nExpected:\n{}",
                    generate(program.clone()),
                    output
                );
            }
        }
    }

    #[test]
    fn if_statement() {
        let input = String::from(
            "import memory 1 js.mem

fn main(): void {
    if (0) { log(3.14) } { log(42) };
}

export main main",
        );
        let output = String::from(
            "(module
  (import \"js\" \"mem\" (memory 1))
  (func $main
    (if
      (f32.const 0)
      (then
        (f32.const 3.14)
        (call $log)
      )
      (else
        (f32.const 42)
        (call $log)
      )
    )
  )
  (export \"main\" (func $main))
)",
        );

        match parse(input.clone()) {
            Err(err) => panic!("{}", err),
            Ok(program) => {
                assert_eq!(
                    generate(program.clone()),
                    output,
                    "Generated:\n{}\n\n\n========\nExpected:\n{}",
                    generate(program.clone()),
                    output
                );
            }
        }
    }

    #[test]
    fn boolean() {
        let input = String::from(
            "import memory 1 js.mem

fn main(): void {
    if (true) { log(true) } { log(false) };
}

export main main",
        );

        let output = String::from(
            "(module
  (import \"js\" \"mem\" (memory 1))
  (func $main
    (if
      (i32.const 0)
      (then
        (i32.const 0)
        (call $log)
      )
      (else
        (i32.const 1)
        (call $log)
      )
    )
  )
  (export \"main\" (func $main))
)",
        );

        match parse(input.clone()) {
            Err(err) => panic!("{}", err),
            Ok(program) => {
                assert_eq!(
                    generate(program.clone()),
                    output,
                    "Generated:\n{}\n\n\n========\nExpected:\n{}",
                    generate(program.clone()),
                    output
                );
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

        let output = String::from(
            "(module
  (import \"console\" \"log\" (func $log (param i32)))
  (func $main
    (local $x i32)
    (local.set $x (i32.const 0))
    (loop $loop
      (local.get $x)
      (call $log)
      (local.get $x)
      (i32.const 1)
      (i32.add)
      (local.set $x)
      (local.get $x)
      (i32.const 10)
      (i32.lt_s)
      (br_if $loop)
    )
  )
  (export \"main\" (func $main))
)",
        );

        match parse(input.clone()) {
            Err(err) => panic!("{}", err),
            Ok(program) => {
                assert_eq!(
                    generate(program.clone()),
                    output,
                    "Generated:\n{}\n\n\n========\nExpected:\n{}",
                    generate(program.clone()),
                    output
                );
            }
        }
    }
}
