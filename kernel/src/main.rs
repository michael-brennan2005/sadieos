#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points

use bootloader_api::{BootInfo, BootloaderConfig, config::Mapping, entry_point};
use x86_64::instructions::hlt;
use core::fmt::Write;

use crate::framebuffer::FrameBufferWriter;

pub mod framebuffer;
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
    
    let mut fbw = if let Some(fb) = boot_info.framebuffer.take() {
        let info = fb.info();
        Some(FrameBufferWriter::new(fb.into_buffer(), info))
    } else {
        None
    };
    
    if let Some(mut fbw) = fbw {
        fbw.write_str("Hello, World!").unwrap();
    }

    sprintln!("\n=(^.^)= meow\n");

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
