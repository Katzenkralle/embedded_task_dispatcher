extern crate custom_error;
use std::collections::HashMap;

use std::time::{self};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::sync::RwLock;


use crate::tasks::{Conditional, ConditionalTypes};
use crate::unix_now;
use crate::evaluator::suite::Suite;
use crate::types::StateType;
use crate::errors::TaskError;

use super::logger::LogLevel;
use super::{RunningTreeState, enviorment, EvalResult};


fn evaluate_context_v2<'a>(
    running_tasks: &mut HashMap<String, JoinHandle<()>>,
    unit: &'a ConditionalTypes,
    environment: Arc<RwLock<enviorment::Environment>>,
    running_tree: &RunningTreeState) -> EvalResult<'a> {
    let as_conditional = unit.get_inner_conditional();
    if (as_conditional.min_delay_between_exec() > 
        unix_now!(f64) - environment.read().unwrap().app_state.get(&format!("{}_executed", as_conditional.get_name()))
                            .or_else(|| Some(&StateType::Int(0.))).unwrap().as_int()) ||
            running_tasks.contains_key(&as_conditional.get_name()) {
            return EvalResult::Stay;
        }

        let trigger_result = if unit.is_context() && running_tree.currently_active {
            as_conditional.trigger_stay(environment.clone(), running_tree)
        } else {
            as_conditional.trigger(environment.clone(), running_tree)
        };

        if let Ok(result) = trigger_result{
            match unit {
                ConditionalTypes::Task(task) => {
                    if !result {
                        return EvalResult::Stay;
                    }
                    environment.read().unwrap().log(&format!("Executing Task: {}", task.get_name()), LogLevel::Debug);
                    environment.write()
                        .unwrap()
                        .app_state
                        .insert(format!("{}_executed", task.get_name()), StateType::Int(unix_now!() as f64));
                    
                    let enviorment = environment.clone();
                    let task = Arc::clone(task); // Clone the Arc to safely share between threads
                    running_tasks.insert(task.get_name(), thread::spawn(move || {
                        if let Err(error) = task.action(enviorment.clone()) {
                            enviorment.read().unwrap().log(&format!("Task {} failed: {}", task.get_name(), error), LogLevel::Error);
                        };
                    }));
                }
                ConditionalTypes::TaskContext(context) => {
                    if !running_tree.currently_active{
                        return if result {
                            EvalResult::MoveTo(unit)
                        } else {
                            EvalResult::MoveOut
                        };
                    };
                    for subtask in context.subunits.iter().collect::<Vec<_>>() {
                        let result = evaluate_context_v2(running_tasks, subtask, environment.clone(), &running_tree.get_running_tree_for_subtask());
                        if let EvalResult::MoveTo(result) = result {
                            return EvalResult::MoveTo(result);
                        }
                    }
                    if !result {
                        if let Some(on_exit) = context.on_exit.clone() {
                            evaluate_context_v2(running_tasks, &ConditionalTypes::Task(on_exit), environment, running_tree);
                        }
                        return EvalResult::MoveOut;
                    }
                }
            }
        }else if let Err(err) = trigger_result {
            environment.read().unwrap().log(&format!("Trigger {} failed: {}", as_conditional.get_name(), err), LogLevel::Error);
        } 
        
        EvalResult::Stay

    }

pub fn suite_dispatcher(suite: Suite) -> Result<(), TaskError> {
    let environment = suite.structure;
    let mut running_tasks: HashMap<String, JoinHandle<()>> = HashMap::new();

    environment.read().unwrap().log("Entering loop", LogLevel::Debug);
    
    let mut context_pointer_tree: HashMap<&str, (Vec<&ConditionalTypes>, RunningTreeState)> = HashMap::new();
    for (name, unit) in suite.tasks.iter() {
        context_pointer_tree.insert(*name, (vec![unit], RunningTreeState::new()));
    }
    loop {
        // Update gpio states from environment
        environment.write()
            .unwrap()
            .input_gpios
            .iter_mut()
            .for_each(|(_, value)|{
                value.last_state = value.current_state;
                    value.current_state = value.handler.is_low();
                    if value.current_state != value.last_state {
                        value.last_change = unix_now!(f64);
                }});
        
        
        // Execute Units
        for (name, unit) in suite.tasks.iter(){
            let mut repeat = true;
            if let Some((active_iteration, tree_state)) = context_pointer_tree.get_mut(name){
                while repeat {
                    let current_context = active_iteration.last().copied().unwrap_or(unit);
                    let result = evaluate_context_v2(
                        &mut running_tasks,
                        current_context,
                        environment.clone(),
                        &tree_state);
                    
                    match result {
                        EvalResult::Stay => {repeat = false; tree_state.first_iteration_after_move = false;},
                        EvalResult::MoveOut => {
                            active_iteration.pop();
                            tree_state.first_iteration_after_move = true;
                            tree_state.moved_in_from_back = true;
                            environment.read().unwrap().log(&format!("Moving out of context: {}", current_context.get_inner_conditional().get_name()), LogLevel::Debug);
                        },
                        EvalResult::MoveTo(result) => {
                            active_iteration.push(result);
                            tree_state.first_iteration_after_move = true;
                            tree_state.moved_in_from_back = false;
                            environment.read().unwrap().log(&format!("Moving to context: {}", result.get_inner_conditional().get_name()), LogLevel::Debug);
                        }
                    }
                }
            } else {
                environment.read().unwrap().log(&format!("Context {} not found in context tree", name), LogLevel::Error);
            }
        }
        
        
        // Other methodes would require the implementation of Copy trait
        let mut unfinished_tasks: HashMap<String, JoinHandle<()>> = HashMap::new();
        for (name, task) in running_tasks.into_iter() {
            if task.is_finished() {
                if let Err(_) = task.join() {
                    environment.read().unwrap().log(&format!("Task {} could not re-join main loop", name), LogLevel::Error);
                }
            } else {
                unfinished_tasks.insert(name.to_string(), task);
            }
        }
        running_tasks = unfinished_tasks;
        thread::sleep(time::Duration::new(0, suite.suite_options.sleep_time.or_else(|| Some(250_000_000)).unwrap() as u32));
    }
}