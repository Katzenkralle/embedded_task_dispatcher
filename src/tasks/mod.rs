extern crate custom_error;

use std::fmt::Debug;
use std::sync::{Arc, RwLock};
use std::any::Any;

pub mod general_task;
pub mod task_context;

use crate::conditions::Condition;
use crate::errors::TaskError;
use crate::evaluator::enviorment::Environment;
use crate::evaluator::RunningTreeState;
use crate::tasks::task_context::Unit;
use crate::tasks::general_task::Task;

#[derive(Debug)]
pub enum ConditionalTypes {
    Task(Arc<Task>),
    TaskContext(Arc<Unit>),
} 
impl ConditionalTypes {
    pub(crate) fn get_inner_conditional(&self) -> &dyn Conditional {
        match self {
            ConditionalTypes::Task(task) => task.as_ref(),
            ConditionalTypes::TaskContext(context) => context.as_ref(),
        }
    }
    pub(crate) fn is_context(&self) -> bool {
        match self {
            ConditionalTypes::TaskContext(_) => true,
            _ => false,
        }
    }

    pub fn new_task(task: Task) -> ConditionalTypes {
        ConditionalTypes::Task(Arc::new(task))
    }
    pub fn new_unit(context: Unit) -> ConditionalTypes {
        ConditionalTypes::TaskContext(Arc::new(context))
    }

}

pub trait Conditional {
    fn get_name(&self) -> String;
    fn min_delay_between_exec(&self) -> f64;
    fn as_any(&self) -> &dyn Any;
    fn get_conditions(&self) -> Option<&Box<dyn Condition>> {
        return None
    }
    fn trigger(&self, environment: Arc<RwLock<Environment>>, running_tree: &RunningTreeState) -> Result<bool, TaskError> {
        let conditions = self.get_conditions();
        if let Some(conditions) = conditions {
            return conditions.eval(&environment.read().unwrap(), running_tree)
        };
        Ok(false)
    }
    fn get_stay_conditions(&self) -> Option<&Box<dyn Condition>> {
        return None
    }
    fn trigger_stay(&self, environment: Arc<RwLock<Environment>>, running_tree: &RunningTreeState) -> Result<bool, TaskError> {
        let conditions = self.get_stay_conditions();
        if let Some(conditions) = conditions {
            return conditions.eval(&environment.read().unwrap(), running_tree)
        };
        return self.trigger(environment, running_tree);
    }
}
