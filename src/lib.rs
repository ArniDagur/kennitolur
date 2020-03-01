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
use dates::days_in_month;
pub use error::KennitalaError;

const VALIDATION_DIGITS: [u8; 8] = [3, 2, 7, 6, 5, 4, 3, 2];

/// Struct that represents the kennitala of an Icelandic citizen or resident.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Kennitala {
    dob_year: u16,
    dob_month: u8,
    dob_day: u8,
    rest: u16,
}

impl Kennitala {
    /// Create new kennitala object from the given string. Validation is done
    /// beforehand.
    pub fn new(kennitala: &str) -> Result<Self, KennitalaError> {
        let kt_integer = match kennitala.parse::<u32>() {
            Ok(n) => n,
            Err(error) => {
                return Err(KennitalaError::InvalidNumber(error));
            }
        };

        let mut kt_array = [0; 10];
        kt_to_array(kt_integer, &mut kt_array);

        let checksum_digit = kt_array[8];
        let calculated_checksum_digit = calculate_checksum_digit(&kt_array);
        if checksum_digit != calculated_checksum_digit {
            return Err(KennitalaError::InvalidChecksum);
        }

        if ((kt_array[6] * 10) + kt_array[7]) < 20 {
            return Err(KennitalaError::InvalidRandomDigits);
        }

        let century_digit = kt_array[9];
        if !((century_digit == 0) || (century_digit == 9)) {
            return Err(KennitalaError::InvalidCentury);
        }
        let year_offset = if century_digit == 0 { 2000 } else { 1900 };

        let dob_month = kt_array[2] * 10 + kt_array[3];
        if (dob_month > 12) || (dob_month <= 0) {
            return Err(KennitalaError::InvalidMonth);
        }

        let dob_year = (kt_array[4] * 10) as u16 + kt_array[5] as u16 + year_offset;

        let dob_day = kt_array[0] * 10 + kt_array[1];
        if (dob_day > days_in_month(dob_month, dob_year)) || (dob_day <= 0) {
            return Err(KennitalaError::InvalidDay);
        }

        let rest = (kt_integer % 1_00_00) as u16;

        Ok(Self {
            dob_year,
            dob_month,
            dob_day,
            rest,
        })
    }

    /// Get the birthday of this kennitala's holder.
    #[cfg(feature = "chrono")]
    pub fn get_birthday(&self) -> NaiveDate {
        NaiveDate::from_ymd(
            self.dob_year as i32,
            self.dob_month as u32,
            self.dob_day as u32,
        )
    }
}

fn kt_to_array(kt_integer: u32, array: &mut [u8; 10]) {
    let mut n = kt_integer;
    let mut i = 0;
    while n > 0 {
        array[9 - i] = (n % 10) as u8;
        n /= 10;
        i += 1
    }
}

fn calculate_checksum_digit(kt_array: &[u8; 10]) -> u8 {
    let mut sum: u32 = 0;
    for i in 0..8 {
        sum += (kt_array[i] * VALIDATION_DIGITS[i]) as u32;
    }

    let sum_mod_11 = sum % 11;
    let digit = if sum_mod_11 == 0 { 0 } else { 11 - sum_mod_11 } as u8;

    return digit;
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn my_own_kennitala() {
        let my_kennitala = Kennitala::new("3110002920").unwrap();
        let expected = Kennitala {
            dob_year: 2000,
            dob_month: 10,
            dob_day: 31,
            rest: 2920,
        };
        assert_eq!(my_kennitala, expected);
    }

    #[test]
    fn my_moms_kennitala() {
        let my_kennitala = Kennitala::new("1703715939").unwrap();
        let expected = Kennitala {
            dob_year: 1971,
            dob_month: 03,
            dob_day: 17,
            rest: 5939,
        };
        assert_eq!(my_kennitala, expected);
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
}
