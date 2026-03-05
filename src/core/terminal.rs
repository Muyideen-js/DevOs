use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::models::TerminalEntry;

/// Run a shell command in the given working directory.
/// Returns a TerminalEntry with the output after completion.
pub fn run_command_sync(cmd: &str, cwd: &str) -> TerminalEntry {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    if parts.is_empty() {
        return TerminalEntry {
            command: cmd.to_string(),
            output: "Empty command".to_string(),
            is_error: true,
            running: false,
        };
    }

    // On Windows use cmd /C, on Unix use sh -c
    let result = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", cmd])
            .current_dir(cwd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
    } else {
        Command::new("sh")
            .args(["-c", cmd])
            .current_dir(cwd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
    };

    match result {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let combined = if stderr.is_empty() {
                stdout
            } else if stdout.is_empty() {
                stderr.clone()
            } else {
                format!("{}\n{}", stdout, stderr)
            };
            TerminalEntry {
                command: cmd.to_string(),
                output: combined,
                is_error: !output.status.success(),
                running: false,
            }
        }
        Err(e) => TerminalEntry {
            command: cmd.to_string(),
            output: format!("Failed to execute: {}", e),
            is_error: true,
            running: false,
        },
    }
}

/// Spawn a command asynchronously and stream output into a shared buffer.
/// Returns the child process wrapped in Arc<Mutex<>> for kill support.
pub fn run_command_async(
    cmd: &str,
    cwd: &str,
    output_buffer: Arc<Mutex<String>>,
    is_running: Arc<Mutex<bool>>,
) -> Option<Arc<Mutex<std::process::Child>>> {
    let child_result = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", cmd])
            .current_dir(cwd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    } else {
        Command::new("sh")
            .args(["-c", cmd])
            .current_dir(cwd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    };

    match child_result {
        Ok(mut child) => {
            let stdout = child.stdout.take();
            let stderr = child.stderr.take();
            let child = Arc::new(Mutex::new(child));
            let child_clone = child.clone();

            // Stream stdout
            if let Some(stdout) = stdout {
                let buf = output_buffer.clone();
                let running = is_running.clone();
                thread::spawn(move || {
                    let reader = BufReader::new(stdout);
                    for line in reader.lines() {
                        if let Ok(line) = line {
                            if let Ok(mut b) = buf.lock() {
                                b.push_str(&line);
                                b.push('\n');
                            }
                        }
                    }
                    if let Ok(mut r) = running.lock() {
                        *r = false;
                    }
                });
            }

            // Stream stderr
            if let Some(stderr) = stderr {
                let buf = output_buffer.clone();
                thread::spawn(move || {
                    let reader = BufReader::new(stderr);
                    for line in reader.lines() {
                        if let Ok(line) = line {
                            if let Ok(mut b) = buf.lock() {
                                b.push_str("[stderr] ");
                                b.push_str(&line);
                                b.push('\n');
                            }
                        }
                    }
                });
            }

            Some(child_clone)
        }
        Err(_) => None,
    }
}

/// Kill a running child process.
pub fn kill_process(child: &Arc<Mutex<std::process::Child>>) -> Result<(), String> {
    let mut child = child.lock().map_err(|e| format!("Lock error: {}", e))?;
    child.kill().map_err(|e| format!("Failed to kill process: {}", e))
}
