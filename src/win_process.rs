use crate::sys;

use crate::win_dll::*;
use crate::rwlist::*;

/// Structure representing a Windows process
///
/// The implementation provides necessary functions for reading and writing inside the process, as
/// well as parsing its loaded modules.
#[derive(Clone)]
pub struct WinProcess {
    pub proc: sys::WinProc,
    pub name: String,
    pub module_list: Vec<WinDll>,
}

impl WinProcess {
    pub fn new(proc: sys::WinProc) -> WinProcess {
        let mut ret = WinProcess {
            proc: proc,
            name: unsafe { std::ffi::CStr::from_ptr(proc.name).to_str().unwrap().to_string() },
            module_list: vec![],
        };

        ret.proc.name = std::ptr::null_mut::<i8>();

        ret
    }

    /// Get a read/write list for process virtual memory
    ///
    /// If multiple RW operations are to be performed at the same time, it is more efficient to use RWList
    /// for the task
    ///
    /// # Arguments
    ///
    /// * `ctx` - vmread C context
    pub fn rwlist<'a>(&self, ctx: &'a sys::WinCtx) -> RWList<'a> {
        RWList::new(&ctx, self.proc.dirBase)
    }

    /// Read process virtual memory
    ///
    /// Returns a value of type `T` at a given process' virtual address
    ///
    /// # Arguments
    /// 
    /// * `ctx` - vmread C context
    /// * `address` - address to read the data from
    pub fn read<T>(self, ctx: &sys::WinCtx, address: u64) -> T {
        let mut ret : T = unsafe { std::mem::MaybeUninit::uninit().assume_init() };

        unsafe {
            sys::VMemRead(&ctx.process, self.proc.dirBase, &mut ret as *mut T as u64, address, std::mem::size_of::<T>() as u64);
        }

        ret
    }

    /// Write physical VM memory
    ///
    /// Write `value` into a given process' virtual address
    ///
    /// # Arguments
    ///
    /// * `ctx` - vmread C context
    /// * `address` - address to write the data to
    /// * `value` - reference to the value that is to be written
    pub fn write<T>(&self, ctx: &sys::WinCtx, address: u64, value: &T) -> &WinProcess {
        unsafe {
            sys::VMemWrite(&ctx.process, self.proc.dirBase, value as *const T as u64, address, std::mem::size_of::<T>() as u64);
        }

        self
    }

    /// Refresh process module list
    ///
    /// # Arguments
    ///
    /// * `ctx` - vmread C context
    pub fn refresh_modules(&mut self, ctx: sys::WinCtx) -> &mut Self {
        let c_list = unsafe { sys::GenerateModuleList(&ctx, &self.proc) };

        self.module_list.clear();
        self.module_list.reserve(c_list.size as usize);

        let lslice = unsafe { std::slice::from_raw_parts(c_list.list, c_list.size as usize) };

        for i in lslice.iter() {
            self.module_list.push(WinDll::new(*i));
        }

        unsafe {
            sys::FreeModuleList(c_list);
        }

        self
    }

    /// Get process PEB
    pub fn get_peb(self, ctx: sys::WinCtx) -> sys::_PEB {
        unsafe { sys::GetPeb(&ctx, &self.proc) }
    }
}
