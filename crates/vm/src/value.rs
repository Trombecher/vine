use crate::object::Object;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::mem::transmute;
use crate::GC;

pub enum PointerValue<'heap> {
    StrongRef(&'heap Object),
    WeakRef(*const Object, PhantomData<&'heap Object>),
    TypeOffset(u64),
}

pub enum PointerOrRaw<'heap> {
    Pointer(PointerValue<'heap>),
    Raw(u64),
}

/// A value.
///
/// This value is tagged:
///
/// - if the most significant bit is a one
///   - if the least significant bit is a one: then the value
///     is a zero-sized object (not allocated) and `(value << 1) >> 2` is the type.
///   - else the value is a non-zero-sized object (allocated) and `value << 1`
///     is a pointer to a `u8` (type pointer), indicating the type's length (value's field count).
/// - else the remaining bits contain arbitrary data, commonly `u63`s, `f32`s or `f64`s
#[derive(Clone, Copy)]
pub struct Value<'heap>(u64, PhantomData<&'heap Object>);

pub struct ValueDisplay<'input: 'heap, 'heap> {
    value: Value<'heap>,
    gc: &'heap GC<'input>
}

impl<'input: 'heap, 'heap> Debug for ValueDisplay<'input, 'heap> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.value.destructure() {
            PointerOrRaw::Pointer(PointerValue::StrongRef(object)) => {
                write!(f, "StrongRef({:?})", object.lock().display(self.gc))
            }
            PointerOrRaw::Pointer(PointerValue::WeakRef(weak_ref, _)) => {
                if let Some(strong_ref) = self.gc.upgrade(weak_ref) {
                    write!(f, "ValidWeakRef({:?})", strong_ref.lock().display(self.gc))
                } else {
                    f.write_str("ExpiredWeakRef")
                }
            }
            PointerOrRaw::Pointer(PointerValue::TypeOffset(type_index)) => {
                write!(f, "ZST({type_index})")
            }
            PointerOrRaw::Raw(raw) => {
                write!(f, "Raw({raw})")
            }
        }
    }
}

impl<'heap> Value<'heap> {
    pub fn display<'input: 'heap>(self, gc: &'heap GC<'input>) -> ValueDisplay<'input, 'heap> {
        ValueDisplay {
            value: self,
            gc,
        }
    }
    
    #[inline]
    pub const unsafe fn from_u64_unchecked(value: u64) -> Self {
        Self(value, PhantomData)
    }

    /// Constructs a new value from an offset into the type table.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the corresponding type entry specifies a length of zero.
    #[inline]
    pub const unsafe fn from_zero_sized(type_offset: u32) -> Self {
        // Enable pointer and zero-size flags.
        Self(((type_offset as u64) << 1) | 1 | (1 << 63), PhantomData)
    }

    /// Returns `true` if the stored value is a pointer.
    #[inline]
    pub const fn is_pointer(self) -> bool {
        self.0 >> 63 == 1
    }

    #[inline]
    pub fn destructure(self) -> PointerOrRaw<'heap> {
        if self.is_pointer() {
            PointerOrRaw::Pointer(if self.0 & 1 == 1 {
                PointerValue::TypeOffset((self.0 << 1) >> 2)
            } else {
                PointerValue::StrongRef(unsafe { transmute(self.0 << 1) })
            })
        } else {
            PointerOrRaw::Raw(self.0)
        }
    }
    
    #[inline]
    pub fn get_object(self) -> Option<&'heap Object> {
        if let PointerOrRaw::Pointer(PointerValue::StrongRef(obj)) = self.destructure() {
            Some(obj)
        } else {
            None
        }
    }

    #[inline]
    pub fn from_strong(strong_ref: &'heap Object) -> Self {
        Self((strong_ref as *const Object as usize as u64 >> 1) | (1_u64 << 63), PhantomData)
    }
}

macro_rules! impl_safe_u {
    ($t:ty) => {
        impl<'heap> From<$t> for Value<'heap> {
            #[inline]
            fn from(value: $t) -> Self {
                Self(value as u64, PhantomData)
            }
        }
        
        impl<'heap> TryInto<$t> for Value<'heap> {
            type Error = ();
            
            #[inline]
            fn try_into(self) -> Result<$t, Self::Error> {
                if self.is_pointer() {
                    Err(())
                } else {
                    Ok(self.0 as $t)
                }
            }
        }
    };
}

/*
macro_rules! impl_safe_s {
    ($t:ty) => {
        impl From<$t> for Value {
            #[inline]
            fn from(value: $t) -> Self {
                // Clear high bit
                Self(unsafe { transmute::<_, u64>(value as i64) } & !(1 << 63))
            }
        }
        
        impl TryInto<$t> for Value {
            #[inline]
            fn try_into(self) -> Result<$t, Self::Error> {
                if self.is_object() {
                    Err(())
                } else {
                    Ok((self.0 << 1).)
                }
            }
        }
    };
}
 */

impl_safe_u!(u8);
impl_safe_u!(u16);
impl_safe_u!(u32);

impl<'heap> From<f32> for Value<'heap> {
    fn from(value: f32) -> Self {
        Self(unsafe { transmute::<_, u32>(value) } as u64, PhantomData)
    }
}

impl<'heap> TryInto<f32> for Value<'heap> {
    type Error = ();

    #[inline]
    fn try_into(self) -> Result<f32, Self::Error> {
        if self.is_pointer() {
            Err(())
        } else {
            Ok(unsafe { transmute(self.0 as u32) })
        }
    }
}

impl<'heap> From<f64> for Value<'heap> {
    #[inline]
    fn from(value: f64) -> Self {
        unsafe {
            Self::from_u64_unchecked(transmute::<_, u64>(value) >> 1)
        }
    }
}

impl<'heap> From<u64> for Value<'heap> {
    fn from(value: u64) -> Self {
        unsafe {
            Self::from_u64_unchecked(value & !(1 << 63))
        }
    }
}

impl<'heap> TryInto<u64> for Value<'heap> {
    type Error = ();

    fn try_into(self) -> Result<u64, Self::Error> {
        if self.is_pointer() {
            Err(())
        } else {
            Ok(self.0)
        }
    }
}