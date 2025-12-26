#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use windows::search_for_window;

#[cfg(all(target_family = "unix", not(target_os = "macos")))]
mod unix;
#[cfg(all(target_family = "unix", not(target_os = "macos")))]
pub use unix::search_for_window;

#[cfg(all(target_os = "macos"))]
mod macos;
#[cfg(all(target_os = "macos"))]
pub use macos::search_for_window;

fn kill_proc(pid: u32, sys: &sysinfo::System) -> bool {
    for (proc_pid, proc) in sys.processes() {
        if proc_pid.as_u32() == pid {
            _ = proc.kill();
            return true;
        }
    }
    false
}
