//! # Vine Virtual Machine (VVM).
//!
//! This is the implementation of the vine virtual machine. It is operating on byte code.
//!
//! ## Safety
//!
//! Byte code should never be able to do unsafe operations.
//!
//! ## FFI
//!
//! Vine should integrate seamlessly with existing languages, like JavaScript, TypeScript or C.
//! All those languages should be able to compile to the VVM.
//!
//! This means that developers can use JavaScript and TypeScript libraries while developing in Vine.

pub mod instruction;
mod stack;
mod tests;

use std::alloc::{alloc, dealloc, Layout};
use std::collections::HashSet;
use std::iter::Copied;
use std::mem::{swap, transmute};
use std::ptr::{NonNull, slice_from_raw_parts};
use std::slice;
use crate::vm::instruction::Instruction;
use crate::vm::stack::Stack;

const NUMBER_TYPE: *const u64 = &0_u64 as *const u64;
const NIL_TYPE: *const u64 = 0_usize as *const u64;
const ARRAY_TYPE: *const u64 = &0_u64 as *const u64;

#[derive(Copy, Clone)]
union ValueValue {
    nil: u64,
    number: f64,
    ptr: NonNull<Object>,
}

#[repr(align(8))]
struct Object {
    is_used: bool,
    // here are some hidden values
}

impl Object {
    #[inline]
    pub fn layout(len: u64) -> Layout {
        Layout::array::<u64>(len as usize * 2 + 1).expect("Layout creation has failed")
    }
}

#[derive(Clone, Copy)]
pub struct Value(*const u64, ValueValue);

impl Value {
    #[inline]
    pub const fn nil() -> Self {
        Self(NIL_TYPE, unsafe { transmute(0_u64) })
    }

    #[inline]
    pub const fn nil_value(value: u64) -> Self {
        Self(NIL_TYPE, unsafe { transmute(value) })
    }

    #[inline]
    pub const fn number(value: f64) -> Self {
        Self(NUMBER_TYPE, unsafe { transmute(value) })
    }

    #[inline]
    pub fn cast_number(self) -> Self {
        Self(NUMBER_TYPE, match self.0 {
            NUMBER_TYPE => self.1,
            NIL_TYPE => unsafe { transmute(self.1.nil as f64) }
            _ => unsafe { transmute(f64::NAN) }
        })
    }

    #[inline]
    pub fn get_u64_unchecked(&self) -> u64 {
        unsafe { transmute::<_, &(u64, u64)>(self) }.1
    }

    #[inline]
    pub fn get_f64_unchecked(&self) -> f64 {
        unsafe { transmute::<_, &(u64, f64)>(self) }.1
    }

    #[inline]
    pub fn get_object(&self) -> Option<NonNull<Object>> {
        match self.0 {
            NUMBER_TYPE | NUMBER_TYPE => None,
            _ => Some(unsafe { self.1.ptr }),
        }
    }
}

pub struct VM<'a> {
    /// General purpose register
    a: Value,

    /// General purpose register
    b: Value,

    /// Return value register
    r: Value,

    /// Executable code
    code: &'a [u8],

    /// Instruction/Byte pointer
    next_byte: usize,

    /// A table containing u64 offsets for functions, locations, etc.
    offset_table: &'a [usize],
    
    /// A table containing a series of lengths. The pointers to those lengths are the types.
    type_table: &'a [u64],

    /// A table containing static values.
    static_table: &'a [Value],

    /// A stack for maintaining values.
    stack: Stack,

    allocated_objects: HashSet<NonNull<Object>>,
}

#[derive(Debug)]
pub enum Error {
    IllegalInstruction,
    BufferOverrun,
    StackOverflow,
    CannotCastToU64,
    BufferUnderrun,
    IndexOutOfStaticTable,
}

#[derive(Debug)]
pub enum FileFormatError {
    UnexpectedEndOfInput,
    InvalidMagicBytes([u8; 8]),
}

