//! # Kennitölur
//! A kennitala (plural form: kennitölur) is a unique national identification
//! number assigned by the Icleandic government, assigned to individuals (and
//! organizations) in Iceland.
//!
//! ## Number specification
//! Kennitalas are composed of 10 digits. The first six of these are the
//! individual's date of birth in DDMMYY format. The seventh and eight digits
//! are randomly chosen when the kennitala is allocated, ranging from 22 to 99.
//! The ninth digit is the checksum digit, and the tenth indicates the century
//! of the individual's birth.
//!
//! ### Checksum digit
//! The dot product of the vector containing the first 8 digits of the kennitala
//! is taken with the vector `[3, 2, 7, 6, 5, 4, 3, 2]`. Take the modulo 11 of
//! that computation. If the result `r` is 0, the checksum digit is 0, otherwise it
//! is `11 - r`.
#![deny(
    missing_docs,
    future_incompatible,
    nonstandard_style,
    rust_2018_idioms,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unused_qualifications
)]
mod dates;
mod error;

#[cfg(feature = "chrono")]
use chrono::naive::NaiveDate;
use std::convert::TryFrom;
use std::fmt;

use dates::days_in_month;
pub use error::KennitalaError;

const VALIDATION_DIGITS: [u8; 8] = [3, 2, 7, 6, 5, 4, 3, 2];

const DAY_MASK: u32 = 0b00000000_00000000_00000000_00011111;
const DAY_OFFSET: u32 = 0;
const MONTH_MASK: u32 = 0b00000000_00000000_00000001_11100000;
const MONTH_OFFSET: u32 = DAY_OFFSET + 5;
const YEAR_MASK: u32 = 0b00000000_00000000_11111110_00000000;
const YEAR_OFFSET: u32 = MONTH_OFFSET + 4;
const REST_MASK: u32 = 0b00000011_11111111_00000000_00000000;
const REST_OFFSET: u32 = YEAR_OFFSET + 7;
const CENTURY_MASK: u32 = 0b00000100_00000000_00000000_00000000;
const CENTURY_OFFSET: u32 = REST_OFFSET + 10;

/// Struct that represents the kennitala of an Icelandic citizen or resident.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Kennitala {
    internal: u32,
}

impl Kennitala {
    /// Create new kennitala object from the given string. Validation is done
    /// beforehand.
    pub fn new(kennitala: &str) -> Result<Self, KennitalaError> {
        let only_numbers = kennitala
            .chars()
            .all(|c| ((c as u32) >= 48) && ((c as u32) <= 57));
        if !only_numbers {
            return Err(KennitalaError::InvalidNumber);
        }

        if kennitala.len() != 10 {
            // A valid kennitala string consists of 10 ASCII bytes.
            return Err(KennitalaError::InvalidLength(kennitala.len()));
        }

        let mut kennitala_array = <[u8; 10]>::try_from(kennitala.as_bytes()).unwrap();
        for d in &mut kennitala_array {
            // The ASCII codes for the arabic numerals share a contiguous range
            // from 48 to 57.
            *d -= 48;
        }
        Kennitala::from_slice(&kennitala_array)
    }

    // Create new kennitala object from the given u32. Validation is done
    /// beforehand.
    pub fn from_u32(kennitala_u32: u32) -> Result<Self, KennitalaError> {
        let mut kennitala = [0; 10];
        kt_to_array(kennitala_u32, &mut kennitala)?;
        Kennitala::from_slice(&kennitala)
    }

