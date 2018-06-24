use cpu::event::CPUEvent;
use cpu::registers::A3;
use cpu::registers::RegisterFile;
use cpu::registers::V0;
use cpu::registers::STACK_POINTER;
use cpu::control::CPUFlags;
/// List of MIPS syscall numbers can be found here:
/// https://github.com/torvalds/linux/blob/master/arch/mips/include/uapi/asm/unistd.h
use memory::Memory;
use num_traits::cast::ToPrimitive;
use std::ffi::CString;
use std::io::Error;
use std::time::SystemTime;

#[allow(non_camel_case_types)]
#[derive(Debug, Eq, PartialEq)]
enum SyscallO32 {
    NRSyscall,
    NRExit,
    NRFork,
    NRRead,
    NRWrite,
    NROpen,
    NRClose,
    NRWaitpid,
    NRCreat,
    NRLink,
    NRUnlink,
    NRExecve,
    NRChdir,
    NRTime,
    NRMknod,
    NRChmod,
    NRLchown,
    NRBreak,
    NRUnused18,
    NRLseek,
    NRGetpid,
    NRMount,
    NRUmount,
    NRSetuid,
    NRGetuid,
    NRStime,
    NRPtrace,
    NRAlarm,
    NRUnused28,
    NRPause,
    NRUtime,
    NRStty,
    NRGtty,
    NRAccess,
    NRNice,
    NRFtime,
    NRSync,
    NRKill,
    NRRename,
    NRMkdir,
    NRRmdir,
    NRDup,
    NRPipe,
    NRTimes,
    NRProf,
    NRBrk,
    NRSetgid,
    NRGetgid,
    NRSignal,
    NRGeteuid,
    NRGetegid,
    NRAcct,
    NRUmount2,
    NRLock,
    NRIoctl,
    NRFcntl,
    NRMpx,
    NRSetpgid,
    NRUlimit,
    NRUnused59,
    NRUmask,
    NRChroot,
    NRUstat,
    NRDup2,
    NRGetppid,
    NRGetpgrp,
    NRSetsid,
    NRSigaction,
    NRSgetmask,
    NRSsetmask,
    NRSetreuid,
    NRSetregid,
    NRSigsuspend,
    NRSigpending,
    NRSethostname,
    NRSetrlimit,
    NRGetrlimit,
    NRGetrusage,
    NRGettimeofday,
    NRSettimeofday,
    NRGetgroups,
    NRSetgroups,
    NRReserved82,
    NRSymlink,
    NRUnused84,
    NRReadlink,
    NRUselib,
    NRSwapon,
    NRReboot,
    NRReaddir,
    NRMmap,
    NRMunmap,
    NRTruncate,
    NRFtruncate,
    NRFchmod,
    NRFchown,
    NRGetpriority,
    NRSetpriority,
    NRProfil,
    NRStatfs,
    NRFstatfs,
    NRIoperm,
    NRSocketcall,
    NRSyslog,
    NRSetitimer,
    NRGetitimer,
    NRStat,
    NRLstat,
    NRFstat,
    NRUnused109,
    NRIopl,
    NRVhangup,
    NRIdle,
    NRVm86,
    NRWait4,
    NRSwapoff,
    NRSysinfo,
    NRIpc,
    NRFsync,
    NRSigreturn,
    NRClone,
    NRSetdomainname,
    NRUname,
    NRModify_ldt,
    NRAdjtimex,
    NRMprotect,
    NRSigprocmask,
    NRCreate_module,
    NRInit_module,
    NRDelete_module,
    NRGet_kernel_syms,
    NRQuotactl,
    NRGetpgid,
    NRFchdir,
    NRBdflush,
    NRSysfs,
    NRPersonality,
    NRSetfsuid,
    NRSetfsgid,
    NR_llseek,
    NRGetdents,
    NR_newselect,
    NRFlock,
    NRMsync,
    NRReadv,
    NRWritev,
    NRCacheflush,
    NRCachectl,
    NRSysmips,
    NRUnused150,
    NRGetsid,
    NRFdatasync,
    NR_sysctl,
    NRMlock,
    NRMunlock,
    NRMlockall,
    NRMunlockall,
    NRSched_setparam,
    NRSched_getparam,
    NRSched_setscheduler,
    NRSched_getscheduler,
    NRSched_yield,
    NRSched_get_priority_max,
    NRSched_get_priority_min,
    NRSched_rr_get_interval,
    NRNanosleep,
    NRMremap,
    NRAccept,
    NRBind,
    NRConnect,
    NRGetpeername,
    NRGetsockname,
    NRGetsockopt,
    NRListen,
    NRRecv,
    NRRecvfrom,
    NRRecvmsg,
    NRSend,
    NRSendmsg,
    NRSendto,
    NRSetsockopt,
    NRShutdown,
    NRSocket,
    NRSocketpair,
    NRSetresuid,
    NRGetresuid,
    NRQuery_module,
    NRPoll,
    NRNfsservctl,
    NRSetresgid,
    NRGetresgid,
    NRPrctl,
    NRRt_sigreturn,
    NRRt_sigaction,
    NRRt_sigprocmask,
    NRRt_sigpending,
    NRRt_sigtimedwait,
    NRRt_sigqueueinfo,
    NRRt_sigsuspend,
    NRPread64,
    NRPwrite64,
    NRChown,
    NRGetcwd,
    NRCapget,
    NRCapset,
    NRSigaltstack,
    NRSendfile,
    NRGetpmsg,
    NRPutpmsg,
    NRMmap2,
    NRTruncate64,
    NRFtruncate64,
    NRStat64,
    NRLstat64,
    NRFstat64,
    NRPivot_root,
    NRMincore,
    NRMadvise,
    NRGetdents64,
    NRFcntl64,
    NRReserved221,
    NRGettid,
    NRReadahead,
    NRSetxattr,
    NRLsetxattr,
    NRFsetxattr,
    NRGetxattr,
    NRLgetxattr,
    NRFgetxattr,
    NRListxattr,
    NRLlistxattr,
    NRFlistxattr,
    NRRemovexattr,
    NRLremovexattr,
    NRFremovexattr,
    NRTkill,
    NRSendfile64,
    NRFutex,
    NRSched_setaffinity,
    NRSched_getaffinity,
    NRIo_setup,
    NRIo_destroy,
    NRIo_getevents,
    NRIo_submit,
    NRIo_cancel,
    NRExit_group,
    NRLookup_dcookie,
    NREpoll_create,
    NREpoll_ctl,
    NREpoll_wait,
    NRRemap_file_pages,
    NRSet_tid_address,
    NRRestart_syscall,
    NRFadvise64,
    NRStatfs64,
    NRFstatfs64,
    NRTimer_create,
    NRTimer_settime,
    NRTimer_gettime,
    NRTimer_getoverrun,
    NRTimer_delete,
    NRClock_settime,
    NRClock_gettime,
    NRClock_getres,
    NRClock_nanosleep,
    NRTgkill,
    NRUtimes,
    NRMbind,
    NRGet_mempolicy,
    NRSet_mempolicy,
    NRMq_open,
    NRMq_unlink,
    NRMq_timedsend,
    NRMq_timedreceive,
    NRMq_notify,
    NRMq_getsetattr,
    NRVserver,
    NRWaitid,
    NRAdd_key,
    NRRequest_key,
    NRKeyctl,
    NRSet_thread_area,
    NRInotify_init,
    NRInotify_add_watch,
    NRInotify_rm_watch,
    NRMigrate_pages,
    NROpenat,
    NRMkdirat,
    NRMknodat,
    NRFchownat,
    NRFutimesat,
    NRFstatat64,
    NRUnlinkat,
    NRRenameat,
    NRLinkat,
    NRSymlinkat,
    NRReadlinkat,
    NRFchmodat,
    NRFaccessat,
    NRPselect6,
    NRPpoll,
    NRUnshare,
    NRSplice,
    NRSync_file_range,
    NRTee,
    NRVmsplice,
    NRMove_pages,
    NRSet_robust_list,
    NRGet_robust_list,
    NRKexec_load,
    NRGetcpu,
    NREpoll_pwait,
    NRIoprio_set,
    NRIoprio_get,
    NRUtimensat,
    NRSignalfd,
    NRTimerfd,
    NREventfd,
    NRFallocate,
    NRTimerfd_create,
    NRTimerfd_gettime,
    NRTimerfd_settime,
    NRSignalfd4,
    NREventfd2,
    NREpoll_create1,
    NRDup3,
    NRPipe2,
    NRInotify_init1,
    NRPreadv,
    NRPwritev,
    NRRt_tgsigqueueinfo,
    NRPerf_event_open,
    NRAccept4,
    NRRecvmmsg,
    NRFanotify_init,
    NRFanotify_mark,
    NRPrlimit64,
    NRName_to_handle_at,
    NROpen_by_handle_at,
    NRClock_adjtime,
    NRSyncfs,
    NRSendmmsg,
    NRSetns,
    NRProcess_vm_readv,
    NRProcess_vm_writev,
    NRKcmp,
    NRFinit_module,
    NRSched_setattr,
    NRSched_getattr,
    NRRenameat2,
    NRSeccomp,
    NRGetrandom,
    NRMemfd_create,
    NRBpf,
    NRExecveat,
    NRUserfaultfd,
    NRMembarrier,
    NRMlock2,
    NRCopy_file_range,
    NRPreadv2,
    NRPwritev2,
    NRPkey_mprotect,
    NRPkey_alloc,
    NRPkey_free,
    NRStatx,
    NRUnknown,
}

