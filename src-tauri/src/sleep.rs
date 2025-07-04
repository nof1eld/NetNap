#[cfg(target_os = "windows")]
pub fn sleep() {
    use std::process::Command;
    Command::new("rundll32.exe")
        .args(["powrprof.dll,SetSuspendState", "0", "1", "0"])
        .status()
        .expect("Failed to sleep");
}

#[cfg(target_os = "linux")]
pub fn sleep() {
    use std::process::Command;
    Command::new("systemctl")
        .arg("suspend")
        .status()
        .expect("Failed to sleep");
}


