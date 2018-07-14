use cpu::control::CPUFlagsSyscalls;
use cpu::event::CPUEvent;
use cpu::registers::A3;
use cpu::registers::RegisterFile;
use cpu::registers::V0;
use cpu::registers::STACK_POINTER;
/// List of MIPS syscall numbers can be found here:
/// https://github.com/torvalds/linux/blob/master/arch/mips/include/uapi/asm/unistd.h
use memory::Memory;
use nix::sys::signal::{sigaction, SaFlags, SigAction, SigHandler, SigSet, Signal};
use num_traits::cast::ToPrimitive;
use std::collections::HashMap;
use std::ffi::CStr;
use std::io::Error;
use std::mem::size_of;
use std::time::SystemTime;
use syscall_numbers::*;

struct Iovec {
    pub iov_base: u32,
    pub iov_len: usize,
}

#[repr(C)]
struct MipsStat {
    st_dev: u64,
    _st_padding1: [u32; 2],
    st_ino: u64,
    st_mode: u32,
    st_nlink: u32,
    st_uid: u32,
    st_gid: u32,
    st_rdev: u64,
    _st_padding2: [u32; 2],
    st_size: u64,
    st_atime: u32,
    st_atime_nsec: u32,
    st_mtime: u32,
    st_mtime_nsec: u32,
    st_ctime: u32,
    st_ctime_nsec: u32,
    st_blksize: u32,
    _st_padding3: u32,
    st_blocks: u64,
    _st_padding4: [u32; 14],
}

#[repr(C)]
struct MipsSigaction {
    __sa_handler: u32,
    sa_mask: [u32; 4],
    sa_flags: i32,
    sa_restorer: u32,
}

impl From<[u32; 7]> for MipsSigaction {
    fn from(struc: [u32; 7]) -> MipsSigaction {
        unsafe { ::std::mem::transmute(struc) }
    }
}

impl From<SigAction> for MipsSigaction {
    fn from(struc: SigAction) -> MipsSigaction {
        // glibc sigset_t has 1024bits, kernel sigset_t has 128bits
        let x: [u32; 32] = unsafe { ::std::mem::transmute(struc.mask()) };
        MipsSigaction {
            __sa_handler: ::libc::SIG_DFL as u32,
            sa_restorer: 0,
            sa_mask: [x[0], x[1], x[2], x[3]],
            sa_flags: 0,
        }
    }
}

impl Into<[u32; 7]> for MipsSigaction {
    fn into(self) -> [u32; 7] {
        unsafe { ::std::mem::transmute(self) }
    }
}

