
use crate::win_export::*;

/// Represents a single Windows process module
///
/// Has basic information about the module in `info`, and a function to retrieve list of its exports.
#[derive(Clone)]
pub struct WinDll {
    pub name: String,
    pub info: sys::WinModule,
    pub export_list: Vec<WinExport>
}

impl WinDll {
    pub fn new(info: sys::WinModule) -> WinDll {
        let mut ret = WinDll{
            info: info,
            name: unsafe { std::ffi::CStr::from_ptr(info.name).to_str().unwrap_or("").to_string() },
            export_list: vec![],
        };
        
        ret.info.name = std::ptr::null_mut::<i8>();

        ret
    }

    /// Refresh the export list for the module
    ///
    /// # Arguments
    ///
    /// * `proc` - target process
    /// * `ctx` - vmread C context
    pub fn refresh_exports(&mut self, proc: &sys::WinProc, ctx: sys::WinCtx) -> &mut Self {
        let mut c_list = sys::WinExportList {
            list: std::ptr::null_mut(),
            size: 0 as u64
        };

        unsafe {
            sys::GenerateExportList(&ctx, proc, self.info.baseAddress, &mut c_list);
        }

        self.export_list.clear();
        self.export_list.reserve(c_list.size as usize);

        let lslice = unsafe { std::slice::from_raw_parts(c_list.list, c_list.size as usize) };

        for i in lslice.iter() {
            self.export_list.push(WinExport::new(*i));
        }

        unsafe {
            sys::FreeExportList(c_list);
        }

        self
    }

}
