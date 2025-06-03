extern crate custom_error;
use core::fmt;
use std::collections::HashMap;
use std::path::PathBuf;
use rppal::gpio::Gpio;

use std::thread::{self};
use std::sync::mpsc::{self, Sender};

use crate::unix_now;
use crate::errors::TaskError;
use crate::types::StateType;
use crate::types::InputPinHandler;
use crate::lcd_driver::LCDdriver;
use crate::tasks::ConditionalTypes;

use crate::types::OutputPinHandler;
use super::logger::{logger, LoggerCommand, LogLevel};


#[derive(Debug)]
pub struct Environment {
    pub input_gpios: HashMap<u8, InputPinHandler>,
    pub app_state: HashMap<String, StateType>,
    pub output_gpios: HashMap<u8, OutputPinHandler>,
    pub lcd_driver: Result<LCDdriver, PathBuf>,
    logger: Sender<LoggerCommand>,
    pub (crate) pid: u32,
}

fn recursively_initialize(mut enviorment: Environment, unit: &ConditionalTypes) -> Result<Environment, TaskError> {
    let as_condtional = unit.get_inner_conditional();
    
    for condition in as_condtional.get_conditions().iter()
        .chain(as_condtional.get_stay_conditions().iter()) {
        if let Some(automaticlt_initializables) = condition.as_automaticlt_initializable() {
            for auto_initializable in automaticlt_initializables{
                match auto_initializable {
                    crate::conditions::AutomaticltInitializable::AppCondition(app_condition) => {
                        if !enviorment.app_state.contains_key(app_condition.key) {
                            enviorment.app_state.insert(
                                app_condition.key.to_string(),
                                app_condition.value.as_default()
                            );
                        }
                    },
                    crate::conditions::AutomaticltInitializable::DigitalGpioCondition(digital_gpio_condition) => {
                        if digital_gpio_condition.is_output{
                            enviorment.add_output_gpio(digital_gpio_condition.pin)?;
                            continue;
                        }
                        if !enviorment.input_gpios.contains_key(&digital_gpio_condition.pin) {
                            enviorment.input_gpios.insert(
                                digital_gpio_condition.pin,
                                InputPinHandler {
                                    handler: Gpio::new()
                                        .map_err(|e| TaskError::IoError { comment: (format!("Could not access GPIOs: {}", e)) })?
                                        .get(digital_gpio_condition.pin)
                                        .map_err(|e| TaskError::IoError { comment: (format!("Could not get pin {}: {}", digital_gpio_condition.pin, e)) })?
                                        .into_input_pullup(),
                                    last_state: false,
                                    current_state: false,
                                    last_change: unix_now!(f64),
                                },
                            );
                        }
                    },
                }                    
            }
        }
        enviorment.app_state.insert(format!("{}_executed", as_condtional.get_name()), StateType::Int(-1.));
    }
        
    if let ConditionalTypes::TaskContext(task_context) = unit {
        for task in task_context.subunits.iter() {
            enviorment = recursively_initialize(enviorment, task)?;
        }
    }
    
    Ok(enviorment)
}

impl Environment {
    pub(super) fn new(tasks: &HashMap<&str, ConditionalTypes>, lcd_driver_path: Option<&PathBuf>, log_file: Option<PathBuf>) -> Result<Environment, TaskError> {
        let (tx, rx) = mpsc::channel();
        thread::spawn(move ||{
            logger(rx,  log_file);
        });
        let mut enviorment = Environment {
            input_gpios: HashMap::new(),
            app_state: HashMap::new(),
            logger: tx,
            pid: std::process::id(),
            output_gpios: HashMap::new(),
            lcd_driver: match lcd_driver_path {
                Some(p) => LCDdriver::new(p, true).map_err(|_| p.clone()),
                None => Err(PathBuf::new())
            }

        };
        for (_, unit) in tasks.iter() {
            enviorment = recursively_initialize(enviorment, unit)?;
        }
        
        return Ok(enviorment)
    }

    pub(super) fn add_output_gpio(&mut self, pin: u8) -> Result<(), TaskError> {
        if self.output_gpios.contains_key(&pin) {
            return Ok(());
        }
        let mut gpio = Gpio::new()
            .map_err(|e| TaskError::IoError { comment: (format!("Could not access GPIOs: {}", e)) })?
            .get(pin)
            .map_err(|e| TaskError::IoError { comment: (format!("Could not get pin {}: {}", pin, e)) })?
            .into_output();
        gpio.set_high();

        let new_handler = OutputPinHandler {
            handler: gpio,
            last_state: false,
            current_state: false,
            last_change: unix_now!(f64),
        };

        self.output_gpios.insert(pin, new_handler);
        Ok(())
    }

    pub fn log(&self, msg: &str, log_level: LogLevel) -> (){
        let _ = self.logger.send(LoggerCommand::Log(msg.to_string(), log_level));
    }
    pub fn change_log_level(&self, log_level: LogLevel) -> (){
        let _ = self.logger.send(LoggerCommand::ChangeLogLevel(log_level));
    }
}

impl fmt::Display for Environment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[derive(serde::Serialize)]
        struct EnviormentDisplay {
            time: u64,
            environment: HashMap<String, HashMap<String, String>>,
        }
        let mut print_env = EnviormentDisplay {
            time: unix_now!(),
            environment: HashMap::new(),
        };
        let mut gpio_state_print = HashMap::new();
        for (key, value) in self.input_gpios.iter() {
            gpio_state_print.insert(
                key.to_string(),
                value.to_string(),
            );
        }
        print_env.environment.insert("gpio_state".to_string(), gpio_state_print);
        let mut app_state_print = HashMap::new();
        for (key, value) in self.app_state.iter() {
            app_state_print.insert(
                key.to_string(),
                value.to_string(),
            );
        }
        print_env.environment.insert("app_state".to_string(), app_state_print);
        write!(f, "{}", serde_json::to_string_pretty(&print_env).unwrap())
    }
}