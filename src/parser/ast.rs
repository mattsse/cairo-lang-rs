//! AST for Cairo-lang based on https://cairo-lang.org/docs/reference/syntax.html
use crate::{
    compiler::{sema::ScopedName, VResult, Visitable, Visitor},
    error::CairoError,
    parser::{
        self,
        lexer::{CairoLexer, CairoLexerError},
    },
};
use std::{
    fmt::{self, Write},
    path::Path,
};

///  start offset, end offset (in bytes)
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Loc(pub usize, pub usize);

impl fmt::Display for Loc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.0, self.1)
    }
}

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

impl AsRef<Vec<Instruction>> for CairoFile {
    fn as_ref(&self) -> &Vec<Instruction> {
        &self.0
    }
}

impl Visitable for CairoFile {
    fn visit(&mut self, v: &mut dyn Visitor) -> VResult {
        self.0.visit(v)
    }
}

impl fmt::Display for CairoFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt_trailing_newline(&self.0, f)
    }
}

/// An identifier with a potential alias `x as y`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AliasedId {
    pub id: String,
    pub alias: Option<String>,
    pub loc: Loc,
}

impl AliasedId {
    pub fn identifier(&self) -> &str {
        if let Some(ref alias) = self.alias {
            alias
        } else {
            &self.id
        }
    }
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
    pub loc: Loc,
    /// the path segments of the module name like `starkware.cairo.common.math`
    pub path: Identifier,
    /// function names after the import
    pub functions: FunctionImport,
}

impl ImportDirective {
    pub fn name(&self) -> String {
        self.path.join(".")
    }

    pub fn aliased_identifier(&self) -> &[AliasedId] {
        self.functions.aliased_identifier()
    }
}

impl Visitable for ImportDirective {
    fn visit(&mut self, v: &mut dyn Visitor) -> VResult {
        v.visit_import(self)
    }
}

impl fmt::Display for ImportDirective {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("from ")?;
        puncuated(&self.path, f)?;
        write!(f, " {}", self.functions)
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FunctionImport {
    Direct(Loc, Vec<AliasedId>),
    Parantheses(Loc, Vec<AliasedId>),
}

impl FunctionImport {
    pub fn aliased_identifier(&self) -> &[AliasedId] {
        match self {
            FunctionImport::Direct(_, ids) => ids,
            FunctionImport::Parantheses(_, ids) => ids,
        }
    }
}

impl fmt::Display for FunctionImport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("import ")?;
        match self {
            FunctionImport::Direct(_, imports) => comma_separated(imports, f),
            FunctionImport::Parantheses(_, imports) => {
                f.write_char('(')?;
                comma_separated(imports, f)?;
                f.write_char(')')
            }
        }
    }
}

/// Cairo lang builtins
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Builtin {
    Pedersen,
    RangeCheck,
    Ecdsa,
    Other(String),
}

impl Builtin {
    pub fn is_pedersen(&self) -> bool {
        matches!(self, Builtin::Pedersen)
    }
    pub fn is_range_check(&self) -> bool {
        matches!(self, Builtin::RangeCheck)
    }
    pub fn is_ecdsa(&self) -> bool {
        matches!(self, Builtin::Ecdsa)
    }
    pub fn is_other(&self) -> bool {
        matches!(self, Builtin::Other(_))
    }
}

impl<T: Into<String>> From<T> for Builtin {
    fn from(s: T) -> Self {
        let s = s.into();
        match s.as_str() {
            "pedersen" => Builtin::Pedersen,
            "range_check" => Builtin::RangeCheck,
            "ecdsa" => Builtin::Ecdsa,
            _ => Builtin::Other(s),
        }
    }
}

impl fmt::Display for Builtin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Builtin::Pedersen => f.write_str("pedersen"),
            Builtin::RangeCheck => f.write_str("range_check"),
            Builtin::Ecdsa => f.write_str("ecdsa"),
            Builtin::Other(s) => s.fmt(f),
        }
    }
}

/// Cairo lang decorators
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decorator {
    View,
    External,
    Constructor,
    StorageVar,
    Other(String),
}

impl Decorator {
    pub fn is_view(&self) -> bool {
        matches!(self, Decorator::View)
    }
    pub fn is_external(&self) -> bool {
        matches!(self, Decorator::External)
    }
    pub fn is_constructor(&self) -> bool {
        matches!(self, Decorator::Constructor)
    }
    pub fn is_storage_var(&self) -> bool {
        matches!(self, Decorator::StorageVar)
    }
    pub fn is_other(&self) -> bool {
        matches!(self, Decorator::Other(_))
    }
}

