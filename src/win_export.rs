
/// A structure representing a single Windows module export
#[derive(Clone, Default)]
pub struct WinExport {
    pub name: String,
    pub address: u64,
}

impl WinExport {
    pub fn new(exp: sys::WinExport) -> WinExport {
        WinExport {
            name: unsafe { std::ffi::CStr::from_ptr(exp.name).to_str().unwrap().to_string() },
            address: exp.address,
        }
    }
}
