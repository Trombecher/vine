#![feature(str_from_raw_parts)]
#![feature(ptr_sub_ptr)]

//! # Vine Assembly
//!
//! This crate contains functionality for compiling vine assembly (.vna) code
//! into the binary representation of vine byte code from the vm crate.

pub mod parse;
pub mod lex;
pub mod token;

#[derive(Copy, Clone, Debug)]
pub enum Error {
    Bytes(bytes::Error),
    VM(vm::Error),
}