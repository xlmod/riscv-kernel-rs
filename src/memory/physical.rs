use core::{ops, fmt};

use super::page;

const PHYSADDR_MAX: u64 = 0xff_ffff_ffff_ffff;

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
}

impl PhysFrame {
    pub fn new(addr: PhysAddr, page_type: page::PageType) -> Self {
        PhysFrame { addr, page_type }
    }
}

impl fmt::Display for PhysFrame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PhysFrame:\r\n\t{}\r\n\t{}", self.addr, self.page_type)
    }
}

// ///////////////////////////////////
// Physical Frame Allocator
// ///////////////////////////////////

#[derive(Debug, PartialEq, Eq)]
pub struct PhysFrameAllocator {
    start_addr: PhysAddr,
    bitmap: &'static mut [u8],
    nb_pages: usize,
    free_pages: usize,
}

impl PhysFrameAllocator {
    pub unsafe fn new(start_addr: PhysAddr, ptr: *const u8, memsize: usize) -> Self {
        let nb_pages = memsize / page::PAGE_SIZE;
        let ptr_entries = &mut [ptr as usize, nb_pages / 8] as *mut _ as *mut &mut [u8];
        Self { start_addr, bitmap: *ptr_entries, nb_pages, free_pages: nb_pages}
    }

    /// Return the first PhysFrame free and alloc
    pub unsafe fn alloc(&mut self, page_type: page::PageType) -> Option<PhysFrame> {
        let nb_pages: usize = match page_type {
            page::PageType::Page => 1,
            page::PageType::MegaPage => 512,
            page::PageType::GigaPage => 512 * 512,
        };
        if nb_pages > self.nb_pages {
            return None;
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
                    self.bitmap[(index + j) / 8] |= 1 << ((index + j)  % 8);
                }
                self.free_pages -= nb_pages;
                return Some(PhysFrame::new(self.start_addr + (page::PAGE_SIZE * index) as u64, page_type));
            }
            index += nb_pages;
        }
        None
    } 
}
