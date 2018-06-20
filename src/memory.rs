const MEMORY_SIZE: usize = 0xFF_FF_FF_FF + 1;

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
            program_break: 0
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
            Endianness::LittleEndian => {
                (self.read_byte(address + 1) as u32) << 8 | (self.read_byte(address) as u32)
            }
            Endianness::BigEndian => {
                (self.read_byte(address) as u32) << 8 | (self.read_byte(address + 1) as u32)
            }
        }
    }

    pub fn read_word(&self, address: u32) -> u32 {
        match self.endianness {
            Endianness::LittleEndian => {
                (self.read_byte(address + 3) as u32) << 24 | (self.read_byte(address + 2) as u32) << 16 | (self.read_byte(address + 1) as u32) << 8 | (self.read_byte(address) as u32)
            }
            Endianness::BigEndian => {
                (self.read_byte(address) as u32) << 24 | (self.read_byte(address + 1) as u32) << 16 | (self.read_byte(address + 2) as u32) << 8 | (self.read_byte(address + 3) as u32)
            }
        }
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

    pub fn write_block_and_update_program_break(&mut self, address: u32, data: &[u8]) {
        if data.len() == 0 { return; }

        // FIXME Coredumps contain stack, but we want program break address to be lower than that
        // so we just expect the program break to be lower than 0x70000000
        if address + (data.len() as u32) < 0x70000000 {
            self.program_break = self.program_break.max(address + (data.len() as u32));
        }

        self.write_block(address, data);
    }

    pub fn write_block(&mut self, address: u32, data: &[u8]) {
        if data.len() == 0 { return; }

        let data_slice = &mut (self.data.as_mut_slice()[address as usize..(address as usize + data.len())]);
        data_slice.copy_from_slice(data);
    }

    pub fn get_slice(&self, start: usize, end: usize) -> &[u8]{
        &self.data.as_slice()[start..end]
    }

    pub fn translate_address(&self, address: u32) -> *const u8{
        self.data[address as usize..].as_ptr()
    }

    pub fn translate_address_mut(&mut self, address: u32) -> *mut u8{
        self.data[address as usize..].as_mut_ptr()
    }

    pub fn get_program_break(&self) -> u32 {
        self.program_break
    }

    pub fn update_program_break(&mut self, new_value: u32) {
        self.program_break = new_value;
    }

    pub fn initialize_stack_at(&mut self, address: u32, environment_variables: Vec<(String, String)>, arguments: Vec<String>) {
        assert_eq!(address % 8, 0);
        debug!("Generating new stack:");
        let mut pointer_address = address + 4;
        let mut data_address = pointer_address + (1 + arguments.len() as u32 + 1 + environment_variables.len() as u32 + 1 + 40) * 4;

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

        // auxilary vector
        /* Legal values for a_type (entry type).
        #define AT_NULL         0               /* End of vector */
        #define AT_IGNORE       1               /* Entry should be ignored */
        #define AT_EXECFD       2               /* File descriptor of program */
        #define AT_PHDR         3               /* Program headers for program */
        #define AT_PHENT        4               /* Size of program header entry */
        #define AT_PHNUM        5               /* Number of program headers */
        #define AT_PAGESZ       6               /* System page size */
        #define AT_BASE         7               /* Base address of interpreter */
        #define AT_FLAGS        8               /* Flags */
        #define AT_ENTRY        9               /* Entry point of program */
        #define AT_NOTELF       10              /* Program is not ELF */
        #define AT_UID          11              /* Real uid */
        #define AT_EUID         12              /* Effective uid */
        #define AT_GID          13              /* Real gid */
        #define AT_EGID         14              /* Effective gid */
        #define AT_CLKTCK       17              /* Frequency of times() */
        /* Pointer to the global system page used for system calls and other nice things.  */
        #define AT_SYSINFO      32
        #define AT_SYSINFO_EHDR 33  */


        let mut write_vector = |key: u32, val: u32| {
            self.write_word(pointer_address, key);
            self.write_word(pointer_address + 4, val);
            pointer_address += 8;
        };

        write_vector(33, 0x77_FF_F0_00);
        write_vector(16, 0);
        write_vector(6, 0x00_00_10_00);
        write_vector(17, 100);
        write_vector(3, 0x00_40_00_34);
        write_vector(4, 32);
        write_vector(5, 4);
        write_vector(7, 0);
        write_vector(8, 0);
        write_vector(9, 0x00_40_01_B0);
        write_vector(11, 0);
        write_vector(12, 0);
        write_vector(13, 0);
        write_vector(14, 0);
        write_vector(23, 0);
        write_vector(25, 0x7F_FF_FF_18);
        write_vector(31, 0x7F_FF_FF_ED);
        write_vector(0,0);

    }
}