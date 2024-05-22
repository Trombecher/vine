use std::mem::transmute;
use crate::vm::{Error, Value};

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum Instruction {
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

    /// Returns with the value in the return register.
    Return,

    /// Returns [Value::Nil].
    ReturnNil,

    // --- Registers ---
    
    // A
    
    /// Casts the value in A to a number corresponding to the following scheme:
    /// 
    /// - number -> do nothing
    /// - 
    CastANumber,
    
    /// Loads 0 as u64 into A with type [Value::Nil].
    LoadANil0,

    /// Loads 1 as u64 into A with type [Value::Nil].
    LoadANil1,

    /// Loads the next byte into A with type [Value::Nil].
    LoadANilB1,

    /// Loads the next two bytes into A with type [Value::Nil].
    LoadANilB2,

    /// Loads the next four bytes into A with type [Value::Nil].
    LoadANilB4,

    /// Loads the next eight bytes into A with type [Value::Nil].
    LoadANilB8,

    /// Loads `0.0` into A.
    LoadANumber0,

    /// Loads `1.0` into A.
    LoadANumber1,

    /// Loads the next eight bytes as f64 into A with type [Value::Number].
    LoadANumber,

    /// Loads the entry from the static table into A, with the next byte being the index.
    LoadAStatic,
    
    // Register B
    
    /// Loads 0 as u64 into B with type [Value::Nil].
    LoadBNil0,

    /// Loads 1 as u64 into B with type [Value::Nil].
    LoadBNil1,

    /// Loads the next byte into B with type [Value::Nil].
    LoadBNilB1,

    /// Loads the next two bytes into B with type [Value::Nil].
    LoadBNilB2,

    /// Loads the next four bytes into B with type [Value::Nil].
    LoadBNilB4,

    /// Loads the next eight bytes into B with type [Value::Nil].
    LoadBNilB8,

    /// Loads `0.0` into B.
    LoadBNumber0,

    /// Loads `1.0` into B.
    LoadBNumber1,

    /// Loads the next eight bytes as f64 into B with type [Value::Number].
    LoadBNumber,

    /// Loads the entry from the static table into B, with the next byte being the index.
    LoadBStatic,

    // Register R
    
    /// Loads 0 as u64 into R with type [Value::Nil].
    LoadRNil0,
    
    /// Loads 1 as u64 into R with type [Value::Nil].
    LoadRNil1,
    
    /// Loads the next byte into R with type [Value::Nil].
    LoadRNilB1,
    
    /// Loads the next two bytes into R with type [Value::Nil].
    LoadRNilB2,
    
    /// Loads the next four bytes into R with type [Value::Nil].
    LoadRNilB4,
    
    /// Loads the next eight bytes into R with type [Value::Nil].
    LoadRNilB8,

    /// Loads `0.0` into R
    LoadRNumber0,

    /// Loads `1.0` into R.
    LoadRNumber1,
    
    /// Loads the next eight bytes as f64 into R with type [Value::Number].
    LoadRNumber,
    
    /// Loads the entry from the static table with the next byte being the index.
    LoadRStatic,

    SwapAB,
    SwapAR,
    SwapBR,

    // Operations
    
    /// Assumes and adds integers in A and B. The type of R will be [Value::Nil].
    AddU64Unchecked,
    
    /// Assumes and adds floats in A and B. The type of R will be [Value::Number].
    AddF64Unchecked,
    
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
    
    /// A = A + B, A and B must be of type int or float.
    Add,

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
    
    WriteFile,
    ReadFile,
    FileExists,
    CreateDirectory,
    ReadDirectory,
    DeleteFile,
    DeleteEmptyDirectory,
    DeleteDirectoryRecursively,
    
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