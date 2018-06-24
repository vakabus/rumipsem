# Rumipsem - toy MIPS emulator written in Rust

Rumipsem aims to execute statically compiled ELF binaries for systems with MIPS32 CPUs. Architecture version is not exactly specified. However, majority of instructions are implemented for release 2, because the emulator is tested against [processor MIPS 74Kc](https://wikidevi.com/wiki/MIPS_74K) - CPU inside my OpenWrt router. Target platform is locked to Linux, because the emulator attempts to translate syscalls.

## Build

The emulator is developed in stable release channel of Rust.

To build it, run:
```bash
cargo build --release
```

Result executable will be stored in `target/release/rumipsem`.

## Usage

```
rumipsem [OPTIONS] -- ELF_BINARY [ARGS...]
```

The doubledash `--` is not necessary, it just prevents the emulator from consuming arguments for the actual emulated program. Most notable option is probably `-v` for verbosity. You can stack them as you like - `-vvvv` logs every single instruction emulated. For more options, run with `--help`.

## Testing and coredumps

The emulator can execute coredumps. To work properly, the coredump must have been created just after loading the program into memory. Before the first instruction is executed. A small problem is, that entry point and stack pointer are not stored in the coredump. It is necessary to supply them manually to the emulator through command line options.

Coredumps are valuable, because the provide a way to test, if the emulator behaves the same way real hardware does. Using GDB on any MIPS device, you can run any executable, create coredump and then store every single instruction address and register value in a custom trace file. The emulator can than check itself for correctness. After every register read, the value can be compared to the trace file. And addresses of executed instructions are checked, so that the execution path does not diverge.

Test traces, coredumps and binaries are stored inside `mips_binaries/` directory. `test.sh` script in root of this project runs them with proper options one after the other. Inside `tools/` directory, there is a script for connecting to remote GDB server and for creating the traces.