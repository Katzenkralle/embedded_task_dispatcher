extern crate custom_error;
use core::fmt;
use crate::unix_now;
use rppal::gpio::{InputPin, OutputPin};


#[derive(PartialEq, Clone, Debug)]
pub enum StateType {
    Str(String),
    Bool(bool),
    Int(f64),
}
impl StateType {
    pub fn as_default(&self) -> StateType {
        match self {
            StateType::Str(_) => StateType::Str("".to_string()),
            StateType::Bool(_) => StateType::Bool(false),
            StateType::Int(_) => StateType::Int(0.),
        }
    }

    pub fn as_bool(&self) -> bool {
        match self {
            StateType::Str(s) => *s != "",
            StateType::Bool(b) => *b,
            StateType::Int(i) => *i != 0.,
        }
    }

    pub fn as_int(&self) -> f64 {
        match self {
            StateType::Str(s) => s.parse::<f64>().unwrap_or(0.),
            StateType::Bool(b) => if *b { 1. } else { 0. },
            StateType::Int(i) => *i,
        }
    }
}

impl fmt::Display for StateType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StateType::Str(s) => write!(f, "{}", s),
            StateType::Bool(b) => write!(f, "{}", b),
            StateType::Int(i) => write!(f, "{:?}", i),
        }
    }
    
}


pub trait PinHandler {
    fn current_state(&self) -> bool;
    fn last_state(&self) -> bool;
    fn last_change(&self) -> f64;
}

#[derive(Debug)]
pub struct InputPinHandler {
    pub(crate) handler: InputPin,
    pub current_state: bool,
    pub last_state: bool,
    pub last_change: f64,
}

impl PinHandler for InputPinHandler {
    fn current_state(&self) -> bool {
        self.current_state
    }

    fn last_state(&self) -> bool {
        self.last_state
    }

    fn last_change(&self) -> f64 {
        self.last_change
    }
    
}

impl fmt::Display for InputPinHandler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Pin: {}, Current state: {}, Last state: {}, Last change: {}",
            self.handler.pin(),
            self.current_state,
            self.last_state,
            self.last_change
        )
    }
    
}


#[derive(Debug)]
pub struct OutputPinHandler {
    pub(crate) handler: OutputPin,
    pub current_state: bool,
    pub last_state: bool,
    pub last_change: f64,
}

impl PinHandler for OutputPinHandler {
    fn current_state(&self) -> bool {
        self.current_state
    }

    fn last_state(&self) -> bool {
        self.last_state
    }

    fn last_change(&self) -> f64 {
        self.last_change
    }
}

impl OutputPinHandler{
    pub fn change_state(&mut self, new_state: bool) {
        if self.current_state != new_state {
            self.last_state = self.current_state;
            self.current_state = new_state;
            self.last_change = unix_now!(f64);
            if !new_state {
                self.handler.set_high();
            } else {
                self.handler.set_low();
            }
        }
    }
}

impl fmt::Display for OutputPinHandler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Pin: {}, Current state: {}, Last state: {}, Last change: {}",
            self.handler.pin(),
            self.current_state,
            self.last_state,
            self.last_change
        )
    }
    
}