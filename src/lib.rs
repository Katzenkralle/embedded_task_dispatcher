extern crate custom_error;


pub mod lcd_driver;
pub mod evaluator;
pub mod errors;
pub mod conditions;
pub mod tasks;
pub mod prebuilds;
pub mod types;

#[macro_export]
macro_rules! unix_now{
    () => {(||{ 
        use std::time::{SystemTime, UNIX_EPOCH};
        return SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() })()};
    ($x:ty) => {{
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as $x
    }};
}
