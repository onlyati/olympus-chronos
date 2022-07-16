pub fn help(_options: Vec<String>) -> Result<String, String> {
    let mut response = String::new();

    response += "Possible Chronos commands:\n";
    response += "List active timers:       list active\n";
    response += "List started timers:      list startup\n";
    response += "List all timer config:    list all\n";
    response += "Purge timer:              purge <timer-id>\n";
    response += "Add timer:                add <timer-id>\n";

    return Ok(response);
}

pub fn list(options: Vec<String>) -> Result<String, String> {
    if options.len() == 0 {
        return Err(String::from("You must specifiy what you want list: active, startup or all. See help for more info"));
    }

    if options[0] == String::from("active") {

    }

    if options[0] == String::from("startup") {
        
    }

    if options[0] == String::from("all") {
        
    }

    return Err(format!("Specified parameter is invalid: {}", options[0]));
}