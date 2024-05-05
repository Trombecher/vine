pub mod instruction;
mod stack;

use std::iter::Copied;
use std::mem::{swap, transmute};
use std::ptr::slice_from_raw_parts;
use std::slice;
use crate::vm::instruction::Instruction;
use crate::vm::stack::Stack;

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum Value {
    Nil(u64) = 0,
    Boolean(bool),
    Number(f64),
    ReturnIndex(usize),
    Custom(u32, *mut ()),
}

impl Value {
    #[inline]
    pub fn get_u64(&self) -> Result<u64, Error> {
        match self {
            Value::Nil(x) => Ok(*x),
            Value::Boolean(value) => Ok(*value as u64),
            Value::Number(number) => Ok(*number as u64),
            _ => Err(Error::CannotCastToU64)
        }
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
    pub fn is_return_index(&self) -> bool {
        match self {
            Value::ReturnIndex(_) => true,
            _ => false
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::vm::Value;

    #[test]
    fn dj() {
        assert_eq!(2.0, Value::Number(2.0).get_f64_unchecked())
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

    /// A table containing static values.
    static_table: &'a [Value],

    stack: Stack
}

#[derive(Debug)]
pub enum Error {
    IllegalInstruction,
    BufferOverrun,
    StackOverflow,
    CannotCastToU64,
    BufferUnderrun,
    IndexOutOfStaticTable
}

#[derive(Debug)]
pub enum FileFormatError {
    UnexpectedEndOfInput,
    InvalidMagicBytes([u8; 8])
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
            return Err(FileFormatError::InvalidMagicBytes(magic_bytes))
        }

        let entry = read_usize!();
        let offset_table_length = read_usize!();
        let offset_table_pointer = unsafe { transmute::<_, &slice::Iter<'a, u8>>(&iter) }.as_slice().as_ptr();
        
        iter.advance_by(offset_table_length)
            .map_err(|_| FileFormatError::UnexpectedEndOfInput)?;
        
        Ok(Self {
            a: Value::Nil(0),
            b: Value::Nil(0),
            r: Value::Nil(0),
            code: unsafe { transmute::<_, &slice::Iter<'a, u8>>(&iter) }.as_slice(),
            next_byte: entry,
            offset_table: unsafe { transmute(slice_from_raw_parts(offset_table_pointer, offset_table_length)) },
            static_table: &[],
            stack: Stack::new(),
        })
    }

    #[inline]
    pub fn new(
        code: &'a [u8],
        entry: usize,
        offset_table: &'a [usize],
        static_table: &'a [Value],
    ) -> VM<'a> {
        Self {
            a: Value::Nil(0),
            b: Value::Nil(0),
            r: Value::Nil(0),
            code,
            next_byte: entry,
            offset_table,
            static_table,
            stack: Stack::new(),
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
    fn get_u32(&mut self) -> Result<u32, Error> {
        if self.next_byte + 3 < self.code.len() {
            let u32: u32 = unsafe {
                transmute(*(self.code.as_ptr().add(self.next_byte) as *const [u8; 4]))
            };
            self.next_byte += 4;
            Ok(u32)
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
    
    pub fn execute_next_instruction(&mut self) -> Result<Option<u64>, Error> {
        match self.get_next_instruction()? {
            Instruction::NoOperation => {}

            // A
            Instruction::DeclareABoolean => self.a = Value::Boolean(self.a.get_u64_unchecked() == 1),
            Instruction::DeclareANumber => self.a = Value::Number(self.a.get_f64_unchecked()),
            Instruction::DeclareANil => self.a = Value::Nil(self.a.get_u64_unchecked()),

            Instruction::LoadANil0 => self.a = Value::Nil(0),
            Instruction::LoadANil1 => self.a = Value::Nil(1),
            Instruction::LoadANumber0 => self.a = Value::Number(0.0),
            Instruction::LoadANumber1 => self.a = Value::Number(1.0),
            Instruction::LoadANilB1 => self.a = Value::Nil(self.get_u8()? as u64),
            Instruction::LoadANilB2 => self.a = Value::Nil(self.get_u16()? as u64),
            Instruction::LoadAStatic => {
                let index = self.get_u8()? as usize;
                if index >= self.static_table.len() {
                    return Err(Error::IndexOutOfStaticTable)
                }
                self.a = self.static_table[index];
            },

            // B
            Instruction::LoadBNilB1 => self.b = Value::Nil(self.get_u8()? as u64),
            Instruction::LoadBNilB2 => self.b = Value::Nil(self.get_u16()? as u64),
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
                    return Ok(Some(self.r.get_u64()?))
                }
            },
            Instruction::ReturnNil => {
                if let Some(return_index) = self.stack.clear_scope() {
                    self.r = Value::Nil(0);
                    self.next_byte = return_index;
                } else {
                    return Ok(Some(0))
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
            Instruction::SwapTopA => {
                if let Some(top) = self.stack.top_mut() {
                    swap(
                        // SAFETY: Yes, `self` is borrowed mutably via `top`,
                        // but only the stack is borrowed. So it is safe to
                        // borrow `a` mutably as well
                        unsafe { transmute(top) },
                        &mut self.a
                    )
                }
            }
            
            // Stack
            Instruction::PushA => self.stack.push(self.a.clone())?,
            Instruction::PushB => self.stack.push(self.b.clone())?,
            Instruction::PushR => self.stack.push(self.r.clone())?,

            Instruction::Pop => {
                if let Some(Value::ReturnIndex(_)) = self.stack.top() {
                } else {
                    self.stack.pop()
                }
            },
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
            Instruction::AddU64Unchecked => {
                self.r = Value::Nil(self.a.get_u64_unchecked() + self.b.get_u64_unchecked());
            }
            Instruction::AddF64Unchecked => {
                self.r = Value::Number(self.a.get_f64_unchecked() + self.b.get_f64_unchecked());
            }
            Instruction::InvokeNative => {
                match self.get_u8()? {
                    0 => {
                        println!("{:?}", self.r);
                    }
                    x => todo!("Native function {} not implemented", x)
                }
            }

            i => todo!("Instruction {i:?} not implemented")
        }
        
        Ok(None)
    }
}