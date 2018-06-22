#!/bin/bash

if test -z "$VERBOSITY"; then
    VERBOSITY="-vv"
fi

cargo run -- $VERBOSITY --coredump -e 4194736 -s 2147483216 --tracefile mips_binaries/core_busybox-mips_noarg/trace.gz mips_binaries/core_busybox-mips_noarg/coredump &&
cargo run -- $VERBOSITY --coredump -e 4194736 -s 2147483200 --tracefile mips_binaries/core_busybox-mips_whoami/trace.gz mips_binaries/core_busybox-mips_whoami/coredump &&
cargo run -- $VERBOSITY --coredump -e 4194736 -s 2147483200 --tracefile mips_binaries/core_busybox-mips_pwd/trace.gz mips_binaries/core_busybox-mips_pwd/coredump &&
cargo run -- $VERBOSITY --coredump -e 4194992 -s 2147483216 --tracefile mips_binaries/core_busybox-mips2_whoami/trace.gz mips_binaries/core_busybox-mips2_whoami/coredump &&
cargo run -- $VERBOSITY --coredump -e 4194992 -s 2147483216 --tracefile mips_binaries/core_busybox-mips2_noarg/trace.gz mips_binaries/core_busybox-mips2_noarg/coredump
