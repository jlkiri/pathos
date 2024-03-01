use core::ptr;

#[cfg(test)]
use std::println;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Coalesced = 0,
    Allocated = 1,
    Free = 2,
}

#[derive(Debug)]
pub struct BuddyAllocator {
    nodes: &'static mut [State],
    heap_size: usize,
    pub min_block_size: usize,
}

impl BuddyAllocator {
    pub unsafe fn new(ptr: usize, heap_size: usize, min_block_size: usize) -> Self {
        let max_order: usize = (heap_size / min_block_size).ilog2() as usize;
        let num_nodes: usize = 2usize.pow(max_order as u32 + 1) - 1;

        // This assumes that the memory is zeroed out
        let nodes = &mut *ptr::slice_from_raw_parts_mut(ptr as *mut State, num_nodes);

        nodes[0] = State::Free;

        Self {
            nodes,
            heap_size,
            min_block_size,
        }
    }

    fn split_ancestors(&mut self, idx: usize) {
        match Self::parent(idx) {
            None => return, // Root node
            Some(next) => {
                // Avoid messing up already split ancestors
                if self.nodes[next] == State::Allocated {
                    return;
                }

                self.split(next);
                self.split_ancestors(next);
            }
        }
    }

    fn split(&mut self, idx: usize) {
        if let Some(left) = self.left(idx) {
            self.nodes[left] = State::Free;
        }
        if let Some(right) = self.right(idx) {
            self.nodes[right] = State::Free;
        }
    }

    fn mark_ancestors_allocated(&mut self, idx: usize) {
        if idx == 0 {
            return;
        }

        let parent = Self::parent(idx).unwrap();
        self.nodes[parent] = State::Allocated;
        self.mark_ancestors_allocated(parent);
    }

    fn mark_descendants_allocated(&mut self, idx: usize) {
        if let Some(left) = self.left(idx) {
            self.nodes[left] = State::Allocated;
            self.mark_descendants_allocated(left);
        }
        if let Some(right) = self.right(idx) {
            self.nodes[right] = State::Allocated;
            self.mark_descendants_allocated(right);
        }
    }

    fn mark_descendants_unavailable(&mut self, idx: usize) {
        if let Some(left) = self.left(idx) {
            self.nodes[left] = State::Coalesced;
            self.mark_descendants_unavailable(left);
        }
        if let Some(right) = self.right(idx) {
            self.nodes[right] = State::Coalesced;
            self.mark_descendants_unavailable(right);
        }
    }

    pub fn free_block(&mut self, idx: usize) {
        assert!(idx < self.nodes.len(), "Index {} out of bounds", idx);
        assert_eq!(
            self.nodes[idx],
            State::Allocated,
            "Block not allocated, double free"
        );

        self.nodes[idx] = State::Free;

        if let Some(buddy) = self.buddy(idx) {
            if self.nodes[buddy] == State::Free {
                return self.coalesce(idx);
            }
        }

        // When freeing blocks > min_block_size, we need to mark descendants as
        // unavailable, because they are marked as allocated. However, here we
        // delay coalescing previously possibly allocated blocks, until we cannot
        // coalesce parents anymore.
        self.mark_descendants_unavailable(idx);
    }

    fn coalesce(&mut self, idx: usize) {
        let parent = Self::parent(idx).unwrap();

        if let Some(left) = self.left(parent) {
            self.nodes[left] = State::Coalesced;
        }
        if let Some(right) = self.right(parent) {
            self.nodes[right] = State::Coalesced;
        }

        self.free_block(parent); // Recursively coalesce ancestors if possible
    }

    pub fn order_start_index(&self, block_size: usize) -> usize {
        assert!(block_size >= self.min_block_size, "Block size too small");

        let max_order = (self.heap_size / self.min_block_size).ilog2();
        let order = (block_size / self.min_block_size).ilog2();
        2usize.pow(max_order as u32 - order) - 1
    }

