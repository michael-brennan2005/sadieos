use core::ptr;

use elf::{ElfBytes, endian::{LittleEndian}, file::parse_ident};
use x86_64::{VirtAddr, registers::{control::{Cr3, Cr3Flags}, rflags::RFlags}, structures::paging::{FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PageTableFlags, Size4KiB, page::{self, PageRangeInclusive}}};

use crate::{memory::translate_addr, println};

pub enum ProcessState {
    Ready,
    Running,
}

pub struct Process {
    state: ProcessState,
    context: u8,
}

impl Process {
    pub fn init_from_elf_bytes(frame_allocator: &mut impl FrameAllocator<Size4KiB>, physical_memory_offset: VirtAddr, bytes: &[u8]) {
        let file = ElfBytes::<LittleEndian>::minimal_parse(bytes).expect("Open failed");
        
        // Safety checks
        let (_, class, abi, _) = parse_ident::<LittleEndian>(bytes).expect("Parse identifier failed");

        match class {
            elf::file::Class::ELF64 => {},
            _ => panic!("Wrong ELF format (expected 64-bit)")
        }   

        if abi != 0 {
            panic!("Wrong ABI (expected System V ABI)");
        }     

        if file.ehdr.e_type != 0x2 {
            panic!("Wrong file type (expected ET_EXEC - non-PIE, absolute addresses)");
        } 

        let entry = file.ehdr.e_entry;
        
        // TODO: Page table for process needs to include kernel mapped memory as well, marked
        // supervisor-only, so that when 

        // Create page table for process
        let l4_table_frame = frame_allocator.allocate_frame().expect("Failed to allocate frame for L4 page table");
        let mut l4_table = {
            let virt = physical_memory_offset + l4_table_frame.start_address().as_u64();

            // x86_64 crate types are low-level wrappers so we can do this to get the mem frame as a page table. 
            let page_table_ptr: *mut PageTable = virt.as_mut_ptr();
            let page_table_ref: &mut PageTable = unsafe { &mut *page_table_ptr };
            page_table_ref.zero();

            // Create offset page tale so we can take advantage of mapper::map_to
            unsafe { OffsetPageTable::new(page_table_ref, physical_memory_offset) }
        };
        
        // Map code/data segments into pages
        let phdrs = file.segments().expect("Failed to parse segments");
        for phdr in phdrs {
            if phdr.p_type == 1 { // Loadable segments only
                println!("Mapping file [{:#X}, {:#X}] to mem [{:#X}, {:#X}]", phdr.p_offset, phdr.p_offset + phdr.p_filesz, phdr.p_vaddr, phdr.p_vaddr + phdr.p_memsz);
                
                let vaddr_start = VirtAddr::new(phdr.p_vaddr);
                let vaddr_end = VirtAddr::new(phdr.p_vaddr + phdr.p_memsz);
                
                let data = file.segment_data(&phdr).unwrap();
    
                // Create page range for the segment
                let page_range: PageRangeInclusive<Size4KiB> =Page::range_inclusive(
                    Page::containing_address(vaddr_start), 
                    Page::containing_address(vaddr_end));
                
                // Allocate necessary frames for each page, and create corresponding page table entries
                for page in page_range {
                    let frame = frame_allocator.allocate_frame().expect("Page alloc failed");
                    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE; // TODO: finer-grained page permissions?
                    unsafe { l4_table.map_to(page, frame, flags, frame_allocator).expect("Mapping failed :(").flush() };

                    println!("  Page @ {:#X} -> Frame @ {:#X}", page.start_address(), frame.start_address());
                }

                for page in page_range {
                    // TODO: zero-filling
                    let data_start = page.start_address().as_u64().saturating_sub(vaddr_start.as_u64());
                    let data_end = (data_start + page.size()).min(phdr.p_memsz);
                    let virt_start = page.start_address().as_u64().max(phdr.p_vaddr);
                
                    
                    if data_start > phdr.p_vaddr + phdr.p_memsz {
                        println!("TODO: something with zero-filling");
                    }

                    println!("  Data [{:#X}, {:#X}] is getting copied to Virt {:#X}", data_start, data_end, virt_start);
                    let phys_start = {
                        let phys = unsafe { translate_addr(VirtAddr::new(virt_start), l4_table_frame.start_address(), physical_memory_offset).unwrap() };
                        physical_memory_offset + phys.as_u64() // offset so we can write to it 
                    };

                    unsafe {    
                        ptr::copy_nonoverlapping(
                            data.as_ptr().add(data_start as usize), 
                            phys_start.as_mut_ptr(), 
                            (data_end - data_start) as usize);
                    }
                }
            }
        }

        // Create 2MB heap and 2MB stack, save addresses
        
        // Init 2MiB heap
        let heap_start: VirtAddr = VirtAddr::new(0x200_0000);
        const HEAP_SIZE: u64 = 0x2_0000; // 2 MiB
        let page_range: PageRangeInclusive<Size4KiB> =Page::range_inclusive(
            Page::containing_address(heap_start), 
            Page::containing_address(heap_start + HEAP_SIZE));
        
        println!("Creating heap");
        println!("  Creating pages {:#X} to {:#X}", page_range.start.start_address(), page_range.end.start_address());
        for page in page_range {
            let frame = frame_allocator.allocate_frame().expect("Page alloc failed");
            let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE; // TODO: finer-grained page permissions?
            unsafe { l4_table.map_to(page, frame, flags, frame_allocator).expect("Mapping failed :(").flush() };
        }

        // Init 2MiB for stack
        let stack_start: VirtAddr = VirtAddr::new(0x400_0000);
        const STACK_SIZE: u64 = 0x2_0000; // 2 MiB
        let page_range: PageRangeInclusive<Size4KiB> =Page::range_inclusive(
            Page::containing_address(stack_start - STACK_SIZE), 
            Page::containing_address(stack_start));
        
        println!("Creating stack");
        println!("  Creating pages {:#X} to {:#X}", page_range.start.start_address(), page_range.end.start_address());
        for page in page_range {
            let frame = frame_allocator.allocate_frame().expect("Page alloc failed");
            let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE; // TODO: finer-grained page permissions?
            unsafe { l4_table.map_to(page, frame, flags, frame_allocator).expect("Mapping failed :(").flush() };
        }

        //Find the registers
        println!("PERFORMING USER MODE TRANSITION!!!!!");
        
        unsafe {
            Cr3::write(l4_table_frame, Cr3Flags::empty());

            loop {
                x86_64::instructions::hlt();
            }
            // core::arch::asm!(
            //     "
            //     cli                 // Disable interrupts before switching to ensure atomic transition
            //     push {user_data_ss} // SS: User Data Segment Selector
            //     push {user_rsp}     // RSP: User Stack Pointer
            //     push {user_rflags}  // RFLAGS: Flags for user mode
            //     push {user_code_cs} // CS: User Code Segment Selector
            //     push {user_rip}     // RIP: User Entry Point
            //     iretq               // Interrupt Return: pops the above values and jumps
            //     ",
            //     user_data_ss = in(reg) 0x18,
            //     user_rsp = in(reg) stack_start.as_u64(),
            //     user_rflags = in(reg) (RFlags::INTERRUPT_FLAG).bits(),
            //     user_code_cs = in(reg) 0x23,
            //     user_rip = in(reg) entry,
            //     options(noreturn) // This assembly block will not return
            // );
        };
    }
}