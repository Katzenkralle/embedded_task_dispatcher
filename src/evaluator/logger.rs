extern crate custom_error;
use std::path::PathBuf;

use std::sync::mpsc::Receiver;
use std::io::{self, BufRead, Write};
use std::fs;






#[derive(Clone, Copy, Debug)]
pub enum LogLevel {
    Debug = 0,
    Info = 1,
    Warning = 2,
    Error = 3,
}
impl LogLevel {
    pub fn from_str(level: &str) -> Option<LogLevel> {
        match level.to_uppercase().as_str() {
            "DEBUG" => Some(LogLevel::Debug),
            "INFO" => Some(LogLevel::Info),
            "WARNING" => Some(LogLevel::Warning),
            "ERROR" => Some(LogLevel::Error),
            _ => None,
        }
    }
    
}

pub(crate) enum LoggerCommand {
    Log(String, LogLevel),
    ChangeLogLevel(LogLevel),
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warning => write!(f, "WARNING"),
            LogLevel::Error => write!(f, "ERROR"),
        }
    }
    
}

pub(super) fn logger(rx: Receiver<LoggerCommand>, log_file: Option<PathBuf>) {
    let stdout = io::stdout(); // Get the stdout handle
    let mut handle = stdout.lock(); // Lock it here inside the loop
    let mut log_level = LogLevel::Debug; // Default log level
    loop {
        let (to_write, write_level) = match rx.recv() {
            Ok(LoggerCommand::Log(msg, msg_level)) => {
                (format!("{}: {}", msg_level, msg), msg_level)
            }
            Ok(LoggerCommand::ChangeLogLevel(new_level)) => {
                log_level = new_level; // Change the log level
                (format!("Log level changed to: {}", new_level), LogLevel::Info)
            }
            Err(e) => {
                (format!("Error receiving log message: {}", e), LogLevel::Error)
            } 
        };
        if to_write != "" {
            if log_level as u8 <= write_level as u8 {
                writeln!(handle, "{}", to_write).unwrap();
                if let Some(ref path) = log_file {
                    let file = fs::OpenOptions::new()
                        .open(path);
                    if file.is_err() {
                        writeln!(handle, "ERROR: Could not write to log file").unwrap();
                    }


                    let reader: io::BufReader<_> = io::BufReader::new(file.expect(""));
                    let file = match  reader.lines().count() > 1000 {
                        true => fs::OpenOptions::new()
                            .write(true)
                            .truncate(true)
                            .open(path),
                        false => fs::OpenOptions::new()
                            .append(true)
                            .create(true)
                            .open(path),
                    };

                    let _ = writeln!(file.expect(""), "{}", to_write);
                    }
                }
            }
        }
    }
