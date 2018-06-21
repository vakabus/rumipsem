extern crate libc;
extern crate goblin;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate flate2;
extern crate argparse;
extern crate num_traits;

#[macro_use]
extern crate log;
//extern crate simple_logger;
extern crate simplelog;

mod memory;
mod syscalls;
mod config;
mod cpu;
mod elf;
mod args;
mod mylog;

use elf::load_elf_into_mem_and_get_init_pc;
use args::parse_arguments;
use mylog::configure_logging;

fn main() {
    let args = parse_arguments();
    configure_logging(args.verbosity_level);

    if args.is_coredump {
        let entry_point = args.entry_point.expect("Coredumps do not contain entry point. Must be specified manually.");
        let stack_pointer = args.stack_pointer.expect("Coredumps contain stack, but I don't know where. You need to specify it manually.");
        run_coredump(args.executable, entry_point, stack_pointer, args.trace_file);
    } else {
        run_binary(args.executable, args.arguments);
    }

    //run_coredump("mips_binaries/core_busybox-mips_noarg/coredump", 0x4001b0, 0x7ffffe50, Some("mips_binaries/core_busybox-mips_noarg/trace.gz"));
    //run_coredump("mips_binaries/core_busybox-mips_pwd/coredump", 0x4001b0, 0x7ffffe40, Some("mips_binaries/core_busybox-mips_pwd/trace.gz"));
    //run_coredump("mips_binaries/core_busybox-mips_whoami/coredump", 0x4001b0, 0x7ffffe40,Some("mips_binaries/core_busybox-mips_whoami/trace.gz"));
    //run_coredump("mips_binaries/core_wrtbinbusybox_id/coredump", 0x77f6b0d0, 0x7ffffe60, Some("mips_binaries/core_wrtbinbusybox_id/trace.gz"));
    //run_binary("mips_binaries/busybox-mips");
}


/// With coredumps, all we need is to load the correct sections into memory. Than just run
/// the CPU. Small problem is, that there is no entrypoint in coredumps. So it must be provided
/// by other means.
pub fn run_coredump(path: String, entry_point: u32, stack_pointer: u32, trace_file: Option<String>) {
    let (memory, _) = load_elf_into_mem_and_get_init_pc(path.as_str()).expect("Failed to process ELF file");
    info!("Starting CPU loop:");
    cpu::control::run_cpu(memory, cpu::control::CPUConfig {
        tracefile: trace_file,
        entry_point,
        stack_pointer,
    });
    info!("Program terminated gracefully");
}

pub fn run_binary(path: String, arguments: Vec<String>) {
    let (mut memory, entry_point) = load_elf_into_mem_and_get_init_pc(path.as_str()).expect("Failed to process ELF file");

    //initialize stack
    let mut arguments = arguments;
    arguments.insert(0, path);

    let environment_vars = std::env::vars().into_iter().collect();
    let stack_pointer = 0x7ffffe50;
    memory.initialize_stack_at(stack_pointer, environment_vars, arguments);

    info!("Starting CPU loop:");
    cpu::control::run_cpu(memory, cpu::control::CPUConfig {
        tracefile: None,
        entry_point,
        stack_pointer,
    });
    info!("Program terminated gracefully");
}

#[test]
fn test_busybox_noarg_coredump() {
    run_coredump("mips_binaries/core_busybox-mips_noarg/coredump", 0x4001b0, 0x7ffffe50, Some("mips_binaries/core_busybox-mips_noarg/trace.gz"));
}
