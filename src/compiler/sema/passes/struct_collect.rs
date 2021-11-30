use crate::{
    compiler::{
        constants::{ARG_SCOPE, IMPLICIT_ARG_SCOPE, RETURN_SCOPE},
        sema::{
            ast::{MemberDefinition, StructDefinition},
            identifiers::{IdentifierDefinitionType, Identifiers},
            passes::Pass,
            PreprocessedProgram, ScopedName,
        },
        VResult, Visitable, Visitor,
    },
    error::{CairoError, Result},
    parser::ast::*,
};
use std::rc::Rc;

#[derive(Debug, Default)]
pub struct StructCollectorPass;

impl Pass for StructCollectorPass {
    fn run(&mut self, prg: &mut PreprocessedProgram) -> Result<()> {
        log::trace!("starting pass: Struct Collector");
        for module in prg.modules.iter_mut() {
            prg.identifiers.scope_tracker_mut().enter_scope(module.module_name.clone());
            prg.identifiers.scope_tracker_mut().enter_lang(module.lang()?);

            let mut visitor = StructVisitor { identifiers: &mut prg.identifiers };
            module.cairo_file.visit(&mut visitor)?;

            prg.identifiers.scope_tracker_mut().exit_scope();
            prg.identifiers.scope_tracker_mut().exit_lang();
        }
        Ok(())
    }
}

/// A scope aware AST visitor
struct StructVisitor<'a> {
    identifiers: &'a mut Identifiers,
}

impl<'a> StructVisitor<'a> {
    fn current_identifier(&self, identifier: String) -> ScopedName {
        self.identifiers.scope_tracker.next_scope(identifier)
    }

    fn add_struct_def(
        &mut self,
        members_list: Vec<MemberInfo>,
        struct_name: ScopedName,
        loc: Loc,
    ) -> Result<()> {
        let mut offset = 0;
        let mut members = Vec::<MemberDefinition>::with_capacity(members_list.len());
        for member_info in members_list {
            let cairo_type = self.identifiers.resolve_type(member_info.ty)?;

            if members.iter().any(|m| m.name == member_info.name) {
                return Err(CairoError::Redefinition(struct_name.appended(member_info.name), loc))
            }
            let size = self.identifiers.get_size(&cairo_type)?;
            members.push(MemberDefinition {
                offset,
                name: member_info.name,
                cairo_type,
                loc: member_info.loc,
            });
            offset += size;
        }

        self.identifiers.add_name_definition(
            struct_name.clone(),
            IdentifierDefinitionType::Struct(Some(Rc::new(StructDefinition {
                full_name: struct_name,
                members,
                size: offset,
                loc,
            }))),
            loc,
            true,
        )
    }

    fn create_struct_from_identifier_list(
        &mut self,
        identifier_list: &[TypedIdentifier],
        scope: ScopedName,
        loc: Loc,
    ) -> Result<()> {
        let members = identifier_list
            .iter()
            .map(|arg| MemberInfo { name: arg.id.clone(), ty: arg.get_type(), loc: arg.loc })
            .collect();
        self.add_struct_def(members, scope, loc)
    }
}

impl<'a> Visitor for StructVisitor<'a> {
    fn visit_struct_def(&mut self, elem: &mut StructDef) -> VResult {
        if !elem.decorators.is_empty() {
            return Err(CairoError::Preprocess(format!(
                "Decorators for structs are not supported {} {}",
                elem.name, elem.loc
            )))
        }
        let struct_name = self.current_identifier(elem.name.clone());

        self.add_struct_def(elem.members.clone(), struct_name, elem.loc)
    }

