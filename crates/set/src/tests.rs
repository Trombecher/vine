#![cfg(test)]

use std::mem::transmute;

use crate::Set256;

#[repr(u8)]
#[derive(Debug, PartialEq)]
enum ABC {
    A,
    B,
    C = 2,
}

impl Into<u8> for ABC {
    fn into(self) -> u8 {
        self as u8
    }
}

impl TryFrom<u8> for ABC {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > ABC::C as u8 {
            Err(())
        } else {
            Ok(unsafe { transmute(value) })
        }
    }
}

#[test]
fn test() {
    let set = Set256::from_iter([ABC::A]);
    
}