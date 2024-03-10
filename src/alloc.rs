extern crate alloc;

use crate::{serial_debug, ALLOC_START, HEAP_SIZE, HEAP_START};

use allocator::buddy::BuddyAllocator;
use core::alloc::{GlobalAlloc, Layout};
use once_cell::unsync::OnceCell;

const MIN_BLOCK_SIZE: usize = 64;

pub struct Locked<A> {
    inner: spin::Mutex<A>,
}

impl<A> Locked<A> {
    pub const fn new(inner: A) -> Self {
        Locked {
            inner: spin::Mutex::new(inner),
        }
    }

    pub fn lock(&self) -> spin::MutexGuard<A> {
        self.inner.lock()
    }
}

unsafe impl GlobalAlloc for Locked<OnceCell<BuddyAllocator>> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut cell = self.lock();
        let allocator = cell.get_mut().expect("Allocator not initialized");

        // Align size to next power of 2 and respect layout alignment. To
        // respect layout alignment in the context of buddy allocation, we need
        // to find the smallest power of 2 that is greater than or equal to the alignment.
        // This assumes that the heap is aligned to the largest alignment
        // required by any type. In PathOS, we assume that it is at least 4KiB aligned.

        // We still need to align the size to the next power of two,
        // because default alignment is 1 while the size can be not a power of two.
        let size = layout.pad_to_align().size().next_power_of_two();

        // Align size to at least MIN_BLOCK_SIZE.
        let size = size.max(allocator.min_block_size);
        // serial_debug!("Allocating {} bytes", size);

        let idx = allocator.find_block(size).unwrap();
        // serial_debug!("Found index: {}", idx);

        let order_start_idx = allocator.order_start_index(size);

        // The start address of the block is as far from the start of the heap
        // as the index is from the first index of the order multiplied by the
        // size of the block. This assumes that the allocator always returns the
        // next free block in the order.
        let block_start_addr = ALLOC_START + (idx - order_start_idx) * size;

        serial_debug!("Found block at adress: {:?}", block_start_addr as *mut u8);
        block_start_addr as *mut u8
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let mut cell = self.lock();
        let allocator = cell.get_mut().expect("Allocator not initialized");

        let size = layout.pad_to_align().size().next_power_of_two();
        // Align size to at least MIN_BLOCK_SIZE. This is required because e.g.
        // a request to free 1 byte will have layout size 1 (with default
        // alignment = 1).
        let size = size.max(allocator.min_block_size);

        // serial_debug!("Deallocating {} bytes at {:?}", size, ptr);

        let order_start_idx = allocator.order_start_index(size);

        // To find the index of the block, we reverse the formula used in the
        // alloc function.
        // index - order_start_idx = (block_start - heap_start) / size
        // index = order_start_idx + (block_start - heap_start) / size
        let idx = order_start_idx + (ptr as usize - ALLOC_START) / size;
        // serial_debug!("Calculated index: {}", idx);

        allocator.free_block(idx);
        // serial_debug!("Deallocated block at index: {}", idx);
    }
}

#[global_allocator]
static ALLOCATOR: Locked<OnceCell<BuddyAllocator>> = Locked::new(OnceCell::new());

pub fn init_allocator() {
    let allocator = ALLOCATOR.lock();
    unsafe {
        allocator
            .set(BuddyAllocator::new(HEAP_START, HEAP_SIZE, MIN_BLOCK_SIZE))
            .expect("Allocator already initialized");
    }
}
