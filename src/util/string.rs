//! Helper routines for strings

use core::num::ParseIntError;

/// Trait for doing more interesting string conversions involving prefixes
pub trait FromStrExt: Sized {
    /// `from_str_radix` does not seem to exist in an existing Trait
    fn from_str_radix(inpur: &str, radix: u32) -> Result<Self, ParseIntError>;

    /// Convert a string, that potentially contains a prefix that can
    /// override the passed radix, to the current type
    fn from_str_prefix_radix(input: &str, radix: u32) -> Result<Self, ParseIntError> {
        let s = if input.len() < 2 {
                (radix, input)
            } else {
                match &input[0..2] {
                    "0x"| "0X" => (16, &input[2..]),
                    _ => (radix, input),
                }
            };
        return Self::from_str_radix(s.1, s.0);
    }

    /// Use `from_str_prefix_radix` with a default radix of 10
    fn from_str_prefix(input: &str) -> Result<Self, ParseIntError> {
        Self::from_str_prefix_radix(input, 10)
    }
}

/// TODO: automate this with a macro for all the usual types
impl FromStrExt for u16 {
    fn from_str_radix(input: &str, radix: u32) -> Result<u16, ParseIntError> {
        u16::from_str_radix(input, radix)
    }
}

