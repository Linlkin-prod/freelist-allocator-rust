use core::alloc::{Layout, GlobalAlloc};
use core::{ptr, mem};
use core::fmt::Write;
use core::cell::UnsafeCell;

const HEAP_SIZE : usize = 1024 * 1024; // 1 MiB
const DEBUG_BUFFER_SIZE: usize = 4096;
const MIN_FREE_BLOCK: usize = mem::size_of::<BlockHeader>() 
                            + mem::align_of::<BlockHeader>() 
                            + mem::size_of::<*mut BlockHeader>();

//---------------------- Debug logging buffer-------------------------
struct DebugLogger {
    buffer: [u8; DEBUG_BUFFER_SIZE],
    index: usize,
}

impl DebugLogger {
    const fn new() -> Self {
        DebugLogger {
            buffer: [0; DEBUG_BUFFER_SIZE],
            index: 0,
        }
    }

    fn reset(&mut self) {
        self.index = 0;
    }

    fn get_messages(&self) -> &[u8] {
        &self.buffer[..self.index]
    }
}

impl Write for DebugLogger {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let bytes = s.as_bytes();
        let available = DEBUG_BUFFER_SIZE - self.index;
        let to_write = core::cmp::min(bytes.len(), available);
        
        if to_write > 0 {
            self.buffer[self.index..self.index + to_write].copy_from_slice(&bytes[..to_write]);
            self.index += to_write;
        }
        
        Ok(())
    }
}

static mut DEBUG_LOGGER: DebugLogger = DebugLogger {
    buffer: [0; DEBUG_BUFFER_SIZE],
    index: 0,
};

// Macro for debug logging (only in debug builds)
#[allow(unused_macros)]
macro_rules! debug_log {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        unsafe {
            let logger = ptr::addr_of_mut!(DEBUG_LOGGER);
            let _ = writeln!(&mut *logger, $($arg)*);
        }
    };
}

pub fn get_debug_logs() -> &'static [u8] {
    unsafe {
        let logger = ptr::addr_of!(DEBUG_LOGGER);
        (*logger).get_messages()
    }
}

pub fn clear_debug_logs() {
    unsafe {
        let logger = ptr::addr_of_mut!(DEBUG_LOGGER);
        (*logger).reset();
    }
}


//---------------------- Simple Free List Allocator -------------------------
#[repr(align(16))]
struct AlignedHeap([u8; HEAP_SIZE]);

static mut HEAP : AlignedHeap = AlignedHeap([0; HEAP_SIZE]);

pub struct SAllocator {
    inner: UnsafeCell<FreeListAllocator>,
}

#[repr(C)]
struct BlockHeader {
    // Size of the block after the header
    // (back_ptr + padding + user data + free space)
    size: usize,
    next: *mut BlockHeader,
}

pub struct FreeListAllocator {
    heap_start: *mut u8,
    heap_size: usize,
    head: *mut BlockHeader,
}

unsafe impl Sync for SAllocator {}

unsafe impl GlobalAlloc for SAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        unsafe {
            let allocator = &mut *self.inner.get();
            allocator.alloc(layout)
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        unsafe {
            let allocator = &mut *self.inner.get();
            allocator.dealloc(ptr);
        }
    }
}

#[global_allocator]
static ALLOCATOR: SAllocator = SAllocator {
    inner: UnsafeCell::new(FreeListAllocator {
        heap_start: ptr::null_mut(),
        heap_size: 0,
        head: ptr::null_mut(),
    }),
};


impl FreeListAllocator {
    pub unsafe fn init(&mut self) {
        debug_log!("Initializing allocator with {} bytes", HEAP_SIZE);
        unsafe {
            self.heap_start = ptr::addr_of_mut!(HEAP.0[0]);
        }
        
        self.heap_size = HEAP_SIZE;

        let header = self.heap_start as *mut BlockHeader;
        unsafe {
            (*header).size = self.heap_size - mem::size_of::<BlockHeader>();    
            (*header).next = ptr::null_mut();
        }
        self.head = header;
    }

