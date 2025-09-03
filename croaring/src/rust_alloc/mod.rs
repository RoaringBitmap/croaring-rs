#[cfg(not(feature = "allocator-api2"))]
use alloc::alloc::Layout;
#[cfg(feature = "allocator-api2")]
use allocator_api2::alloc::Layout;

#[cfg(feature = "alloc")]
mod global_alloc;
#[cfg(feature = "alloc")]
pub use global_alloc::configure_rust_alloc;
#[cfg(feature = "allocator-api2")]
mod custom_alloc;
#[cfg(feature = "allocator-api2")]
pub use custom_alloc::configure_custom_alloc;

fn layout(size: usize) -> Option<Layout> {
    if size == 0 {
        return None;
    }
    let (layout, _) = Layout::new::<usize>()
        .extend(Layout::array::<u8>(size).ok()?)
        .ok()?;
    Some(layout)
}

struct AlignedLayout(Layout);

#[repr(C)]
struct SizeAlign {
    size: usize,
    align: usize,
}

const fn padding_for_align(align: usize) -> usize {
    align.saturating_sub(size_of::<SizeAlign>())
}

impl AlignedLayout {
    fn new(size: usize, align: usize) -> Option<Self> {
        if size == 0 || align == 0 || !align.is_power_of_two() {
            return None;
        }
        // Will store two usizes before the data: the size and alignment.
        // Additionally, there may be additional padding to ensure the data is aligned to
        // the required alignment.
        let align = align.max(align_of::<SizeAlign>());
        let padding = padding_for_align(align);
        let size = padding
            .checked_add(size_of::<SizeAlign>())?
            .checked_add(size)?;
        let layout = Layout::from_size_align(size, align).ok()?;
        debug_assert_eq!((padding + size_of::<SizeAlign>()) % align, 0);
        Some(Self(layout))
    }

    const fn padding(&self) -> usize {
        padding_for_align(self.0.align())
    }

    unsafe fn store_and_return(&self, allocated_ptr: *mut u8) -> *mut u8 {
        let size_ptr = allocated_ptr.add(self.padding()).cast::<SizeAlign>();
        size_ptr.write(SizeAlign {
            size: self.0.size(),
            align: self.0.align(),
        });
        size_ptr.add(1).cast()
    }

    unsafe fn from_raw(raw_ptr: *mut core::ffi::c_void) -> (*mut core::ffi::c_void, Self) {
        let size_ptr = raw_ptr.cast::<SizeAlign>().sub(1);
        let SizeAlign { size, align } = size_ptr.read();
        let padding = padding_for_align(align);
        let orig_ptr = size_ptr.cast::<u8>().sub(padding);
        let layout = Layout::from_size_align_unchecked(size, align);
        (orig_ptr.cast(), Self(layout))
    }
}

#[test]
fn test_aligned_layout_no_padding() {
    let aligned_layout = AlignedLayout::new(10, 2 * size_of::<usize>()).unwrap();
    assert_eq!(aligned_layout.padding(), 0);
    assert_eq!(aligned_layout.0.size(), 10 + size_of::<SizeAlign>());
    assert_eq!(aligned_layout.0.align(), 16);
}

#[test]
fn test_aligned_layout_big_align() {
    let aligned_layout = AlignedLayout::new(10, 1024).unwrap();
    assert_eq!(aligned_layout.padding(), 1024 - size_of::<SizeAlign>());
    assert_eq!(aligned_layout.0.size(), 10 + 1024);
    assert_eq!(aligned_layout.0.align(), 1024);
}

#[test]
fn aligned_layout_big() {
    let align = 0x2000000000000000;
    let layout = AlignedLayout::new(0x2000000000000000, align).unwrap();
    assert_ne!(layout.0.size(), 0);
    // The beginning of the allocation is aligned to at least the required alignment.
    assert_eq!(layout.0.align() % align, 0);
    // The beginning of the data is aligned to at least the required alignment.
    assert_eq!((layout.padding() + size_of::<SizeAlign>()) % align, 0);
}

// run with `cargo kani`
#[cfg(kani)]
#[kani::proof]
fn aligned_layout() {
    let size = kani::any();
    let align = kani::any();
    if let Some(layout) = AlignedLayout::new(size, align) {
        assert!(layout.0.size() != 0);
        // The beginning of the allocation is aligned to at least the required alignment.
        assert!(layout.0.align() % align == 0);
        // The size and align after the padding are aligned enough
        assert!(layout.padding() % align_of::<SizeAlign>() == 0);
        // The beginning of the data is aligned to at least the required alignment.
        assert!((layout.padding() + size_of::<SizeAlign>()) % align == 0);
        // There is enough space after the padding and size/align to store `size` bytes
        assert!(layout.0.size() - (layout.padding() + size_of::<SizeAlign>()) >= size);
    }
}
