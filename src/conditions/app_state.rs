use super::*;

#[derive(Debug)]
pub struct AppCondition {
    pub(crate) key: &'static str,
    pub(crate) value: StateType,
}


impl AppCondition {
    pub fn new(key: &'static str, value: StateType) -> Box<Self> {
        Box::new(AppCondition { key, value })
    }
}

impl Condition for AppCondition {
    fn eval(&self, environment: &Environment, _: &RunningTreeState) -> Result<bool, TaskError> {
        match environment.app_state.get(self.key) {
            Some(state_value) => Ok(state_value == &self.value),
            None => Ok(false),
        }
    }
    fn as_automaticlt_initializable(&self) -> Option<Vec<AutomaticltInitializable>> {
        Some(vec![AutomaticltInitializable::AppCondition(self)])
    }
    
}
