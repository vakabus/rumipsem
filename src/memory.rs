use std::env::Vars;
use std::env::Args;

const PAGE_SIZE: usize = 65536;
const MEMORY_SIZE: usize = 0xFF_FF_FF_FF + 1;

pub enum Endianness {
    LITTLE_ENDIAN,
    BIG_ENDIAN,
}


/// This simple data structure represents the 4GB RAM of the emulated machine. But we don't want
/// to hold onto 4GB of real RAM, when we don't actually need it. The trick here is, that when
/// we create the Vector filled with zeros, Rust runtime will trust the OS to provide zeroed
/// memory. And because the Linux kernel uses copy-on-write, we can actually allocate all memory
/// we need and the request will be fullfilled lazily. So the initial allocation does not take
/// space and time.
pub struct Memory {

    endianness: Endianness,
    data: Vec<u8>,
}

impl Memory {
    fn memory_allocated_fully(&self) -> bool {
        self.data.len() == 65536
    }

    pub fn new(endianness: Endianness) -> Memory {
        Memory {
            endianness,
            data: vec![0; MEMORY_SIZE],
        }
    }


    pub fn read_byte(&mut self, address: u32) -> u32 {
        self.data[address as usize] as u32
    }

    pub fn write_byte(&mut self, address: u32, value: u32) {
        self.data[address as usize] = value as u8;
    }

    pub fn read_halfword(&mut self, address: u32) -> u32 {
        match self.endianness {
            Endianness::LITTLE_ENDIAN => {
                (self.read_byte(address + 1) as u32) << 8 | (self.read_byte(address) as u32)
            }
            Endianness::BIG_ENDIAN => {
                (self.read_byte(address) as u32) << 8 | (self.read_byte(address + 1) as u32)
            }
        }
    }

    pub fn read_word(&mut self, address: u32) -> u32 {
        match self.endianness {
            Endianness::LITTLE_ENDIAN => {
                (self.read_byte(address + 3) as u32) << 24 | (self.read_byte(address + 2) as u32) << 16 | (self.read_byte(address + 1) as u32) << 8 | (self.read_byte(address) as u32)
            }
            Endianness::BIG_ENDIAN => {
                (self.read_byte(address) as u32) << 24 | (self.read_byte(address + 1) as u32) << 16 | (self.read_byte(address + 2) as u32) << 8 | (self.read_byte(address + 3) as u32)
            }
        }
    }

    pub fn read_word_ignore_endianness(&mut self, address: u32) -> u32 {
        (self.read_byte(address) as u32) << 24 | (self.read_byte(address + 1) as u32) << 16 | (self.read_byte(address + 2) as u32) << 8 | (self.read_byte(address + 3) as u32)
    }

    pub fn write_halfword(&mut self, address: u32, value: u32) {
        match self.endianness {
            Endianness::BIG_ENDIAN => {
                self.write_byte(address + 1, value >> 0);
                self.write_byte(address + 0, value >> 8);
            }
            Endianness::LITTLE_ENDIAN => {
                self.write_byte(address + 0, value >> 0);
                self.write_byte(address + 1, value >> 8);
            }
        }
    }

    pub fn write_word(&mut self, address: u32, value: u32) {
        match self.endianness {
            Endianness::BIG_ENDIAN => {
                self.write_byte(address + 3, value >> 0);
                self.write_byte(address + 2, value >> 8);
                self.write_byte(address + 1, value >> 16);
                self.write_byte(address + 0, value >> 24);
            }
            Endianness::LITTLE_ENDIAN => {
                self.write_byte(address + 0, value >> 0);
                self.write_byte(address + 1, value >> 8);
                self.write_byte(address + 2, value >> 16);
                self.write_byte(address + 3, value >> 24);
            }
        }
    }

    pub fn write_block(&mut self, address: u32, data: &[u8]) {
        if data.len() == 0 { return; }

        let data_slice = &mut (self.data.as_mut_slice()[address as usize..(address as usize + data.len())]);
        data_slice.copy_from_slice(data);
    }

    pub fn initialize_stack_at(&mut self, address: u32, environment_variables: Vec<(String, String)>, arguments: Vec<String>) {
        assert_eq!(address % 8, 0);
        let mut pointer_address = address + 4;
        let mut data_address = pointer_address + (1 + arguments.len() as u32 + 1 + environment_variables.len() as u32 + 1 + 2) * 4;

        self.write_word(address, arguments.len() as u32);
        // arguments
        for argument in arguments {
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
        for (name, value) in environment_variables {
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

        // empty auxilary vector
        self.write_word(pointer_address, 0);
        self.write_word(pointer_address + 4, 0);
    }

    pub fn translate_address(&self, address: u32) -> *const u8{
        self.data[address as usize..].as_ptr()
    }

    pub fn translate_address_mut(&mut self, address: u32) -> *mut u8{
        self.data[address as usize..].as_mut_ptr()
    }
}