use cpu::event::CPUEvent;
use cpu::instructions::eval_instruction;
use cpu::registers::RegisterFile;
use cpu::watchdog::Watchdog;
use memory::Memory;
use std::cell::*;
use std::collections::VecDeque;
use std::io;
use std::io::Read;

#[derive(Debug)]
pub struct CPUConfig {
    pub tracefile: Option<String>,
    pub entry_point: u32,
    pub stack_pointer: u32,
    pub flags: CPUFlags,
}

#[derive(Debug)]
pub struct CPUFlags {
    pub fake_root: bool,
    pub fake_root_directory: bool,
    pub checked_register_reads: bool,
    pub checked_register_writes: bool,
    pub full_register_values_check: bool,
    pub panic_on_invalid_read: bool,
    pub panic_on_invalid_write: bool,
    pub block_ioctl_on_stdio: bool,
    pub ioctl_fail_always: bool,
}

impl CPUFlags {
    pub fn default() -> CPUFlags {
        CPUFlags {
            fake_root: false,
            fake_root_directory: false,
            checked_register_reads: true,
            checked_register_writes: false,
            full_register_values_check: false,
            panic_on_invalid_read: false,
            panic_on_invalid_write: false,
            block_ioctl_on_stdio: false,
            ioctl_fail_always: false,
        }
    }
}

pub fn run_cpu(mut memory: Memory, cpu_config: CPUConfig) {
    info!("Running CPU with configuration {:?}", cpu_config);

    let tracefile = cpu_config.tracefile.as_ref().map(|s| s.clone());
    let watchdog_status = RefCell::from(Watchdog::new(tracefile, &cpu_config.flags));

    let mut register_file = RegisterFile::new(
        cpu_config.stack_pointer,
        |reg: u32, val: u32| {
            watchdog_status.borrow().check_read(reg, val);
        },
        |reg: u32, val: u32| {
            watchdog_status.borrow().check_write(reg, val);
        });

    let mut program_counter: VecDeque<u32> = VecDeque::with_capacity(3);
    program_counter.push_back(cpu_config.entry_point);
    program_counter.push_back(cpu_config.entry_point + 4);

    register_file.set_pc(cpu_config.entry_point);

    let mut debug_mode = false;

    loop {
        let pc = program_counter.pop_front().unwrap();
        register_file.set_pc(pc);

        watchdog_status
            .borrow_mut()
            .run_cpu_watchdogs(&mut register_file, &memory);

        let instruction = memory.fetch_instruction(pc);
        let exit = eval_instruction(instruction, &mut register_file, &mut memory, &cpu_config.flags);
        match exit {
            CPUEvent::Exit => break,
            CPUEvent::AtomicLoadModifyWriteBegan => watchdog_status
                .borrow_mut()
                .atomic_read_modify_write_began(),
            _ => {}
        }

        if register_file.get_pc() == pc {
            let npc = program_counter.front().unwrap() + 4;
            program_counter.push_back(npc);
        } else {
            program_counter.push_back(register_file.get_pc())
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
