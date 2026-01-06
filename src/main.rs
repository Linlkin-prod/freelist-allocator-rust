#![allow(dead_code)]


mod mem_allocator;
use core::alloc::Layout;


fn main() {
    
    mem_allocator::init_allocator();

    println!("Allocator initialized");

    // =========================
    // Box<T>
    // =========================
    let a = Box::new(42);
    let b = Box::new(1337);

    println!("Box a = {}", a);
    println!("Box b = {}", b);

    // =========================
    // Vec<T>
    // =========================
    let mut v = Vec::new();

    for i in 0..10 {
        v.push(i * i);
    }

    println!("Vec = {:?}", v);

    // =========================
    // String
    // =========================
    let s = String::from("Hello allocator!");
    println!("String = {}", s);

    // =========================
    // Stress test
    // =========================
    for i in 0..100 {
        let x = Box::new(i);
        drop(x);
    }

    println!("Stress test done");

    // =========================
    // Raw allocation
    // =========================
    unsafe {
        let layout = Layout::from_size_align(64, 8).unwrap();
        let ptr = std::alloc::alloc(layout);

        if ptr.is_null() {
            panic!("allocation failed");
        }

        // Write data
        for i in 0..64 {
            ptr.add(i).write(i as u8);
        }

        // Freed data
        std::alloc::dealloc(ptr, layout);
    }

    println!("Manual allocation OK");

    // Display debug logs
    println!("\n=== Debug Logs ===");
    let logs = mem_allocator::get_debug_logs();
    if let Ok(log_str) = core::str::from_utf8(logs) {
        println!("{}", log_str);
    } else {
        println!("(Debug logs contain invalid UTF-8)");
    }
}
