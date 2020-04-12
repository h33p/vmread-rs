

pub struct Tlb {
    tlbp: *mut sys::tlb_t
}

impl Tlb {
    /// Create the TLB of current thread
    ///
    /// # Remarks
    ///
    /// In VMRead, each thread has its own Translation Lookaside Buffer so as to not spend
    /// resources on locking mechanisms.
    pub fn from_current_thread() -> Tlb {
        Tlb {
            tlbp: unsafe { sys::GetTlb() }
        }
    }

    /// Flush TLB
    pub fn flush_tlb(&self) -> &Self {
        unsafe {
            sys::FlushTlb(self.tlbp);
        }
        self
    }

    /// Verify TLB
    pub fn verify_tlb(&self, c_ctx: &sys::WinCtx) -> &Self {
        unsafe {
            sys::VerifyTlb(&c_ctx.process, self.tlbp, 1, 0);
        }
        self
    }

    /// Set global TLB validity time in milliseconds
    ///
    /// Defines for how long translation caches (TLB and page buffer) should be valid. Higher
    /// values lead to higher performance, but could potentially lead to incorrect translation if
    /// the page tables update in that period. Especially dangerous if write operations are to be
    /// performed.
    ///
    /// # Arguments
    ///
    /// * `new_time` - new validity time
    pub fn set_mem_cache_time(new_time: usize) {
        unsafe {
            sys::SetMemCacheTime(new_time as u64);
        }
    }

    /// Get the default TLB validity in milliseconds
    pub fn get_default_mem_cache_time() -> usize {
        unsafe {
            sys::GetDefaultMemCacheTime() as usize
        }
    }
}
