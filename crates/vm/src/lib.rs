#![feature(let_chains)]
#![feature(new_uninit)]
#![feature(ptr_as_ref_unchecked)]

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
pub mod stack;
mod tests;
mod value;
mod object;
mod gc;

pub use gc::*;
pub use object::*;
pub use value::*;

use crate::instruction::Instruction;
use crate::stack::Stack;
use std::mem::{swap, transmute};

pub struct VM<'input: 'heap, 'heap, const MAX_STACK_SIZE: usize> {
    /// General purpose register
    a: Value<'heap>,

    /// General purpose register
    b: Value<'heap>,

    /// Return value register
    r: Value<'heap>,

    /// Executable code
    code: &'input [u8],

    /// Instruction/Byte pointer
    next_byte: usize,

    /// A table containing u64 offsets for functions, locations, etc.
    offset_table: &'input [u64],

    /// A table containing static values.
    static_table: &'input [Value<'heap>],

    /// A stack for maintaining values.
    stack: Stack<'heap, MAX_STACK_SIZE>,

    /// The garbage collector.
    gc: &'heap GC<'input>,
}

#[derive(Debug, Copy, Clone)]
pub enum Error {
    IllegalInstruction,
    BufferOverrun,
    StackOverflow,
    CannotCastToU64,
    BufferUnderrun,
    InvalidTypeIndex,
    IndexOutOfStaticTable,
}

#[derive(Debug)]
pub enum FileFormatError {
    UnexpectedEndOfInput,
    InvalidMagicBytes([u8; 8]),
}

impl<'input: 'heap, 'heap, const MAX_STACK_SIZE: usize> VM<'input, 'heap, MAX_STACK_SIZE> {
    /* 
    pub fn from(mut iter: Copied<slice::Iter<'a, u8>>) -> Result<VM<'a, MAX_STACK_SIZE>, FileFormatError> {
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
    }*/

    #[inline]
    pub fn new(
        code: &'input [u8],
        entry: usize,
        offset_table: &'input [u64],
        static_table: &'input [Value],
        gc: &'heap GC,
    ) -> VM<'input, 'heap, MAX_STACK_SIZE> {
        Self {
            a: 0u32.into(),
            b: 0u32.into(),
            r: 0u32.into(),
            code,
            next_byte: entry,
            offset_table,
            static_table,
            stack: Stack::new(),
            gc,
        }
    }

    #[inline]
    fn get_next_instruction(&mut self) -> Result<Instruction, Error> {
        self.get_u8().and_then(Instruction::try_from)
    }

    #[inline]
    fn get_u8(&mut self) -> Result<u8, Error> {
        self.code.get(self.next_byte)
            .map(|x| {
                self.next_byte += 1;
                *x
            })
            .ok_or(Error::BufferOverrun)
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
    fn get_u32(&mut self) -> Result<u32, Error> {
        if self.next_byte + 3 < self.code.len() {
            let u32: u32 = unsafe {
                (self.code.as_ptr().add(self.next_byte) as *const u32).read_unaligned()
            };
            self.next_byte += 4;
            Ok(u32)
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
            self.next_byte += 8;
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
    pub fn execute(&mut self) -> Result<(), Error> {
        loop {
            if self.execute_next_instruction()? {
                break;
            }
        }

        Ok(())
    }

    /*
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
    }*/

    #[inline]
    pub fn execute_next_instruction(&mut self) -> Result<bool, Error> {
        match self.get_next_instruction()? {
            Instruction::NoOperation => {}

            Instruction::Return => return Ok(true),

            // A
            Instruction::LoadA0Int => self.a = 0_u8.into(),
            Instruction::LoadA1Int => self.a = 1_u8.into(),
            Instruction::LoadA0F32 => self.a = 0_f32.into(),

            Instruction::LoadAImmediate1 => self.a = self.get_u8()?.into(),
            Instruction::LoadAImmediate2 => self.a = self.get_u16()?.into(),
            Instruction::LoadAImmediate4 => self.a = self.get_u32()?.into(),
            Instruction::LoadAImmediate8 => self.a = self.get_u64()?.into(),

            Instruction::LoadAStatic => {
                let index = self.get_u8()? as usize;
                if index >= self.static_table.len() {
                    return Err(Error::IndexOutOfStaticTable);
                }
                self.a = self.static_table[index];
            }

            // Swap
            Instruction::SwapAB => swap(&mut self.a, &mut self.b),
            Instruction::SpreadAB => self.b = self.a,
            Instruction::SwapAR => swap(&mut self.a, &mut self.r),
            Instruction::SpreadAR => self.r = self.a,
            Instruction::SwapBR => swap(&mut self.b, &mut self.r),
            Instruction::SpreadBR => self.r = self.b,

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
            Instruction::Pop => self.stack.pop(),

            Instruction::AddU63 => {
                if let Ok(a) = <Value as TryInto<u64>>::try_into(self.a)
                    && let Ok(b) = <Value as TryInto<u64>>::try_into(self.b) {
                    self.a = (a + b).into();
                }
            }

            // Objects

            Instruction::CreateObject1 => {
                let type_index = self.get_u8()?;
                self.a = Value::from_strong(self.gc.allocate(type_index as u32));
            }

            /*
            Instruction::CreateObjectOffset => {
                let ty = &self.type_table[self.offset_table[self.get_u8()? as usize] as usize] as *const u8;
                if (ty as usize as u64) < 2 {
                    panic!("Hmmm");
                }
                self.a = unsafe { self.alloc(ty) };
            }
             */

            Instruction::ReadProperty0 => {
                if let Some(object_ref) = self.b.get_object() {
                    unsafe {
                        // SAFETY: object_refs have a non-zero size.
                        self.a = *object_ref.lock().get_unchecked(0);
                    }
                } else {
                    todo!("no object in b")
                }
            }
            Instruction::WriteProperty0 => {
                if let Some(object_ref) = self.b.get_object() {
                    unsafe {
                        // SAFETY: object_refs have a non-zero size.
                        *object_ref.lock().get_unchecked_mut(0) = self.a;
                    }
                } else {
                    todo!("no object in b")
                }
            }

            // STDIO

            Instruction::WriteStdoutLF => {
                println!("{:?}", self.a.display(self.gc));
            }
            Instruction::DebugPrintAllocatedObjects => {
                println!("# Objects allocated: {:?}", self.gc.count());
            }
            Instruction::DebugTriggerGC => {
                self.gc.mark_and_sweep(
                    self.stack.as_slice()
                        .iter()
                        .copied()
                        .chain([self.a, self.b, self.r]
                            .iter()
                            .copied())
                );
            }

            i => todo!("Instruction {i:?} not implemented")
        }

        Ok(false)
    }
}