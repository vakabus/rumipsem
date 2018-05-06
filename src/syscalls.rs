/// List of MIPS syscall numbers can be found here:
/// https://github.com/torvalds/linux/blob/master/arch/mips/include/uapi/asm/unistd.h


use ::sc::nr::*;
use ::cpu::RegisterFile;
use ::memory::Memory;
use ::std::ffi::CString;

const AMD64_STAT64: usize = 195;

macro_rules! translate_syscall_number {
    ($sn:expr) => {
        match $sn {
            4153 => _SYSCTL,
            4168 => ACCEPT,
            4334 => ACCEPT4,
            4033 => ACCESS,
            4051 => ACCT,
            4280 => ADD_KEY,
            4124 => ADJTIMEX,
            4137 => AFS_SYSCALL,
            4027 => ALARM,
            4169 => BIND,
            4355 => BPF,
            4045 => BRK,
            4204 => CAPGET,
            4205 => CAPSET,
            4012 => CHDIR,
            4015 => CHMOD,
            4202 => CHOWN,
            4061 => CHROOT,
            4341 => CLOCK_ADJTIME,
            4264 => CLOCK_GETRES,
            4263 => CLOCK_GETTIME,
            4265 => CLOCK_NANOSLEEP,
            4262 => CLOCK_SETTIME,
            4120 => CLONE,
            4006 => CLOSE,
            4170 => CONNECT,
            4360 => COPY_FILE_RANGE,
            4008 => CREAT,
            4127 => CREATE_MODULE,
            4129 => DELETE_MODULE,
            4041 => DUP,
            4063 => DUP2,
            4327 => DUP3,
            4248 => EPOLL_CREATE,
            4326 => EPOLL_CREATE1,
            4249 => EPOLL_CTL,
            4313 => EPOLL_PWAIT,
            4250 => EPOLL_WAIT,
            4319 => EVENTFD,
            4325 => EVENTFD2,
            4011 => EXECVE,
            4356 => EXECVEAT,
            4001 => EXIT,
            4246 => EXIT_GROUP,
            4300 => FACCESSAT,
            4254 => FADVISE64,
            4320 => FALLOCATE,
            4336 => FANOTIFY_INIT,
            4337 => FANOTIFY_MARK,
            4133 => FCHDIR,
            4094 => FCHMOD,
            4299 => FCHMODAT,
            4095 => FCHOWN,
            4291 => FCHOWNAT,
            4055 => FCNTL,
            4152 => FDATASYNC,
            4229 => FGETXATTR,
            4348 => FINIT_MODULE,
            4232 => FLISTXATTR,
            4143 => FLOCK,
            4002 => FORK,
            4235 => FREMOVEXATTR,
            4226 => FSETXATTR,
            4108 => FSTAT,
            4100 => FSTATFS,
            4118 => FSYNC,
            4093 => FTRUNCATE,
            4238 => FUTEX,
            4292 => FUTIMESAT,
            4130 => GET_KERNEL_SYMS,
            4269 => GET_MEMPOLICY,
            4310 => GET_ROBUST_LIST,
            4312 => GETCPU,
            4203 => GETCWD,
            4141 => GETDENTS,
            4219 => GETDENTS64,
            4050 => GETEGID,
            4049 => GETEUID,
            4047 => GETGID,
            4080 => GETGROUPS,
            4105 => GETITIMER,
            4171 => GETPEERNAME,
            4132 => GETPGID,
            4065 => GETPGRP,
            4020 => GETPID,
            4208 => GETPMSG,
            4064 => GETPPID,
            4096 => GETPRIORITY,
            4353 => GETRANDOM,
            4191 => GETRESGID,
            4186 => GETRESUID,
            4076 => GETRLIMIT,
            4077 => GETRUSAGE,
            4151 => GETSID,
            4172 => GETSOCKNAME,
            4173 => GETSOCKOPT,
            4222 => GETTID,
            4078 => GETTIMEOFDAY,
            4024 => GETUID,
            4227 => GETXATTR,
            4128 => INIT_MODULE,
            4285 => INOTIFY_ADD_WATCH,
            4284 => INOTIFY_INIT,
            4329 => INOTIFY_INIT1,
            4286 => INOTIFY_RM_WATCH,
            4245 => IO_CANCEL,
            4242 => IO_DESTROY,
            4243 => IO_GETEVENTS,
            4241 => IO_SETUP,
            4244 => IO_SUBMIT,
            4054 => IOCTL,
            4101 => IOPERM,
            4110 => IOPL,
            4315 => IOPRIO_GET,
            4314 => IOPRIO_SET,
            4347 => KCMP,
            4311 => KEXEC_LOAD,
            4282 => KEYCTL,
            4037 => KILL,
            4016 => LCHOWN,
            4228 => LGETXATTR,
            4009 => LINK,
            4296 => LINKAT,
            4174 => LISTEN,
            4230 => LISTXATTR,
            4231 => LLISTXATTR,
            4247 => LOOKUP_DCOOKIE,
            4234 => LREMOVEXATTR,
            4019 => LSEEK,
            4225 => LSETXATTR,
            4107 => LSTAT,
            4218 => MADVISE,
            4268 => MBIND,
            4358 => MEMBARRIER,
            4354 => MEMFD_CREATE,
            4287 => MIGRATE_PAGES,
            4217 => MINCORE,
            4039 => MKDIR,
            4289 => MKDIRAT,
            4014 => MKNOD,
            4290 => MKNODAT,
            4154 => MLOCK,
            4359 => MLOCK2,
            4156 => MLOCKALL,
            4090 => MMAP,
            4123 => MODIFY_LDT,
            4021 => MOUNT,
            4308 => MOVE_PAGES,
            4125 => MPROTECT,
            4276 => MQ_GETSETATTR,
            4275 => MQ_NOTIFY,
            4271 => MQ_OPEN,
            4274 => MQ_TIMEDRECEIVE,
            4273 => MQ_TIMEDSEND,
            4272 => MQ_UNLINK,
            4167 => MREMAP,
            4144 => MSYNC,
            4155 => MUNLOCK,
            4157 => MUNLOCKALL,
            4091 => MUNMAP,
            4339 => NAME_TO_HANDLE_AT,
            4166 => NANOSLEEP,
            4189 => NFSSERVCTL,
            4005 => OPEN,
            4340 => OPEN_BY_HANDLE_AT,
            4288 => OPENAT,
            4029 => PAUSE,
            4333 => PERF_EVENT_OPEN,
            4136 => PERSONALITY,
            4042 => PIPE,
            4328 => PIPE2,
            4216 => PIVOT_ROOT,
            4364 => PKEY_ALLOC,
            4365 => PKEY_FREE,
            4363 => PKEY_MPROTECT,
            4188 => POLL,
            4302 => PPOLL,
            4192 => PRCTL,
            4200 => PREAD64,
            4330 => PREADV,
            4361 => PREADV2,
            4338 => PRLIMIT64,
            4345 => PROCESS_VM_READV,
            4346 => PROCESS_VM_WRITEV,
            4301 => PSELECT6,
            4026 => PTRACE,
            4209 => PUTPMSG,
            4201 => PWRITE64,
            4331 => PWRITEV,
            4362 => PWRITEV2,
            4187 => QUERY_MODULE,
            4131 => QUOTACTL,
            4003 => READ,
            4223 => READAHEAD,
            4085 => READLINK,
            4298 => READLINKAT,
            4145 => READV,
            4088 => REBOOT,
            4176 => RECVFROM,
            4335 => RECVMMSG,
            4177 => RECVMSG,
            4251 => REMAP_FILE_PAGES,
            4233 => REMOVEXATTR,
            4038 => RENAME,
            4295 => RENAMEAT,
            4351 => RENAMEAT2,
            4281 => REQUEST_KEY,
            4253 => RESTART_SYSCALL,
            4040 => RMDIR,
            4194 => RT_SIGACTION,
            4196 => RT_SIGPENDING,
            4195 => RT_SIGPROCMASK,
            4198 => RT_SIGQUEUEINFO,
            4193 => RT_SIGRETURN,
            4199 => RT_SIGSUSPEND,
            4197 => RT_SIGTIMEDWAIT,
            4332 => RT_TGSIGQUEUEINFO,
            4163 => SCHED_GET_PRIORITY_MAX,
            4164 => SCHED_GET_PRIORITY_MIN,
            4240 => SCHED_GETAFFINITY,
            4350 => SCHED_GETATTR,
            4159 => SCHED_GETPARAM,
            4161 => SCHED_GETSCHEDULER,
            4165 => SCHED_RR_GET_INTERVAL,
            4239 => SCHED_SETAFFINITY,
            4349 => SCHED_SETATTR,
            4158 => SCHED_SETPARAM,
            4160 => SCHED_SETSCHEDULER,
            4162 => SCHED_YIELD,
            4352 => SECCOMP,
            4207 => SENDFILE,
            4343 => SENDMMSG,
            4179 => SENDMSG,
            4180 => SENDTO,
            4270 => SET_MEMPOLICY,
            4309 => SET_ROBUST_LIST,
            4283 => SET_THREAD_AREA,
            4252 => SET_TID_ADDRESS,
            4121 => SETDOMAINNAME,
            4139 => SETFSGID,
            4138 => SETFSUID,
            4046 => SETGID,
            4081 => SETGROUPS,
            4074 => SETHOSTNAME,
            4104 => SETITIMER,
            4344 => SETNS,
            4057 => SETPGID,
            4097 => SETPRIORITY,
            4071 => SETREGID,
            4190 => SETRESGID,
            4185 => SETRESUID,
            4070 => SETREUID,
            4075 => SETRLIMIT,
            4066 => SETSID,
            4181 => SETSOCKOPT,
            4079 => SETTIMEOFDAY,
            4023 => SETUID,
            4224 => SETXATTR,
            4182 => SHUTDOWN,
            4206 => SIGALTSTACK,
            4317 => SIGNALFD,
            4324 => SIGNALFD4,
            4183 => SOCKET,
            4184 => SOCKETPAIR,
            4304 => SPLICE,
            4106 => STAT,
            4213 => AMD64_STAT64,        //FIXME works only on amd64
            4099 => STATFS,
            4366 => STATX,
            4115 => SWAPOFF,
            4087 => SWAPON,
            4083 => SYMLINK,
            4297 => SYMLINKAT,
            4036 => SYNC,
            4305 => SYNC_FILE_RANGE,
            4342 => SYNCFS,
            4135 => SYSFS,
            4116 => SYSINFO,
            4103 => SYSLOG,
            4306 => TEE,
            4266 => TGKILL,
            4013 => TIME,
            4257 => TIMER_CREATE,
            4261 => TIMER_DELETE,
            4260 => TIMER_GETOVERRUN,
            4259 => TIMER_GETTIME,
            4258 => TIMER_SETTIME,
            4321 => TIMERFD_CREATE,
            4322 => TIMERFD_GETTIME,
            4323 => TIMERFD_SETTIME,
            4043 => TIMES,
            4236 => TKILL,
            4092 => TRUNCATE,
            4060 => UMASK,
            4052 => UMOUNT2,
            4122 => UNAME,
            4010 => UNLINK,
            4294 => UNLINKAT,
            4303 => UNSHARE,
            4086 => USELIB,
            4357 => USERFAULTFD,
            4062 => USTAT,
            4030 => UTIME,
            4316 => UTIMENSAT,
            4267 => UTIMES,
            4111 => VHANGUP,
            4307 => VMSPLICE,
            4277 => VSERVER,
            4114 => WAIT4,
            4278 => WAITID,
            4004 => WRITE,
            4146 => WRITEV,
            _ => 0xFF_FF_FF_FF,
        }
    };
}

