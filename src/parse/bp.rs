//! # Binding precedence.

pub const COMMA_AND_SEMICOLON: u8 = 0;
pub const RETURN: u8 = 1;
pub const ASSIGNMENT: (u8, u8) = (3, 2);
pub const LOGICAL_OR: (u8, u8) = (4, 5);
pub const LOGICAL_AND: (u8, u8) = (6, 7);
pub const BITWISE_OR: (u8, u8) = (8, 9);
pub const BITWISE_AND: (u8, u8) = (10, 11);
pub const BITWISE_XOR: (u8, u8) = (12, 13);
pub const EQUALITY: (u8, u8) = (14, 15);
pub const RELATIONAL: (u8, u8) = (16, 17);
pub const SHIFT: (u8, u8) = (18, 19);
pub const ADDITIVE: (u8, u8) = (20, 21);
pub const MULTIPLICATIVE: (u8, u8) = (22, 23);
pub const EXPONENTIAL: (u8, u8) = (24, 25);
pub const NEGATE_AND_NOT: u8 = 26;
pub const CALL: u8 = 27;
pub const ACCESS_AND_OPTIONAL_ACCESS: u8 = 28;
pub const BLOCK: u8 = 29;