use crate::{
    compiler::{
        constants::{ARG_SCOPE, IMPLICIT_ARG_SCOPE, N_LOCALS_CONSTANT, RETURN_SCOPE},
        sema::{
            ast::ScopeTracker, identifiers::IdentifierDefinitionType, passes::Pass, Identifiers,
            PreprocessedProgram, ScopedName,
        },
        VResult, Visitable, Visitor,
    },
    error::{CairoError, Result},
    parser::ast::*,
};
use std::collections::HashSet;

/// Resolves identifiers for cairo code elements.
#[derive(Debug, Default)]
pub struct IdentifierCollectorPass;

impl Pass for IdentifierCollectorPass {
    fn run(&mut self, prg: &mut PreprocessedProgram) -> Result<()> {
        log::trace!("starting pass: Identifier Collector");
        let mut scope_tracker = ScopeTracker::default();
        for module in prg.modules.iter_mut() {
            scope_tracker.enter_scope(module.module_name.clone());
            scope_tracker.enter_lang(module.lang()?);

            let mut visitor =
                IdVisitor { identifiers: &mut prg.identifiers, scope_tracker: &mut scope_tracker };
            module.cairo_file.visit(&mut visitor)?;

            scope_tracker.exit_scope();
            scope_tracker.exit_lang();
        }
        Ok(())
    }
}

/// A scope aware AST visitor that resolves full names of identifiers
struct IdVisitor<'a> {
    identifiers: &'a mut Identifiers,
    /// keeps track of the current scope
    scope_tracker: &'a mut ScopeTracker,
}

impl<'a> IdVisitor<'a> {
    fn current_identifier(&self, identifier: String) -> ScopedName {
        self.scope_tracker.next_scope(identifier)
    }

    /// adds an identifier to the underlying
    fn add_identifier(
        &mut self,
        name: ScopedName,
        ty: IdentifierDefinitionType,
        loc: Loc,
    ) -> VResult {
        if let Some(existing_def) = self.identifiers.get_by_full_name(&name) {
            if !existing_def.is_unresolved() || !ty.is_unresolved() {
                return Err(CairoError::Redefinition(name, loc))
            }
            if !(existing_def.is_reference() || existing_def.is_unresolved_reference()) ||
                !(ty.is_reference() || ty.is_unresolved_reference())
            {
                return Err(CairoError::Redefinition(name, loc))
            }
        }
        self.identifiers.add_identifier(name, ty);
        Ok(())
    }

    fn add_unresolved_identifier(
        &mut self,
        name: ScopedName,
        ty: IdentifierDefinitionType,
        loc: Loc,
    ) -> VResult {
        self.add_identifier(name, IdentifierDefinitionType::Unresolved(Box::new(ty)), loc)
    }

    fn handle_function_arguments(
        &mut self,
        function_scope: ScopedName,
        identifier_list: &[TypedIdentifier],
    ) -> VResult {
        for arg_id in identifier_list {
            if arg_id.id == N_LOCALS_CONSTANT {
                return Err(CairoError::Preprocess(format!(
                    "The name {} is reserved and cannot be used as argument {}",
                    N_LOCALS_CONSTANT, arg_id.loc
                )))
            }
            self.add_unresolved_identifier(
                function_scope.clone().appended(arg_id.id.clone()),
                IdentifierDefinitionType::Reference,
                arg_id.loc,
            )?;
        }
        Ok(())
    }
}

impl<'a> Visitor for IdVisitor<'a> {
    fn visit_const_def(&mut self, c: &mut ConstantDef) -> VResult {
        self.add_unresolved_identifier(
            self.current_identifier(c.name.clone()),
            IdentifierDefinitionType::ConstDef,
            c.loc,
        )
    }

    fn visit_struct_def(&mut self, s: &mut Struct) -> VResult {
        self.add_unresolved_identifier(
            self.current_identifier(s.name.clone()),
            IdentifierDefinitionType::Struct,
            s.loc,
        )
    }

    fn visit_with(&mut self, el: &mut WithStatement) -> VResult {
        for id in &el.ids {
            if let Some(alias) = id.alias.clone() {
                self.add_unresolved_identifier(
                    self.current_identifier(alias),
                    IdentifierDefinitionType::Reference,
                    el.loc,
                )?;
            }
        }
        Ok(())
    }

    fn visit_label(&mut self, id: &mut Identifier, loc: Loc) -> VResult {
        self.add_unresolved_identifier(
            self.current_identifier(id.join(".")),
            IdentifierDefinitionType::Label,
            loc,
        )
    }

