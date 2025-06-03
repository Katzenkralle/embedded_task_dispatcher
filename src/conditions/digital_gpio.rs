use super::*;
#[derive(Debug)]
pub struct DigitalGpioCondition {
    pub(crate) pin: u8,
    pub(crate) state: bool,
    pub(crate) is_output: bool,
    pub(crate) delay: f64, // delay uses time of last gpio change, not of last time true
    pub(crate) use_flank: bool,
    pub(crate) has_flanked: Mutex<bool>,
    pub(crate) first_eval_at: Mutex<f64>, 
}

impl DigitalGpioCondition {
    pub fn new_output(pin: u8) -> Box<Self> {
        Box::new(DigitalGpioCondition { pin, state: true, delay: 0., use_flank: false, is_output: true, has_flanked: Mutex::new(false), first_eval_at: Mutex::new(-1.) })
    }
    pub fn new_input(pin: u8) -> Box<Self> {
        Box::new(DigitalGpioCondition { pin: pin, state: true, delay: 0., use_flank: false, is_output: false, has_flanked: Mutex::new(false), first_eval_at: Mutex::new(-1.) })
    }
    pub fn when_false(mut self: Box<Self>) -> Box<Self> {
        self.state = false;
        self
    }
    pub fn after_delay(mut self: Box<Self>, delay: f64) -> Box<Self> {
        self.delay = delay;
        self
    }
    pub fn on_flank(mut self: Box<Self>) ->  Box<Self>{
        self.use_flank = true;
        self
    }

}

impl Condition for DigitalGpioCondition {
    fn eval(&self, environment: &Environment, running_tree: &RunningTreeState) -> Result<bool, TaskError> {
        let main_condition_handler: &dyn PinHandler  = match self.is_output {
            true => environment.output_gpios.get(&self.pin)
                .ok_or(TaskError::IoError { comment: format!("Pin {} not found in output GPIO state", self.pin) })?
                as &dyn PinHandler,
            false => environment.input_gpios.get(&self.pin)
                .ok_or(TaskError::IoError { comment: format!("Pin {} not found in input GPIO state", self.pin) })?
                as &dyn PinHandler,
        };
        let mut first_eval_at = self.first_eval_at.try_lock();
        if running_tree.first_iteration_after_move {
            self.has_flanked.try_lock().and_then(|mut lock| Ok((*lock) = false)).ok();
            if !running_tree.currently_active || running_tree.moved_in_from_back 
                || first_eval_at.as_ref().map(|val| **val).unwrap_or(-1.) == -1. {
                first_eval_at.as_mut()
                    .and_then(|lock| Ok((**lock) = unix_now!(f64))).ok();
             }
        }
        //let most_recent_change = first_eval_at.map(|val| *val).unwrap_or_else(|_| main_condition_handler.last_change());
        let most_recent_change = first_eval_at.map(|val| *val)
            .unwrap_or(-1.)
            .max(main_condition_handler.last_change());
        let a = unix_now!(f64) - most_recent_change;
        if main_condition_handler.current_state() != self.state || 
            a < self.delay {

            let _ = self.has_flanked.try_lock().and_then(|mut lock| Ok((*lock) = false));
            Ok(false)
        }
        else {
            let lock = self.has_flanked.try_lock();
            if self.use_flank && (lock.is_err() || *lock.unwrap()) {
                return Ok(false)
            }
            let lock = self.has_flanked.try_lock();
            if let Ok(mut lock) = lock {
                *lock = true;
            }
            Ok(true)
        }
    }
    fn as_automaticlt_initializable(&self) -> Option<Vec<AutomaticltInitializable>> {
        Some(vec![AutomaticltInitializable::DigitalGpioCondition(self)])
    }
}


