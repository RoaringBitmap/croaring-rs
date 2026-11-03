use super::{AlignedLayout, layout};
use allocator_api2::alloc::Allocator;
use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use core::ptr::NonNull;

unsafe extern "C" fn malloc(size: usize) -> *mut core::ffi::c_void {
    unsafe {
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
}

unsafe extern "C" fn calloc(nmemb: usize, size: usize) -> *mut core::ffi::c_void {
    unsafe {
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
}

unsafe extern "C" fn realloc(ptr: *mut core::ffi::c_void, size: usize) -> *mut core::ffi::c_void {
    unsafe {
        let Some(ptr) = NonNull::new(ptr) else {
            return malloc(size);
        };
        let size_ptr = ptr.cast::<usize>().sub(1);
        let allocation_start = size_ptr.cast::<u8>();
        let old_size = size_ptr.read();
        let old_layout = layout(old_size).unwrap();
        let allocator = ALLOCATOR.get();
        if size == 0 {
            allocator.deallocate(allocation_start, old_layout);
            return core::ptr::null_mut();
        }
        let Some(new_layout) = layout(size) else {
            return core::ptr::null_mut();
        };
        let resized = if size >= old_size {
            allocator.grow(allocation_start, old_layout, new_layout)
        } else {
            allocator.shrink(allocation_start, old_layout, new_layout)
        };
        let Ok(new_ptr) = resized else {
            return core::ptr::null_mut();
        };
        let new_size_ptr = new_ptr.cast::<usize>();
        new_size_ptr.write(size);
        new_size_ptr.add(1).cast().as_ptr()
    }
}

unsafe extern "C" fn free(ptr: *mut core::ffi::c_void) {
    unsafe {
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
}

unsafe extern "C" fn aligned_malloc(align: usize, size: usize) -> *mut core::ffi::c_void {
    unsafe {
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
}

unsafe extern "C" fn aligned_free(ptr: *mut core::ffi::c_void) {
    unsafe {
        let Some(ptr) = NonNull::new(ptr) else {
            return;
        };
        let (allocated_ptr, layout) = AlignedLayout::from_raw(ptr.cast().as_ptr());
        let allocated_ptr = NonNull::new_unchecked(allocated_ptr);
        let allocator = ALLOCATOR.get();
        allocator.deallocate(allocated_ptr.cast(), layout.0);
    }
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
        unsafe { (*self.0.get()).assume_init() }
    }

    pub unsafe fn set(&self, allocator: &'static dyn Allocator) {
        unsafe {
            (*self.0.get()).write(allocator);
        }
    }
}

// All access to the inner value is unsafe, so we implement Sync unsafely
unsafe impl Sync for AllocatorSlot {}

static ALLOCATOR: AllocatorSlot = AllocatorSlot::new();

/// Install custom memory allocation hooks for `CRoaring` which will use the passed allocator.
///
/// # Safety
///
/// The caller must ensure that:
/// - there are not any objects allocated by `CRoaring` at the time this function is called.
///   This includes objects allocated while a different allocator was configured, for example by
///   `configure_rust_alloc`: memory must be freed by the allocator which allocated it.
/// - no other thread is calling into `CRoaring`, or either of the `configure_*` functions,
///   while this function runs. The hooks are process-global and are updated without
///   synchronization.
/// - `allocator` can be used from any thread concurrently, as if it were [`Sync`].
///   `CRoaring` objects are `Send`, so they may be allocated, resized, and freed on any thread.
///
/// Ideally, this function should be called early in the program's execution, before any other
/// `CRoaring` functions are called.
pub unsafe fn configure_custom_alloc(allocator: &'static dyn Allocator) {
    unsafe {
        ALLOCATOR.set(allocator);
        ffi::roaring_init_memory_hook(MEMORY_HOOKS);
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;
    use allocator_api2::alloc::{AllocError, Global, Layout};
    use core::ffi::c_void;
    use std::collections::BTreeMap;
    use std::string::String;
    use std::sync::{Mutex, MutexGuard, PoisonError};
    use std::vec::Vec;
    use std::{format, vec};

    struct State {
        live: BTreeMap<usize, Layout>,
        violations: Vec<String>,
    }

    impl State {
        fn check_live(&mut self, op: &str, ptr: NonNull<u8>, layout: Layout) -> bool {
            let addr = ptr.as_ptr() as usize;
            match self.live.get(&addr) {
                Some(live) if *live == layout => true,
                Some(live) => {
                    self.violations.push(format!(
                        "{op}: layout {layout:?} does not match allocation {live:?}"
                    ));
                    false
                }
                None => {
                    self.violations.push(format!(
                        "{op}: {addr:#x} is not the start of a live allocation"
                    ));
                    false
                }
            }
        }

        fn replace(&mut self, old: NonNull<u8>, new: NonNull<[u8]>, new_layout: Layout) {
            self.live.remove(&(old.as_ptr() as usize));
            self.live
                .insert(new.cast::<u8>().as_ptr() as usize, new_layout);
        }
    }

    /// Forwards to `Global`, but refuses (and records) calls that violate the `Allocator`
    /// contract instead of passing them on, so tests fail cleanly rather than corrupting the heap.
    struct TrackingAllocator(Mutex<State>);

    impl TrackingAllocator {
        const fn new() -> Self {
            Self(Mutex::new(State {
                live: BTreeMap::new(),
                violations: Vec::new(),
            }))
        }

        fn state(&self) -> MutexGuard<'_, State> {
            self.0.lock().unwrap_or_else(PoisonError::into_inner)
        }

        fn reset(&self) {
            let mut state = self.state();
            state.live.clear();
            state.violations.clear();
        }

        fn violations(&self) -> Vec<String> {
            self.state().violations.clone()
        }

        fn live_count(&self) -> usize {
            self.state().live.len()
        }
    }

    unsafe impl Allocator for TrackingAllocator {
        fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
            let block = Global.allocate(layout)?;
            self.state()
                .live
                .insert(block.cast::<u8>().as_ptr() as usize, layout);
            Ok(block)
        }

        fn allocate_zeroed(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
            let block = Global.allocate_zeroed(layout)?;
            self.state()
                .live
                .insert(block.cast::<u8>().as_ptr() as usize, layout);
            Ok(block)
        }

        unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
            let mut state = self.state();
            if !state.check_live("deallocate", ptr, layout) {
                return;
            }
            state.live.remove(&(ptr.as_ptr() as usize));
            unsafe { Global.deallocate(ptr, layout) }
        }

        unsafe fn grow(
            &self,
            ptr: NonNull<u8>,
            old_layout: Layout,
            new_layout: Layout,
        ) -> Result<NonNull<[u8]>, AllocError> {
            let mut state = self.state();
            if new_layout.size() < old_layout.size() {
                state.violations.push(format!(
                    "grow: new size {} is smaller than old size {}",
                    new_layout.size(),
                    old_layout.size()
                ));
                return Err(AllocError);
            }
            if !state.check_live("grow", ptr, old_layout) {
                return Err(AllocError);
            }
            let block = unsafe { Global.grow(ptr, old_layout, new_layout)? };
            state.replace(ptr, block, new_layout);
            Ok(block)
        }

        unsafe fn shrink(
            &self,
            ptr: NonNull<u8>,
            old_layout: Layout,
            new_layout: Layout,
        ) -> Result<NonNull<[u8]>, AllocError> {
            let mut state = self.state();
            if new_layout.size() > old_layout.size() {
                state.violations.push(format!(
                    "shrink: new size {} is larger than old size {}",
                    new_layout.size(),
                    old_layout.size()
                ));
                return Err(AllocError);
            }
            if !state.check_live("shrink", ptr, old_layout) {
                return Err(AllocError);
            }
            let block = unsafe { Global.shrink(ptr, old_layout, new_layout)? };
            state.replace(ptr, block, new_layout);
            Ok(block)
        }
    }

    static TRACKER: TrackingAllocator = TrackingAllocator::new();
    static TEST_LOCK: Mutex<()> = Mutex::new(());

    fn with_tracker(body: impl FnOnce()) {
        let _guard = TEST_LOCK.lock().unwrap_or_else(PoisonError::into_inner);
        TRACKER.reset();
        unsafe { ALLOCATOR.set(&TRACKER) };
        body();
        assert_eq!(TRACKER.violations(), Vec::<String>::new());
        assert_eq!(TRACKER.live_count(), 0, "allocations were leaked");
    }

    unsafe fn fill(ptr: *mut c_void, len: usize) {
        for i in 0..len {
            unsafe { ptr.cast::<u8>().add(i).write(i as u8) };
        }
    }

    unsafe fn read(ptr: *mut c_void, len: usize) -> Vec<u8> {
        let mut bytes = vec![0; len];
        for (i, byte) in bytes.iter_mut().enumerate() {
            *byte = unsafe { ptr.cast::<u8>().add(i).read() };
        }
        bytes
    }

    fn pattern(len: usize) -> Vec<u8> {
        let mut bytes = Vec::new();
        for i in 0..len {
            bytes.push(i as u8);
        }
        bytes
    }

    #[test]
    fn realloc_grow_preserves_data() {
        with_tracker(|| unsafe {
            let ptr = malloc(16);
            fill(ptr, 16);
            let grown = realloc(ptr, 64);
            assert_eq!(TRACKER.violations(), Vec::<String>::new());
            assert!(!grown.is_null());
            assert_eq!(read(grown, 16), pattern(16));
            free(grown);
        });
    }

    #[test]
    fn realloc_shrink_preserves_data() {
        with_tracker(|| unsafe {
            let ptr = malloc(64);
            fill(ptr, 64);
            let shrunk = realloc(ptr, 16);
            assert_eq!(TRACKER.violations(), Vec::<String>::new());
            assert!(!shrunk.is_null());
            assert_eq!(read(shrunk, 16), pattern(16));
            free(shrunk);
        });
    }

    #[test]
    fn realloc_same_size() {
        with_tracker(|| unsafe {
            let ptr = malloc(32);
            fill(ptr, 32);
            let same = realloc(ptr, 32);
            assert_eq!(TRACKER.violations(), Vec::<String>::new());
            assert!(!same.is_null());
            assert_eq!(read(same, 32), pattern(32));
            free(same);
        });
    }

    #[test]
    fn realloc_of_calloc_block() {
        with_tracker(|| unsafe {
            let ptr = calloc(4, 8);
            let grown = realloc(ptr, 100);
            assert_eq!(TRACKER.violations(), Vec::<String>::new());
            assert!(!grown.is_null());
            assert_eq!(read(grown, 32), vec![0; 32]);
            free(grown);
        });
    }

    #[test]
    fn realloc_to_zero_frees() {
        with_tracker(|| unsafe {
            let ptr = malloc(16);
            let result = realloc(ptr, 0);
            assert!(result.is_null());
        });
    }

    #[test]
    fn realloc_null_allocates() {
        with_tracker(|| unsafe {
            let ptr = realloc(core::ptr::null_mut(), 24);
            assert!(!ptr.is_null());
            free(ptr);
        });
    }
}
