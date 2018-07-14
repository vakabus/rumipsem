use cpu::watchdog::Watchdog;

pub const V0: u32 = 2;
pub const A0: u32 = 4;
pub const A3: u32 = 7;
pub const STACK_POINTER: u32 = 29;
pub const RETURN_ADDRESS: u32 = 31;


pub struct RegisterFile<'a> {
    gpr: [u32; 31],
    fpr: [f32; 32],
    pc: u32,
    hi: u32,
    lo: u32,
    watchdog: Option<&'a Watchdog>,
}

impl<'a> RegisterFile<'a> {
    pub fn new(stack_pointer: u32) -> RegisterFile<'a> {
        let mut r = RegisterFile { gpr: [0u32; 31], fpr: [0f32; 32], pc: 0u32, hi: 0u32, lo: 0u32, watchdog: None};
        r.write_register(29, stack_pointer);
        r
    }

    pub fn configure_watchdog(&mut self, watchdog: &'a Watchdog) {
        self.watchdog = Some(watchdog);
    }

    pub fn read_register(&self, id: u32) -> u32 {
        let res = if id == 0 {
            0
        } else {
            self.gpr[id as usize - 1]
        };

        // runtime check
        if let Some(watchdog) = self.watchdog {
            watchdog.check_read(id, res);
        }

        res
    }

    pub fn write_register(&mut self, id: u32, value: u32) {
        // runtime check
        if let Some(watchdog) = self.watchdog {
            watchdog.check_write(id, value);
        }

        if id != 0 {
            self.gpr[id as usize - 1] = value;
        }
    }

    pub fn read_fpr(&self, id: u32) -> f32 {
        self.fpr[id as usize]
    }

    pub fn write_fpr(&mut self, id: u32, val: f32) {
        self.fpr[id as usize] = val;
    }

    pub fn get_pc(&self) -> u32 {
        self.pc
    }

    pub fn set_pc(&mut self, value: u32) {
        self.pc = value;
    }

    pub fn read_hi(&self) -> u32 {
        self.hi
    }

    pub fn read_lo(&self) -> u32 {
        self.lo
    }

    pub fn write_hi(&mut self, value: u32) {
        self.hi = value;
    }

    pub fn write_lo(&mut self, value: u32) {
        self.lo = value;
    }

    pub fn print_registers(&self) {
        println!("\nREGISTERS:");
        for i in 0..32 {
            println!("{}:\t0x{:08x}", get_register_name(i), self.read_register(i));
        }
        println!("------");
        println!();
    }
}

pub fn get_register_name(id: u32) -> &'static str {
    match id {
        0 => "zero",
        1 => "at",
        2 => "v0",
        3 => "v1",
        4 => "a0",
        5 => "a1",
        6 => "a2",
        7 => "a3",
        8 => "t0",
        9 => "t1",
        10 => "t2",
        11 => "t3",
        12 => "t4",
        13 => "t5",
        14 => "t6",
        15 => "t7",
        16 => "s0",
        17 => "s1",
        18 => "s2",
        19 => "s3",
        20 => "s4",
        21 => "s5",
        22 => "s6",
        23 => "s7",
        24 => "t8",
        25 => "t9",
        26 => "k0",
        27 => "k1",
        28 => "gp",
        29 => "sp",
        30 => "fp",
        31 => "ra",
        _ => unreachable!(),
    }
}
