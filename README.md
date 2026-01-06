# Freelist Allocator Rust

A custom memory allocator implementation in Rust using a free list algorithm with block splitting and coalescence.

## Overview

This project implements a functional memory allocator with the following features:

- **Free List Algorithm**: Manages free and allocated memory blocks using a linked list structure
- **Block Splitting**: Automatically splits large free blocks to reduce memory waste
- **Block Coalescence**: Merges adjacent free blocks to prevent fragmentation
- **Alignment Support**: Respects custom alignment requirements for allocations
- **GlobalAlloc Trait**: Integrates with Rust's standard allocator interface
- **Debug Logging**: Built-in debug logging system (4KB buffer) for troubleshooting
- **Fixed Heap Size**: 1 MiB pre-allocated heap

## Architecture

### Components

- **`FreeListAllocator`**: Core allocator managing the heap with first-fit allocation strategy
- **`SAllocator`**: Thread-safe wrapper implementing the `GlobalAlloc` trait
- **`BlockHeader`**: Metadata structure for each memory block
  - `size`: Size of the block's user data (excluding header and alignment padding)
  - `next`: Pointer to the next block in the free list
- **`DebugLogger`**: Circular buffer for allocation event logging

### Heap Management

- **Total heap size**: 1 MiB (1024 * 1024 bytes)
- **Static allocation**: Pre-allocated 16-byte aligned heap buffer
- **Initialization**: Sets up the first free block spanning the entire heap
- **Minimum block size**: Enforced to prevent fragmentation
- **Alignment handling**: Supports arbitrary power-of-two alignment requirements

## Algorithm Details

### Allocation (First-Fit Strategy)
1. Searches the free list for the first block large enough to satisfy the request
2. Applies alignment padding before the user data
3. Splits the block if remaining space exceeds the minimum threshold
4. Returns a pointer to aligned user data

### Deallocation
1. Locates the block header using a back pointer stored before user data
2. Reinserts the block into the free list in sorted order
3. **Forward coalescing**: Merges with the next block if adjacent
4. **Backward coalescing**: Merges with the previous block if adjacent

## Usage

### Basic Example

```rust
fn main() {
    let a = Box::new(42);
    let b = Box::new("Hello, World!");
    
    println!("Box a = {}", a);
    println!("Box b = {}", b);
    
    // Display debug logs
    let logs = mem_allocator::get_debug_logs();
    if let Ok(log_str) = core::str::from_utf8(logs) {
        println!("Debug logs:\n{}", log_str);
    }
}
```

### Direct Allocator Usage

```rust
use core::alloc::Layout;

unsafe {
    let allocator = &mut *ALLOCATOR.inner.get();
    allocator.init();
    
    // Allocate memory
    let layout = Layout::new::<i32>();
    let ptr = allocator.alloc(layout);
    
    if !ptr.is_null() {
        *(ptr as *mut i32) = 42;
        println!("Value: {}", *(ptr as *mut i32));
        allocator.dealloc(ptr);
    }
}
```

## API

### Public Functions

#### `get_debug_logs() -> &'static [u8]`
Retrieves the current debug log buffer contents as a byte slice. Logs include allocation/deallocation events with sizes and alignments.

#### `clear_debug_logs()`
Clears the debug log buffer, resetting it for fresh logging.

## Debug Logging

Debug logs are automatically captured during:
- **Allocator initialization** - Records heap setup
- **Memory allocation** - Logs allocation size and alignment requirements
- **Memory deallocation** - Records dealloc pointer addresses

Logs are only generated in debug builds (`#[cfg(debug_assertions)]`) to minimize overhead in release mode.

## Safety Considerations

- All unsafe operations are contained within explicit `unsafe` blocks
- The allocator uses raw pointers to avoid mutable static reference violations
- Proper alignment is maintained throughout the allocation process
- Back pointers enable efficient deallocation without external tracking
- Users must ensure:
  - No use-after-free violations
  - No double-free errors
  - Correct Layout specifications

## Building

```bash
cargo build
```

## Running

```bash
cargo run
```

## Testing

```bash
cargo test
```

## Requirements

- Rust 1.70+ (stable)
- Nightly Rust features: `alloc_error_handler` (for panic on allocation failure)
- `core` library (no_std compatible)

## Performance Characteristics

- **Allocation**: O(n) where n = number of free blocks (first-fit search)
- **Deallocation**: O(n) for free list reinsertion and coalescing
- **Fragmentation**: Minimized through block splitting and coalescence strategies
- **Memory overhead**: ~24 bytes per block (BlockHeader + back pointer + alignment)
