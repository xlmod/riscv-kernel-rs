use core::{fmt, ops};
use lazy_static::lazy_static;
use spin::Mutex;

use super::page;

const PHYSADDR_MAX: u64 = 0xff_ffff_ffff_ffff;

lazy_static! {
 pub static ref PHYSFRAMEALLOCATOR: Mutex<PhysFrameAllocator> = Mutex::new(unsafe {
    PhysFrameAllocator::new(
        crate::TEXT_START,
        crate::HEAP_START.get_ptr(),
        (crate::MEMORY_END - crate::MEMORY_START).get_u64() as usize)
    });
}

// ///////////////////////////////////
// Physical Address
// ///////////////////////////////////

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct PhysAddr(u64);

impl PhysAddr {
    pub fn new(ptr: u64) -> Self {
        Self(ptr & PHYSADDR_MAX)
    }

    /// Clear the last nth bit of the address
    /// `0xdead_beef -> clear_nth_last_bit(12) -> 0xdead_b000`
    pub fn clear_nth_last_bit(&self, n: usize) -> Self {
        Self(self.0 & (PHYSADDR_MAX << n) & PHYSADDR_MAX)
    }

    /// Get page offset
    pub fn get_page_offset(&self) -> u64 {
        self.0 & page::PAGE_OFFSET_4K
    }

    /// Get the PPN (Physical Page Number)
    pub fn get_ppn(&self) -> u64 {
        self.0 >> 12
    }

    pub fn get_ptr(&self) -> *const u8 {
        self.0 as *const u8
    }

    pub fn get_u64(&self) -> u64 {
        self.0
    }
}

impl fmt::Display for PhysAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PhysAddr(0x{:x})", self.0)
    }
}

impl ops::Add for PhysAddr {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.0 + rhs.0)
    }
}

impl ops::Add<u64> for PhysAddr {
    type Output = Self;
    fn add(self, rhs: u64) -> Self::Output {
        Self::new(self.0 + rhs)
    }
}

impl ops::AddAssign for PhysAddr {
    fn add_assign(&mut self, rhs: Self) {
        *self = Self::new(self.0 + rhs.0);
    }
}

impl ops::AddAssign<u64> for PhysAddr {
    fn add_assign(&mut self, rhs: u64) {
        *self = Self::new(self.0 + rhs);
    }
}

impl ops::BitAnd for PhysAddr {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self::new(self.0 & rhs.0)
    }
}

impl ops::BitAndAssign for PhysAddr {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = Self::new(self.0 & rhs.0);
    }
}

impl ops::BitOr for PhysAddr {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self::new(self.0 | rhs.0)
    }
}

impl ops::BitOrAssign for PhysAddr {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = Self::new(self.0 | rhs.0);
    }
}

impl ops::Deref for PhysAddr {
    type Target = u64;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ops::DerefMut for PhysAddr {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl ops::Shl<u64> for PhysAddr {
    type Output = Self;
    fn shl(self, rhs: u64) -> Self::Output {
        Self::new(self.0 << rhs)
    }
}

impl ops::ShlAssign<u64> for PhysAddr {
    fn shl_assign(&mut self, rhs: u64) {
        *self = Self::new(self.0 << rhs);
    }
}

impl ops::Shr<u64> for PhysAddr {
    type Output = Self;
    fn shr(self, rhs: u64) -> Self::Output {
        Self::new(self.0 >> rhs)
    }
}

impl ops::ShrAssign<u64> for PhysAddr {
    fn shr_assign(&mut self, rhs: u64) {
        *self = Self::new(self.0 >> rhs);
    }
}

impl ops::Sub for PhysAddr {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.0 - rhs.0)
    }
}

impl ops::Sub<u64> for PhysAddr {
    type Output = Self;
    fn sub(self, rhs: u64) -> Self::Output {
        Self::new(self.0 - rhs)
    }
}

impl ops::SubAssign for PhysAddr {
    fn sub_assign(&mut self, rhs: Self) {
        *self = Self::new(self.0 - rhs.0);
    }
}

impl ops::SubAssign<u64> for PhysAddr {
    fn sub_assign(&mut self, rhs: u64) {
        *self = Self::new(self.0 - rhs);
    }
}

// ///////////////////////////////////
// Physical Frame
// ///////////////////////////////////

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysFrame {
    addr: PhysAddr,
    page_type: page::PageType,
    size: usize,
}

impl PhysFrame {
    pub fn new(addr: PhysAddr, page_type: page::PageType, size: usize) -> Self {
        PhysFrame {
            addr,
            page_type,
            size,
        }
    }
}

impl fmt::Display for PhysFrame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "PhysFrame{{ {}, {}K }}",
            self.addr,
            self.page_type.get_nb_pages() * self.size * page::PAGE_SIZE
        )
    }
}

// ///////////////////////////////////
// Physical Frame Allocator
// ///////////////////////////////////

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysFrameAllocError(&'static str);

impl fmt::Display for PhysFrameAllocError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PhysAllocError{{ {} }}", self.0)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct PhysFrameAllocator {
    start_addr: PhysAddr,
    bitmap: &'static mut [u8],
    nb_pages: usize,
    free_pages: usize,
}

impl PhysFrameAllocator {
    /// # Safety
    /// `ptr` have to point to a valid and accesible zone of size `memsize`
    pub unsafe fn new(start_addr: PhysAddr, ptr: *const u8, memsize: usize) -> Self {
        let nb_pages = memsize / page::PAGE_SIZE;
        let ptr_entries = &mut [ptr as usize, nb_pages / 8] as *mut _ as *mut &mut [u8];
        Self {
            start_addr,
            bitmap: *ptr_entries,
            nb_pages,
            free_pages: nb_pages,
        }
    }

    /// Allocate `nb` pages of type `page_type` and return a `PhysFrame` or `PhysAllocError`
    /// # Safety
    pub unsafe fn alloc(
        &mut self,
        page_type: page::PageType,
        nb: usize,
    ) -> Result<PhysFrame, PhysFrameAllocError> {
        let nb_pages: usize = page_type.get_nb_pages() * nb;
        if nb_pages > self.nb_pages {
            return Err(PhysFrameAllocError("Not enought memory!"));
        }
        let mut index: usize = 0;
        while index < self.nb_pages {
            let mut n = 0;
            let mut i;
            while n < nb_pages {
                i = index + n;
                if (self.bitmap[i / 8] >> (i % 8)) & 1 != 0 {
                    break;
                }
                n += 1;
            }
            if n == nb_pages {
                for j in 0..nb_pages {
                    self.bitmap[(index + j) / 8] |= 1 << ((index + j) % 8);
                }
                self.free_pages -= nb_pages;
                return Ok(PhysFrame::new(
                    self.start_addr + (page::PAGE_SIZE * index) as u64,
                    page_type,
                    nb,
                ));
            }
            index += nb_pages;
        }
        Err(PhysFrameAllocError("Not enought contiguous memory!"))
    }
}
