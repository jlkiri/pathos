extern crate alloc;

use core::alloc::Layout;

use alloc::{alloc::alloc_zeroed, boxed::Box};
use hal_core::page::{EntryFlags, Frame, Paddr, Page, PageRange, PageTable, PageTableEntry, Vaddr};

use crate::serial_debug;

fn map_to_frame(root: &mut PageTable, page: Page, frame: Frame, flags: EntryFlags) {
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
                *entry = PageTableEntry::new(
                    EntryFlags::Valid.as_u64()
                        | EntryFlags::Accessed.as_u64()
                        | EntryFlags::Dirty.as_u64()
                        | flags.as_u64(),
                );
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

pub fn id_map(root: &mut PageTable, page: Page, flags: EntryFlags) {
    let frame = Frame::containing_address(page.addr().inner());
    map_to_frame(root, page, frame, flags);
}

pub fn map_alloc(root: &mut PageTable, page: Page, flags: EntryFlags) {
    let layout = Layout::from_size_align(4096, 4096).expect("Invalid layout");
    let frame = unsafe { alloc_zeroed(layout) };
    map_to_frame(root, page, Frame::containing_address(frame as u64), flags);
}

pub fn map_alloc_range(root: &mut PageTable, start: usize, end: usize, flags: EntryFlags) {
    let start = Vaddr::new(start as u64);
    let end = Vaddr::new(end as u64);

    let range = PageRange::new(start, end);
    for page in range {
        map_alloc(root, page, flags.clone());
    }
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
            serial_debug!("LV: {}, {:x?}, {:b}", lv, entry.paddr(), entry.flags());
            return Some(entry.paddr());
        }

        let next_page_table_paddr = entry.paddr();
        table = unsafe { &mut *next_page_table_paddr.as_mut_ptr::<PageTable>() };
    }

    serial_debug!("This code must not be reached");
    None
}
