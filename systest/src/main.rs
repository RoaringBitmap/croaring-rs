#![allow(bad_style)]

extern crate croaring_sys;
extern crate libc;

use libc::*;
use croaring_sys::*;

include!(concat!(env!("OUT_DIR"), "/all.rs"));
