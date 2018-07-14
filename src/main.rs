extern crate goblin;
extern crate libc;
extern crate nix;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate argparse;
extern crate byteorder;
extern crate flate2;
extern crate num_traits;

#[macro_use]
extern crate log;
extern crate simplelog;

mod args;
mod cpu;
mod elf;
mod memory;
mod mylog;
mod syscall_numbers;
mod syscalls;

use args::parse_arguments;
use cpu::control::CPUFlags;
use elf::load_elf;
use mylog::configure_logging;

fn main() {
    let args = parse_arguments();
    configure_logging(args.verbosity_level);

    if args.is_coredump {
        let entry_point = args.entry_point.expect(
            "Coredumps do not contain entry point. Must be specified manually.",
        );
        let stack_pointer = args.stack_pointer.expect(
            "Coredumps contain stack, but I don't know where. You need to specify it manually.",
        );
        run_coredump(args.executable, entry_point, stack_pointer, args.flags);
    } else {
        run_binary(args.executable, args.arguments, args.flags);
    }
}

/// With coredumps, all we need is to load the correct sections into memory. Than just run
/// the CPU. Small problem is, that there is no entrypoint in coredumps. So it must be provided
/// by other means.
pub fn run_coredump(path: String, entry_point: u32, stack_pointer: u32, flags: CPUFlags) {
    // initialize memory
    let (memory, _) = load_elf(path.as_str()).expect("Failed to process ELF file");

    // run
    info!("Starting CPU loop:");
    cpu::control::EmulatorContext::start(memory, stack_pointer, entry_point, flags);

    info!("Program terminated gracefully");
}


/// Loads and runs ordinary statically compiled ELF binaries.
pub fn run_binary(path: String, arguments: Vec<String>, flags: CPUFlags) {
    //initialize memory and stack
    let (mut memory, entry_point) = load_elf(path.as_str()).expect("Failed to process ELF file");

    let mut arguments = arguments;
    arguments.insert(0, path);

    let environment_vars = std::env::vars().into_iter().collect();
    let stack_pointer = 0x7ffffe50;
    memory.initialize_stack_at(stack_pointer, environment_vars, arguments);

    // run
    info!("Starting CPU loop:");
    cpu::control::EmulatorContext::start(memory, stack_pointer, entry_point, flags);

    info!("Program terminated gracefully");
}
