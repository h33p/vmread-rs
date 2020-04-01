
use crate::win_process::*;
use crate::win_dll::*;
use crate::rwlist::*;

/// Context describing a particular VM instance
///
/// This structure provides interfaces to parse windows process information and to perform reads and
/// writes to memory of the VM.
///
/// There is no `new` implementation, use `create_context` to retrieve an initialized context.
pub struct WinContext {
    ctx: sys::WinCtx,
    pub process_list: Vec<WinProcess>,
    pub kmod_list: Vec<WinDll>,
}

#[cfg(feature="internal_rw")]
fn set_vmread_dfile() {
        unsafe {
            sys::vmread_dfile = libc::fopen("/tmp/vmread_out.txt".as_bytes().as_ptr() as *const i8, "w".as_bytes().as_ptr() as *const i8)
        };
}

#[cfg(not(feature="internal_rw"))]
fn set_vmread_dfile() {}

/// Initialize a new vmread context based on the specified process ID.
///
/// Returns a tuple containing high-level and C contexts on success;
/// Or a tuple containing the error number and string description on error
///
/// # Arguments
///
/// * `pid` - target process ID. Value of 0 indicates automatic detection
pub fn create_context(pid : i32) -> Result<(WinContext, sys::WinCtx), (i32, &'static str)> {
    let mut ctx = sys::WinCtx::default();

    set_vmread_dfile();

    let err = unsafe { sys::InitializeContext(&mut ctx, pid) };

    match err {
        0 => {
            Ok((WinContext {
                ctx: ctx,
                process_list: vec![],
                kmod_list: vec![],
            }, ctx))
        },
        e => Err((e, match e {
            -1 => "Failed to find VM process",
            1 => "Failed to parse memory maps",
            2 => "Failed to find largest memory map",
            3 => "CheckLow fail",
            4 => "FindNTKernel fail",
            5 => "GenerateExportList fail",
            6 => "Find PsInitialSystemProcess fail",
            7 => "Failed to read PsInitialSystemProcess",
            8 => "GetNTVersion/GetNTBuild fail",
            9 => "SetupOffsets fail",
            100 => "Kernel module connection fail",
            101 => "VM mapping fail",
            _ => "Unknown error"
        }))
    }
}

impl Drop for WinContext {
    fn drop(&mut self) {
        unsafe {
            sys::FreeContext(&mut self.ctx);
        }
    }
}

impl WinContext {
    /// Get a read/write list for physical VM memory
    ///
    /// If multiple RW operations are to be performed at the same time, it is more efficient to use RWList
    /// for the task
    pub fn rwlist(&self) -> RWList {
        RWList::new(&self.ctx, 0)
    }

    /// Read physical VM memory
    ///
    /// Returns a value of type `T` at a given VM's physical address
    ///
    /// # Arguments
    /// 
    /// * `address` - address to read the data from
    pub fn read<T>(self, address: u64) -> T {
        let mut ret : T = unsafe { std::mem::MaybeUninit::uninit().assume_init() };

        unsafe {
            sys::MemRead(&self.ctx.process, &mut ret as *mut T as u64, address, std::mem::size_of::<T>() as u64);
        }

        ret
    }

    /// Write physical VM memory
    ///
    /// Write `value` into a given VM's physical address
    ///
    /// # Arguments
    ///
    /// * `address` - address to write the data to
    /// * `value` - reference to the value that is to be written
    pub fn write<T>(&self, address: u64, value: &T) -> &WinContext {
        unsafe {
            sys::MemWrite(&self.ctx.process, value as *const T as u64, address, std::mem::size_of::<T>() as u64);
        }

        self
    }

    /// Refresh the process list
    pub fn refresh_processes(&mut self) -> &mut Self {
        let c_list = unsafe { sys::GenerateProcessList(&self.ctx) };

        self.process_list.clear();
        self.process_list.reserve(c_list.size as usize);

        let lslice = unsafe { std::slice::from_raw_parts(c_list.list, c_list.size as usize) };

        for i in lslice.iter() {
            self.process_list.push(WinProcess::new(*i));
        }

        unsafe {
            sys::FreeProcessList(c_list);
        }

        self
    }

    /// Refresh the kernel module list
    ///
    /// # Remarks
    ///
    /// The kernel modules are not loaded into all processes,
    /// and not all of them are loaded into the system process either.
    pub fn refresh_kmods(&mut self) -> &mut Self {
        let c_list = unsafe { sys::GenerateKernelModuleList(&self.ctx) };

        self.kmod_list.clear();
        self.kmod_list.reserve(c_list.size as usize);

        let lslice = unsafe { std::slice::from_raw_parts(c_list.list, c_list.size as usize) };

        for i in lslice.iter() {
            self.kmod_list.push(WinDll::new(*i));
        }

        unsafe {
            sys::FreeModuleList(c_list);
        }

        self
    }
}
