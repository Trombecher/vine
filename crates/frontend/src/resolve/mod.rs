use hashbrown::HashMap;
use crate::parse::ast as parse_ast;
use ast::*;

pub mod ast;

pub fn resolve_module_content<'sf: 'resolve_arena + 'parse_arena, 'resolve_arena, 'parse_arena>(
    st: &mut SymbolTable<'sf, 'resolve_arena>,
    items: impl Iterator<Item = &'parse_arena parse_ast::TopLevelItem<'sf, 'parse_arena>>,
    resolve_arena: &'resolve_arena Bump,
    source_file_arena: &'sf Bump
) {
    for item in items {
        let parse_ast::Statement {
            statement_kind,
            annotations: _
        } = &item.statement.value;

        match statement_kind {
            parse_ast::StatementKind::TypeParameterAlias { .. } => todo!(),
            parse_ast::StatementKind::Enum { .. } => todo!(),
            parse_ast::StatementKind::Declaration {
                id,
                doc_comments,
                value,
                ty,
                is_mutable
            } => {
                
                todo!()
            },
            parse_ast::StatementKind::Struct { .. } => todo!(),
            parse_ast::StatementKind::TypeAlias { .. } => todo!(),
            parse_ast::StatementKind::Use(_) => todo!(),
            parse_ast::StatementKind::RootUse(_) => todo!(),
            parse_ast::StatementKind::Module {
                content,
                id,
                doc_comments: _
            } => {
                let mut inner_st = HashMap::new_in(resolve_arena);
                
                if let Some(content) = content {
                    resolve_module_content(&mut inner_st, content.0.iter(), resolve_arena, source_file_arena);
                } else {
                    todo!("external module")
                }

                let entry_ref = resolve_arena.alloc(RefCell::new(SymbolTableEntry {
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