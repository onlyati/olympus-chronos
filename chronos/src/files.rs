/// Verify that directory structure exists
/// 
/// This program check that file sturcture exist which is requires for the program.
/// If some directory does not exist, it will try to create it.
/// 
/// # Return values
/// 
/// Return with `Result<(), String>. In case of Err, the parameter is the name of directory which had problem.
pub fn check_and_build_dirs() -> Result<(), String> {
    if let Err(_) = create_dir_if_not_exist("/all_timers") {
        return Err(format!("all_timers"));
    }

    if let Err(_) = create_dir_if_not_exist(("active_timers")) {
        return Err(format!("active_timers"));
    }

    if let Err(_) = create_dir_if_not_exist(("logs")) {
        return Err(format!("active_timers"));
    }

    return Ok(());
}

/// If specified directory does not exist, then try to create it.
fn create_dir_if_not_exist(path: &str) -> Result<(), ()> {
    if !std::path::Path::new(path).is_dir() {
        if let Err(_) = std::fs::create_dir_all(path) {
            return Err(());
        }
    }
    return Ok(());
}