    fn visit_function(&mut self, fun: &mut FunctionDef) -> VResult {
        let function_scope = self.identifiers.current_scope().as_ref().clone();

        let arg_scope = function_scope.clone().appended(ARG_SCOPE);
        self.create_struct_from_identifier_list(&fun.input_args, arg_scope, fun.loc)?;

        if let Some(ref implicit) = fun.implicit_args {
            let implicit_arg_scope = function_scope.clone().appended(IMPLICIT_ARG_SCOPE);
            self.create_struct_from_identifier_list(implicit, implicit_arg_scope, fun.loc)?;
        }

        let return_scope = function_scope.appended(RETURN_SCOPE);
        if let Some(ref return_args) = fun.return_values {
            self.create_struct_from_identifier_list(return_args, return_scope, fun.loc)?;
        } else {
            self.create_struct_from_identifier_list(&[], return_scope, fun.loc)?;
        }

        Ok(())
    }

    fn enter_function(&mut self, f: &mut FunctionDef) -> VResult {
        self.identifiers.enter_function(f)
    }

    fn exit_function(&mut self, f: &mut FunctionDef) -> VResult {
        self.identifiers.exit_function(f)
    }

    fn enter_namespace(&mut self, n: &mut Namespace) -> VResult {
        self.identifiers.enter_namespace(n)
    }

    fn visit_namespace(&mut self, ns: &mut Namespace) -> VResult {
        let function_scope = self.identifiers.scope_tracker.current_scope().as_ref().clone();
        let arg_scope = function_scope.clone().appended(ARG_SCOPE);
        self.create_struct_from_identifier_list(&[], arg_scope, ns.loc)?;
        let implicit_arg_scope = function_scope.clone().appended(IMPLICIT_ARG_SCOPE);
        self.create_struct_from_identifier_list(&[], implicit_arg_scope, ns.loc)?;
        let return_scope = function_scope.appended(RETURN_SCOPE);
        self.create_struct_from_identifier_list(&[], return_scope, ns.loc)
    }

    fn exit_namespace(&mut self, n: &mut Namespace) -> VResult {
        self.identifiers.exit_namespace(n)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::sema::{passes::identifier::IdentifierCollectorPass, CairoModule};
    use std::collections::HashMap;

    fn try_collect_struct_def<'a>(codes: impl IntoIterator<Item = (&'a str, &'a str)>) -> Result<Identifiers> {
        let modules = codes
            .into_iter()
            .map(|(name, code)| {
                CairoModule::new(ScopedName::from_str(name), CairoFile::parse(code).unwrap())
            })
            .collect::<Vec<_>>();

        let mut prg = PreprocessedProgram::with_modules(ScopedName::main_scope(), modules);

        let mut id_pass = IdentifierCollectorPass::default();
        id_pass.run(&mut prg)?;

        let mut struct_pass = StructCollectorPass::default();
        struct_pass.run(&mut prg)?;
        Ok(prg.identifiers)
    }

