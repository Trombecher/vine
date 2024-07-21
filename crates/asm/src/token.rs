use vm::instruction::Instruction;

pub enum Token<'a> {
    EndOfFile,
    Instruction(Instruction),
    Keyword(Keyword),
    Identifier(&'a str),
    Integer(u64),
    Float(f64),
    String(&'a str),
    LeftBrace,
    RightBrace
}

pub enum Keyword {
    Entry,
    Fn,
    Pub,
}

pub static KEYWORD_MAP: phf::Map<&'static str, Keyword> = phf::phf_map!(
    "entry" => Keyword::Entry,
    "fn" => Keyword::Fn,
    "pub" => Keyword::Pub,
);

pub static INSTRUCTION_MAP: phf::Map<&'static str, Instruction> = phf::phf_map!(
    "unreachable" => Instruction::Unreachable,
    "noop" => Instruction::NoOperation,
    "push_a" => Instruction::PushA,
    "push_b" => Instruction::PushB,
    "push_r" => Instruction::PushR,
    "+" => Instruction::Add,
    "-" => Instruction::Subtract,
    "*" => Instruction::Multiply,
    "%" => Instruction::Remainder,
    "/" => Instruction::Divide,
);