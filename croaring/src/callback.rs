use std::any::Any;
use std::ops::ControlFlow;
use std::panic::AssertUnwindSafe;
use std::{panic, ptr};

pub struct CallbackWrapper<F, O> {
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

    unsafe extern "C" fn raw_callback<I>(value: I, arg: *mut std::ffi::c_void) -> bool
    where
        I: panic::UnwindSafe,
        F: FnMut(I) -> ControlFlow<O>,
    {
        let wrapper = &mut *(arg as *mut Self);
        let mut f = AssertUnwindSafe(&mut wrapper.f);
        let result = panic::catch_unwind(move || f(value));
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
        unsafe extern "C" fn(I, *mut std::ffi::c_void) -> bool,
        *mut std::ffi::c_void,
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
