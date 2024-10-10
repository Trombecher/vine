use core::cell::RefCell;
use bumpalo::Bump;
use hashbrown::hash_map::DefaultHashBuilder;
use hashbrown::HashMap;

/// Maps identifiers (symbols) to items.
pub type SymbolTable<'sf, 'arena> = HashMap<
    &'sf str,
    &'arena RefCell<SymbolTableEntry<'sf, 'arena>>,
    DefaultHashBuilder,
    &'arena Bump
>;

#[derive(Clone, Debug)]
pub struct SymbolTableEntry<'sf, 'arena> {
    // annotations: Vec<Annotation<'a>>,
    pub kind: SymbolTableEntryKind<'sf, 'arena>,
}

#[derive(Clone, Debug)]
pub enum SymbolTableEntryKind<'sf, 'arena> {
    Struct {
        fields: HashMap<&'sf str, StructField<'arena>, &'arena Bump>
    },
    Module {
        st: SymbolTable<'sf, 'arena>,
    },
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
        // remaining: Vec<RawTypeUse<'arena>>
    }
}

#[derive(Clone, Debug)]
pub enum RawTypeUse<'arena> {
    Function,
    TypeRef {
        target: &'arena TypeUse<'arena>
    }
}