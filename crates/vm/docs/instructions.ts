type Category = "Control Flow" | "File IO" | "Registers" | "Stack" | "Standard IO" | "Math" | "Objects";

type ImmediateCodeArgument = "Absolute Offset" | "Integer";

type Instruction = {
    longName?: string,
    name: string,
    altName?: string,
    category: Category,
    description?: string,
    shorthandFor?: string,
    expects?: string,
    pure?: true,
    icas?: ImmediateCodeArgument[]
};

export const INSTRUCTIONS: Record<number, Instruction> = {
    0x00: {
        category: "Control Flow",
        longName: "Unreachable",
        name: "unreachable",
        altName: "!!!",
        description: "Reaching this instruction stops execution and informs the surrounding environment."
    },
    0x01: {
        category: "Control Flow",
        longName: "No Operation",
        name: "noop",
        altName: "_",
        description: "Does nothing."
    },
    0x02: {
        category: "Control Flow",
        longName: "Return",
        name: "ret",
        description: "Returns from the function. This causes the stack before the function call to be restored."
    },
    0x03: {
        category: "Control Flow",
        longName: "Return Zero",
        name: "retz",
        shorthandFor: "load_r_nil return"
    },
    0x04: {
        category: "Control Flow",
        longName: "Call Function",
        name: "call"
    },
    0x08: {
        category: "Control Flow",
        longName: "Jump Not Zero",
        name: "jnz",
        description: "Jumps",
        icas: ["Absolute Offset"]
    },
    0x10: {
        category: "Registers",
        name: "load_a_int_0",
        pure: true,
        shorthandFor: "load_a_int 0x0",
    },
    0x11: {
        category: "Registers",
        name: "load_a_int_1",
        pure: true,
        shorthandFor: "load_a_int 0x1",
    },
    0x12: {
        category: "Registers",
        pure: true,
        name: "load_a_int",
        description: "Loads the following integer into _A_.",
        icas: ["Integer"]
    },
    0x13: {
        category: "Registers",
        pure: true,
        name: "load_a_int_const"
    },
    0x14: {
        category: "Registers",
        pure: true,
        name: "load_a_float_0",
    },
    0x15: {
        category: "Registers",
        pure: true,
        name: "load_a_float_1",
    },
    0x16: {
        category: "Registers",
        pure: true,
        name: "load_a_float",
    },
    0x17: {
        category: "Registers",
        pure: true,
        name: "load_a_float_const",
    },
    // ...
    0x28: {
        category: "Registers",
        pure: true,
        name: "swap_ab",
    },
    0x29: {
        category: "Registers",
        pure: true,
        name: "swap_ar",
    },
    0x2A: {
        category: "Registers",
        pure: true,
        name: "swap_br",
    },
    0x30: {
        category: "Stack",
        pure: true,
        name: "push_a",
        description: "Pushes the value of _A_ onto the stack."
    },
    0x31: {
        category: "Stack",
        pure: true,
        name: "push_b",
        description: "Pushes the value of _B_ onto the stack."
    },
    0x32: {
        category: "Stack",
        pure: true,
        name: "push_r",
        description: "Pushes the value of _B_ onto the stack."
    },
    0x33: {
        category: "Stack",
        pure: true,
        name: "clear",
        description: "Truncates the stack to the size before the invocation of the function this instruction is running in. If the function is the entry function, it clears the stack completely.",
    },
    0x34: {
        category: "Stack",
        pure: true,
        name: "pop",
        description: "Removes the top value of the stack if it was created in the function this instruction is running in.",
    },
    0x35: {
        category: "Stack",
        pure: true,
        name: "pop_into_a",
        shorthandFor: "top_into_a pop",
    },
    0x36: {
        category: "Stack",
        pure: true,
        name: "pop_into_b",
        shorthandFor: "top_into_b pop"
    },
    0x37: {
        category: "Stack",
        pure: true,
        name: "pop_into_r",
        shorthandFor: "top_into_r pop"
    },
    0x38: {
        category: "Stack",
        pure: true,
        name: "swap",
        description: "Swaps the pair of top values if both values exist and are in frame."
    },
    0x39: {
        category: "Stack",
        pure: true,
        name: "swap_a",
        shorthandFor: "push_a swap pop_into_a"
    },
    0x3A: {
        category: "Stack",
        pure: true,
        name: "swap_b",
        shorthandFor: "push_b swap pop_into_b"
    },
    0x3B: {
        category: "Stack",
        pure: true,
        name: "swap_r",
        shorthandFor: "push_r swap pop_into_r"
    },
    0x3C: {
        category: "Stack",
        pure: true,
        name: "duplicate",
        description: "Pushes the top value of the stack onto itself if there is a top value and it is in frame."
    },
    0x3D: {
        category: "Stack",
        pure: true,
        name: "frame_size",
        description: "Puts the frame size (unsigned integer) into _A_. (The number of values pushed onto the stack in this frame.)"
    },
    0x40: {
        category: "Standard IO",
        pure: true,
        name: "args",
        description: "Puts an array of strings into _A_, containing the arguments passed to the program."
    },
    0x41: {
        category: "Standard IO",
        pure: true,
        name: "print",
        description: "Writes the value in _A_ to the standard output."
    },
    0x42: {
        category: "Standard IO",
        pure: true,
        name: "printlf",
        shorthandFor: "write_stdout load_a \"\\n\" write_stdout"
    },
    0x43: {
        category: "Standard IO",
        pure: true,
        name: "eprint",
        description: "Writes the value in _A_ to the standard error."
    },
    0x44: {
        category: "Standard IO",
        pure: true,
        name: "eprintlf",
        shorthandFor: "write_stderr load_a \"\\n\" write_stderr"
    },
    0x45: {
        category: "Standard IO",
        pure: true,
        name: "readln",
        description: "Reads a line from the standard input.",
    },
    0x48: {
        category: "File IO",
        name: "does_file_exist",
        shorthandFor: "push_b push_a pop_into_b is_file swap_a_b is_directory or pop_into_b",
        description: "Checks if either `is_file` or `is_directory` is true."
    },
    0x49: {
        category: "File IO",
        name: "is_file",
        description: "Loads `1` (unsigned integer) into _A_ if the string in _A_ resolves to a file; else it loads `0` (unsigned integer)."
    },
    0x4A: {
        category: "File IO",
        name: "is_dir",
        description: "Loads `1` (unsigned integer) into _A_ if the string in _A_ resolves to a directory; else it loads `0` (unsigned integer)."
    },
    0x4B: {
        category: "File IO",
        name: "read_fs",
        description: "Expects a string (path) in _A_.\nIf the path resolves to a file, it reads the entire file into a mutable buffer which is put into _A_.\nIf the path resolves to a directory, an array of strings is put into _A_."
    },
    0x4C: {
        category: "File IO",
        name: "write_file",
        description: "Expects a string (path) in _A_ and a value implementing `Serialize` in _B_.\nIf the path resolves to a file, `serialize(...)` is called on _B_ and written to file."
    },
    0x4D: {
        category: "File IO",
        name: "del_fs"
    },
    0x4E: {
        category: "File IO",
        name: "del_empty"
    },
    // ...
    0x60: {
        category: "Math",
        pure: true,
        name: "sin",
        description: "Expects a float or an int in _A_; else it throws. If _A_ is an int, it will be converted into a float.\n\nPuts the sine of _A_ in _A_."
    },
    // ...
    0x61: {
        category: "Math",
        pure: true,
        name: "cos"
    },
    0x62: {
        category: "Math",
        pure: true,
        name: "tan"
    },
    0x63: {
        category: "Math",
        pure: true,
        name: "asin"
    },
    0x64: {
        category: "Math",
        pure: true,
        name: "acos"
    },
    0x65: {
        category: "Math",
        pure: true,
        name: "atan"
    },
    0x66: {
        category: "Math",
        pure: true,
        name: "sinh"
    },
    0x67: {
        category: "Math",
        pure: true,
        name: "cosh"
    },
    0x68: {
        category: "Math",
        pure: true,
        name: "tanh"
    },
    0x69: {
        category: "Math",
        pure: true,
        name: "asinh"
    },
    0x6A: {
        category: "Math",
        pure: true,
        name: "acosh"
    },
    0x6B: {
        category: "Math",
        pure: true,
        name: "atanh"
    },
    0x6C: {
        category: "Math",
        pure: true,
        name: "hypot"
    },
    0x6D: {
        category: "Math",
        name: "pow",
        pure: true,
        altName: "e^"
    },
    0x6E: {
        category: "Math",
        pure: true,
        name: "loge"
    },
    0x6F: {
        category: "Math",
        pure: true,
        name: "log2"
    },
    0x70: {
        category: "Math",
        pure: true,
        name: "log10"
    },
    0x71: {
        category: "Math",
        pure: true,
        name: "logn"
    },
    0x72: {
        category: "Math",
        pure: true,
        name: "rand"
    },
    0x73: {
        category: "Math",
        pure: true,
        name: "sqrt"
    },
    0x74: {
        category: "Math",
        pure: true,
        name: "cbrt"
    },
    0x75: {
        category: "Math",
        pure: true,
        name: "nrt"
    },
    0x76: {
        category: "Math",
        pure: true,
        name: "abs"
    },
    0x77: {
        category: "Math",
        pure: true,
        name: "ceil"
    },
    0x78: {
        category: "Math",
        pure: true,
        name: "floor"
    },
    0x79: {
        category: "Math",
        pure: true,
        name: "round"
    },
    0x7A: {
        category: "Math",
        pure: true,
        name: "max_u"
    },
    0x7B: {
        category: "Math",
        pure: true,
        name: "max_s"
    },
    0x7C: {
        category: "Math",
        pure: true,
        name: "min_u"
    },
    0x7D: {
        category: "Math",
        pure: true,
        name: "min_s"
    },
    0x7E: {
        category: "Math",
        name: "sign"
    },
    0x7F: {
        category: "Math",
        name: "trunc"
    },
    0x80: {
        category: "Math",
        name: "add",
        altName: "+"
    },
    0x81: {
        category: "Math",
        name: "sub",
        altName: "-"
    },
    0x82: {
        category: "Math",
        name: "mul",
        altName: "*"
    },
    0x83: {
        category: "Math",
        name: "div_u",
        altName: "/u"
    },
    0x84: {
        category: "Math",
        name: "div_s",
        altName: "/s"
    },
    0x85: {
        category: "Math",
        name: "rem_u",
        altName: "%u"
    },
    0x86: {
        category: "Math",
        name: "rem_s",
        altName: "%s"
    },
    0x87: {
        category: "Math",
        name: "exp_u",
        altName: "**u"
    },
    0x88: {
        category: "Math",
        name: "exp_s",
        altName: "**s"
    },
    0x89: {
        category: "Math",
        name: "inv",
        altName: "~"
    },
    0x8A: {
        category: "Math",
        name: "leadz"
    },
    0x8B: {
        category: "Math",
        name: "or",
        altName: "|"
    },
    0x8C: {
        category: "Math",
        name: "and",
        altName: "&"
    },
    0x8D: {
        category: "Math",
        name: "xor",
        altName: "^"
    },
    0x8E: {
        category: "Math",
        name: "neg",
        altName: "!"
    },
    0x8F: {
        category: "Math",
        longName: "Count Ones",
        name: "popcnt",
        description: ""
    },
    0x90: {
        category: "Math",
        name: "shz",
        altName: "<0>",
        expects: "Expects an int or a float in _A_ and _B_, throws otherwise. If _B_ is a float, it is truncated into a signed int.",
        description: "Interprets the int in _B_ as signed and shifts _A_ by _B_: left if _B_ is positive and right if _B_ is negative. Overflown bits are dropped, on right shift *zeroes are inserted*."
    },
    0x91: {
        category: "Math",
        name: "shs",
        altName: "<?>",
        description: "The same as `shz` but instead of zeroes, the high bit is repeated during right shifts.",
    },
    0x92: {
        category: "Math",
        name: "eq",
        altName: "==",
        description: "Does a value comparison on _A_ and _B_; stores the boolean in _A_."
    },
    0x93: {
        category: "Math",
        name: "neq",
        altName: "!="
    },
    0x94: {
        category: "Math",
        name: "gt_u",
        altName: ">u"
    },
    0x95: {
        category: "Math",
        name: "gt_s",
        altName: ">s"
    },
    0x96: {
        category: "Math",
        name: "lt_u",
        altName: "<u"
    },
    0x97: {
        category: "Math",
        name: "lt_s",
        altName: "<s"
    },
    0x98: {
        category: "Math",
        name: "ge_u",
        altName: ">=u"
    },
    0x99: {
        category: "Math",
        name: "ge_s",
        altName: ">=s"
    },
    0x9A: {
        category: "Math",
        name: "le_u",
        altName: "<=u"
    },
    0x9B: {
        category: "Math",
        name: "le_s",
        altName: "<=s"
    },
    0x9C: {
        category: "Math",
        name: "dec",
        altName: "--"
    },
    0x9D: {
        category: "Math",
        name: "inc",
        altName: "++"
    },
    0xA8: {
        category: "Objects",
        name: "alloc",
    },
    0xA9: {
        category: "Objects",
        name: "casteq"
    },
    0xAA: {
        category: "Objects",
        name: "prop0",
        altName: ".0"
    },
    0xAB: {
        category: "Objects",
        name: "prop1",
        altName: ".1"
    },
    0xAC: {
        category: "Objects",
        name: "prop2",
        altName: ".2"
    },
    0xAD: {
        category: "Objects",
        name: "prop3",
        altName: ".3"
    },
    0xAE: {
        category: "Objects",
        name: "prop",
        altName: "."
    },
    0xAF: {
        category: "Objects",
        name: "is",
    }
};

function html() {    
    const instr = Object.entries(INSTRUCTIONS);
    instr.sort(([a, _a], [b, _b]) => +a - +b);
    return instr.map(([code, {name}]) => `<h2><code>${name}</code></h2><p>Code: 0x${(+code).toString(16)}</p>`).join("");
}

await Bun.write("output.html", html());