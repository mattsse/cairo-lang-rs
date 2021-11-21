use crate::error::Result;
use crate::parser::ast::*;

/// the general purpose result type used in passes
pub type VResult = Result<()>;

/// A trait intended to be implemented by compiler passes that make it easier to traverse the AST and only do operations on specific nodes.
pub trait Visitor {
    fn visit_lang(&mut self, _id: &mut Identifier) -> VResult {
        Ok(())
    }

    fn visit_import(&mut self, _import: &mut ImportDirective) -> VResult {
        Ok(())
    }

    fn visit_function(&mut self, _import: &mut FunctionDef) -> VResult {
        Ok(())
    }
}

/// A trait for AST nodes that get called by their parent nodes with the current compiler pass `Vistor`
pub trait Visitable {
    /// enter the node with the given visitor. The node is expected to call the visitor's callback functions while visiting its child nodes.
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
