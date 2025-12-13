use core::alloc::Layout;
use core::{ptr, mem};

const HEAP_SIZE : usize = 1024 * 1024; // 1 MiB

static mut HEAP : [u8; HEAP_SIZE] = [0; HEAP_SIZE];

#[repr(C)]
struct BlockHeader {
    size: usize,
    is_free: bool,
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
            (*header).is_free = true;
            (*header).next = ptr::null_mut();
        }
        self.head = header;
    }

    pub unsafe fn alloc(&mut self, layout: Layout) -> *mut u8 {
        let mut prev: *mut BlockHeader = ptr::null_mut();
        let mut current = self.head;

        while !current.is_null() {
            unsafe {
                if (*current).is_free && (*current).size >= layout.size() {
                
                    if prev.is_null() {
                        self.head = (*current).next;
                    } else {
                        (*prev).next = (*current).next;
                    }
                    (*current).is_free = false;

                    return (current as *mut u8).add(mem::size_of::<BlockHeader>());
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
            let header = (ptr as *mut BlockHeader).sub(1);

            (*header).is_free = true;
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
