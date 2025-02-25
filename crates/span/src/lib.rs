#![no_std]
#![feature(new_range_api)]

use core::range::Range;
use core::fmt::Debug;

#[cfg(feature = "huge_files")]
pub type Index = u64;

#[cfg(not(feature = "huge_files"))]
pub type Index = u32;

/// Links the value back to a view of the source file.
#[derive(Debug, PartialEq, Clone)]
pub struct Span<T: Debug + PartialEq + Clone> {
    pub value: T,
    pub source: Range<Index>,
}

impl<'a, T: Debug + Clone + PartialEq> Span<T> {
    #[inline]
    pub fn map<U: Debug + Clone + PartialEq>(self, map: impl FnOnce(T) -> U) -> Span<U> {
        Span {
            value: map(self.value),
            source: self.source
        }
    }
}