    pub fn find_block(&mut self, size: usize) -> Result<usize, &str> {
        assert!(size.is_power_of_two(), "Size is not a power of 2");
        assert!(
            size <= self.heap_size,
            "Requested size is greater than memory size"
        );

        let search_start_idx = self.order_start_index(size);
        let end = 2 * search_start_idx + 1;
        assert!(
            end <= self.nodes.len(),
            "End index out of bounds, not enough nodes for heap size"
        );

        let blocks = &self.nodes[search_start_idx..end];

        if let Some(first_free) = blocks.iter().position(|b| *b == State::Free) {
            let idx = search_start_idx + first_free;

            self.mark_ancestors_allocated(idx);
            self.mark_descendants_allocated(idx);
            self.nodes[idx] = State::Allocated;

            return Ok(idx);
        }

        if let Some(first_to_split) = blocks.iter().position(|b| *b == State::Coalesced) {
            let idx = search_start_idx + first_to_split;

            assert_ne!(idx, 0, "Highest order block cannot be unavailable");

            self.split_ancestors(idx);
            self.mark_ancestors_allocated(idx);
            self.mark_descendants_allocated(idx);
            self.nodes[idx] = State::Allocated;

            return Ok(idx);
        }

        Err("No block found for allocation")
    }

    fn left(&self, idx: usize) -> Option<usize> {
        let i = 2 * idx + 1;
        if i < self.nodes.len() {
            return Some(i);
        }
        None
    }

    fn right(&self, idx: usize) -> Option<usize> {
        let i = 2 * idx + 2;
        if i < self.nodes.len() {
            return Some(i);
        }
        None
    }

    fn parent(idx: usize) -> Option<usize> {
        if idx == 0 {
            return None;
        }
        Some((idx - 1) / 2)
    }

