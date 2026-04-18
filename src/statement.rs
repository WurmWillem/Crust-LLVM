use crate::{expression::Expr, value::ValueType};

#[derive(Debug, Clone)]
pub struct Stmt {
    pub stmt: StmtType,
    // TODO: make this private add getter
    pub line: u32,
}
impl Stmt {
    pub fn new(stmt: StmtType, line: u32) -> Stmt {
        Stmt { stmt, line }
    }
}

#[derive(Debug, Clone)]
pub enum StmtType {
    Expr(Expr),
    VarDecl {
        name: String,
        value: Expr,
        ty: ValueType,
    },
    Println(Expr),
    Return(Expr),
    Break,
    Continue,
    Block(Vec<Stmt>),
    If {
        condition: Expr,
        body: Box<Stmt>,
        final_else: Option<Box<Stmt>>,
    },
    While {
        condition: Expr,
        body: Box<Stmt>,
    },
    For {
        var: Box<Stmt>,
        condition: Expr,
        body: Box<Stmt>,
    },
    Func {
        name: String,
        parameters: Vec<(ValueType, String)>,
        body: Vec<Stmt>,
        return_ty: ValueType,
        use_self: bool,
    },
    Struct {
        name: String,
        fields: Vec<(ValueType, String)>,
        methods: Vec<Stmt>,
    },
    Enum {
        name: String,
        variants: Vec<String>,
    },
}
