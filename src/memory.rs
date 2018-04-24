use std::env::Vars;
use std::env::Args;

const PAGE_SIZE: usize = 65536;

pub enum Endianness {
    LITTLE_ENDIAN,
    BIG_ENDIAN,
}

/// 4GB RAM in a two layer tree
pub struct Memory {
    endianness: Endianness,
    data: Vec<[u8; 65536]>,
    pages: Box<[u16; 65536]>,
}

impl Memory {
    fn memory_allocated_fully(&self) -> bool {
        self.data.len() == 65536
    }

    pub fn new(endianness: Endianness) -> Memory {
        Memory {
            endianness,
            pages: box [65535; 65536],
            data: Vec::new(),
        }
    }

    fn get_page(&mut self, address: u32) -> &mut [u8; PAGE_SIZE] {
        let index = self.pages[address as usize / PAGE_SIZE] as usize;
        if index == 65535 && !self.memory_allocated_fully() {
            self.data.push([0u8; PAGE_SIZE]);
            self.pages[address as usize / PAGE_SIZE] = self.data.len() as u16 - 1u16;
            self.data.last_mut().unwrap()
        } else {
            &mut self.data[index]
        }
    }

    pub fn read_byte(&mut self, address: u32) -> u32 {
        self.get_page(address)[address as usize % PAGE_SIZE] as u32
    }

    pub fn write_byte(&mut self, address: u32, value: u32) {
        self.get_page(address)[address as usize % PAGE_SIZE] = value as u8;
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

        {
            let mut page = self.get_page(address);
            let data_end = if data.len() >= PAGE_SIZE { PAGE_SIZE - (address as usize % PAGE_SIZE) } else { data.len() };
            let page_slice_end = if data.len() >= PAGE_SIZE { PAGE_SIZE } else { (address as usize + data.len()) % PAGE_SIZE };
            let data_slice = &data[..data_end];
            let page_slice = &mut page[address as usize % PAGE_SIZE..page_slice_end];

            page_slice.copy_from_slice(data_slice);
        }

        if PAGE_SIZE - (address as usize % PAGE_SIZE) > data.len() { return; }
        let ndata = &data[PAGE_SIZE - (address as usize % PAGE_SIZE)..data.len()];
        self.write_block((address + PAGE_SIZE as u32) - (address + PAGE_SIZE as u32) % PAGE_SIZE as u32, ndata)       // should get get optimized as tail-call
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
        self.write_word(pointer_address+4, 0);
    }
}