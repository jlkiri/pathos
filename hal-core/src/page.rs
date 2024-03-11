/// Sv39 page table
#[repr(align(4096))]
pub struct PageTable {
    entries: [PageTableEntry; 512],
}

/// Sv39 page table entry
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct PageTableEntry(u64);

#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct Vaddr(u64);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct Paddr(u64);

/// 4KiB aligned physical frame
#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct Frame(Paddr);

/// 4KiB aligned page
#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct Page(Vaddr);

/// Sv39 page table entry flags
#[repr(u64)]
#[derive(Debug, Clone)]
pub enum EntryFlags {
    Valid = 1 << 0,
    Read = 1 << 1,
    Write = 1 << 2,
    Execute = 1 << 3,
    User = 1 << 4,
    Global = 1 << 5,
    Accessed = 1 << 6,
    Dirty = 1 << 7,

    // Convenience combinations
    RW = 1 << 1 | 1 << 2,
    RX = 1 << 1 | 1 << 3,
    RWX = 1 << 1 | 1 << 2 | 1 << 3,
}

impl EntryFlags {
    pub fn as_u64(self) -> u64 {
        self as u64
    }
}

impl PageTableEntry {
    pub fn new(flags: u64) -> Self {
        Self(flags)
    }

    pub fn is_valid(&self) -> bool {
        self.0 & EntryFlags::Valid.as_u64() != 0
    }

    pub fn is_leaf(&self) -> bool {
        self.0 & EntryFlags::RWX.as_u64() != 0
    }

    pub fn flags(&self) -> u64 {
        // unsafe { core::mem::transmute(self.0 & 0xff) }
        self.0
    }

    pub fn set_paddr(&mut self, paddr: Paddr) {
        let ppn = paddr.ppn();
        self.0 |= ppn << 10;
    }

    pub fn paddr(&self) -> Paddr {
        let ppn = self.0 >> 10;
        Paddr(ppn << 12)
    }
}

impl PageTable {
    pub fn new() -> Self {
        Self {
            entries: [PageTableEntry::new(0); 512],
        }
    }

    pub fn entry_mut(&mut self, index: usize) -> &mut PageTableEntry {
        &mut self.entries[index]
    }
}

impl Vaddr {
    pub fn new(addr: u64) -> Self {
        // Make bits 39-63 copy of bit 38 to form a canonical address
        Self(((addr << 25) as i64 >> 25) as u64)
    }

    pub fn indexed_vpn(self) -> [usize; 3] {
        let vpns = self.0 >> 12;
        [
            (vpns & 0x1ff) as usize,
            ((vpns >> 9) & 0x1ff) as usize,
            ((vpns >> 18) & 0x1ff) as usize,
        ]
    }

    pub fn inner(&self) -> u64 {
        self.0
    }
}

impl Paddr {
    pub fn new(addr: u64) -> Self {
        Self(addr)
    }

    pub fn as_mut_ptr<T>(self) -> *mut T {
        self.0 as *mut T
    }

    pub fn ppn(&self) -> u64 {
        self.0 >> 12
    }

    pub fn inner(&self) -> u64 {
        self.0
    }
}

impl Frame {
    pub fn containing_address(addr: u64) -> Self {
        // Align physical address down to the nearest 4KiB
        Frame(Paddr(addr & !0xfff))
    }

    pub fn addr(&self) -> Paddr {
        self.0
    }
}

impl Page {
    pub fn containing_address(addr: u64) -> Self {
        Page(Vaddr(addr & !0xfff))
    }

    pub fn addr(&self) -> Vaddr {
        self.0
    }
}

pub struct PageRange {
    start: Vaddr,
    end: Vaddr,
}

impl PageRange {
    pub fn new(start: Vaddr, end: Vaddr) -> Self {
        Self { start, end }
    }
}

impl Iterator for PageRange {
    type Item = Page;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start.0 <= self.end.0 {
            let page = Page::containing_address(self.start.0);
            self.start.0 += 0x1000;
            Some(page)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_table_entry() {
        let entry = PageTableEntry::new(EntryFlags::Valid as u64);
        assert!(entry.is_valid());
    }
}
