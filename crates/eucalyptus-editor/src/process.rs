//! Functions related to processes and PIDs that aren't part of the std library.

use std::io;
use std::process::Command;

#[cfg(unix)]
pub fn kill_process(pid: u32) -> io::Result<()> {
    let status = Command::new("kill")
        .arg("-15")  // SIGTERM
        .arg(pid.to_string())
        .status()?;

    if !status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to kill process {}", pid)
        ));
    }

    Ok(())
}

#[cfg(windows)]
pub fn kill_process(pid: u32) -> io::Result<()> {
    let status = Command::new("taskkill")
        .arg("/PID")
        .arg(pid.to_string())
        .status()?;

    if !status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to kill process {}", pid)
        ));
    }

    Ok(())
}