impl<T: Into<String>> From<T> for Decorator {
    fn from(s: T) -> Self {
        let s = s.into();
        match s.as_str() {
            "view" => Decorator::View,
            "external" => Decorator::External,
            "constructor" => Decorator::Constructor,
            "storage_var" => Decorator::StorageVar,
            _ => Decorator::Other(s),
        }
    }
}

impl fmt::Display for Decorator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_char('@')?;
        match self {
            Decorator::View => f.write_str("view"),
            Decorator::External => f.write_str("external"),
            Decorator::Constructor => f.write_str("constructor"),
            Decorator::StorageVar => f.write_str("storage_var"),
            Decorator::Other(s) => s.fmt(f),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructDef {
    pub decorators: Vec<Decorator>,
    pub name: String,
    pub members: Vec<MemberInfo>,
    pub loc: Loc,
}

impl Visitable for StructDef {
    fn visit(&mut self, v: &mut dyn Visitor) -> VResult {
        v.visit_struct_def(self)
    }
}

impl fmt::Display for StructDef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt_trailing_newline(&self.decorators, f)?;
        writeln!(f, "struct {}:", self.name)?;
        for mem in &self.members {
            writeln!(f, "    member {}", mem)?;
        }
        f.write_str("end")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Namespace {
    pub decorators: Vec<Decorator>,
    pub name: String,
    pub instructions: Vec<Instruction>,
    pub loc: Loc,
}

impl Visitable for Namespace {
    fn visit(&mut self, v: &mut dyn Visitor) -> VResult {
        v.visit_namespace(self)?;
        self.instructions.visit(v)
    }
}

impl fmt::Display for Namespace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt_trailing_newline(&self.decorators, f)?;
        writeln!(f, "namespace {}:", self.name)?;
        fmt_trailing_newline(&self.instructions, f)?;
        f.write_str("end")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Member {
    pub name: String,
    pub ty: CairoType,
}

impl fmt::Display for Member {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "member {}: {}", self.name, self.ty)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeStruct {
    pub name: Identifier,
    /// Indicates whether scope refers to the fully resolved name
    pub is_fully_resolved: bool,
    pub loc: Loc,
}

impl TypeStruct {
    /// Returns the scope of this type if it was resolved previously
    pub fn resolved_scope(&self) -> Option<ScopedName> {
        if self.is_fully_resolved {
            Some(ScopedName::new(self.name.clone()))
        } else {
            None
        }
    }
}

impl fmt::Display for TypeStruct {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name.join("."))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CairoType {
    Felt,
    Id(TypeStruct),
    /// A tuple is a finite, ordered, unchangeable list of elements.
    Tuple(Vec<CairoType>),
    Pointer(Box<PointerType>),
}

impl Visitable for CairoType {
    fn visit(&mut self, v: &mut dyn Visitor) -> VResult {
        v.visit_type(self)
    }
}

impl fmt::Display for CairoType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CairoType::Felt => f.write_str("felt"),
            CairoType::Id(ty) => ty.fmt(f),
            CairoType::Tuple(els) => {
                f.write_char('(')?;
                comma_separated(els, f)?;
                f.write_char(')')
            }
            CairoType::Pointer(p) => p.fmt(f),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PointerType {
    Single(CairoType),
    Double(CairoType),
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
    Int(i128, Loc),
    HexInt(String, Loc),
    ShortString(String, Loc),
    Hint(String, Loc),
    Register(Register, Loc),
    FunctionCall(FunctionCall),
    Id(Identifier, Loc),
    Deref(Box<Expr>, Loc),
    Subscript(Box<Expr>, Box<Expr>, Loc),
    Dot(Box<Expr>, String, Loc),
    Cast(Box<Expr>, CairoType, Loc),
    Parentheses(Vec<ExprAssignment>, Loc),
    Address(Box<Expr>, Loc),
    Neg(Box<Expr>, Loc),
    Pow(Box<Expr>, Box<Expr>, Loc),
    Mul(Box<Expr>, Box<Expr>, Loc),
    Div(Box<Expr>, Box<Expr>, Loc),
    Add(Box<Expr>, Box<Expr>, Loc),
    Sub(Box<Expr>, Box<Expr>, Loc),
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Int(expr, _) => expr.fmt(f),
            Expr::HexInt(expr, _) => expr.fmt(f),
            Expr::ShortString(expr, _) => {
                write!(f, "'{}'", expr)
            }
            Expr::Hint(expr, _) => {
                write!(f, "nondet %{{{}%}}", expr)
            }
            Expr::Register(expr, _) => expr.fmt(f),
            Expr::FunctionCall(expr) => expr.fmt(f),
            Expr::Id(expr, _) => puncuated(expr, f),
            Expr::Deref(expr, _) => {
                write!(f, "[{}]", expr)
            }
            Expr::Subscript(lhs, rhs, _) => {
                write!(f, "{} [{}]", lhs, rhs)
            }
            Expr::Dot(lhs, rhs, _) => {
                write!(f, "{}.{}", lhs, rhs)
            }
            Expr::Cast(lhs, rhs, _) => {
                write!(f, "cast({}, {})", lhs, rhs)
            }
            Expr::Parentheses(expr, _) => {
                f.write_char('(')?;
                comma_separated(expr, f)?;
                f.write_char(')')
            }
            Expr::Address(expr, _) => {
                write!(f, "&{}", expr)
            }
            Expr::Neg(expr, _) => {
                write!(f, "-{}", expr)
            }
            Expr::Pow(lhs, rhs, _) => {
                write!(f, "{}**{}", lhs, rhs)
            }
            Expr::Mul(lhs, rhs, _) => {
                write!(f, "{} * {}", lhs, rhs)
            }
            Expr::Div(lhs, rhs, _) => {
                write!(f, "{} / {}", lhs, rhs)
            }
            Expr::Add(lhs, rhs, _) => {
                write!(f, "{} + {}", lhs, rhs)
            }
            Expr::Sub(lhs, rhs, _) => {
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
    pub loc: Loc,
}

impl fmt::Display for ConstantDef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "const {} = {}", self.name, self.init)
    }
}

