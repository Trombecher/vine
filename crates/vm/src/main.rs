use vm::instruction::Instruction;
use vm::{Value, GC, VM};

fn old_main() {
    let gc = GC::new(&[2, 3]);

    let mut vm = VM::<1024>::new(
        &[
            Instruction::CreateObject1 as u8,
            0,
            Instruction::SwapAR as u8,
            Instruction::CreateObject1 as u8,
            1,
            Instruction::SwapAB as u8,
            Instruction::LoadA1Int as u8,
            Instruction::WriteProperty0 as u8,
            Instruction::SwapAB as u8,
            Instruction::WriteStdoutLF as u8,
            Instruction::DebugPrintAllocatedObjects as u8,
            Instruction::DebugTriggerGC as u8,
            Instruction::DebugPrintAllocatedObjects as u8,
            Instruction::SwapAR as u8,
            Instruction::LoadA0Int as u8,
            Instruction::DebugTriggerGC as u8,
            Instruction::DebugPrintAllocatedObjects as u8,
            Instruction::Return as u8,
        ],
        0,
        &[],
        &[],
        &gc,
    );

    vm.execute().unwrap();
}

fn main() {
    let gc = GC::new(&[2]);
    let value = Value::from_strong(gc.allocate(0));

    println!("{:b} -> {:?}", value, value.display(&gc));
}