# Emulator architecture

[Schema](architecture.pdf)

* emulator run from start to end
    * `main`
        * parsing arguments
        * initialization of logging facilities
        * creates memory image using `elf` module
        * creates initial stack frame in memory if necessary
        * starts emulator `cpu::control::EmulatorContext`
    * `EmulatorContext`
        * stores itself as a singleton (the only reason are signal handlers, otherwise it's useless and unsafe)
        * initilizes `Watchdog` to perform runtime checks of execution
        * initializes registers and data structure for supporting syscalls
        * starts `EmulatorContext::cpu_loop`
    * `EmulatorContext::cpu_loop`
        * in an infinite loop
            * fetch instruction
            * eval instruction
            * run `Watchdog` to perform checks
            * using result of last instruction, plans next instruction
        * in case of `exit` or `exit_group` syscall, the actual syscall is ignored and the loop ends
* `Memory`
    * allocates 4GB array of zeroes - kernel handles deduplication, so it does not actually take 4GBs of RAM
    * supports both big and little endian access modes
* `syscalls`
    * parses syscall instruction
    * attempts to translate data structures passed around and calls the kernel using `libc` or its `nix` Rust wrapper
* `Watchdog`
    * can load gziped JSON with instruction addresses and register values created with GDB and real HW / Qemu
    * has hooks in registers, so that register reads and writes can be checked
    * checks for jumps to null and too many nops in succession

# Code documentation

A documentation is inside the actual source code. You can also access it by running `cargo doc`, but there's not much of it, because almost everything is internal and not exported.