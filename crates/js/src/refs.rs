type Size = u32;

#[derive(Copy, Clone)]
pub struct RefGenerator {
    /// Number of references generated
    count: Size,
}

impl RefGenerator {
    #[inline]
    pub const fn new() -> Self {
        Self {
            count: 0
        }
    }

    #[inline]
    pub const fn from_ref(r: Ref) -> Self {
        Self {
            count: r.raw
        }
    }
}

impl Iterator for RefGenerator {
    type Item = Ref;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count == !0_u32 {
            None
        } else {
            let res = Some(Ref::from_raw(self.count));
            self.count += 1;
            res
        }
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct Ref {
    pub raw: Size,
}

const ID_START_LOOKUP_LENGTH: usize = 54;
static ID_START_LOOKUP: [u8; ID_START_LOOKUP_LENGTH] = *b"_$abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

/// Maps a byte (char) to its numerical value (basically fast indexOf operation on the lookup).
static ID_START_REVERSE_LOOKUP: [u8; 256] = [
    //0 _1  _2  _3  _4  _5  _6  _7  _8  _9  _A  _B  _C  _D  _E  _F
    00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, // 0_
    00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, // 1_
    00, 00, 00, 00, 01, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, // 2_
    00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, // 3_
    00, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, // 4_
    43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 00, 00, 00, 00, 00, // 5_
    00, 02, 03, 04, 05, 06, 07, 08, 09, 10, 11, 12, 13, 14, 15, 16, // 6_
    17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 00, 00, 00, 00, 00, // 7_
    00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, // 8_
    00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, // 9_
    00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, // A_
    00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, // B_
    00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, // C_
    00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, // D_
    00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, // E_
    00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, // F_
];

const ID_CONTINUE_LOOKUP_LENGTH: usize = 64;
static ID_CONTINUE_LOOKUP: [u8; ID_CONTINUE_LOOKUP_LENGTH] = *b"_$abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

/// Maps a byte (char) to its numerical value (basically fast indexOf operation on the lookup).
static ID_CONTINUE_REVERSE_LOOKUP: [u8; 256] = [
    //0 _1  _2  _3  _4  _5  _6  _7  _8  _9  _A  _B  _C  _D  _E  _F
    00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, // 0_
    00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, // 1_
    00, 00, 00, 00, 01, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, // 2_
    54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 00, 00, 00, 00, 00, 00, // 3_
    00, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, // 4_
    43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 00, 00, 00, 00, 00, // 5_
    00, 02, 03, 04, 05, 06, 07, 08, 09, 10, 11, 12, 13, 14, 15, 16, // 6_
    17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 00, 00, 00, 00, 00, // 7_
    00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, // 8_
    00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, // 9_
    00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, // A_
    00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, // B_
    00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, // C_
    00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, // D_
    00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, // E_
    00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, // F_
];

/// Maximum buffer length to store the string representation of a [Ref]. Is always at least 1.
const MAX_LENGTH: usize = {
    let mut length = 1;
    let mut result = ID_START_LOOKUP_LENGTH;

    while result < Size::MAX as usize {
        result *= ID_CONTINUE_LOOKUP_LENGTH;
        length += 1;
    }

    length
};

// static CACHE: RwLock<Vec<[u8; MAX_LENGTH]>> = RwLock::new(Vec::new());

impl Ref {
    #[inline]
    pub const fn from_raw(n: Size) -> Self {
        Self {
            raw: n
        }
    }

    pub fn from_encoded_iter(mut iter: impl Iterator<Item=u8>) -> Self {
        let mut n = unsafe { *ID_START_REVERSE_LOOKUP.get_unchecked(iter.next().unwrap() as usize) } as u32;

        if let Some(next) = iter.next() {
            n = (n + 1) * ID_START_LOOKUP_LENGTH as u32;
            n += unsafe { *ID_CONTINUE_REVERSE_LOOKUP.get_unchecked(next as usize) } as u32;
        }

        for u in iter {
            n *= ID_CONTINUE_LOOKUP_LENGTH as u32;
            n += unsafe { *ID_CONTINUE_REVERSE_LOOKUP.get_unchecked(u as usize) } as u32;
        }

        Self { raw: n }
    }

    pub fn write_into(mut self, target: &mut Vec<u8>) {
        let mut buffer = [0_u8; MAX_LENGTH];
        let mut ptr = &mut buffer as *mut u8;

        macro_rules! push {
            ($byte:expr) => {unsafe {
                *ptr = $byte;
                ptr = ptr.add(1);
            }};
        }

        push!(*ID_START_LOOKUP.as_ptr().add((self.raw % ID_START_LOOKUP_LENGTH as u32) as usize));
        self.raw /= ID_START_LOOKUP_LENGTH as u32;

        while self.raw > 0 {
            push!(*ID_CONTINUE_LOOKUP.as_ptr().add((self.raw % (ID_CONTINUE_LOOKUP_LENGTH as u32 + 1) - 1) as usize));
            self.raw /= ID_CONTINUE_LOOKUP_LENGTH as u32 + 1;
        }

        let end = &buffer as *const u8;
        unsafe {
            let mut ptr = (&buffer as *const u8).add(MAX_LENGTH);

            while ptr != end {
                ptr = ptr.sub(1);

                match *ptr {
                    0 => {}
                    byte => target.push(byte)
                }
            }
        }
    }
}