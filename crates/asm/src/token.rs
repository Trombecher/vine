use bytes::Span;
use vm::instruction::Instruction;
use crate::Error;

#[derive(Clone, Debug, PartialEq)]
pub enum Token<'a> {
    EndOfFile,
    Instruction(Instruction),
    Keyword(Keyword),
    Identifier(&'a str),
    Integer(u64),
    SmallFloat(f32),
    BigFloat(f64),
    // String(&'a str),
    LineBreak,
    LeftBrace,
    RightBrace
}

impl<'a> Token<'a> {
    pub fn encode(&self, target: &mut Vec<u8>) {
        match self {
            Token::EndOfFile => {}
            Token::Instruction(i) => target.push(*i as u8),
            Token::Keyword(_) => todo!("We cannot encode keywords rn"),
            Token::Identifier(_) => todo!("We cannot encode ids rn"),
            Token::Integer(i) => target.extend_from_slice(&i.to_le_bytes()),
            Token::SmallFloat(f) => target.extend_from_slice(&f.to_le_bytes()),
            Token::BigFloat(f) => target.extend_from_slice(&f.to_le_bytes()),
            Token::LineBreak => {},
            Token::LeftBrace => todo!("We cannot encode {{ rn"),
            Token::RightBrace => todo!("We cannot encode }} rn")
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Keyword {
    Entry,
    Fn,
}

pub static KEYWORD_MAP: phf::Map<&'static str, Keyword> = phf::phf_map!(
    "entry" => Keyword::Entry,
    "fn" => Keyword::Fn,
);

pub static INSTRUCTION_MAP: phf::Map<&'static str, Instruction> = phf::phf_map!(
    "unreachable" => Instruction::Unreachable,
    "noop" => Instruction::NoOperation,
    "ret" => Instruction::Return,
    "push_a" => Instruction::PushA,
    "push_b" => Instruction::PushB,
    "push_r" => Instruction::PushR,
    "write_stdout_lf" => Instruction::WriteStdoutLF,
    
    // Alternative symbols
    "..." => Instruction::Unimplemented,
    "_" => Instruction::NoOperation,
    
    "a=0" => Instruction::LoadA0Int,
    "a=1" => Instruction::LoadA1Int,
    "a=0f" => Instruction::LoadA0F32,
    "a=1f" => Instruction::LoadA1F32,
    "a=0F" => Instruction::LoadA0F64,
    "a=1F" => Instruction::LoadA1F64,
    "a=..1" => Instruction::LoadAImmediate1,
    "a=..2" => Instruction::LoadAImmediate2,
    "a=..4" => Instruction::LoadAImmediate4,
    "a=..8" => Instruction::LoadAImmediate8,
    
    "a<->b" => Instruction::SwapAB,
    "a<->r" => Instruction::SwapAR,
    "b<->r" => Instruction::SwapBR,
    
    "a+=b,u" => Instruction::AddU63,
    "a-=b" => Instruction::Subtract,
    "a*=b" => Instruction::Multiply,
    "a/=b" => Instruction::Divide,
    "a%=b" => Instruction::Remainder,
    
    "a=(..1)" => Instruction::CreateObject1,
    "a=(..2)" => Instruction::CreateObject2,
    "a=(..3)" => Instruction::CreateObject3,
    "a=(..4)" => Instruction::CreateObject4,
    "a=b.0" => Instruction::ReadProperty0,
    "a=b.1" => Instruction::ReadProperty1,
    "a=b.2" => Instruction::ReadProperty2,
    "a=b.3" => Instruction::ReadProperty3,
    "b.0=a" => Instruction::WriteProperty0,
    "b.1=a" => Instruction::WriteProperty1,
    "b.2=a" => Instruction::WriteProperty2,
    "b.3=a" => Instruction::WriteProperty3,
);

pub trait TokenIterator<'a> {
    fn next_token(&mut self) -> Result<Span<Token<'a>>, Error>;
}