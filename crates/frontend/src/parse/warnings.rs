#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Warning {
    UnnecessarySemicolon,
    UnnecessaryComma,

    /// This warning is emitted when a `pub(mod)` modifier is used on a top-level item in a module,
    /// as it is redundant and does not affect visibility.
    PubModInModule,
}

impl Warning {
    #[inline]
    pub const fn is_extendable(self) -> bool {
        match self {
            Warning::UnnecessaryComma | Warning::UnnecessarySemicolon => true,
            Warning::PubModInModule => false,
        }
    }
}
