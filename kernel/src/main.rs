#![no_std]
#![no_main]

use core::panic::PanicInfo;
// use sd_kernel::{allocator, memory::{self, BootInfoFrameAllocator}, println, process::Process, serial_println};
use bootloader_api::{BootInfo, BootloaderConfig, config::Mapping, entry_point};
use x86_64::{VirtAddr, structures::paging::{Page, PageTable, Translate}};

// extern crate alloc;
// use alloc::boxed::Box;

// pub mod allocator;
// pub mod gdt;
// pub mod interrupts;
//pub mod memory;
// pub mod process;
// pub mod serial;
// pub mod vga_buffer;

pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};

entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

#[unsafe(no_mangle)]
fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    // Init framebuffer
    // if let Some(fb) = boot_info.framebuffer.take();


    // serial_println!("Hello World{}", "!");
    
    // Init code
    // gdt::init();
    // interrupts::init_idt();
    // unsafe { interrupts::PICS.lock().initialize(); };
    // x86_64::instructions::interrupts::enable();

    
    // let program = include_bytes!("../../userland/hello-asm/hello");
    // let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    // println!("PTR -> {:#X}", (&phys_mem_offset as *const VirtAddr) as u64);
    // let mut mapper = unsafe { memory::init(phys_mem_offset) };
    // let mut frame_allocator = unsafe {
    //     BootInfoFrameAllocator::init(&boot_info.memory_map)
    // };

    // println!("--- Memory map ---");
    // for region in boot_info.memory_map.iter() {
    //     serial_println!("region.range = {:?}, region.region_type = {:?}", region.range, region.region_type);
    // }

    // new
    // allocator::init_heap(&mut mapper, &mut frame_allocator)
    //     .expect("heap initialization failed");

    // Process::init_from_elf_bytes(&mut frame_allocator, phys_mem_offset, program);

    // println!("It did not crash!");
    hlt_loop();
}

/// This function is called on panic.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // println!("{}", info);
    hlt_loop();
}

fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}