impl Visitable for ConstantDef {
    fn visit(&mut self, v: &mut dyn Visitor) -> VResult {
        v.visit_const_def(self)?;
        v.visit_expr(&mut self.init)
    }
}

/// An identifier with an optional type hint `local <id> : ty`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedIdentifier {
    pub is_local: bool,
    pub id: String,
    pub ty: Option<CairoType>,
    pub loc: Loc,
}

impl Visitable for TypedIdentifier {
    fn visit(&mut self, v: &mut dyn Visitor) -> VResult {
        v.visit_typed_identifier(self)?;
        if let Some(ty) = self.ty.as_mut() {
            ty.visit(v)?;
        }
        Ok(())
    }
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
    Member(TypedIdentifier, Loc),
    Let(RefBinding, Box<RValue>, Loc),
    Local(TypedIdentifier, Option<Expr>, Loc),
    Tempvar(TypedIdentifier, Option<Expr>, Loc),
    Assert(Expr, Expr, Loc),
    StaticAssert(Expr, Expr, Loc),
    Return(Vec<ExprAssignment>, Loc),
    ReturnFunctionCall(FunctionCall, Loc),
    If(IfStatement),
    Label(Identifier, Loc),
    Function(FunctionDef),
    FunctionCall(FunctionCall),
    Struct(StructDef),
    Namespace(Namespace),
    WithAttrStatement(WithAttrStatement),
    WithStatement(WithStatement),
    Hint(String, Loc),
    Directive(Directive),
    Import(ImportDirective),
    AllocLocals(Loc),

