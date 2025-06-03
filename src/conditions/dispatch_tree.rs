use super::*;

#[derive(Debug)]
pub struct TreeCondition {
    pub when_moving_up: bool,
}
impl TreeCondition {
    pub fn new_when_moving_up() -> Box<Self> {
        Box::new(TreeCondition { when_moving_up: true })
    }
    pub fn new_when_moving_down() -> Box<Self> {
        Box::new(TreeCondition { when_moving_up: true })
    }
}

impl Condition for TreeCondition {
    fn eval(&self, _: &Environment, running_tree: &RunningTreeState) -> Result<bool, TaskError> {
        if running_tree.moved_in_from_back && !self.when_moving_up {
            Ok(true)
        } else if !running_tree.moved_in_from_back && self.when_moving_up {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}