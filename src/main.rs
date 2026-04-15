use analysis::Analyser;
use error::{PRINT_PARSE_TREE, PRINT_TOKENS};
use scanner::Scanner;

use colored::Colorize;

mod analysis;
mod analysis_types;
mod codegen;
mod error;
mod expression;
mod func_compiler;
mod parse_types;
mod parser;
mod scanner;
mod statement;
mod token;
mod value;
mod genn;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let source = if args.len() <= 1 {
        let msg = "Could not find file.crs. The file should be in the same directory as either the executable file or Cargo.toml.";
        std::fs::read_to_string("file.crs").expect(msg)
    } else {
        let msg = format!("Could not find file '{}'.", args[1]);
        std::fs::read_to_string(&args[1]).expect(&msg)
    };

    let scanner = Scanner::new(source);
    let tokens = match scanner.scan_tokens() {
        Ok(tokens) => tokens,
        Err(_) => {
            println!(
                "{}",
                "Scan error(s) detected, terminating program.".purple()
            );
            return;
        }
    };

    if PRINT_TOKENS {
        for token in &tokens {
            println!("{:?} type: {:?}", token, token.ty as u8);
        }
        println!();
    }

    let mut statements = match parser::Parser::compile(tokens) {
        Some(statements) => statements,
        None => {
            println!(
                "{}",
                "Parse error(s) detected, terminating program.".purple()
            );
            return;
        }
    };
    if PRINT_PARSE_TREE {
        dbg!(&statements);
    }

    let entities = match Analyser::analyse_stmts(&mut statements) {
        Some(func_data) => func_data,
        None => return,
    };

    // let mut codegen = Codegen::new();
    match genn::CodeGen::compile(statements) {
        Ok(_) => (),
        Err(e) => println!("{}", e),
    }
    // codegen.compile();
    // codegen.compile_statements(statements);
    // codegen.write_to_file("out.asm");
}
