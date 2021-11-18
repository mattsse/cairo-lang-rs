//! AST for Cairo-lang based on https://cairo-lang.org/docs/reference/syntax.html
use crate::error::CairoError;
use crate::lexer::{CairoLexer, CairoLexerError};
use crate::parser;
use std::fmt::{self, Write};
use std::path::Path;

///  start offset, end offset (in bytes)
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Loc(pub usize, pub usize);

// [a-zA-Z_][\w_]+?
pub type IDStr = String;

// (Id)(.Id)*
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

    /// Read the contents of a cairo file and parse all instructions
    pub fn read(path: impl AsRef<Path>) -> Result<Self, CairoError> {
        let content = std::fs::read_to_string(path.as_ref())?;
        Ok(Self::parse(&content)?)
    }
}

impl fmt::Display for CairoFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt_instructions(&self.0, f)
    }
}

/// An identifier with a potential alias `x as y`
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportDirective {
    /// the segments of the module name like `starkware.cairo.common.math`
    pub segments: Identifier,
    /// function names after the import
    pub functions: FunctionImport,
}

impl fmt::Display for ImportDirective {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("from ")?;
        puncuated(&self.segments, f)?;
        write!(f, " {}", self.functions)
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FunctionImport {
    Direct(Vec<AliasedId>),
    Parantheses(Vec<AliasedId>),
}

impl fmt::Display for FunctionImport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("import ")?;
        match self {
            FunctionImport::Direct(imports) => comma_separated(imports, f),
            FunctionImport::Parantheses(imports) => {
                f.write_char('(')?;
                comma_separated(imports, f)?;
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

impl fmt::Display for Struct {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for s in &self.decorators {
            write!(f, "@{} ", s)?;
        }
        writeln!(f, "struct {}:", self.name)?;
        for mem in &self.members {
            writeln!(f, "    member {}", mem)?;
        }
        f.write_str("end")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Namespace {
    pub decorators: Vec<String>,
    pub name: String,
    pub instructions: Vec<Instruction>,
}

impl fmt::Display for Namespace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for s in &self.decorators {
            write!(f, "@{} ", s)?;
        }
        writeln!(f, "namespace {}:", self.name)?;
        fmt_instructions(&self.instructions, f)?;
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
                comma_separated(els, f)?;
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

/// A cairo expression
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

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Int(expr) => expr.fmt(f),
            Expr::HexInt(expr) => expr.fmt(f),
            Expr::ShortString(expr) => {
                write!(f, "'{}'", expr)
            }
            Expr::Hint(expr) => {
                write!(f, "nondet %{{{}%}}", expr)
            }
            Expr::Register(expr) => expr.fmt(f),
            Expr::FunctionCall(expr) => expr.fmt(f),
            Expr::Id(expr) => puncuated(expr, f),
            Expr::Deref(expr) => {
                write!(f, "[{}]", expr)
            }
            Expr::Subscript(lhs, rhs) => {
                write!(f, "{} [{}]", lhs, rhs)
            }
            Expr::Dot(lhs, rhs) => {
                write!(f, "{}.{}", lhs, rhs)
            }
            Expr::Cast(lhs, rhs) => {
                write!(f, "cast({}, {})", lhs, rhs)
            }
            Expr::Parentheses(expr) => {
                f.write_char('(')?;
                comma_separated(expr, f)?;
                f.write_char(')')
            }
            Expr::Address(expr) => {
                write!(f, "&{}", expr)
            }
            Expr::Neg(expr) => {
                write!(f, "-{}", expr)
            }
            Expr::Pow(lhs, rhs) => {
                write!(f, "{}**{}", lhs, rhs)
            }
            Expr::Mul(lhs, rhs) => {
                write!(f, "{} * {}", lhs, rhs)
            }
            Expr::Div(lhs, rhs) => {
                write!(f, "{} / {}", lhs, rhs)
            }
            Expr::Add(lhs, rhs) => {
                write!(f, "{} + {}", lhs, rhs)
            }
            Expr::Sub(lhs, rhs) => {
                write!(f, "{} - {}", lhs, rhs)
            }
        }
    }
}

/// Expression of  `expr | id  = expr`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExprAssignment {
    Expr(Expr),
    Id(String, Expr),
}

impl fmt::Display for ExprAssignment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExprAssignment::Expr(expr) => expr.fmt(f),
            ExprAssignment::Id(lhs, rhs) => {
                write!(f, "{} = {}", lhs, rhs)
            }
        }
    }
}

/// Expression as condition for an if statement
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BoolExpr {
    Equal(Expr, Expr),
    NotEqual(Expr, Expr),
}

