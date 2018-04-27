use std::io;
use std::io::Read;
use ::memory::Memory;
use ::syscalls::eval_syscall;

pub struct RegisterFile {
    reg: [u32; 31],
    pc: u32,
}

impl RegisterFile {
    pub fn new(stack_pointer: u32) -> RegisterFile {
        let mut r = RegisterFile { reg: [0u32; 31], pc: 0u32 };
        r.write_register(29, stack_pointer);
        r
    }

    pub fn read_register(&self, id: u32) -> u32 {
        if id == 0 {
            0
        } else {
            self.reg[id as usize - 1]
        }
    }

    pub fn write_register(&mut self, id: u32, value: u32) {
        if id != 0 {
            self.reg[id as usize - 1] = value;
        }
    }

    pub fn get_pc(&self) -> u32 {
        self.pc
    }

    pub fn increment_pc(&mut self) {
        self.pc = self.pc + 4;
    }

    pub fn set_pc(&mut self, value: u32) {
        self.pc = value;
    }

    fn print_registers(&self) {
        for i in 0..32 {
            println!("reg{:02}:  0x{:08x}", i, self.read_register(i));
        }
        println!("------");
        let mut buf = [0; 1];
    }
}


fn get_opcode(instruction: u32) -> u32 {
    (instruction & 0xFC_00_00_00) >> 26
}

fn get_rs(instruction: u32) -> u32 {
    (instruction & 0x03_E0_00_00) >> 21
}

fn get_rt(instruction: u32) -> u32 {
    (instruction & 0x00_1F_00_00) >> 16
}

fn get_rd(instruction: u32) -> u32 {
    (instruction & 0x00_00_F8_00) >> 11
}

fn get_shift(instruction: u32) -> u32 {
    (instruction & 0x00_00_07_C0) >> 6
}

fn get_funct(instruction: u32) -> u32 {
    (instruction & 0x00_00_00_3F) >> 0
}

fn get_offset(instruction: u32) -> u16 {
    (instruction & 0x00_00_FF_FF) as u16
}

fn add_signed_offset(word: u32, offset: u16) -> u32 {
    ((word as i32) + ((offset as i16) as i32)) as u32
}

fn add_to_upper_bits(word: u32, immediate: u16) -> u32 {
    ((word as i32) + (((immediate as u32) << 16) as i32)) as u32
}

fn sign_extend(word: u32, length: u8) -> i32 {
    assert!(length < 32);
    ((word as i32) << (32 - length)) >> (32 - length)
    //(word | (0xFF_FF_FF_FF ^ (((word & (1 << (length - 1))) << 1) - 1))) as i32
}

