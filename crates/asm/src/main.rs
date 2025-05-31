use asm::lex::Lexer;
use asm::token::{Token, TokenIterator};
use asm::Error;
use std::fs;
use vm::{GC, VM};

fn main() -> Result<(), Error> {
    let asm = fs::read("test.vna").unwrap();
    let mut lexer = Lexer::new(asm.as_slice());

    let mut code = Vec::new();

    loop {
        match lexer.next_token()?.value {
            Token::EndOfFile => break,
            token => token.encode(&mut code),
        }
    }

    let gc = GC::new(&[1, 2, 3]);
    let mut vm = VM::<1024>::new(code.as_slice(), 0, &[], &[], &gc);
    vm.execute().unwrap();

    Ok(())
}
