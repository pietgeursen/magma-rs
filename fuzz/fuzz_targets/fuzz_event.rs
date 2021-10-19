#![no_main]

use libfuzzer_sys::fuzz_target;
use magma_rs::*; 
use blake2::Blake2b;

type MyEvent = Event<Blake2b>;

fuzz_target!(|data: &[u8]| {
    match MyEvent::decode(data) {
        Err(_) => {}
        Ok(event) => {
            let mut out = Vec::new();
            out.resize(event.encoding_length(), 0);
            let sz = event.encode(&mut out[..]).unwrap();

            assert_eq!(data[..sz], out[..sz]);
        }
    }
});
