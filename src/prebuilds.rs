use std::sync::{RwLock, Arc};
use std::collections::HashMap;

use crate::errors::TaskError;
use crate::evaluator::enviorment::{Environment};
use crate::lcd_driver::{LCDcommand, LCDArg, LCDProgramm, LCDdriver};


use crate::{clear_lcd, write_lcd, move_lcd};

pub fn prepare_lcd (environment: Arc<RwLock<Environment>>, msg: Option<&str>, clear_lcd: Option<bool> ) -> Result<(), TaskError> {
    let mut env = environment.write().unwrap();
    let lcd = if let Ok(lcd) = &mut env.lcd_driver {
        lcd
    } else {
        let path = match &env.lcd_driver {
            Err(p) => p.clone(),
            _ => unreachable!(),
        };
        let lcd_instance = LCDdriver::new(&path, clear_lcd.unwrap_or(true))
            .map_err(|e| TaskError::IoError { comment: e.to_string() })?;
        env.lcd_driver = Ok(lcd_instance);
        match &mut env.lcd_driver {
            Ok(lcd) => lcd,
            _ => unreachable!(),
        }
    };
        
    if let Some(clear) = clear_lcd {
        if clear {
            lcd.exec(clear_lcd!()).ok();
        }
    }
    if let Some(message) = msg {
        lcd.exec(move_lcd!(0, 0)).ok();
        lcd.exec(write_lcd!(message)).ok();
    }

    Ok(())
}

#[macro_export]
macro_rules! lcd_prepare{
    ($name:expr_2021, $msg:expr_2021, $clear_lcd:expr_2021) => {(||{ 
        use embedded_task_dispatcher::prebuilds::prepare_lcd;
        return Task::new($name)
        .when_condition(AllwaysTrue::new().on_flank())
        .with_action(move |environment| {
            prepare_lcd(environment, $msg, $clear_lcd)
        })
        .to_eveluatable()
        })()};
    ($name:expr_2021, $msg:expr_2021, $clear_lcd:expr_2021, $extra_condition:expr_2021) => {(||{ 
            use embedded_task_dispatcher::prebuilds::prepare_lcd;
            return Task::new("test")
            .when_condition(Gates::and()
                .condition(AllwaysTrue::new().on_flank())
                .condition($extra_condition)
            )
            .with_action(move |environment| {
                prepare_lcd(environment, $msg, $clear_lcd)
            })
            .to_eveluatable()
            })()};
} 