extern crate alloc;

use alloc::boxed::Box;
use hal_core::page::{EntryFlags, Frame, Paddr, Page, PageRange, PageTable, PageTableEntry, Vaddr};

use crate::serial_debug;

fn map(root: &mut PageTable, page: Page, frame: Frame, flags: EntryFlags) {
    let vpn = page.addr().indexed_vpn();
    let mut table = root;

    for lv in (0..=2).rev() {
        let index = vpn[lv];
        let entry = table.entry_mut(index);

        if entry.is_valid() {
            if entry.is_leaf() {
                // This address is already mapped, nothing to do
                // serial_debug!(
                //     "Address 0x{:x?} is already mapped to 0x{:x?}",
                //     page.addr(),
                //     entry.paddr()
                // );
                return;
            }

            let next_page_table_paddr = entry.paddr();
            table = unsafe { &mut *next_page_table_paddr.as_mut_ptr::<PageTable>() };
        } else {
            if lv == 0 {
                // Create a leaf entry and return
                *entry = PageTableEntry::new(EntryFlags::Valid.as_u64() | flags.as_u64());
                entry.set_paddr(frame.addr());
                return;
            }

            let next_page_table = PageTable::new();
            let ptr = Box::into_raw(Box::new(next_page_table));
            let next_page_table_paddr = Paddr::new(ptr as u64);

            *entry = PageTableEntry::new(EntryFlags::Valid.as_u64());
            entry.set_paddr(next_page_table_paddr);
            table = unsafe { &mut *ptr };
        }
    }
}

pub fn allocate_root() -> &'static mut PageTable {
    let root = PageTable::new();
    let ptr = Box::into_raw(Box::new(root));

    serial_debug!("Allocated L2 (root) page table at 0x{:x}", ptr as usize);

    unsafe { &mut *ptr }
}

fn map_range(
    root: &mut PageTable,
    start: usize,
    end: usize,
    alloc_start: usize,
    alloc_size: usize,
    flags: EntryFlags,
) {
    let start = Vaddr::new(start as u64);
    let end = Vaddr::new(end as u64);
    let range = PageRange::new(start, end);
    let mut phys_start = alloc_start;
    for page in range {
        if phys_start >= alloc_start + alloc_size {
            panic!("Out of physical memory for page table");
        }

        let frame = Frame::containing_address(phys_start as u64);
        map(root, page, frame, flags.clone());
        phys_start += 0x1000;
    }
}

pub fn id_map(root: &mut PageTable, page: Page, flags: EntryFlags) {
    let frame = Frame::containing_address(page.addr().inner());
    map(root, page, frame, flags);
}

pub fn id_map_range(root: &mut PageTable, start: usize, end: usize, flags: EntryFlags) {
    let start = Vaddr::new(start as u64);
    let end = Vaddr::new(end as u64);

    let range = PageRange::new(start, end);
    for page in range {
        id_map(root, page, flags.clone());
    }
}

pub fn translate_vaddr(root: &mut PageTable, vaddr: Vaddr) -> Option<Paddr> {
    let vpn = vaddr.indexed_vpn();
    let mut table = root;

    for lv in (0..=2).rev() {
        let index = vpn[lv];
        let entry = table.entry_mut(index);
        if !entry.is_valid() {
            serial_debug!("Entry is not valid: LV: {}, INDEX: {}", lv, index);
            return None;
        }

        if entry.is_leaf() {
            return Some(entry.paddr());
        }

        let next_page_table_paddr = entry.paddr();
        table = unsafe { &mut *next_page_table_paddr.as_mut_ptr::<PageTable>() };
    }

    serial_debug!("This code must not be reached");
    None
}