    pub unsafe fn alloc(&mut self, layout: Layout) -> *mut u8 {
        debug_log!("Allocating {} bytes with alignment {}", layout.size(), layout.align());
        let mut prev: *mut BlockHeader = ptr::null_mut();
        let mut current = self.head;

        let header_size = mem::size_of::<BlockHeader>();
        let back_ptr_size = mem::size_of::<*mut BlockHeader>();

        while !current.is_null() {

            let block_start = current as usize;
            let raw_user_start = block_start + header_size + back_ptr_size;
            let user_start = align_up(raw_user_start, layout.align());
            let padding = user_start - raw_user_start;
            let bytes_consumed_in_block = back_ptr_size + padding + layout.size()  ;

            unsafe {
                if (*current).size >= bytes_consumed_in_block {
                    let remaining_size = (*current).size - bytes_consumed_in_block;
                    let unaligned_addr = block_start + header_size + bytes_consumed_in_block;
                    let aligned_addr = align_up(unaligned_addr, mem::align_of::<BlockHeader>()) as *mut BlockHeader;
                    let wasted_space = (aligned_addr as usize) - unaligned_addr;
                    if can_be_split(remaining_size) && remaining_size >= wasted_space + mem::size_of::<BlockHeader>(){

                        let new_block_addr = aligned_addr;
                        (*new_block_addr).size = remaining_size - wasted_space - header_size;
                        (*new_block_addr).next = (*current).next;

                        (*current).size = bytes_consumed_in_block;
                        (*current).next = ptr::null_mut();

                        if prev.is_null() {
                            self.head = new_block_addr;
                        } else {
                            (*prev).next = new_block_addr;
                        }
                    } else {
                
                        if prev.is_null() {
                            self.head = (*current).next;
                        } else {
                            (*prev).next = (*current).next;
                        }
                    }
                    let back_ptr_location = (user_start - back_ptr_size) as *mut *mut BlockHeader;
                    *back_ptr_location = current;
                    
                    return user_start as *mut u8;
                }

                prev = current;
                current = (*current).next;
            }
        }

        ptr::null_mut()
    }

    pub unsafe fn dealloc(&mut self, ptr: *mut u8) {
        debug_log!("Deallocating pointer {:p}", ptr);
        if ptr.is_null() {
            return;
        }

        unsafe {
            let back_ptr_location = (ptr as *mut *mut BlockHeader).offset(-1);
            let header = *back_ptr_location;
            let mut prev : *mut BlockHeader = ptr::null_mut();
            let mut current = self.head;

            while !current.is_null() && (current as usize) < (header as usize) {
                prev = current;
                current = (*current).next;
            }

            (*header).next = current;

            if prev.is_null() {
                self.head = header;
            } else {
                (*prev).next = header;
            }

            let header_end = (header as usize) + mem::size_of::<BlockHeader>() + (*header).size;

            if !(*header).next.is_null() && header_end == (*header).next as usize {
                let next_block = (*header).next;
                (*header).size += mem::size_of::<BlockHeader>() + (*next_block).size;
                (*header).next = (*next_block).next;

            }

            if prev.is_null() == false {
                let prev_end = (prev as usize) + mem::size_of::<BlockHeader>() + (*prev).size;
                if prev_end == (header as usize) {
                    (*prev).size += mem::size_of::<BlockHeader>() + (*header).size;
                    (*prev).next = (*header).next;
                }
            }
        }
    }
}

fn align_up(addr : usize, align : usize) -> usize {
    debug_assert!(align.is_power_of_two());
    (addr + align - 1) & !(align - 1)
}

fn can_be_split(size: usize) -> bool {
    size >= MIN_FREE_BLOCK
}

pub fn init_allocator() {
    unsafe {
        (*ALLOCATOR.inner.get()).init();
    }
}

