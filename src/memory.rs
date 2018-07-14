use byteorder::{BigEndian, ByteOrder, LittleEndian, ReadBytesExt, NativeEndian};
use std::fs::File;
use std::io::Cursor;
use std::io::Read;

pub const MEMORY_SIZE: usize = 0xFF_FF_FF_FF + 1;

pub enum Endianness {
    LittleEndian,
    BigEndian,
}

/// This simple data structure represents the 4GB RAM of the emulated machine. But we don't want
/// to hold onto 4GB of real RAM, when we don't actually need it. The trick here is, that when
/// we create the Vector filled with zeros, Rust runtime will trust the OS to provide zeroed
/// memory. And because the Linux kernel uses copy-on-write, we can actually allocate all memory
/// we need and the request will be fullfilled lazily. So the initial allocation does not take
/// space and time.
pub struct Memory {
    endianness: Endianness,
    program_break: u32,
    data: Vec<u8>,
}

impl Memory {
    pub fn new(endianness: Endianness) -> Memory {
        Memory {
            endianness,
            data: vec![0; MEMORY_SIZE],
            program_break: 0,
        }
    }

    pub fn read_byte(&self, address: u32) -> u32 {
        self.data[address as usize] as u32
    }

    pub fn write_byte(&mut self, address: u32, value: u32) {
        self.data[address as usize] = value as u8;
    }

    pub fn read_halfword(&self, address: u32) -> u32 {
        match self.endianness {
            Endianness::LittleEndian => LittleEndian::read_u16(self.read_slice(address, 2)) as u32,
            Endianness::BigEndian => BigEndian::read_u16(self.read_slice(address, 2)) as u32,
        }
    }

    pub fn read_word(&self, address: u32) -> u32 {
        match self.endianness {
            Endianness::LittleEndian => LittleEndian::read_u32(self.read_slice(address, 4)),
            Endianness::BigEndian => BigEndian::read_u32(self.read_slice(address, 4)),
        }
    }

    pub fn read_slice(&self, address: u32, len: u32) -> &[u8] {
        &self.data[address as usize..(address + len) as usize]
    }

    pub fn fetch_instruction(&self, address: u32) -> u32 {
        self.read_word(address)
    }

    pub fn write_halfword(&mut self, address: u32, value: u32) {
        match self.endianness {
            Endianness::BigEndian => {
                self.write_byte(address + 1, value >> 0);
                self.write_byte(address + 0, value >> 8);
            }
            Endianness::LittleEndian => {
                self.write_byte(address + 0, value >> 0);
                self.write_byte(address + 1, value >> 8);
            }
        }
    }

    pub fn write_word(&mut self, address: u32, value: u32) {
        match self.endianness {
            Endianness::BigEndian => {
                self.write_byte(address + 3, value >> 0);
                self.write_byte(address + 2, value >> 8);
                self.write_byte(address + 1, value >> 16);
                self.write_byte(address + 0, value >> 24);
            }
            Endianness::LittleEndian => {
                self.write_byte(address + 0, value >> 0);
                self.write_byte(address + 1, value >> 8);
                self.write_byte(address + 2, value >> 16);
                self.write_byte(address + 3, value >> 24);
            }
        }
    }

    /// SWL instruction support
    pub fn write_word_unaligned_swl(&mut self, eff_address: u32, value: u32) {
        let vaddr = eff_address % 4;
        let addr = eff_address - vaddr;
        // Spagetti code, but understandable
        match self.endianness {
            Endianness::BigEndian => {
                match vaddr {
                    0 => {
                        self.write_byte(addr + 3, value >> 0);
                        self.write_byte(addr + 2, value >> 8);
                        self.write_byte(addr + 1, value >> 16);
                        self.write_byte(addr + 0, value >> 24);
                    }
                    1 => {
                        self.write_byte(addr + 3, value >> 8);
                        self.write_byte(addr + 2, value >> 16);
                        self.write_byte(addr + 1, value >> 24);
                    }
                    2 => {
                        self.write_byte(addr + 3, value >> 16);
                        self.write_byte(addr + 2, value >> 24);
                    }
                    3 => {
                        self.write_byte(addr + 3, value >> 24);
                    }
                    _ => unreachable!(),
                }
            }
            Endianness::LittleEndian => {
                match vaddr {
                    0 => {
                        self.write_byte(addr + 3, value >> 24);
                    }
                    1 => {
                        self.write_byte(addr + 3, value >> 16);
                        self.write_byte(addr + 2, value >> 24);
                    }
                    2 => {
                        self.write_byte(addr + 3, value >> 8);
                        self.write_byte(addr + 2, value >> 16);
                        self.write_byte(addr + 1, value >> 24);
                    }
                    3 => {
                        self.write_byte(addr + 3, value >> 0);
                        self.write_byte(addr + 2, value >> 8);
                        self.write_byte(addr + 1, value >> 16);
                        self.write_byte(addr + 0, value >> 24);
                    }
                    _ => unreachable!(),
                }
            }
        }
    }

