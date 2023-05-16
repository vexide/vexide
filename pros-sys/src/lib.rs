#![no_std]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

extern "C" {
    pub fn memalign(alignment: usize, size: usize) -> *mut core::ffi::c_void;
}

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
