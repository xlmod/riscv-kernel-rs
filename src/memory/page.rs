use core::{fmt, ops};

use super::physical::PhysAddr;

// CONST
pub const PAGE_SIZE: usize = 4096;

pub const PAGE_OFFSET_4K: u64 = 0xfff;
pub const PAGE_OFFSET_2M: u64 = 0x1f_ffff;
pub const PAGE_OFFSET_1G: u64 = 0x3fff_ffff;

pub const PAGE_TABLE_NUMBER_ENTRIES: u64 = 512;
pub const PAGE_TABLE_ENTRY_PHYSADDR_MASK: u64 = 0x3f_ffff_ffff_fc00;

// ///////////////////////////////////
// Page Table
// ///////////////////////////////////

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageType {
    Page,
    MegaPage,
    GigaPage,
}

impl PageType {
    /// Return the number of 4K pages in the page type
    pub fn get_nb_pages(&self) -> usize {
        match self {
            PageType::Page => 1,
            PageType::MegaPage => 512,
            PageType::GigaPage => 512 * 512,
        }
    }
}

impl fmt::Display for PageType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PageType::Page => write!(f, "Page (4K)"),
            PageType::MegaPage => write!(f, "MegaPage (2M)"),
            PageType::GigaPage => write!(f, "GigaPage (1G)"),
        }
    }
}

// ///////////////////////////////////
// Page Table
// ///////////////////////////////////

#[derive(Debug, PartialEq, Eq)]
pub struct PageTable<'a> {
    entries: &'a mut [PageTableEntry],
}

impl<'a> PageTable<'a> {
    /// Return a PageTable from a pointer that point on the page table
    /// # Safety
    /// `ptr` must point on a valid page table and be align to 4K
    pub unsafe fn from_ptr(ptr: *const u8) -> Self {
        let ptr_entries = &mut [ptr as usize, 512] as *mut _ as *mut &mut [PageTableEntry];
        Self {
            entries: *ptr_entries,
        }
    }
}

impl<'a> ops::Index<usize> for PageTable<'a> {
    type Output = PageTableEntry;
    fn index(&self, index: usize) -> &Self::Output {
        &self.entries[index % PAGE_TABLE_NUMBER_ENTRIES as usize]
    }
}

impl<'a> ops::IndexMut<usize> for PageTable<'a> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.entries[index % PAGE_TABLE_NUMBER_ENTRIES as usize]
    }
}

// ///////////////////////////////////
// Page Table Entry
// ///////////////////////////////////

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub enum PageTableEntryFlags {
    Valid = 1 << 0,
    Readable = 1 << 1,
    Writable = 1 << 2,
    Executable = 1 << 3,
    User = 1 << 4,
    Global = 1 << 5,
    Accessed = 1 << 6,
    Dirty = 1 << 7,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct PageTableEntry(u64);

impl PageTableEntry {
    /// Return a new zeroed page table entry
    fn new(entry: u64) -> Self {
        Self(entry)
    }

    /// Status of a Flag
    fn is_set(&self, bit: PageTableEntryFlags) -> bool {
        self.0 & bit as u64 != 0
    }
    /// Set a Flag
    fn set(&mut self, bit: PageTableEntryFlags) {
        self.0 |= bit as u64;
    }
    /// Unset a Flag
    fn unset(&mut self, bit: PageTableEntryFlags) {
        self.0 &= !(bit as u64);
    }

    /// Return true if the Valid flag is set
    /// Same as self.is_set(PageTableEntryFlags::Valid)
    fn is_valid(&self) -> bool {
        self.is_set(PageTableEntryFlags::Valid)
    }

    /// Return true if one of Readable, Writable or Executable flag are set
    fn is_leaf(&self) -> bool {
        self.is_set(PageTableEntryFlags::Readable)
            || self.is_set(PageTableEntryFlags::Writable)
            || self.is_set(PageTableEntryFlags::Executable)
    }

    /// Return the Physical Address stored in the entry
    fn get_physaddr(&self) -> PhysAddr {
        PhysAddr::new(self.0 << 2).clear_nth_last_bit(12)
    }

    /// Set the physical address in the entry
    fn set_physaddr(&mut self, phys: PhysAddr) {
        self.0 |= PAGE_TABLE_ENTRY_PHYSADDR_MASK;
        self.0 &= *phys.clear_nth_last_bit(12) >> 2;
    }

    /// Return an Option to a PageTable pointed by the physical address of the entry.
    unsafe fn get_next_level_table(&self) -> Option<PageTable> {
        (self.is_valid() && !self.is_leaf())
            .then_some(PageTable::from_ptr(self.get_physaddr().get_ptr()))
    }
}
