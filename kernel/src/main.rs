#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
#![feature(abi_x86_interrupt)] // needed for "x86-interrupt" call-conv

use alloc::vec::Vec;
use bootloader_api::{BootInfo, BootloaderConfig, config::Mapping, entry_point};
use x86_64::{VirtAddr, instructions::hlt, structures::paging::FrameAllocator};
use core::fmt::Write;

use crate::{framebuffer::FrameBufferWriter, memory::BootInfoFrameAllocator};

extern crate alloc;

pub mod framebuffer;
pub mod interrupts;
pub mod memory;
pub mod serial;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) -> ! {
    use x86_64::instructions::{nop, port::Port};

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }

    loop {
        nop();
    }
}

pub const BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config.mappings.dynamic_range_start = Some(0xffff_8000_0000_0000);
    
    config
};

entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    sprintln!("Entered kernel with boot info: {:?}", boot_info);
    
    // GDT/IDT init
    interrupts::init_gdt();
    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize(); };
    x86_64::instructions::interrupts::enable();

    // Memory init
    let physical_mem_offset = VirtAddr::new(boot_info.physical_memory_offset.take().unwrap());
    let mut kernel_mem_table = unsafe { memory::init(physical_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_regions) };
    memory::init_heap(&mut kernel_mem_table, &mut frame_allocator).expect("Heap init failed");

    // Framebuffer init
    if let Some(fb) = boot_info.framebuffer.take() {
        framebuffer::init_fb(fb);
    }

    fbprint!("\nyoooo\n");
    
    sprintln!("Physical mem offset -> {:#X}", physical_mem_offset);
    sprintln!("MEMORY REGIONS");
    for region in boot_info.memory_regions.iter() {
        sprintln!("    ({:?}) {:#X} <-> {:#X}", region.kind, region.start, region.end);
    }

    loop {
        hlt();
    }

    exit_qemu(QemuExitCode::Success);
}

/// This function is called on panic.
#[panic_handler]
#[cfg(not(test))]
fn panic(info: &core::panic::PanicInfo) -> ! {
    sprintln!("PANIC: {}", info);
    exit_qemu(QemuExitCode::Failed);
}
