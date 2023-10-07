pub mod web_assembly {
    use crate::blocks::blocks::{Block, Function};

    pub fn generate(program: crate::parser::parser::Program) -> String {
        let blocks: Vec<String> = program.blocks.into_iter().map(generate_block).collect();

        blocks.join("\n\n")
    }

    fn generate_function(function: Function) -> String {
        format!("(func {})", function.name)
    }

    fn generate_block(block: Block) -> String {
        match block {
            Block::FunctionBlock(function) => generate_function(function),
        }
    }
}
