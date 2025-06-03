
use core::fmt;
use std::any::Any;
use std::sync::RwLock;
use std::sync::Arc;
use std::path::PathBuf;
use std::fs;
use std::io::Write;

use crate::conditions::constants::AllwaysTrue;
use crate::evaluator::logger::LogLevel;
use crate::tasks::{Conditional, ConditionalTypes};
use crate::tasks::TaskError;
use crate::evaluator::enviorment::Environment;
use crate::conditions::Condition;


pub struct Task {
    name: &'static str,
    conditions: Option<Box<dyn Condition>>,
    action: fn(Arc<RwLock<Environment>>) -> Result<(), TaskError>,
    min_delay_between_exec: f64,
}

impl Task {
    pub(crate) fn action(&self, environment: Arc<RwLock<Environment>>) -> Result<(), TaskError> {
        (self.action)(environment)
    }

    pub fn new (name: &'static str) -> Task {
        Task {
            name,
            conditions: None,
            action: move |_| {Err(TaskError::ActionError { comment: "No action provided".to_string() })},
            min_delay_between_exec: 0.,
        }
    }

    pub fn when_condition(mut self, conditions: Box<dyn Condition>) -> Task {
        self.conditions = Some(conditions);
        self
    }
    pub fn with_action(mut self, action: fn(Arc<RwLock<Environment>>) -> Result<(), TaskError>) -> Task {
        self.action = action;
        self
    }
    pub fn with_min_delay_between_exec(mut self, min_delay_between_exec: f64) -> Task {
        self.min_delay_between_exec = min_delay_between_exec;
        self
    }

    pub fn to_eveluatable(self) -> ConditionalTypes {
        ConditionalTypes::Task(Arc::new(self))
    }

}
impl Conditional for Task {
    fn get_conditions(&self) -> Option<&Box<dyn Condition>> {
        self.conditions.as_ref()
    }

    fn min_delay_between_exec(&self) -> f64 {
        self.min_delay_between_exec
    }

    fn get_name(&self) -> String {
        self.name.to_string()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    
}

impl fmt::Debug for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GeneralTask {{ name: {}, min_delay_between_exec: {} }}", self.name, self.min_delay_between_exec)
    }
}

pub(crate) fn get_periodic_state_writer(min_delay_between_exec: u32) -> Arc<Task> {
    Arc::new(Task {
        name: "periodic_print_state_to_file",
        conditions: Some(AllwaysTrue::new()),
        action: write_appstate_to_file,
        min_delay_between_exec: min_delay_between_exec as f64,
    })
    }

fn write_appstate_to_file(enviorment: Arc<RwLock<Environment>>) -> Result<(), TaskError> {
    let env_reader = enviorment.read().unwrap();
    let state_file = PathBuf::from(format!("/tmp/embedded_task_dispatcher_{}.log", env_reader.pid));
    let file = fs::OpenOptions::new()
        .append(false)
        .create(true)
        .write(true)
        .open(state_file);
    if file.is_err() {
        env_reader.log("Faild to write state to statefile!", LogLevel::Error);
    } else {
        if let Err(e) = write!(file.unwrap(), "{}", env_reader.to_string()) {
            env_reader.log(&format!("Failed to write to file: {}", e), LogLevel::Error);
            return Err(TaskError::SystemError { comment: format!("Failed to write to file: {}", e) });
        }
    }
    Ok(())
}