use std::error::Error;
use std::fmt;

/// Errors which can come up when validating a given kennitala.
#[derive(Debug, Copy, Clone)]
pub enum KennitalaError {
    /// The kennitala given does not have 10 digits.
    InvalidLength(usize),
    /// The string given cannot be coverted into a valid `u32` number.
    InvalidNumber,
    /// The 1st and 2nd digits representing the day of birth are invalid for
    /// the given month and year.
    InvalidDay,
    /// The 3rd and 4th digits representing the month of birth are invalid.
    InvalidMonth,
    /// The 7th and 8th digits, which can be from 20 up to 99, are not in said
    /// range.
    InvalidRandomDigits,
    /// The 9th digit -- containing the checksum for this kennital --is
    /// invalid.
    InvalidChecksum,
    /// The 10th digit -- representing the century of birth -- is not `9` or
    /// `0`. This means that the person was born in the future!
    InvalidCentury,
}

impl fmt::Display for KennitalaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KennitalaError::InvalidLength(n) => write!(f, "Length {} is invalid", n),
            KennitalaError::InvalidNumber => write!(f, "Invalid number"),
            KennitalaError::InvalidDay => write!(f, "Day of birth is invalid"),
            KennitalaError::InvalidMonth => write!(f, "Month of birth is invalid"),
            KennitalaError::InvalidRandomDigits => write!(f, "The random digits are invalid"),
            KennitalaError::InvalidChecksum => write!(f, "The kennitala's checksum is invalid"),
            KennitalaError::InvalidCentury => write!(f, "Century of birth is invalid"),
        }
    }
}

impl Error for KennitalaError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}
