use croaring::{Bitmap64, Portable};
use std::fs;

fn expected_serialized_bitmap() -> Bitmap64 {
    let mut bitmap = Bitmap64::new();

    for i in 0..2u64 {
        let base = i << 32;
        // Range container
        bitmap.add_range(base | 0x0_0000..=base | 0x0_9000);
        bitmap.add_range(base | 0x0_A000..=base | 0x1_0000);
        // Array container
        bitmap.add(base | 0x2_0000);
        bitmap.add(base | 0x2_0005);
        // Bitmap container
        for j in (0..0x1_0000).step_by(2) {
            bitmap.add(base | 0x80000 + j);
        }
    }
    bitmap
}

#[test]
fn test_portable_deserialize() {
    let buffer = fs::read("tests/data/portable_bitmap64.bin").unwrap();
    let bitmap = Bitmap64::deserialize::<Portable>(&buffer);
    let expected = expected_serialized_bitmap();
    assert_eq!(bitmap, expected);
    assert!(bitmap.iter().eq(expected.iter()))
}
