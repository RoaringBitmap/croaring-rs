#![no_main]

use croaring::{Bitmap64, Portable};
use libfuzzer_sys::fuzz_target;
use std::hint::black_box;

fn check_bitmap<D: croaring::bitmap64::Deserializer>(input: &[u8]) {
    let bitmap = Bitmap64::try_deserialize::<D>(input);
    _ = black_box(bitmap);
    /*
    if let Some(mut bitmap) = bitmap {
        bitmap.internal_validate().unwrap();

        let start_cardinality = bitmap.cardinality();
        let mut new_cardinality = start_cardinality;
        for i in 100..1000 {
            if !bitmap.contains(i) {
                bitmap.add(i);
                new_cardinality += 1;
            }
        }
        assert_eq!(new_cardinality, bitmap.cardinality());

        let unsafe_version = unsafe { D::try_deserialize_unchecked(input) };
        assert_eq!(bitmap, unsafe_version);
    }
     */
}

fuzz_target!(|input: &[u8]| {
    check_bitmap::<Portable>(input);
});
