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

    fn emit_pop(&mut self) {
        self.emit("add rsp, 8");
    }

    fn emit_stmt(&mut self, stmt: Stmt) {
        // dbg!(&stmt);
        match stmt.stmt {
            StmtType::Expr(expr) => {
                self.emit_expr(expr);
                self.emit_pop();
            }
            StmtType::Println(expr) => {
                self.emit_expr(expr);
                self.emit_pop();
                self.emit("call print_int");
            }
            // StmtType::VarDecl {name, value, ty } => {
            //     // self.emit_expr(expr);
            //     self.emit_pop();
            // }
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
            ExprType::Binary { left, op, right } => {
                self.emit_expr(*left);
                self.emit_expr(*right);

                self.emit_opcode(op);
            }
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
        self.emit("section .bss");
        self.emit("buffer resb 32");

        self.emit("section .text");
        self.emit("global _start");

        self.emit_print_int();

        self.emit("_start:");
    }

    fn emit_print_int(&mut self) {
        self.emit("print_int:");
        self.emit("mov rcx, buffer + 31");
        self.emit("mov byte [rcx], 10");
        self.emit("dec rcx");
        self.emit("mov rbx, 10");

        self.emit(".convert:");
        self.emit("xor rdx, rdx");
        self.emit("div rbx");
        self.emit("add dl, '0'");
        self.emit("mov [rcx], dl");
        self.emit("dec rcx");
        self.emit("test rax, rax");
        self.emit("jnz .convert");

        self.emit("inc rcx");

        self.emit("mov rax, 1");
        self.emit("mov rdi, 1");
        self.emit("mov rsi, rcx");
        self.emit("mov rdx, buffer + 32");
        self.emit("sub rdx, rcx");
        self.emit("syscall");
        self.emit("ret");
    }

    fn emit_postlude(&mut self) {
        self.emit("mov rax, 60"); // exit syscall
        self.emit("xor rdi, rdi");
        self.emit("syscall");
    }

fn emit_opcode(&mut self, op: crate::parse_types::BinaryOp) {
    use crate::parse_types::BinaryOp;
    match op {
        BinaryOp::Add => {
            self.emit("pop rbx");
            self.emit("pop rax");
            self.emit("add rax, rbx");
            self.emit("push rax");
        }
        BinaryOp::Sub => {
            self.emit("pop rbx");
            self.emit("pop rax");
            self.emit("sub rax, rbx");
            self.emit("push rax");
        }
        BinaryOp::Mul => {
            self.emit("pop rbx");
            self.emit("pop rax");
            self.emit("mul rax, rbx");
            self.emit("push rax");
        }
        BinaryOp::Div => {
            self.emit("pop rbx");
            self.emit("pop rax");
            self.emit("div rbx");
            self.emit("push rax");
        }
        BinaryOp::Equal => todo!(),
        BinaryOp::NotEqual => todo!(),
        BinaryOp::Less => todo!(),
        BinaryOp::LessEqual => todo!(),
        BinaryOp::Greater => todo!(),
        BinaryOp::GreaterEqual => todo!(),
        BinaryOp::And => todo!(),
        BinaryOp::Or => todo!(),
    }
}
}

