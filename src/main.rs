mod mem_allocator;
use mem_allocator::create_allocator;
use core::alloc::Layout;

fn main() {
    println!("Memory Allocator Test");
    let mut allocator = create_allocator();
    
    // Initialize the allocator
    unsafe {
        allocator.init();
        
        // Allocate memory
        let layout = Layout::new::<i32>();
        let ptr = allocator.alloc(layout);
        
        if !ptr.is_null() {
            // Write to allocated memory
            *(ptr as *mut i32) = 42;
            println!("Allocated memory, value: {}", *(ptr as *mut i32));
            
            // Deallocate memory
            allocator.dealloc(ptr);
            println!("Memory deallocated");
        } else {
            println!("Allocation failed");
        }
    }
}
