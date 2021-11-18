//! AST for Cairo-lang based on https://cairo-lang.org/docs/reference/syntax.html
use crate::lexer::{CairoLexer, CairoLexerError};
use crate::parser;
use std::fmt::{self, Write};
use std::path::Path;

///  start offset, end offset (in bytes)
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Loc(pub usize, pub usize);

// [a-zA-Z_][\w_]+?
pub type IDStr = String;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AliasedId {
    pub id: String,
    pub alias: Option<String>,
}

impl fmt::Display for AliasedId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(alias) = self.alias.as_ref() {
            write!(f, "{} as {}", self.id, alias)
        } else {
            self.id.fmt(f)
        }
    }
}

// [a-zA-Z_][\w_]+?
pub type Identifier = Vec<String>;

/// Represents a set of cairo instructions, like all instructions in a file
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CairoFile(pub Vec<Instruction>);

impl CairoFile {
    /// Parses the contents of a Cairo file.
    pub fn parse(input: &str) -> Result<Self, CairoLexerError> {
        let input = input.trim_start();
        let instructions =
            parser::cairo_grammar::CodeBlockParser::new().parse(input, CairoLexer::new(input))?;
        Ok(CairoFile(instructions))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Imports {
    pub builtins: Vec<String>,
    pub imports: Vec<ImportDirective>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportDirective {
    /// the segments of the module name like `starkware.cairo.common.math`
    pub segments: Identifier,
    /// function names after the import
    pub functions: FunctionImport,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FunctionImport {
    Direct(Vec<AliasedId>),
    Parantheses(Vec<AliasedId>),
}

impl fmt::Display for FunctionImport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FunctionImport::Direct(imports) => {
                write!(f, "import (")?;
                for (idx, fun) in imports.iter().enumerate() {
                    fun.fmt(f)?;
                    if idx < imports.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                Ok(())
            }
            FunctionImport::Parantheses(imports) => {
                for (idx, fun) in imports.iter().enumerate() {
                    fun.fmt(f)?;
                    if idx < imports.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                f.write_char(')')
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Struct {
    pub decorators: Vec<String>,
    pub name: String,
    pub members: Vec<Pair>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Namespace {
    pub decorators: Vec<String>,
    pub name: String,
    pub instructions: Vec<Instruction>,
}

impl fmt::Display for Struct {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "struct {}:", self.name)?;
        for mem in &self.members {
            writeln!(f, "    member {}", mem)?;
        }
        f.write_str("end")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Member {
    pub name: String,
    pub ty: Type,
}

impl fmt::Display for Member {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "member {}: {}", self.name, self.ty)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Felt,
    Id(Identifier),
    /// A tuple is a finite, ordered, unchangeable list of elements.
    Tuple(Vec<Type>),
    Pointer(Box<PointerType>),
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Felt => f.write_str("felt"),
            Type::Id(name) => write!(f, "{}", name.join(".")),
            Type::Tuple(els) => {
                f.write_char('(')?;
                for (idx, el) in els.iter().enumerate() {
                    el.fmt(f)?;
                    if idx < els.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                f.write_char(')')
            }
            Type::Pointer(p) => p.fmt(f),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PointerType {
    Single(Type),
    Double(Type),
}

impl fmt::Display for PointerType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PointerType::Single(ty) => {
                write!(f, "{}+", ty)
            }
            PointerType::Double(ty) => write!(f, "{}++", ty),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Atom {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Int(i128),
    HexInt(String),
    ShortString(String),
    Hint(String),
    Register(Register),
    FunctionCall(FunctionCall),
    Id(Identifier),
    Deref(Box<Expr>),
    Subscript(Box<Expr>, Box<Expr>),
    Dot(Box<Expr>, String),
    Cast(Box<Expr>, Type),
    Parentheses(Vec<ExprAssignment>),
    Address(Box<Expr>),
    Neg(Box<Expr>),
    Pow(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExprAssignment {
    Expr(Expr),
    Id(String, Expr),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BoolExpr {
    Equal(Expr, Expr),
    NotEqual(Expr, Expr),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Register {
    Ap,
    Fp,
}

impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Register::Ap => f.write_str("ap"),
            Register::Fp => f.write_str("fp"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstantDef {
    pub name: String,
    pub init: Expr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedIdentifier {
    pub is_local: bool,
    pub id: String,
    pub ty: Option<Type>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instruction {
    Const(ConstantDef),
    Member(TypedIdentifier),
    Let(RefBinding, Box<RValue>),
    Local(TypedIdentifier, Option<Expr>),
    Tempvar(TypedIdentifier, Option<Expr>),
    Assert(Expr, Expr),
    StaticAssert(Expr, Expr),
    Return(Vec<ExprAssignment>),
    ReturnFunctionCall(FunctionCall),
    If(IfStatement),
    Label(Identifier),
    Function(FunctionDef),
    FunctionCall(FunctionCall),
    Struct(Struct),
    Namespace(Namespace),
    WithAttrStatement(WithAttrStatement),
    WithStatement(WithStatement),
    Hint(String),
    Directive(Directive),
    Import(ImportDirective),
    AllocLocals,

    // instruction
    Assign(Expr, Expr),
    Jmp(Jmp),
    CallInstruction(Call),
    Ret,
    ApAddAssign(Expr),
    ApAdd(Box<Instruction>),
}

impl Instruction {
    /// Parses a Cairo instruction
    pub fn parse(input: &str) -> Result<Self, CairoLexerError> {
        let input = input.trim_start();
        let instruction =
            parser::cairo_grammar::CodeElementParser::new().parse(input, CairoLexer::new(input))?;
        Ok(instruction)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Directive {
    Lang(Identifier),
    Builtins(Vec<String>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RValue {
    Call(Call),
    Expr(Expr),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Call {
    Rel(Expr),
    Abs(Expr),
    Id(Identifier),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Jmp {
    Rel(Expr),
    Abs(Expr),
    Id(Identifier),
    RelIf(Expr, Expr, i128),
    IdIf(Identifier, Expr, i128),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WithStatement {
    pub ids: Vec<AliasedId>,
    pub instructions: Vec<Instruction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WithAttrStatement {
    pub id: String,
    pub attr_val: Option<Vec<String>>,
    pub instructions: Vec<Instruction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RefBinding {
    Id(TypedIdentifier),
    List(Vec<TypedIdentifier>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionDef {
    pub decorators: Vec<String>,
    pub name: String,
    pub implicit_args: Option<Vec<TypedIdentifier>>,
    pub input_args: Vec<TypedIdentifier>,
    pub return_values: Option<Vec<TypedIdentifier>>,
    pub instructions: Vec<Instruction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionCall {
    pub id: Identifier,
    pub implicit_args: Option<Vec<ExprAssignment>>,
    pub args: Vec<ExprAssignment>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pair {
    pub name: String,
    pub ty: Type,
}

impl fmt::Display for Pair {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} : {}", self.name, self.ty)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IfStatement {
    pub cond: BoolExpr,
    pub instructions: Vec<Instruction>,
    pub else_branch: Option<Vec<Instruction>>,
}
