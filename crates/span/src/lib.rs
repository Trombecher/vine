#![no_std]

use core::fmt::{Debug, Formatter};
use core::ops::Range;

#[cfg(feature = "huge_files")]
pub type Index = u64;

#[cfg(not(feature = "huge_files"))]
pub type Index = u32;

/// Links the value back to a view of the source file.
pub struct Span<T> {
    pub value: T,
    pub source: Range<Index>,
}

impl<T: Debug> Debug for Span<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Span")
            .field("value", &self.value)
            .field("source", &self.source)
            .finish()
    }
}

impl<T: Clone> Clone for Span<T>  {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            source: self.source.clone(),
        }
    }
}

impl<T: PartialEq> PartialEq for Span<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value && self.source == other.source
    }
}

impl<'a, T> Span<T> {
    #[inline]
    pub fn map<U>(self, map: impl FnOnce(T) -> U) -> Span<U> {
        Span {
            value: map(self.value),
            source: self.source,
        }
    }
}
