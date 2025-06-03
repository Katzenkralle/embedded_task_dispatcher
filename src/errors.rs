use custom_error::custom_error;

custom_error!{pub TaskError
    TriggerError{comment:String} = "{comment}",
    ActionError{comment:String} = "{comment}",
    IoError{comment:String} = "{comment}",
    SystemError{comment:String} = "{comment}",
}