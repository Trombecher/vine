use frontend::parse::ast;
use span::{Index, Span};
use std::alloc::Allocator;
use std::cell::Cell;
use std::ops::Range;

use oxc_allocator::Allocator as OAllocator;
use oxc_allocator::Vec as OVec;
use oxc_allocator::Box as OBox;
use oxc_ast::ast as oast;
use oxc_ast::ast::Span as OSpan;

fn to_ospan(range: &Range<Index>) -> OSpan {
    OSpan::new(range.start, range.end)
}

pub trait Transform<'alloc, Into: 'alloc> {
    fn transform(&self, alloc: &'alloc OAllocator) -> Into;
}

impl<'alloc, A: Allocator> Transform<'alloc, oast::Program<'alloc>>
    for Span<&ast::ModuleContent<'_, A>>
{
    fn transform(&self, alloc: &'alloc OAllocator) -> oast::Program<'alloc> {
        oast::Program {
            span: to_ospan(&self.source),
            source_text: "", // TODO
            comments: OVec::new_in(alloc),
            hashbang: None,
            directives: OVec::new_in(alloc),
            body: {
                let mut body = OVec::new_in(alloc);

                for top_level_item in &self.value.0 {
                    body.push(
                        (Span {
                            source: top_level_item.source(),
                            value: top_level_item,
                        }).transform(alloc),
                    )
                }

                body
            },
            scope_id: Cell::new(None),
            source_type: Default::default(),
        }
    }
}

impl<'alloc, A: Allocator> Transform<'alloc, oast::Statement<'alloc>> for Span<&ast::TopLevelItem<'_, A>> {
    fn transform(&self, alloc: &'alloc OAllocator) -> oast::Statement<'alloc> {
        match &self.value.statement.statement_kind.value {
            ast::StatementKind::For { .. } => todo!(),
            ast::StatementKind::Type {
                id,
                ty_visibility,
                ty,
                ty_is_mutable,
                const_parameters
            } => todo!(),
            ast::StatementKind::Enum { .. } => todo!(),
            ast::StatementKind::Alias { .. } => todo!(),
            ast::StatementKind::Let { .. } => todo!(),
            ast::StatementKind::Function {
                pattern,
                const_parameters,
                body,
                id,
                output_type
            } => {
                oast::Statement::FunctionDeclaration(OBox::new_in(
                    oast::Function {
                        span: Default::default(),
                        id: None,
                        type_parameters: None,
                        this_param: None,
                        params: OBox::new_in(oast::FormalParameters {
                            span: Default::default(),
                            items: {
                                let mut params = OVec::new_in(alloc);
                                
                                params.push(oast::FormalParameter {
                                    span: to_ospan(&pattern.source()),
                                    decorators: OVec::new_in(alloc),
                                    pattern: oast::BindingPattern {
                                        kind: (),
                                        type_annotation: None,
                                        optional: false,
                                    },
                                    accessibility: None,
                                    readonly: false,
                                    r#override: false,
                                });
                                
                                params
                            },
                            rest: None,
                            kind: oast::FormalParameterKind::FormalParameter,
                        }, alloc),
                        return_type: None,
                        body: None,
                        scope_id: Cell::new(None),
                        r#type: oast::FunctionType::FunctionDeclaration,
                        generator: false,
                        r#async: false,
                        declare: false,
                        pure: false,
                    },
                    alloc
                ))
            }
            ast::StatementKind::Use(_) => todo!(),
            ast::StatementKind::RootUse(_) => todo!(),
            ast::StatementKind::Module { .. } => todo!(),
            ast::StatementKind::Impl { .. } => todo!(),
        }
    }
}