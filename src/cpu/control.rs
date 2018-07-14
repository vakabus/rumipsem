use cpu::event::CPUEvent;
use cpu::instructions::eval_instruction;
use cpu::registers::RegisterFile;
use cpu::watchdog::Watchdog;
use memory::Memory;
use std::collections::VecDeque;
use std::io;
use std::io::Read;
use syscalls::System;

#[derive(Debug)]
pub struct CPUFlags {
    pub tracefile: Option<String>,
    pub syscalls_conf: CPUFlagsSyscalls,
    pub watchdog_conf: CPUFlagsWatchdog,
}

#[derive(Debug)]
pub struct CPUFlagsWatchdog {
    pub trace_checked_register_reads: bool,
    pub trace_checked_register_writes: bool,
    pub trace_full_register_values_check: bool,
    pub trace_panic_on_invalid_read: bool,
    pub trace_panic_on_invalid_write: bool,
}

#[derive(Debug)]
pub struct CPUFlagsSyscalls {
    pub sys_fake_root: bool,
    pub sys_fake_root_directory: bool,
    pub sys_block_ioctl_on_stdio: bool,
    pub sys_ioctl_fail_always: bool,
}

impl CPUFlags {
    pub fn default() -> CPUFlags {
        CPUFlags {
            tracefile: None,
            syscalls_conf: CPUFlagsSyscalls {
                sys_fake_root: false,
                sys_fake_root_directory: false,
                sys_block_ioctl_on_stdio: false,
                sys_ioctl_fail_always: false,
            },
            watchdog_conf: CPUFlagsWatchdog {
                trace_checked_register_reads: true,
                trace_checked_register_writes: true,
                trace_full_register_values_check: false,
                trace_panic_on_invalid_read: false,
                trace_panic_on_invalid_write: false,
            },
        }
    }
}

pub struct EmulatorContext {
    memory: Memory,
    system: System,
    watchdog: Watchdog,
    registers: RegisterFile<'static>,
}

static mut EMULATOR_STATE: Option<EmulatorContext> = None;

impl EmulatorContext {
    pub fn main_loop(memory: Memory, stack_pointer: u32, entry_point: u32, flags: CPUFlags) {
        let watchdog = Watchdog::new(flags.tracefile, flags.watchdog_conf);
        let registers = RegisterFile::new(stack_pointer);
        let system = System::new(flags.syscalls_conf);

        let state = EmulatorContext {
            memory,
            system,
            watchdog,
            registers,
        };

        let state = unsafe {
            // Creating self referencing struct. It bypasses the borrow
            // checker by using reference obtained from the singleton itself using unsafe block.
            // It should not cause any memory corruption, because read-only reference is used
            // and the Watchdog will not be discarded until the end of whole program...
            state
                .init_singleton()
                .registers
                .configure_watchdog(&EmulatorContext::get_ref().watchdog);
            EmulatorContext::get_mut_ref()
        };
        state.run_cpu(entry_point);
    }

