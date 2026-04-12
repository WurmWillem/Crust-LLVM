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
        Self {
            output: String::new(),
            locals: vec![],
            scope_depth: 0,
            break_stack: vec![],
            continue_stack: vec![],
        }
    }
     
    pub fn end(self) -> (String, usize) {
        (self.output, self.locals.len())
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
