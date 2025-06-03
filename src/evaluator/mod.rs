extern crate custom_error;



pub mod enviorment;
pub mod suite;
pub mod dispatcher;
pub mod logger;

use crate::tasks::ConditionalTypes;

enum EvalResult<'a> {
    Stay,
    MoveOut,
    MoveTo(&'a ConditionalTypes)
}

pub struct RunningTreeState {
    pub(crate) moved_in_from_back: bool,
    pub(crate) first_iteration_after_move: bool,
    pub(crate) currently_active: bool,
}
impl RunningTreeState {
    pub(crate) fn new() -> Self {
        RunningTreeState {
            moved_in_from_back: false,
            first_iteration_after_move: true,
            currently_active: true,
        }
    }
    pub(crate) fn get_running_tree_for_subtask<'a>(&'a self) -> RunningTreeState {
        RunningTreeState {
            moved_in_from_back: false,
            first_iteration_after_move: self.first_iteration_after_move,
            currently_active: false,
        }
    }
}

