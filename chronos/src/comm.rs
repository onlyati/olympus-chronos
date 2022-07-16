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