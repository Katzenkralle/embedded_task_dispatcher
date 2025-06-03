use custom_error::custom_error;
use serde::Serialize;
use serde::ser::Serializer;
use serde_json::{json, to_vec};
use std::collections::HashMap;
use std::io::Write;
use std::os::unix::net::UnixStream;
use std::path::Path;

#[macro_export]
macro_rules! write_lcd {
    ($x:expr_2021) => {
        LCDcommand{
            cmd: LCDProgramm::Buffer,
            args: Some(HashMap::from([
                ("text".to_string(), LCDArg::String($x.to_string())),
                ("directly".to_string(), LCDArg::Bool(true)),
            ])),
        }
    };
}
#[macro_export]
macro_rules! move_lcd {
    ($x:expr_2021, $y:expr_2021) => {
        LCDcommand{
            cmd: LCDProgramm::Move,
            args: Some(HashMap::from([
                ("x".to_string(), LCDArg::Int($x)),
                ("y".to_string(), LCDArg::Int($y)),
            ])),
        }
    };
}
#[macro_export]
macro_rules! clear_lcd {
    () => {
        LCDcommand{
            cmd: LCDProgramm::Clear,
            args: None,
        }
    };
}
#[macro_export]
macro_rules! toggle_bcklight {
    ($x:expr_2021) => {
        LCDcommand{
            cmd: LCDProgramm::Bcklight,
            args: Some(HashMap::from([
                ("state".to_string(), LCDArg::Bool($x)),
            ])),
        }
    };
}

custom_error! {pub LCDError
    DriverError{comment:&'static str} = "{comment}"
}


#[derive(Serialize)]
pub enum LCDProgramm {
    Buffer,
    Clear,
    Move,
    Bcklight,
    CursorMode,
    ShiftDisplay,
    Home,
    Write,
}

pub enum LCDArg {
    String(String),
    Int(i128),
    Bool(bool)
}

impl Serialize for LCDArg {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            LCDArg::String(s) => serializer.serialize_str(s),
            LCDArg::Int(i) => serializer.serialize_i128(*i),
            LCDArg::Bool(b) => serializer.serialize_bool(*b),
        }
    }
}

#[derive(Serialize)]
pub struct LCDcommand {
    pub cmd: LCDProgramm,
    pub args: Option<HashMap<String, LCDArg>>,
}

#[derive(Debug)]
pub struct LCDdriver {
    pub(crate) driver_stream: UnixStream,
    pub(crate) path: std::path::PathBuf,
}

impl LCDdriver {
    pub(crate) fn new(socket_path: &Path, clear: bool) -> Result<LCDdriver, LCDError> {
        let mut driver = LCDdriver {
            driver_stream: LCDdriver::connect(socket_path)?,
            path: socket_path.to_path_buf(),
        };

        if clear {
            driver.exec(LCDcommand {
                cmd: LCDProgramm::Clear,
                args: None,
            })?;
            driver.exec(LCDcommand {
                cmd: LCDProgramm::Home,
                args: None,
            })?;
        }
        Ok(driver)
    }
    fn connect(socket_path: &Path) -> Result<UnixStream, LCDError> {
        return UnixStream::connect(socket_path)
            .map_err(|_| LCDError::DriverError { comment: "Could not reconnect to driver!" });
    }

    pub fn exec(&mut self, command: LCDcommand) -> Result<(), LCDError> {
        let mut json_command = to_vec(&json!(command))
            .map_err(|_| LCDError::DriverError { comment: "Serialization failed" })?;
        json_command.push('\n' as u8);
        for _ in 0..2 {
            let result = self.driver_stream
                .write_all(&json_command)
                .map_err(|e| {let msg = format!("Could not write to socket {:?}", e.to_string()); 
                    return LCDError::DriverError { comment: Box::leak(msg.into_boxed_str()) }});
            if let Ok(Some(err)) = self.driver_stream.take_error() {
                let msg = format!("Unrecoverable Socket error: {:?}", err);
                return Err(LCDError::DriverError { comment: Box::leak(msg.into_boxed_str()) });
            }
            if result.is_ok() {
                break;
            }
            // If write fails, try to reconnect
            self.driver_stream = LCDdriver::connect(&self.path)?;
        }
        self.driver_stream.flush()
                .map_err(|e| {let msg = format!("Could not flush socket {:?}", e.to_string()); 
                    return LCDError::DriverError { comment: Box::leak(msg.into_boxed_str()) }})
        
    }
}
