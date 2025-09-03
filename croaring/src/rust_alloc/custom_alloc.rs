use super::{layout, AlignedLayout};
use allocator_api2::alloc::Allocator;
use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use core::ptr::NonNull;

unsafe extern "C" fn malloc(size: usize) -> *mut core::ffi::c_void {
    let Some(layout) = layout(size) else {
        return core::ptr::null_mut();
    };
    let allocator = ALLOCATOR.get();
    let Ok(ptr) = allocator.allocate(layout) else {
        return core::ptr::null_mut();
    };
    let size_ptr = ptr.cast::<usize>();
    size_ptr.write(size);
    size_ptr.add(1).cast().as_ptr()
}

unsafe extern "C" fn calloc(nmemb: usize, size: usize) -> *mut core::ffi::c_void {
    let Some(total_size) = nmemb.checked_mul(size) else {
        return core::ptr::null_mut();
    };
    let Some(layout) = layout(total_size) else {
        return core::ptr::null_mut();
    };
    let allocator = ALLOCATOR.get();
    let Ok(ptr) = allocator.allocate_zeroed(layout) else {
        return core::ptr::null_mut();
    };
    let size_ptr = ptr.cast::<usize>();
    size_ptr.write(total_size);
    size_ptr.add(1).cast().as_ptr()
}

unsafe extern "C" fn realloc(ptr: *mut core::ffi::c_void, size: usize) -> *mut core::ffi::c_void {
    let Some(ptr) = NonNull::new(ptr) else {
        return malloc(size);
    };
    let ptr = ptr.cast::<u8>();
    let size_ptr = ptr.cast::<usize>().sub(1);
    let old_size = size_ptr.read();
    let old_layout = layout(old_size).unwrap();
    let allocator = ALLOCATOR.get();
    if size == 0 {
        allocator.deallocate(ptr, old_layout);
        return core::ptr::null_mut();
    }
    let Some(new_layout) = layout(size) else {
        return core::ptr::null_mut();
    };
    let Ok(new_ptr) = allocator.grow(ptr, old_layout, new_layout) else {
        return core::ptr::null_mut();
    };
    let new_size_ptr = new_ptr.cast::<usize>();
    new_size_ptr.write(size);
    new_size_ptr.add(1).cast().as_ptr()
}

unsafe extern "C" fn free(ptr: *mut core::ffi::c_void) {
    let Some(ptr) = NonNull::new(ptr) else {
        return;
    };
    let size_ptr = ptr.cast::<usize>().sub(1);
    let size = size_ptr.read();
    // If the size would overflow, it would have failed to be allocated in the first place.
    let layout = layout(size).unwrap();
    let allocator = ALLOCATOR.get();
    allocator.deallocate(size_ptr.cast(), layout);
}

unsafe extern "C" fn aligned_malloc(align: usize, size: usize) -> *mut core::ffi::c_void {
    let Some(layout) = AlignedLayout::new(size, align) else {
        return core::ptr::null_mut();
    };
    let allocator = ALLOCATOR.get();
    let Ok(allocated_ptr) = allocator.allocate(layout.0) else {
        return core::ptr::null_mut();
    };
    layout
        .store_and_return(allocated_ptr.cast().as_ptr())
        .cast()
}

unsafe extern "C" fn aligned_free(ptr: *mut core::ffi::c_void) {
    let Some(ptr) = NonNull::new(ptr) else {
        return;
    };
    let (allocated_ptr, layout) = AlignedLayout::from_raw(ptr.cast().as_ptr());
    let allocated_ptr = NonNull::new_unchecked(allocated_ptr);
    let allocator = ALLOCATOR.get();
    allocator.deallocate(allocated_ptr.cast(), layout.0);
}

const MEMORY_HOOKS: ffi::roaring_memory_t = ffi::roaring_memory_t {
    malloc: Some(malloc),
    realloc: Some(realloc),
    calloc: Some(calloc),
    free: Some(free),
    aligned_malloc: Some(aligned_malloc),
    aligned_free: Some(aligned_free),
};

struct AllocatorSlot(UnsafeCell<MaybeUninit<&'static dyn Allocator>>);

impl AllocatorSlot {
    pub const fn new() -> Self {
        Self(UnsafeCell::new(MaybeUninit::uninit()))
    }

    pub unsafe fn get(&self) -> &'static dyn Allocator {
        (*self.0.get()).assume_init()
    }

    pub unsafe fn set(&self, allocator: &'static dyn Allocator) {
        (*self.0.get()).write(allocator);
    }
}

// All access to the inner value is unsafe, so we implement Sync unsafely
unsafe impl Sync for AllocatorSlot {}

static ALLOCATOR: AllocatorSlot = AllocatorSlot::new();

/// Install custom memory allocation hooks for `CRoaring` which will use the passed allocator.
///
/// # Safety
///
/// The caller must ensure there are not any objects allocated by `CRoaring` at the time this
/// function is called.
///
/// Ideally, this function should be called early in the program's execution, before any other
/// `CRoaring` functions are called.
pub unsafe fn configure_custom_alloc(allocator: &'static dyn Allocator) {
    ALLOCATOR.set(allocator);
    ffi::roaring_init_memory_hook(MEMORY_HOOKS);
}
