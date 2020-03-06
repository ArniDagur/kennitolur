#![no_main]
use kennitolur::Kennitala;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(string) = std::str::from_utf8(data) {
        match Kennitala::new(string) {
            Ok(kt) => {
                kt.get_day();
                kt.get_month();
                kt.get_short_year();
                kt.get_short_century();
                kt.get_randoms();
                kt.get_year();
                assert_eq!(kt.to_string(), string);
            }
            Err(_) => {}
        }
    }
});