fn translate_syscall_number(sn: u32) -> SyscallO32 {
    match sn {
        4000 => SyscallO32::NRSyscall,
        4001 => SyscallO32::NRExit,
        4002 => SyscallO32::NRFork,
        4003 => SyscallO32::NRRead,
        4004 => SyscallO32::NRWrite,
        4005 => SyscallO32::NROpen,
        4006 => SyscallO32::NRClose,
        4007 => SyscallO32::NRWaitpid,
        4008 => SyscallO32::NRCreat,
        4009 => SyscallO32::NRLink,
        4010 => SyscallO32::NRUnlink,
        4011 => SyscallO32::NRExecve,
        4012 => SyscallO32::NRChdir,
        4013 => SyscallO32::NRTime,
        4014 => SyscallO32::NRMknod,
        4015 => SyscallO32::NRChmod,
        4016 => SyscallO32::NRLchown,
        4017 => SyscallO32::NRBreak,
        4018 => SyscallO32::NRUnused18,
        4019 => SyscallO32::NRLseek,
        4020 => SyscallO32::NRGetpid,
        4021 => SyscallO32::NRMount,
        4022 => SyscallO32::NRUmount,
        4023 => SyscallO32::NRSetuid,
        4024 => SyscallO32::NRGetuid,
        4025 => SyscallO32::NRStime,
        4026 => SyscallO32::NRPtrace,
        4027 => SyscallO32::NRAlarm,
        4028 => SyscallO32::NRUnused28,
        4029 => SyscallO32::NRPause,
        4030 => SyscallO32::NRUtime,
        4031 => SyscallO32::NRStty,
        4032 => SyscallO32::NRGtty,
        4033 => SyscallO32::NRAccess,
        4034 => SyscallO32::NRNice,
        4035 => SyscallO32::NRFtime,
        4036 => SyscallO32::NRSync,
        4037 => SyscallO32::NRKill,
        4038 => SyscallO32::NRRename,
        4039 => SyscallO32::NRMkdir,
        4040 => SyscallO32::NRRmdir,
        4041 => SyscallO32::NRDup,
        4042 => SyscallO32::NRPipe,
        4043 => SyscallO32::NRTimes,
        4044 => SyscallO32::NRProf,
        4045 => SyscallO32::NRBrk,
        4046 => SyscallO32::NRSetgid,
        4047 => SyscallO32::NRGetgid,
        4048 => SyscallO32::NRSignal,
        4049 => SyscallO32::NRGeteuid,
        4050 => SyscallO32::NRGetegid,
        4051 => SyscallO32::NRAcct,
        4052 => SyscallO32::NRUmount2,
        4053 => SyscallO32::NRLock,
        4054 => SyscallO32::NRIoctl,
        4055 => SyscallO32::NRFcntl,
        4056 => SyscallO32::NRMpx,
        4057 => SyscallO32::NRSetpgid,
        4058 => SyscallO32::NRUlimit,
        4059 => SyscallO32::NRUnused59,
        4060 => SyscallO32::NRUmask,
        4061 => SyscallO32::NRChroot,
        4062 => SyscallO32::NRUstat,
        4063 => SyscallO32::NRDup2,
        4064 => SyscallO32::NRGetppid,
        4065 => SyscallO32::NRGetpgrp,
        4066 => SyscallO32::NRSetsid,
        4067 => SyscallO32::NRSigaction,
        4068 => SyscallO32::NRSgetmask,
        4069 => SyscallO32::NRSsetmask,
        4070 => SyscallO32::NRSetreuid,
        4071 => SyscallO32::NRSetregid,
        4072 => SyscallO32::NRSigsuspend,
        4073 => SyscallO32::NRSigpending,
        4074 => SyscallO32::NRSethostname,
        4075 => SyscallO32::NRSetrlimit,
        4076 => SyscallO32::NRGetrlimit,
        4077 => SyscallO32::NRGetrusage,
        4078 => SyscallO32::NRGettimeofday,
        4079 => SyscallO32::NRSettimeofday,
        4080 => SyscallO32::NRGetgroups,
        4081 => SyscallO32::NRSetgroups,
        4082 => SyscallO32::NRReserved82,
        4083 => SyscallO32::NRSymlink,
        4084 => SyscallO32::NRUnused84,
        4085 => SyscallO32::NRReadlink,
        4086 => SyscallO32::NRUselib,
        4087 => SyscallO32::NRSwapon,
        4088 => SyscallO32::NRReboot,
        4089 => SyscallO32::NRReaddir,
        4090 => SyscallO32::NRMmap,
        4091 => SyscallO32::NRMunmap,
        4092 => SyscallO32::NRTruncate,
        4093 => SyscallO32::NRFtruncate,
        4094 => SyscallO32::NRFchmod,
        4095 => SyscallO32::NRFchown,
        4096 => SyscallO32::NRGetpriority,
        4097 => SyscallO32::NRSetpriority,
        4098 => SyscallO32::NRProfil,
        4099 => SyscallO32::NRStatfs,
        4100 => SyscallO32::NRFstatfs,
        4101 => SyscallO32::NRIoperm,
        4102 => SyscallO32::NRSocketcall,
        4103 => SyscallO32::NRSyslog,
        4104 => SyscallO32::NRSetitimer,
        4105 => SyscallO32::NRGetitimer,
        4106 => SyscallO32::NRStat,
        4107 => SyscallO32::NRLstat,
        4108 => SyscallO32::NRFstat,
        4109 => SyscallO32::NRUnused109,
        4110 => SyscallO32::NRIopl,
        4111 => SyscallO32::NRVhangup,
        4112 => SyscallO32::NRIdle,
        4113 => SyscallO32::NRVm86,
        4114 => SyscallO32::NRWait4,
        4115 => SyscallO32::NRSwapoff,
        4116 => SyscallO32::NRSysinfo,
        4117 => SyscallO32::NRIpc,
        4118 => SyscallO32::NRFsync,
        4119 => SyscallO32::NRSigreturn,
        4120 => SyscallO32::NRClone,
        4121 => SyscallO32::NRSetdomainname,
        4122 => SyscallO32::NRUname,
        4123 => SyscallO32::NRModify_ldt,
        4124 => SyscallO32::NRAdjtimex,
        4125 => SyscallO32::NRMprotect,
        4126 => SyscallO32::NRSigprocmask,
        4127 => SyscallO32::NRCreate_module,
        4128 => SyscallO32::NRInit_module,
        4129 => SyscallO32::NRDelete_module,
        4130 => SyscallO32::NRGet_kernel_syms,
        4131 => SyscallO32::NRQuotactl,
        4132 => SyscallO32::NRGetpgid,
        4133 => SyscallO32::NRFchdir,
        4134 => SyscallO32::NRBdflush,
        4135 => SyscallO32::NRSysfs,
        4136 => SyscallO32::NRPersonality,
        4138 => SyscallO32::NRSetfsuid,
        4139 => SyscallO32::NRSetfsgid,
        4140 => SyscallO32::NR_llseek,
        4141 => SyscallO32::NRGetdents,
        4142 => SyscallO32::NR_newselect,
        4143 => SyscallO32::NRFlock,
        4144 => SyscallO32::NRMsync,
        4145 => SyscallO32::NRReadv,
        4146 => SyscallO32::NRWritev,
        4147 => SyscallO32::NRCacheflush,
        4148 => SyscallO32::NRCachectl,
        4149 => SyscallO32::NRSysmips,
        4150 => SyscallO32::NRUnused150,
        4151 => SyscallO32::NRGetsid,
        4152 => SyscallO32::NRFdatasync,
        4153 => SyscallO32::NR_sysctl,
        4154 => SyscallO32::NRMlock,
        4155 => SyscallO32::NRMunlock,
        4156 => SyscallO32::NRMlockall,
        4157 => SyscallO32::NRMunlockall,
        4158 => SyscallO32::NRSched_setparam,
        4159 => SyscallO32::NRSched_getparam,
        4160 => SyscallO32::NRSched_setscheduler,
        4161 => SyscallO32::NRSched_getscheduler,
        4162 => SyscallO32::NRSched_yield,
        4163 => SyscallO32::NRSched_get_priority_max,
        4164 => SyscallO32::NRSched_get_priority_min,
        4165 => SyscallO32::NRSched_rr_get_interval,
        4166 => SyscallO32::NRNanosleep,
        4167 => SyscallO32::NRMremap,
        4168 => SyscallO32::NRAccept,
        4169 => SyscallO32::NRBind,
        4170 => SyscallO32::NRConnect,
        4171 => SyscallO32::NRGetpeername,
        4172 => SyscallO32::NRGetsockname,
        4173 => SyscallO32::NRGetsockopt,
        4174 => SyscallO32::NRListen,
        4175 => SyscallO32::NRRecv,
        4176 => SyscallO32::NRRecvfrom,
        4177 => SyscallO32::NRRecvmsg,
        4178 => SyscallO32::NRSend,
        4179 => SyscallO32::NRSendmsg,
        4180 => SyscallO32::NRSendto,
        4181 => SyscallO32::NRSetsockopt,
        4182 => SyscallO32::NRShutdown,
        4183 => SyscallO32::NRSocket,
        4184 => SyscallO32::NRSocketpair,
        4185 => SyscallO32::NRSetresuid,
        4186 => SyscallO32::NRGetresuid,
        4187 => SyscallO32::NRQuery_module,
        4188 => SyscallO32::NRPoll,
        4189 => SyscallO32::NRNfsservctl,
        4190 => SyscallO32::NRSetresgid,
        4191 => SyscallO32::NRGetresgid,
        4192 => SyscallO32::NRPrctl,
        4193 => SyscallO32::NRRt_sigreturn,
        4194 => SyscallO32::NRRt_sigaction,
        4195 => SyscallO32::NRRt_sigprocmask,
        4196 => SyscallO32::NRRt_sigpending,
        4197 => SyscallO32::NRRt_sigtimedwait,
        4198 => SyscallO32::NRRt_sigqueueinfo,
        4199 => SyscallO32::NRRt_sigsuspend,
        4200 => SyscallO32::NRPread64,
        4201 => SyscallO32::NRPwrite64,
        4202 => SyscallO32::NRChown,
        4203 => SyscallO32::NRGetcwd,
        4204 => SyscallO32::NRCapget,
        4205 => SyscallO32::NRCapset,
        4206 => SyscallO32::NRSigaltstack,
        4207 => SyscallO32::NRSendfile,
        4208 => SyscallO32::NRGetpmsg,
        4209 => SyscallO32::NRPutpmsg,
        4210 => SyscallO32::NRMmap2,
        4211 => SyscallO32::NRTruncate64,
        4212 => SyscallO32::NRFtruncate64,
        4213 => SyscallO32::NRStat64,
        4214 => SyscallO32::NRLstat64,
        4215 => SyscallO32::NRFstat64,
        4216 => SyscallO32::NRPivot_root,
        4217 => SyscallO32::NRMincore,
        4218 => SyscallO32::NRMadvise,
        4219 => SyscallO32::NRGetdents64,
        4220 => SyscallO32::NRFcntl64,
        4221 => SyscallO32::NRReserved221,
        4222 => SyscallO32::NRGettid,
        4223 => SyscallO32::NRReadahead,
        4224 => SyscallO32::NRSetxattr,
        4225 => SyscallO32::NRLsetxattr,
        4226 => SyscallO32::NRFsetxattr,
        4227 => SyscallO32::NRGetxattr,
        4228 => SyscallO32::NRLgetxattr,
        4229 => SyscallO32::NRFgetxattr,
        4230 => SyscallO32::NRListxattr,
        4231 => SyscallO32::NRLlistxattr,
        4232 => SyscallO32::NRFlistxattr,
        4233 => SyscallO32::NRRemovexattr,
        4234 => SyscallO32::NRLremovexattr,
        4235 => SyscallO32::NRFremovexattr,
        4236 => SyscallO32::NRTkill,
        4237 => SyscallO32::NRSendfile64,
        4238 => SyscallO32::NRFutex,
        4239 => SyscallO32::NRSched_setaffinity,
        4240 => SyscallO32::NRSched_getaffinity,
        4241 => SyscallO32::NRIo_setup,
        4242 => SyscallO32::NRIo_destroy,
        4243 => SyscallO32::NRIo_getevents,
        4244 => SyscallO32::NRIo_submit,
        4245 => SyscallO32::NRIo_cancel,
        4246 => SyscallO32::NRExit_group,
        4247 => SyscallO32::NRLookup_dcookie,
        4248 => SyscallO32::NREpoll_create,
        4249 => SyscallO32::NREpoll_ctl,
        4250 => SyscallO32::NREpoll_wait,
        4251 => SyscallO32::NRRemap_file_pages,
        4252 => SyscallO32::NRSet_tid_address,
        4253 => SyscallO32::NRRestart_syscall,
        4254 => SyscallO32::NRFadvise64,
        4255 => SyscallO32::NRStatfs64,
        4256 => SyscallO32::NRFstatfs64,
        4257 => SyscallO32::NRTimer_create,
        4258 => SyscallO32::NRTimer_settime,
        4259 => SyscallO32::NRTimer_gettime,
        4260 => SyscallO32::NRTimer_getoverrun,
        4261 => SyscallO32::NRTimer_delete,
        4262 => SyscallO32::NRClock_settime,
        4263 => SyscallO32::NRClock_gettime,
        4264 => SyscallO32::NRClock_getres,
        4265 => SyscallO32::NRClock_nanosleep,
        4266 => SyscallO32::NRTgkill,
        4267 => SyscallO32::NRUtimes,
        4268 => SyscallO32::NRMbind,
        4269 => SyscallO32::NRGet_mempolicy,
        4270 => SyscallO32::NRSet_mempolicy,
        4271 => SyscallO32::NRMq_open,
        4272 => SyscallO32::NRMq_unlink,
        4273 => SyscallO32::NRMq_timedsend,
        4274 => SyscallO32::NRMq_timedreceive,
        4275 => SyscallO32::NRMq_notify,
        4276 => SyscallO32::NRMq_getsetattr,
        4277 => SyscallO32::NRVserver,
        4278 => SyscallO32::NRWaitid,
        4280 => SyscallO32::NRAdd_key,
        4281 => SyscallO32::NRRequest_key,
        4282 => SyscallO32::NRKeyctl,
        4283 => SyscallO32::NRSet_thread_area,
        4284 => SyscallO32::NRInotify_init,
        4285 => SyscallO32::NRInotify_add_watch,
        4286 => SyscallO32::NRInotify_rm_watch,
        4287 => SyscallO32::NRMigrate_pages,
        4288 => SyscallO32::NROpenat,
        4289 => SyscallO32::NRMkdirat,
        4290 => SyscallO32::NRMknodat,
        4291 => SyscallO32::NRFchownat,
        4292 => SyscallO32::NRFutimesat,
        4293 => SyscallO32::NRFstatat64,
        4294 => SyscallO32::NRUnlinkat,
        4295 => SyscallO32::NRRenameat,
        4296 => SyscallO32::NRLinkat,
        4297 => SyscallO32::NRSymlinkat,
        4298 => SyscallO32::NRReadlinkat,
        4299 => SyscallO32::NRFchmodat,
        4300 => SyscallO32::NRFaccessat,
        4301 => SyscallO32::NRPselect6,
        4302 => SyscallO32::NRPpoll,
        4303 => SyscallO32::NRUnshare,
        4304 => SyscallO32::NRSplice,
        4305 => SyscallO32::NRSync_file_range,
        4306 => SyscallO32::NRTee,
        4307 => SyscallO32::NRVmsplice,
        4308 => SyscallO32::NRMove_pages,
        4309 => SyscallO32::NRSet_robust_list,
        4310 => SyscallO32::NRGet_robust_list,
        4311 => SyscallO32::NRKexec_load,
        4312 => SyscallO32::NRGetcpu,
        4313 => SyscallO32::NREpoll_pwait,
        4314 => SyscallO32::NRIoprio_set,
        4315 => SyscallO32::NRIoprio_get,
        4316 => SyscallO32::NRUtimensat,
        4317 => SyscallO32::NRSignalfd,
        4318 => SyscallO32::NRTimerfd,
        4319 => SyscallO32::NREventfd,
        4320 => SyscallO32::NRFallocate,
        4321 => SyscallO32::NRTimerfd_create,
        4322 => SyscallO32::NRTimerfd_gettime,
        4323 => SyscallO32::NRTimerfd_settime,
        4324 => SyscallO32::NRSignalfd4,
        4325 => SyscallO32::NREventfd2,
        4326 => SyscallO32::NREpoll_create1,
        4327 => SyscallO32::NRDup3,
        4328 => SyscallO32::NRPipe2,
        4329 => SyscallO32::NRInotify_init1,
        4330 => SyscallO32::NRPreadv,
        4331 => SyscallO32::NRPwritev,
        4332 => SyscallO32::NRRt_tgsigqueueinfo,
        4333 => SyscallO32::NRPerf_event_open,
        4334 => SyscallO32::NRAccept4,
        4335 => SyscallO32::NRRecvmmsg,
        4336 => SyscallO32::NRFanotify_init,
        4337 => SyscallO32::NRFanotify_mark,
        4338 => SyscallO32::NRPrlimit64,
        4339 => SyscallO32::NRName_to_handle_at,
        4340 => SyscallO32::NROpen_by_handle_at,
        4341 => SyscallO32::NRClock_adjtime,
        4342 => SyscallO32::NRSyncfs,
        4343 => SyscallO32::NRSendmmsg,
        4344 => SyscallO32::NRSetns,
        4345 => SyscallO32::NRProcess_vm_readv,
        4346 => SyscallO32::NRProcess_vm_writev,
        4347 => SyscallO32::NRKcmp,
        4348 => SyscallO32::NRFinit_module,
        4349 => SyscallO32::NRSched_setattr,
        4350 => SyscallO32::NRSched_getattr,
        4351 => SyscallO32::NRRenameat2,
        4352 => SyscallO32::NRSeccomp,
        4353 => SyscallO32::NRGetrandom,
        4354 => SyscallO32::NRMemfd_create,
        4355 => SyscallO32::NRBpf,
        4356 => SyscallO32::NRExecveat,
        4357 => SyscallO32::NRUserfaultfd,
        4358 => SyscallO32::NRMembarrier,
        4359 => SyscallO32::NRMlock2,
        4360 => SyscallO32::NRCopy_file_range,
        4361 => SyscallO32::NRPreadv2,
        4362 => SyscallO32::NRPwritev2,
        4363 => SyscallO32::NRPkey_mprotect,
        4364 => SyscallO32::NRPkey_alloc,
        4365 => SyscallO32::NRPkey_free,
        4366 => SyscallO32::NRStatx,
        _ => SyscallO32::NRUnknown,
    }
}

