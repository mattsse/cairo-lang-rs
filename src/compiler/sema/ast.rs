use crate::{
    compiler::sema::ScopedName,
    error::{CairoError, Result},
    parser::ast::*,
};
use std::rc::Rc;

/// the general purpose result type used in passes
pub type VResult = Result<()>;

/// A trait intended to be implemented by compiler passes that make it easier to traverse the AST
/// and only do operations on specific nodes.
pub trait Visitor {
    fn visit_lang(&mut self, _: &mut Identifier) -> VResult {
        Ok(())
    }

    fn visit_hint(&mut self, _: &mut String) -> VResult {
        Ok(())
    }

    fn visit_const_def(&mut self, _: &mut ConstantDef) -> VResult {
        Ok(())
    }

    fn visit_struct_def(&mut self, _: &mut StructDef) -> VResult {
        Ok(())
    }
    fn visit_with(&mut self, _: &mut WithStatement) -> VResult {
        Ok(())
    }

    fn visit_label(&mut self, _: &mut Identifier, _loc: Loc) -> VResult {
        Ok(())
    }

    fn visit_typed_identifier(&mut self, _: &mut TypedIdentifier) -> VResult {
        Ok(())
    }

    fn visit_type(&mut self, _: &mut CairoType) -> VResult {
        Ok(())
    }

    fn visit_reference(&mut self, b: &mut RefBinding, rvalue: &mut RValue) -> VResult {
        match b {
            RefBinding::Id(ty) => match rvalue {
                RValue::Call(call) => self.visit_return_value_reference(ty, call),
                RValue::Expr(expr) => self.visit_element_reference(ty, expr),
            },
            RefBinding::List(ty) => self.visit_unpack_binding(ty, rvalue),
        }
    }

    fn visit_unpack_binding(&mut self, _: &mut [TypedIdentifier], _: &mut RValue) -> VResult {
        Ok(())
    }

    fn visit_return_value_reference(&mut self, _: &mut TypedIdentifier, _: &mut Call) -> VResult {
        Ok(())
    }

    fn visit_element_reference(&mut self, _: &mut TypedIdentifier, _: &mut Expr) -> VResult {
        Ok(())
    }

    fn visit_builtins(&mut self, _: &mut [Builtin], _: Loc) -> VResult {
        Ok(())
    }

    fn visit_import(&mut self, _: &mut ImportDirective) -> VResult {
        Ok(())
    }

    fn visit_function(&mut self, _: &mut FunctionDef) -> VResult {
        Ok(())
    }

    fn enter_namespace(&mut self, _: &mut Namespace) -> VResult {
        Ok(())
    }

    fn visit_if(&mut self, _: &mut IfStatement) -> VResult {
        Ok(())
    }

    fn visit_local_var(&mut self, _: &mut TypedIdentifier, _: &mut Option<Expr>) -> VResult {
        Ok(())
    }

    fn visit_temp_var(&mut self, _: &mut TypedIdentifier, _: &mut Option<Expr>) -> VResult {
        Ok(())
    }

    fn visit_expr(&mut self, _: &mut Expr) -> VResult {
        Ok(())
    }

    fn visit_expr_dot(&mut self, _: &mut Expr, _: &mut String, _: Loc) -> VResult {
        Ok(())
    }

    fn visit_expr_cat(&mut self, _: &mut Expr, _: &mut CairoType, _: Loc) -> VResult {
        Ok(())
    }

    fn visit_expr_assignment(&mut self, _: &mut ExprAssignment) -> VResult {
        Ok(())
    }

    fn visit_expr_identifier(&mut self, _: &mut Identifier, _: Loc) -> VResult {
        Ok(())
    }

    fn enter_function(&mut self, _: &mut FunctionDef) -> VResult {
        Ok(())
    }

    fn exit_function(&mut self, _: &mut FunctionDef) -> VResult {
        Ok(())
    }

    fn visit_namespace(&mut self, _: &mut Namespace) -> VResult {
        Ok(())
    }

    fn exit_namespace(&mut self, _: &mut Namespace) -> VResult {
        Ok(())
    }
}

