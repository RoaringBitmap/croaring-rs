use croaring::{Bitmap64, Bitmap64View, Frozen, Portable};
use std::fs;

fn init() {
    #[cfg(feature = "alloc")]
    {
        static INIT: std::sync::Once = std::sync::Once::new();
        INIT.call_once(|| unsafe { croaring::configure_rust_alloc() });
    }
}

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
    init();
    let buffer = fs::read("tests/data/portable_bitmap64.bin").unwrap();
    let bitmap = Bitmap64::deserialize::<Portable>(&buffer);
    let expected = expected_serialized_bitmap();
    assert_eq!(bitmap, expected);
    assert!(bitmap.iter().eq(expected.iter()))
}

#[test]
fn test_r64_contains_max() {
    init();
    let mut bitmap = Bitmap64::new();
    assert!(!bitmap.contains_range((u64::MAX - 1)..=u64::MAX));
    bitmap.add(u64::MAX - 1);
    bitmap.add(u64::MAX);
    assert!(bitmap.contains_range((u64::MAX - 1)..=u64::MAX));
}

#[test]
fn test_r64_cursor_reset() {
    init();
    let bitmap = Bitmap64::of(&[0, 1, 100, 1000, u64::MAX]);
    let mut cursor = bitmap.cursor();
    cursor.reset_at_or_after(0);
    assert_eq!(cursor.current(), Some(0));
    cursor.reset_at_or_after(0);
    assert_eq!(cursor.current(), Some(0));

    cursor.reset_at_or_after(101);
    assert_eq!(cursor.current(), Some(1000));
    assert_eq!(cursor.next(), Some(u64::MAX));
    assert_eq!(cursor.next(), None);
    cursor.reset_at_or_after(u64::MAX);
    assert_eq!(cursor.current(), Some(u64::MAX));
    assert_eq!(cursor.next(), None);
}

#[test]
fn empty_intersect_with_range() {
    init();
    let bitmap = Bitmap64::new();
    assert_eq!(0, bitmap.range_cardinality(0..1));
    assert!(!bitmap.intersect_with_range(0..1));
}

#[test]
fn empty_reset_iterator() {
    init();
    let bitmap = Bitmap64::new();
    let mut iterator = bitmap.iter();
    assert_eq!(iterator.peek(), None);
    iterator.reset_at_or_after(0);
    assert_eq!(iterator.peek(), None);
}

#[test]
fn copy_from_view() {
    init();
    let mut orig_bitmap = Bitmap64::of(&[1, 2, 3, 4]);
    orig_bitmap.shrink_to_fit();
    let mut buf = [0; 1024];
    let data: &[u8] = orig_bitmap.try_serialize_into::<Frozen>(&mut buf).unwrap();
    let view: Bitmap64View = unsafe { Bitmap64View::deserialize::<Frozen>(data) }.unwrap();
    view.internal_validate().unwrap();
    assert_eq!(view, orig_bitmap);
    let mut mutable_bitmap: Bitmap64 = view.to_bitmap64();
    assert_eq!(view, mutable_bitmap);
    mutable_bitmap.add(10);
    assert!(!view.contains(10));
    assert!(mutable_bitmap.contains(10));
}
