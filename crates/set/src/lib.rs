use std::marker::PhantomData;

#[derive(Clone)]
pub struct Set256<T: Into<u8>>(u128, u128, PhantomData<T>);

impl<T: Into<u8>> Set256<T> {
    pub const fn empty() -> Self {
        Self(0, 0, PhantomData)
    }

    pub const fn full() -> Self {
        Self(u128::MAX, u128::MAX, PhantomData)
    }

    #[inline]
    fn parts(&self, t: T) -> (&u128, u128) {
        let byte: u8 = t.into();

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

    pub fn has(&self, element: T) -> bool {
        let (state, test) = self.parts(element);
        (*state & test) != 0
    }
}

impl<T: Into<u8>> std::ops::BitOr for Set256<T> {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0, self.1 | rhs.1, PhantomData)
    }
}

impl<T: Into<u8>> std::ops::BitOrAssign for Set256<T> {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
        self.1 |= rhs.1;
    }
}

impl<T: Into<u8>> std::ops::BitAnd for Set256<T> {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0, self.1 & rhs.1, PhantomData)
    }
}

impl<T: Into<u8>> std::ops::BitAndAssign for Set256<T> {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
        self.1 &= rhs.1;
    }
}

impl<T: Into<u8>> std::ops::BitXor for Set256<T> {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0, self.1 ^ rhs.1, PhantomData)
    }
}

impl<T: Into<u8>> std::ops::BitXorAssign for Set256<T> {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0;
        self.1 ^= rhs.1;
    }
}

impl<T: Into<u8>> std::ops::Add<T> for Set256<T> {
    type Output = Self;

    fn add(mut self, rhs: T) -> Self::Output {
        let (state, test) = self.parts_mut(rhs);
        *state |= test;
        self
    }
}

impl<T: Into<u8>> std::ops::AddAssign<T> for Set256<T> {
    fn add_assign(&mut self, rhs: T) {
        let (state, test) = self.parts_mut(rhs);
        *state |= test;
    }
}

impl<T: Into<u8>> std::ops::Sub<T> for Set256<T> {
    type Output = Self;

    fn sub(mut self, rhs: T) -> Self::Output {
        let (state, i) = self.parts_mut(rhs);
        *state ^= *state & i;
        self
    }
}

impl<T: Into<u8>> std::ops::SubAssign<T> for Set256<T> {
    fn sub_assign(&mut self, rhs: T) {
        let (state, i) = self.parts_mut(rhs);
        *state ^= *state & i;
    }
}