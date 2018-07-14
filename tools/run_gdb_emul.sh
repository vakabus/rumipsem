#!/bin/bash

COMMAND="./busybox-mips sh -c 'echo hello | cat'"

start_emul() {
    cd ../mips_binaries
    { sleep 12; echo "root"; sleep 1; echo "./gdbserver 0.0.0.0:8002 $COMMAND" ; echo "poweroff" ; } | arm_now start mips32 --clean --sync --redir tcp:8001::8002 > /dev/null
}

killall qemu-system-mips
( start_emul ) &
sleep 14

case "$1" in
    "trace")
        gdb-multiarch \
            -ex "target remote localhost:8001" \
            -ex "set heuristic-fence-post 0" \
            -ex "set confirm off" \
            -ex "generate-core-file" \
            -x "./gdb_trace.py"
        mv core.* coredump
        ;;
    *)
        gdb-multiarch \
            -ex "target remote localhost:8001" \
            -ex "layout regs"
        ;;
esac

# killall qemu-system-mips
