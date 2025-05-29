use crate::parse::ast::Expression;
use bumpalo::Bump;
use bytes::Span;
use core::cell::RefCell;
use hashbrown::{DefaultHashBuilder, HashMap};

/// Maps identifiers (symbols) to items.
pub type SymbolTable<'sf, 'ast> =
    HashMap<&'sf str, &'ast RefCell<SymbolTableEntry<'sf, 'ast>>, DefaultHashBuilder, &'ast Bump>;

#[derive(Clone, Debug)]
pub struct SymbolTableEntry<'sf, 'ast> {
    // annotations: Vec<Annotation<'a>>,
    pub kind: SymbolTableEntryKind<'sf, 'ast>,
}

#[derive(Clone, Debug)]
pub enum SymbolTableEntryKind<'source, 'ast> {
    Struct {
        fields: HashMap<&'source str, StructField<'ast>, &'ast Bump>,
    },
    Module {
        st: SymbolTable<'source, 'ast>,
    },
    Enum {
        variants: HashMap<&'source str, EnumVariant<'source, 'ast>, &'ast Bump>,
    },
}

#[derive(Clone, Debug)]
pub struct EnumVariant<'source, 'ast> {
    value: Option<Span<Expression<'source, 'ast>>>,
}

#[derive(Clone, Debug)]
pub struct StructField<'ast> {
    pub is_mutable: bool,
    pub is_public: bool,
    pub ty: TypeUse<'ast>,
}

#[derive(Clone, Debug)]
pub enum TypeUse<'ast> {
    Never,
    Union {
        first: RawTypeUse<'ast>,
        // remaining: Vec<RawTypeUse<'ast>>
    },
}

#[derive(Clone, Debug)]
pub enum RawTypeUse<'ast> {
    Function,
    TypeRef { target: &'ast TypeUse<'ast> },
}
