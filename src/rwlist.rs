use std::marker::PhantomData;
use smallvec::{SmallVec, smallvec};

/// A list of memory operations to be executed on its destruction
///
/// This provides a more efficient way of performing RW operations when the data is not needed
/// immediately. The operations get cached and executed when the object goes out of scope.
pub struct RWList<'a> {
    process: *const sys::ProcessData,
    dir_base: u64,
    read_list: SmallVec<[sys::RWInfo; 8]>,
    write_list: SmallVec<[sys::RWInfo; 8]>,
    phantom: PhantomData<&'a u8>,
}

impl<'a> RWList<'a> {
    /// Create a new RWList instance
    ///
    /// # Arguments
    ///
    /// * `ctx` - vmread C context
    /// * `dir_base` - virtual address translation entry point. 0 for physical address mode
    pub fn new(ctx: &'a sys::WinCtx, dir_base: u64) -> RWList<'a> {
        RWList {
            process: &ctx.process,
            dir_base: dir_base,
            read_list: smallvec![],
            write_list: smallvec![],
            phantom: PhantomData,
        }
    }

    /// Queue a write operation
    ///
    /// # Arguments
    /// 
    /// * `address` - address to write the data to
    /// * `val` - reference to the value to be written
    pub fn write<T>(&mut self, address: u64, val: &'a T) -> &mut Self {
        self.write_list.push(sys::RWInfo {
            local: val as *const T as u64,
            remote: address,
            size: std::mem::size_of::<T>() as u64
        });
        self
    }

    /// Queue an array write operation
    ///
    /// # Arguments
    /// 
    /// * `address` - address to write the data to
    /// * `val` - reference to the slice to be written
    pub fn write_arr<T>(&mut self, address: u64, val: &'a [T]) -> &mut Self {
        self.write_list.push(sys::RWInfo {
            local: val.as_ptr() as u64,
            remote: address,
            size: (std::mem::size_of::<T>() * val.len()) as u64
        });
        self
    }

    /// Queue a read operation
    ///
    /// # Arguments
    /// 
    /// * `address` - address to read the data from
    /// * `val` - reference to the value to read the data into
    pub fn read<T>(&mut self, address: u64, val: &'a mut T) -> &mut Self {
        self.read_list.push(sys::RWInfo {
            local: val as *mut T as u64,
            remote: address,
            size: std::mem::size_of::<T>() as u64
        });
        self
    }

    /// Queue an array read operation
    ///
    /// # Arguments
    /// 
    /// * `address` - address to read the data from
    /// * `val` - reference to the slice to read the data into
    pub fn read_arr<T>(&mut self, address: u64, val: &'a mut [T]) -> &mut Self {
        self.read_list.push(sys::RWInfo {
            local: val.as_mut_ptr() as u64,
            remote: address,
            size: (std::mem::size_of::<T>() * val.len()) as u64
        });
        self
    }

    /// Perform all cached memory operations
    ///
    /// Both read and write lists get iterated from the starting points and vmread C library gets invoked.
    /// The lists then get truncated to the size of given starting points. The lists work like a
    /// stack, with the latest elements having priority over the older elements.
    ///
    /// # Arguments
    ///
    /// * `read_start` - starting index for read operations
    /// * `write_start` - starting index for write operations
    pub fn commit(&mut self, read_start: usize, write_start: usize) -> (&mut Self, usize, usize) {
        let mut done_rwlen : usize = 0;
        let mut queued_rwlen : usize = 0;

        if read_start < self.read_list.len() {
            {
                let read_list = &mut self.read_list[read_start..];
                read_list.sort_unstable_by(|a, b| (a.remote & !0xfff).partial_cmp(&(b.remote & !0xfff)).unwrap());
                queued_rwlen += read_list.iter().fold(0, |acc, a| acc + a.size) as usize;
               
                done_rwlen += unsafe {
                    (if self.dir_base != 0 {
                        sys::VMemReadMul(self.process, self.dir_base, read_list.as_mut_ptr(), read_list.len() as u64)
                    } else {
                        sys::MemReadMul(self.process, read_list.as_mut_ptr(), read_list.len() as u64)
                    }) as usize
                };
            }

            self.read_list.truncate(read_start);
        }

        if write_start < self.write_list.len() {
            {
                let write_list = &mut self.write_list[write_start..];
                write_list.sort_unstable_by(|a, b| (a.remote & !0xfff).partial_cmp(&(b.remote & !0xfff)).unwrap());
                queued_rwlen += write_list.iter().fold(0, |acc, a| acc + a.size) as usize;

                done_rwlen += unsafe {
                    (if self.dir_base != 0 {
                        sys::VMemWriteMul(self.process, self.dir_base, write_list.as_mut_ptr(), write_list.len() as u64)
                    } else {
                        sys::MemWriteMul(self.process, write_list.as_mut_ptr(), write_list.len() as u64)
                    }) as usize
                } 
            }
            
            self.write_list.truncate(write_start);

        }

        (self, queued_rwlen, done_rwlen)
    }

    /// Commit all RW operations in the list
    pub fn commit_rw(&mut self) -> (&mut Self, usize, usize) {
        self.commit(0, 0)
    }

    /// Commit only read operations in the list
    pub fn commit_read(&mut self) -> (&mut Self, usize, usize) {
        self.commit(0, self.write_list.len())
    }

    /// Commit only write operations in the list
    pub fn commit_write(&mut self) -> (&mut Self, usize, usize) {
        self.commit(self.read_list.len(), 0)
    }

}

impl Drop for RWList<'_> {
    fn drop(&mut self) {
        self.commit_rw();
    }
}