    fn buddy(&self, idx: usize) -> Option<usize> {
        match idx {
            0 => None,
            idx if idx % 2 == 0 => Some(idx - 1),
            _ => {
                assert!(idx < self.nodes.len(), "Index out of bounds");
                Some(idx + 1)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::{assert_matches::assert_matches, println};

    const HEAP_SIZE: usize = 64;
    const MIN_BLOCK_SIZE: usize = 8;
    const MAX_ORDER: usize = (HEAP_SIZE / MIN_BLOCK_SIZE).ilog2() as usize;
    const NUM_NODES: usize = 2usize.pow(MAX_ORDER as u32 + 1) - 1;

    fn assert_descendants_allocated(idx: usize, allocator: &BuddyAllocator, msg: &str) {
        assert_eq!(
            allocator.nodes[idx],
            State::Allocated,
            "Node (idx: {}) not marked as allocated: {}",
            idx,
            msg
        );

        if let Some(left) = allocator.left(idx) {
            assert_descendants_allocated(left, allocator, msg);
        }

        if let Some(right) = allocator.right(idx) {
            assert_descendants_allocated(right, allocator, msg);
        }
    }

    fn assert_descendants_unallocated(idx: usize, allocator: &BuddyAllocator) {
        assert_ne!(
            allocator.nodes[idx],
            State::Allocated,
            "Left child not marked as allocated"
        );

        if let Some(left) = allocator.left(idx) {
            assert_descendants_unallocated(left, allocator);
        }
        if let Some(right) = allocator.right(idx) {
            assert_descendants_unallocated(right, allocator);
        }
    }

    #[test]
    fn test_split_ancestors() {
        static mut HEAP: [u8; 64] = [0; 64];

        let mut allocator =
            unsafe { BuddyAllocator::new(HEAP.as_ptr() as usize, HEAP_SIZE, MIN_BLOCK_SIZE) };

        assert_eq!(
            allocator.nodes[1],
            State::Coalesced,
            "32 byte block not marked as unavailable"
        );

        allocator.split_ancestors(7); // First 8 byte block

        for i in &[0, 1, 3, 4, 7, 8] {
            assert_eq!(
                allocator.nodes[*i],
                State::Free,
                "Ancestor not marked as free"
            );
        }

        // Assert the rest of the nodes are still unavailable
        assert_eq!(allocator.nodes[5], State::Coalesced);
        assert_eq!(allocator.nodes[6], State::Coalesced);

        for i in 9..NUM_NODES {
            assert_eq!(
                allocator.nodes[i],
                State::Coalesced,
                "Node not marked as unavailable"
            );
        }
    }

    #[test]
    fn test_find_smallest_free_block() {
        static mut HEAP: [u8; 64] = [0; 64];

        let mut allocator =
            unsafe { BuddyAllocator::new(HEAP.as_ptr() as usize, HEAP_SIZE, MIN_BLOCK_SIZE) };

        println!("{:#?}", allocator.nodes);

        let idx = allocator.find_block(8).unwrap();

        assert_eq!(idx, 7, "Block not found at expected index");
        assert_eq!(
            allocator.nodes[idx],
            State::Allocated,
            "Block not marked as allocated"
        );

        // Check if ancestors are also marked as allocated
        assert_eq!(
            allocator.nodes[3],
            State::Allocated,
            "16 byte block not marked as allocated"
        ); // 16 byte block
        assert_eq!(
            allocator.nodes[1],
            State::Allocated,
            "32 byte block not marked as allocated"
        ); // 32 byte block
        assert_eq!(
            allocator.nodes[0],
            State::Allocated,
            "64 byte block not marked as allocated"
        ); // 64 byte block
    }

    #[test]
    fn test_find_largest_free_block() {
        static mut HEAP: [u8; 64] = [0; 64];

        let mut allocator =
            unsafe { BuddyAllocator::new(HEAP.as_ptr() as usize, HEAP_SIZE, MIN_BLOCK_SIZE) };

        let idx = allocator.find_block(64).unwrap();

        assert_eq!(idx, 0, "Block not found at expected index");
        assert_eq!(
            allocator.nodes[idx],
            State::Allocated,
            "Block not marked as allocated"
        );
        assert_descendants_allocated(idx, &allocator, "");
    }

    #[test]
    #[should_panic(expected = "Size is not a power of 2")]
    fn test_find_block_not_powerof_2() {
        static mut HEAP: [u8; 64] = [0; 64];

        let mut allocator =
            unsafe { BuddyAllocator::new(HEAP.as_ptr() as usize, HEAP_SIZE, MIN_BLOCK_SIZE) };
        allocator.find_block(61).unwrap();
    }

    #[test]
    #[should_panic(expected = "Requested size is greater than memory size")]
    fn test_find_block_too_big() {
        static mut HEAP: [u8; 64] = [0; 64];

        let mut allocator =
            unsafe { BuddyAllocator::new(HEAP.as_ptr() as usize, HEAP_SIZE, MIN_BLOCK_SIZE) };
        allocator.find_block(128).unwrap();
    }

    #[test]
    fn test_simple_out_of_space() {
        static mut HEAP: [u8; 64] = [0; 64];

        let mut allocator =
            unsafe { BuddyAllocator::new(HEAP.as_ptr() as usize, HEAP_SIZE, MIN_BLOCK_SIZE) };

        allocator.find_block(32).unwrap();
        allocator.find_block(16).unwrap();
        allocator.find_block(8).unwrap();
        allocator.find_block(8).unwrap();

        let last = allocator.find_block(8);
        assert_matches!(last, Err("No block found for allocation"));

        for i in 0..NUM_NODES {
            assert_eq!(
                allocator.nodes[i],
                State::Allocated,
                "Node not marked as allocated"
            );
        }
    }

    #[test]
    fn test_allocate_own_descendants() {
        static mut HEAP: [u8; 64] = [0; 64];

        let mut allocator =
            unsafe { BuddyAllocator::new(HEAP.as_ptr() as usize, HEAP_SIZE, MIN_BLOCK_SIZE) };

        let idx = allocator.find_block(32).unwrap();
        let buddy = allocator.buddy(idx).expect("Node has no buddy");

        assert_descendants_allocated(idx, &allocator, "");
        assert_descendants_unallocated(buddy, &allocator);
    }

    #[test]
    fn test_find_block_multiple_times() {
        static mut HEAP: [u8; 64] = [0; 64];

        let mut allocator =
            unsafe { BuddyAllocator::new(HEAP.as_ptr() as usize, HEAP_SIZE, MIN_BLOCK_SIZE) };

        let first = allocator.find_block(32).unwrap();
        let first_buddy = allocator.buddy(first).expect("Node has no buddy");
        assert_descendants_allocated(first, &allocator, "");
        assert_eq!(allocator.nodes[first_buddy], State::Free);

        let second = allocator.find_block(16).unwrap();
        let second_buddy = allocator.buddy(second).expect("Node has no buddy");
        assert_eq!(allocator.nodes[second_buddy], State::Free);
        assert_descendants_allocated(second, &allocator, "");
        assert_descendants_allocated(first, &allocator, "Second allocation messes up first");

        let expected_idx = allocator.left(first_buddy).unwrap(); // Left child of buddy of first

        assert_eq!(
            second, expected_idx,
            "Block not found at expected index: {:#?}",
            allocator.nodes
        );
    }

    #[test]
    fn test_basic_coalesce() {
        static mut HEAP: [u8; 64] = [0; 64];

        let mut allocator =
            unsafe { BuddyAllocator::new(HEAP.as_ptr() as usize, HEAP_SIZE, MIN_BLOCK_SIZE) };

        let first = allocator.find_block(32).unwrap();
        assert_eq!(
            allocator.nodes[1],
            State::Allocated,
            "Block not marked as allocated"
        );
        assert_eq!(
            allocator.nodes[allocator.buddy(first).unwrap()],
            State::Free,
            "Buddy not marked as free"
        );
        assert_eq!(
            allocator.nodes[0],
            State::Allocated,
            "Root not marked as allocated"
        );

        allocator.free_block(first);

        assert_eq!(allocator.nodes[1], State::Coalesced, "Block not coalesced");
        assert_eq!(
            allocator.nodes[allocator.buddy(first).unwrap()],
            State::Coalesced,
            "Buddy not coalesced"
        );
        assert_eq!(allocator.nodes[0], State::Free, "Root not marked as free");
    }

    #[test]
    fn test_full_coalesce() {
        static mut HEAP: [u8; 64] = [0; 64];

        let mut allocator =
            unsafe { BuddyAllocator::new(HEAP.as_ptr() as usize, HEAP_SIZE, MIN_BLOCK_SIZE) };

        let a = allocator.find_block(32).unwrap();
        let b = allocator.find_block(16).unwrap();
        let c = allocator.find_block(8).unwrap();
        let d = allocator.find_block(8).unwrap();

        println!("{:#?}", allocator.nodes);

        allocator.free_block(d);
        allocator.free_block(c);
        allocator.free_block(b);
        allocator.free_block(a);

        println!("{:#?}", allocator.nodes);

        assert_eq!(
            allocator.nodes[0],
            State::Free,
            "Root not marked as free after coalescing"
        );

        for i in 1..NUM_NODES {
            assert_eq!(
                allocator.nodes[i],
                State::Coalesced,
                "Node ({i}) not marked as unavailable"
            );
        }
    }

    #[test]
    #[should_panic]
    fn test_request_less_than_min_block_size() {
        static mut HEAP: [u8; 64] = [0; 64];

        let mut allocator =
            unsafe { BuddyAllocator::new(HEAP.as_ptr() as usize, HEAP_SIZE, MIN_BLOCK_SIZE) };

        let idx = allocator.find_block(4);
    }
}
