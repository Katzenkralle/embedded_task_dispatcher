use core::fmt;
use std::any::Any;
use std::sync::Arc;


use crate::tasks::{
    Task,
    Condition,
    Conditional,
    ConditionalTypes};


pub struct Unit {
    pub name: &'static str,
    pub condition: Option<Box<dyn Condition>>,
    pub subunits: Vec<ConditionalTypes>,
    pub stay_condition: Option<Box<dyn Condition>>,
    pub on_exit: Option<Arc<Task>>
}
impl Unit {
    pub fn new(name: &'static str) -> Self {
        Unit {
            name,
            condition: None,
            subunits: Vec::new(),
            stay_condition: None,
            on_exit: None,
        }
    }

    pub fn when_condition(mut self, condition: Box<dyn Condition>) -> Self {
        self.condition = Some(condition);
        self
    }

    pub fn stay_while_condition(mut self, condition: Box<dyn Condition>) -> Self {
        self.stay_condition = Some(condition);
        self
    }

    pub fn subunit(mut self, subtask: ConditionalTypes) -> Self {
        self.subunits.push(subtask);
        self
    }

    pub fn subunits(mut self, subtasks: Vec<ConditionalTypes>) -> Self {
        self.subunits.extend(subtasks);
        self
    }

    pub fn on_exit(mut self, task: Task) -> Self {
        self.on_exit = Some(Arc::new(task));
        self
    }

    pub fn to_eveluatable(self) -> ConditionalTypes {
        ConditionalTypes::TaskContext(Arc::new(self))
    }
}

impl Conditional for Unit  {
    fn get_name(&self) -> String {
        self.name.to_string()
    }
    fn min_delay_between_exec(&self) -> f64 {
        0.
    }
    fn get_conditions(&self) -> Option<&Box<dyn Condition>> {
        self.condition.as_ref()
    }

    fn get_stay_conditions(&self) -> Option<&Box<dyn Condition>> {
        self.stay_condition.as_ref()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl fmt::Debug for Unit  {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TaskContext {{ name: {}, subtasks: {:?} }}", self.name, self.subunits)
    }
}