// pub mod format;
mod generated_codes;

use parse_tools::bytes;
use parse_tools::bytes::Cursor;
pub use generated_codes::*;

impl Into<Error> for bytes::Error {
    fn into(self) -> Error {
        match self {
            bytes::Error::EncounteredContinuationByte => Error::E0000,
            bytes::Error::Missing2ndOf2 => Error::E0001,
            bytes::Error::Invalid2ndOf2 => Error::E0001,
            bytes::Error::Missing2ndOf3 => Error::E0003,
            bytes::Error::Invalid2ndOf3 => Error::E0003,
            bytes::Error::Missing3rdOf3 => Error::E0005,
            bytes::Error::Invalid3rdOf3 => Error::E0005,
            bytes::Error::Missing2ndOf4 => Error::E0007,
            bytes::Error::Invalid2ndOf4 => Error::E0007,
            bytes::Error::Missing3rdOf4 => Error::E0009,
            bytes::Error::Invalid3rdOf4 => Error::E0009,
            bytes::Error::Missing4thOf4 => Error::E0011,
            bytes::Error::Invalid4thOf4 => Error::E0011,
        }
    }
}

pub fn get_lines_and_columns(source: &str, byte_offset: usize) -> (usize, usize) {
    let mut lines = 0;
    let mut columns = 0;
    let mut i = 0;

    let mut cursor = Cursor::new(source.as_bytes());

    loop {
        if byte_offset <= i { break }

        match cursor.peek() {
            Some(b'\r') => {
                lines += 1;
                columns = 0;
                
                unsafe { cursor.advance_unchecked() }

                if let Some(b'\n') = cursor.peek() {
                    unsafe { cursor.advance_unchecked() }
                    i += 1;
                }
            }
            Some(b'\n') => {
                lines += 1;
                columns = 0;
                
                unsafe { cursor.advance_unchecked() }
            }
            Some(_) => {
                columns += 1;
                unsafe { cursor.advance_char_unchecked() }
            }
            None => break
        }

        i += 1;
    }

    (lines, columns)
}