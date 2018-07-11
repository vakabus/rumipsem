use cpu::registers::RegisterFile;
use std::ops::Add;

#[derive(Debug)]
pub enum FloatFmt {
    S(f32),
    D(f64),
    W(f32),
    L(f64),
    PS(f32, f32),
}

impl FloatFmt {
    pub fn from_raw(fmt: u32, id: u32, registers: &RegisterFile) -> FloatFmt {
        //FIXME no idea what to do with these format strings, but they appear in binaries. So this is guesswork.
        let fmt = match fmt {
            0x2 => 0x14,
            _ => fmt,
        };

        match fmt {
            0x10 => FloatFmt::S(registers.read_fpr(id)),
            0x11 => FloatFmt::D(
                ((((registers.read_fpr(id + 1) as u32) as u64) << 32)
                    | (registers.read_fpr(id) as u32) as u64) as f64,
            ),
            0x14 => FloatFmt::W(registers.read_fpr(id)),
            0x15 => FloatFmt::L(
                ((((registers.read_fpr(id + 1) as u32) as u64) << 32)
                    | (registers.read_fpr(id) as u32) as u64) as f64,
            ),
            0x16 => FloatFmt::PS(registers.read_fpr(id), registers.read_fpr(id + 1)),
            _ => panic!("Unknown float format 0x{:x}", fmt),
        }
    }

    pub fn save(self, id: u32, registers: &mut RegisterFile) {
        match self {
            FloatFmt::S(a) => registers.write_fpr(id, a),
            FloatFmt::W(a) => registers.write_fpr(id, a),
            FloatFmt::D(a) => {
                registers.write_fpr(id + 1, (((a as u64) >> 32) as u32) as f32);
                registers.write_fpr(id, ((a as u64) as u32) as f32);
            }
            FloatFmt::L(a) => {
                registers.write_fpr(id + 1, (((a as u64) >> 32) as u32) as f32);
                registers.write_fpr(id, ((a as u64) as u32) as f32);
            }
            FloatFmt::PS(a, b) => {
                registers.write_fpr(id, a);
                registers.write_fpr(id + 1, b);
            }
        }
    }
}

impl Add for FloatFmt {
    type Output = FloatFmt;

    fn add(self, other: FloatFmt) -> FloatFmt {
        match (self, other) {
            (FloatFmt::S(a), FloatFmt::S(b)) => FloatFmt::S(a + b),
            (FloatFmt::D(a), FloatFmt::D(b)) => FloatFmt::D(a + b),
            (FloatFmt::W(a), FloatFmt::W(b)) => FloatFmt::W(a + b),
            (FloatFmt::L(a), FloatFmt::L(b)) => FloatFmt::L(a + b),
            (FloatFmt::PS(a, b), FloatFmt::PS(c, d)) => FloatFmt::PS(a + c, b + d),
            _ => panic!("Incompatible float types addition..."),
        }
    }
}
