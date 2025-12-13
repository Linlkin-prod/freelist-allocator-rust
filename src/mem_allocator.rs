use core::alloc::Layout;
use core::{ptr, mem};
use std::os::raw;

const HEAP_SIZE : usize = 1024 * 1024; // 1 MiB

static mut HEAP : [u8; HEAP_SIZE] = [0; HEAP_SIZE];

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


impl FreeListAllocator {
    pub unsafe fn init(&mut self) {
        self.heap_start = ptr::addr_of_mut!(HEAP) as *mut u8;
        self.heap_size = HEAP_SIZE;

        let header = self.heap_start as *mut BlockHeader;
        unsafe {
            (*header).size = self.heap_size - mem::size_of::<BlockHeader>();    
            (*header).next = ptr::null_mut();
        }
        self.head = header;
    }

    pub unsafe fn alloc(&mut self, layout: Layout) -> *mut u8 {
        let mut prev: *mut BlockHeader = ptr::null_mut();
        let mut current = self.head;

        let header_size = mem::size_of::<BlockHeader>();
        let back_ptr_size = mem::size_of::<*mut BlockHeader>();

        while !current.is_null() {

            let block_start = current as usize;
            let raw_user_start = block_start + header_size + back_ptr_size;
            let user_start = align_up(raw_user_start, layout.align());
            let padding = user_start - (block_start + header_size);
            let bytes_consumed_in_block = layout.size() + padding + back_ptr_size;

            unsafe {
                if (*current).size >= bytes_consumed_in_block {
                
                    if prev.is_null() {
                        self.head = (*current).next;
                    } else {
                        (*prev).next = (*current).next;
                    }
                    
                    // Store back-pointer just before user data
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
        if ptr.is_null() {
            return;
        }

        unsafe {
            let back_ptr_location = (ptr as *mut *mut BlockHeader).offset(-1);
            let header = *back_ptr_location;

            (*header).next = self.head;

            self.head = header;
        }
    }
}

pub fn create_allocator() -> FreeListAllocator {
    FreeListAllocator {
        heap_start: ptr::null_mut(),
        heap_size: 0,
        head: ptr::null_mut(),
    }
}

fn align_up(addr : usize, align : usize) -> usize {
    debug_assert!(align.is_power_of_two());
    (addr + align - 1) & !(align - 1)
}
