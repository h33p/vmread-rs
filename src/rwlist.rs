
#[derive(Debug)]
pub struct RWList {
    process: *const sys::ProcessData,
    dir_base: u64,
    read_list: Vec<sys::RWInfo>,
    write_list: Vec<sys::RWInfo>,
}

pub struct RWListBuilder<'a> {
    list: &'a mut RWList,
    read_start: usize,
    write_start: usize,
}

impl<'a> RWListBuilder<'a> {
    pub fn new(list: &mut RWList) -> RWListBuilder {
        RWListBuilder {
            read_start: list.read_list.len(),
            write_start: list.write_list.len(),
            list: list,
        }
    }

    pub fn write<T>(self, address: u64, val: &'a T) -> RWListBuilder<'a> {
        self.list.write_list.push(sys::RWInfo {
            local: val as *const T as u64,
            remote: address,
            size: std::mem::size_of::<T>() as u64
        });
        self
    }

    pub fn write_arr<T>(self, address: u64, val: &'a [T]) -> RWListBuilder<'a> {
        self.list.write_list.push(sys::RWInfo {
            local: val.as_ptr() as u64,
            remote: address,
            size: (std::mem::size_of::<T>() * val.len()) as u64
        });
        self
    }

    pub fn read<T>(self, address: u64, val: &'a mut T) -> RWListBuilder<'a> {
        self.list.read_list.push(sys::RWInfo {
            local: val as *mut T as u64,
            remote: address,
            size: std::mem::size_of::<T>() as u64
        });
        self
    }

    pub fn read_arr<T>(self, address: u64, val: &'a mut [T]) -> RWListBuilder<'a> {
        self.list.read_list.push(sys::RWInfo {
            local: val.as_mut_ptr() as u64,
            remote: address,
            size: (std::mem::size_of::<T>() * val.len()) as u64
        });
        self
    }
}

impl RWList {
    pub fn new(ctx: &sys::WinCtx, dir_base: u64) -> RWList {
        RWList {
            process: &ctx.process,
            dir_base: dir_base,
            read_list: vec![],
            write_list: vec![],
        }
    }

    pub fn start<'a>(&'a mut self) -> RWListBuilder<'a> {
        RWListBuilder::<'a>::new(self)
    }

    pub fn commit(&mut self, read_start: usize, write_start: usize) -> (&mut RWList, usize, usize) {

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

    pub fn commit_rw(&mut self) -> (&mut RWList, usize, usize) {
        self.commit(self.read_list.len(), self.write_list.len())
    }

    pub fn commit_read(&mut self) -> (&mut RWList, usize, usize) {
        self.commit(0, self.write_list.len())
    }

    pub fn commit_write(&mut self) -> (&mut RWList, usize, usize) {
        self.commit(self.read_list.len(), 0)
    }

}

impl<'a> Drop for RWListBuilder<'a> {
    fn drop(&mut self) {
        self.list.commit(self.read_start, self.write_start);
    }
}

impl Drop for RWList {
    fn drop(&mut self) {
        self.commit_rw();
    }
}
