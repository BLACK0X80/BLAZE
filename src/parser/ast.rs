#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub items: Vec<Item>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    Function(Function),
    Struct(Struct),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Struct {
    pub name: String,
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    I32,
    I64,
    F32,
    F64,
    Bool,
    Char,
    String,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Let { name: String, mutable: bool, ty: Option<Type>, value: Expression },
    Return(Option<Expression>),
    Expression(Expression),
    While { condition: Expression, body: Vec<Statement> },
    If { condition: Expression, then_body: Vec<Statement>, else_body: Option<Vec<Statement>> },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    IntLit(i64),
    FloatLit(f64),
    StringLit(String),
    CharLit(char),
    BoolLit(bool),
    Ident(String),
    Binary { op: BinaryOp, left: Box<Expression>, right: Box<Expression> },
    Unary { op: UnaryOp, expr: Box<Expression> },
    Call { func: Box<Expression>, args: Vec<Expression> },
    Assignment { target: Box<Expression>, value: Box<Expression> },
    FieldAccess { object: Box<Expression>, field: String },
    MethodCall { object: Box<Expression>, method: String, args: Vec<Expression> },
    Index { object: Box<Expression>, index: Box<Expression> },
    TupleLiteral(Vec<Expression>),
    ArrayLiteral(Vec<Expression>),
    StructLiteral { name: String, fields: Vec<FieldInit> },
    Block(Vec<Statement>),
    If { condition: Box<Expression>, then_branch: Box<Expression>, else_branch: Option<Box<Expression>> },
    Match { expression: Box<Expression>, arms: Vec<MatchArm> },
    Closure { params: Vec<ClosureParam>, body: Box<Expression> },
}

#[derive(Debug, Clone, PartialEq)]
pub struct FieldInit {
    pub name: String,
    pub value: Expression,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Expression>,
    pub body: Expression,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Literal(Literal),
    Identifier(String),
    Wildcard,
    Tuple(Vec<Pattern>),
    Struct { name: String, fields: Vec<(String, Pattern)> },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Integer(i64),
    Float(f64),
    String(String),
    Char(char),
    Boolean(bool),
    Unit,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClosureParam {
    pub name: String,
    pub ty: Option<Type>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinaryOp {
    Add, Sub, Mul, Div, Mod,
    Eq, Ne, Lt, Le, Gt, Ge,
    And, Or,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnaryOp {
    Neg, Not,
}

// Type aliases for compatibility
pub type BinaryOperator = BinaryOp;
pub type UnaryOperator = UnaryOp;