#![no_std]
extern crate alloc;

use alloc::string::String;

#[macro_export]
macro_rules! error {
    ($msg:literal) => {{
        static ERROR_DATA: $crate::ErrorData = $crate::ErrorData {
            file_path: module_path!(),
            source_line: line!(), 
            source_column: column!(),
            file_name: file!()
        };
        
        Err($crate::Error {
            data: &ERROR_DATA,
            message: $msg.into()
        })
    }};
    ($msg:literal, $($arg:tt)*) => {{
        static ERROR_DATA: $crate::ErrorData = $crate::ErrorData {
            file_path: module_path!(),
            source_line: line!(), 
            source_column: column!(),
            file_name: file!()
        };
        
        Err($crate::Error {
            data: &ERROR_DATA,
            message: ::alloc::format!($msg, $($arg)*)
        })
    }}
}

#[derive(Clone, Debug)]
pub struct Error {
    pub data: &'static ErrorData,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ErrorData {
    pub file_path: &'static str,
    pub source_line: u32,
    pub source_column: u32,
    pub file_name: &'static str,
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}