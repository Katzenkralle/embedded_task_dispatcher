use std::result::Result;
use std::sync::Mutex;
use std::fmt::Debug;

use crate::errors::TaskError;
use crate::types::StateType;
use crate::evaluator::enviorment::Environment;
use crate::types::{PinHandler};
use crate::evaluator::RunningTreeState;
use crate::unix_now;

pub mod app_state;
pub mod digital_gpio;
pub mod dispatch_tree;
pub mod logic_gates;
pub mod constants;

pub use app_state::AppCondition;
pub use digital_gpio::DigitalGpioCondition;
pub use dispatch_tree::TreeCondition;
pub use logic_gates::Gates;
pub use constants::AllwaysTrue;



pub trait Condition: Send + Sync + Debug {
    fn eval(&self, environment: &Environment, running_tree_state: &RunningTreeState) -> Result<bool, TaskError>;
    fn as_automaticlt_initializable(&self) -> Option<Vec<AutomaticltInitializable>> {
        None
    }
} 


pub enum AutomaticltInitializable <'a>{
    AppCondition(&'a AppCondition),
    DigitalGpioCondition(&'a DigitalGpioCondition),
}
