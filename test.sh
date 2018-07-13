#!/bin/bash

if test -z "$VERBOSITY"; then
    VERBOSITY="-vv"
fi

cargo run -- $VERBOSITY --coredump --fake-root --syscall-ioctl-always-fail -e 4194992 -s 2147483200 --tracefile mips_binaries/core_busybox-mips2_wc_passwd/trace.gz mips_binaries/core_busybox-mips2_wc_passwd/coredump &&
cargo run -- $VERBOSITY --coredump --fake-root --syscall-ioctl-always-fail -e 4194992 -s 2147483200 --tracefile mips_binaries/core_busybox-mips2_grep_passwd/trace.gz mips_binaries/core_busybox-mips2_grep_passwd/coredump &&
cargo run -- $VERBOSITY mips_binaries/busybox-mips-preR6 &&
cargo run -- $VERBOSITY --coredump --fake-root -e 4194736 -s 2147483216 --tracefile mips_binaries/core_busybox-mips_noarg/trace.gz mips_binaries/core_busybox-mips_noarg/coredump &&
cargo run -- $VERBOSITY --coredump --fake-root -e 4194736 -s 2147483200 --tracefile mips_binaries/core_busybox-mips_whoami/trace.gz mips_binaries/core_busybox-mips_whoami/coredump &&
cargo run -- $VERBOSITY --coredump --fake-root --fake-root-dir -e 4194736 -s 2147483200 --tracefile mips_binaries/core_busybox-mips_pwd/trace.gz mips_binaries/core_busybox-mips_pwd/coredump &&
cargo run -- $VERBOSITY --coredump --fake-root --syscall-ioctl-always-fail -e 4194992 -s 2147483216 --tracefile mips_binaries/core_busybox-mips2_whoami/trace.gz mips_binaries/core_busybox-mips2_whoami/coredump &&
cargo run -- $VERBOSITY --coredump --fake-root --syscall-ioctl-always-fail -e 4194992 -s 2147483216 --tracefile mips_binaries/core_busybox-mips2_noarg/trace.gz mips_binaries/core_busybox-mips2_noarg/coredump &&
cargo run -- $VERBOSITY --coredump --fake-root -e 4194736 -s 2147483200 --tracefile mips_binaries/core_busybox-mips_clear/trace.gz mips_binaries/core_busybox-mips_clear/coredump