    unsafe fn init_singleton(self) -> &'static mut EmulatorContext {
        EMULATOR_STATE.get_or_insert(self)
    }

    pub unsafe fn get_mut_ref() -> &'static mut EmulatorContext {
        EMULATOR_STATE
            .as_mut()
            .expect("Emulator singleton not initialized!")
    }

    pub unsafe fn get_ref() -> &'static EmulatorContext {
        EMULATOR_STATE
            .as_ref()
            .expect("Emulator singleton not initialized!")
    }

    pub fn get_system(&self) -> &System {
        &self.system
    }

    pub fn run_cpu(&mut self, entry_point: u32) {
        let memory = &mut self.memory;
        let system = &mut self.system;
        let register_file = &mut self.registers;
        let watchdog = &mut self.watchdog;

        //info!("Running CPU with configuration {:?}", cpu_config);

        // initialize flow control
        let mut program_counter: VecDeque<u32> = VecDeque::with_capacity(3);
        program_counter.push_back(entry_point);
        register_file.set_pc(entry_point);

        let mut debug_mode = false;

        // work loop
        loop {
            let pc = program_counter.pop_front().unwrap();
            register_file.set_pc(pc);

            watchdog.run_cpu_watchdogs(register_file, memory, true);

            let instruction = memory.fetch_instruction(pc);
            let instruction_result = eval_instruction(instruction, register_file, memory, system);

            // instruction result handling
            match instruction_result {
                CPUEvent::Nothing => {
                    if program_counter.len() == 0 {
                        program_counter.push_back(pc + 4);
                    }
                }
                CPUEvent::Exit => break,
                CPUEvent::AtomicLoadModifyWriteBegan => {
                    watchdog.atomic_read_modify_write_began();

                    if program_counter.len() == 0 {
                        program_counter.push_back(pc + 4);
                    }
                }
                CPUEvent::FlowChangeImmediate(npc) => {
                    if program_counter.len() == 0 {
                        program_counter.push_back(npc);
                    } else {
                        panic!("Flow control failed. Multiple jumps at once.");
                    }
                }
                CPUEvent::FlowChangeDelayed(npc) => {
                    if program_counter.len() == 0 {
                        program_counter.push_back(pc + 4);
                        program_counter.push_back(npc);
                    } else {
                        panic!("Flow control failed. Multiple jumps at once.");
                    }
                }
            }

            // compile time debugging breakpoint ;)
            /*if pc == 0x53391c {
                debug_mode = true;
            }*/

            if debug_mode {
                register_file.print_registers();
                let stdin = io::stdin();
                let mut buf = [0; 16];
                let _ = stdin.lock().read(&mut buf);
                if buf[0] != ('\n' as u8) {
                    print!("Continuing...");
                    debug_mode = false;
                }
            }
        }
    }

    pub fn run_function(&mut self, func: u32, arguments: &[u32]) {
        let memory = &mut self.memory;
        let system = &mut self.system;
        let mut register_file = RegisterFile::new(
            self.registers
                .read_register(::cpu::registers::STACK_POINTER) - 16
                - arguments.len() as u32,
        ); //just for sure
        let watchdog = &mut self.watchdog;

        // initialize stack
        let sp = register_file.read_register(::cpu::registers::STACK_POINTER);
        for (i, a) in arguments.iter().enumerate() {
            let i = i as u32;
            memory.write_word(sp + i * 4, *a);
            if i < 4 {
                register_file.write_register(::cpu::registers::A0 + i, *a);
            }
        }

        // initialize flow control
        let mut program_counter: VecDeque<u32> = VecDeque::with_capacity(3);
        program_counter.push_back(func);
        register_file.write_register(::cpu::registers::RETURN_ADDRESS, 4);

        // work loop
        loop {
            let pc = program_counter.pop_front().unwrap();
            // return from function
            if pc == 4 {
                return;
            }
            register_file.set_pc(pc);

            watchdog.run_cpu_watchdogs(&mut register_file, memory, false);

            let instruction = memory.fetch_instruction(pc);
            let instruction_result =
                eval_instruction(instruction, &mut register_file, memory, system);

            // instruction result handling
            match instruction_result {
                CPUEvent::Nothing => {
                    if program_counter.len() == 0 {
                        program_counter.push_back(pc + 4);
                    }
                }
                CPUEvent::Exit => break,
                CPUEvent::AtomicLoadModifyWriteBegan => {
                    watchdog.atomic_read_modify_write_began();

                    if program_counter.len() == 0 {
                        program_counter.push_back(pc + 4);
                    }
                }
                CPUEvent::FlowChangeImmediate(npc) => {
                    if program_counter.len() == 0 {
                        program_counter.push_back(npc);
                    } else {
                        panic!("Flow control failed. Multiple jumps at once.");
                    }
                }
                CPUEvent::FlowChangeDelayed(npc) => {
                    if program_counter.len() == 0 {
                        program_counter.push_back(pc + 4);
                        program_counter.push_back(npc);
                    } else {
                        panic!("Flow control failed. Multiple jumps at once.");
                    }
                }
            }
        }
    }
}
