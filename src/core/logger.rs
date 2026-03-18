use std::fs::OpenOptions;
use std::io::Write;
use chrono::Local;

pub fn info(msg: &str) {
    append_log("INFO", msg);
}

pub fn error(msg: &str) {
    append_log("ERROR", msg);
}

fn append_log(level: &str, msg: &str) {
    if let Some(mut path) = dirs::home_dir() {
        path.push(".devos");
        let _ = std::fs::create_dir_all(&path);
        path.push("devos.log");
        
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
        let log_line = format!("[{}] [{}] {}\n", timestamp, level, msg);
        
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
            let _ = file.write_all(log_line.as_bytes());
        }
    }
}
