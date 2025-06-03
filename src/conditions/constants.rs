use super::*;

#[derive(Debug)]
pub struct AllwaysTrue{
    pub(crate) with_delay: f64,
    pub(crate) with_flank: bool,
    pub(crate) has_flanked: Mutex<bool>,
    pub(crate) first_exec_at: Mutex<f64>,
}
impl Condition for AllwaysTrue {
    fn eval(&self, _environment: &Environment, running_tree: &RunningTreeState) -> Result<bool, TaskError> {
        let mut first_exec = self.first_exec_at
            .try_lock()
            .map_err(|_| TaskError::IoError { comment: "Failed to lock first_exec_at".to_string() })?;
        let mut has_flanked = self.has_flanked
            .try_lock()
            .map_err(|_| TaskError::IoError { comment: "Failed to lock has_flanked".to_string() })?;
        
        if running_tree.first_iteration_after_move {
            *first_exec = unix_now!(f64);
            *has_flanked = false;
        }

        if self.with_delay > 0. && unix_now!(f64) - self.with_delay < *first_exec {
            return Ok(false);
        }
        if self.with_flank && *has_flanked {
            return Ok(false);
        }
        *has_flanked = true;
        Ok(true)
    }
}

impl AllwaysTrue {
    pub fn new() -> Box<Self> {
        Box::new(AllwaysTrue { with_delay: 0., with_flank: false, has_flanked: Mutex::new(false), first_exec_at: Mutex::new(-1.) })
    }
    pub fn after_delay(mut self: Box<Self>, delay: f64) -> Box<Self> {
        self.with_delay = delay;
        self
    }
    pub fn on_flank(mut self: Box<Self>) -> Box<Self> {
        self.with_flank = true;
        self
    }
    
}

