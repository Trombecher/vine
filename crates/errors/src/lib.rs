#![no_std]
extern crate alloc;

#[macro_export]
macro_rules! error {
    ($msg:literal) => {{
        static OBJ: $crate::ErrorData = $crate::ErrorData {
            file_path: module_path!(),
            source_line: line!(),
            source_column: column!(),
            file_name: file!(),
            message: $msg
        };

        Err(Error(&OBJ))
    }}
}

#[derive(Copy, Clone, Debug)]
pub struct Error(pub &'static ErrorData);

#[derive(Debug)]
pub struct ErrorData {
    pub file_path: &'static str,
    pub source_line: u32,
    pub source_column: u32,
    pub message: &'static str,
    pub file_name: &'static str
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        self.0 as *const ErrorData == other.0 as *const ErrorData
    }
}