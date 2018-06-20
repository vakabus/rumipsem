extern crate libc;
extern crate goblin;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate flate2;

#[macro_use]
extern crate log;
extern crate simple_logger;

mod memory;
mod syscalls;
mod config;
mod cpu;

use memory::{Memory, Endianness};
use std::{env,
          fs::File,
          io::Read,
};
use goblin::error;

#[test]
fn test_busybox_noarg_coredump() {
    run_coredump("mips_binaries/core_busybox-mips_noarg/coredump", 0x4001b0, 0x7ffffe50, Some("mips_binaries/core_busybox-mips_noarg/trace.gz"));
}


fn main() {
    ::simple_logger::init().unwrap();
    //::simplelog::TermLogger::init(::simplelog::LevelFilter::Debug, ::simplelog::Config::default()).unwrap();

    //run_coredump("mips_binaries/core_busybox-mips_noarg/coredump", 0x4001b0, 0x7ffffe50, Some("mips_binaries/core_busybox-mips_noarg/trace.gz"));
    //run_coredump("mips_binaries/core_busybox-mips_pwd/coredump", 0x4001b0, 0x7ffffe40, Some("mips_binaries/core_busybox-mips_pwd/trace.gz"));
    run_coredump("mips_binaries/core_busybox-mips_whoami/coredump", 0x4001b0, 0x7ffffe40,Some("mips_binaries/core_busybox-mips_whoami/trace.gz"));
    //run_coredump("mips_binaries/core_wrtbinbusybox_id/coredump", 0x77f6b0d0, 0x7ffffe60, Some("mips_binaries/core_wrtbinbusybox_id/trace.gz"));
    //run_binary("mips_binaries/busybox-mips");
}

pub fn run_coredump(path: &str, entry_point: u32, stack_pointer: u32, trace_file: Option<&str>) {
    /// With coredumps, all we need is to load the correct sections into memory. Than just run
    /// the CPU. Small problem is, that there is no entrypoint in coredumps. So it must be provided
    /// by other means.

    let (mut memory, _) = load_elf_into_mem_and_get_init_pc_and_brk(path).expect("Failed to process ELF file");
    println!("Starting CPU loop:");
    cpu::control::run_cpu(memory, cpu::control::CPUConfig {
        tracefile: trace_file,
        entry_point,
        stack_pointer,
    });
}

pub fn run_binary(path: &str) {
    let (mut memory, entry_point) = load_elf_into_mem_and_get_init_pc_and_brk(path).expect("Failed to process ELF file");

    //initialize stack
    let arguments = std::env::args().into_iter().skip(1).collect();
    //let arguments: Vec<String> = vec![format!("./{}", path).to_owned()];
    let environment_vars = std::env::vars().into_iter().collect();
    let stack_pointer = 0x7ffffe50;
    memory.initialize_stack_at(stack_pointer, environment_vars, arguments);

    println!("Starting CPU loop:");
    cpu::control::run_cpu(memory, cpu::control::CPUConfig {
        tracefile: None,
        entry_point,
        stack_pointer
    });
}

use std::path::Path;

fn load_elf_into_mem_and_get_init_pc_and_brk(path: &str) -> error::Result<(Memory, u32)> {
    println!("Parsing ELF file and loading program image into memory...");
    let path = Path::new(path);
    let mut fd = File::open(path)?;
    let mut buffer = Vec::new();
    fd.read_to_end(&mut buffer)?;
    if let goblin::Object::Elf(elf) = goblin::Object::parse(&buffer)? {

        let mut memory = if elf.header.endianness().unwrap().is_little() {
            Memory::new(Endianness::LittleEndian)
        } else {
            Memory::new(Endianness::BigEndian)
        };


        for ph in elf.program_headers.into_iter() {
            println!("\t{:?}", ph);
            println!("\t   -> Need to copy from ELF file {} bytes from offset {} to address {}", ph.p_filesz, ph.p_offset, ph.p_vaddr);

            let data = &buffer.as_slice()[ph.p_offset as usize..(ph.p_offset + ph.p_filesz) as usize];
            memory.write_block_and_update_program_break(ph.p_vaddr as u32, data);
        }
        println!("\tEntry point is at {:?}", elf.header.e_entry);

        println!("Parsed ELF binary...");
        Ok((memory, elf.header.e_entry as u32))
    } else {
        panic!("File is not an ELF binary");
    }
}
