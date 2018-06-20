use memory::Memory;
use cpu::registers::RegisterFile;
use std::collections::VecDeque;
use cpu::instructions::eval_instruction;
use cpu::watchdog::Watchdog;
use std::io;
use std::cell::*;
use std::io::Read;
use cpu::event::CPUEvent;

#[derive(Debug)]
pub struct CPUConfig<'a> {
    pub tracefile: Option<&'a str>,
    pub entry_point: u32,
    pub stack_pointer: u32,
}

pub fn run_cpu(mut memory: Memory, cpu_config: CPUConfig) {
    info!("Running CPU with configuration: {:?}", cpu_config);

    let watchdog_status = RefCell::from(Watchdog::new(cpu_config.tracefile));


    let mut register_file = RegisterFile::new(cpu_config.stack_pointer, |reg: u32, val: u32| {
        watchdog_status.borrow().check_read(reg, val);
    });


    let mut program_counter: VecDeque<u32> = VecDeque::with_capacity(3);
    program_counter.push_back(cpu_config.entry_point);
    program_counter.push_back(cpu_config.entry_point + 4);

    register_file.set_pc(cpu_config.entry_point);

    let mut debug_mode = false;

    loop {
        let pc = program_counter.pop_front().unwrap();
        register_file.set_pc(pc);

        watchdog_status.borrow_mut().run_cpu_watchdogs(&mut register_file, &memory);

        let instruction = memory.fetch_instruction(pc);
        let exit = eval_instruction(instruction, &mut register_file, &mut memory);
        match exit {
            CPUEvent::Exit => break,
            CPUEvent::AtomicLoadModifyWriteBegan =>
                watchdog_status.borrow_mut().atomic_read_modify_write_began(),
            _ => {}
        }

        if register_file.get_pc() == pc {
            let npc = program_counter.front().unwrap() + 4;
            program_counter.push_back(npc);
        } else {
            program_counter.push_back(register_file.get_pc())
        }

        /*if pc == 0x77fdc6ec {
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