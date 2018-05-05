use std::io;
use std::io::Read;
use ::memory::Memory;
use ::syscalls::eval_syscall;
use std::collections::VecDeque;
use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;

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

    pub fn jump_to(&mut self, address: u32) {
        self.set_pc(address);
    }

    fn print_registers(&self) {
        println!("\nREGISTERS:");
        for i in 0..32 {
            println!("{}:\t0x{:08x}", ::helpers::get_register_name(i), self.read_register(i));
        }
        println!("------");
        println!();
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

    print!("0x{:x}:    ", registers.get_pc());

    match opcode {
        // ALU operation
        0b000000 => {
            //print!("ALUOp ");
            match funct {
                // SLL
                0b000000 => {
                    if instruction == 0 { print!("nop\t"); } else { print!("sll\t"); }
                    let r = registers.read_register(rt) << get_shift(instruction);
                    registers.write_register(rd, r);
                }
                // ROTR
                0b000010 => {
                    print!("rotr\t");
                    let r = registers.read_register(rt).rotate_right(get_shift(instruction));
                    registers.write_register(rd, r);
                }
                // ADD
                0b100000 => {
                    print!("add\t");
                    let (r, overflow) = registers.read_register(rs).overflowing_add(registers.read_register(rt));
                    if overflow {
                        panic!("Overflow occured during addition. Should TRAP. Please FIX");
                    }
                    registers.write_register(rd, r);
                }
                // ADDU
                0b100001 => {
                    print!("addu\t");
                    let (r, _) = registers.read_register(rs).overflowing_add(registers.read_register(rt));
                    registers.write_register(rd, r);
                }
                // SUBU
                0b100011 => {
                    print!("subu\t");
                    let (r, _) = registers.read_register(rs).overflowing_sub(registers.read_register(rt));
                    registers.write_register(rd, r);
                }
                // OR
                0b100101 => {
                    print!("or\t - {:08x} | {:08x}", registers.read_register(rs), registers.read_register(rt));
                    let r = registers.read_register(rs) | registers.read_register(rt);
                    registers.write_register(rd, r);
                }
                // NOR
                0b100111 => {
                    print!("nor\t");
                    let r = !(registers.read_register(rs) | registers.read_register(rt));
                    registers.write_register(rd, r);
                }
                // SRA
                0b000011 => {
                    assert_eq!(rs, 0);

                    print!("sra\t");
                    let r = ((registers.read_register(rt) as i32) >> get_shift(instruction)) as u32;
                    registers.write_register(rd, r);
                }
                // AND
                0b100100 => {
                    print!("and\t");
                    let r = registers.read_register(rs) & registers.read_register(rt);
                    registers.write_register(rd, r);
                }
                //JALR
                0b001001 => {
                    print!("jalr\t");
                    let pc = registers.get_pc();
                    registers.write_register(rd, pc + 8);
                    let r = registers.read_register(rs);
                    assert_eq!(r & 0x00_00_00_03, 0);
                    registers.jump_to(r);
                }
                // JR
                0b001000 => {
                    print!("jr\t");
                    let t = registers.read_register(rs);
                    registers.jump_to(t);
                }
                // SLTU
                0b101011 => {
                    print!("sltu\t");
                    let r = registers.read_register(rs) < registers.read_register(rt);
                    registers.write_register(rd, r as u32);
                }
                // SYSCALL
                0b001100 => {
                    print!("syscall\t");
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
            print!("addiu\t");
            let r = add_signed_offset(registers.read_register(rs), get_offset(instruction));
            registers.write_register(rt, r);
        }
        // ANDI
        0b001100 => {
            print!("andi\t{},{},0x{:x}", ::helpers::get_register_name(rt), ::helpers::get_register_name(rs), get_offset(instruction));
            let r = registers.read_register(rs) & (get_offset(instruction) as u32);
            registers.write_register(rt, r);
        }
        // ORI
        0b001101 => {
            print!("ori\t");
            let r = registers.read_register(rs) | (get_offset(instruction) as u32);
            registers.write_register(rt, r);
        }
        // BAL or BGEZAL
        0b000001 => {
            let mut lower = false;
            let mut equal = false;
            let mut higher = false;
            if rs == 0 && rt == 0b10001 {
                print!("bal\t");
                equal = true;
            } else if rt == 0b10001 {
                // if necessary, just remove this panic
                panic!("BGEZAL was removed in release 6");
                print!("BGEZAL\n");
                higher = true;
                equal = true;
            } else if rt == 0b00000 {
                print!("bltz\t");
                lower = true;
            } else if rt == 0b000001 {
                print!("bgez\t");
                higher = true;
                equal = true;
            } else {
                panic!("Unknown weird conditional jump with rt=0b{:05b}", rt);
            }

            let val = (registers.read_register(rs) as i32);
            if (lower && val < 0) || (equal && val == 0) || (higher && val > 0) {
                print!(" - branch taken");
                let pc = registers.get_pc();
                registers.write_register(31, pc + 8);
                registers.jump_to((pc as i32 + 4 + sign_extend((get_offset(instruction) as u32) << 2, 18)) as u32);
            } else {
                print!(" - branch NOT taken");
            }
        }
        // BEQ
        0b000100 => {
            print!("beq\t");
            let target_offset = sign_extend((get_offset(instruction) as u32) << 2, 18);
            let r = (registers.get_pc() as i32 + 4 + target_offset) as u32;
            print!("{},0x{:x} - ", ::helpers::get_register_name(rs), r);
            if registers.read_register(rs) == registers.read_register(rt) {
                registers.jump_to(r);
                print!("taken")
            } else {
                print!("not taken");
            }
        }
        // BNE
        0b000101 => {
            print!("bne\t");
            let target_offset = sign_extend((get_offset(instruction) as u32) << 2, 18);
            let r = (registers.get_pc() as i32 + 4 + target_offset) as u32;
            print!("{},0x{:x} - ", ::helpers::get_register_name(rs), r);
            if registers.read_register(rs) != registers.read_register(rt) {
                registers.jump_to(r);
                print!("taken")
            } else {
                print!("not taken");
            }
        }
        // J
        0b000010 => {
            print!("j\t");
            let pc = registers.get_pc() + 4;
            let target = (pc & 0xF0_00_00_00) | ((instruction & 0x03_FF_FF_FF) << 2);
            registers.jump_to(target);
        }
        // JAL
        0b000011 => {
            print!("jal\t");
            let pc = registers.get_pc() + 4;
            let target = (pc & 0xF0_00_00_00) | ((instruction & 0x03_FF_FF_FF) << 2);
            registers.write_register(31, pc + 8);
            registers.jump_to(target);
        }
        // AUI
        0b001111 => {
            print!("aui\t");
            let r = add_to_upper_bits(registers.read_register(rs), get_offset(instruction));
            registers.write_register(rt, r);
        }
        // LB
        0b100000 => {
            print!("lb\t");
            let addr = add_signed_offset(registers.read_register(rs), get_offset(instruction));
            let r = sign_extend(memory.read_byte(addr), 8);
            print!("{},0x{:x} - data=0x{:08x}", ::helpers::get_register_name(rt), addr, r);
            registers.write_register(rt, r as u32);
        }
        //LHU
        0b100101 => {
            print!("lhu\t");
            let r = memory.read_halfword(add_signed_offset(registers.read_register(rs), get_offset(instruction)));
            print!(" data={:08x}", r);
            registers.write_register(rt, r);
        }
        // LBU
        0b100100 => {
            print!("lbu\t");
            let r = memory.read_byte(add_signed_offset(registers.read_register(rs), get_offset(instruction)));
            registers.write_register(rt, r);
        }
        // LW
        0b100011 => {
            print!("lw\t");
            let addr = add_signed_offset(registers.read_register(rs), get_offset(instruction));
            let r = memory.read_word(addr);
            print!("{},0x{:x} - data=0x{:08x}", ::helpers::get_register_name(rt), addr, r);
            registers.write_register(rt, r);
        }
        // SB
        0b101000 => {
            print!("sb\t");
            memory.write_byte(add_signed_offset(registers.read_register(rs), get_offset(instruction)), registers.read_register(rt));
        }
        // SW
        0b101011 => {
            print!("sw\t");
            let address = add_signed_offset(registers.read_register(rs), get_offset(instruction));
            print!("{},0x{:x} - data=0x{:08x}", ::helpers::get_register_name(rt), address, registers.read_register(rt));
            memory.write_word(address, registers.read_register(rt));
        }
        // SLTIU
        0b001011 => {
            print!("sltiu\t");
            let a = (registers.read_register(rs) as i32) < sign_extend(get_offset(instruction) as u32, 16);
            if a {
                registers.write_register(rt, 1);
                print!(" true");
            } else {
                registers.write_register(rt, 0);
                print!(" false");
            }
        }
        // PCREL
        0b111011 => {
            print!("PCREL ");
            match rt {
                0b11111 => {
                    print!("ALUIPC");
                    let r = 0xFF_FF_00_00 & (((registers.get_pc() as i32) + (((get_offset(instruction) as u32) << 16) as i32)) as u32);
                    registers.write_register(rs, r);
                }
                _ => panic!("Unknown PCREL operation"),
            }
        }
        // SPECIAL3
        0b011111 => {
            print!("SPECIAL3 ");
            match funct {
                // ALIGN
                0b100000 => {
                    print!("ALIGN");
                    let shift = get_shift(instruction);
                    assert_eq!(shift & 0xFC, 0b01000);
                    let bp = shift & 0x03;
                    let r = (registers.read_register(rt) << (8 * bp)) | (registers.read_register(rs) >> (32 - 8 * bp));
                    registers.write_register(rd, r);
                }
                0b111011 => {
                    print!("RDHWR");
                    let sel = get_offset(instruction) & 0x07;
                    match rd {
                        29 => {
                            println!();
                            print!("\tWARNING - Attempt to read from COPROCESSOR. Unsupported. Faking it.");
                            registers.write_register(rt, 0x58e950);     // this value was copyied from gdb on real HW
                        }
                        _ => {
                            println!();

                            panic!("Attempt to read unknown CPU hardware register - rd={} into register rt={}", rd, rt);
                        }
                    }
                }
                _ => {
                    println!();
                    panic!("Tried to execute SPECIAL3 instruction with unknown FUNCT - 0b{:06b}", funct);
                }
            }
        }
        _ => {
            print!("!!!ERROR!!!\n");
            panic!("Tried to execute instruction with unknown OPCODE - 0b{:06b}", opcode)
        }
    }
    println!();
    //println!(" instruction=0x{:08x}", instruction);
}

pub fn run_cpu(entry_point: u32, mut memory: Memory, stack_pointer: u32) {
    let mut register_file = RegisterFile::new(stack_pointer);
    let mut program_counter: VecDeque<u32> = VecDeque::with_capacity(3);
    program_counter.push_back(entry_point);
    program_counter.push_back(entry_point + 4);

    register_file.set_pc(entry_point);

    let mut nop_count = 0;
    let mut debug_mode = false;

    let mut watchdog_status = WatchdogStatus::new();

    loop {
        let pc = program_counter.pop_front().unwrap();
        register_file.set_pc(pc);

        watchdog_status = cpu_watchdogs(watchdog_status, &register_file, &memory);

        let instruction = memory.fetch_instruction(pc);
        eval_instruction(instruction, &mut register_file, &mut memory);

        if register_file.get_pc() == pc {
            let npc = program_counter.front().unwrap() + 4;
            program_counter.push_back(npc);
        } else {
            program_counter.push_back(register_file.get_pc())
        }

        //if pc == 0x403300 {
          //  debug_mode = true;
        //}

        if debug_mode {
            register_file.print_registers();
            let mut stdin = io::stdin();
            let mut buf = [0; 1];
            let _ = stdin.lock().read(&mut buf);
        }
    }
}

struct WatchdogStatus {
    instruction_number: usize,
    real_trace: Vec<u32>,
    nop_count: usize,
}

impl WatchdogStatus {
    fn new() -> WatchdogStatus {
        let f = File::open("mips_binaries/instruction.trace").unwrap();
        let file = BufReader::new(&f);
        let mut real_trace = Vec::new();
        for (num, line) in file.lines().enumerate() {
            let line = line.unwrap();
            let addr: &str = line.split(':').next().unwrap();
            if addr.len() < 2 {
                continue;
            }
            match u32::from_str_radix(&addr[2..], 16) {
                Ok(val) => real_trace.push(val),
                Err(_) => {}
            }
        }

        WatchdogStatus {
            instruction_number: 0,
            real_trace,
            nop_count: 0,
        }
    }
}

fn cpu_watchdogs(status: WatchdogStatus, register_file: &RegisterFile, memory: &Memory) -> WatchdogStatus {
    let mut status = status;

    // null pointer
    if register_file.get_pc() == 0 {
        panic!("Jumped to address 0 - probably wrong behaviour!");
    }

    // too many nops
    if memory.fetch_instruction(register_file.get_pc()) == 0 {
        status.nop_count += 1
    } else {
        status.nop_count = 0;
    }
    if status.nop_count > 3 {
        panic!("Too many NOPs in sequence. Aborting!");
    }

    // trace
    if register_file.get_pc() == *status.real_trace.get(status.instruction_number).expect("Real trace not long enough.") {
        status.instruction_number += 1;
    } else {
        panic!("Execution diverged from real execution trace - upcoming instruction is at address 0x{:x}. One of the executed instructions must be implemented differently.", register_file.get_pc());
    }


    status
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
