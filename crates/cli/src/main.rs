#![feature(allocator_api)]

use owo_colors::OwoColorize;
use std::env::args;

fn main() {
    let mut args = args();
    args.next();

    match args.next() {
        None => {
            println!(
                "{}   {}\n\n\n{}{} <command> {}",
                " _    ___\n| |  / (_)___  ___\n| | / / / __ \\/ _ \\\n| |/ / / / / /  __/\n|___/_/_/ /_/\\___/".green().bold(),
                "v0.0.3".bright_black(),
                "Usage: ".bright_white().bold(),
                "vine".green(),
                "[...flags]".cyan(),
            );
        }
        _ => {} /*
                Some(file) => {
                    let content = read_to_string(File::open(file).unwrap()).unwrap();
                    let mut lexer = Lexer::new(content.as_bytes(), Global);

                    let mut last_bound: Index = 0_u32 as Index;

                    let mut token;

                    loop {
                        token = match lexer.next_token() {
                            Err(e) => {
                                println!("{:?}", e);
                                break;
                            }
                            Ok(Span { value: Token::EndOfInput, .. }) => break,
                            Ok(x) => x
                        };

                        let token_text: &str = &content[token.source.start as usize..token.source.end as usize];

                        print!("{}", &content[last_bound as usize..token.source.start as usize]);

                        last_bound = token.source.end;

                        match token.value {
                            Token::Char(_) => print!("{}", token_text.green()),
                            Token::Identifier(_) => print!("{}", token_text),
                            Token::Number(_) => print!("{}", token_text.blue()),
                            Token::DocComment(_) => print!("{}", token_text),
                            Token::Symbol(_) => print!("{}", token_text),
                            Token::Keyword(_) => print!("{}", token_text.red()),
                            Token::String(_) => print!("{}", token_text.green()),
                            _ => print!("{}", token_text)
                        }
                    }
                }
                 */
    }
}