    /// Create new kennitala object from the given slice. Validation is done
    /// beforehand. Each element in the slice must be equal or less than 9.
    fn from_slice(kennitala: &[u8; 10]) -> Result<Self, KennitalaError> {
        debug_assert!(kennitala.iter().all(|d| *d <= 9));

        let checksum_digit = kennitala[8];
        let calculated_checksum_digit = calculate_checksum_digit(&kennitala);
        if checksum_digit != calculated_checksum_digit {
            return Err(KennitalaError::InvalidChecksum);
        }

        if ((kennitala[6] * 10) + kennitala[7]) < 20 {
            return Err(KennitalaError::InvalidRandomDigits);
        }

        let century_digit = kennitala[9] as u32;
        if !((century_digit == 0) || (century_digit == 9)) {
            return Err(KennitalaError::InvalidCentury);
        }
        let year_offset = if century_digit == 0 { 2000 } else { 1900 };

        let dob_month = (kennitala[2] * 10) as u32 + kennitala[3] as u32;
        if (dob_month > 12) || (dob_month <= 0) {
            return Err(KennitalaError::InvalidMonth);
        }

        let dob_year = (kennitala[4] * 10) as u32 + kennitala[5] as u32;

        let dob_day = (kennitala[0] * 10) as u32 + kennitala[1] as u32;
        if (dob_day > days_in_month(dob_month, dob_year + year_offset)) || (dob_day <= 0) {
            return Err(KennitalaError::InvalidDay);
        }

        let rest = (kennitala[6] as u32) * 100 + (kennitala[7] * 10) as u32 + kennitala[8] as u32;

        let mut value = dob_day << DAY_OFFSET;
        value += dob_month << MONTH_OFFSET;
        value += dob_year << YEAR_OFFSET;
        value += rest << REST_OFFSET;
        value += ((century_digit == 0) as u32) << CENTURY_OFFSET;

        Ok(Self { internal: value })
    }

    /// Get day in the range [1, 31]
    #[inline]
    pub fn get_day(&self) -> u32 {
        let day = (self.internal & DAY_MASK) >> DAY_OFFSET;
        debug_assert!((day >= 1) && (day <= 31));
        day
    }

    /// Get month in the range [1, 12]
    #[inline]
    pub fn get_month(&self) -> u32 {
        let month = (self.internal & MONTH_MASK) >> MONTH_OFFSET;
        debug_assert!((month >= 1) && (month <= 12));
        month
    }

    /// Get year in the range [0, 99]
    #[inline]
    pub fn get_short_year(&self) -> u32 {
        let short_year = (self.internal & YEAR_MASK) >> YEAR_OFFSET;
        debug_assert!(short_year <= 99);
        short_year
    }

    /// Get year in the range [1900, 2099]
    #[inline]
    pub fn get_year(&self) -> u32 {
        let offset = if self.get_century_bit() == 0 {
            1900
        } else {
            2000
        };
        self.get_short_year() + offset
    }

    /// Get the value of the bit storing which century this Kennitala's holder
    /// was born in.
    #[inline]
    fn get_century_bit(&self) -> u32 {
        let bit = (self.internal & CENTURY_MASK) >> CENTURY_OFFSET;
        debug_assert!((bit == 0) || (bit == 1));
        bit
    }

    /// Get century digit in the set {0, 9}
    #[inline]
    pub fn get_short_century(&self) -> u32 {
        if self.get_century_bit() == 0 {
            9
        } else {
            0
        }
    }

    /// Get the two random digits plus the checksum digit, these are in the
    /// range [20, 999]
    #[inline]
    pub fn get_randoms(&self) -> u32 {
        let randoms = (self.internal & REST_MASK) >> REST_OFFSET;
        debug_assert!((randoms >= 20) && (randoms <= 999));
        randoms
    }

    /// Get the birthday of this kennitala's holder.
    #[cfg(feature = "chrono")]
    pub fn get_birthday(&self) -> NaiveDate {
        NaiveDate::from_ymd(self.get_year() as i32, self.get_month(), self.get_day())
    }
}

impl fmt::Display for Kennitala {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:02}{:02}{:02}{:03}{}",
            self.get_day(),
            self.get_month(),
            self.get_short_year(),
            self.get_randoms(),
            self.get_short_century()
        )
    }
}

