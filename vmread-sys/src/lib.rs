#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use libc::*;
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

unsafe impl Send for WinCtx {}
unsafe impl Sync for WinCtx {}
unsafe impl Send for WinProc {}
unsafe impl Sync for WinProc {}
unsafe impl Send for WinModule {}
unsafe impl Sync for WinModule {}
