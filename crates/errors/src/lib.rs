#![no_std]

extern crate alloc;

use alloc::boxed::Box;

#[macro_export]
macro_rules! error {
    ($msg:literal) => {
        Err(::alloc::boxed::Box::new($crate::ErrorData {
            file_path: module_path!(),
            source_line: line!(),
            source_column: column!(),
            message: $msg
        }))
    }
}

pub type Error = Box<ErrorData>;

impl PartialEq for Box<ErrorData> {
    fn eq(&self, other: &Self) -> bool {
        self == other
    }
}

#[derive(Debug)]
pub struct ErrorData {
    pub file_path: &'static str,
    pub source_line: u32,
    pub source_column: u32,
    pub message: &'static str
}