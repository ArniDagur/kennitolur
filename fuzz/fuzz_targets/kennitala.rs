#![no_main]
use libfuzzer_sys::fuzz_target;
use kennitolur::Kennitala;

fuzz_target!(|data: &[u8]| {
    if let Ok(string) = std::str::from_utf8(data) {
        let _ = Kennitala::new(string);
    }
});