struct Iovec {
    pub iov_base: u32,
    pub iov_len: usize,
}

fn translate_iovec(iovec_addr: u32, iovcnt: u32, memory: &mut Memory) -> Vec<Iovec> {
    let mut iovec: Vec<Iovec> = Vec::with_capacity(iovcnt as usize);
    for i in 0..(iovcnt as u32) {
        let addr = memory.read_word(iovec_addr + i * 8);
        let len = memory.read_word(iovec_addr + i * 8 + 4);

        iovec.push(Iovec {
            iov_base: addr,
            iov_len: len as usize,
        });
    }
    iovec
}

fn translate_iovec_libc(iovec_addr: u32, iovcnt: u32, memory: &mut Memory) -> Vec<::libc::iovec> {
    let mut iovec: Vec<::libc::iovec> = Vec::with_capacity(iovcnt as usize);
    for i in 0..(iovcnt as u32) {
        let addr = memory.read_word(iovec_addr + i * 8);
        let len = memory.read_word(iovec_addr + i * 8 + 4);

        iovec.push(::libc::iovec {
            iov_base: memory.translate_address_mut(addr) as *mut ::libc::c_void,
            iov_len: len as usize,
        });
    }
    iovec
}

fn check_error<T: Default + Ord + ToPrimitive>(num: T) -> Result<u32, Error> {
    if num < T::default() {
        let e = Error::last_os_error();
        warn!("Syscall error: {:?}", e);
        Err(e)
    } else {
        Ok(num.to_u32().unwrap())
    }
}

