//! Process management syscalls
use crate::{
    config::PAGE_SIZE, mm::{find_pte, translated_byte_buffer, VirtAddr}, task::{change_program_brk, contains_vaddr, current_user_token, exit_current_and_run_next, get_syscall_time, mmap, suspend_current_and_run_next, ummap}, timer::{get_time_us, MICRO_PER_SEC}
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let us = get_time_us();
    let cur_token = current_user_token();
    let dsts = translated_byte_buffer(cur_token, ts as *const u8, core::mem::size_of::<TimeVal>());
    let src = &TimeVal {
        sec: us / MICRO_PER_SEC,
        usec: us % MICRO_PER_SEC,
    } as *const TimeVal;
    for (id, dst) in dsts.into_iter().enumerate() {
        let len = dst.len();
        unsafe {
            dst.copy_from_slice(core::slice::from_raw_parts(
                src.wrapping_byte_add(id * len) as *const u8, len)
            );
        }
    }
    0
}

/// TODO: Finish sys_trace to pass testcases
/// HINT: You might reimplement it with virtual memory management.
pub fn sys_trace(trace_request: usize, id: usize, data: usize) -> isize {
    trace!("kernel: sys_trace");
    match trace_request {
        0 => {
            if !contains_vaddr(id) {
                return -1;
            }
            let pte = find_pte(current_user_token(), id as *const u8);
            if let Some(pte) = pte {
                debug!("readale: {}, writeable: {}", pte.readable(), pte.writable());
                if !pte.readable() {
                    return -1
                } else {
                    let ppn = pte.ppn();
                    debug!("ppn: {:?}", ppn);
                    let pa = ((ppn.0 << 12) | (id & 0xfff))as usize as *const u8;
                    debug!("pa: {:?}", pa);
                    return unsafe {
                        pa.read() as isize
                    }
                }
            }
            -1
        }
        1 => {
            if !contains_vaddr(id) {
                return -1;
            }
            let pte = find_pte(current_user_token(), id as *const u8);
            if let Some(pte) = pte {
                if!pte.writable() {
                    return -1
                }
                let ppn = pte.ppn();
                let pa = ((ppn.0 << 12) | (id & 0xfff) )as usize as *mut u8;
                unsafe {
                    *pa = data as u8;
                }
                return 0;
            }
            -1
        }
        2 => {
            get_syscall_time(id)
        }
        _ => {
            -1
        }
    }
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(start: usize, len: usize, port: usize) -> isize {
    trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");
    if port & !0b111 != 0 {
        return -1;
    }
    if port & 0b111 == 0{
        return -1;
    }
    if len == 0 || start % PAGE_SIZE != 0 {
        return -1;
    }
    mmap(start, len, port)
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(start: usize, len: usize) -> isize {
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");
    if start % PAGE_SIZE != 0 {
        return -1;
    }
    let start_va: VirtAddr = start.into();
    let end_va: VirtAddr = (start+len).into();
    if  !start_va.aligned() || !end_va.aligned(){
        return -1;
    }
    if len == 0 || start % PAGE_SIZE != 0 {
        return -1;
    }
    ummap(start, len)
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
