#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(sd_kernel::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use bootloader_api::BootloaderConfig;
use sd_kernel::{allocator, memory::{self, BootInfoFrameAllocator}, println, process::Process, serial_println};
use bootloader::{BootInfo, entry_point};
use x86_64::{VirtAddr, structures::paging::{Page, PageTable, Translate}};

extern crate alloc;
use alloc::boxed::Box;

pub const BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.dynamic_range_start = Some(0xffff_8000_0000_0000);
    config
};

entry_point!(kernel_main);

#[unsafe(no_mangle)]
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    println!("Hello World{}", "!");
    
    let program = include_bytes!("../../userland/hello-asm/hello");

    sd_kernel::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    println!("PTR -> {:#X}", (&phys_mem_offset as *const VirtAddr) as u64);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe {
        BootInfoFrameAllocator::init(&boot_info.memory_map)
    };

    println!("--- Memory map ---");
    for region in boot_info.memory_map.iter() {
        serial_println!("region.range = {:?}, region.region_type = {:?}", region.range, region.region_type);
    }

    // new
    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("heap initialization failed");

    // Process::init_from_elf_bytes(&mut frame_allocator, phys_mem_offset, program);

    
    #[cfg(test)]
    test_main();

    println!("It did not crash!");
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