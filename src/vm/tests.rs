#![cfg(test)]

#[test]
fn test() {
    assert_eq!(crate::vm::Instruction::LastInstruction as u8, 20);
}