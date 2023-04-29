use core::{fmt, ops};

// ///////////////////////////////////
// Virtual Address
// ///////////////////////////////////
const PHYSADDR_MAX: u64 = 0xff_ffff_ffff_ffff;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct VirtAddr(u64);

impl VirtAddr {
    pub fn new(ptr: u64) -> Self {
        Self(ptr & PHYSADDR_MAX)
    }

    /// Clear the last nth bit of the address
    /// `0xdead_beef -> clear_nth_last_bit(12) -> 0xdead_b000`
    pub fn clear_nth_last_bit(&self, n: usize) -> Self {
        Self(self.0 & (PHYSADDR_MAX << n) & PHYSADDR_MAX)
    }

    pub fn get_ptr(&self) -> *const u8 {
        self.0 as *const u8
    }

    pub fn get_u64(&self) -> u64 {
        self.0
    }
}

impl fmt::Display for VirtAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "VirtAddr(0x{:x})", self.0)
    }
}

