```
// Smallest block = 8 bytes
// Allocatable memory size = 64 bytes
// Highest order = 3 (2^3 * 8 = 64)
// Calculate order from smallest block and allocatable memory size:
// 2 ^ n = memsize / blocksize
// n = log2(memsize / blocksize)

// 3               64
// 2        32             32
// 1   16      16      16     16
// 0 8   8   8   8   8   8   8   8

// Number of nodes = 2 ^ (order + 1) - 1
// Total size required to bookkeep allocatable memory = num_nodes * node_size
// For order = 3 and node_size = 8, total size = (2 ^ (3 + 1) - 1) * 8 = 15 * 8
// = 120
```