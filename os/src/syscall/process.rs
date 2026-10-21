//! Process management syscalls
use crate::{
    config::PAGE_SIZE,
    mm::{translated_byte_buffer, PageTable, VirtAddr},
    task::{
        change_program_brk, current_get_systemcall_count, current_map_new_area, current_unmap,
        current_user_token, exit_current_and_run_next, suspend_current_and_run_next,
    },
    timer::get_time,
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
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let time_us = get_time();
    let dst = translated_byte_buffer(current_user_token(), _ts as *const u8, 16);

    let tv = TimeVal {
        sec: time_us / 1_000_000,
        usec: time_us,
    };

    let src = unsafe { core::slice::from_raw_parts(core::ptr::addr_of!(tv) as *const u8, 16) };

    for (i, d) in dst.into_iter().flatten().enumerate() {
        *d = src[i as usize];
    }

    0
}

/// TODO: Finish sys_trace to pass testcases
/// HINT: You might reimplement it with virtual memory management.
pub fn sys_trace(_trace_request: usize, _id: usize, _data: usize) -> isize {
    trace!("kernel: sys_trace");

    let page_table = PageTable::from_token(current_user_token());
    let vpn = VirtAddr(_id).floor();
    let offset = VirtAddr(_id).page_offset();

    let buf;
    let pte;

    let t = _id as isize;

    if (t << 25) >> 25 != t {
        return -1;
    }

    if let Some(_pte) = page_table.translate(vpn) {
        pte = _pte;
        buf = pte.ppn().get_bytes_array();
    } else {
        return -1;
    }

    match _trace_request {
        0 => {
            if pte.readable() {
                buf[offset] as isize
            } else {
                -1
            }
        }
        1 => {
            if pte.writable() {
                buf[offset] = _data as u8;
                0
            } else {
                -1
            }
        }
        2 => current_get_systemcall_count(_id) as isize,
        _ => -1,
    }
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    // trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");
    if _start & (PAGE_SIZE - 1) != 0 || _port & 0x7 == 0 || _port & !0x7 != 0 {
        return -1;
    }

    current_map_new_area(_start, _len, _port)
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    // trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");
    if _start & (PAGE_SIZE - 1) != 0 {
        return -1;
    }

    current_unmap(_start, _len)
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
