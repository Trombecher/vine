mod tests;

use std::marker::PhantomData;

#[derive(Clone)]
pub struct Set256<T: Into<u8> + TryFrom<u8>>(u128, u128, PhantomData<T>);

pub struct IntoIter<T: Into<u8> + TryFrom<u8>> {
    index: u8,
    set: Set256<T>
}

impl<T: Into<u8> + TryFrom<u8>> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.index == 255 {
                break None
            }

            if self.set.has_raw(self.index) {
                let t = unsafe {
                    // SAFETY: `self.index` is a valid `T` because it was found in the set.
                    T::try_from(self.index).unwrap_unchecked()
                };

                self.index += 1;
                break Some(t)
            }

            self.index += 1;
        }
    }
}

impl<T: Into<u8> + TryFrom<u8>> IntoIterator for Set256<T>  {
    type Item = T;

    type IntoIter = crate::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter { index: 0, set: self }
    }
}

impl<T: Into<u8> + TryFrom<u8>> FromIterator<T> for Set256<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        iter.into_iter()
            .fold(Self::empty(), |set, item| set + item)
    }
}

impl<T: Into<u8> + TryFrom<u8>> Set256<T> {
    pub const fn empty() -> Self {
        Self(0, 0, PhantomData)
    }

    #[inline]
    fn parts(&self, byte: u8) -> (&u128, u128) {
        if byte > 0x7F {
            (&self.1, (2_u128 << byte.wrapping_sub(0x7F)))
        } else {
            (&self.0, (2_u128 << byte))
        }
    }

    #[inline]
    fn parts_mut(&mut self, t: T) -> (&mut u128, u128) {
        let byte: u8 = t.into();

        if byte > 0x7F {
            (&mut self.1, (2_u128 << byte.wrapping_sub(0x7F)))
        } else {
            (&mut self.0, (2_u128 << byte))
        }
    }

    #[inline]
    pub fn has(&self, element: T) -> bool {
        self.has_raw(element.into())
    }

    #[inline]
    fn has_raw(&self, byte: u8) -> bool {
        let (state, test) = self.parts(byte);
        (*state & test) != 0
    }

    /// Iterate through the items of this set.
    #[inline]
    pub fn iter<'a>(&'a self) -> impl Iterator<Item = T> + 'a {
        struct Iter<'a, T: Into<u8> + TryFrom<u8>> {
            index: u8,
            set: &'a Set256<T>
        }

        impl<'a, T: Into<u8> + TryFrom<u8>> Iterator for Iter<'a, T> {
            type Item = T;
        
            fn next(&mut self) -> Option<Self::Item> {
                loop {
                    if self.index == 255 {
                        break None
                    }

                    if self.set.has_raw(self.index) {
                        let t = unsafe {
                            // SAFETY: `self.index` is a valid `T` because it was found in the set.
                            T::try_from(self.index).unwrap_unchecked()
                        };

                        self.index += 1;
                        break Some(t)
                    }

                    self.index += 1;
                }
            }
        }

        Iter {
            index: 0,
            set: self,
        }
    }
}

impl<T: Into<u8> + TryFrom<u8>> std::ops::BitOr for Set256<T> {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0, self.1 | rhs.1, PhantomData)
    }
}

impl<T: Into<u8> + TryFrom<u8>> std::ops::BitOrAssign for Set256<T> {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
        self.1 |= rhs.1;
    }
}

impl<T: Into<u8> + TryFrom<u8>> std::ops::BitAnd for Set256<T> {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0, self.1 & rhs.1, PhantomData)
    }
}

impl<T: Into<u8> + TryFrom<u8>> std::ops::BitAndAssign for Set256<T> {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
        self.1 &= rhs.1;
    }
}

impl<T: Into<u8> + TryFrom<u8>> std::ops::BitXor for Set256<T> {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0, self.1 ^ rhs.1, PhantomData)
    }
}

impl<T: Into<u8> + TryFrom<u8>> std::ops::BitXorAssign for Set256<T> {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0;
        self.1 ^= rhs.1;
    }
}

impl<T: Into<u8> + TryFrom<u8>> std::ops::Add<T> for Set256<T> {
    type Output = Self;

    fn add(mut self, rhs: T) -> Self::Output {
        let (state, test) = self.parts_mut(rhs);
        *state |= test;
        self
    }
}

impl<T: Into<u8> + TryFrom<u8>> std::ops::AddAssign<T> for Set256<T> {
    fn add_assign(&mut self, rhs: T) {
        let (state, test) = self.parts_mut(rhs);
        *state |= test;
    }
}

impl<T: Into<u8> + TryFrom<u8>> std::ops::Sub<T> for Set256<T> {
    type Output = Self;

    fn sub(mut self, rhs: T) -> Self::Output {
        let (state, i) = self.parts_mut(rhs);
        *state ^= *state & i;
        self
    }
}

impl<T: Into<u8> + TryFrom<u8>> std::ops::SubAssign<T> for Set256<T> {
    fn sub_assign(&mut self, rhs: T) {
        let (state, i) = self.parts_mut(rhs);
        *state ^= *state & i;
    }
}