pub fn eval_syscall<T>(_inst: u32, registers: &mut RegisterFile<T>, memory: &mut Memory, flags: &CPUFlags) -> CPUEvent
where
    T: Fn(u32, u32),
{
    macro_rules! itrace {
        ($fmt:expr, $($arg:tt)*) => (
            info!(concat!("0x{:x}:\tsyscall\t", $fmt), registers.get_pc(), $($arg)*);
        );
        ($fmt:expr) => (
            info!(concat!("0x{:x}:\tsyscall\t", $fmt), registers.get_pc());
        );
    }

    let syscall_number = registers.read_register(2);
    let arg1 = registers.read_register(4);
    let arg2 = registers.read_register(5);
    let arg3 = registers.read_register(6);
    let arg4 = registers.read_register(7);

    let translated_syscall_number = translate_syscall_number(syscall_number);
    let mut exit = CPUEvent::Nothing;

    if translated_syscall_number == SyscallO32::NRUnknown {
        error!(
            "sysnum={} arg1={} arg2={} arg3={} arg4={}\n",
            syscall_number, arg1, arg2, arg3, arg4
        );
        error!("Failed to translate syscall.");
        panic!("Unknown SYSCALL");
    } else {
        let result: Result<u32, Error> = match translated_syscall_number {
            SyscallO32::NRBrk => {
                // TODO consider whether this should not be implemented differently
                itrace!("BRK (faked)");
                if arg1 == 0 {
                    Ok(memory.get_program_break())
                } else {
                    memory.update_program_break(arg1);
                    Ok(arg1)
                }
            }
            SyscallO32::NRSet_thread_area => {
                itrace!("SET_THREAD_AREA (ignored)");
                Ok(0)
            }
            SyscallO32::NRSet_tid_address => {
                itrace!("SET_TID_ADDRESS (ignored)");
                Ok(0)
            }
            SyscallO32::NRRt_sigprocmask => {
                itrace!("RT_SIGPROCMASK (ignored)");
                Ok(0)
            }
            SyscallO32::NRGetuid => {
                if flags.fake_root {
                    itrace!("GETUID (faked)");
                    Ok(0)
                } else {
                    itrace!("GETUID (real)");
                    check_error(unsafe { ::libc::getuid() })
                }
            }
            SyscallO32::NRGeteuid => {
                if flags.fake_root {
                    itrace!("GETEUID (faked)");
                    Ok(0)
                } else {
                    itrace!("GETEUID (real)");
                    check_error(unsafe { ::libc::getuid() })
                }
            }
            SyscallO32::NRGetgid => {
                itrace!("GETGID");
                if flags.fake_root {
                    Ok(0)
                } else {
                    check_error(unsafe { ::libc::getgid() })
                }
            }
            SyscallO32::NRGetegid => {
                itrace!("GETEGID");
                if flags.fake_root {
                    Ok(0)
                } else {
                    check_error(unsafe { ::libc::getegid() })
                }
            }
            SyscallO32::NRGetpid => {
                itrace!("GETPID");
                check_error(unsafe { ::libc::getpid() } )
            }
            SyscallO32::NRStat64 => {
                //panic!("This syscall was disabled!");

                //FIXME struct translation ??

                let (file, res) = unsafe {
                    (
                        CString::from_raw(memory.translate_address_mut(arg1) as *mut i8),
                        ::libc::stat(
                            memory.translate_address(arg1) as *mut i8,
                            memory.translate_address_mut(arg2) as *mut ::libc::stat,
                        ),
                    )
                };
                itrace!(
                    "STAT64 file={:?} struct_at=0x{:08x} res={}",
                    file,
                    arg2,
                    res
                );
                check_error(res)
            }
            SyscallO32::NRIoctl => {
                itrace!("IOCTL a0={} a1=0x{:x} a2=0x{:x}", arg1, arg2, arg3);

                let fd = arg1;
                if flags.block_ioctl_on_stdio && fd < 3 {
                    warn!("IOCTL ignored - manipulating with FD<=2. Returning success.");
                    Ok(0)
                } else if flags.ioctl_fail_always {
                    warn!("IOCTL forced to fail! Returning EINVAL.");
                    Err(Error::from_raw_os_error(::libc::EINVAL))
                } else  {
                    warn!("Syscall IOCTL might not work as expected due to struct translation missing and probably impossible.");
                    check_error(unsafe {
                        ::libc::ioctl(arg1 as i32, arg2 as u64, memory.translate_address(arg3))
                    })
                }
            }
            SyscallO32::NRDup2 => {
                itrace!("DUP2 oldfd={} newfd={}", arg1, arg2);

                check_error(unsafe { ::libc::dup2(arg1 as i32, arg2 as i32) })
            }
            SyscallO32::NRWrite => {
                itrace!("WRITE fd={} ptr=0x{:x} len={}", arg1, arg2, arg3);

                check_error(unsafe {
                    ::libc::write(
                        arg1 as i32,
                        memory.translate_address(arg2) as *const ::libc::c_void,
                        arg3 as usize,
                    )
                })
            }
            SyscallO32::NRWritev => {
                // translating data structure
                itrace!("WRITEV (emulated)");

                const DIRECT_WRITE: bool = false;

                let fd = arg1 as i32;
                if DIRECT_WRITE {
                    // This branch translates 32bit iovec array to native one and directly calls the kernel
                    let mut iovec = translate_iovec_libc(arg2, arg3, memory);

                    check_error(unsafe {
                        ::libc::writev(
                            fd as i32,
                            iovec.as_slice().as_ptr() as *const ::libc::iovec,
                            arg3 as i32,
                        )
                    })
                } else {
                    // This branch here carefully copies data from emulated memory to single buffer
                    // and then uses WRITE syscall to dump it
                    // This should prevent kernel to doing unexpected stuff to our emulated memory

                    let mut iovec = translate_iovec(arg2, arg3, memory);
                    let total_size = iovec.iter().map(|iov| iov.iov_len).sum();
                    let mut buffer = vec![0; total_size];
                    let mut i = 0;
                    for iov in iovec.into_iter() {
                        let bufslice = &mut buffer.as_mut_slice()[i..i + iov.iov_len];
                        let memslice = memory.get_slice(
                            iov.iov_base as usize,
                            (iov.iov_base as usize) + iov.iov_len,
                        );
                        bufslice.copy_from_slice(memslice);
                        i += iov.iov_len;
                    }
                    let ptr = buffer.as_slice().as_ptr();

                    check_error(unsafe {
                        ::libc::write(fd as i32, ptr as *const ::libc::c_void, total_size)
                    })
                }
            }
            SyscallO32::NRRead => {
                let fd = arg1 as i32;
                let size = arg3 as usize;
                itrace!("READ fd={} buf_size={}", fd, size);
                let ptr = memory.translate_address_mut(arg2) as *mut ::libc::c_void;
                
                check_error(unsafe { ::libc::read(fd, ptr, size) })
            }
            SyscallO32::NRReadv => {
                itrace!("READV (emulated)");
                let fd = arg1 as i32;
                let mut iovec = translate_iovec(arg2, arg3, memory);
                let total_size = iovec.iter().map(|iovec: &Iovec| iovec.iov_len).sum();

                let mut buffer = vec![0u8; total_size];
                let ptr = buffer.as_mut_slice().as_mut_ptr() as *mut ::libc::c_void;

                let data_read = unsafe { ::libc::read(fd, ptr, total_size) };

                if data_read == -1 {
                    check_error(data_read)
                } else {
                    {
                        let mut data_read = data_read as usize;
                        let mut already_written = 0;
                        for iov in iovec.into_iter() {
                            let l = if data_read > iov.iov_len {
                                iov.iov_len
                            } else {
                                data_read
                            };
                            let slice = &buffer.as_slice()[already_written..already_written + l];
                            memory.write_block(iov.iov_base, slice);
                            data_read -= l;

                            if data_read == 0 {
                                break;
                            }
                        }
                    }

                    Ok(data_read as u32)
                }
            }
            SyscallO32::NROpen => {
                let mut flags = arg2 as i32;
                /*
                let mips_O_LARGEFILE = 8192;
                if flags & mips_O_LARGEFILE == mips_O_LARGEFILE {
                    flags ^= mips_O_LARGEFILE;
                }
                */
                let (file, res) = unsafe {
                    (
                        CString::from_raw(memory.translate_address_mut(arg1) as *mut i8),
                        ::libc::open(memory.translate_address(arg1) as *const i8, flags),
                    )
                };
                itrace!("OPEN file={:?} flags=0x{:08x} res_fd={}", file, arg2, res);
                check_error(res)
            }
            SyscallO32::NRClose => {
                let fd = arg1 as i32;

                itrace!("CLOSE fd={}", fd);

                check_error(unsafe { ::libc::close(fd as i32) as isize })
            }
            SyscallO32::NRExit_group => {
                itrace!("EXIT_GROUP");
                exit = CPUEvent::Exit;
                Ok(0)
            }
            SyscallO32::NRExit => {
                itrace!("EXIT");
                exit = CPUEvent::Exit;
                Ok(0)
            }
            SyscallO32::NR_llseek => {
                itrace!("_LLSEEK (emulated)");

                let fd = arg1 as i32;
                let offset: i64 = (((arg2 as u64) << 32) | (arg3 as u64)) as i64;
                let result_pointer = arg4;
                let whence = memory.read_word(registers.read_register(STACK_POINTER) + 4 * 4);

                let mut result = unsafe { ::libc::lseek(fd as i32, offset, whence as i32) };

                if result != -1 {
                    memory.write_word(result_pointer, result as u32);
                    memory.write_word(result_pointer + 4, (result << 32) as u32);
                    Ok(0)
                } else {
                    check_error(result)
                }
            }
            SyscallO32::NRGetcwd => {
                let buf_addr = arg1;
                let buf_size = arg2;

                if flags.fake_root_directory && buf_size >= 6 {
                    itrace!("GETCWD (faked) ptr=0x{:x}", buf_addr);
                    memory.write_byte(buf_addr + 0, '/' as u32);
                    memory.write_byte(buf_addr + 1, 'r' as u32);
                    memory.write_byte(buf_addr + 2, 'o' as u32);
                    memory.write_byte(buf_addr + 3, 'o' as u32);
                    memory.write_byte(buf_addr + 4, 't' as u32);
                    memory.write_byte(buf_addr + 5, 0);
                    Ok(buf_addr)
                } else {
                    let (res, cwd) = unsafe {
                        (
                            ::libc::getcwd(
                                memory.translate_address_mut(buf_addr) as *mut ::libc::c_char,
                                buf_size as usize,
                            ) as usize,
                            CString::from_raw(memory.translate_address_mut(buf_addr) as *mut i8),
                        )
                    };

                    itrace!("GETCWD result_cwd={:?} real_ptr=0x{:x} emu_ptr=0x{:x}", cwd, res, buf_addr);
                    if res == 0 {
                        check_error(-1i32)
                    } else {
                        Ok(buf_addr)
                    }
                }
            }
            SyscallO32::NRTime => {
                itrace!("TIME tloc_ptr={}", arg1);
                let tloc_ptr = arg1;
                let seconds = match SystemTime::now().duration_since(::std::time::UNIX_EPOCH) {
                    Ok(duration) => duration.as_secs(),
                    Err(_) => panic!("Weird time! UNIX epoch is in the future?")
                };

                if tloc_ptr != 0 {
                    memory.write_word(tloc_ptr, seconds as u32);
                }

                Ok(seconds as u32)
            }
            SyscallO32::NRSetgid => {
                itrace!("SETGID gid={}", arg1);
                check_error(unsafe { ::libc::setgid(arg1 as ::libc::gid_t)})
            }
            SyscallO32::NRSetuid => {
                itrace!("SETUID uid={}", arg1);
                check_error(unsafe { ::libc::setuid(arg1 as ::libc::uid_t)})
            }
            _ => {
                error!(
                    "sysnum={} arg1={} arg2={} arg3={} arg4={}\n",
                    syscall_number, arg1, arg2, arg3, arg4
                );
                panic!(
                    "Syscall translated, but unknown. OrigNum={} TrNum={:?}",
                    syscall_number, translated_syscall_number
                )
            }
        };

        match result {
            Ok(res) => {
                debug!("Syscall result - SUCCESS - return_value=0x{:x}", res);
                registers.write_register(V0, res as u32);
                registers.write_register(A3, 0); // no error
            }
            Err(err) => {
                registers.write_register(
                    V0,
                    err.raw_os_error().expect("Could not access errno.") as u32,
                );
                registers.write_register(A3, 1); // error
            }
        }

        return exit;
    }
}
