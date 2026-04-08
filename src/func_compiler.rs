use crate::error::EmitErr;

#[derive(Debug)]
pub struct FuncCompilerStack {
    comps: Vec<FuncCompiler>,
    current: usize,
}
impl FuncCompilerStack {
    pub fn new() -> Self {
        Self {
            comps: vec![],
            current: 0,
        }
    }

    pub fn begin_scope(&mut self) {
        self.comps[self.current].scope_depth += 1;
    }

    // pub fn end_scope(&mut self) {
    //     self.comps[self.current].scope_depth -= 1;
    //
    //     while self.should_remove_local() {
    //         // self.emit_byte(OpCode::Pop as u8, 69);
    //         self.comps[self.current].local_count -= 1;
    //     }
    // }
    //
    // pub fn end_compiler(&mut self, line: u32) {
    //     self.emit_return(line);
    //
    //     self.current = 0;
    //     // self.comps.pop().unwrap().get_func()
    // }
    //
    // pub fn emit_return(&mut self, line: u32) {
    //     // self.emit_byte(OpCode::Null as u8, line);
    //     // self.emit_byte(OpCode::Return as u8, line);
    // }
    //
    // pub fn emit_byte(&mut self, byte: u8, line: u32) {
    //     // self.write_byte_to_chunk(byte, line);
    // }
    //
    // pub fn emit_bytes(&mut self, byte_0: u8, byte_1: u8, line: u32) {
    //     self.emit_byte(byte_0, line);
    //     self.emit_byte(byte_1, line);
    // }
    //
    // pub fn decrement_local_count(&mut self) {
    //     self.comps[self.current].local_count -= 1;
    // }
    //
    // pub fn get_local_count(&self) -> usize {
    //     self.comps[self.current].local_count
    // }
    //
    // pub fn add_local(&mut self, name: String, line: u32) -> Result<(), EmitErr> {
    //     if self.current().local_count == MAX_LOCAL_AMT {
    //         return Err(EmitErr::new(line, "Too many locals."));
    //     }
    //
    //     let local = Local::new(name, self.current().scope_depth);
    //
    //     let local_count = self.current().local_count;
    //     self.comps[self.current].locals[local_count] = local;
    //     self.comps[self.current].local_count += 1;
    //     Ok(())
    // }
    //
    // pub fn push(&mut self, func_name: String) {
    //     let new_compiler = FuncCompiler::new(func_name);
    //     self.comps.push(new_compiler);
    //     self.current = self.comps.len() - 1;
    // }
    //
    // pub fn push_new_continue_stack(&mut self) {
    //     self.comps[self.current].continue_stack.push(vec![]);
    // }
    //
    // pub fn resolve_local(&mut self, name: &str) -> Option<u8> {
    //     for i in (0..self.current().local_count).rev() {
    //         if self.current().locals[i].name == name {
    //             return Some(i as u8);
    //         }
    //     }
    //     None
    // }
    //
    // fn should_remove_local(&self) -> bool {
    //     let depth = self.current().locals[self.current().local_count - 1].depth;
    //     self.current().local_count > 0 && depth > self.current().scope_depth
    // }
    //
    fn current(&self) -> &FuncCompiler {
        &self.comps[self.current]
    }
}

#[derive(Debug, Clone)]
struct Local {
    name: String,
    depth: usize,
}
impl Local {
    fn new(name: String, depth: usize) -> Self {
        Self { name, depth }
    }
}

const MAX_LOCAL_AMT: usize = u8::MAX as usize;

#[derive(Debug)]
pub struct FuncCompiler {
    output: String,
    locals: Vec<Local>,
    scope_depth: usize,
    break_stack: Vec<Vec<usize>>,
    continue_stack: Vec<Vec<usize>>,
}
impl FuncCompiler {
    pub fn new(func_name: String) -> Self {
        // let local = Local::new("".to_string(), 0);
        let mut func = Self {
            output: String::new(),
            locals: vec![],
            scope_depth: 0,
            break_stack: vec![],
            continue_stack: vec![],
        };
        func.emit("push rbp");
        func.emit("mov rbp, rsp");
        func.emit("sub rsp, 16");
        func
    }

    pub fn get_output(&self) -> String {
        self.output.clone()
    }

    pub fn emit(&mut self, line: &str) {
        self.output.push_str(line);
        self.output.push('\n');
    }

    pub fn add_local(&mut self, name: String) {
        self.locals.push(Local::new(name, self.scope_depth));
    }

    pub fn resolve_local(&self, name: &str) -> Option<u8> {
        for i in (0..self.locals.len()).rev() {
            if self.locals[i].name == name {
                return Some(i as u8);
            }
        }
        None
    }

    pub fn get_local_count(&self) -> usize {
        self.locals.len()
    }
}