    fn visit_unpack_binding(&mut self, ids: &mut [TypedIdentifier], _: &mut RValue) -> VResult {
        for id in ids.iter().filter(|s| s.id != "_") {
            self.add_unresolved_identifier(
                self.current_identifier(id.id.clone()),
                IdentifierDefinitionType::Reference,
                id.loc,
            )?;
        }
        Ok(())
    }

    fn visit_return_value_reference(&mut self, id: &mut TypedIdentifier, _: &mut Call) -> VResult {
        self.add_unresolved_identifier(
            self.current_identifier(id.id.clone()),
            IdentifierDefinitionType::Reference,
            id.loc,
        )
    }

    fn visit_element_reference(&mut self, id: &mut TypedIdentifier, _: &mut Expr) -> VResult {
        self.add_unresolved_identifier(
            self.current_identifier(id.id.clone()),
            IdentifierDefinitionType::Reference,
            id.loc,
        )
    }

    fn visit_import(&mut self, el: &mut ImportDirective) -> VResult {
        for item in el.aliased_identifier() {
            let alias_dest = ScopedName::new(el.path.clone()).appended(item.id.clone());

            // ensure destination is a valid id
            if self.identifiers.get_by_full_name(&alias_dest).is_none() {
                let _ = self.identifiers.get_scope(&alias_dest)?;
            }
            self.add_identifier(
                self.current_identifier(item.identifier().to_string()),
                IdentifierDefinitionType::Alias(alias_dest),
                el.loc,
            )?;
        }
        Ok(())
    }

    fn enter_function(&mut self, f: &mut FunctionDef) -> VResult {
        self.scope_tracker.enter_function(f)
    }

    fn visit_function(&mut self, fun: &mut FunctionDef) -> VResult {
        let function_scope = self.scope_tracker.current_scope().as_ref().clone();

        self.add_unresolved_identifier(
            function_scope.clone(),
            IdentifierDefinitionType::Function,
            fun.loc,
        )?;

        let arg_scope = function_scope.clone().appended(ARG_SCOPE);
        self.add_unresolved_identifier(arg_scope, IdentifierDefinitionType::Struct, fun.loc)?;

        self.handle_function_arguments(function_scope.clone(), &fun.input_args)?;

        if let Some(ref implicit) = fun.implicit_args {
            let implicit_arg_scope = function_scope.clone().appended(IMPLICIT_ARG_SCOPE);
            self.add_unresolved_identifier(
                implicit_arg_scope,
                IdentifierDefinitionType::Struct,
                fun.loc,
            )?;
            self.handle_function_arguments(function_scope.clone(), implicit)?;
        }

        let return_scope = function_scope.clone().appended(RETURN_SCOPE);
        self.add_unresolved_identifier(return_scope, IdentifierDefinitionType::Struct, fun.loc)?;

        // ensure there is no name collision
        if let Some(ref implicit) = fun.implicit_args {
            let implicit_arg_names = implicit.iter().map(|arg| &arg.id).collect::<HashSet<_>>();
            let mut arg_and_return_identifiers = fun.input_args.iter().collect::<Vec<_>>();

            if let Some(ref returns) = fun.return_values {
                arg_and_return_identifiers.extend(returns);
            }
            for arg_id in arg_and_return_identifiers {
                if implicit_arg_names.contains(&arg_id.id) {
                    return Err(CairoError::Preprocess(format!("Arguments and return values cannot have the same name of an implicit argument {} at {}",arg_id.id, arg_id.loc)))
                }
            }
        }

        self.add_unresolved_identifier(
            function_scope.appended(N_LOCALS_CONSTANT),
            IdentifierDefinitionType::ConstDef,
            fun.loc,
        )
    }

    fn exit_function(&mut self, f: &mut FunctionDef) -> VResult {
        self.scope_tracker.exit_function(f)
    }

    fn enter_namespace(&mut self, n: &mut Namespace) -> VResult {
        self.scope_tracker.enter_namespace(n)
    }

    fn visit_namespace(&mut self, ns: &mut Namespace) -> VResult {
        let function_scope = self.current_identifier(ns.name.clone());

        self.add_unresolved_identifier(
            function_scope.clone(),
            IdentifierDefinitionType::Namespace,
            ns.loc,
        )?;

        let arg_scope = function_scope.clone().appended(ARG_SCOPE);
        self.add_unresolved_identifier(arg_scope, IdentifierDefinitionType::Struct, ns.loc)?;

        let return_scope = function_scope.clone().appended(RETURN_SCOPE);
        self.add_unresolved_identifier(return_scope, IdentifierDefinitionType::Struct, ns.loc)?;

        self.add_unresolved_identifier(
            function_scope.appended(N_LOCALS_CONSTANT),
            IdentifierDefinitionType::ConstDef,
            ns.loc,
        )
    }

