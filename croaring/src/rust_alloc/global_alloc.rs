use super::{layout, AlignedLayout};

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
    // If the size would overflow, it would have failed to be allocated in the first place.
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
