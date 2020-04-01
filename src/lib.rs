

//! A library for reading and writing windows memory running on a KVM-based virtual machine
//!
//! ## Feature flags
//!
//! vmread uses a set of [feature flags](https://doc.rust-lang.org/cargo/reference/manifest.html#the-features-section)
//! to switch between different modes of operation. This is to allow maximum performance in given
//! circumstances. Currently there are 3 available modes:
//!
//! - `default`: Uses system calls to perform memory read/write operations. It is the safest option
//! available, although rather slow.
//! - `internal_rw`: Accesses memory directly. This is meant for shared libraries that get loaded
//! into the KVM process (usually qemu-system-x86_64). This is the least safe option, and is very
//! inconsistent to pull off across various system installations.
//! - `kmod_rw`: With the help of a kernel module we are able to map the entirety of KVM address
//! space into our current address space and access it directly. It is a great blend between the
//! default and internal modes, and is the best way forward if running custom kernel modules is an
//! option.
//!
//! ## Example
//!
//! A simple process list:
//!
//! ```no_run
//! extern crate vmread;
//!
//! fn main() {
//!     let ctx_ret = vmread::create_context(0);
//!
//!     if ctx_ret.is_ok() {
//!         let (mut ctx, _) = ctx_ret.unwrap();
//!         println!("VMRead initialized!");
//!
//!         println!("Process List:\nPID\tVIRT\t\t\tPHYS\t\tBASE\t\tNAME");
//!         for i in &(ctx.refresh_processes().process_list) {
//!            println!("{:#4x}\t{:#16x}\t{:#9x}\t{:#9x}\t{}", i.proc.pid, i.proc.process, i.proc.physProcess, i.proc.dirBase, i.name); 
//!         }
//!     } else {
//!         let (eval, estr) = ctx_ret.err().unwrap();
//!         println!("Initialization error {}: {}", eval, estr);
//!     }
//! }
//! ```
//! 

#[cfg(not(any(feature="kmod_rw", feature="internal_rw")))]
pub extern crate vmread_sys as sys;
#[cfg(feature="internal_rw")]
pub extern crate vmread_sys_internal as sys;
#[cfg(feature="kmod_rw")]
pub extern crate vmread_sys_kmod as sys;

mod win_context;
mod win_process;
mod win_dll;
mod win_export;
mod rwlist;

pub use self::win_context::*;
pub use self::win_process::*;
pub use self::win_dll::*;
pub use self::win_export::*;
pub use self::rwlist::*;

#[cfg(feature="internal_rw")]
extern crate libc;


