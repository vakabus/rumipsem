use cpu::control::CPUFlags;
use cpu::event::CPUEvent;
use cpu::registers::A3;
use cpu::registers::RegisterFile;
use cpu::registers::V0;
use cpu::registers::STACK_POINTER;
/// List of MIPS syscall numbers can be found here:
/// https://github.com/torvalds/linux/blob/master/arch/mips/include/uapi/asm/unistd.h
use memory::Memory;
use num_traits::cast::ToPrimitive;
use std::ffi::CStr;
use std::io::Error;
use std::time::SystemTime;
use syscall_numbers::*;

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

pub fn eval_syscall<T, S>(
    _inst: u32,
    registers: &mut RegisterFile<T, S>,
    memory: &mut Memory,
    flags: &CPUFlags,
) -> CPUEvent
where T: Fn(u32,u32), S: Fn(u32,u32) {
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
                if flags.fake_root {
                    itrace!("GETGID (faked)");
                    Ok(0)
                } else {
                    itrace!("GETGID (real)");
                    check_error(unsafe { ::libc::getgid() })
                }
            }
            SyscallO32::NRGetegid => {
                if flags.fake_root {
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
            SyscallO32::NRStat64 => {
                //panic!("This syscall was disabled!");

                //FIXME struct translation ??

                let (file, res) = unsafe {
                    (
                        CStr::from_ptr(memory.translate_address_mut(arg1) as *mut i8),
                        ::libc::stat(
                            memory.translate_address_mut(arg1) as *mut i8,
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
                warn!("Struct in argument was not translated.");
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
                } else {
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
                        ::libc::open(memory.translate_address_mut(arg1) as *mut i8, flags, arg3),
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
