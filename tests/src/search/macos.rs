use core_foundation::{array::CFArray, base::TCFType, dictionary::CFDictionary, number::CFNumber};
use core_graphics::window::{kCGNullWindowID, kCGWindowListOptionAll, CGWindowListCopyWindowInfo};
use std::os::raw::c_int;

use crate::search::kill_proc;

pub fn search_for_window(pid: u32, sys: &sysinfo::System) -> bool {
    // SAFETY: Quartz returns a retained CFArray
    let window_list =
        unsafe { CGWindowListCopyWindowInfo(kCGWindowListOptionAll, kCGNullWindowID) };

    if window_list.is_null() {
        return false;
    }

    // Wrap raw CFArrayRef
    let windows = unsafe { CFArray::<CFDictionary>::wrap_under_create_rule(window_list) };

    for window in windows.iter() {
        // kCGWindowOwnerPID is a CFString constant
        let pid_value = window.find(unsafe { &core_graphics::window::kCGWindowOwnerPID });

        if let Some(pid_value) = pid_value {
            // Value is a CFNumber
            let pid_number: CFNumber = unsafe { CFNumber::wrap_under_get_rule(pid_value) };

            if let Some(pid) = pid_number.to_i32() {
                if pid == target_pid {
                    kill_proc(pid, sys);
                    return true;
                }
            }
        }
    }

    false
}
