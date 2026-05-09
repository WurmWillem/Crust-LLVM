use colored::Colorize;

use crate::value::ValueType;

pub const PRINT_TOKENS: bool = false;
pub const PRINT_PARSE_TREE: bool = false;

pub fn print_error(line: u32, msg: &str) {
    let l = "[line ".blue();
    // let line = line.to_string().magenta();
    let closing_bracket = "]".blue();
    let i = " Error: ".bright_red();
    let msg = msg.yellow();
    println!("{l}{line}{closing_bracket}{i}{msg}");
}

#[derive(Debug)]
pub struct ParseErr {
    pub msg: String,
    pub line: u32,
}
impl ParseErr {
    pub fn new(line: u32, msg: &str) -> Self {
        Self {
            msg: msg.to_string(),
            line,
        }
    }
}
