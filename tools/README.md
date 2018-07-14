# GDB Tracing

The fish script `run_gdb.fish` connects to MIPS Linux device over SSH and runs GDB. `run_gdb_emul.sh` does the same with Qemu. When argument `trace` is supplied, it creates a tracefile and a coredump. Otherwise it starts GDB in interactive mode. Configuration is inside the actual shell scripts.

## Issues encountered with GDB

* warning, that stack frame could not be found, seems impossible to disable
* atomic read-modify-write block can't be traced
* fork causes trace gap
* on the emulator, coredump must be created after first instruction. Otherwise it's missing some data

# Qemu

## Dependencies

* `arm_now` from `pip`
    * `sudo pip install arm_now`
* `qemu-arch-extra`
    * `sudo pacman -S qemu-arch-extra`

## Running things manually

* run `arm_now start mips32 --clean --sync` inside a directory, that will be copied into the emulator as `/root`
* use username `root`, there is no password
* to shutdown, use `poweroff`