fn eval_instruction(instruction: u32, registers: &mut RegisterFile, memory: &mut Memory) {
    let opcode = get_opcode(instruction);
    let funct = get_funct(instruction);
    let rs = get_rs(instruction);
    let rt = get_rt(instruction);
    let rd = get_rd(instruction);

    print!("Executing instruction 0x{:08X} at addr=0x{:08X} - ", instruction, registers.get_pc() - 8);

    match opcode {
        // ALU operation
        0b000000 => {
            print!("ALUOp ");
            match funct {
                // SLL
                0b000000 => {
                    print!("SLL");
                    let r = registers.read_register(rt) << get_shift(instruction);
                    registers.write_register(rd, r);
                }
                // ROTR
                0b000010 => {
                    print!("ROTR");
                    let r = registers.read_register(rt).rotate_right(get_shift(instruction));
                    registers.write_register(rd, r);
                }
                // ADD
                0b100000 => {
                    print!("ADD");
                    let (r, overflow) = registers.read_register(rs).overflowing_add(registers.read_register(rt));
                    if overflow {
                        panic!("Overflow occured during addition. Should TRAP. Please FIX");
                    }
                    registers.write_register(rd, r);
                }
                // ADDU
                0b100001 => {
                    print!("ADDU");
                    let (r, _) = registers.read_register(rs).overflowing_add(registers.read_register(rt));
                    registers.write_register(rd, r);
                }
                // SUBU
                0b100011 => {
                    print!("SUBU");
                    let (r, _) = registers.read_register(rs).overflowing_sub(registers.read_register(rt));
                    registers.write_register(rd, r);
                }
                // OR
                0b100101 => {
                    print!("OR - {:08x} | {:08x}", registers.read_register(rs), registers.read_register(rt));
                    let r = registers.read_register(rs) | registers.read_register(rt);
                    registers.write_register(rd, r);
                }
                // NOR
                0b100111 => {
                    print!("NOR");
                    let r = !(registers.read_register(rs) | registers.read_register(rt));
                    registers.write_register(rd, r);
                }
                // SRA
                0b000011 => {
                    assert_eq!(rs, 0);

                    print!("SRA");
                    let r = ((registers.read_register(rt) as i32) >> get_shift(instruction)) as u32;
                    registers.write_register(rd, r);
                }
                // AND
                0b100100 => {
                    print!("AND");
                    let r = registers.read_register(rs) & registers.read_register(rt);
                    registers.write_register(rd, r);
                }
                //JALR
                0b001001 => {
                    print!("JALR");
                    let pc = registers.get_pc();
                    registers.write_register(rd, pc);
                    let r = registers.read_register(rs);
                    assert_eq!(r & 0x00_00_00_03, 0);
                    registers.set_pc(r);
                }
                // JR
                0b001000 => {
                    print!("JR");
                    let t = registers.read_register(rs);
                    registers.set_pc(t);
                }
                // SLTU
                0b101011 => {
                    print!("SLTU");
                    let r = registers.read_register(rs) < registers.read_register(rt);
                    registers.write_register(rd, r as u32);
                }
                // SYSCALL
                0b001100 => {
                    print!("SYSCALL ");
                    eval_syscall(instruction, registers, memory);
                }
                _ => {
                    print!(" - ERROR!!!\n");
                    panic!("Unsupported ALU operation function code 0b{:06b}", funct)
                }
            }
        }
        // ADDIU
        0b001001 => {
            print!("ADDIU");
            let r = add_signed_offset(registers.read_register(rs), get_offset(instruction));
            registers.write_register(rt, r);
        }
        // ANDI
        0b001100 => {
            print!("ANDI");
            let r = registers.read_register(rs) & (get_offset(instruction) as u32);
            registers.write_register(rt, r);
        }
        // ORI
        0b001101 => {
            print!("ORI");
            let r = registers.read_register(rs) | (get_offset(instruction) as u32);
            registers.write_register(rt, r);
        }
        // BAL or BGEZAL
        0b000001 => {
            let mut lower = false;
            let mut equal = false;
            let mut higher = false;
            if rt == 0b10001 && rs == 0 {
                print!("BAL");
                equal = true;
            } else if rt == 0b10001 {
                // if necessary, just remove this panic
                panic!("BGEZAL was removed in release 6");
                print!("BGEZAL\n");
                higher = true;
                equal = true;
            } else if rt == 0b00000 {
                print!("BLTZ");
                lower = true;
            } else if rt == 0b000001 {
                print!("BGEZ");
                higher = true;
                equal = true;

            } else {
                panic!("Unknown weird conditional jump with rt=0b{:05b}", rt);
            }

            let val = (registers.read_register(rs) as i32);
            if (lower && val < 0) || (equal && val == 0) || (higher && val > 0) {
                let pc = registers.get_pc();
                registers.write_register(31, pc);
                registers.set_pc((pc as i32 + sign_extend((get_offset(instruction) as u32) << 2, 18) - 4) as u32);
            }
        }
        // BEQ
        0b000100 => {
            print!("BEQ ");
            let target_offset = sign_extend((get_offset(instruction) as u32) << 2, 18);
            let r = (registers.get_pc() as i32 - 4 + target_offset) as u32;
            if registers.read_register(rs) == registers.read_register(rt) {
                registers.set_pc(r);
                print!("taken")
            } else {
                print!("not taken");
            }
        }
        // BNE
        0b000101 => {
            print!("BNE ");
            let target_offset = sign_extend((get_offset(instruction) as u32) << 2, 18);
            let r = (registers.get_pc() as i32 - 4 + target_offset) as u32;
            if registers.read_register(rs) != registers.read_register(rt) {
                registers.set_pc(r);
                print!("taken")
            } else {
                print!("not taken");
            }
        }
        // J
        0b000010 => {
            print!("J");
            let pc = registers.get_pc() - 4;
            let target = (pc & 0xF0_00_00_00) | ((instruction & 0x03_FF_FF_FF) << 2);
            registers.set_pc(target);
        }
        // JAL
        0b000011 => {
            print!("J");
            let pc = registers.get_pc() - 4;
            let target = (pc & 0xF0_00_00_00) | ((instruction & 0x03_FF_FF_FF) << 2);
            registers.write_register(31, pc + 4);
            registers.set_pc(target);
        }
        // AUI
        0b001111 => {
            print!("AUI");
            let r = add_to_upper_bits(registers.read_register(rs), get_offset(instruction));
            registers.write_register(rt, r);
        }
        // LB
        0b100000 => {
            print!("LB");
            let r = sign_extend(memory.read_byte(add_signed_offset(registers.read_register(rs), get_offset(instruction))), 8);
            print!(" data={:08X}", r);
            registers.write_register(rt, r as u32);
        }
        // LBU
        0b100100 => {
            print!("LBU");
            let r = memory.read_byte(add_signed_offset(registers.read_register(rs), get_offset(instruction)));
            registers.write_register(rt, r);
        }
        // LW
        0b100011 => {
            print!("LW");
            let addr = add_signed_offset(registers.read_register(rs), get_offset(instruction));
            print!(" address={:08X}", addr);
            let r = memory.read_word(addr);
            registers.write_register(rt, r);
        }
        // SB
        0b101000 => {
            print!("SB");
            memory.write_byte(add_signed_offset(registers.read_register(rs), get_offset(instruction)), registers.read_register(rt));
        }
        // SW
        0b101011 => {
            print!("SW");
            let address = add_signed_offset(registers.read_register(rs), get_offset(instruction));
            print!(" address=0x{:08x} data=0x{:08x}", address, registers.read_register(rt));
            memory.write_word(address, registers.read_register(rt));
        }
        // SLTIU
        0b001011 => {
            print!("SLTIU");
            let a = (registers.read_register(rs) as i32) < sign_extend(get_offset(instruction) as u32, 16);
            if a {
                registers.write_register(rt, 1);
                print!(" true");
            } else {
                registers.write_register(rt, 0);
                print!(" false");
            }
        }
        _ => {
            print!("!!!ERROR!!!\n");
            panic!("Tried to execute instruction with unknown OPCODE - 0b{:06b}", opcode)
        }
    }
    println!("");
}

