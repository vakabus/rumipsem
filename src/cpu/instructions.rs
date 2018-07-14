//! Mainly one HUGE `eval_instruction` function.

use cpu::bitutils::*;
use cpu::event::*;
use cpu::float::FloatFmt;
use cpu::instructions_constants::*;
use cpu::registers::get_register_name;
use cpu::registers::RegisterFile;
use syscalls::System;
use memory::Memory;

pub fn eval_instruction(
    instruction: u32,
    registers: &mut RegisterFile,
    memory: &mut Memory,
    system: &mut System,
) -> CPUEvent {
    let mut result_cpu_event = CPUEvent::Nothing;

    macro_rules! itrace {
        ($fmt:expr, $($arg:tt)*) => (
            trace!(concat!("0x{:x}:\t", $fmt), registers.get_pc(), $($arg)*);
        );
        ($fmt:expr) => (
            trace!(concat!("0x{:x}:\t", $fmt), registers.get_pc());
        );
    }

    let opcode = get_opcode(instruction);
    let opcode = translate_opcode(opcode);
    let funct = get_funct(instruction);
    let rs = get_rs(instruction);
    let rt = get_rt(instruction);
    let rd = get_rd(instruction);

    //print!("0x{:x}:    ", registers.get_pc());

    match opcode {
        // ALU operation
        InstructionOpcode::SPECIAL => {
            //print!("ALUOp ");
            match funct {
                // SLL
                0b000000 => {
                    if instruction == 0 {
                        itrace!("nop\t");
                    } else {
                        assert_eq!(rs, 0);
                        itrace!(
                            "sll\t{},{},{}",
                            get_register_name(rd),
                            get_register_name(rt),
                            get_shift(instruction)
                        );
                        let r = registers.read_register(rt) << get_shift(instruction);
                        registers.write_register(rd, r);
                    }
                }
                // SLLV
                0b000100 => {
                    itrace!(
                        "sllv\t{},{},{}",
                        get_register_name(rd),
                        get_register_name(rt),
                        get_register_name(rs)
                    );
                    let r = registers.read_register(rt) << (registers.read_register(rs) & 0x1F);
                    registers.write_register(rd, r);
                }
                // SRLV
                0b000110 => {
                    itrace!(
                        "srlv\t{},{},{}",
                        get_register_name(rd),
                        get_register_name(rt),
                        get_register_name(rs)
                    );
                    let r = registers.read_register(rt) >> (registers.read_register(rs) & 0x1F);
                    registers.write_register(rd, r);
                }
                // rotation right / shift right logical
                0b000010 => {
                    match rs {
                        1 => {
                            itrace!(
                                "rotr\t{},{},{}",
                                get_register_name(rd),
                                get_register_name(rt),
                                get_shift(instruction)
                            );
                            let r = registers.read_register(rt).rotate_right(
                                get_shift(instruction),
                            );
                            registers.write_register(rd, r);
                        }
                        0 => {
                            itrace!(
                                "srl\t{},{},{}",
                                get_register_name(rd),
                                get_register_name(rt),
                                get_shift(instruction)
                            );
                            let r = registers.read_register(rt) >> get_shift(instruction);
                            registers.write_register(rd, r);
                        }
                        _ => panic!("unknown right bitshift variant"),
                    }
                }
                // ADD
                0b100000 => {
                    itrace!(
                        "add\t{},{},{}",
                        get_register_name(rd),
                        get_register_name(rs),
                        get_register_name(rt)
                    );
                    let (r, overflow) = registers.read_register(rs).overflowing_add(
                        registers.read_register(rt),
                    );
                    if overflow {
                        panic!("Overflow occured during addition. Should TRAP. Please FIX");
                    }
                    registers.write_register(rd, r);
                }
                // ADDU
                0b100001 => {
                    itrace!(
                        "addu\t{},{},{}",
                        get_register_name(rd),
                        get_register_name(rs),
                        get_register_name(rt)
                    );
                    let (r, _) = registers.read_register(rs).overflowing_add(
                        registers.read_register(rt),
                    );
                    registers.write_register(rd, r);
                }
                // SUBU
                0b100011 => {
                    let (r, _) = registers.read_register(rs).overflowing_sub(
                        registers.read_register(rt),
                    );
                    itrace!(
                        "subu\t{},{},{}",
                        get_register_name(rd),
                        get_register_name(rs),
                        get_register_name(rt)
                    );
                    registers.write_register(rd, r);
                }
                // OR
                0b100101 => {
                    let r = registers.read_register(rs) | registers.read_register(rt);
                    itrace!(
                        "or\t{},{},{} - res=0x{:08x}",
                        get_register_name(rd),
                        get_register_name(rs),
                        get_register_name(rt),
                        r
                    );
                    registers.write_register(rd, r);
                }
                // NOR
                0b100111 => {
                    let r = !(registers.read_register(rs) | registers.read_register(rt));
                    itrace!(
                        "nor\t{},{},{}",
                        get_register_name(rd),
                        get_register_name(rs),
                        get_register_name(rt)
                    );
                    registers.write_register(rd, r);
                }
                // SRA
                0b000011 => {
                    assert_eq!(rs, 0);

                    itrace!(
                        "sra\t{},{},{}",
                        get_register_name(rd),
                        get_register_name(rt),
                        get_shift(instruction)
                    );
                    let r = ((registers.read_register(rt) as i32) >> get_shift(instruction)) as u32;
                    registers.write_register(rd, r);
                }
                // SRAV
                0b000111 => {
                    assert_eq!(get_shift(instruction), 0);

                    itrace!(
                        "srav\t{},{},{}",
                        get_register_name(rd),
                        get_register_name(rt),
                        get_register_name(rs)
                    );
                    let r = ((registers.read_register(rt) as i32) >>
                                 (registers.read_register(rs) & 0x1F)) as
                        u32;
                    registers.write_register(rd, r);
                }
                // AND
                0b100100 => {
                    let r = registers.read_register(rs) & registers.read_register(rt);
                    itrace!(
                        "and\t{},{},{}",
                        get_register_name(rd),
                        get_register_name(rs),
                        get_register_name(rt)
                    );
                    registers.write_register(rd, r);
                }
                // XOR
                0b100110 => {
                    let r = registers.read_register(rs) ^ registers.read_register(rt);
                    itrace!(
                        "xor\t{},{},{}",
                        get_register_name(rd),
                        get_register_name(rs),
                        get_register_name(rt)
                    );
                    registers.write_register(rd, r);
                }
                //JALR
                0b001001 => {
                    itrace!(
                        "jalr\t{},{} - target=0x{:08x}",
                        get_register_name(rd),
                        get_register_name(rs),
                        registers.read_register(rs)
                    );
                    let pc = registers.get_pc();
                    registers.write_register(rd, pc + 8);
                    let r = registers.read_register(rs);
                    assert_eq!(r & 0x00_00_00_03, 0);
                    result_cpu_event = CPUEvent::FlowChangeDelayed(r);
                }
                // JR
                0b001000 => {
                    itrace!("jr\t{}", get_register_name(rs));
                    let t = registers.read_register(rs);
                    result_cpu_event = CPUEvent::FlowChangeDelayed(t);
                }
                // SLT
                0b101010 => {
                    itrace!(
                        "slt\t{},{},{}",
                        get_register_name(rd),
                        get_register_name(rs),
                        get_register_name(rt)
                    );
                    let r = (registers.read_register(rs) as i32) <
                        (registers.read_register(rt) as i32);
                    registers.write_register(rd, r as u32);
                }
                // SLTU
                0b101011 => {
                    itrace!("sltu\t");
                    let r = registers.read_register(rs) < registers.read_register(rt);
                    registers.write_register(rd, r as u32);
                }
                // MOVN
                0b001011 => {
                    assert_eq!(0, get_shift(instruction));
                    if registers.read_register(rt) != 0 {
                        let r = registers.read_register(rs);
                        registers.write_register(rd, r);
                    }
                    itrace!(
                        "movn\t{},{},{} - value_written={}",
                        get_register_name(rd),
                        get_register_name(rs),
                        get_register_name(rt),
                        registers.read_register(rt) != 0
                    );
                }
                // MOVZ
                0b001010 => {
                    assert_eq!(0, get_shift(instruction));
                    if registers.read_register(rt) == 0 {
                        let r = registers.read_register(rs);
                        registers.write_register(rd, r);
                    }
                    itrace!(
                        "movz\t{},{},{} - value_written={}",
                        get_register_name(rd),
                        get_register_name(rs),
                        get_register_name(rt),
                        registers.read_register(rt) != 0
                    );
                }
                // SOP30
                0b011000 => {
                    let op = get_shift(instruction);
                    match op {
                        0b00010 => {
                            let v1 = (registers.read_register(rs) as i32) as i64;
                            let v2 = (registers.read_register(rt) as i32) as i64;
                            let r = v1 * v2;
                            let r = (r as u64) as u32;
                            registers.write_register(rd, r);
                            itrace!(
                                "mul\t{},{},{}",
                                get_register_name(rd),
                                get_register_name(rs),
                                get_register_name(rt)
                            );
                        }
                        0b00011 => {
                            let v1 = (registers.read_register(rs) as i32) as i64;
                            let v2 = (registers.read_register(rt) as i32) as i64;
                            let r = v1 * v2;
                            let r = ((r as u64) >> 32) as u32;
                            registers.write_register(rd, r);
                            itrace!(
                                "muh\t{},{},{}",
                                get_register_name(rd),
                                get_register_name(rs),
                                get_register_name(rt)
                            );
                        }
                        0 => {
                            assert_eq!(rd, 0);
                            let v1 = (registers.read_register(rs) as i32) as i64;
                            let v2 = (registers.read_register(rt) as i32) as i64;
                            let r = v1 * v2;
                            let hi = ((r as u64) >> 32) as u32;
                            let lo = (r as u64) as u32;
                            registers.write_hi(hi);
                            registers.write_lo(lo);
                            itrace!("mult\t{},{}", get_register_name(rs), get_register_name(rt));
                        }
                        _ => {
                            error!("Unknown SOP30 instruction code: {:05b}", op);
                            panic!("Unknown instruction");
                        }
                    }
                }
                // SOP31
                0b011001 => {
                    let op = get_shift(instruction);
                    match op {
                        _ => {
                            error!("Unknown SOP31 instruction code: {:05b}", op);
                            panic!("Unknown instruction");
                        }
                    }
                }
                // SOP32
                0b011010 => {
                    let op = get_shift(instruction);
                    match op {
                        0b00000 => {
                            assert_eq!(rd, 0);
                            // warn!("Deprecated DIV instruction. Removed in release 6 of MIPS32");
                            itrace!("divu\t{},{}", get_register_name(rs), get_register_name(rt));
                            let l = (registers.read_register(rs) as i32) /
                                (registers.read_register(rt) as i32);
                            let h = (registers.read_register(rs) as i32) %
                                (registers.read_register(rt) as i32);
                            registers.write_lo(l as u32);
                            registers.write_hi(h as u32);
                        }
                        _ => {
                            error!("Unknown SOP32 instruction code: {:05b}", op);
                            panic!("Unknown instruction");
                        }
                    }
                }
                // SOP33
                0b011011 => {
                    let op = get_shift(instruction);
                    match op {
                        0b00000 => {
                            assert_eq!(rd, 0);
                            // warn!("Deprecated DIVU instruction. Removed in release 6 of MIPS32");
                            itrace!("divu\t{},{}", get_register_name(rs), get_register_name(rt));
                            let l = registers.read_register(rs) / registers.read_register(rt);
                            let h = registers.read_register(rs) % registers.read_register(rt);
                            registers.write_lo(l);
                            registers.write_hi(h);
                        }
                        _ => {
                            error!("Unknown SOP33 instruction code: {:05b}", op);
                            panic!("Unknown instruction");
                        }
                    }
                }
                // TEQ
                0b110100 => {
                    itrace!("teq\t{},{}", get_register_name(rs), get_register_name(rt));
                    if (registers.read_register(rs) as i32) ==
                        (registers.read_register(rt) as i32)
                    {
                        error!("TEQ instruction assert did not pass. Trap!");
                        panic!("Error");
                    }
                }
                // MFHI
                0b010000 => {
                    assert_eq!(rt, 0);
                    assert_eq!(rs, 0);
                    assert_eq!(get_shift(instruction), 0);
                    itrace!("mfhi\t{}", get_register_name(rd));
                    //warn!("Deprecated MFHI instruction. Removed in release 6 of MIPS32");
                    let r = registers.read_hi();
                    registers.write_register(rd, r);
                }
                0b010010 => {
                    assert_eq!(rt, 0);
                    assert_eq!(rs, 0);
                    assert_eq!(get_shift(instruction), 0);
                    itrace!("mflo\t{}", get_register_name(rd));
                    //warn!("Deprecated MFLO instruction. Removed in release 6 of MIPS32");
                    let r = registers.read_lo();
                    registers.write_register(rd, r);
                }
                // SYSCALL
                0b001100 => {
                    result_cpu_event = system.eval_syscall(instruction, registers, memory);
                }
                // SYNC
                0b001111 => {
                    itrace!("sync - instruction ignored");
                }
                // BREAK
                0b001101 => {
                    itrace!("break");
                    error!(
                        "Breakpoint instruction reached. No idea, how to continue. Terminating!"
                    );
                    panic!();
                }
                _ => {
                    error!("Unsupported ALU operation function code 0b{:06b}", funct);
                    panic!("Unsupported ALU operation function code 0b{:06b}", funct)
                }
            }
        }
        // ADDIU
        InstructionOpcode::ADDIU => {
            let r = add_signed_offset(registers.read_register(rs), get_offset(instruction));
            itrace!(
                "addiu\t{},{},0x{:04x} - res=0x{:x}",
                get_register_name(rt),
                get_register_name(rs),
                get_offset(instruction),
                r
            );
            registers.write_register(rt, r);
        }
        // ANDI
        InstructionOpcode::ANDI => {
            itrace!(
                "andi\t{},{},0x{:x}",
                get_register_name(rt),
                get_register_name(rs),
                get_offset(instruction)
            );
            let r = registers.read_register(rs) & (get_offset(instruction) as u32);
            registers.write_register(rt, r);
        }
        // XORI
        InstructionOpcode::XORI => {
            itrace!(
                "xori\t{},{},0x{:x}",
                get_register_name(rt),
                get_register_name(rs),
                get_offset(instruction)
            );
            let r = registers.read_register(rs) ^ (get_offset(instruction) as u32);
            registers.write_register(rt, r);
        }
        // ORI
        InstructionOpcode::ORI => {
            itrace!(
                "ori\t{},{},0x{:04x}",
                get_register_name(rt),
                get_register_name(rs),
                get_offset(instruction)
            );
            let r = registers.read_register(rs) | (get_offset(instruction) as u32);
            registers.write_register(rt, r);
        }
        // BAL or BGEZAL
        InstructionOpcode::REGIMM => {
            let mut lower = false;
            let mut equal = false;
            let mut higher = false;
            let mut link;
            let mut inst;
            if rs == 0 && rt == 0b10001 {
                inst = "bal";
                equal = true;
                link = true;
            } else if rt == 0b10001 {
                // if necessary, just remove this panic
                panic!("BGEZAL was removed in release 6");
            /*
                inst = "BGEZAL";
                higher = true;
                equal = true;
                link = true;
                */
            } else if rt == 0b00000 {
                inst = "bltz";
                lower = true;
                link = false;
            } else if rt == 0b000001 {
                inst = "bgez";
                higher = true;
                equal = true;
                link = false;
            } else {
                panic!("Unknown weird conditional jump with rt=0b{:05b}", rt);
            }

            let val = registers.read_register(rs) as i32;
            let pc = registers.get_pc();
            let address =
                (pc as i32 + 4 + sign_extend((get_offset(instruction) as u32) << 2, 18)) as u32;
            let mut jumped = false;

            if (lower && val < 0) || (equal && val == 0) || (higher && val > 0) {
                if link {
                    registers.write_register(31, pc + 8);
                }
                result_cpu_event = CPUEvent::FlowChangeDelayed(address);
                jumped = true;
            }
            itrace!(
                "{}\t{},0x{:x} - val=0x{:x} => {}",
                inst,
                get_register_name(rs),
                address,
                val,
                jumped
            );
        }
        // BEQ
        InstructionOpcode::BEQ => {
            let target_offset = sign_extend((get_offset(instruction) as u32) << 2, 18);
            let r = (registers.get_pc() as i32 + 4 + target_offset) as u32;
            itrace!(
                "beq\t{},{},0x{:x} - jumped={}",
                get_register_name(rs),
                get_register_name(rt),
                r,
                registers.read_register(rs) == registers.read_register(rt)
            );
            if registers.read_register(rs) == registers.read_register(rt) {
                result_cpu_event = CPUEvent::FlowChangeDelayed(r);
            }
        }
        // BNE
        InstructionOpcode::BNE => {
            let target_offset = sign_extend((get_offset(instruction) as u32) << 2, 18);
            let r = (registers.get_pc() as i32 + 4 + target_offset) as u32;
            itrace!(
                "bne\t{},{},0x{:x} - jumped={}",
                get_register_name(rs),
                get_register_name(rt),
                r,
                registers.read_register(rs) != registers.read_register(rt)
            );
            if registers.read_register(rs) != registers.read_register(rt) {
                result_cpu_event = CPUEvent::FlowChangeDelayed(r);
            }
        }
        // BGTZ
        InstructionOpcode::BGTZ => {
            assert_eq!(rt, 0);
            let target_offset = sign_extend((get_offset(instruction) as u32) << 2, 18);
            let target = (registers.get_pc() as i32 + 4 + target_offset) as u32;
            itrace!("bgtz\t{},0x{:x}", get_register_name(rs), target);
            if (registers.read_register(rs) as i32) > 0 {
                result_cpu_event = CPUEvent::FlowChangeDelayed(target);
            }
        }
        //BLEZ
        InstructionOpcode::BLEZ => {
            assert_eq!(rt, 0);
            let target_offset = sign_extend((get_offset(instruction) as u32) << 2, 18);
            let target = (registers.get_pc() as i32 + 4 + target_offset) as u32;
            itrace!("blez\t{},0x{:x}", get_register_name(rs), target);
            if (registers.read_register(rs) as i32) <= 0 {
                result_cpu_event = CPUEvent::FlowChangeDelayed(target);
            }
        }
        // J
        InstructionOpcode::J => {
            itrace!("j\t");
            let pc = registers.get_pc() + 4;
            let target = (pc & 0xF0_00_00_00) | ((instruction & 0x03_FF_FF_FF) << 2);
            result_cpu_event = CPUEvent::FlowChangeDelayed(target);
        }
        // JAL
        InstructionOpcode::JAL => {
            itrace!("jal\t");
            let pc = registers.get_pc();
            let target = (pc & 0xF0_00_00_00) | ((instruction & 0x03_FF_FF_FF) << 2);
            registers.write_register(31, pc + 8);
            result_cpu_event = CPUEvent::FlowChangeDelayed(target);
        }
        // AUI
        InstructionOpcode::AUI => {
            itrace!("aui\t");
            let r = add_to_upper_bits(registers.read_register(rs), get_offset(instruction));
            registers.write_register(rt, r);
        }
        // LB
        InstructionOpcode::LB => {
            let addr = add_signed_offset(registers.read_register(rs), get_offset(instruction));
            let r = sign_extend(memory.read_byte(addr), 8);
            itrace!(
                "lb\t{},0x{:x} - data=0x{:08x}",
                get_register_name(rt),
                addr,
                r
            );
            registers.write_register(rt, r as u32);
        }
        //LHU
        InstructionOpcode::LHU => {
            let r = memory.read_halfword(add_signed_offset(
                registers.read_register(rs),
                get_offset(instruction),
            ));
            itrace!("lhu\tdata={:08x}", r);
            registers.write_register(rt, r);
        }
        //LH
        InstructionOpcode::LH => {
            let r = memory.read_halfword(add_signed_offset(
                registers.read_register(rs),
                get_offset(instruction),
            ));
            itrace!("lh\tdata={:08x}", r);
            let r = sign_extend(r, 16) as u32;
            registers.write_register(rt, r);
        }
        // LBU
        InstructionOpcode::LBU => {
            let addr = add_signed_offset(registers.read_register(rs), get_offset(instruction));
            let r = memory.read_byte(addr);
            itrace!(
                "lbu\t{},0x{:x} - data=0x{:08x}",
                get_register_name(rt),
                addr,
                r
            );
            registers.write_register(rt, r);
        }
        // LW
        InstructionOpcode::LW => {
            let addr = add_signed_offset(registers.read_register(rs), get_offset(instruction));
            let r = memory.read_word(addr);
            itrace!(
                "lw\tmem[0x{:x}] -> {}, data=0x{:08x}",
                addr,
                get_register_name(rt),
                r
            );
            registers.write_register(rt, r);
        }
        // LWL
        InstructionOpcode::LWL => {
            // FIXME this is tested just on BigEndian

            let addr = add_signed_offset(registers.read_register(rs), get_offset(instruction));
            let r = memory.read_word(addr);
            let vaddr = (addr % 4) * 8;
            let mask = (0xFF_FF_FF_FF >> vaddr) << vaddr;
            itrace!(
                "lwl\t{},0x{:x} - data=0x{:08x}, mask=0x{:x} (the data will be overlayed by previous register content)",
                get_register_name(rt),
                addr,
                r,
                mask
            );
            let pv = registers.read_register(rt);
            registers.write_register(rt, (pv & !mask) | (r & mask));
        }
        // LWR
        InstructionOpcode::LWR => {
            //FIXME this is tested just on BigEndian

            let addr = add_signed_offset(registers.read_register(rs), get_offset(instruction)) - 3;
            let r = memory.read_word(addr);
            let vaddr = ((4 - (addr % 4)) % 4) * 8;
            let mask = (0xFF_FF_FF_FF << vaddr) >> vaddr;
            itrace!(
                "lwr\t{},0x{:x} - data=0x{:08x}, mask=0x{:x} (the data will be overlayed by previous register content)",
                get_register_name(rt),
                addr,
                r,
                mask
            );
            let pv = registers.read_register(rt);
            registers.write_register(rt, (pv & !mask) | (r & mask));
        }
        // LL
        InstructionOpcode::LL => {
            let addr = add_signed_offset(registers.read_register(rs), get_offset(instruction));
            let r = memory.read_word(addr);
            itrace!(
                "ll\t{},0x{:x} - data=0x{:08x} - synchronized nature of the instruction is ignored, this is preR6 version of the instruction",
                get_register_name(rt),
                addr,
                r
            );
            registers.write_register(rt, r);
            result_cpu_event = CPUEvent::AtomicLoadModifyWriteBegan;
        }
        // SB
        InstructionOpcode::SB => {
            itrace!("sb\t");
            memory.write_byte(
                add_signed_offset(registers.read_register(rs), get_offset(instruction)),
                registers.read_register(rt),
            );
        }
        // SH
        InstructionOpcode::SH => {
            let address = add_signed_offset(registers.read_register(rs), get_offset(instruction));
            itrace!(
                "sh\t{},0x{:x} - data=0x{:04x}",
                get_register_name(rt),
                address,
                registers.read_register(rt) & 0xFFFF
            );
            memory.write_halfword(address, registers.read_register(rt));
        }
        // SW
        InstructionOpcode::SW => {
            let address = add_signed_offset(registers.read_register(rs), get_offset(instruction));
            itrace!(
                "sw\t{} -> mem[0x{:x}], data=0x{:08x}",
                get_register_name(rt),
                address,
                registers.read_register(rt)
            );
            memory.write_word(address, registers.read_register(rt));
        }
        // SWL
        InstructionOpcode::SWL => {
            let address = add_signed_offset(registers.read_register(rs), get_offset(instruction));
            itrace!(
                "swl\t{},0x{:x} - data=0x{:08x} (only part of the data will be stored)",
                get_register_name(rt),
                address,
                registers.read_register(rt)
            );
            memory.write_word_unaligned_swl(address, registers.read_register(rt));
        }
        // SWR
        InstructionOpcode::SWR => {
            let address = add_signed_offset(registers.read_register(rs), get_offset(instruction));
            itrace!(
                "swr\t{},0x{:x} - data=0x{:08x} (only part of the data will be stored)",
                get_register_name(rt),
                address,
                registers.read_register(rt)
            );
            memory.write_word_unaligned_swr(address, registers.read_register(rt));
        }
        // SC
        InstructionOpcode::SC => {
            let address = add_signed_offset(registers.read_register(rs), get_offset(instruction));
            itrace!(
                "sc\t{},0x{:x} - data=0x{:08x}",
                get_register_name(rt),
                address,
                registers.read_register(rt)
            );
            memory.write_word(address, registers.read_register(rt));
            registers.write_register(rt, 1); // to indicate success
        }
        // SLTI
        InstructionOpcode::SLTI => {
            let a = registers.read_register(rs) < (get_offset(instruction) as u32);
            itrace!(
                "slti\t{},{},0x{:x} - {:08x} < {:08x} = {}",
                get_register_name(rt),
                get_register_name(rs),
                get_offset(instruction),
                registers.read_register(rs),
                get_offset(instruction),
                a
            );
            if a {
                registers.write_register(rt, 1);
            } else {
                registers.write_register(rt, 0);
            }
        }
        // SLTIU
        InstructionOpcode::SLTIU => {
            let a = registers.read_register(rs) <
                (sign_extend(get_offset(instruction) as u32, 16) as u32);
            itrace!(
                "sltiu\t{},{},0x{:x} - {:08x} < {:08x} = {}",
                get_register_name(rt),
                get_register_name(rs),
                sign_extend(get_offset(instruction) as u32, 16),
                registers.read_register(rs),
                (sign_extend(get_offset(instruction) as u32, 16) as u32),
                a
            );
            if a {
                registers.write_register(rt, 1);
            } else {
                registers.write_register(rt, 0);
            }
        }
        // PCREL
        InstructionOpcode::PCREL => {
            itrace!("PCREL ");
            match rt {
                0b11111 => {
                    print!("ALUIPC");
                    let r = 0xFF_FF_00_00 &
                        (((registers.get_pc() as i32) +
                              (((get_offset(instruction) as u32) << 16) as i32)) as
                             u32);
                    registers.write_register(rs, r);
                }
                _ => panic!("Unknown PCREL operation"),
            }
        }
        // SPECIAL2
        InstructionOpcode::SPECIAL2 => {
            match funct {
                // MUL
                0b000010 => {
                    assert_eq!(get_shift(instruction), 0);
                    let (r, _) = (registers.read_register(rs) as i32).overflowing_mul(
                        registers.read_register(rt) as
                            i32,
                    );
                    itrace!(
                        "mul\t{},{},{} - res_value={}",
                        get_register_name(rd),
                        get_register_name(rs),
                        get_register_name(rt),
                        r
                    );
                    registers.write_register(rd, r as u32);
                }
                _ => {
                    error!("Unknown SPECIAL3 funct: {:06b}", funct);
                    panic!("Error");
                }
            }
        }
        // SPECIAL3
        InstructionOpcode::SPECIAL3 => {
            match funct {
                // ALIGN
                0b100000 => {
                    itrace!("align");
                    let shift = get_shift(instruction);
                    assert_eq!(shift & 0xFC, 0b01000);
                    let bp = shift & 0x03;
                    let r = (registers.read_register(rt) << (8 * bp)) |
                        (registers.read_register(rs) >> (32 - 8 * bp));
                    registers.write_register(rd, r);
                }
                0b111011 => {
                    itrace!("rdhwr");
                    //let sel = get_offset(instruction) & 0x07;
                    match rd {
                        29 => {
                            // should throw an exception
                            warn!(
                                "Attempt to read from UserLocalRegister in coprocessor. Unsupported. Faking it with some constant value."
                            );
                            registers.write_register(rt, 0x58e950); // this value was copied from gdb on real HW
                            // UserLocal Register. This register provides read access to the coprocessor 0
                            // UserLocal register, if it is implemented.
                            // In some operating environments, the UserLocal register is a pointer to a
                            // thread-specific storage block.
                        }
                        _ => {
                            println!();

                            panic!(
                                "Attempt to read unknown CPU hardware register - rd={} into register rt={}",
                                rd,
                                rt
                            );
                        }
                    }
                }
                //EXT
                0b000000 => {
                    let pos = get_shift(instruction);
                    let size = rd + 1;

                    itrace!(
                        "ext\t{},{},pos={},size={}",
                        get_register_name(rt),
                        get_register_name(rs),
                        pos,
                        size
                    );

                    assert!(pos < 32);
                    assert!(size <= 32);
                    assert!(pos + size <= 32);
                    let r = ((((1 << (size + 1)) - 1) << pos) & registers.read_register(rs)) >> pos;
                    registers.write_register(rt, r);
                }
                _ => {
                    error!(
                        "Tried to execute SPECIAL3 instruction with unknown FUNCT - 0b{:06b}",
                        funct
                    );
                    panic!(
                        "Tried to execute SPECIAL3 instruction with unknown FUNCT - 0b{:06b}",
                        funct
                    );
                }
            }
        }
        InstructionOpcode::SWC1 => {
            let addr = add_signed_offset(registers.read_register(rs), get_offset(instruction));
            let r = registers.read_fpr(rt) as u32;
            itrace!("swc1\tfpr[{}] -> mem[0x{:x}], data=0x{:08x}", rt, addr, r);
            memory.write_word(addr, r);
        }
        InstructionOpcode::LWC1 => {
            let addr = add_signed_offset(registers.read_register(rs), get_offset(instruction));
            let r = memory.read_word(addr);
            itrace!("lwc1\tmem[0x{:x}] -> fpr[{}], data=0x{:08x}", addr, rt, r);
            registers.write_fpr(rt, r as f32);
        }
        InstructionOpcode::COP1 => {
            let fmt = rs;
            let ft = rt;
            let fs = rd;
            let fd = get_shift(instruction);
            match funct {
                0b000000 => {
                    itrace!("float ADD {} <- {} + {}", fd, fs, ft);
                    let a = FloatFmt::from_raw(fmt, fs, registers);
                    let b = FloatFmt::from_raw(fmt, ft, registers);
                    (a + b).save(fd, registers);
                    unimplemented!("needs testing");
                }
                _ => panic!("Unknown COP1 funct, 0b{:06b}", funct),
            }
        }
        _ => {
            panic!(
                "Tried to execute unimplemented instruction, OPCODE = {:?}",
                opcode
            )
        }
    };

    return result_cpu_event;
    //println!(" instruction=0x{:08x}", instruction);
}
