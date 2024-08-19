use std::cell::RefCell;
use bumpalo::Bump;
use hashbrown::HashMap;
use crate::parse::ast as parse_ast;
use crate::resolve::ast::*;

pub mod ast;

pub fn resolve_module_content<'ids, 'arena>(
    st: &mut SymbolTable<'ids, 'arena>,
    mc: &parse_ast::ModuleContent<'ids>,
    resolve_arena: &'arena Bump,
    source_file_arena: &'ids Bump
) {
    for item in mc.0.iter() {
        let parse_ast::Statement {
            statement_kind,
            annotations: _
        } = &item.statement.value;

        match statement_kind {
            parse_ast::StatementKind::TypeParameterAlias { .. } => todo!(),
            parse_ast::StatementKind::Enum { .. } => todo!(),
            parse_ast::StatementKind::Declaration { .. } => todo!(),
            parse_ast::StatementKind::Struct { .. } => todo!(),
            parse_ast::StatementKind::TypeAlias { .. } => todo!(),
            parse_ast::StatementKind::Use(_) => todo!(),
            parse_ast::StatementKind::RootUse(_) => todo!(),
            parse_ast::StatementKind::Module {
                content,
                id,
                doc_comments: _
            } => {
                let mut inner_st = HashMap::new_in(arena);
                
                if let Some(content) = content {
                    resolve_module_content(&mut inner_st, &content, &arena);
                } else {
                    todo!("external module")
                }

                let entry_ref = arena.alloc(RefCell::new(SymbolTableEntry {
                    kind: SymbolTableEntryKind::Module {
                        st: inner_st,
                    },
                }));
                
                st.insert(
                    id,
                    &*entry_ref
                );
            },
        };
    }
}