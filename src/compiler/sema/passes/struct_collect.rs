use crate::{
    compiler::{
        sema::{
            ast::ScopeTracker, identifiers::Identifiers, passes::Pass, PreprocessedProgram,
            ScopedName,
        },
        VResult, Visitable, Visitor,
    },
    error::{CairoError, Result},
    parser::ast::*,
};

#[derive(Debug, Default)]
pub struct StructCollectorPass;

impl Pass for StructCollectorPass {
    fn run(&mut self, prg: &mut PreprocessedProgram) -> Result<()> {
        log::trace!("starting pass: Struct Collector");
        let mut scope_tracker = ScopeTracker::default();
        for module in prg.modules.iter_mut() {
            scope_tracker.enter_scope(module.module_name.clone());
            scope_tracker.enter_lang(module.lang()?);

            let mut visitor = StructVisitor {
                identifiers: &mut prg.identifiers,
                scope_tracker: &mut scope_tracker,
            };
            module.cairo_file.visit(&mut visitor)?;

            scope_tracker.exit_scope();
            scope_tracker.exit_lang();
        }
        Ok(())
    }
}

/// A scope aware AST visitor
struct StructVisitor<'a> {
    identifiers: &'a mut Identifiers,
    /// keeps track of the current scope
    scope_tracker: &'a mut ScopeTracker,
}

impl<'a> StructVisitor<'a> {
    fn current_identifier(&self, identifier: String) -> ScopedName {
        self.scope_tracker.next_scope(identifier)
    }
}

impl<'a> Visitor for StructVisitor<'a> {
    fn visit_struct_def(&mut self, elem: &mut StructDef) -> VResult {
        let _new_scope = self.current_identifier(elem.name.clone());

        if !elem.decorators.is_empty() {
            return Err(CairoError::Preprocess(format!(
                "Decorators for structs are not supported {} {}",
                elem.name, elem.loc
            )))
        }

        Ok(())
    }

    fn visit_function(&mut self, _fun: &mut FunctionDef) -> VResult {
        Ok(())
    }

    fn enter_function(&mut self, f: &mut FunctionDef) -> VResult {
        self.scope_tracker.enter_function(f)
    }

    fn exit_function(&mut self, f: &mut FunctionDef) -> VResult {
        self.scope_tracker.exit_function(f)
    }

    fn enter_namespace(&mut self, n: &mut Namespace) -> VResult {
        self.scope_tracker.enter_namespace(n)
    }

    fn visit_namespace(&mut self, _ns: &mut Namespace) -> VResult {
        Ok(())
    }

    fn exit_namespace(&mut self, n: &mut Namespace) -> VResult {
        self.scope_tracker.exit_namespace(n)
    }
}
