#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(sd_kernel::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use sd_kernel::{allocator, memory::{self, BootInfoFrameAllocator, translate_addr}, println};
use bootloader::{BootInfo, entry_point};
use x86_64::{VirtAddr, structures::paging::{Page, PageTable, Translate}};

extern crate alloc;
use alloc::boxed::Box;

entry_point!(kernel_main);

#[unsafe(no_mangle)]
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    println!("Hello World{}", "!");
    
    let program = include_bytes!("../../userland/hello-asm/hello-nasm-zig");

    sd_kernel::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe {
        BootInfoFrameAllocator::init(&boot_info.memory_map)
    };

    // new
    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("heap initialization failed");

    let x = Box::new(41);

    #[cfg(test)]
    test_main();

    println!("It did not crash!");
    println!("Program is {} bytes", program.len());
    sd_kernel::hlt_loop();
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    sd_kernel::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    sd_kernel::test_panic_handler(info)
}