fn translate_stat(stat: ::libc::stat) -> [u32; size_of::<MipsStat>() / 4] {
    let s = MipsStat {
        st_dev: stat.st_dev,
        st_ino: stat.st_ino,
        st_mode: stat.st_mode,
        st_nlink: stat.st_nlink as u32,
        st_uid: stat.st_uid,
        st_gid: stat.st_gid,
        st_rdev: stat.st_rdev,
        st_size: stat.st_size as u64,
        st_atime: stat.st_atime as u32,
        st_atime_nsec: stat.st_atime_nsec as u32,
        st_mtime: stat.st_mtime as u32,
        st_mtime_nsec: stat.st_mtime_nsec as u32,
        st_ctime: stat.st_ctime as u32,
        st_ctime_nsec: stat.st_ctime_nsec as u32,
        st_blksize: stat.st_blksize as u32,
        st_blocks: stat.st_blocks as u64,
        _st_padding1: [0, 0],
        _st_padding2: [0, 0],
        _st_padding3: 0,
        _st_padding4: [0u32; 14],
    };

    unsafe { ::std::mem::transmute(s) }
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

#[allow(overflowing_literals)]
fn translate_signal_flags(mask: i32) -> i32 {
    const SA_ONSTACK: i32 = 0x08000000;
    const SA_RESETHAND: i32 = 0x80000000;
    const SA_RESTART: i32 = 0x10000000;
    const SA_SIGINFO: i32 = 0x00000008;
    const SA_NODEFER: i32 = 0x40000000;
    const SA_NOCLDWAIT: i32 = 0x00010000;
    const SA_NOCLDSTOP: i32 = 0x00000001;

    let mut res = 0i32;
    if mask & SA_ONSTACK == SA_ONSTACK {
        res ^= ::libc::SA_ONSTACK;
    }
    if mask & SA_RESETHAND == SA_RESETHAND {
        res ^= ::libc::SA_RESETHAND;
    }
    if mask & SA_RESTART == SA_RESTART {
        res ^= ::libc::SA_RESTART;
    }
    if mask & SA_SIGINFO == SA_SIGINFO {
        res ^= ::libc::SA_SIGINFO;
    }
    if mask & SA_NODEFER == SA_NODEFER {
        res ^= ::libc::SA_NODEFER;
    }
    if mask & SA_NOCLDWAIT == SA_NOCLDWAIT {
        res ^= ::libc::SA_NOCLDWAIT;
    }
    if mask & SA_NOCLDSTOP == SA_NOCLDSTOP {
        res ^= ::libc::SA_NOCLDSTOP;
    }
    res
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

fn read_argument_from_memory(argn: u32, registers: &RegisterFile, memory: &Memory) -> u32 {
    assert!(argn > 4);
    let argn = argn - 1;
    memory.read_word(registers.read_register(STACK_POINTER) + 4 * argn)
}

pub struct System {
    config: CPUFlagsSyscalls,
    sigactions: HashMap<u32, MipsSigaction>,
}

impl System {
    pub fn new(config: CPUFlagsSyscalls) -> System {
        System {
            config,
            sigactions: HashMap::new(),
        }
    }

    pub fn eval_syscall(
        &mut self,
        _inst: u32,
        registers: &mut RegisterFile,
        memory: &mut Memory,
    ) -> CPUEvent {
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
                    let how = arg1;
                    itrace!("RT_SIGPROCMASK how={}", how);

                    // sigset is 128bits wide = 16 bytes (kernel)
                    // sigset is 1024bits wide = 128bytes (glibc)
                    let mut sigset = [0u32; 32];
                    if arg2 != 0 {
                        for i in 0..32 {
                            sigset[i as usize] = memory.read_word(arg2 + i * 4);
                        }
                    }


                    let mut oldsigset = [0u32; 32];
                    if arg3 != 0 {
                        for i in 0..32 {
                            oldsigset[i as usize] = memory.read_word(arg3 + i * 4);
                        }
                    }

                    let result = unsafe {
                        ::libc::sigprocmask(
                            how as ::libc::c_int,
                            if arg2 == 0 {
                                0 as *const ::libc::sigset_t
                            } else {
                                sigset.as_ptr() as *const ::libc::sigset_t
                            },
                            if arg3 == 0 {
                                0 as *mut ::libc::sigset_t
                            } else {
                                oldsigset.as_mut_ptr() as *mut ::libc::sigset_t
                            },
                        )
                    };

                    if arg3 != 0 {
                        for i in 0..32 {
                            memory.write_word(arg3 + i * 4, sigset[i as usize]);
                        }
                    }

                    check_error(result)
                }
                SyscallO32::NRRt_sigaction => {
                    let signum = arg1;
                    itrace!(
                        "RT_SIGACTION signal={} &act=0x{:x} &oldact=0x{:x}",
                        signum,
                        arg2,
                        arg3
                    );

                    // read sigaction in argument
                    // struct sigaction is 140bytes wide
                    const SIGACTION_SIZE: usize = ::std::mem::size_of::<MipsSigaction>() / 4;
                    let mut sigaction = [0u32; SIGACTION_SIZE];
                    if arg2 != 0 {
                        for i in 0..SIGACTION_SIZE as u32 {
                            sigaction[i as usize] = memory.read_word(arg2 + i * 4);
                        }
                    }

                    // save it for futuru use and obtain previous value
                    let mut oldsigaction = self.sigactions
                        .insert(signum, MipsSigaction::from(sigaction));

                    let result = self.reannounce_signal_handlers(signum);

                    // create old sigaction. First option is old stored, then modified result from system, then 0array as a fallback
                    let oldsigaction: [u32; SIGACTION_SIZE] = if let Some(oldsigaction) = oldsigaction {
                        oldsigaction.into()
                    } else if let Ok(oldsigaction) = result {
                        MipsSigaction::from(oldsigaction).into()
                    } else {
                        [0u32; SIGACTION_SIZE]
                    };

                    // write it back into memory
                    if arg3 != 0 {
                        for i in 0..SIGACTION_SIZE as u32 {
                            memory.write_word(arg3 + i * 4, oldsigaction[i as usize]);
                        }
                    }

                    if let Err(e) = result {
                        Err(e)
                    } else {
                        Ok(0)
                    }
                }
                SyscallO32::NRGetuid => {
                    if self.config.sys_fake_root {
                        itrace!("GETUID (faked)");
                        Ok(0)
                    } else {
                        itrace!("GETUID (real)");
                        check_error(unsafe { ::libc::getuid() })
                    }
                }
                SyscallO32::NRGeteuid => {
                    if self.config.sys_fake_root {
                        itrace!("GETEUID (faked)");
                        Ok(0)
                    } else {
                        itrace!("GETEUID (real)");
                        check_error(unsafe { ::libc::getuid() })
                    }
                }
                SyscallO32::NRGetgid => {
                    if self.config.sys_fake_root {
                        itrace!("GETGID (faked)");
                        Ok(0)
                    } else {
                        itrace!("GETGID (real)");
                        check_error(unsafe { ::libc::getgid() })
                    }
                }
                SyscallO32::NRGetegid => {
                    if self.config.sys_fake_root {
                        itrace!("GETEGID (faked)");
                        Ok(0)
                    } else {
                        itrace!("GETEGID (real)");
                        check_error(unsafe { ::libc::getegid() })
                    }
                }
                SyscallO32::NRGetpid => {
                    itrace!("GETPID");

                    check_error(unsafe { ::libc::getpid() })
                }
                SyscallO32::NRGetppid => {
                    itrace!("GETPPID");

                    //check_error(unsafe { ::libc::getppid() })
                    Ok(0x47)
                }
                SyscallO32::NRUname => {
                    itrace!("UNAME addr=0x{:x}", arg1);

                    let utsname = memory.translate_address_mut(arg1) as *mut ::libc::utsname;
                    check_error(unsafe { ::libc::uname(utsname) })
                    //memory.write_block(arg1, "Linux\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0buildroot\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\04.11.3\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0#1 SMP Sun Mar 4 03:29:34 UTC 2018\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0#1 SMP Sun Mar 4 03:29:34 UTC 2018\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0(none)\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0".as_bytes());
                    //Ok(0)
                }
                SyscallO32::NRWait4 => {
                    itrace!("WAIT4");

                    let pid = arg1;
                    let wstatus = memory.translate_address_mut(arg2);
                    let options = arg3;
                    let mut rusage = [0u64; 18];

                    let respis = unsafe {
                        ::libc::wait4(
                            pid as i32,
                            wstatus as *mut i32,
                            options as i32,
                            rusage.as_mut_ptr() as *mut ::libc::rusage,
                        )
                    };

                    // write back rusage
                    if arg4 != 0 {
                        memory.write_word(arg3 + 0 * 4, rusage[0] as u32);
                        memory.write_word(arg3 + 1 * 4, rusage[1] as u32);
                        for i in 1..18 {
                            memory.write_word(arg3 + 2 * i * 4, (rusage[i as usize]) as u32);
                            memory.write_word(
                                arg3 + 2 * i * 4 + 1 * 4,
                                (rusage[i as usize] >> 32) as u32,
                            );
                        }
                    }

                    check_error(respis)
                }
                SyscallO32::NRStat64 => {
                    let file =
                        unsafe { CStr::from_ptr(memory.translate_address_mut(arg1) as *mut i8) };
                    itrace!("STAT64 file={:?} struct_at=0x{:08x}", file, arg2,);
                    let res = ::nix::sys::stat::stat(file);
                    if let Ok(stat) = res {
                        let addr = arg2;
                        let repr = translate_stat(stat);

                        for i in 0..40 {
                            memory.write_word(addr + i * 4, repr[i as usize]);
                        }

                        Ok(0)
                    } else {
                        check_error(-1)
                    }
                }
                SyscallO32::NRLstat64 => {
                    let file =
                        unsafe { CStr::from_ptr(memory.translate_address_mut(arg1) as *mut i8) };
                    itrace!("LSTAT64 file={:?} struct_at=0x{:08x}", file, arg2,);
                    let res = ::nix::sys::stat::lstat(file);
                    if let Ok(stat) = res {
                        let addr = arg2;
                        let repr = translate_stat(stat);

                        for i in 0..40 {
                            memory.write_word(addr + i * 4, repr[i as usize]);
                        }

                        Ok(0)
                    } else {
                        check_error(-1)
                    }
                }
                SyscallO32::NRFstat64 | SyscallO32::NRFstat => {
                    itrace!("FSTAT64 fd={} struct_at=0x{:08x}", arg1, arg2,);
                    let res = ::nix::sys::stat::fstat(arg1 as ::libc::c_int);
                    if let Ok(stat) = res {
                        let addr = arg2;
                        let repr = translate_stat(stat);

                        for i in 0..40 {
                            memory.write_word(addr + i * 4, repr[i as usize]);
                        }

                        Ok(0)
                    } else {
                        check_error(-1)
                    }
                }
                SyscallO32::NRGettid => {
                    itrace!("GETTID");

                    check_error(unsafe { ::libc::syscall(::libc::SYS_gettid) })
                }
                SyscallO32::NRFork => {
                    itrace!("FORK");

                    check_error(unsafe { ::libc::fork() })
                }
                SyscallO32::NRExecve => {
                    let filename =
                        unsafe { CStr::from_ptr(memory.translate_address(arg1) as *const i8) };

                    fn f(i: u32, memory: &Memory) -> Vec<*const i8> {
                        (i..)
                            .filter(|i| i % 4 == 0)
                            .map(|i| memory.read_word(i))
                            .take_while(|i| *i != 0)
                            .map(|a| memory.translate_address(a) as *const i8)
                            .chain(::std::iter::once(0 as *const i8))
                            .collect()
                    }

                    let argv = f(arg2, memory);
                    let envp = f(arg3, memory);

                    let argv_str: Vec<&CStr> = argv.iter().take_while(|a| *a != &(0 as *const i8)).map(|a| unsafe {CStr::from_ptr(*a)}).collect();
                    let envp_str: Vec<&CStr> = envp.iter().take_while(|a| *a != &(0 as *const i8)).map(|a| unsafe {CStr::from_ptr(*a)}).collect();

                    itrace!(
                        "EXECVE filename={:?} argv={:?} envp={:?}",
                        filename,
                        argv_str,
                        envp_str
                    );

                    check_error(unsafe {
                        ::libc::execve(filename.as_ptr(), argv.as_ptr(), envp.as_ptr())
                    })
                }
                SyscallO32::NRIoctl => {
                    itrace!("IOCTL a0={} a1=0x{:x} a2=0x{:x}", arg1, arg2, arg3);

                    let fd = arg1;
                    if self.config.sys_block_ioctl_on_stdio && fd < 3 {
                        warn!("IOCTL ignored - manipulating with FD<=2. Returning success.");
                        Ok(0)
                    } else if self.config.sys_ioctl_fail_always {
                        warn!("IOCTL forced to fail! Returning EINVAL.");
                        Err(Error::from_raw_os_error(::libc::EINVAL))
                    } else {
                        warn!("Syscall IOCTL might not work as expected due to struct translation missing and probably impossible.");
                        check_error(unsafe {
                            ::libc::ioctl(arg1 as i32, arg2 as u64, memory.translate_address(arg3))
                        })
                    }
                }
                SyscallO32::NRFutex => {
                    itrace!("FUTEX");
                    let uaddr_ptr = memory.translate_address(arg1);
                    let futex_op = arg2;
                    let val = arg3;
                    let timeout_ptr = arg4;
                    let uaddr2_ptr =
                        memory.translate_address(read_argument_from_memory(5, registers, memory));
                    let val3 = read_argument_from_memory(6, registers, memory);

                    let timeout = ::libc::timespec {
                        tv_sec: memory.read_word(timeout_ptr) as i64,
                        tv_nsec: memory.read_word(timeout_ptr + 4) as i64,
                    };

                    check_error(unsafe {
                        ::libc::syscall(
                            ::libc::SYS_futex,
                            uaddr2_ptr,
                            futex_op,
                            val,
                            &timeout,
                            uaddr_ptr,
                            val3,
                        )
                    })
                }
                SyscallO32::NRClock_gettime => {
                    itrace!("CLOCK_GETTIME");

                    let clockid = arg1;
                    let mut time = ::libc::timespec {
                        tv_sec: 0,
                        tv_nsec: 0,
                    };
                    let res = unsafe {
                        ::libc::clock_gettime(clockid as i32, &mut time as *mut ::libc::timespec)
                    };

                    memory.write_word(arg2, time.tv_sec as u32);
                    memory.write_word(arg2 + 4, time.tv_nsec as u32);

                    check_error(res)
                }
                SyscallO32::NRDup2 => {
                    itrace!("DUP2 oldfd={} newfd={}", arg1, arg2);

                    check_error(unsafe { ::libc::dup2(arg1 as i32, arg2 as i32) })
                }
                SyscallO32::NROpen => {
                    let mut flags = arg2 as i32;

                    // this here drops flag FASYNC. On MIPS, it means LARGEFILES
                    // and therefore is meaningless on 64bit systems
                    if flags & 0x2000 == 0x2000 {
                        flags ^= 0x2000;
                    }

                    let (file, res) = unsafe {
                        (
                            CStr::from_ptr(memory.translate_address_mut(arg1) as *mut i8),
                            ::libc::open(
                                memory.translate_address_mut(arg1) as *mut i8,
                                flags,
                                arg3,
                            ),
                        )
                    };
                    itrace!(
                        "OPEN file={:?} flags=0x{:08x} mode=0x{:x} res_fd={}",
                        file,
                        arg2,
                        arg3,
                        res
                    );
                    check_error(res)
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
                    itrace!("WRITEV");

                    let fd = arg1 as i32;
                    // This branch translates 32bit iovec array to native one and directly calls the kernel
                    let mut iovec = translate_iovec_libc(arg2, arg3, memory);

                    check_error(unsafe {
                        ::libc::writev(
                            fd as i32,
                            iovec.as_slice().as_ptr() as *const ::libc::iovec,
                            arg3 as i32,
                        )
                    })
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
                                let slice =
                                    &buffer.as_slice()[already_written..already_written + l];
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
                    let whence = read_argument_from_memory(5, registers, memory);

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

                    if self.config.sys_fake_root_directory && buf_size >= 6 {
                        itrace!("GETCWD (faked) ptr=0x{:x}", buf_addr);
                        memory.write_byte(buf_addr + 0, '/' as u32);
                        memory.write_byte(buf_addr + 1, 'r' as u32);
                        memory.write_byte(buf_addr + 2, 'o' as u32);
                        memory.write_byte(buf_addr + 3, 'o' as u32);
                        memory.write_byte(buf_addr + 4, 't' as u32);
                        memory.write_byte(buf_addr + 5, 0);
                        Ok(buf_addr)
                    } else {
                        let (cwd, res) = unsafe {
                            (
                                CStr::from_ptr(memory.translate_address_mut(buf_addr) as *mut i8),
                                ::libc::getcwd(
                                    memory.translate_address_mut(buf_addr) as *mut i8,
                                    buf_size as usize,
                                ) as usize,
                            )
                        };

                        itrace!(
                            "GETCWD result_cwd={:?} real_ptr=0x{:x} emu_ptr=0x{:x}",
                            cwd,
                            res,
                            buf_addr
                        );
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
                        Err(_) => panic!("Weird time! UNIX epoch is in the future?"),
                    };

                    if tloc_ptr != 0 {
                        memory.write_word(tloc_ptr, seconds as u32);
                    }

                    Ok(seconds as u32)
                }
                SyscallO32::NRSetgid => {
                    itrace!("SETGID gid={}", arg1);
                    check_error(unsafe { ::libc::setgid(arg1 as ::libc::gid_t) })
                }
                SyscallO32::NRSetuid => {
                    itrace!("SETUID uid={}", arg1);
                    check_error(unsafe { ::libc::setuid(arg1 as ::libc::uid_t) })
                }
                SyscallO32::NRChdir => {
                    let dir =
                        unsafe { CStr::from_ptr(memory.translate_address(arg1) as *const i8) };
                    itrace!("CHDIR {:?}", dir);
                    check_error(unsafe { ::libc::chdir(dir.as_ptr()) })
                }
                SyscallO32::NRMmap2 => {
                    let addr = arg1;
                    let len = arg2;

                    itrace!("MMAP2 addr=0x{:x} len={}", arg1, len);

                    if len == 0 {
                        Err(Error::from_raw_os_error(::libc::EINVAL))
                    } else {
                        panic!("MMAP2 syscall is not implemented!");
                    }
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

    fn reannounce_signal_handlers(&self, signum: u32) -> Result<SigAction, Error> {
        let sigact = self.sigactions.get(&signum);
        let action;

        if let Some(sigact) = sigact {
            let mut ss = [0u32; 32];
            ss[0] = sigact.sa_mask[0];
            ss[1] = sigact.sa_mask[1];
            ss[2] = sigact.sa_mask[2];
            ss[3] = sigact.sa_mask[3];
            let sigset = unsafe { ::std::mem::transmute(ss) };
            let mut flags = translate_signal_flags(sigact.sa_flags);

            action = SigAction::new(
                SigHandler::SigAction(signal_handler),
                SaFlags::from_bits(flags).expect("invalid sigaction flags"),
                sigset,
            );
        } else {
            action = SigAction::new(SigHandler::SigDfl, SaFlags::empty(), SigSet::empty());
        }

        let res = unsafe {
            sigaction(
                Signal::from_c_int(signum as i32).expect("invalid signal"),
                &action,
            )
        };
        if let Ok(oldact) = res {
            Ok(oldact)
        } else {
            Err(check_error(-1i32).err().unwrap())
        }
    }
}

extern "C" fn signal_handler(
    signal: ::libc::c_int,
    _: *mut ::libc::siginfo_t,
    _: *mut ::libc::c_void,
) {
    println!("ohhhhhh prisel signal {}", signal)
}
