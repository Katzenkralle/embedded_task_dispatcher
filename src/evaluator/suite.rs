extern crate custom_error;
use std::collections::HashMap;
use std::path::PathBuf;

use std::sync::Arc;
use std::sync::RwLock;
use std::fs;
use serde_json::Value as JsonValue;

use crate::conditions::constants::AllwaysTrue;
use crate::errors::TaskError;
use crate::types::StateType;
use crate::tasks::{general_task::get_periodic_state_writer, ConditionalTypes};
use crate::tasks::task_context::Unit;
use crate::evaluator::enviorment::{Environment};
use super::logger::LogLevel;



#[derive(Clone)]
pub struct SutieOptions {
    pub periodicly_print_state_to_file: Option<u64>,
    pub sleep_time: Option<u64>,
    pub log_file: Option<PathBuf>,
    pub log_level: LogLevel,
    pub ignore_errors_when_possible: bool, // Not implemented yet
    pub config_file: Option<PathBuf>, 
    pub lcd_driver: Option<PathBuf>
}

pub struct Suite<'a> {
    pub(crate) structure: Arc<RwLock<Environment>>,
    pub(crate) tasks: HashMap<&'a str, ConditionalTypes>,
    pub(crate) config_path: Option<PathBuf>,
    pub(crate) suite_options: SutieOptions
}
impl SutieOptions {
    pub fn new() -> SutieOptions {
        SutieOptions {
            periodicly_print_state_to_file: None,
            sleep_time: Some(250_000_000),
            log_file: None,
            log_level: LogLevel::Info,
            ignore_errors_when_possible: false,
            config_file: None,
            lcd_driver: None,
        }
    }
}

fn json_config_loader(
    app_state: &mut HashMap<String, StateType>,
    config: JsonValue,
) -> Result<(), TaskError> {
    if let JsonValue::Object(content) = config {
        for (key, item) in content {
            match item {
                JsonValue::String(value) => {
                    app_state.insert(key, StateType::Str(value));
                }
                JsonValue::Number(value) => {
                    app_state.insert(key, StateType::Int(value.as_f64().unwrap_or(0.)));
                }
                JsonValue::Bool(value) => {
                    app_state.insert(key, StateType::Bool(value));
                }
                _ => {
                    return Err(TaskError::SystemError { comment: "Unsupported JSON value type".to_string() });
                }
            }
            }
        } else {
        return Err(TaskError::SystemError { comment: "Invalid JSON format; Only OBJECT.<key> = str|int|bool".to_string() });
    }
    Ok(())
}

impl <'a> Suite<'a> {
    pub fn new(tasks: HashMap<&'a str, Vec<ConditionalTypes>>, output_gpio: Option<Vec<u8>>, options: Option<SutieOptions>) -> Result<Suite<'a>, TaskError> {
        let optios = options.unwrap_or(SutieOptions::new());
        let options_ = optios.clone();
        let mut task_layers = HashMap::new();
        for (key, value) in tasks.into_iter() {
            task_layers.insert(key, 
                ConditionalTypes::TaskContext(Arc::new(Unit {
                name: "root",
                condition: Some(AllwaysTrue::new()),
                subunits: value,
                stay_condition: Some(AllwaysTrue::new()),
                on_exit: None
            })));
        }    
        if optios.periodicly_print_state_to_file.is_some() {
            task_layers.insert("periodic_print_state_to_file", 
                ConditionalTypes::Task(
                    get_periodic_state_writer(optios.periodicly_print_state_to_file.unwrap() as u32)));
        }

        let structure = Arc::new(RwLock::new(
            Environment::new(&task_layers, optios.lcd_driver.as_ref(), optios.log_file)?));
        if let Some(output_gpio) = output_gpio {
            for pin in output_gpio {
                structure.write().unwrap()
                    .add_output_gpio(pin)?;
            }
        }

        structure.write().unwrap().change_log_level(optios.log_level);
        structure.read().unwrap().log(&format!("Environment initialized with: {:#?}", structure.read().unwrap()), LogLevel::Debug);
        Ok(Suite {
            structure,
            tasks: task_layers,
            config_path: optios.config_file,
            suite_options: options_,
        })   
    }

    pub fn load_config(&self, handler_fn: Option<fn(&mut HashMap<String, StateType>, JsonValue) -> Result<(), TaskError>>) 
        -> Result<(), TaskError> {
        if let Some(path) = &self.config_path {
            let raw_config = fs::read_to_string(path)
                .map_err(|e| TaskError::SystemError { comment: (format!("Could not read config file: {}", e)) })?;
            let config: JsonValue = serde_json::from_str(&raw_config)
                .map_err(|e| TaskError::SystemError { comment: (format!("Could not parse config file: {}", e)) })?;
            
            let pre_handler_state = self.structure.read().unwrap().app_state.clone();
            if let Err(e) = handler_fn.unwrap_or_else(|| json_config_loader)
                (&mut self.structure.write().unwrap().app_state, config){
                    self.structure.write().unwrap().app_state = pre_handler_state;
                    return Err(e);
                }
        } else {
            return Err(TaskError::SystemError { comment: "No config path provided".to_string() });
        }
        Ok(())
    }
}