impl fmt::Display for BoolExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BoolExpr::Equal(lhs, rhs) => {
                write!(f, "{} == {}", lhs, rhs)
            }
            BoolExpr::NotEqual(lhs, rhs) => {
                write!(f, "{} != {}", lhs, rhs)
            }
        }
    }
}

/// cairo registers
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

/// Definition of a constant
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstantDef {
    pub name: String,
    pub init: Expr,
}

impl fmt::Display for ConstantDef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "const {} = {}", self.name, self.init)
    }
}

/// An identifier with an optional type hint `local <id> : ty`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedIdentifier {
    pub is_local: bool,
    pub id: String,
    pub ty: Option<Type>,
}

impl fmt::Display for TypedIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_local {
            f.write_str("local ")?;
        }
        self.id.fmt(f)?;
        if let Some(ref ty) = self.ty {
            write!(f, " {}", ty)?
        }

        Ok(())
    }
}

/// Various cairo instructions a file is made up of
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

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Instruction::Const(ins) => ins.fmt(f),
            Instruction::Member(ins) => {
                write!(f, "member {}", ins)
            }
            Instruction::Let(lhs, rhs) => {
                write!(f, "let {} = {}", lhs, rhs)
            }
            Instruction::Local(lhs, rhs) => {
                write!(f, "local {}", lhs)?;
                if let Some(rhs) = rhs {
                    write!(f, " = {}", rhs)?;
                }
                Ok(())
            }
            Instruction::Tempvar(lhs, rhs) => {
                write!(f, "tempvar {}", lhs)?;
                if let Some(rhs) = rhs {
                    write!(f, " = {}", rhs)?;
                }
                Ok(())
            }
            Instruction::Assert(lhs, rhs) => {
                write!(f, "assert {} = {}", lhs, rhs)
            }
            Instruction::StaticAssert(lhs, rhs) => {
                write!(f, "static_assert {} == {}", lhs, rhs)
            }
            Instruction::Return(ins) => {
                f.write_str("return(")?;
                comma_separated(ins, f)?;
                f.write_char(')')
            }
            Instruction::ReturnFunctionCall(ins) => {
                write!(f, "return {}", ins)
            }
            Instruction::If(ins) => ins.fmt(f),
            Instruction::Label(ins) => {
                puncuated(ins, f)?;
                f.write_char(':')
            }
            Instruction::Function(ins) => ins.fmt(f),
            Instruction::FunctionCall(ins) => ins.fmt(f),
            Instruction::Struct(ins) => ins.fmt(f),
            Instruction::Namespace(ins) => ins.fmt(f),
            Instruction::WithAttrStatement(ins) => ins.fmt(f),
            Instruction::WithStatement(ins) => ins.fmt(f),
            Instruction::Hint(ins) => {
                write!(f, "%{{{}%}}", ins)
            }
            Instruction::Directive(ins) => ins.fmt(f),
            Instruction::Import(ins) => ins.fmt(f),
            Instruction::AllocLocals => f.write_str("alloc_locals"),
            Instruction::Assign(lhs, rhs) => {
                write!(f, "{} = {}", lhs, rhs)
            }
            Instruction::Jmp(ins) => ins.fmt(f),
            Instruction::CallInstruction(ins) => ins.fmt(f),
            Instruction::Ret => f.write_str("ret"),
            Instruction::ApAddAssign(ins) => {
                write!(f, "ap+={}", ins)
            }
            Instruction::ApAdd(ins) => {
                write!(f, "{}; ap ++", ins)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Directive {
    Lang(Identifier),
    Builtins(Vec<String>),
}

impl fmt::Display for Directive {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Directive::Lang(lang) => {
                f.write_str("%lang ")?;
                puncuated(lang, f)
            }
            Directive::Builtins(b) => {
                f.write_str("%builtins ")?;
                separated(b, f, ' ')
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RValue {
    Call(Call),
    Expr(Expr),
}

impl fmt::Display for RValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RValue::Call(expr) => expr.fmt(f),
            RValue::Expr(expr) => expr.fmt(f),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Call {
    Rel(Expr),
    Abs(Expr),
    Id(Identifier),
}

impl fmt::Display for Call {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("call ")?;
        match self {
            Call::Rel(expr) => {
                write!(f, "rel {}", expr)
            }
            Call::Abs(expr) => {
                write!(f, "abs {}", expr)
            }
            Call::Id(ident) => puncuated(ident, f),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Jmp {
    Rel(Expr),
    Abs(Expr),
    Id(Identifier),
    RelIf(Expr, Expr, i128),
    IdIf(Identifier, Expr, i128),
}

impl fmt::Display for Jmp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("jmp ")?;
        match self {
            Jmp::Rel(expr) => {
                write!(f, "rel {}", expr)
            }
            Jmp::Abs(expr) => {
                write!(f, "abs {}", expr)
            }
            Jmp::Id(id) => puncuated(id, f),
            Jmp::RelIf(lhs, rhs, num) => {
                write!(f, "rel {} if {} != {}", lhs, rhs, num)
            }
            Jmp::IdIf(id, rhs, num) => {
                puncuated(id, f)?;
                write!(f, " if {} != {}", rhs, num)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WithStatement {
    pub ids: Vec<AliasedId>,
    pub instructions: Vec<Instruction>,
}

impl fmt::Display for WithStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("with ")?;
        comma_separated(&self.ids, f)?;
        f.write_str(" :\n")?;
        fmt_instructions(&self.instructions, f)?;
        f.write_str("end")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WithAttrStatement {
    pub id: String,
    pub attr_val: Option<Vec<String>>,
    pub instructions: Vec<Instruction>,
}

impl fmt::Display for WithAttrStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "with_attr {} ", self.id)?;
        if let Some(ref attr) = self.attr_val {
            f.write_char('(')?;
            let mut iter = attr.iter().peekable();
            while let Some(item) = iter.next() {
                write!(f, "\"{}\"", item)?;
                if iter.peek().is_some() {
                    f.write_char(' ')?;
                }
            }
            f.write_char(')')?;
        }
        f.write_str(" :\n")?;
        fmt_instructions(&self.instructions, f)?;
        f.write_str("end")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RefBinding {
    Id(TypedIdentifier),
    List(Vec<TypedIdentifier>),
}

impl fmt::Display for RefBinding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RefBinding::Id(id) => id.fmt(f),
            RefBinding::List(ids) => {
                f.write_char('(')?;
                comma_separated(ids, f)?;
                f.write_char(')')
            }
        }
    }
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

impl fmt::Display for FunctionDef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for s in &self.decorators {
            write!(f, "@{} ", s)?;
        }
        write!(f, "func {}", self.name)?;
        if let Some(ref args) = self.implicit_args {
            f.write_char('{')?;
            comma_separated(args, f)?;
            f.write_char('}')?;
        }
        f.write_char('(')?;
        comma_separated(&self.instructions, f)?;
        f.write_char(')')?;
        if let Some(ref args) = self.return_values {
            f.write_str("-> (")?;
            comma_separated(args, f)?;
            f.write_char(')')?;
        }
        fmt_instructions(&self.instructions, f)?;
        f.write_str("end")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionCall {
    pub id: Identifier,
    pub implicit_args: Option<Vec<ExprAssignment>>,
    pub args: Vec<ExprAssignment>,
}

impl fmt::Display for FunctionCall {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        puncuated(&self.id, f)?;
        if let Some(ref args) = self.implicit_args {
            f.write_char('{')?;
            comma_separated(args, f)?;
            f.write_char('}')?;
        }
        f.write_char('(')?;
        comma_separated(&self.args, f)?;
        f.write_char(')')
    }
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

impl fmt::Display for IfStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "if {} :", self.cond)?;
        fmt_instructions(&self.instructions, f)?;
        if let Some(ref el) = self.else_branch {
            writeln!(f, "else:")?;
            fmt_instructions(el, f)?;
        }
        f.write_str("end")
    }
}

fn fmt_instructions(i: &[Instruction], f: &mut fmt::Formatter<'_>) -> fmt::Result {
    if i.is_empty() {
        return Ok(());
    }
    separated(i, f, '\n')?;
    f.write_char('\n')
}

fn comma_separated<I, D>(items: I, f: &mut fmt::Formatter<'_>) -> fmt::Result
where
    I: IntoIterator<Item = D>,
    D: fmt::Display,
{
    separated(items, f, ", ")
}

fn puncuated<I, D>(items: I, f: &mut fmt::Formatter<'_>) -> fmt::Result
where
    I: IntoIterator<Item = D>,
    D: fmt::Display,
{
    separated(items, f, '.')
}

fn separated<I, D, S>(items: I, f: &mut fmt::Formatter<'_>, separator: S) -> fmt::Result
where
    I: IntoIterator<Item = D>,
    D: fmt::Display,
    S: fmt::Display,
{
    let mut iter = items.into_iter().peekable();
    while let Some(item) = iter.next() {
        item.fmt(f)?;
        if iter.peek().is_some() {
            separator.fmt(f)?;
        }
    }
    Ok(())
}
