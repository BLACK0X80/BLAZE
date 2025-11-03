#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub items: Vec<Item>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    Function(Function),
    Struct(Struct),
    Enum(Enum),
    Trait(Trait),
    Impl(Impl),
    Use(UseDeclaration),
    Mod(Module),
    Type(TypeAlias),
    Const(ConstDeclaration),
    Static(StaticDeclaration),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub attributes: Vec<Attribute>,
    pub visibility: Visibility,
    pub name: String,
    pub generics: Vec<GenericParam>,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    pub where_clause: Option<WhereClause>,
    pub body: Vec<Statement>,
    pub is_async: bool,
    pub is_unsafe: bool,
    pub is_const: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Struct {
    pub attributes: Vec<Attribute>,
    pub visibility: Visibility,
    pub name: String,
    pub generics: Vec<GenericParam>,
    pub fields: Vec<Field>,
    pub where_clause: Option<WhereClause>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub name: String,
    pub ty: Type,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Type {
    I32,
    I64,
    F32,
    F64,
    Bool,
    Char,
    String,
    Custom(String),
    Usize,
    Isize,
    U8,
    I8,
    U16,
    I16,
    U32,
    U64,
    I128,
    U128,
    Generic(String, Vec<Type>),
    Reference { mutable: bool, inner: Box<Type> },
    Pointer { mutable: bool, inner: Box<Type> },
    Array { element: Box<Type>, size: Option<usize> },
    Tuple(Vec<Type>),
    Function { params: Vec<Type>, return_type: Box<Type> },
    TraitObject(String),
    Impl(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Let { name: String, mutable: bool, ty: Option<Type>, value: Option<Expression> },
    Return(Option<Expression>),
    Expression(Expression),
    While { condition: Expression, body: Vec<Statement> },
    For { variable: String, iterable: Expression, body: Vec<Statement> },
    Loop { body: Vec<Statement> },
    Break(Option<Expression>),
    Continue,
    Block(Vec<Statement>),
    If { condition: Expression, then_body: Vec<Statement>, else_body: Option<Vec<Statement>> },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    IntLit(i64),
    FloatLit(f64),
    StringLit(String),
    CharLit(char),
    BoolLit(bool),
    Literal(Literal),
    Ident(String),
    Identifier(String),
    Binary { op: BinaryOp, left: Box<Expression>, right: Box<Expression> },
    Unary { op: UnaryOp, expr: Box<Expression> },
    BinaryOp { left: Box<Expression>, operator: BinaryOperator, right: Box<Expression> },
    UnaryOp { operator: UnaryOperator, operand: Box<Expression> },
    Call { func: Box<Expression>, args: Vec<Expression> },
    CallAlt { callee: Box<Expression>, args: Vec<Expression> },
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
    BitwiseAnd, BitwiseOr, BitwiseXor,
    LeftShift, RightShift,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinaryOperator {
    Add, Subtract, Multiply, Divide, Modulo,
    Equal, NotEqual, Less, LessEqual, Greater, GreaterEqual,
    LogicalAnd, LogicalOr,
    BitwiseAnd, BitwiseOr, BitwiseXor,
    LeftShift, RightShift,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnaryOp {
    Neg, Not,
    Ref, RefMut, Deref,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnaryOperator {
    Minus, Not,
    Reference, MutableReference, Dereference,
}

pub type Parameter = Param;

#[derive(Debug, Clone, PartialEq)]
pub struct Enum {
    pub attributes: Vec<Attribute>,
    pub visibility: Visibility,
    pub name: String,
    pub generics: Vec<GenericParam>,
    pub variants: Vec<EnumVariant>,
    pub where_clause: Option<WhereClause>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumVariant {
    pub name: String,
    pub data: EnumVariantData,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EnumVariantData {
    Unit,
    Tuple(Vec<Type>),
    Struct(Vec<Field>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Trait {
    pub attributes: Vec<Attribute>,
    pub visibility: Visibility,
    pub name: String,
    pub generics: Vec<GenericParam>,
    pub supertraits: Vec<String>,
    pub items: Vec<TraitItem>,
    pub where_clause: Option<WhereClause>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TraitItem {
    Function(TraitFunction),
    Type(AssociatedType),
    Const(ConstDeclaration),
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraitFunction {
    pub name: String,
    pub generics: Vec<GenericParam>,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    pub where_clause: Option<WhereClause>,
    pub default_body: Option<Vec<Statement>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AssociatedType {
    pub name: String,
    pub bounds: Vec<String>,
    pub default: Option<Type>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Impl {
    pub attributes: Vec<Attribute>,
    pub generics: Vec<GenericParam>,
    pub trait_path: Option<String>,
    pub self_type: Type,
    pub where_clause: Option<WhereClause>,
    pub items: Vec<ImplItem>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ImplItem {
    Function(Function),
    Type(TypeAlias),
    Const(ConstDeclaration),
}

#[derive(Debug, Clone, PartialEq)]
pub struct GenericParam {
    pub name: String,
    pub bounds: Vec<TypeBound>,
    pub default: Option<Type>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeBound {
    Trait(String),
    Lifetime(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct WhereClause {
    pub predicates: Vec<WherePredicate>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WherePredicate {
    pub type_param: String,
    pub bounds: Vec<TypeBound>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Attribute {
    pub name: String,
    pub args: Vec<AttributeArg>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AttributeArg {
    Literal(Literal),
    NameValue(String, Literal),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Visibility {
    Public,
    Private,
    Crate,
    Super,
    Path(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct UseDeclaration {
    pub visibility: Visibility,
    pub path: String,
    pub alias: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Module {
    pub attributes: Vec<Attribute>,
    pub visibility: Visibility,
    pub name: String,
    pub items: Option<Vec<Item>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeAlias {
    pub attributes: Vec<Attribute>,
    pub visibility: Visibility,
    pub name: String,
    pub generics: Vec<GenericParam>,
    pub ty: Type,
    pub where_clause: Option<WhereClause>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConstDeclaration {
    pub attributes: Vec<Attribute>,
    pub visibility: Visibility,
    pub name: String,
    pub ty: Type,
    pub value: Expression,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StaticDeclaration {
    pub attributes: Vec<Attribute>,
    pub visibility: Visibility,
    pub name: String,
    pub ty: Type,
    pub mutable: bool,
    pub value: Expression,
}