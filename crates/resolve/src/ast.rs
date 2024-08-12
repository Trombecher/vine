use std::cell::{Ref, RefCell};
use parse::ast::TypeParameters;

pub enum Item<'a> {
    Enum {
        tps: TypeParameters<'a>,
    }
}

pub enum Expression<'a> {
    Reference(Ref<'a, Item<'a>>)
}