use crate::{
    expression::{Expr, ExprType},
    statement::{Stmt, StmtType},
    token::Literal,
};

pub struct Codegen {
    pub output: String,
}

impl Codegen {
    pub fn new() -> Self {
        Self {
            output: String::new(),
        }
    }

    pub fn compile_statements(&mut self, stmts: Vec<Stmt>) {
        self.emit_prelude();
        for stmt in stmts {
            self.emit_stmt(stmt);
        }
        self.emit_postlude();
        self.print_output();
    }

    pub fn write_to_file(&self, path: &str) {
        std::fs::write(path, &self.output).unwrap();
    }


    fn emit_stmt(&mut self, stmt: Stmt) {
        match stmt.stmt {
            StmtType::Expr(expr) => self.emit_expr(expr),
            _ => todo!(),
        }
    }
    fn emit_expr(&mut self, expr: Expr) {
        // dbg!(&expr.expr);
        match expr.expr {
            ExprType::Lit(lit) => match lit {
                Literal::I64(num) => {
                    self.emit(&format!("mov rax, {}", num));
                    self.emit("push rax");
                }
                _ => todo!(),
            },
            _ => todo!(),
        }
    }

    fn print_output(&self) {
        println!("{}", self.output);
    }

    fn emit(&mut self, line: &str) {
        self.output.push_str(line);
        self.output.push('\n');
    }

    fn emit_prelude(&mut self) {
        self.emit("section .text");
        self.emit("global _start");
        self.emit("_start:");
    }

    fn emit_postlude(&mut self) {
        self.emit("mov rax, 60"); // exit syscall
        self.emit("xor rdi, rdi");
        self.emit("syscall");
    }
}
