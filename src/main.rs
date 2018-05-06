#![feature(box_syntax)]

extern crate libc;
extern crate goblin;

#[macro_use]
extern crate sc;
#[macro_use]
extern crate log;
extern crate simplelog;

mod memory;
mod cpu;
mod syscalls;
mod helpers;

use memory::{Memory, Endianness};
use std::{env,
          path::PathBuf,
          fs::File,
          io::Read,
};
use goblin::error;

fn main() {
    ::simplelog::TermLogger::init(::simplelog::LevelFilter::Trace, ::simplelog::Config::default()).unwrap();

    //run_coredump("mips_binaries/core_busybox-mips_noarg/coredump", 0x4001b0, 0x7ffffe50, Some("/home/vasek/SKOLA/Programovani/mips-emulator/mips_binaries/core_busybox-mips_noarg/trace"));
    run_coredump("mips_binaries/core_wrtbinbusybox_id/coredump", 0x77f6b0d0, 0x7ffffe50, None/*Some("/home/vasek/SKOLA/Programovani/mips-emulator/mips_binaries/core_busybox-mips_noarg/trace")*/);
    //run_binary("mips_binaries/busybox-mips");
}

fn run_coredump(path: &str, entry_point: u32, stack_pointer: u32, trace_file: Option<&str>) {
    /// With coredumps, all we need is to load the correct sections into memory. Than just run
    /// the CPU. Small problem is, that there is no entrypoint in coredumps. So it must be provided
    /// by other means.

    let (mut memory, _) = load_elf_into_mem_and_get_init_pc(path).expect("Failed to process ELF file");
    println!("Starting CPU loop:");
    cpu::run_cpu(entry_point, memory, stack_pointer, cpu::CPUConfig{ tracefile: trace_file});
}

fn run_binary(path: &str) {
    let (mut memory, entry_point) = load_elf_into_mem_and_get_init_pc(path).expect("Failed to process ELF file");

    //initialize stack
    let arguments = std::env::args().into_iter().skip(1).collect();
    //let arguments: Vec<String> = vec![format!("./{}", path).to_owned()];
    let environment_vars = std::env::vars().into_iter().collect();
    let stack_pointer = 0x7ffffe50;
    //let environment_vars: Vec<(String, String)> = vec![("SSH_CLIENT".to_owned(), "10.11.8.77 59562 22".to_owned()), ("USER".to_owned(), "ROOT".to_owned()), ("SHLVL".to_owned(), "1".to_owned()), ("HOME".to_owned(), "/root".to_owned()), ("LOGNAME".to_owned(), "root".to_owned()), ("PATH".to_owned(), "/usr/sbin:/usr/bin:/sbin:/bin".to_owned()), ("SHELL".to_owned(), "/bin/ash".to_owned()), ("PWD".to_owned(), "/root".to_owned()), ("SSH_CONNECTION".to_owned(), "10.11.8.77 59562 10.11.8.65 22".to_owned())];
    memory.initialize_stack_at(stack_pointer, environment_vars, arguments);

    println!("Starting CPU loop:");
    cpu::run_cpu(entry_point, memory, stack_pointer, cpu::CPUConfig{tracefile: None});
}

use std::path::Path;

fn load_elf_into_mem_and_get_init_pc(path: &str) -> error::Result<(Memory, u32)> {
    println!("Parsing ELF file and loading program image into memory...");
    let path = Path::new(path);
    let mut fd = File::open(path)?;
    let mut buffer = Vec::new();
    fd.read_to_end(&mut buffer)?;
    if let goblin::Object::Elf(elf) = goblin::Object::parse(&buffer)? {
        let mut memory = if elf.header.endianness().unwrap().is_little() {
            Memory::new(Endianness::LITTLE_ENDIAN)
        } else {
            Memory::new(Endianness::BIG_ENDIAN)
        };


        for ph in elf.program_headers.into_iter() {
            println!("\t{:?}", ph);
            println!("\t   -> Need to copy from ELF file {} bytes from offset {} to address {}", ph.p_filesz, ph.p_offset, ph.p_vaddr);

            let data = &buffer.as_slice()[ph.p_offset as usize..(ph.p_offset + ph.p_filesz) as usize];
            memory.write_block(ph.p_vaddr as u32, data);
        }
        println!("\tEntry point is at {:?}", elf.header.e_entry);

        println!("Parsed ELF binary...");
        Ok((memory, elf.header.e_entry as u32))
    } else {
        panic!("File is not an ELF binary");
    }
}