    fn exit_namespace(&mut self, n: &mut Namespace) -> VResult {
        self.scope_tracker.exit_namespace(n)
    }

    fn visit_if(&mut self, el: &mut IfStatement) -> VResult {
        let label_neq = el.label_neq.clone().ok_or(CairoError::MissingLabel(el.loc))?;
        let label_end = el.label_end.clone().ok_or(CairoError::MissingLabel(el.loc))?;
        self.add_unresolved_identifier(
            self.current_identifier(label_neq),
            IdentifierDefinitionType::Label,
            el.loc,
        )?;
        self.add_unresolved_identifier(
            self.current_identifier(label_end),
            IdentifierDefinitionType::Label,
            el.loc,
        )
    }

    fn visit_local_var(&mut self, id: &mut TypedIdentifier, _: &mut Option<Expr>) -> VResult {
        self.add_unresolved_identifier(
            self.current_identifier(id.id.clone()),
            IdentifierDefinitionType::Reference,
            id.loc,
        )
    }

    fn visit_temp_var(&mut self, id: &mut TypedIdentifier, _: &mut Option<Expr>) -> VResult {
        self.add_unresolved_identifier(
            self.current_identifier(id.id.clone()),
            IdentifierDefinitionType::Reference,
            id.loc,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{collections::HashMap, rc::Rc};

    fn try_visit(s: &str) -> Result<Identifiers> {
        let mut cairo = CairoFile::parse(s).unwrap();
        let mut identifiers = Identifiers::default();
        let mut scope_tracker = ScopeTracker::default();
        scope_tracker.enter_scope(Rc::new(ScopedName::root()));
        let mut vistor =
            IdVisitor { identifiers: &mut identifiers, scope_tracker: &mut scope_tracker };
        cairo.visit(&mut vistor)?;
        Ok(identifiers)
    }

    fn visit(s: &str) -> Identifiers {
        try_visit(s).unwrap()
    }

    #[test]
    fn test_single_binds() {
        let s = r#"
tempvar a = [ap]
const b = [ap]
local c = [ap]
let d = [fp] + 2
f:
let g : H = f(1, 2, 3)
        "#;
        let _ = visit(s);
    }

    #[test]
    fn test_collect_multi_binds() {
        let s = r#"
func a(b, c) -> (d):
    [ap] = [ap]
end
let (e, f) = g()
        "#;
        let _ids = visit(s);
    }

    #[test]
    fn test_nested_funcs() {
        let s = r#"
func foo{z}(x):
    local a
    func bar(y):
        tempvar b = [ap]
    end
end
        "#;
        let _ids = visit(s);
    }

    #[test]
    fn test_redefinition() {
        let s = r#"
tempvar name = [ap]
local name = [ap]
        "#;
        let ids = visit(s);
        assert_eq!(
            ids.identifiers,
            HashMap::from([(
                ScopedName::from_str("name"),
                Rc::new(IdentifierDefinitionType::Unresolved(Box::new(
                    IdentifierDefinitionType::Reference
                )))
            )])
        );
    }

    #[test]
    fn can_identify_redefinitions() {
        let s = r#"
foo:
local foo = [ap]
        "#;
        let res = try_visit(s).unwrap_err();
        assert!(matches!(res, CairoError::Redefinition(_, _)));

        let s = r#"
func bar():
end

func bar():
end
        "#;
        let res = try_visit(s).unwrap_err();
        assert!(matches!(res, CairoError::Redefinition(_, _)));
    }

    #[test]
    fn can_identify_imports() {
        let s = r#"
from a import b as b0
        "#;
        let mut cairo = CairoFile::parse(s).unwrap();
        let mut identifiers = Identifiers::default();
        let mut scope_tracker = ScopeTracker::default();
        identifiers.add_identifier(ScopedName::from_str("a.b"), IdentifierDefinitionType::ConstDef);
        scope_tracker.enter_scope(Rc::new(ScopedName::root()));
        let mut vistor =
            IdVisitor { identifiers: &mut identifiers, scope_tracker: &mut scope_tracker };
        cairo.visit(&mut vistor).unwrap();
        let scope =
            identifiers.identifiers.get(&ScopedName::from_str("b0")).unwrap().as_ref().clone();

        assert_eq!(scope, IdentifierDefinitionType::Alias(ScopedName::from_str("a.b")));
    }
}