    /// SWR instruction support
    pub fn write_word_unaligned_swr(&mut self, eff_address: u32, value: u32) {
        let vaddr = eff_address % 4;
        let address = eff_address - vaddr;
        // spagetti again, yay
        match self.endianness {
            Endianness::BigEndian => {
                match vaddr {
                    0 => {
                        self.write_byte(address + 0, value >> 0);
                    }
                    1 => {
                        self.write_byte(address + 0, value >> 8);
                        self.write_byte(address + 1, value >> 0);
                    }
                    2 => {
                        self.write_byte(address + 0, value >> 16);
                        self.write_byte(address + 1, value >> 8);
                        self.write_byte(address + 2, value >> 0);
                    }
                    3 => {
                        self.write_byte(address + 0, value >> 24);
                        self.write_byte(address + 1, value >> 16);
                        self.write_byte(address + 2, value >> 8);
                        self.write_byte(address + 3, value >> 0);
                    }
                    _ => unreachable!(),
                }
            }
            Endianness::LittleEndian => {
                match vaddr {
                    0 => {
                        self.write_byte(address + 0, value >> 24);
                        self.write_byte(address + 1, value >> 16);
                        self.write_byte(address + 2, value >> 8);
                        self.write_byte(address + 3, value >> 0);
                    }
                    1 => {
                        self.write_byte(address + 0, value >> 16);
                        self.write_byte(address + 1, value >> 8);
                        self.write_byte(address + 2, value >> 0);
                    }
                    2 => {
                        self.write_byte(address + 0, value >> 8);
                        self.write_byte(address + 1, value >> 0);
                    }
                    3 => {
                        self.write_byte(address + 0, value >> 0);
                    }
                    _ => unreachable!(),
                }
            }
        }
    }

    pub fn write_block_and_update_program_break(&mut self, address: u32, data: &[u8]) {
        if data.len() == 0 {
            return;
        }

        // FIXME Coredumps contain stack, but we want program break address to be lower than that
        // so we just expect the program break to be lower than 0x70000000
        if address + (data.len() as u32) < 0x70000000 {
            self.program_break = self.program_break.max(address + (data.len() as u32));
        }

        self.write_block(address, data);
    }

    pub fn write_block(&mut self, address: u32, data: &[u8]) {
        if data.len() == 0 {
            return;
        }

        let data_slice = &mut (self.data.as_mut_slice()[address as usize..
                                                            (address as usize + data.len())]);
        data_slice.copy_from_slice(data);
    }

    pub fn translate_address(&self, address: u32) -> *const u8 {
        if address == 0 {
            return 0 as *const u8;
        }

        self.data[address as usize..].as_ptr()
    }

    pub fn translate_address_mut(&mut self, address: u32) -> *mut u8 {
        if address == 0 {
            return 0 as *mut u8;
        }

        self.data[address as usize..].as_mut_ptr()
    }

    pub fn get_program_break(&self) -> u32 {
        self.program_break
    }

    pub fn update_program_break(&mut self, new_value: u32) {
        self.program_break = new_value;
    }

    pub fn initialize_stack_at(
        &mut self,
        address: u32,
        environment_variables: Vec<(String, String)>,
        arguments: Vec<String>,
    ) {
        assert_eq!(address % 8, 0);
        info!("Generating new stack");
        let mut pointer_address = address + 4;
        let mut data_address = pointer_address +
            (1 + arguments.len() as u32 + 1 + environment_variables.len() as u32 + 1 + 40) * 4;

        debug!("\tArguments: {}", arguments.len());
        self.write_word(address, arguments.len() as u32);
        // arguments
        for argument in arguments {
            debug!("\t\tArg: \"{}\" at 0x{:x}", argument, data_address);
            self.write_word(pointer_address, data_address);
            pointer_address += 4;

            for c in argument.bytes() {
                self.write_byte(data_address, c as u32);
                data_address += 1;
            }
            self.write_byte(data_address, 0);
            data_address += 1;
        }

        // zero after argument pointers
        self.write_word(pointer_address, 0);
        pointer_address += 4;

        // environment variables
        debug!("\tEnvironment variables:");
        for (name, value) in environment_variables {
            debug!("\t\t Env: {}=\"{}\"", name, value);
            self.write_word(pointer_address, data_address);
            pointer_address += 4;

            for c in name.bytes() {
                self.write_byte(data_address, c as u32);
                data_address += 1;
            }
            self.write_byte(data_address, 0x3D); //=
            data_address += 1;
            for c in value.bytes() {
                self.write_byte(data_address, c as u32);
                data_address += 1;
            }
            self.write_byte(data_address, 0);
            data_address += 1;
        }

        // zero after environment variables pointers
        self.write_word(pointer_address, 0);
        pointer_address += 4;


        // auxiliary vector
        debug!("\tAuxilary vector:");
        let mut auxv: Vec<u8> = Vec::new();
        File::open("/proc/self/auxv")
            .expect("Could not open auxv file in /proc FS!")
            .read_to_end(&mut auxv)
            .expect("Could not read auxv file in /proc/self");


        let mut write_vector = |key: u32, val: u32| {
            debug!("\t\tauxv key={} value=0x{:x}", key, val);
            self.write_word(pointer_address, key);
            self.write_word(pointer_address + 4, val);
            pointer_address += 8;
        };

        let mut rdr = Cursor::new(auxv);

        loop {
            let v = rdr.read_u64::<NativeEndian>().expect("auxv parsing failed");
            let val = rdr.read_u64::<NativeEndian>().expect("auxv parsing failed");

            {
                match v {
                    0 | 1 | 2 | 6 | 8 | 11 | 12 | 13 | 14 | 17 => {
                        write_vector(v as u32, val as u32);
                    }
                    _ => {}
                }
            }

            if v == 0 {
                break;
            }
        }
    }
}