/// A trait for AST nodes that get called by their parent nodes with the current compiler pass
/// `Vistor`
pub trait Visitable {
    /// enter the node with the given visitor. The node is expected to call the visitor's callback
    /// functions while visiting its child nodes.
    fn visit(&mut self, v: &mut dyn Visitor) -> VResult;
}

impl<T: Visitable> Visitable for Vec<T> {
    fn visit(&mut self, v: &mut dyn Visitor) -> VResult {
        for t in self {
            t.visit(v)?;
        }
        Ok(())
    }
}

/// A visitor that returns the %lang directive of a cairo file
#[derive(Default)]
pub struct LangVisitor(Option<String>);

impl LangVisitor {
    pub fn lang(file: &mut CairoFile) -> Result<Option<String>> {
        let mut lang = Self::default();
        file.visit(&mut lang)?;
        Ok(lang.0)
    }
}
impl Visitor for LangVisitor {
    fn visit_lang(&mut self, id: &mut Identifier) -> VResult {
        let id = id.join(".");
        if self.0.is_some() {
            return Err(CairoError::msg(format!("Found two %lang directives {}", id)))
        }
        self.0 = Some(id);
        Ok(())
    }
}

/// Tracks the current scope when traversing the AST
#[derive(Clone, Debug, Default)]
pub struct ScopeTracker {
    accessible_scopes: Vec<Rc<ScopedName>>,
    file_lang: Option<String>,
    tmp_lang: Option<String>,
}

impl ScopeTracker {
    pub fn enter_scope(&mut self, scope: Rc<ScopedName>) {
        self.accessible_scopes.push(scope);
    }

    pub fn exit_scope(&mut self) {
        self.accessible_scopes.pop();
    }

    pub fn enter_lang(&mut self, lang: Option<String>) {
        self.tmp_lang = lang;
        std::mem::swap(&mut self.file_lang, &mut self.tmp_lang)
    }

    pub fn exit_lang(&mut self) {
        std::mem::swap(&mut self.file_lang, &mut self.tmp_lang)
    }

    pub fn current_scope(&self) -> &Rc<ScopedName> {
        debug_assert!(!self.accessible_scopes.is_empty());
        self.accessible_scopes.last().expect("requires at least one scope")
    }

    pub fn accessible_scopes(&self) -> &[Rc<ScopedName>] {
        &self.accessible_scopes
    }

    pub fn next_scope(&self, name: String) -> ScopedName {
        let mut s = self.current_scope().as_ref().clone();
        s.push(name);
        s
    }
}

impl Visitor for ScopeTracker {
    fn enter_namespace(&mut self, n: &mut Namespace) -> VResult {
        self.enter_scope(Rc::new(self.next_scope(n.name.clone())));
        Ok(())
    }

    fn enter_function(&mut self, f: &mut FunctionDef) -> VResult {
        self.enter_scope(Rc::new(self.next_scope(f.name.clone())));
        Ok(())
    }

    fn exit_function(&mut self, _: &mut FunctionDef) -> VResult {
        self.exit_scope();
        Ok(())
    }

    fn exit_namespace(&mut self, _: &mut Namespace) -> VResult {
        self.exit_scope();
        Ok(())
    }
}

pub(crate) mod macros {

    /// an internal macro that delegates entering and exiting scopes
    macro_rules! delegate_scope_tracking {
        () => {
            fn enter_function(&mut self, f: &mut FunctionDef) -> VResult {
                self.identifiers.enter_function(f)
            }

            fn exit_function(&mut self, f: &mut FunctionDef) -> VResult {
                self.identifiers.exit_function(f)
            }

            fn enter_namespace(&mut self, n: &mut Namespace) -> VResult {
                self.identifiers.enter_namespace(n)
            }

            fn exit_namespace(&mut self, n: &mut Namespace) -> VResult {
                self.identifiers.exit_namespace(n)
            }
        };
    }

    pub(crate) use delegate_scope_tracking;
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct MemberDefinition {
    pub offset: u64,
    pub name: String,
    pub cairo_type: CairoType,
    pub loc: Loc,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct StructDefinition {
    pub full_name: ScopedName,
    pub members: Vec<MemberDefinition>,
    pub size: u64,
    pub loc: Loc,
}
