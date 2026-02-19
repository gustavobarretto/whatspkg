//! Token byte constants for the binary protocol.
//! String compression uses a dictionary; we use raw BINARY_8/BINARY_20 for strings when not in the dictionary.

#[allow(dead_code)]
pub const DICT_VERSION: u8 = 3;

pub const LIST_EMPTY: u8 = 0;
pub const LIST_8: u8 = 248;
pub const LIST_16: u8 = 249;
pub const BINARY_8: u8 = 252;
pub const BINARY_20: u8 = 253;
#[allow(dead_code)]
pub const BINARY_32: u8 = 254;
