#![no_main]

use croaring::{Bitmap, Bitmap64, Native, Portable};
use libfuzzer_sys::fuzz_target;
use libfuzzer_sys::arbitrary::{self, Arbitrary};

fn check_bitmap<D: croaring::bitmap::Deserializer>(input: &[u8]) {
    let bitmap = Bitmap::try_deserialize::<D>(input);
    if let Some(mut bitmap) = bitmap {
        bitmap.internal_validate().unwrap();

        let unsafe_version = unsafe { D::try_deserialize_unchecked(input) };
        assert_eq!(
            bitmap,
            unsafe_version,
            "Unsafe doesn't match safe {}",
            std::any::type_name::<D>()
        );

        let start_cardinality = bitmap.cardinality();
        let mut new_cardinality = start_cardinality;
        for i in 100..1000 {
            if !bitmap.contains(i) {
                bitmap.add(i);
                new_cardinality += 1;
            }
        }
        assert_eq!(
            new_cardinality,
            bitmap.cardinality(),
            "Cardinality mismatch in {}",
            std::any::type_name::<D>()
        );
    }
}

fn check_bitmap64<D: croaring::bitmap64::Deserializer>(input: &[u8]) {
    let bitmap = Bitmap64::try_deserialize::<D>(input);
    if let Some(mut bitmap) = bitmap {
        bitmap.internal_validate().unwrap();

        let unsafe_version = unsafe { D::try_deserialize_unchecked(input) };
        assert_eq!(
            bitmap,
            unsafe_version,
            "Unsafe doesn't match safe {}",
            std::any::type_name::<D>()
        );

        let start_cardinality = bitmap.cardinality();
        let mut new_cardinality = start_cardinality;
        for i in 100..1000 {
            if !bitmap.contains(i) {
                bitmap.add(i);
                new_cardinality += 1;
            }
        }
        assert_eq!(
            new_cardinality,
            bitmap.cardinality(),
            "Cardinality mismatch in {}",
            std::any::type_name::<D>()
        );
    }
}

#[derive(Arbitrary, Debug)]
enum BitmapType {
    Portable32,
    Native32,
    Portable64,
}

fuzz_target!(|input: (BitmapType, &[u8])| {
    let (ty, input) = input;
    match ty {
        BitmapType::Portable32 => check_bitmap::<Portable>(input),
        BitmapType::Native32 => check_bitmap::<Native>(input),
        BitmapType::Portable64 => check_bitmap64::<Portable>(input),
    }
});
