- PLAN: get the rust hello world program to work
    - No PIE, no ASLR
- Processes
    - (DONE) Embed file for now to make things easy (no filesystem)
    - Need some way to do ELF parsing/loading
        - Goal: VGA debug print what needs to go where, see if we can cross validate with elfinfo or sumn
    - Need way to create page table for program, then map & load memory
        - Goal: ???
    - Direct execution (from OSTEP - release into usermode, have syscall interrupts)
        - Goal: Program runs and it triggers a fault on syscall (when it tries to print)
    - Basic syscall mocks: Read and write for stdout (straight to VGA buffer for now)
        - Goal: It prints

        - 
        offset page table is just used because thats how it knows how to convert between physical and virtual addresses!!! All that stuff is getting the active level 4 table at init -
        hence the kernel mem, so we can initialize a heap and stuff


- TODO: (key problem: we need to move to higher half kernel, and map kernel pages to process pages so we code still actually runs)
    - use bootloader_api to move to a higher half kernel - see bootloader/tests for how to do this
    - have a function that initializes a page table for a process, key thing being this page table having identity mappings for kernel memory
    - if you go through and initialize/allocate the necessary pages for the process, then switch your CR3 to the process's page table, data copying becomes extremely simple (from code standpoint).
        - This may blow up with PIE and ASLR but fuck that don't worry about that for now; I think it also keeps it simple because it would still look like copying just to some offset now.

    - Move debug printing to a serial line
    - Initializing pages for process memory and copying data should be moved to memory.rs functions


- 