pub fn run_cpu(entry_point: u32, mut memory: Memory, stack_pointer: u32) {
    let mut register_file = RegisterFile::new(stack_pointer);
    register_file.set_pc(entry_point);

    let mut instruction_load = 0; // nop
    let mut instruction_exec = 0; // nop
    let mut nop_count = 0;

    loop {
        // watchdogs
        if register_file.get_pc() == 0 {
            panic!("Jumped to address 0 - probably wrong behaviour!");
        }
        if instruction_exec == 0 {
            nop_count += 1
        } else {
            nop_count = 0;
        }
        if nop_count > 3 {
            panic!("Too many NOPs in sequence. Aborting!");
        }

        // load new
        let instruction_load = memory.read_word_ignore_endianness(register_file.get_pc());

        register_file.increment_pc();

        // during instruction execution, PC points to address of executing instruction + 8

        // execute old
        eval_instruction(instruction_exec, &mut register_file, &mut memory);

        // memory.print_allocations();
        // register_file.print_registers();
        //let mut stdin = io::stdin();
        //let _ = stdin.lock().read(&mut buf);

        // prepare for next instruction
        instruction_exec = instruction_load;
    }
}





#[test]
fn test_apply_offset() {
    assert_eq!(add_signed_offset(0, 10), 10);
    assert_eq!(add_signed_offset(65535, 10), 65545);
    assert_eq!(add_signed_offset(65535, 65535), 65534);
    assert_eq!(add_signed_offset(0xFF_FF_FF_00, 0xFF), 0xFF_FF_FF_FF);
    assert_eq!(add_signed_offset(0xFF_FF_FF_FF, 0x80_00), 0xFF_FF_FF_FF - (65535 / 2) - 1);
}

#[test]
fn test_sign_extend() {
    assert_eq!(sign_extend(0xFF, 8), 0xFF_FF_FF_FF);
    assert_eq!(sign_extend(0x00_FF_FF_FF, 24), 0xFF_FF_FF_FF);
}
