use crate::sys;

use crate::win_dll::*;
use crate::rwlist::*;

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

    pub fn read<T>(self, ctx: &sys::WinCtx, address: u64) -> T {
        let mut ret : T = unsafe { std::mem::MaybeUninit::uninit().assume_init() };

        unsafe {
            sys::VMemRead(&ctx.process, self.proc.dirBase, &mut ret as *mut T as u64, address, std::mem::size_of::<T>() as u64);
        }

        ret
    }

    pub fn write<T>(&self, ctx: &sys::WinCtx, address: u64, value: &T) -> &WinProcess {
        unsafe {
            sys::VMemWrite(&ctx.process, self.proc.dirBase, value as *const T as u64, address, std::mem::size_of::<T>() as u64);
        }

        self
    }

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

    pub fn get_peb(self, ctx: sys::WinCtx) -> sys::_PEB {
        unsafe { sys::GetPeb(&ctx, &self.proc) }
    }

    pub fn get_rwlist(&self, ctx: &sys::WinCtx) -> RWList {
        RWList::new(&ctx, self.proc.dirBase)
    }
}
