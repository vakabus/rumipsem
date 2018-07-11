use argparse::{
    ArgumentParser,
    StoreOption,
    StoreTrue,
    IncrBy,
    Store,
    Collect
};

use cpu::control::CPUFlags;


pub struct Arguments {
    pub executable: String,
    pub is_coredump: bool,
    pub entry_point: Option<u32>,
    pub stack_pointer: Option<u32>,
    pub verbosity_level: u32,
    pub arguments: Vec<String>,
    pub flags: CPUFlags,
}

pub fn parse_arguments() -> Arguments {
    let mut args = Arguments{
        executable: String::new(),
        is_coredump: false,
        entry_point: None,
        verbosity_level: 0,
        stack_pointer: None,
        arguments: Vec::new(),
        flags: CPUFlags::default(),
    };

    {  // this block limits scope of borrows by ap.refer() method
        let mut ap = ArgumentParser::new();
        ap.set_description("Rust MIPS emulator capable of running ELF binaries on Linux.");
        ap.refer(&mut args.executable)
            .add_argument("ELF binary", Store,
                        "ELF binary to run.");
        ap.refer(&mut args.arguments)
            .add_argument("args", Collect,
            "Arguments for the emulated program.");
        ap.refer(&mut args.is_coredump)
            .add_option(&["-c", "--coredump"], StoreTrue,
                        "Flag signaling, that the ELF binary is coredump.");
        ap.refer(&mut args.entry_point)
            .add_option(&["-e", "--entry-point"], StoreOption,
                        "Optional. Specify entry point, which will override the one in ELF binary.");
        ap.refer(&mut args.verbosity_level)
            .add_option(&["-v", "--verbose"], IncrBy(1),
            "Verbosity level. Can be used multiple times.");
        ap.refer(&mut args.flags.tracefile)
            .add_option(&["--tracefile"], StoreOption,
            "Check flow of the emulation using this tracefile.");
        ap.refer(&mut args.stack_pointer)
            .add_option(&["-s", "--stack-pointer"], StoreOption,
                        "Optional. Specify stack pointer. This will prevent the emulator from creating its own stack. Use with coredumps.");

        ap.refer(&mut args.flags.syscalls_conf.sys_fake_root).add_option(&["--fake-root"], StoreTrue, "Pretend to be running as root.");
        ap.refer(&mut args.flags.syscalls_conf.sys_fake_root_directory).add_option(&["--fake-root-dir"], StoreTrue, "Pretend to be running in /root.");
        ap.refer(&mut args.flags.watchdog_conf.trace_full_register_values_check).add_option(&["--trace-check-all-register-values"], StoreTrue, "When tracing, check all register values after every instruction.");
        ap.refer(&mut args.flags.watchdog_conf.trace_panic_on_invalid_read).add_option(&["--trace-panic-on-different-register-value-read"], StoreTrue, "When tracing, abort when different value has been read from register.");
        ap.refer(&mut args.flags.syscalls_conf.sys_block_ioctl_on_stdio).add_option(&["--syscall-ioctl-block-on-stdio"], StoreTrue, "Prevent emulated program from interfering with stdio.");
        ap.refer(&mut args.flags.syscalls_conf.sys_ioctl_fail_always).add_option(&["--syscall-ioctl-always-fail"], StoreTrue, "Let all IOCTL syscalls fail.");

        ap.parse_args_or_exit();
    }

    if args.executable.len() == 0 {
        eprintln!("No executable specified! Can't do anything!");
        ::std::process::exit(1);
    }

    args
}