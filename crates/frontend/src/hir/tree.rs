use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::alloc::Allocator;
use derive_where::derive_where;
use span::Span;

pub enum Type {
    Derived(Arc<DerivedType>),
    U8,
    U16,
    U32,
    U64,
    U128,
    I8,
    I16,
    I32,
    I64,
    I128,
    F32,
    F64,
    Ref(Box<Type>),
    RefMut(Box<Type>)
}

pub struct DerivedType {
    inner: Type,
}

pub struct Function {
    
}

#[derive(Clone)]
#[derive_where(Debug, PartialEq)]
pub enum Expression<'source, A: Allocator> {
    Number(f64),
    
}