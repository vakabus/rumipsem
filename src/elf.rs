
use std::path::Path;
use memory::Endianness;
use memory::Memory;
use std::fs::File;
use goblin::error;
use std::io::Read;

pub fn load_elf_into_mem_and_get_init_pc(path: &str) -> error::Result<(Memory, u32)> {
    info!("Parsing ELF file and loading program image into memory...");
    let path = Path::new(path);
    let mut fd = File::open(path)?;
    let mut buffer = Vec::new();
    fd.read_to_end(&mut buffer)?;
    if let ::goblin::Object::Elf(elf) = ::goblin::Object::parse(&buffer)? {
        let mut memory = if elf.header.endianness().unwrap().is_little() {
            Memory::new(Endianness::LittleEndian)
        } else {
            Memory::new(Endianness::BigEndian)
        };


        for ph in elf.program_headers.into_iter() {
            debug!("\t{:?}", ph);
            debug!("\t   -> Need to copy from ELF file {} bytes from offset {} to address {}", ph.p_filesz, ph.p_offset, ph.p_vaddr);

            let data = &buffer.as_slice()[ph.p_offset as usize..(ph.p_offset + ph.p_filesz) as usize];
            memory.write_block_and_update_program_break(ph.p_vaddr as u32, data);
        }
        debug!("\tEntry point is at {:?}", elf.header.e_entry);

        debug!("Parsed ELF binary...");
        Ok((memory, elf.header.e_entry as u32))
    } else {
        panic!("File is not an ELF binary");
    }
}