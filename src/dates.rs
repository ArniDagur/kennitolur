const DAYS_IN_MONTH: [u8; 12] = [
    31, // January
    28, // Febuary
    31, // March
    30, // April
    31, // May
    30, // June
    31, // July
    31, // August
    30, // September
    31, // October
    30, // November
    31, // December
];

#[inline]
fn is_leap_year(year: u16) -> bool {
    (year % 4 == 0) && (year % 100 != 0 || year % 400 == 0)
}

#[inline]
pub fn days_in_month(month: u8, year: u16) -> u8 {
    if (month == 2) && is_leap_year(year) {
        29
    } else {
        DAYS_IN_MONTH[(month - 1) as usize]
    }
}
