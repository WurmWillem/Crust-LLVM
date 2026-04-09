use crate::{
    expression::{Expr, ExprType},
    func_compiler::FuncCompiler,
    statement::{Stmt, StmtType},
    token::{Literal, TokenType},
};

pub struct Codegen {
    output: String,
    funcs: Vec<FuncCompiler>,
}

impl Codegen {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            funcs: Vec::new(),
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
            StmtType::VarDecl { name, value, ty: _ } => {
                self.funcs.last_mut().unwrap().add_local(name.to_string());
                self.emit_expr(value);
                self.emit("pop rax");

                let var_index = self.funcs.last().unwrap().get_local_count() * 8;
                let emit_var = format!("mov qword [rbp-{}], rax", var_index);
                self.emit(&emit_var);
                // self.emit_pop();
            }
            StmtType::Func {
                name,
                parameters,
                body,
                return_ty,
                use_self,
            } => {
                self.funcs.push(FuncCompiler::new("main".to_string()));

                for stmt in body {
                    self.emit_stmt(stmt);
                }
                let (output, local_amt) = self.funcs.pop().unwrap().end();

                let mut func_start = String::from("push rbp\n");
                func_start.push_str("mov rbp, rsp\n");
                func_start.push_str(&format!("sub rsp, {}\n", local_amt * 8));
                self.output.push_str(&func_start);

                self.output.push_str(&output);
            }
            _ => todo!(),
        }
    }
    fn emit_expr(&mut self, expr: Expr) {
        dbg!(&expr.expr);
        match expr.expr {
            ExprType::Identifier(name) => {
                let index = self.funcs.last().unwrap().resolve_local(name);
                let index = index.unwrap() as i16 + 1;
                self.emit(&format!("mov rax, [rbp-{}]", index * 8));
                self.emit("push rax");
            }
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

                self.emit_binary_op(op);
            }
            ExprType::Unary { prefix, value } => {
                self.emit_expr(*value);
                match prefix {
                    TokenType::Minus => {
                        self.emit("pop rax");
                        self.emit("neg rax");
                        self.emit("push rax");
                    }
                    _ => todo!(),
                }
            }
            _ => todo!(),
        }
    }

    fn print_output(&self) {
        println!("{}", self.output);
    }

    fn emit(&mut self, line: &str) {
        self.funcs.last_mut().unwrap().emit(line);
    }

    fn emit_binary_op(&mut self, op: crate::parse_types::BinaryOp) {
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

    fn direct_emit(&mut self, line: &str) {
        self.output.push_str(line);
        self.output.push('\n');
    }

    fn emit_prelude(&mut self) {
        self.direct_emit("section .bss");
        self.direct_emit("buffer resb 32");

        self.direct_emit("section .text");
        self.direct_emit("global _start");

        self.emit_print_int();

        self.direct_emit("_start:");
    }

    fn emit_print_int(&mut self) {
        self.direct_emit("print_int:");
        self.direct_emit("push rbx");
        self.direct_emit("mov rcx, buffer + 31");
        self.direct_emit("mov byte [rcx], 10");
        self.direct_emit("dec rcx");
        self.direct_emit("mov rbx, 10");

        self.direct_emit("test rax, rax");
        self.direct_emit("jz  .zero_or_positive");
        self.direct_emit("js  .negative");
        self.direct_emit("jmp .positive");

        self.direct_emit(".zero_or_positive:");
        self.direct_emit("xor r8, r8");
        self.direct_emit("jmp .convert_start");

        self.direct_emit(".negative:");
        self.direct_emit("neg rax");
        self.direct_emit("jo  .neg_min");
        self.direct_emit("mov r8, 1");
        self.direct_emit("jmp .convert_start");

        self.direct_emit(".neg_min:");
        self.direct_emit("mov r8, 1");
        self.direct_emit("jmp .convert_start");

        self.direct_emit(".positive:");
        self.direct_emit("xor r8, r8");

        self.direct_emit(".convert_start:");
        self.direct_emit(".convert:");
        self.direct_emit("xor rdx, rdx");
        self.direct_emit("div rbx");
        self.direct_emit("add dl, '0'");
        self.direct_emit("mov [rcx], dl");
        self.direct_emit("dec rcx");
        self.direct_emit("test rax, rax");
        self.direct_emit("jnz .convert");

        self.direct_emit("inc rcx");

        self.direct_emit("test r8, r8");
        self.direct_emit("jz  .print");
        self.direct_emit("dec rcx");
        self.direct_emit("mov byte [rcx], '-'");

        self.direct_emit(".print:");
        self.direct_emit("mov rax, 1");
        self.direct_emit("mov rdi, 1");
        self.direct_emit("mov rsi, rcx");
        self.direct_emit("mov rdx, buffer + 32");
        self.direct_emit("sub rdx, rcx");
        self.direct_emit("syscall");

        self.direct_emit("pop rbx");
        self.direct_emit("ret");
    }

    fn emit_postlude(&mut self) {
        self.direct_emit("mov rax, 60"); // exit syscall
        self.direct_emit("xor rdi, rdi");
        self.direct_emit("syscall");
    }
}