    // instruction
    Assign(Expr, Expr, Loc),
    Jmp(Jmp, Loc),
    CallInstruction(Call),
    Ret(Loc),
    ApAddAssign(Expr, Loc),
    ApAdd(Box<Instruction>, Loc),
    DataWord(Expr, Loc),
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

impl Visitable for Instruction {
    fn visit(&mut self, v: &mut dyn Visitor) -> VResult {
        match self {
            Instruction::Const(i) => {
                i.visit(v)?;
            }
            Instruction::Member(_, _) => {}
            Instruction::Let(id, rvalue, _) => {
                v.visit_reference(id, &mut **rvalue)?;
            }
            Instruction::Local(id, expr, _) => {
                v.visit_local_var(id, expr)?;
                id.visit(v)?;
                if let Some(expr) = expr {
                    v.visit_expr(expr)?;
                }
            }
            Instruction::Tempvar(id, expr, _) => {
                v.visit_temp_var(id, expr)?;
                id.visit(v)?;
                if let Some(expr) = expr {
                    v.visit_expr(expr)?;
                }
            }
            Instruction::Assert(_, _, _) => {}
            Instruction::StaticAssert(_, _, _) => {}
            Instruction::Return(_, _) => {}
            Instruction::ReturnFunctionCall(_, _) => {}
            Instruction::If(i) => {
                i.visit(v)?;
            }
            Instruction::Label(i, loc) => {
                v.visit_label(i, *loc)?;
            }
            Instruction::Function(i) => {
                v.enter_function(i)?;
                i.visit(v)?;
                v.exit_function(i)?;
            }
            Instruction::FunctionCall(_) => {}
            Instruction::Struct(i) => {
                i.visit(v)?;
            }
            Instruction::Namespace(i) => {
                v.enter_namespace(i)?;
                i.visit(v)?;
                v.exit_namespace(i)?;
            }
            Instruction::WithAttrStatement(_) => {}
            Instruction::WithStatement(i) => {
                i.visit(v)?;
            }
            Instruction::Hint(_, _) => {}
            Instruction::Directive(_) => {}
            Instruction::Import(i) => {
                i.visit(v)?;
            }
            Instruction::AllocLocals(_) => {}
            Instruction::Assign(_, _, _) => {}
            Instruction::Jmp(_, _) => {}
            Instruction::CallInstruction(_) => {}
            Instruction::Ret(_) => {}
            Instruction::ApAddAssign(_, _) => {}
            Instruction::ApAdd(_, _) => {}
            Instruction::DataWord(_, _) => {}
        };
        Ok(())
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Instruction::Const(ins) => ins.fmt(f),
            Instruction::Member(ins, _) => {
                write!(f, "member {}", ins)
            }
            Instruction::Let(lhs, rhs, _) => {
                write!(f, "let {} = {}", lhs, rhs)
            }
            Instruction::Local(lhs, rhs, _) => {
                write!(f, "local {}", lhs)?;
                if let Some(rhs) = rhs {
                    write!(f, " = {}", rhs)?;
                }
                Ok(())
            }
            Instruction::Tempvar(lhs, rhs, _) => {
                write!(f, "tempvar {}", lhs)?;
                if let Some(rhs) = rhs {
                    write!(f, " = {}", rhs)?;
                }
                Ok(())
            }
            Instruction::Assert(lhs, rhs, _) => {
                write!(f, "assert {} = {}", lhs, rhs)
            }
            Instruction::StaticAssert(lhs, rhs, _) => {
                write!(f, "static_assert {} == {}", lhs, rhs)
            }
            Instruction::Return(ins, _) => {
                f.write_str("return(")?;
                comma_separated(ins, f)?;
                f.write_char(')')
            }
            Instruction::ReturnFunctionCall(ins, _) => {
                write!(f, "return {}", ins)
            }
            Instruction::If(ins) => ins.fmt(f),
            Instruction::Label(ins, _) => {
                puncuated(ins, f)?;
                f.write_char(':')
            }
            Instruction::Function(ins) => ins.fmt(f),
            Instruction::FunctionCall(ins) => ins.fmt(f),
            Instruction::Struct(ins) => ins.fmt(f),
            Instruction::Namespace(ins) => ins.fmt(f),
            Instruction::WithAttrStatement(ins) => ins.fmt(f),
            Instruction::WithStatement(ins) => ins.fmt(f),
            Instruction::Hint(ins, _) => {
                write!(f, "%{{{}%}}", ins)
            }
            Instruction::Directive(ins) => ins.fmt(f),
            Instruction::Import(ins) => ins.fmt(f),
            Instruction::AllocLocals(_) => f.write_str("alloc_locals"),
            Instruction::Assign(lhs, rhs, _) => {
                write!(f, "{} = {}", lhs, rhs)
            }
            Instruction::Jmp(ins, _) => ins.fmt(f),
            Instruction::CallInstruction(ins) => ins.fmt(f),
            Instruction::Ret(_) => f.write_str("ret"),
            Instruction::ApAddAssign(ins, _) => {
                write!(f, "ap+={}", ins)
            }
            Instruction::ApAdd(ins, _) => {
                write!(f, "{}; ap ++", ins)
            }
            Instruction::DataWord(ins, _) => {
                write!(f, "dw {}", ins)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Directive {
    Lang(Loc, Identifier),
    Builtins(Loc, Vec<Builtin>),
}

impl Visitable for Directive {
    fn visit(&mut self, v: &mut dyn Visitor) -> VResult {
        match self {
            Directive::Lang(_, id) => v.visit_lang(id),
            Directive::Builtins(loc, builtins) => v.visit_builtins(builtins, *loc),
        }
    }
}

impl fmt::Display for Directive {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Directive::Lang(_, lang) => {
                f.write_str("%lang ")?;
                puncuated(lang, f)
            }
            Directive::Builtins(_, b) => {
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
    pub loc: Loc,
}

impl Visitable for WithStatement {
    fn visit(&mut self, v: &mut dyn Visitor) -> VResult {
        v.visit_with(self)?;
        self.instructions.visit(v)
    }
}

impl fmt::Display for WithStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("with ")?;
        comma_separated(&self.ids, f)?;
        f.write_str(" :\n")?;
        fmt_trailing_newline(&self.instructions, f)?;
        f.write_str("end")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WithAttrStatement {
    pub id: String,
    pub attr_val: Option<Vec<String>>,
    pub instructions: Vec<Instruction>,
    pub loc: Loc,
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
        fmt_trailing_newline(&self.instructions, f)?;
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
    pub decorators: Vec<Decorator>,
    pub name: String,
    pub implicit_args: Option<Vec<TypedIdentifier>>,
    pub input_args: Vec<TypedIdentifier>,
    pub return_values: Option<Vec<TypedIdentifier>>,
    pub instructions: Vec<Instruction>,
    pub loc: Loc,
}

impl Visitable for FunctionDef {
    fn visit(&mut self, v: &mut dyn Visitor) -> VResult {
        v.visit_function(self)?;
        self.instructions.visit(v)
    }
}

impl fmt::Display for FunctionDef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt_trailing_newline(&self.decorators, f)?;
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
        fmt_trailing_newline(&self.instructions, f)?;
        f.write_str("end")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionCall {
    pub id: Identifier,
    pub implicit_args: Option<Vec<ExprAssignment>>,
    pub args: Vec<ExprAssignment>,
    pub loc: Loc,
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
pub struct MemberInfo {
    pub name: String,
    pub ty: CairoType,
    pub loc: Loc,
}

impl fmt::Display for MemberInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} : {}", self.name, self.ty)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IfStatement {
    pub cond: BoolExpr,
    pub instructions: Vec<Instruction>,
    pub else_branch: Option<Vec<Instruction>>,
    pub label_neq: Option<String>,
    pub label_end: Option<String>,
    pub loc: Loc,
}

impl Visitable for IfStatement {
    fn visit(&mut self, v: &mut dyn Visitor) -> VResult {
        v.visit_if(self)?;
        self.instructions.visit(v)?;
        if let Some(e) = self.else_branch.as_mut() {
            e.visit(v)?;
        }
        Ok(())
    }
}

impl fmt::Display for IfStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "if {} :", self.cond)?;
        fmt_trailing_newline(&self.instructions, f)?;
        if let Some(ref el) = self.else_branch {
            writeln!(f, "else:")?;
            fmt_trailing_newline(el, f)?;
        }
        f.write_str("end")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notes {
    pub notes: Vec<Note>,
    pub loc: Loc,
}

impl fmt::Display for Notes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for c in &self.notes {
            c.fmt(f)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Note {
    NewLine(Loc),
    Comment(String, Loc),
}

impl fmt::Display for Note {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Note::NewLine(_) => f.write_char('\n'),
            Note::Comment(s, _) => {
                write!(f, "# {}", s)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Separator {
    Comma(Loc),
    NewLine(Loc),
    Comment(String, Loc),
}

impl fmt::Display for Separator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Separator::Comma(_) => f.write_char(','),
            Separator::NewLine(_) => f.write_char('\n'),
            Separator::Comment(s, _) => {
                write!(f, "# {}", s)
            }
        }
    }
}

fn fmt_trailing_newline<I, D>(items: I, f: &mut fmt::Formatter<'_>) -> fmt::Result
where
    I: IntoIterator<Item = D>,
    D: fmt::Display,
{
    let mut iter = items.into_iter().peekable();
    if iter.peek().is_none() {
        return Ok(())
    }
    separated(iter, f, '\n')?;
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
