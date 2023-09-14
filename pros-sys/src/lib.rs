#![no_std]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

const CLOCKS_PER_SEC: u32 = 1000;


extern "C" {
    pub fn memalign(alignment: usize, size: usize) -> *mut core::ffi::c_void;
    pub fn __errno() -> *mut i32;
    pub fn clock() -> i32;
}

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