fn kt_to_array(kt_integer: u32, array: &mut [u8; 10]) -> Result<(), KennitalaError> {
    let mut n = kt_integer;
    let mut i = 0;
    while n > 0 {
        let digit = n % 10;
        debug_assert!(digit <= 9);
        array[9 - i] = digit as u8;
        n /= 10;
        i += 1
    }
    if i < 9 {
        Err(KennitalaError::InvalidLength(i))
    } else {
        Ok(())
    }
}

// This function can return the number 10, which is not a valid digit in the
// range [0, 9]. That's okay, since the number 10 will not match the checksum
// digit in the given kennitala, so an error will be raised.
fn calculate_checksum_digit(kennitala: &[u8; 10]) -> u8 {
    let mut sum: u32 = 0;
    for i in 0..8 {
        sum += (kennitala[i] * VALIDATION_DIGITS[i]) as u32;
    }
    let sum_mod_11 = sum % 11;
    let digit = if sum_mod_11 == 0 { 0 } else { 11 - sum_mod_11 };
    debug_assert!(digit <= 10);
    digit as u8
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::string::ToString;

    #[test]
    fn my_own_kennitala() {
        let my_kennitala = Kennitala::new("3110002920").unwrap();
        assert_eq!(my_kennitala.get_day(), 31);
        assert_eq!(my_kennitala.get_month(), 10);
        assert_eq!(my_kennitala.get_short_year(), 0);
        assert_eq!(my_kennitala.get_short_century(), 0);
        assert_eq!(my_kennitala.get_randoms(), 292);
        assert_eq!(my_kennitala.get_year(), 2000);
        #[cfg(feature = "chrono")]
        {
            let my_birthday = NaiveDate::from_ymd(2000, 10, 31);
            assert_eq!(my_kennitala.get_birthday(), my_birthday);
        }
        assert_eq!(my_kennitala.to_string(), "3110002920");
    }

    #[test]
    fn my_moms_kennitala() {
        let my_moms_kennitala = Kennitala::new("1703715939").unwrap();
        assert_eq!(my_moms_kennitala.get_day(), 17);
        assert_eq!(my_moms_kennitala.get_month(), 03);
        assert_eq!(my_moms_kennitala.get_short_year(), 71);
        assert_eq!(my_moms_kennitala.get_short_century(), 9);
        assert_eq!(my_moms_kennitala.get_randoms(), 593);
        assert_eq!(my_moms_kennitala.get_year(), 1971);
        #[cfg(feature = "chrono")]
        {
            let my_moms_birthday = NaiveDate::from_ymd(1971, 03, 17);
            assert_eq!(my_moms_kennitala.get_birthday(), my_moms_birthday);
        }
        assert_eq!(my_moms_kennitala.to_string(), "1703715939");
    }

    #[test]
    fn made_up_kennitala() {
        let kt = Kennitala::new("0311203149").unwrap();
        assert_eq!(kt.get_day(), 3);
        assert_eq!(kt.get_month(), 11);
        assert_eq!(kt.get_short_year(), 20);
        assert_eq!(kt.get_short_century(), 9);
        assert_eq!(kt.get_randoms(), 314);
        assert_eq!(kt.get_year(), 1920);
        #[cfg(feature = "chrono")]
        {
            let my_moms_birthday = NaiveDate::from_ymd(1920, 11, 3);
            assert_eq!(kt.get_birthday(), my_moms_birthday);
        }
        assert_eq!(kt.to_string(), "0311203149");
    }

    #[test]
    fn max_u32() {
        let kt = Kennitala::new(&std::u32::MAX.to_string());
        assert!(kt.is_err());
    }

    #[test]
    fn failed_fuzz_1() {
        let kt = Kennitala::new("3999999999");
        assert!(kt.is_err());
    }

    #[test]
    fn failed_fuzz_2() {
        let kt = Kennitala::new("9999");
        assert!(kt.is_err());
    }

    #[test]
    fn failed_fuzz_3() {
        let kt = Kennitala::new("01011413300");
        assert!(kt.is_err());
    }
}
