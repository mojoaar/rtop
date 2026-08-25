use std::mem::size_of;

pub fn process_cpu_time(pid: u32) -> Option<u64> {
    let mut info: libc::proc_taskinfo = unsafe { std::mem::zeroed() };
    let ret = unsafe {
        libc::proc_pidinfo(
            pid as libc::c_int,
            libc::PROC_PIDTASKINFO,
            0,
            &mut info as *mut libc::proc_taskinfo as *mut libc::c_void,
            size_of::<libc::proc_taskinfo>() as libc::c_int,
        )
    };
    if ret <= 0 {
        return None;
    }
    let nanos = info.pti_total_user.saturating_add(info.pti_total_system);
    Some(nanos / 1_000_000_000)
}
