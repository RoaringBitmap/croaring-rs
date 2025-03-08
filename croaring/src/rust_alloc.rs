use alloc::alloc::Layout;

/// Install custom memory allocation hooks for `CRoaring` which will use rust's global allocator.
///
/// # Safety
///
/// The caller must ensure there are not any objects allocated by `CRoaring` at the time this
/// function is called.
///
/// Ideally, this function should be called early in the program's execution, before any other
/// `CRoaring` functions are called.
pub unsafe fn configure_rust_alloc() {
    ffi::roaring_init_memory_hook(MEMORY_HOOKS);
}

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

unsafe extern "C" fn malloc(size: usize) -> *mut core::ffi::c_void {
    let Some(layout) = layout(size) else {
        return core::ptr::null_mut();
    };
    let ptr = alloc::alloc::alloc(layout);
    if ptr.is_null() {
        return ptr.cast();
    }
    let size_ptr = ptr.cast::<usize>();
    size_ptr.write(size);
    size_ptr.add(1).cast()
}

unsafe extern "C" fn calloc(nmemb: usize, size: usize) -> *mut core::ffi::c_void {
    let Some(total_size) = nmemb.checked_mul(size) else {
        return core::ptr::null_mut();
    };
    let Some(layout) = layout(total_size) else {
        return core::ptr::null_mut();
    };
    let ptr = alloc::alloc::alloc_zeroed(layout);
    if ptr.is_null() {
        return core::ptr::null_mut();
    }
    let size_ptr = ptr.cast::<usize>();
    size_ptr.write(total_size);
    size_ptr.add(1).cast()
}

unsafe extern "C" fn realloc(ptr: *mut core::ffi::c_void, size: usize) -> *mut core::ffi::c_void {
    if ptr.is_null() {
        return malloc(size);
    }
    let size_ptr = ptr.cast::<usize>().sub(1);
    let old_size = size_ptr.read();
    let old_layout = layout(old_size).unwrap();
    if size == 0 {
        alloc::alloc::dealloc(size_ptr.cast::<u8>(), old_layout);
        return core::ptr::null_mut();
    }
    let new_ptr = alloc::alloc::realloc(size_ptr.cast(), old_layout, size + size_of::<usize>());
    if new_ptr.is_null() {
        return core::ptr::null_mut();
    }
    let new_size_ptr = new_ptr.cast::<usize>();
    new_size_ptr.write(size);
    new_size_ptr.add(1).cast()
}

unsafe extern "C" fn free(ptr: *mut core::ffi::c_void) {
    if ptr.is_null() {
        return;
    }
    let size_ptr = ptr.cast::<usize>().sub(1);
    let size = size_ptr.read();
    // If the size would
    let layout = layout(size).unwrap();
    alloc::alloc::dealloc(size_ptr.cast(), layout);
}

unsafe extern "C" fn aligned_malloc(align: usize, size: usize) -> *mut core::ffi::c_void {
    let Some(layout) = AlignedLayout::new(size, align) else {
        return core::ptr::null_mut();
    };
    let allocated_ptr = alloc::alloc::alloc(layout.0);
    if allocated_ptr.is_null() {
        return core::ptr::null_mut();
    }
    layout.store_and_return(allocated_ptr).cast()
}

unsafe extern "C" fn aligned_free(ptr: *mut core::ffi::c_void) {
    if ptr.is_null() {
        return;
    }
    let (allocated_ptr, layout) = AlignedLayout::from_raw(ptr);
    alloc::alloc::dealloc(allocated_ptr.cast(), layout.0);
}

const MEMORY_HOOKS: ffi::roaring_memory_t = ffi::roaring_memory_t {
    malloc: Some(malloc),
    realloc: Some(realloc),
    calloc: Some(calloc),
    free: Some(free),
    aligned_malloc: Some(aligned_malloc),
    aligned_free: Some(aligned_free),
};

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
fn impossible_aligned_alloc() {
    unsafe {
        let ptr = aligned_malloc(usize::MAX, usize::MAX);
        assert!(ptr.is_null());

        let max_pow_2 = 1usize << (size_of::<usize>() * 8 - 1);
        let ptr = aligned_malloc(max_pow_2, max_pow_2);
        assert!(ptr.is_null());

        let max_pow_2_isize = 1usize << (size_of::<usize>() * 8 - 2);
        let ptr = aligned_malloc(max_pow_2_isize, max_pow_2_isize);
        assert!(ptr.is_null());
    }
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
