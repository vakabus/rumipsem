#![feature(box_syntax)]

extern crate goblin;

#[macro_use]
extern crate sc;

mod memory;
mod cpu;

use memory::{Memory, Endianness};

use std::{env,
          path::PathBuf,
          fs::File,
          io::Read,
};
use goblin::error;


fn main() {
    let testFile = PathBuf::from("example/a.out");
    let (mut memory, entry_point) = load_elf_into_mem_and_get_init_pc("crosscompilation/busybox-mips").expect("Failed to process ELF file");

    //initialize stack
    //let arguments = std::env::args().into_iter().skip(1).collect();
    let arguments: Vec<String> = vec!["".to_owned()];
    let environment_vars = std::env::vars().into_iter().collect();

    let stack_pointer = 0x7ffffe50;
    memory.initialize_stack_at(stack_pointer, environment_vars, arguments);

    println!("Starting CPU loop:");
    cpu::run_cpu(entry_point, memory, stack_pointer);
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