    fn collect_struct_def<'a>(codes: impl IntoIterator<Item = (&'a str, &'a str)>) -> Identifiers {
        try_collect_struct_def(codes).unwrap()
    }

        #[test]
    fn test_struct_collect() {
        let ids = collect_struct_def([
            (
                "module",
                r#"
struct S:
    member x : S*
    member y : S*
end
"#,
            ),
            (
                "__main__",
                r#"
from module import S

func foo{z}(a : S, b) -> (c : S):
    struct T:
        member x : S*
    end
    const X = 5
    return (c=a + X)
end
const Y = 1 + 1
"#,
            ),
        ]);

        let mut resolved: HashMap<_, _> =
            ids.resolved_identifiers().map(|(n, t)| (n.clone(), t.as_ref().clone())).collect();

        let ty = resolved.remove(&"module.S".into()).unwrap();
        pretty_assertions::assert_eq!(
            ty,
            IdentifierDefinitionType::Struct(Some(Rc::new(StructDefinition {
                full_name: "module.S".into(),
                members: vec![
                    MemberDefinition {
                        offset: 0,
                        name: "x".to_string(),
                        cairo_type: CairoType::Pointer(Box::new(PointerType::Single(
                            CairoType::Id(TypeStruct {
                                name: ScopedName::from("module.S").into_inner(),
                                is_fully_resolved: true,
                                loc: Loc::default()
                            })
                        ))),
                        loc: Loc::default()
                    },
                    MemberDefinition {
                        offset: 1,
                        name: "y".to_string(),
                        cairo_type: CairoType::Pointer(Box::new(PointerType::Single(
                            CairoType::Id(TypeStruct {
                                name: ScopedName::from("module.S").into_inner(),
                                is_fully_resolved: true,
                                loc: Loc::default()
                            })
                        ))),
                        loc: Loc::default()
                    }
                ],
                size: 2,
                loc: Loc::default()
            })))
        );

        let ty = resolved.remove(&"__main__.S".into()).unwrap();
        pretty_assertions::assert_eq!(ty, IdentifierDefinitionType::Alias("module.S".into()));

        let ty = resolved.remove(&"__main__.foo.Args".into()).unwrap();
        pretty_assertions::assert_eq!(
            ty,
            IdentifierDefinitionType::Struct(Some(Rc::new(StructDefinition {
                full_name: "__main__.foo.Args".into(),
                members: vec![
                    MemberDefinition {
                        offset: 0,
                        name: "a".to_string(),
                        cairo_type: CairoType::Id(TypeStruct {
                            name: ScopedName::from("module.S").into_inner(),
                            is_fully_resolved: true,
                            loc: Loc::default()
                        }),
                        loc: Loc::default()
                    },
                    MemberDefinition {
                        offset: 2,
                        name: "b".to_string(),
                        cairo_type: CairoType::Felt,
                        loc: Loc::default()
                    },
                ],
                size: 3,
                loc: Loc::default()
            })))
        );

        let ty = resolved.remove(&"__main__.foo.ImplicitArgs".into()).unwrap();
        pretty_assertions::assert_eq!(
            ty,
            IdentifierDefinitionType::Struct(Some(Rc::new(StructDefinition {
                full_name: "__main__.foo.ImplicitArgs".into(),
                members: vec![MemberDefinition {
                    offset: 0,
                    name: "z".to_string(),
                    cairo_type: CairoType::Felt,
                    loc: Loc::default()
                },],
                size: 1,
                loc: Loc::default()
            })))
        );

        let ty = resolved.remove(&"__main__.foo.Return".into()).unwrap();
        pretty_assertions::assert_eq!(
            ty,
            IdentifierDefinitionType::Struct(Some(Rc::new(StructDefinition {
                full_name: "__main__.foo.Return".into(),
                members: vec![MemberDefinition {
                    offset: 0,
                    name: "c".to_string(),
                    cairo_type: CairoType::Id(TypeStruct {
                        name: ScopedName::from("module.S").into_inner(),
                        is_fully_resolved: true,
                        loc: Loc::default()
                    }),
                    loc: Loc::default()
                },],
                size: 2,
                loc: Loc::default()
            })))
        );

        let ty = resolved.remove(&"__main__.foo.T".into()).unwrap();
        pretty_assertions::assert_eq!(
            ty,
            IdentifierDefinitionType::Struct(Some(Rc::new(StructDefinition {
                full_name: "__main__.foo.T".into(),
                members: vec![MemberDefinition {
                    offset: 0,
                    name: "x".to_string(),
                    cairo_type: CairoType::Pointer(Box::new(PointerType::Single(CairoType::Id(
                        TypeStruct {
                            name: ScopedName::from("module.S").into_inner(),
                            is_fully_resolved: true,
                            loc: Loc::default()
                        }
                    )))),
                    loc: Loc::default()
                },],
                size: 1,
                loc: Loc::default()
            })))
        );

        assert!(resolved.is_empty());
    }


    #[test]
    fn can_collect_errors() {
        let ids = try_collect_struct_def([
            (
                "module",
                r#"
struct S:
    member z : S*
    member z : S*
end
"#,
            ),
        ]);
        match ids {
            Err(CairoError::Redefinition(name, _)) => {
                assert_eq!(name, "module.S.z".into())
            }
            _ => panic!()
        }
    }
}
