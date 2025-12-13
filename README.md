# Freelist Allocator Rust

A custom memory allocator implementation in Rust using a free list algorithm.

## Overview

This project implements a simple but functional memory allocator with the following features:

- **Free List Algorithm**: Tracks free and allocated memory blocks using a linked list
- **Fixed Heap Size**: 1 MiB pre-allocated heap (for now, mb dynamic later)
- **Block Header Metadata**: Each allocation includes header information for size and status tracking
- **Allocation & Deallocation**: Core memory management operations

## Architecture

### Components

- **`FreeListAllocator`**: Main allocator struct managing the heap
- **`BlockHeader`**: Metadata structure for tracking block information
  - `size`: Size of the allocated/free block
  - `is_free`: Boolean flag indicating if block is available
  - `next`: Pointer to next block in the free list

### Heap Management

- Total heap size: 1 MiB (1024 * 1024 bytes) (for now)
- Static mutable buffer storing the heap data
- Initialization sets up the first free block spanning the entire heap

## Usage

```rust
use mem_alloc_rust::create_allocator;
use core::alloc::Layout;

fn main() {
    let mut allocator = create_allocator();
    
    // Initialize the allocator
    unsafe {
        allocator.init();
        
        // Allocate memory
        let layout = Layout::new::<i32>();
        let ptr = allocator.alloc(layout);
        
        if !ptr.is_null() {
            // Use allocated memory
            *(ptr as *mut i32) = 42;
            println!("Value: {}", *(ptr as *mut i32));
            
            // Free the memory
            allocator.dealloc(ptr);
        }
    }
}
```

## API

### `create_allocator() -> FreeListAllocator`
Creates a new uninitialized allocator instance.

### `init(&mut self)` (unsafe)
Initializes the allocator and sets up the heap structure.

### `alloc(&mut self, layout: Layout) -> *mut u8` (unsafe)
Allocates memory according to the specified layout. Returns a raw mutable pointer to the allocated memory, or a null pointer if allocation fails.

### `dealloc(&mut self, ptr: *mut u8)` (unsafe)
Deallocates memory at the given pointer, marking it as free for future allocations.

## Safety

All unsafe operations are contained within explicit `unsafe` blocks and require careful usage. The allocator:
- Works with raw pointers and mutable statics
- Requires the caller to ensure proper pointer alignment and validity
- Does not protect against use-after-free or double-free errors

## Building

```bash
cargo build
```

## Running

```bash
cargo run
```

## Requirements

- Rust 2024 edition or later
- `core` library (no_std compatible)

## Future Improvements

- Fragmentation reduction strategies
- Memory coalescing
- Support for custom heap sizes
- Thread-safe allocator variant
- Benchmarking and optimization
