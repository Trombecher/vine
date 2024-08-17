use std::mem::transmute;

use crate::Error;

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum Instruction {
    Unreachable = 0x00,
    NoOperation,

    // Control flow
    JumpOffsetI8,
    JumpOffsetI16,
    JumpU8,
    JumpU16,
    JumpU32,
    JumpU64,
    JumpStatic,
    JumpU8IfUncheckedZero,
    
    /// Invokes a native function. The id is specified via the next byte.
    InvokeNative,
    Invoke0,
    Invoke1,
    Invoke2,
    Invoke3,
    Invoke4,
    
    /// Pushes [Value::ReturnIndex] to the stack, duplicates n values from
    /// below the [Value::ReturnIndex] and finally jumps to the address at the offset entry.
    InvokeNStatic,
    
    /// Replaces the nth value from the top of the stack with a [Value::ReturnIndex].
    InvokePopN,

    /// Returns from the function, or the program if there is no outer function.
    Return,

    // --- Registers ---
    
    // A
    
    LoadA,
    
    /// Loads 0 (int) into A.
    LoadA0Int,

    /// Loads 1 (int) into A.
    LoadA1Int,

    /// Loads 0 (f32) into A.
    LoadA0F32,

    /// Loads 0 (f32) into A.
    LoadA1F32,

    /// Loads 0 (f64) into A.
    LoadA0F64,

    /// Loads 1 (f64) into A.
    LoadA1F64,

    /// Loads the next byte into A, the rest is zeroed.
    LoadAImmediate1,

    /// Loads the next two bytes into A, the rest is zeroed.
    LoadAImmediate2,

    /// Loads the next four bytes into A, the rest is zeroed.
    LoadAImmediate4,

    /// Loads the next eight bytes shr by one into A. The high bit is zero.
    LoadAImmediate8,

    /// Loads the entry from the static table into A, with the next byte being the index.
    LoadAStatic,

    SwapAB,
    SpreadAB,
    SwapAR,
    SpreadAR,
    SwapBR,
    SpreadBR,

    // Operations
    
    // Stack

    PushA,
    PushB,
    PushR,

    /// Removes the top item from the stack if there is one.
    Pop,
    
    /// Clears the stack until it is empty or a return address has been reached.
    Clear,

    /// Shortcut for [Instruction::LoadTopIntoA] following [Instruction::Pop].
    PopIntoA,

    /// Shortcut for [Instruction::LoadTopIntoB] following [Instruction::Pop].
    PopIntoB,
    
    /// Shortcut for [Instruction::LoadTopIntoR] following [Instruction::Pop].
    PopIntoR,

    /// Swaps the top item and the item below if the stack is at least two items tall.
    SwapTopBelow,
    
    /// Swaps the top value and A.
    SwapTopA,
    
    /// Swap the top value and B.
    SwapTopB,
    
    /// Swap the top value and R.
    SwapTopR,

    /// Loads the top item of the stack into A. If the stack is empty `Nil` is loaded.
    LoadTopIntoA,

    /// Loads the top item of the stack into B. If the stack is empty `Nil` is loaded.
    LoadTopIntoB,

    /// Loads the top item of the stack into R. If the stack is empty `Nil` is loaded.
    LoadTopIntoR,
    
    // --- Objects ---
    
    /// Uses the next u8 to index into the `offset_table`.
    /// This offset is used to create an object via [Instruction::CreateObject].
    CreateObjectOffset,
    
    /// Allocates a new object. The next `u64` is used to index into the `type_table`.
    /// The resulting pointer to this type is used as the type.
    /// 
    /// The object is placed in A.
    CreateObject,
    ReadProperty0,
    ReadProperty1,
    ReadProperty2,
    ReadProperty3,
    ReadPropertyN,
    WriteProperty0,
    WriteProperty1,
    WriteProperty2,
    WriteProperty3,
    WritePropertyN,
    Implements,
    
    /// Casts the object in A to a same-sized type.
    /// 
    /// # Errors
    /// 
    /// - if the value in A is not an object.
    /// - if the types do not have the same size.
    CastEquivalent,
    
    // Built-In Objects
    
    // --- Unary Operations ---
    
    // Angles
    Sine,
    Cosine,
    Tangent,
    ArcusSine,
    ArcusCosine,
    ArcusTangent,
    HyperbolicSine,
    HyperbolicCosine,
    HyperbolicTangent,
    HyperbolicArcusSine,
    HyperbolicArcusCosine,
    HyperbolicArcusTangent,
    Angle,
    
    /// `A = sqrt(A * A + B * B)`
    Hypotenuse,
    
    // Exponents
    Exp,
    
    /// `A = A ** B`
    Power,
    LogE,
    Log2,
    Log10,
    LogN,
    
    // Numbers
    
    Random,
    
    /// `A = sqrt(A)`. Only works on floats and ints.
    SquareRoot,
    CubeRoot,
    NthRoot,
    Abs,
    Ceil,
    Floor,
    Round,
    Max,
    Min,
    Sign,
    Truncate,
    
    // Bitwise
    Invert,
    LeadingZeroes,
    
    // --- Binary Operations ---
    
    AddU63,
    AddF63,
    
    /// A = A - B
    Subtract,

    /// A = A * B
    Multiply,

    /// A = A / B
    Divide,
    Remainder,

    // IO
    
    /// Returns an array of strings.
    Args,
    
    /// Writes A to stdout coercing the value to bytes.
    WriteStdout,
    WriteStdoutLF,
    WriteStderr,
    WriteStderrLF,
    ReadStdin,
    ReadStdinLine,
    
    CreateDirectory,
    CreateFile,
    // Exists
    IsFile,
    IsDirectory,
    WriteFile,
    ReadFileOrDirectory,
    DeleteFileOrDirectory,
    SizeOfFileOrDirectory,
    MoveFileOrDirectory,
    CopyFileOrDirectory,
    GetCreatedOfFileOrDirectory,
    
    MarkFileOrDirectoryAsTemporary,
    MarkFileOrDirectoryAsPermanent,
    
    DebugTriggerGC,
    DebugPrintAllocatedObjects,
    LastInstruction
}

impl TryFrom<u8> for Instruction {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > Instruction::LastInstruction as u8 {
            Err(Error::IllegalInstruction)
        } else {
            Ok(unsafe { transmute(value) })
        }
    }
}

impl Into<u8> for Instruction {
    #[inline]
    fn into(self) -> u8 {
        self as u8
    }
}