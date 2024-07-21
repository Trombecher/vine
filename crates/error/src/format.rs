use std::io;
use std::io::{Stdout, Write};
use crossterm::{queue, QueueableCommand};
use crossterm::style::{Print, PrintStyledContent, Stylize};
use vine::error::Error;

pub fn format(error: Error, stdout: &mut Stdout) -> io::Result<()> {
    queue!(stdout,
        PrintStyledContent(error.code_str().red().bold()),
        PrintStyledContent(" (error in ".grey()),
        PrintStyledContent(error.source().as_str().grey()),
        PrintStyledContent("): ".grey()),
        PrintStyledContent(error.as_str().white()),
        Print("\n"),
    );
    
    if let Some(context) = error.context() {
        stdout
            .queue(Print("Context: The error occurred "))?
            .queue(Print(context))?
            .queue(Print("\n"))?;
    }
    
    stdout.flush()
}