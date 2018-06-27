use cpu::control::CPUFlags;
use cpu::registers::get_register_name;
use cpu::registers::RegisterFile;
use flate2::read::GzDecoder;
use memory::Memory;
use serde_json;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;

pub struct Watchdog<'a> {
    instruction_number: usize,
    real_trace: Option<Vec<InstructionRecord>>,
    nop_count: usize,
    trace_gap: bool,
    cpu_flags: &'a CPUFlags,
}

impl<'a> Watchdog<'a> {
    pub fn new(tracefile: Option<String>, cpu_flags: &'a CPUFlags) -> Watchdog<'a> {
        let real_trace = if let Some(tracefile) = tracefile {
            info!("Loading trace into memory...");
            let res = Some(read_trace(tracefile));
            info!("Trace loaded...");
            res
        } else {
            None
        };

        Watchdog {
            instruction_number: 0,
            real_trace,
            nop_count: 0,
            trace_gap: false,
            cpu_flags,
        }
    }

    pub fn check_read(&self, reg: u32, val: u32) {
        if !self.cpu_flags.checked_register_reads || self.trace_gap {
            return;
        }

        if let Some(trace) = self.real_trace.as_ref() {
            let res = trace
                .get(self.instruction_number - 1)
                .expect("Trace not long enough.")
                .registers
                .get(&reg);
            if let Some(res) = res {
                if *res != val {
                    warn!(
                        "Value 0x{:x} was read from register {}. Should have been 0x{:x}",
                        val,
                        get_register_name(reg),
                        *res
                    );
                    if self.cpu_flags.panic_on_invalid_read {
                        panic!("Read wrong value from register...");
                    }
                }
            }
        }
    }

    pub fn check_write(&self, reg: u32, val: u32) {
        if !self.cpu_flags.checked_register_writes || self.trace_gap {
            return;
        }

        if let Some(trace) = self.real_trace.as_ref() {
            let res = trace
                .get(self.instruction_number)
                .and_then(|x| x.registers.get(&reg));
            if let Some(res) = res {
                if *res != val {
                    warn!("Value 0x{:x} was written into register {}. Should have been 0x{:x}. (jumps are off by one)", val, get_register_name(reg), *res);
                    /*if let Some(n) = trace.get(self.instruction_number+1).and_then(|x| x.registers.get(&reg)) {
                        warn!("Value of the register after next instruction is 0x{:x}", n);
                    }*/
                    if self.cpu_flags.panic_on_invalid_write {
                        panic!("Wrote wrong value from register...");
                    }
                }
            }
        }
    }

    pub fn run_cpu_watchdogs<T, U>(
        &mut self,
        register_file: &mut RegisterFile<T, U>,
        memory: &Memory,
    ) where
        T: Fn(u32, u32),
        U: Fn(u32, u32),
    {
        // null pointer
        if register_file.get_pc() == 0 {
            panic!("Jumped to address 0 - probably wrong behaviour!");
        }

        // too many nops
        if memory.fetch_instruction(register_file.get_pc()) == 0 {
            self.nop_count += 1
        } else {
            self.nop_count = 0;
        }
        if self.nop_count > 3 {
            panic!("Too many NOPs in sequence. Aborting!");
        }

        // trace if enabled
        if let Some(trace) = self.real_trace.as_ref() {
            let instruction_record = trace
                .get(self.instruction_number)
                .expect("Trace not long enough");

            if register_file.get_pc() == instruction_record.address {
                if self.trace_gap {
                    warn!("Trace gap is over...");
                    self.trace_gap = false;
                }
                self.instruction_number += 1;
            } else {
                if !self.trace_gap {
                    panic!("Execution diverged from real execution trace - upcoming instruction is at address 0x{:x}, but 0x{:x} was expected. One of the executed instructions must be implemented differently.", register_file.get_pc(), instruction_record.address);
                }
            }

            if self.cpu_flags.full_register_values_check && !self.trace_gap {
                for (reg, val) in &instruction_record.registers {
                    // exclude these registers
                    match *reg {
                        1 | 8 | 9 | 10 | 11 | 12 | 13 | 14 | 15 | 24 | 25 | 26 | 27 => continue,
                        //2 | 31 => continue,
                        _ => {}
                    }

                    if self.instruction_number > 3 {
                        if register_file.read_register(*reg) != *val {
                            error!(
                                "Unexpected value in register {}. Found 0x{:x} instead of 0x{:x}.",
                                get_register_name(*reg),
                                register_file.read_register(*reg),
                                *val,
                            );
                        }
                    } else {
                        if register_file.read_register(*reg) != *val {
                            warn!("Initial register values in trace are different. Overwriting register {}!!!", get_register_name(*reg));
                            register_file.write_register(*reg, *val);
                        }
                    }
                }
            }
        }
    }

    pub fn atomic_read_modify_write_began(&mut self) {
        if self.real_trace.is_some() {
            warn!("Trace checking temporarily disabled - atomic read-modify-write block. This is here to bypass GDB limitations...");
            self.trace_gap = true;
        }
    }
}

#[derive(Deserialize)]
pub struct InstructionRecord {
    pub address: u32,
    pub registers: HashMap<u32, u32>,
}

pub fn read_trace(tracefile: String) -> Vec<InstructionRecord> {
    let f = File::open(tracefile.as_str()).unwrap();
    let gz = GzDecoder::new(f).expect("Could not read GZIPed trace.");
    let file = BufReader::new(gz);
    let mut real_trace = Vec::new();
    for (_num, line) in file.lines().enumerate() {
        let line = line.unwrap();
        if let Ok(record) = serde_json::from_str(&line) {
            real_trace.push(record);
        }
    }
    real_trace
}