impl<'a> VM<'a> {
    pub fn from(mut iter: Copied<slice::Iter<'a, u8>>) -> Result<VM<'a>, FileFormatError> {
        let magic_bytes = iter.next_chunk::<8>()
            .map_err(|_| FileFormatError::UnexpectedEndOfInput)?;

        macro_rules! read_usize {
            () => {
                unsafe {
                    transmute::<_, usize>(iter.next_chunk::<8>()
                        .map_err(|_| FileFormatError::UnexpectedEndOfInput)?)
                }
            };
        }

        if &magic_bytes != b"VVMM\0\0\0\0" {
            return Err(FileFormatError::InvalidMagicBytes(magic_bytes));
        }

        let entry = read_usize!();
        let offset_table_length = read_usize!();
        let offset_table_pointer = unsafe { transmute::<_, &slice::Iter<'a, u8>>(&iter) }.as_slice().as_ptr();

        iter.advance_by(offset_table_length)
            .map_err(|_| FileFormatError::UnexpectedEndOfInput)?;

        Ok(Self {
            a: Value::nil(),
            b: Value::nil(),
            r: Value::nil(),
            code: unsafe { transmute::<_, &slice::Iter<'a, u8>>(&iter) }.as_slice(),
            next_byte: entry,
            offset_table: unsafe { transmute(slice_from_raw_parts(offset_table_pointer, offset_table_length)) },
            type_table: &[],
            static_table: &[],
            stack: Stack::new(),
            allocated_objects: Default::default(),
        })
    }

    #[inline]
    pub fn new(
        code: &'a [u8],
        entry: usize,
        offset_table: &'a [usize],
        static_table: &'a [Value],
        type_table: &'a [u64]
    ) -> VM<'a> {
        Self {
            a: Value::nil(),
            b: Value::nil(),
            r: Value::nil(),
            code,
            next_byte: entry,
            offset_table,
            static_table,
            stack: Stack::new(),
            allocated_objects: Default::default(),
            type_table,
        }
    }

    #[inline]
    fn get_next_instruction(&mut self) -> Result<Instruction, Error> {
        self.get_u8().and_then(Instruction::try_from)
    }

    #[inline]
    fn get_u8(&mut self) -> Result<u8, Error> {
        if self.next_byte < self.code.len() {
            let byte = unsafe { *self.code.get_unchecked(self.next_byte) };
            self.next_byte += 1;
            Ok(byte)
        } else {
            Err(Error::BufferOverrun)
        }
    }

    #[inline]
    fn get_u16(&mut self) -> Result<u16, Error> {
        if self.next_byte + 1 < self.code.len() {
            let u16: u16 = unsafe {
                transmute(*(self.code.get_unchecked(self.next_byte) as *const u8 as *const (u8, u8)))
            };
            self.next_byte += 2;
            Ok(u16)
        } else {
            Err(Error::BufferOverrun)
        }
    }

    #[inline]
    fn get_u64(&mut self) -> Result<u64, Error> {
        if self.next_byte + 7 < self.code.len() {
            let num: u64 = unsafe {
                transmute(*(self.code.as_ptr().add(self.next_byte) as *const [u8; 8]))
            };
            self.next_byte += 4;
            Ok(num)
        } else {
            Err(Error::BufferOverrun)
        }
    }

    #[inline]
    fn get_i8(&mut self) -> Result<i8, Error> {
        unsafe { transmute(self.get_u8()) }
    }

    #[inline]
    fn get_i16(&mut self) -> Result<i16, Error> {
        unsafe { transmute(self.get_u16()) }
    }

    #[inline]
    pub fn execute(&mut self) -> Result<u64, Error> {
        loop {
            match self.execute_next_instruction()? {
                None => {}
                Some(exit_code) => return Ok(exit_code)
            }
        }
    }

    /// Allocates an object with type `ty`.
    /// 
    /// # Safety
    ///
    /// The caller must ensure that `ty` is a valid pointer to an object type.
    pub unsafe fn alloc(&mut self, ty: *const u64) -> Value {
        // Allocate an object with *ty fields.
        let ptr = NonNull::new(unsafe {
            alloc(Object::layout(*ty))
        } as *mut Object).expect("Allocation has produces a null pointer (bad)");

        self.allocated_objects.insert(ptr);
        Value(ty, unsafe { transmute(ptr) })
    }

    pub fn run_gc(&mut self) {
        macro_rules! mark {
            ($target:expr) => {
                if let Some(ptr) = $target.get_object() {
                    unsafe {
                        (*ptr.as_ptr()).is_used = true;
                    }
                }
            };
        }

        mark!(self.a);
        mark!(self.b);
        mark!(self.r);

        for value in self.static_table.iter().copied() {
            mark!(value);
        }

        for value in self.stack.as_slice().into_iter().copied() {
            mark!(value);
        }

        self.allocated_objects.retain(|obj| {
            if unsafe {
                !(*obj.as_ptr()).is_used
            } {
                // Object is unused because it was not marked as used.

                unsafe {
                    dealloc(
                        transmute(obj.as_ptr()),
                        Layout::new::<Object>(),
                    )
                }

                false
            } else {
                // Object is still in use.

                unsafe {
                    (*obj.as_ptr()).is_used = false
                };

                true
            }
        });
    }

    pub fn execute_next_instruction(&mut self) -> Result<Option<u64>, Error> {
        match self.get_next_instruction()? {
            Instruction::NoOperation => {}

            // A
            Instruction::DeclareANumber => self.a.0 = NUMBER_TYPE,
            Instruction::DeclareANil => self.a.0 = NIL_TYPE,

            Instruction::LoadANil0 => self.a = Value::nil(),
            Instruction::LoadANil1 => self.a = Value::nil_value(1),
            Instruction::LoadANumber0 => self.a = Value::number(0.0),
            Instruction::LoadANumber1 => self.a = Value::number(1.0),
            Instruction::LoadANilB1 => self.a = Value::nil_value(self.get_u8()? as u64),
            Instruction::LoadANilB2 => self.a = Value::nil_value(self.get_u16()? as u64),
            Instruction::LoadAStatic => {
                let index = self.get_u8()? as usize;
                if index >= self.static_table.len() {
                    return Err(Error::IndexOutOfStaticTable);
                }
                self.a = self.static_table[index];
            }

            // B
            Instruction::LoadBNilB1 => self.b = Value::nil_value(self.get_u8()? as u64),
            Instruction::LoadBNilB2 => self.b = Value::nil_value(self.get_u16()? as u64),
            Instruction::LoadBStatic => self.b = self.static_table[self.get_u8()? as usize],

            // Swap
            Instruction::SwapAB => swap(&mut self.a, &mut self.b),
            Instruction::SwapAR => swap(&mut self.a, &mut self.r),
            Instruction::SwapBR => swap(&mut self.b, &mut self.r),

            // Control flow
            Instruction::JumpU8 => self.next_byte = self.get_u8()? as usize,
            Instruction::JumpU16 => self.next_byte = self.get_u16()? as usize,
            Instruction::JumpOffsetI8 => self.next_byte = self.next_byte.saturating_add_signed(self.get_u8()? as isize),
            Instruction::JumpOffsetI16 => self.next_byte = self.next_byte.saturating_add_signed(self.get_u16()? as isize),

            /*
            Instruction::InvokeNStatic => {
                let n = self.get_u8()? as usize;
                let jump_to = self.offset_table[self.get_u8()? as usize];

                // Push the return index to the stack.
                self.stack.push(Value::ReturnIndex(self.next_byte))?;

                // Get a pointer to the first element to copy.
                let first_value_to_copy = match self.stack.top_offset_ptr(n) {
                    None => return Err(Error::BufferUnderrun),
                    Some(x) => x
                };

                // Allocate space for the target destination.
                let space = self.stack.preallocate(n)?;
                unsafe {
                    // Copy all items to this new location.
                    space.clone_from_slice(&*slice_from_raw_parts(transmute(first_value_to_copy), n));
                }

                self.next_byte = jump_to;
            }
            
            Instruction::Return => {
                if let Some(return_index) = self.stack.clear_scope() {
                    self.next_byte = return_index;
                } else {
                    return Ok(Some(self.r.get_u64()?));
                }
            }

            Instruction::ReturnNil => {
                if let Some(return_index) = self.stack.clear_scope() {
                    self.r = Value::nil();
                    self.next_byte = return_index;
                } else {
                    return Ok(Some(0));
                }
            }
            Instruction::SwapTopBelow => {
                if let Some(below) = self.stack.top_offset_ptr_mut(1) {
                    unsafe {
                        // SAFETY: A successful call to `top_offset_ptr_mut(1)`
                        // guarantees a minimum stack size of 2 in which case
                        // there is always a top value.
                        swap(self.stack.top_mut().unwrap_unchecked(), &mut *below)
                    };
                }
            }
             */
            
            Instruction::SwapTopA => {
                let Self { stack, a, .. } = self;
                if let Some(top) = stack.top_mut() {
                    swap(
                        top,
                        a,
                    )
                }
            }

            // Stack
            Instruction::PushA => self.stack.push(self.a)?,
            Instruction::PushB => self.stack.push(self.b)?,
            Instruction::PushR => self.stack.push(self.r)?,

            /*
            Instruction::Pop => {
                if let Some(Value::ReturnIndex(_)) = self.stack.top() {} else {
                    self.stack.pop()
                }
            }
            */
            
            Instruction::PopIntoA => if let Some(value) = self.stack.pop_get() {
                self.a = value;
            }
            Instruction::PopIntoB => if let Some(value) = self.stack.pop_get() {
                self.b = value;
            }
            Instruction::PopIntoR => if let Some(value) = self.stack.pop_get() {
                self.r = value;
            }

            // Operations
            /*
            Instruction::AddU64Unchecked => {
                self.r = Value::Nil(self.a.get_u64_unchecked() + self.b.get_u64_unchecked());
            }
            Instruction::AddF64Unchecked => {
                self.r = Value::Number(self.a.get_f64_unchecked() + self.b.get_f64_unchecked());
            }
             */

            // Objects
            
            Instruction::CreateObject => {
                let ty = &self.type_table[self.get_u64()? as usize] as *const u64;
                self.a = unsafe { self.alloc(ty) };
            }
            Instruction::CreateObjectOffset => {
                let ty = &self.type_table[self.offset_table[self.get_u8()? as usize]] as *const u64;
                self.a = unsafe { self.alloc(ty) };
            }
            
            i => todo!("Instruction {i:?} not implemented")
        }

        Ok(None)
    }
}