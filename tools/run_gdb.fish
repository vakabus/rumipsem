#!/usr/bin/fish

set COMMAND "./busybox"
set REMOTE_IP "10.11.8.65"
set REMOTE_USER "root"

read -p 'echo "SSH password: "' -i password
echo

sshpass -p "$password" ssh -f $REMOTE_USER@$REMOTE_IP "./gdbserver $REMOTE_IP:8001 $COMMAND" &
switch "$argv[1]"
    case trace
        gdb-multiarch -ex "target remote $REMOTE_IP:8001" -ex "set heuristic-fence-post 0" -ex "set confirm off" -ex "generate-core-file" -x "./gdb_trace.py"
        mv core.* coredump
    case '*'
        gdb-multiarch -ex "target remote $REMOTE_IP:8001" -ex "layout regs"
end
