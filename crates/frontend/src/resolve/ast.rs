use std::cell::RefCell;
use bytes::Span;
use hashbrown::HashMap;

/// Maps identifiers (symbols) to items.
pub type SymbolTable<'sf, 'arena> = HashMap<
    &'sf str,
    &'arena RefCell<SymbolTableEntry<'sf, 'arena>>,
>;

pub struct SymbolTableEntry<'sf, 'arena> {
    // annotations: Vec<Annotation<'a>>,
    pub kind: SymbolTableEntryKind<'sf, 'arena>,
}

pub enum SymbolTableEntryKind<'sf, 'arena> {
    Struct {
        tps: Vec<()>,
        fields: Vec<Span<StructField<'arena>>>
    },
    Module {
        st: SymbolTable<'sf, 'arena>,
    },
    TypeParameter {
        // instanced_type:
    }
}

#[derive(Clone, Debug)]
pub struct StructField<'arena> {
    pub is_mutable: bool,
    pub is_public: bool,
    pub ty: TypeUse<'arena>,
}

#[derive(Clone, Debug)]
pub enum TypeUse<'arena> {
    Never,
    Union {
        first: RawTypeUse<'arena>,
        remaining: Vec<RawTypeUse<'arena>>
    }
}

#[derive(Clone, Debug)]
pub enum RawTypeUse<'arena> {
    Function,
    TypeRef {
        target: &'arena TypeUse<'arena>
    }
}