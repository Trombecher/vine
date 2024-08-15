#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Warning {
    UnnecessarySemicolon,
    UnnecessaryComma
}

impl Warning {
    #[inline]
    pub const fn is_extendable(self) -> bool {
        match self {
            Warning::UnnecessaryComma | Warning::UnnecessarySemicolon => true,
            _ => false,
        }
    }
}