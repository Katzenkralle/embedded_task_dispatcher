use super::*;
use super::constants::AllwaysTrue;

#[derive(Debug)]
pub enum Gates {
    And(Vec<Box<dyn Condition>>),
    Or(Vec<Box<dyn Condition>>),
    Not(Box<dyn Condition>),
}

impl Gates {
    pub fn and() -> Box<Self> {
        Box::new(Gates::And(Vec::new()))
    }

    pub fn or() -> Box<Self> {
        Box::new(Gates::Or(Vec::new()))
    }
    pub fn not() -> Box<Self> {
        Box::new(Gates::Not(AllwaysTrue::new()))
    }

    pub fn condition(mut self: Box<Self>, condition: Box<dyn Condition>) -> Box<Self> {
        match &mut *self {
            Gates::And(conditions) => conditions.push(condition),
            Gates::Or(conditions) => conditions.push(condition),
            Gates::Not(inner) => *inner = condition,
        }
        self
    }

    pub fn multiple_conditions(mut self: Box<Self>, conditions: Vec<Box<dyn Condition>>) -> Box<Self> {
        match &mut *self {
            Gates::And(existing_conditions) => existing_conditions.extend(conditions),
            Gates::Or(existing_conditions) => existing_conditions.extend(conditions),
            Gates::Not(inner) => {
                if conditions.len() >= 1 {
                    *inner = conditions.into_iter().next().unwrap();
                }
            }
        }
        self
    }
}



impl Condition for Gates {
    fn eval(&self, environment: &Environment, running_tree: &RunningTreeState) -> Result<bool, TaskError> {
        match self {
            Gates::And(conditions) => {
                for condition in conditions {
                    if !condition.eval(environment, running_tree)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            },
            Gates::Or(conditions) => {
                for condition in conditions {
                    if condition.eval(environment, running_tree)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            },
            Gates::Not(condition) => Ok(!condition.eval(environment, running_tree)?),
        }
    }
    fn as_automaticlt_initializable(&self) -> Option<Vec<AutomaticltInitializable>> {
        let mut initializables = Vec::new();
        match self {
            Gates::And(conditions) | Gates::Or(conditions) => {
                for condition in conditions {
                    if let Some(auto_init) = condition.as_automaticlt_initializable() {
                        initializables.extend(auto_init);
                    }
                }
            },
            Gates::Not(condition) => {
                if let Some(auto_init) = condition.as_automaticlt_initializable() {
                    initializables.extend(auto_init);
                }
            },
        }
        if initializables.is_empty() {
            None
        } else {
            Some(initializables)
        }

    }
}