pub fn eval_syscall(inst: u32, registers: &mut RegisterFile, memory: &mut Memory) {
    macro_rules! itrace {
        ($fmt:expr, $($arg:tt)*) => (
            trace!(concat!("0x{:x}:\tsyscall\t", $fmt), registers.get_pc(), $($arg)*);
        );
        ($fmt:expr) => (
            trace!(concat!("0x{:x}:\tsyscall\t", $fmt), registers.get_pc());
        );
    }


    let syscall_number = registers.read_register(2);
    let arg1 = registers.read_register(4);
    let arg2 = registers.read_register(5);
    let arg3 = registers.read_register(6);
    let arg4 = registers.read_register(7);

    let translated_syscall_number = translate_syscall_number!(syscall_number);

    if translated_syscall_number == 0xFF_FF_FF_FF {
        error!("sysnum={} arg1={} arg2={} arg3={} arg4={}\n", syscall_number, arg1, arg2, arg3, arg4);
        error!("Unknown syscall.");
        panic!("Unknown SYSCALL");
    } else {
        let result: isize = match translated_syscall_number {
            SET_THREAD_AREA => {
                itrace!("SET_THREAD_AREA");
                0
            }
            SET_TID_ADDRESS => {
                itrace!("SET_TID_ADDRESS");
                0
            }
            GETUID => {
                itrace!("GETUID");
                /*
                unsafe {
                    syscall!(GETUID) as isize
                }
                */
                1000
            }
            AMD64_STAT64 => {
                //FIXME the struct must have fixed endianness

                let (file, res) = unsafe {
                    (
                        CString::from_raw(memory.translate_address_mut(arg1) as *mut i8),
                        syscall!(STAT, memory.translate_address(arg1), memory.translate_address(arg2)) as isize
                    )
                };
                itrace!("STAT64 file={:?} struct_at=0x{:08x} res={}", file, arg2, res);
                res
            }
            IOCTL => {
                itrace!("IOCTL a0={} a1={} a2=0x{:x}", arg1, arg2, arg3);

                let res: isize = unsafe {
                    syscall!(IOCTL, arg1, arg2, memory.translate_address(arg3)) as isize
                };
                res
            }
            DUP2 => {
                itrace!("DUP2 oldfd={} newfd={}",arg1, arg2);

                unsafe {
                    syscall!(DUP2, arg1, arg2) as isize
                }
            }
            WRITE => {
                itrace!("WRITE");

                unsafe {
                    syscall!(WRITE, arg1, memory.translate_address(arg2), arg3) as isize
                }
            }
            EXIT_GROUP => {
                itrace!("EXIT_GROUP");

                unsafe {
                    syscall!(EXIT_GROUP, arg1) as isize
                }
            }
            _ => panic!("Syscall translated, but unknown. OrigNum={} TrNum={}", syscall_number, translated_syscall_number),
        };

        // this depends on architecture ABI - this is x86_64
        if result < 0 {
            registers.write_register(7, 1); // report error
            registers.write_register(2, (-result) as u32);
        } else {
            registers.write_register(2, result as u32);
            registers.write_register(7, 0);
        }
    }
}
