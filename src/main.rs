mod mem_allocator;
use mem_allocator::create_allocator;
use core::alloc::Layout;

fn main() {
    println!("Memory Allocator Test");
    let mut allocator = create_allocator();
    
    // Initialize the allocator
    unsafe {
        allocator.init();
        
        // Allocate of an integer
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

        // Allocation of a string
        let test_str = "Hello, Memory Allocator!";
        let str_ptr = allocator.alloc_string(test_str);
        if !str_ptr.is_null() {
            let retrieved_str = allocator.ptr_to_str(str_ptr, test_str.len());
            println!("Allocated string: {}", retrieved_str);
            
            // Deallocate string memory
            allocator.dealloc(str_ptr);
            println!("String memory deallocated");
        } else {
            println!("String allocation failed");
        }
    }
}
