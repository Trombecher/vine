use crate::parse::ast as parse_ast;
use ast::*;
use bumpalo::Bump;
use core::cell::RefCell;

pub mod ast;

pub fn index_module<'sf: 'resolve_arena + 'parse_arena, 'resolve_arena, 'parse_arena>(
    items: impl Iterator<Item=&'parse_arena parse_ast::TopLevelItem<'sf, 'parse_arena>>,
    resolve_arena: &'resolve_arena Bump,
    source_file_arena: &'sf Bump,
) -> SymbolTable<'sf, 'resolve_arena> {
    let mut symbol_table = SymbolTable::new_in(resolve_arena);

    for item in items {
        match &item.statement.value.statement_kind {
            parse_ast::StatementKind::Enum {
                doc_comments,
                id,
                tps,
                variants,
            } => symbol_table.insert(
                id,
                resolve_arena.alloc(RefCell::new(SymbolTableEntry {
                    kind: SymbolTableEntryKind::Enum {},
                })),
            ),
            parse_ast::StatementKind::Declaration { .. } => todo!(),
            parse_ast::StatementKind::Struct { .. } => todo!(),
            parse_ast::StatementKind::TypeAlias { .. } => todo!(),
            parse_ast::StatementKind::Use(_) => todo!(),
            parse_ast::StatementKind::RootUse(_) => todo!(),
            parse_ast::StatementKind::Module {
                content,
                id,
                doc_comments: _,
            } => {
                let inner_st = if let Some(content) = content {
                    index_module(content.0.iter(), resolve_arena, source_file_arena)
                } else {
                    todo!("external module")
                };

                let entry_ref = resolve_arena.alloc(RefCell::new(SymbolTableEntry {
                    kind: SymbolTableEntryKind::Module { st: inner_st },
                }));

                symbol_table.insert(id, &*entry_ref);
            }
            parse_ast::StatementKind::Function { .. } => todo!(),
        };
    }

    symbol_table
}
