#![cfg(feature = "std")]

use alloc::boxed::Box;
use core::any::Any;
use core::ops::ControlFlow;
use core::panic::AssertUnwindSafe;
use core::{panic, ptr};

pub(crate) struct CallbackWrapper<F, O> {
    f: F,
    result: Result<ControlFlow<O>, Box<dyn Any + Send + 'static>>,
}

impl<F, O> CallbackWrapper<F, O> {
    pub fn new(f: F) -> Self {
        Self {
            f,
            result: Ok(ControlFlow::Continue(())),
        }
    }

    unsafe extern "C" fn raw_callback<I>(value: I, arg: *mut core::ffi::c_void) -> bool
    where
        F: FnMut(I) -> ControlFlow<O>,
        I: panic::UnwindSafe,
    {
        let wrapper = &mut *(arg as *mut Self);
        let f = &mut wrapper.f;
        let f = AssertUnwindSafe(|| f(value));
        let result = std::panic::catch_unwind(f);
        match result {
            Ok(ControlFlow::Continue(())) => true,
            Ok(cf @ ControlFlow::Break(_)) => {
                wrapper.result = Ok(cf);
                false
            }
            Err(err) => {
                wrapper.result = Err(err);
                false
            }
        }
    }

    pub fn callback_and_ctx<I>(
        &mut self,
    ) -> (
        unsafe extern "C" fn(I, *mut core::ffi::c_void) -> bool,
        *mut core::ffi::c_void,
    )
    where
        I: panic::UnwindSafe,
        F: FnMut(I) -> ControlFlow<O>,
    {
        (Self::raw_callback::<I>, ptr::addr_of_mut!(*self).cast())
    }

    pub fn result(self) -> Result<ControlFlow<O>, Box<dyn Any + Send + 'static>> {
        self.result
    }
}
