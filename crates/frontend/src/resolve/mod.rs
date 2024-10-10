use core::cell::RefCell;
use bumpalo::Bump;
use hashbrown::HashMap;
use crate::parse::ast as parse_ast;
use ast::*;
use crate::parse::ast::StatementKind;

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
            StatementKind::Enum { .. } => todo!(),
            StatementKind::Declaration { .. } => todo!(),
            StatementKind::Struct { .. } => todo!(),
            StatementKind::TypeAlias { .. } => todo!(),
            StatementKind::Use(_) => todo!(),
            StatementKind::RootUse(_) => todo!(),
            StatementKind::Module {
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
            StatementKind::Function { .. } => todo!()
        };
    }
}